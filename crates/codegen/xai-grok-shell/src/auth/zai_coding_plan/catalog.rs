use std::collections::HashSet;
use std::io::Write;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use xai_grok_sampler::AuthScheme;
use xai_grok_sampling_types::{
    ApiBackend, ProviderId, ReasoningEffort, ReasoningEffortOption, ZAI_CODING_PLAN_BASE_URL,
    ZAI_CODING_PLAN_MAX_RESPONSE_BYTES,
};

use super::{ZaiCodingPlanAuthError, ZaiCodingPlanCredentials};
use crate::agent::config::{ModelEntry, ModelInfo};

const CACHE_SCHEMA_VERSION: u32 = 1;
const CACHE_FILE: &str = "zai-coding-plan-models.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZaiCodingPlanModel {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CatalogCache {
    schema_version: u32,
    credential_id: String,
    fetched_at: DateTime<Utc>,
    models: Vec<ZaiCodingPlanModel>,
}

/// Audited Coding Plan models from the public Z.AI model-switching contract.
///
/// Z.AI does not document a Coding Plan `/models` API. Keeping this list
/// local avoids sending a credential to a speculative endpoint; catalog
/// changes are therefore explicit source-review changes.
pub(super) fn default_models() -> Vec<ZaiCodingPlanModel> {
    [
        ("glm-5.3", "GLM-5.3"),
        ("glm-5.3[1m]", "GLM-5.3 (1M)"),
        ("glm-5.3-flash", "GLM-5.3-Flash"),
        ("glm-5.3-flash[1m]", "GLM-5.3-Flash (1M)"),
    ]
    .into_iter()
    .map(|(id, display_name)| ZaiCodingPlanModel {
        id: id.to_owned(),
        display_name: Some(display_name.to_owned()),
    })
    .collect()
}

fn validate_models(models: Vec<ZaiCodingPlanModel>) -> Vec<ZaiCodingPlanModel> {
    let mut seen = HashSet::new();
    models
        .into_iter()
        .filter(|model| {
            !model.id.trim().is_empty()
                && model.id.len() <= 256
                && !model.id.chars().any(char::is_control)
                && !model.id.chars().any(char::is_whitespace)
                && seen.insert(model.id.clone())
                && model.display_name.as_deref().is_none_or(|name| {
                    !name.trim().is_empty()
                        && name.len() <= 512
                        && !name.chars().any(char::is_control)
                })
        })
        .collect()
}

pub(super) fn cache_path(grok_home: &Path) -> PathBuf {
    grok_home.join("cache").join(CACHE_FILE)
}

pub(super) fn save_cache(
    grok_home: &Path,
    credentials: &ZaiCodingPlanCredentials,
    models: &[ZaiCodingPlanModel],
) -> Result<(), ZaiCodingPlanAuthError> {
    let path = cache_path(grok_home);
    let parent = path.parent().ok_or_else(|| {
        ZaiCodingPlanAuthError::Storage(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid Z.AI Coding Plan cache path",
        ))
    })?;
    std::fs::create_dir_all(parent)?;
    let cache = CatalogCache {
        schema_version: CACHE_SCHEMA_VERSION,
        credential_id: credentials.credential_id.clone(),
        fetched_at: Utc::now(),
        models: models.to_vec(),
    };
    let bytes =
        serde_json::to_vec_pretty(&cache).map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    if bytes.len() > ZAI_CODING_PLAN_MAX_RESPONSE_BYTES {
        return Err(ZaiCodingPlanAuthError::InvalidResponse);
    }
    let temporary = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    let result = (|| -> std::io::Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(&bytes)?;
        file.sync_all()
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(ZaiCodingPlanAuthError::Storage(error));
    }
    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(ZaiCodingPlanAuthError::Storage(error));
    }
    Ok(())
}

pub(super) fn remove_cache(grok_home: &Path) -> Result<(), ZaiCodingPlanAuthError> {
    match std::fs::remove_file(cache_path(grok_home)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ZaiCodingPlanAuthError::Storage(error)),
    }
}

pub fn load_cached_model_entries() -> IndexMap<String, ModelEntry> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = super::ZaiCodingPlanCredentialStore::new(&grok_home);
    match store.load() {
        Ok(Some(credentials)) => {
            let cached = load_cached_model_entries_for(&grok_home, &credentials.credential_id);
            if cached.is_empty() {
                map_models(default_models())
            } else {
                cached
            }
        }
        Ok(None) if super::credentials_from_env().is_ok() => map_models(default_models()),
        _ => IndexMap::new(),
    }
}

