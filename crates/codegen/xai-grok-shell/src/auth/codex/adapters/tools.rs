use std::fmt;
use std::sync::Arc;

use super::super::{CodexAuthError, CodexAuthManager, CodexAuthSnapshot, OPENAI_CODEX_PROVIDER_ID};

/// Adapter used by Grok's existing tool registry. It converts the common
/// provider snapshot into the tools-layer compatibility DTO and routes a 401
/// exclusively through the Codex manager, never through xAI `AuthManager`.
#[derive(Clone)]
pub struct CodexApiKeyProvider(pub Arc<CodexAuthManager>);

impl fmt::Debug for CodexApiKeyProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodexApiKeyProvider")
            .field("provider", &OPENAI_CODEX_PROVIDER_ID)
            .finish()
    }
}

impl CodexAuthManager {
    /// Compatibility entry point for HTTP-backed tools.
    ///
    /// New shell-owned HTTP clients consume [`CodexAuthSnapshot`] directly;
    /// only this adapter creates the tools-layer DTO.
    pub async fn request_auth(&self) -> Result<xai_grok_tools::types::RequestAuth, CodexAuthError> {
        tools_request_auth(self.auth_snapshot().await?)
    }
}

impl xai_grok_tools::types::ApiKeyProvider for CodexApiKeyProvider {
    fn current_api_key(&self) -> Option<String> {
        None
    }

    fn current_api_key_async(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<String>> + Send + '_>> {
        Box::pin(std::future::ready(None))
    }

    fn current_request_auth_async(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Option<xai_grok_tools::types::RequestAuth>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move { self.0.request_auth().await.ok() })
    }

    fn recover_unauthorized_async(
        &self,
        rejected: Option<xai_grok_tools::types::RequestCredentialSnapshot>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            let Some(rejected) = rejected else {
                return false;
            };
            let mut binding = xai_grok_sampling_types::CredentialBinding::openai_codex(Some(
                rejected.opaque_id().to_owned(),
            ));
            binding.generation = rejected.generation();
            self.0
                .recover_after_unauthorized(binding)
                .await
                .unwrap_or(false)
        })
    }
}

pub fn shared_api_key_provider(
    manager: Arc<CodexAuthManager>,
) -> xai_grok_tools::types::SharedApiKeyProvider {
    Arc::new(CodexApiKeyProvider(manager))
}

fn tools_request_auth(
    snapshot: CodexAuthSnapshot,
) -> Result<xai_grok_tools::types::RequestAuth, CodexAuthError> {
    let binding = snapshot.credential_binding();
    let record_id = binding
        .record_id
        .as_deref()
        .ok_or(CodexAuthError::AccountChanged)?;
    let credential_snapshot =
        xai_grok_tools::types::RequestCredentialSnapshot::new(record_id, binding.generation);
    let headers = snapshot
        .request_headers()
        .iter()
        .map(|(name, value)| {
            let value = value.to_str().map_err(|_| {
                CodexAuthError::InvalidTokenResponse(
                    "credential could not form request authentication",
                )
            })?;
            let name =
                canonical_tool_header_name(name).ok_or(CodexAuthError::InvalidTokenResponse(
                    "credential could not form request authentication",
                ))?;
            Ok((name.to_owned(), value.to_owned()))
        })
        .collect::<Result<Vec<_>, CodexAuthError>>()?;

    Ok(xai_grok_tools::types::RequestAuth::for_provider_snapshot(
        OPENAI_CODEX_PROVIDER_ID,
        credential_snapshot,
        headers,
    ))
}

fn canonical_tool_header_name(name: &reqwest::header::HeaderName) -> Option<&'static str> {
    if name == reqwest::header::AUTHORIZATION {
        Some("Authorization")
    } else if name == reqwest::header::HeaderName::from_static("chatgpt-account-id") {
        Some("ChatGPT-Account-ID")
    } else if name == reqwest::header::HeaderName::from_static("x-openai-fedramp") {
        Some("X-OpenAI-Fedramp")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use xai_grok_tools::types::ApiKeyProvider as _;

    use super::*;
    use crate::auth::codex::CodexCredentialStore;
    use crate::auth::codex::credentials::credentials_for_test;

    async fn manager() -> (tempfile::TempDir, Arc<CodexAuthManager>) {
        let directory = tempfile::tempdir().unwrap();
        let store = CodexCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let mut credentials = credentials_for_test(
            "sentinel-account-id",
            "sentinel-refresh-token",
            Utc::now() + Duration::hours(1),
        );
        credentials.is_fedramp = true;
        store.save(credentials).await.unwrap();
        let manager = Arc::new(CodexAuthManager::from_store(store).unwrap());
        (directory, manager)
    }

    #[tokio::test]
    async fn tools_adapter_is_the_only_string_dto_boundary_and_remains_redacted() {
        let (_directory, manager) = manager().await;
        let provider = CodexApiKeyProvider(manager.clone());

        assert!(provider.current_api_key().is_none());
        assert!(provider.current_api_key_async().await.is_none());
        let auth = provider.current_request_auth_async().await.unwrap();
        assert_eq!(auth.provider(), Some(OPENAI_CODEX_PROVIDER_ID));
        assert_eq!(
            auth.credential_snapshot().unwrap().generation(),
            manager.current().unwrap().revision
        );
        let names = auth.headers().map(|(name, _)| name).collect::<Vec<_>>();
        assert!(names.contains(&"Authorization"));
        assert!(names.contains(&"ChatGPT-Account-ID"));
        assert!(names.contains(&"X-OpenAI-Fedramp"));
        assert!(!names.iter().any(|name| name.starts_with("X-XAI-")));

        let rendered = format!("{auth:?}");
        assert!(!rendered.contains("sentinel-account-id"));
        assert!(!rendered.contains("sentinel-refresh-token"));
        assert!(!rendered.contains("Bearer"));
    }

    #[tokio::test]
    async fn tools_adapter_fails_closed_after_account_switch() {
        let (_directory, manager) = manager().await;
        let switched = credentials_for_test(
            "different-account-id",
            "different-refresh-token",
            Utc::now() + Duration::hours(1),
        );
        manager.store().save(switched).await.unwrap();
        let provider = CodexApiKeyProvider(manager);

        assert!(provider.current_request_auth_async().await.is_none());
    }
}
