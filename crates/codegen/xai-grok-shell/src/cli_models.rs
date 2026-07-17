//! Data APIs for `grok models`. Clients own display.

use agent_client_protocol as acp;
use anyhow::{Context, Result};
use xai_acp_lib::{AcpAgentTx, acp_send};

use crate::agent::config::Config as AgentConfig;
use crate::remote::{CodexCatalogClient, CodexCatalogClientConfig, CodexCatalogModel};

/// Status for the `grok models` banner (display order ≠ sampling priority; see [`AuthStatus::resolve`]).
#[derive(Debug, PartialEq, Eq)]
pub enum AuthStatus {
    ApiKey,
    /// Auth host from `grok_ws_origin` (scheme stripped).
    LoggedIn(String),
    /// Catalog key of the first model with own `api_key`/`env_key`.
    ModelCredentials(String),
    DeploymentKey,
    NotAuthenticated,
}

impl AuthStatus {
    /// Banner status: env key → session → BYOK → deployment → none.
    ///
    /// Differs from sampling (`resolve_credentials`: BYOK → session → env) so a
    /// logged-in user sees the login host. BYOK uses
    /// [`crate::agent::auth_method::should_advertise_xai_api_key`] so
    /// `disable_api_key_auth` is honored.
    pub fn resolve(agent_config: &AgentConfig) -> Self {
        if crate::agent::auth_method::has_xai_api_key_env() {
            return Self::ApiKey;
        }
        if agent_config.create_auth_manager().current().is_some() {
            let origin = &agent_config.grok_com_config.grok_ws_origin;
            let host = origin
                .strip_prefix("https://")
                .or_else(|| origin.strip_prefix("http://"))
                .unwrap_or(origin);
            return Self::LoggedIn(host.to_owned());
        }
        let models = crate::agent::config::resolve_model_list(agent_config, None);
        if crate::agent::auth_method::should_advertise_xai_api_key(
            agent_config.grok_com_config.api_key_auth_disabled(),
            models.values(),
        ) && let Some(name) = models
            .iter()
            .find_map(|(name, entry)| entry.has_own_credentials().then(|| name.clone()))
        {
            return Self::ModelCredentials(name);
        }
        if agent_config.endpoints.deployment_key.is_some() {
            return Self::DeploymentKey;
        }
        Self::NotAuthenticated
    }
}

/// Fetch model state (available models + default) over an ACP channel.
pub async fn list_models(
    acp_tx: &AcpAgentTx,
    client_type: &str,
    client_version: &str,
) -> Result<acp::SessionModelState> {
    let init_resp: acp::InitializeResponse = acp_send(
        acp::InitializeRequest::new(acp::ProtocolVersion::V1)
            .client_capabilities(
                acp::ClientCapabilities::new()
                    .fs(acp::FileSystemCapabilities::new())
                    .terminal(false),
            )
            .meta(
                serde_json::json!({
                    "clientType": client_type,
                    "clientVersion": client_version,
                })
                .as_object()
                .cloned(),
            ),
        acp_tx,
    )
    .await?;

    let model_state = init_resp
        .meta
        .and_then(|m| m.get("modelState").cloned())
        .ok_or_else(|| anyhow::anyhow!("InitializeResponse missing modelState"))?;
    let state: acp::SessionModelState = serde_json::from_value(model_state)
        .map_err(|e| anyhow::anyhow!("Failed to parse modelState: {}", e))?;

    Ok(state)
}

/// Picker-ready model state for the authenticated ChatGPT Codex account.
///
/// IDs are already namespaced as `openai-codex/<slug>` by the catalog client,
/// preventing collisions with xAI or custom models.
#[derive(Clone, Debug, PartialEq)]
pub struct CodexSubscriptionModelState {
    pub default_model_id: Option<String>,
    pub available_models: Vec<CodexCatalogModel>,
}

/// Query the current ChatGPT Codex model catalog with Grok Build's separately
/// scoped Codex credentials.
///
/// This path never creates or consults the xAI `AuthManager`. The catalog
/// client's provider-local recovery hook reloads/refreshes once on a 401.
pub async fn list_codex_subscription_models() -> Result<CodexSubscriptionModelState> {
    let grok_home = crate::util::grok_home::grok_home();
    let auth = crate::auth::codex::CodexAuthManager::new(&grok_home)
        .context("Failed to load ChatGPT Codex credentials")?;
    if auth.current().is_none() {
        anyhow::bail!(
            "You are not logged in to ChatGPT Codex. Run `grok login --provider openai-codex`."
        );
    }

    let cache_dir = grok_home.join("cache").join("openai-codex");
    let config = CodexCatalogClientConfig::new(cache_dir)
        .context("Failed to configure ChatGPT Codex model discovery")?;
    let client = CodexCatalogClient::new(config)
        .context("Failed to configure ChatGPT Codex model discovery")?;
    let fetched = client
        .fetch(&auth)
        .await
        .context("Failed to fetch ChatGPT Codex models")?;

    Ok(codex_subscription_model_state(fetched.catalog.models))
}

