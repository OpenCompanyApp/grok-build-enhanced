//! Session/model-scoped credential bridge for canonical xAI speech.
//!
//! A pager connection receives a redacted snapshot of positively identified
//! xAI models. Before every recording, the active target model is bound here;
//! bearer resolution then uses only that model's own key, an eligible xAI auth
//! session, or the ambient xAI API key. No model key is published through the
//! shell `AuthManager` or any process-global mutable slot.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use parking_lot::RwLock;
use xai_grok_shell::agent::models::{XaiSpeechModelCatalog, XaiSpeechModelCredential};
use xai_grok_shell::auth::{AuthManager, AuthMode, GrokAuth};
use xai_grok_voice::{SharedVoiceAuth, VoiceAuthProvider};

/// Model-scoped voice auth shared by the lazy pipeline.
struct ScopedXaiVoiceAuth {
    auth_manager: Arc<AuthManager>,
    models: XaiSpeechModelCatalog,
    active: RwLock<Option<XaiSpeechModelCredential>>,
}

impl std::fmt::Debug for ScopedXaiVoiceAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopedXaiVoiceAuth")
            .field("models", &self.models)
            .field("has_active_xai_model", &self.active.read().is_some())
            .finish()
    }
}

impl VoiceAuthProvider for ScopedXaiVoiceAuth {
    fn bind_model(&self, model_id: Option<&str>) {
        *self.active.write() = model_id.and_then(|id| self.models.credential_for(id));
    }

    fn bearer(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        let credential = self.active.read().clone();
        let auth_manager = self.auth_manager.clone();
        Box::pin(async move {
            let credential = credential?;
            if let Some(key) = credential.model_api_key() {
                return nonempty(key.to_owned());
            }

            if auth_manager
                .current_or_expired()
                .as_ref()
                .is_some_and(eligible_xai_auth)
                && let Ok(key) = auth_manager.get_valid_token().await
                && auth_manager
                    .current_or_expired()
                    .as_ref()
                    .is_some_and(eligible_xai_auth)
            {
                return nonempty(key);
            }

            xai_grok_shell::agent::auth_method::read_xai_api_key_env()
                .ok()
                .and_then(nonempty)
        })
    }
}

fn eligible_xai_auth(auth: &GrokAuth) -> bool {
    auth.is_xai_auth() || matches!(auth.auth_mode, AuthMode::ApiKey | AuthMode::WebLogin)
}

