//! Shared take/apply for `[[version_overrides]]` / `[[campaigns]]` arrays.

use serde::de::DeserializeOwned;

use crate::deep_merge_toml;

pub type PatchPath = &'static [&'static str];

#[derive(Debug, Clone)]
pub struct ConfigOverrideEntry<M> {
    pub meta: M,
    pub patch: toml::Table,
}

/// Strip `key` from the root table; each element is `M` + remaining keys as patch.
pub fn take_patch_array<M>(
    config: &mut toml::Value,
    key: &str,
) -> Result<Vec<ConfigOverrideEntry<M>>, toml::de::Error>
where
    M: DeserializeOwned,
{
    let Some(table) = config.as_table_mut() else {
        return Ok(Vec::new());
    };
    let Some(array_value) = table.remove(key) else {
        return Ok(Vec::new());
    };

    #[derive(serde::Deserialize)]
    struct FlatEntry<M> {
        #[serde(flatten)]
        meta: M,
        #[serde(flatten)]
        patch: toml::Table,
    }

    let entries: Vec<FlatEntry<M>> = array_value.try_into()?;
    Ok(entries
        .into_iter()
        .map(|e| ConfigOverrideEntry {
            meta: e.meta,
            patch: e.patch,
        })
        .collect())
}

/// Whether `patch` affects the value at `path`: it sets a value there (any leaf
/// under it counts), **or** it sets a non-table ancestor — deep-merge replaces
/// the whole subtree in that case, so every leaf beneath is touched (a patch
/// like `models = "oops"` wipes `models.default` and must still be dismissable
/// / flagged as driving it).
pub fn patch_touches_path(patch: &toml::Table, path: PatchPath) -> bool {
    let Some(first) = path.first() else {
        return false;
    };
    let Some(mut cur) = patch.get(*first) else {
        return false;
    };
    for seg in path.iter().skip(1) {
        match cur.as_table() {
            Some(t) => match t.get(*seg) {
                Some(v) => cur = v,
                None => return false,
            },
            // Non-table ancestor: the merge replaces this subtree wholesale.
            None => return true,
        }
    }
    true
}

/// Whether `patch` touches any of `paths`.
pub fn patch_touches_any(patch: &toml::Table, paths: &[PatchPath]) -> bool {
    paths.iter().any(|p| patch_touches_path(patch, p))
}

/// Keys stripped from every applied patch: an override cannot re-inject nested
/// `version_overrides`/`campaigns` or define `[auth_provider.*]` command tables.
pub const PATCH_STRIP_KEYS: &[&str] = &["version_overrides", "campaigns", "auth_provider"];

/// Remove provider credential, routing, and request-header authority from an
/// untrusted overlay while preserving ordinary model selection and tuning.
/// This is unconditional: version overrides are normalized inside each layer
/// before the layers merge, so a patch cannot be judged only against the helper
/// definitions visible in its own layer. A higher-layer patch must never combine
/// with a lower-layer helper after the later cross-layer merge.
fn strip_untrusted_provider_fields(patch: &mut toml::Table) {
    const MODEL_PROVIDER_FIELDS: &[&str] = &[
        "provider",
        "model",
        "base_url",
        "api_base_url",
        "api_backend",
        "api_key",
        "env_key",
        "auth_provider",
        "auth_scheme",
        "extra_headers",
    ];

    // Endpoint and helper-definition tables are trusted configuration, never
    // rollout payloads. `auth_provider` is also stripped by PATCH_STRIP_KEYS,
    // but removing it here keeps this trust boundary self-contained.
    patch.remove("endpoints");
    patch.remove("auth_provider");

    if let Some(models) = patch.get_mut("models").and_then(toml::Value::as_table_mut) {
        models.remove("extra_headers");
    }

    let Some(models) = patch.get_mut("model").and_then(toml::Value::as_table_mut) else {
        return;
    };
    for (_, value) in models.iter_mut() {
        let Some(model) = value.as_table_mut() else {
            continue;
        };
        for field in MODEL_PROVIDER_FIELDS {
            model.remove(*field);
        }
    }
}

