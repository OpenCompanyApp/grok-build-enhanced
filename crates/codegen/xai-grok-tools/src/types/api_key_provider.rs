use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Key in `ToolError::details` naming the provider that owns auth recovery
/// for the failed request. Hosts must not route such an error through a
/// different provider's credential manager.
pub const AUTH_RECOVERY_PROVIDER_DETAILS_KEY: &str = "auth_recovery_provider";

/// Key in `ToolError::details` indicating that the provider-owned recovery
/// budget has already been consumed. The error must surface as-is instead of
/// being replayed through a global or unrelated auth manager.
pub const AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY: &str = "auth_recovery_exhausted";

/// Opaque identity and monotonic generation of the credential snapshot that
/// signed one outbound request.
///
/// Provider-owned 401 recovery must use this exact receipt rather than reading
/// whatever credential happens to be current when the response arrives. The
/// opaque ID prevents a late response from an old account being replayed under
/// a newly selected account; formatting deliberately never exposes it.
#[derive(Clone, PartialEq, Eq)]
pub struct RequestCredentialSnapshot {
    opaque_id: String,
    generation: u64,
}

impl RequestCredentialSnapshot {
    pub fn new(opaque_id: impl Into<String>, generation: u64) -> Self {
        Self {
            opaque_id: opaque_id.into(),
            generation,
        }
    }

    pub fn opaque_id(&self) -> &str {
        &self.opaque_id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

impl std::fmt::Debug for RequestCredentialSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestCredentialSnapshot")
            .field("has_opaque_id", &!self.opaque_id.is_empty())
            .field("generation", &self.generation)
            .finish()
    }
}

/// Per-request authentication material for an HTTP-backed tool.
///
/// Values are intentionally private and `Debug` only reports header names so
/// bearer tokens and account identifiers cannot leak through diagnostics.
#[derive(Clone, Default)]
pub struct RequestAuth {
    provider: Option<String>,
    headers: Vec<(String, String)>,
    credential_snapshot: Option<RequestCredentialSnapshot>,
}

impl RequestAuth {
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            provider: None,
            headers: vec![(
                "authorization".to_owned(),
                format!("Bearer {}", token.into()),
            )],
            credential_snapshot: None,
        }
    }

    pub fn from_headers(headers: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            provider: None,
            headers: headers.into_iter().collect(),
            credential_snapshot: None,
        }
    }

    pub fn for_provider(
        provider: impl Into<String>,
        headers: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        Self {
            provider: Some(provider.into()),
            headers: headers.into_iter().collect(),
            credential_snapshot: None,
        }
    }

    /// Construct provider-owned authentication tied to the credential snapshot
    /// that produced these headers.
    pub fn for_provider_snapshot(
        provider: impl Into<String>,
        credential_snapshot: RequestCredentialSnapshot,
        headers: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        Self {
            provider: Some(provider.into()),
            headers: headers.into_iter().collect(),
            credential_snapshot: Some(credential_snapshot),
        }
    }

    pub fn provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }

    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    pub fn credential_snapshot(&self) -> Option<&RequestCredentialSnapshot> {
        self.credential_snapshot.as_ref()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}

impl std::fmt::Debug for RequestAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.headers.iter().map(|(name, _)| name.as_str()).collect();
        f.debug_struct("RequestAuth")
            .field("provider", &self.provider)
            .field("header_names", &names)
            .field("credential_snapshot", &self.credential_snapshot)
            .finish()
    }
}

/// Resolves the current API key for tool HTTP requests.
pub trait ApiKeyProvider: Send + Sync + 'static {
    /// Sync cached read (no refresh). Override point for static providers.
    fn current_api_key(&self) -> Option<String>;

    /// Per-request resolve. `AuthManager` overrides this to drive the
    /// refresh chain; default delegates to the sync method.
    fn current_api_key_async(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        Box::pin(std::future::ready(self.current_api_key()))
    }

    /// Resolve all authentication headers for one request.
    ///
    /// Static/xAI providers inherit the bearer-only implementation. Managed
    /// providers such as ChatGPT Codex override this to also supply their
    /// account and workspace headers without exposing them to tool configs.
    fn current_request_auth_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
        Box::pin(async move { self.current_api_key_async().await.map(RequestAuth::bearer) })
    }

    /// Reload or refresh after a pre-response 401.
    ///
    /// Returns `true` only when the caller may replay the request once with
    /// freshly resolved headers. Static keys do not recover by default.
    fn recover_unauthorized_async(
        &self,
        _rejected: Option<RequestCredentialSnapshot>,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(std::future::ready(false))
    }
}

/// Shared provider used across tool clients.
pub type SharedApiKeyProvider = Arc<dyn ApiKeyProvider>;

/// Resolve the bearer for the next request from the provider.
pub(crate) async fn resolve_bearer(provider: Option<&SharedApiKeyProvider>) -> Option<String> {
    match provider {
        Some(p) => p.current_api_key_async().await,
        None => None,
    }
}

/// Resolve complete per-request authentication for a tool HTTP call.
pub(crate) async fn resolve_request_auth(
    provider: Option<&SharedApiKeyProvider>,
) -> Option<RequestAuth> {
    match provider {
        Some(p) => p.current_request_auth_async().await,
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_auth_debug_redacts_values() {
        let auth = RequestAuth::from_headers([
            (
                "authorization".to_owned(),
                "Bearer sentinel-secret".to_owned(),
            ),
            (
                "chatgpt-account-id".to_owned(),
                "sentinel-account".to_owned(),
            ),
        ]);
        let rendered = format!("{auth:?}");
        assert!(rendered.contains("authorization"));
        assert!(rendered.contains("chatgpt-account-id"));
        assert!(!rendered.contains("sentinel-secret"));
        assert!(!rendered.contains("sentinel-account"));
    }

    #[test]
    fn request_credential_snapshot_debug_redacts_opaque_identity() {
        let auth = RequestAuth::for_provider_snapshot(
            "openai_codex",
            RequestCredentialSnapshot::new("sentinel-credential-id", 7),
            [(
                "authorization".to_owned(),
                "Bearer sentinel-secret".to_owned(),
            )],
        );
        let rendered = format!("{auth:?}");
        assert!(!rendered.contains("sentinel-credential-id"));
        assert!(!rendered.contains("sentinel-secret"));
        assert!(rendered.contains("generation: 7"));
    }
}