fn nonempty(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

/// Build one connection-local voice bearer provider.
pub fn build_voice_auth(
    auth_manager: Arc<AuthManager>,
    models: XaiSpeechModelCatalog,
) -> SharedVoiceAuth {
    Arc::new(ScopedXaiVoiceAuth {
        auth_manager,
        models,
        active: RwLock::new(None),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use xai_grok_sampling_types::ProviderId;
    use xai_grok_shell::agent::config::{EndpointsConfig, ModelEntry};

    fn model(provider: ProviderId, key: Option<&str>) -> ModelEntry {
        let mut entry = ModelEntry::fallback("model", &EndpointsConfig::default());
        entry.info.provider = provider;
        entry.info.base_url = match provider {
            ProviderId::Xai => xai_grok_sampling_types::XAI_API_BASE_URL.to_owned(),
            _ => "https://custom.example/v1".to_owned(),
        };
        entry.api_key = key.map(str::to_owned);
        entry
    }

    fn catalog(entries: &[(&str, ProviderId, Option<&str>)]) -> XaiSpeechModelCatalog {
        let models = entries
            .iter()
            .map(|(id, provider, key)| ((*id).to_owned(), model(*provider, *key)))
            .collect::<IndexMap<_, _>>();
        XaiSpeechModelCatalog::from_models(&models)
    }

    fn manager(home: &std::path::Path) -> Arc<AuthManager> {
        Arc::new(AuthManager::new(home, Default::default()))
    }

    async fn resolved(auth: &SharedVoiceAuth, model: Option<&str>) -> Option<String> {
        auth.bind_model(model);
        auth.bearer().await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn missing_key_fails_closed_for_an_eligible_xai_model() {
        let _xai = xai_grok_test_support::EnvGuard::unset("XAI_API_KEY");
        let _legacy = xai_grok_test_support::EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let home = tempfile::tempdir().unwrap();
        let auth = build_voice_auth(
            manager(home.path()),
            catalog(&[("xai", ProviderId::Xai, None)]),
        );

        assert_eq!(resolved(&auth, Some("xai")).await, None);
        assert_eq!(resolved(&auth, None).await, None);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn mixed_provider_models_never_fall_through_to_xai_credentials() {
        let _xai = xai_grok_test_support::EnvGuard::set("XAI_API_KEY", "ambient-xai-key");
        let _legacy = xai_grok_test_support::EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let home = tempfile::tempdir().unwrap();
        let auth = build_voice_auth(
            manager(home.path()),
            catalog(&[
                ("xai", ProviderId::Xai, None),
                ("codex", ProviderId::OpenAiCodex, Some("codex-key")),
                ("kimi", ProviderId::KimiCode, Some("kimi-key")),
                ("zai", ProviderId::ZaiCodingPlan, Some("zai-key")),
                ("custom", ProviderId::Custom, Some("custom-key")),
            ]),
        );

        assert_eq!(
            resolved(&auth, Some("xai")).await.as_deref(),
            Some("ambient-xai-key")
        );
        for model in ["codex", "kimi", "zai", "custom", "unknown"] {
            assert_eq!(resolved(&auth, Some(model)).await, None, "model={model}");
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn model_switching_rebinds_each_connection_without_stale_fallback() {
        let _xai = xai_grok_test_support::EnvGuard::unset("XAI_API_KEY");
        let _legacy = xai_grok_test_support::EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let home = tempfile::tempdir().unwrap();
        let auth = build_voice_auth(
            manager(home.path()),
            catalog(&[
                ("xai-a", ProviderId::Xai, Some("model-a-key")),
                ("custom", ProviderId::Custom, Some("custom-key")),
                ("xai-b", ProviderId::Xai, Some("model-b-key")),
            ]),
        );

        assert_eq!(
            resolved(&auth, Some("xai-a")).await.as_deref(),
            Some("model-a-key")
        );
        assert_eq!(resolved(&auth, Some("custom")).await, None);
        assert_eq!(
            resolved(&auth, Some("xai-b")).await.as_deref(),
            Some("model-b-key")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn concurrent_connections_keep_model_keys_isolated() {
        let _xai = xai_grok_test_support::EnvGuard::unset("XAI_API_KEY");
        let _legacy = xai_grok_test_support::EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let home_a = tempfile::tempdir().unwrap();
        let home_b = tempfile::tempdir().unwrap();
        let auth_a = build_voice_auth(
            manager(home_a.path()),
            catalog(&[("same-model", ProviderId::Xai, Some("session-a-key"))]),
        );
        let auth_b = build_voice_auth(
            manager(home_b.path()),
            catalog(&[("same-model", ProviderId::Xai, Some("session-b-key"))]),
        );

        let (a, b) = tokio::join!(
            resolved(&auth_a, Some("same-model")),
            resolved(&auth_b, Some("same-model"))
        );
        assert_eq!(a.as_deref(), Some("session-a-key"));
        assert_eq!(b.as_deref(), Some("session-b-key"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn logout_removes_session_fallback_without_retaining_a_snapshot() {
        let _xai = xai_grok_test_support::EnvGuard::unset("XAI_API_KEY");
        let _legacy = xai_grok_test_support::EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let home = tempfile::tempdir().unwrap();
        xai_grok_shell::auth::store_api_key(home.path(), "stored-xai-key").unwrap();
        let manager = manager(home.path());
        let auth = build_voice_auth(manager.clone(), catalog(&[("xai", ProviderId::Xai, None)]));

        assert_eq!(
            resolved(&auth, Some("xai")).await.as_deref(),
            Some("stored-xai-key")
        );
        xai_grok_shell::auth::perform_logout(&manager, None).unwrap();
        assert_eq!(resolved(&auth, Some("xai")).await, None);
    }

    #[test]
    fn debug_output_redacts_model_keys() {
        let home = tempfile::tempdir().unwrap();
        let auth = ScopedXaiVoiceAuth {
            auth_manager: manager(home.path()),
            models: catalog(&[("xai", ProviderId::Xai, Some("sentinel-model-secret"))]),
            active: RwLock::new(None),
        };
        auth.bind_model(Some("xai"));

        let rendered = format!("{auth:?}");
        assert!(!rendered.contains("sentinel-model-secret"));
        assert!(rendered.contains("has_active_xai_model"));
    }
}