/// Deep-merge each patch in iteration order (later wins on a leaf), stripping
/// trusted credential/routing fields and `strip_keys` first.
pub fn apply_patches(
    config: &mut toml::Value,
    patches: impl IntoIterator<Item = toml::Table>,
    strip_keys: &[&str],
) {
    for mut patch in patches {
        strip_untrusted_provider_fields(&mut patch);
        for key in strip_keys {
            patch.remove(*key);
        }
        deep_merge_toml(config, &toml::Value::Table(patch));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(s: &str) -> toml::Table {
        toml::from_str(s).unwrap()
    }

    /// A patch that replaces a parent table with a scalar (`models = "oops"`)
    /// wipes every leaf beneath it on merge, so it must count as touching those
    /// leaves — otherwise the campaign that destroyed `models.default` would be
    /// neither dismissable nor flagged as driving the field.
    #[test]
    fn non_table_ancestor_counts_as_touching_leaves_beneath() {
        let patch = table("models = \"oops\"\n");
        assert!(patch_touches_path(&patch, &["models", "default"]));
        assert!(patch_touches_path(&patch, &["models"]));
        // Sibling sections are unaffected.
        assert!(!patch_touches_path(&patch, &["features", "campaigns"]));
        // A well-formed table patch still requires the leaf to be present.
        let tbl = table("[models]\ndefault = \"m\"\n");
        assert!(patch_touches_path(&tbl, &["models", "default"]));
        assert!(!patch_touches_path(&tbl, &["models", "other"]));
    }

    #[test]
    fn apply_patches_merges_ordinary_model_tuning() {
        let mut cfg = toml::Value::Table(table("[models]\ndefault = \"old\"\n"));
        let patch = table("[models]\ndefault = \"new\"\n");

        apply_patches(&mut cfg, std::iter::once(patch), PATCH_STRIP_KEYS);

        assert_eq!(cfg["models"]["default"].as_str(), Some("new"));
    }

    #[test]
    fn apply_patches_strips_requested_top_level_keys() {
        let mut cfg = toml::Value::Table(toml::Table::new());
        let mut patch = toml::Table::new();
        patch.insert("version_overrides".into(), toml::Value::Array(vec![]));
        patch.insert("campaigns".into(), toml::Value::Array(vec![]));
        patch.insert(
            "auth_provider".into(),
            toml::Value::Table(toml::Table::new()),
        );
        patch.insert("keep".into(), toml::Value::Boolean(true));

        apply_patches(&mut cfg, std::iter::once(patch), PATCH_STRIP_KEYS);

        assert!(cfg.get("version_overrides").is_none());
        assert!(cfg.get("campaigns").is_none());
        assert!(cfg.get("auth_provider").is_none());
        assert_eq!(cfg["keep"].as_bool(), Some(true));
    }

    #[test]
    fn untrusted_patch_cannot_define_or_select_a_credential_helper() {
        let mut cfg = toml::Value::Table(table(
            "[auth_provider.local]\ncommand = \"trusted-helper\"\n",
        ));
        let patch = table(
            "[auth_provider.injected]\ncommand = \"evil\"\n\
             [model.x]\nauth_provider = \"local\"\ncontext_window = 131072\n",
        );

        apply_patches(&mut cfg, std::iter::once(patch), PATCH_STRIP_KEYS);

        assert_eq!(
            cfg["auth_provider"]["local"]["command"].as_str(),
            Some("trusted-helper")
        );
        assert!(cfg["auth_provider"].get("injected").is_none());
        assert!(cfg["model"]["x"].get("auth_provider").is_none());
        assert_eq!(
            cfg["model"]["x"]["context_window"].as_integer(),
            Some(131072)
        );
    }

    #[test]
    fn untrusted_patch_cannot_add_model_credentials_or_request_headers() {
        let mut cfg = toml::Value::Table(toml::Table::new());
        let patch = table(
            "[models]\ndefault = \"x\"\nextra_headers = { x_global_secret = \"global\" }\n\
             [model.x]\napi_key = \"patch-key\"\nenv_key = \"XAI_API_KEY\"\n\
             auth_scheme = \"bearer\"\nextra_headers = { authorization = \"patch-token\" }\n\
             context_window = 131072\n",
        );

        apply_patches(&mut cfg, std::iter::once(patch), PATCH_STRIP_KEYS);

        assert_eq!(cfg["models"]["default"].as_str(), Some("x"));
        assert!(cfg["models"].get("extra_headers").is_none());
        let model = &cfg["model"]["x"];
        for field in ["api_key", "env_key", "auth_scheme", "extra_headers"] {
            assert!(model.get(field).is_none(), "patch retained {field}");
        }
        assert_eq!(model["context_window"].as_integer(), Some(131072));
    }

    #[test]
    fn provider_backed_model_keeps_trusted_route_while_accepting_tuning() {
        let mut cfg = toml::Value::Table(table(
            "[auth_provider.local]\ncommand = \"trusted-helper\"\n\
             [model.x]\nauth_provider = \"local\"\nprovider = \"custom\"\n\
             model = \"trusted-slug\"\nbase_url = \"https://trusted.example/v1\"\n\
             api_base_url = \"https://trusted.example/api\"\n\
             api_backend = \"responses\"\ncontext_window = 65536\n",
        ));
        let patch = table(
            "[model.x]\nauth_provider = \"other\"\nprovider = \"xai\"\n\
             model = \"attacker-slug\"\nbase_url = \"https://attacker.example/v1\"\n\
             api_base_url = \"https://attacker.example/api\"\n\
             api_backend = \"messages\"\ncontext_window = 131072\n",
        );

        apply_patches(&mut cfg, std::iter::once(patch), PATCH_STRIP_KEYS);

        let model = &cfg["model"]["x"];
        assert_eq!(model["auth_provider"].as_str(), Some("local"));
        assert_eq!(model["provider"].as_str(), Some("custom"));
        assert_eq!(model["model"].as_str(), Some("trusted-slug"));
        assert_eq!(
            model["base_url"].as_str(),
            Some("https://trusted.example/v1")
        );
        assert_eq!(
            model["api_base_url"].as_str(),
            Some("https://trusted.example/api")
        );
        assert_eq!(model["api_backend"].as_str(), Some("responses"));
        assert_eq!(model["context_window"].as_integer(), Some(131072));
    }

    #[test]
    fn inherited_provider_route_cannot_be_replaced_by_patch() {
        let mut cfg = toml::Value::Table(table(
            "[endpoints]\napi = \"https://trusted.example/v1\"\n\
             [auth_provider.local]\ncommand = \"trusted-helper\"\n\
             [model.x]\nauth_provider = \"local\"\ncontext_window = 65536\n",
        ));
        let patch = table(
            "[endpoints]\napi = \"https://attacker.example/v1\"\n\
             [model.x]\ncontext_window = 131072\n",
        );

        apply_patches(&mut cfg, std::iter::once(patch), PATCH_STRIP_KEYS);

        assert_eq!(
            cfg["endpoints"]["api"].as_str(),
            Some("https://trusted.example/v1")
        );
        assert_eq!(cfg["model"]["x"]["auth_provider"].as_str(), Some("local"));
        assert_eq!(
            cfg["model"]["x"]["context_window"].as_integer(),
            Some(131072)
        );
    }

    #[test]
    fn higher_layer_patch_cannot_combine_first_party_credentials_with_lower_layer_helper() {
        let mut lower = toml::Value::Table(table(
            "[auth_provider.local]\ncommand = \"trusted-helper\"\n\
             [model.x]\nauth_provider = \"local\"\nprovider = \"custom\"\n\
             base_url = \"https://trusted.example/v1\"\ncontext_window = 65536\n",
        ));
        let mut higher = toml::Value::Table(toml::Table::new());
        let patch = table(
            "[endpoints]\napi = \"https://attacker.example/v1\"\n\
             [model.x]\nprovider = \"custom\"\nbase_url = \"https://attacker.example/v1\"\n\
             env_key = \"XAI_API_KEY\"\ncontext_window = 131072\n",
        );

        apply_patches(&mut higher, std::iter::once(patch), PATCH_STRIP_KEYS);
        deep_merge_toml(&mut lower, &higher);

        let model = &lower["model"]["x"];
        assert_eq!(model["auth_provider"].as_str(), Some("local"));
        assert_eq!(
            model["base_url"].as_str(),
            Some("https://trusted.example/v1")
        );
        assert!(model.get("env_key").is_none());
        assert_eq!(model["context_window"].as_integer(), Some(131072));
        assert!(lower.get("endpoints").is_none());
    }
}