fn load_cached_model_entries_for(
    grok_home: &Path,
    credential_id: &str,
) -> IndexMap<String, ModelEntry> {
    let path = cache_path(grok_home);
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        return IndexMap::new();
    };
    if !metadata.file_type().is_file() || metadata.len() > ZAI_CODING_PLAN_MAX_RESPONSE_BYTES as u64
    {
        return IndexMap::new();
    }
    let Ok(bytes) = std::fs::read(path) else {
        return IndexMap::new();
    };
    if bytes.len() > ZAI_CODING_PLAN_MAX_RESPONSE_BYTES {
        return IndexMap::new();
    }
    let Ok(cache) = serde_json::from_slice::<CatalogCache>(&bytes) else {
        return IndexMap::new();
    };
    if cache.schema_version != CACHE_SCHEMA_VERSION || cache.credential_id != credential_id {
        return IndexMap::new();
    }
    map_models(validate_models(cache.models))
}

pub fn map_models(models: Vec<ZaiCodingPlanModel>) -> IndexMap<String, ModelEntry> {
    models.into_iter().map(map_model).collect()
}

fn map_model(model: ZaiCodingPlanModel) -> (String, ModelEntry) {
    let catalog_key = format!("zai-coding-plan/{}", model.id);
    let (qualified, context_window, max_completion_tokens, efforts, default_effort) =
        capability_overlay(&model.id);
    let mut info = ModelInfo::fallback(&model.id);
    info.id = Some(catalog_key.clone());
    info.provider = ProviderId::ZaiCodingPlan;
    info.base_url = ZAI_CODING_PLAN_BASE_URL.to_owned();
    info.name = model.display_name.filter(|name| !name.trim().is_empty());
    info.description = Some(if qualified {
        "Z.AI GLM Coding Plan model (Chat Completions)".to_owned()
    } else {
        "Z.AI Coding Plan catalog model (capabilities not yet qualified)".to_owned()
    });
    info.api_backend = ApiBackend::ChatCompletions;
    info.auth_scheme = AuthScheme::Bearer;
    info.context_window = NonZeroU64::new(context_window).unwrap();
    info.max_completion_tokens = max_completion_tokens;
    info.reasoning_effort = default_effort;
    info.supports_reasoning_effort = efforts.len() > 1;
    info.reasoning_efforts = efforts;
    info.supports_image_input = model.id.starts_with("glm-5.3-flash");
    info.supports_backend_search = false;
    info.supported_in_api = qualified;
    info.user_selectable = qualified;
    info.hidden = false;
    (
        catalog_key,
        ModelEntry {
            info,
            api_key: None,
            env_key: None,
            auth_provider: None,
            api_base_url: None,
        },
    )
}

fn capability_overlay(
    id: &str,
) -> (
    bool,
    u64,
    Option<u32>,
    Vec<ReasoningEffortOption>,
    Option<ReasoningEffort>,
) {
    let known = matches!(
        id,
        "glm-5.3" | "glm-5.3[1m]" | "glm-5.3-flash" | "glm-5.3-flash[1m]"
    );
    let context = 1_000_000;
    let max_output = known.then_some(131_072);
    if !known {
        return (false, context, None, Vec::new(), None);
    }
    let levels: &[ReasoningEffort] = &[
        ReasoningEffort::Low,
        ReasoningEffort::High,
        ReasoningEffort::Max,
    ];
    let default = ReasoningEffort::Max;
    let efforts = levels
        .iter()
        .copied()
        .map(|value| ReasoningEffortOption {
            id: value.as_str().to_owned(),
            value,
            label: match value {
                ReasoningEffort::Low => "Low",
                ReasoningEffort::High => "High",
                ReasoningEffort::Max => "Max",
                _ => unreachable!(),
            }
            .to_owned(),
            description: None,
            default: value == default,
        })
        .collect();
    (known, context, max_output, efforts, Some(default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glm_53_is_text_only_and_uses_one_million_context() {
        let entries = map_models(vec![ZaiCodingPlanModel {
            id: "glm-5.3".to_owned(),
            display_name: None,
        }]);
        let entry = &entries["zai-coding-plan/glm-5.3"];
        assert_eq!(entry.info.context_window.get(), 1_000_000);
        assert_eq!(entry.info.max_completion_tokens, Some(131_072));
        assert!(!entry.info.supports_image_input);
        assert!(entry.info.user_selectable);
        assert_eq!(entry.info.reasoning_effort, Some(ReasoningEffort::Max));
        assert_eq!(entry.info.reasoning_efforts.len(), 3);
    }

    #[test]
    fn glm_53_flash_is_multimodal() {
        let entries = map_models(vec![ZaiCodingPlanModel {
            id: "glm-5.3-flash".to_owned(),
            display_name: None,
        }]);
        assert!(
            entries["zai-coding-plan/glm-5.3-flash"]
                .info
                .supports_image_input
        );
    }

    #[test]
    fn authenticated_unknown_model_is_visible_but_not_selectable() {
        let entries = map_models(vec![ZaiCodingPlanModel {
            id: "future-model".to_owned(),
            display_name: None,
        }]);
        let entry = &entries["zai-coding-plan/future-model"];
        assert!(!entry.info.user_selectable);
        assert!(!entry.info.supported_in_api);
    }
}