fn codex_subscription_model_state(
    mut models: Vec<CodexCatalogModel>,
) -> CodexSubscriptionModelState {
    // Match the current open-source Codex picker contract: ascending priority,
    // and only entries explicitly marked for listing. Hidden entries can still
    // be addressed by an explicit model ID in the runtime integration.
    models.sort_by_key(|model| model.priority);
    models.retain(CodexCatalogModel::visible_in_picker);
    let default_model_id = models.first().map(|model| model.id.clone());
    CodexSubscriptionModelState {
        default_model_id,
        available_models: models,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::auth_method::{LEGACY_XAI_API_KEY_ENV_VAR, XAI_API_KEY_ENV_VAR};
    use crate::agent::config::Config;
    use crate::auth::{AuthMode, GrokAuth};
    use serial_test::serial;
    use xai_grok_test_support::EnvGuard;

    /// Isolate process-global auth sources that `AuthStatus::resolve` consults.
    ///
    /// Uses `GROK_AUTH_PATH` (not `GROK_HOME`) so a OnceLock-cached real home
    /// with `auth.json` cannot leak into these tests.
    fn isolate_auth_sources() -> (tempfile::TempDir, [EnvGuard; 7]) {
        let dir = tempfile::tempdir().unwrap();
        let auth_path = dir.path().join("no-auth.json");
        let guards = [
            EnvGuard::unset(XAI_API_KEY_ENV_VAR),
            EnvGuard::unset(LEGACY_XAI_API_KEY_ENV_VAR),
            EnvGuard::unset("GROK_AUTH"),
            EnvGuard::set("GROK_AUTH_PATH", auth_path.to_str().unwrap()),
            EnvGuard::unset("GROK_DEPLOYMENT_KEY"),
            EnvGuard::unset("GROK_WS_ORIGIN"),
            EnvGuard::unset("GROK_DISABLE_API_KEY_AUTH"),
        ];
        (dir, guards)
    }

    fn byok_and_deployment_toml(model_id: &str) -> String {
        format!(
            r#"
            [endpoints]
            deployment_key = "deploy-key"

            [model."{model_id}"]
            model = "{model_id}"
            api_key = "sk-byok"
            "#
        )
    }

    #[test]
    fn codex_models_are_namespaced_sorted_and_picker_visible() {
        let visible_later = CodexCatalogModel {
            id: "openai-codex/later".to_owned(),
            slug: "later".to_owned(),
            visibility: Some("list".to_owned()),
            priority: 20,
            ..Default::default()
        };
        let hidden = CodexCatalogModel {
            id: "openai-codex/hidden".to_owned(),
            slug: "hidden".to_owned(),
            visibility: Some("hide".to_owned()),
            priority: 0,
            ..Default::default()
        };
        let visible_default = CodexCatalogModel {
            id: "openai-codex/default".to_owned(),
            slug: "default".to_owned(),
            visibility: Some("list".to_owned()),
            priority: 10,
            ..Default::default()
        };

        let state = codex_subscription_model_state(vec![visible_later, hidden, visible_default]);
        assert_eq!(
            state.default_model_id.as_deref(),
            Some("openai-codex/default")
        );
        assert_eq!(
            state
                .available_models
                .iter()
                .map(|model| model.id.as_str())
                .collect::<Vec<_>>(),
            vec!["openai-codex/default", "openai-codex/later"]
        );
    }

    fn config_from_toml(toml_src: &str) -> Config {
        let toml: toml::Value = toml::from_str(toml_src).unwrap();
        Config::new_from_toml_cfg(&toml).expect("config should parse")
    }

    #[test]
    #[serial]
    fn resolve_api_key_env() {
        let (_dir, _g) = isolate_auth_sources();
        let _key = EnvGuard::set(XAI_API_KEY_ENV_VAR, "xai-test-key");
        assert_eq!(AuthStatus::resolve(&Config::default()), AuthStatus::ApiKey);
    }

    #[test]
    #[serial]
    fn resolve_legacy_api_key_env() {
        let (_dir, _g) = isolate_auth_sources();
        let _key = EnvGuard::set(LEGACY_XAI_API_KEY_ENV_VAR, "legacy-key");
        assert_eq!(AuthStatus::resolve(&Config::default()), AuthStatus::ApiKey);
    }

    #[test]
    #[serial]
    fn resolve_oauth_session() {
        let (_dir, _g) = isolate_auth_sources();
        let token = GrokAuth {
            key: "session-token".into(),
            auth_mode: AuthMode::WebLogin,
            ..GrokAuth::test_default()
        };
        let json = serde_json::to_string(&token).unwrap();
        let _auth = EnvGuard::set("GROK_AUTH", &json);

        assert_eq!(
            AuthStatus::resolve(&Config::default()),
            AuthStatus::LoggedIn("grok.com".to_owned())
        );
    }

    #[test]
    #[serial]
    fn resolve_model_api_key_byok() {
        let (_dir, _g) = isolate_auth_sources();
        let dm = crate::models::default_model();
        let cfg = config_from_toml(&format!(
            r#"
            [model."{dm}"]
            model = "{dm}"
            api_key = "sk-byok-inline"
            "#
        ));
        assert_eq!(
            AuthStatus::resolve(&cfg),
            AuthStatus::ModelCredentials(dm.to_owned())
        );
    }

    #[test]
    #[serial]
    fn resolve_model_env_key_byok() {
        let (_dir, _g) = isolate_auth_sources();
        const TEST_ENV: &str = "TEST_AUTH_STATUS_BYOK_ENV_KEY";
        let dm = crate::models::default_model();
        let cfg = config_from_toml(&format!(
            r#"
            [model."{dm}"]
            model = "{dm}"
            env_key = "{TEST_ENV}"
            "#
        ));

        {
            let _unset = EnvGuard::unset(TEST_ENV);
            assert_eq!(AuthStatus::resolve(&cfg), AuthStatus::NotAuthenticated);
        }
        {
            let _set = EnvGuard::set(TEST_ENV, "secret-token");
            assert_eq!(
                AuthStatus::resolve(&cfg),
                AuthStatus::ModelCredentials(dm.to_owned())
            );
        }
    }

    #[test]
    #[serial]
    fn resolve_deployment_key() {
        let (_dir, _g) = isolate_auth_sources();
        let mut cfg = Config::default();
        cfg.endpoints.deployment_key = Some("deploy-key".into());
        assert_eq!(AuthStatus::resolve(&cfg), AuthStatus::DeploymentKey);
    }

    #[test]
    #[serial]
    fn resolve_not_authenticated() {
        let (_dir, _g) = isolate_auth_sources();
        assert_eq!(
            AuthStatus::resolve(&Config::default()),
            AuthStatus::NotAuthenticated
        );
    }

    #[test]
    #[serial]
    fn resolve_priority_api_key_over_byok_and_deployment() {
        let (_dir, _g) = isolate_auth_sources();
        let _key = EnvGuard::set(XAI_API_KEY_ENV_VAR, "xai-test-key");
        let dm = crate::models::default_model();
        let cfg = config_from_toml(&byok_and_deployment_toml(dm));
        assert_eq!(AuthStatus::resolve(&cfg), AuthStatus::ApiKey);
    }

    #[test]
    #[serial]
    fn resolve_priority_session_over_byok_and_deployment() {
        let (_dir, _g) = isolate_auth_sources();
        let token = GrokAuth {
            key: "session-token".into(),
            auth_mode: AuthMode::WebLogin,
            ..GrokAuth::test_default()
        };
        let json = serde_json::to_string(&token).unwrap();
        let _auth = EnvGuard::set("GROK_AUTH", &json);

        let dm = crate::models::default_model();
        let cfg = config_from_toml(&byok_and_deployment_toml(dm));
        assert_eq!(
            AuthStatus::resolve(&cfg),
            AuthStatus::LoggedIn("grok.com".to_owned())
        );
    }

    #[test]
    #[serial]
    fn resolve_priority_byok_over_deployment() {
        let (_dir, _g) = isolate_auth_sources();
        let dm = crate::models::default_model();
        let cfg = config_from_toml(&byok_and_deployment_toml(dm));
        assert_eq!(
            AuthStatus::resolve(&cfg),
            AuthStatus::ModelCredentials(dm.to_owned())
        );
    }

    #[test]
    #[serial]
    fn resolve_disable_api_key_auth_suppresses_byok_banner() {
        let (_dir, _g) = isolate_auth_sources();
        let dm = crate::models::default_model();
        let cfg = config_from_toml(&format!(
            r#"
            [grok_com_config]
            disable_api_key_auth = true

            [model."{dm}"]
            model = "{dm}"
            api_key = "sk-byok"
            "#
        ));
        assert_eq!(AuthStatus::resolve(&cfg), AuthStatus::NotAuthenticated);
    }

    #[test]
    #[serial]
    fn resolve_disable_api_key_auth_falls_through_to_deployment() {
        let (_dir, _g) = isolate_auth_sources();
        let dm = crate::models::default_model();
        let cfg = config_from_toml(&format!(
            r#"
            [grok_com_config]
            disable_api_key_auth = true

            [endpoints]
            deployment_key = "deploy-key"

            [model."{dm}"]
            model = "{dm}"
            api_key = "sk-byok"
            "#
        ));
        assert_eq!(AuthStatus::resolve(&cfg), AuthStatus::DeploymentKey);
    }

    #[test]
    #[serial]
    fn resolve_model_credentials_uses_first_catalog_key() {
        let (_dir, _g) = isolate_auth_sources();
        let cfg = config_from_toml(
            r#"
            [model."my-openai"]
            model = "gpt-4o"
            api_key = "sk-first"

            [model."my-anthropic"]
            model = "claude"
            api_key = "sk-second"
            "#,
        );
        assert_eq!(
            AuthStatus::resolve(&cfg),
            AuthStatus::ModelCredentials("my-openai".to_owned())
        );
    }
}
