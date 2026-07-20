//! Session/model-scoped credential bridge for canonical xAI speech.
//!
//! A pager connection receives a redacted snapshot of positively identified
//! xAI models. Every recording passes its target model directly; bearer
//! resolution then uses only that model's own key, an eligible xAI auth
//! session, or the ambient xAI API key. No model key is published through the
//! shell `AuthManager` or any process-global mutable slot.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use xai_grok_shell::agent::models::XaiSpeechModelCatalog;
use xai_grok_shell::auth::{AuthManager, xai_voice_bearer};
use xai_grok_voice::{SharedVoiceAuth, VoiceAuthProvider};

/// Model-scoped voice auth shared by the lazy pipeline.
struct ScopedXaiVoiceAuth {
    auth_manager: Arc<AuthManager>,
    models: XaiSpeechModelCatalog,
}

impl std::fmt::Debug for ScopedXaiVoiceAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopedXaiVoiceAuth")
            .field("models", &self.models)
            .finish()
    }
}

impl VoiceAuthProvider for ScopedXaiVoiceAuth {
    fn bearer(
        &self,
        model_id: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        let credential = model_id.and_then(|id| self.models.credential_for(id));
        let auth_manager = self.auth_manager.clone();
        Box::pin(async move {
            let credential = credential?;
            if let Some(key) = credential.model_api_key() {
                return nonempty(key.to_owned());
            }

            if let Some(key) = xai_voice_bearer(&auth_manager).await {
                return nonempty(key);
            }

            xai_grok_shell::agent::auth_method::read_xai_api_key_env()
                .ok()
                .and_then(nonempty)
        })
    }
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
        // Keep every fixture on the trusted xAI route so mixed-provider tests
        // prove provider identity, rather than route rejection, is the gate.
        entry.info.base_url = xai_grok_sampling_types::XAI_API_BASE_URL.to_owned();
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
        auth.bearer(model).await
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
    async fn model_switching_is_command_scoped_without_stale_fallback() {
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
    async fn concurrent_model_lookups_are_command_scoped() {
        let _xai = xai_grok_test_support::EnvGuard::unset("XAI_API_KEY");
        let _legacy = xai_grok_test_support::EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let home = tempfile::tempdir().unwrap();
        let auth = build_voice_auth(
            manager(home.path()),
            catalog(&[
                ("xai-a", ProviderId::Xai, Some("model-a-key")),
                ("xai-b", ProviderId::Xai, Some("model-b-key")),
            ]),
        );

        let (a, b) = tokio::join!(auth.bearer(Some("xai-a")), auth.bearer(Some("xai-b")));
        assert_eq!(a.as_deref(), Some("model-a-key"));
        assert_eq!(b.as_deref(), Some("model-b-key"));
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
        };

        let rendered = format!("{auth:?}");
        assert!(!rendered.contains("sentinel-model-secret"));
        assert!(rendered.contains("eligible_models"));
    }
}
