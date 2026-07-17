use std::fmt;
use std::sync::Arc;

use super::super::{CodexAuthManager, OPENAI_CODEX_PROVIDER_ID};

/// Synchronous sampler hook backed by a disk-verified common auth snapshot.
/// [`CodexAuthManager::ensure_fresh`] is still called before constructing or
/// rebuilding a sampling client; per-request verification additionally makes
/// cross-process logout and account switching fail closed immediately.
#[derive(Clone)]
pub struct CodexSamplerRequestAuth(pub Arc<CodexAuthManager>);

impl fmt::Debug for CodexSamplerRequestAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodexSamplerRequestAuth")
            .field("provider", &OPENAI_CODEX_PROVIDER_ID)
            .finish()
    }
}

impl xai_grok_sampler::RequestAuth for CodexSamplerRequestAuth {
    fn apply(
        &self,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<xai_grok_sampling_types::CredentialBinding, xai_grok_sampler::RequestAuthError>
    {
        let snapshot = self
            .0
            .current_auth_snapshot()
            .map_err(|_| xai_grok_sampler::RequestAuthError::CredentialsUnavailable)?;
        let binding = snapshot.credential_binding().clone();
        let request_headers = snapshot.request_headers();
        let has_fedramp = request_headers
            .contains_key(reqwest::header::HeaderName::from_static("x-openai-fedramp"));
        for (name, value) in request_headers {
            if let Some(name) = name {
                headers.insert(name, value);
            }
        }
        if !has_fedramp {
            headers.remove(reqwest::header::HeaderName::from_static("x-openai-fedramp"));
        }
        Ok(binding)
    }

    fn recover_unauthorized(
        &self,
        rejected: xai_grok_sampling_types::CredentialBinding,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            self.0
                .recover_after_unauthorized(rejected)
                .await
                .unwrap_or(false)
        })
    }
}

pub fn shared_sampler_request_auth(
    manager: Arc<CodexAuthManager>,
) -> xai_grok_sampler::SharedRequestAuth {
    Arc::new(CodexSamplerRequestAuth(manager))
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use xai_grok_sampler::RequestAuth as _;

    use super::*;
    use crate::auth::codex::CodexCredentialStore;
    use crate::auth::codex::credentials::credentials_for_test;

    async fn manager() -> (tempfile::TempDir, Arc<CodexAuthManager>) {
        let directory = tempfile::tempdir().unwrap();
        let store = CodexCredentialStore::from_auth_path(directory.path().join("auth.json"));
        store
            .save(credentials_for_test(
                "sentinel-account-id",
                "sentinel-refresh-token",
                Utc::now() + Duration::hours(1),
            ))
            .await
            .unwrap();
        let manager = Arc::new(CodexAuthManager::from_store(store).unwrap());
        (directory, manager)
    }

    #[tokio::test]
    async fn sampler_applies_exact_sensitive_snapshot_without_xai_headers() {
        let (_directory, manager) = manager().await;
        let expected = manager.current().unwrap().credential_binding();
        let sampler = CodexSamplerRequestAuth(manager);
        let mut headers = reqwest::header::HeaderMap::new();

        let actual = sampler.apply(&mut headers).unwrap();

        assert_eq!(actual, expected);
        for name in [
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderName::from_static("chatgpt-account-id"),
        ] {
            assert!(headers[&name].is_sensitive(), "{name} must be sensitive");
        }
        assert!(headers.get("x-xai-token-auth").is_none());
        assert!(headers.get("x-userid").is_none());
    }

    #[tokio::test]
    async fn sampler_fails_closed_immediately_after_cross_process_logout() {
        let (_directory, manager) = manager().await;
        assert!(manager.store().remove().await.unwrap().is_some());
        let sampler = CodexSamplerRequestAuth(manager);

        assert!(
            sampler
                .apply(&mut reqwest::header::HeaderMap::new())
                .is_err()
        );
    }
}
