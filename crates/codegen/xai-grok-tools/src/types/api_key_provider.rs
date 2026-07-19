use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Canonical provider identity for ChatGPT Codex-owned tool authentication.
///
/// Keep this identity in the auth layer so individual tool backends cannot
/// drift or accidentally accept credentials belonging to another provider.
pub const OPENAI_CODEX_PROVIDER_ID: &str = "openai_codex";

/// Canonical provider identity for Kimi Code-owned tool authentication.
pub const KIMI_CODE_PROVIDER_ID: &str = "kimi_code";

/// Canonical provider identity for global Z.AI Coding Plan-owned tool and MCP
/// authentication. This must never be accepted by pay-go or China endpoints.
pub const ZAI_CODING_PLAN_PROVIDER_ID: &str = "zai_coding_plan";

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
            .finish_non_exhaustive()
    }
}

/// Internal reason provider-owned ChatGPT Codex authentication was rejected.
///
/// Variants deliberately carry no caller-controlled strings. Their `Debug`
/// and `Display` implementations are also fixed-shape: credential generation,
/// record presence, provider identity, and header state must not cross an
/// error or diagnostics boundary.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenAiCodexAuthError {
    Unavailable,
    ProviderMismatch,
    MissingCredentialGeneration,
    DuplicateCredentialHeader,
    InvalidAuthorization,
    InvalidAccount,
    InvalidFedRamp,
    UnsupportedHeader,
    InvalidHeader,
    Incomplete,
}

impl std::fmt::Debug for OpenAiCodexAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OpenAiCodexAuthError")
    }
}

impl std::fmt::Display for OpenAiCodexAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Codex provider authentication is unavailable")
    }
}

impl std::error::Error for OpenAiCodexAuthError {}

/// Validated provider-owned authentication ready to attach to one Codex HTTP
/// request. Header values remain private and sensitive, while the exact
/// credential snapshot is retained for generation-bound 401 recovery.
#[derive(Clone)]
pub(crate) struct ValidatedOpenAiCodexAuth {
    headers: HeaderMap,
    credential_snapshot: RequestCredentialSnapshot,
}

impl ValidatedOpenAiCodexAuth {
    pub(crate) fn credential_snapshot(&self) -> &RequestCredentialSnapshot {
        &self.credential_snapshot
    }

    pub(crate) fn apply(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.headers(self.headers.clone())
    }

    #[cfg(test)]
    pub(crate) fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

impl std::fmt::Debug for ValidatedOpenAiCodexAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidatedOpenAiCodexAuth")
            .finish_non_exhaustive()
    }
}

/// Per-request authentication material for an HTTP-backed tool.
///
/// Values are intentionally private. `Debug` is fixed-shape so provider,
/// header-name, credential-record, generation, and presence metadata cannot
/// leak through diagnostics.
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
        f.debug_struct("RequestAuth").finish_non_exhaustive()
    }
}

/// Resolves the current API key for tool HTTP requests.
pub trait ApiKeyProvider: Send + Sync + 'static {
    /// Sync cached read (no refresh). Override point for static providers.
    fn current_api_key(&self) -> Option<String>;

    /// Explicit identity for providers that own structured per-request auth.
    /// Generic/static key adapters intentionally return `None`.
    fn request_auth_provider_id(&self) -> Option<&str> {
        None
    }

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
        // Provider-marked credentials require their dedicated validator and
        // endpoint contract. Never downgrade them into a generic bearer for
        // an unrelated xAI-compatible tool client.
        Some(provider) if provider.request_auth_provider_id().is_none() => {
            provider.current_api_key_async().await
        }
        Some(_) | None => None,
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

/// Require a provider that explicitly owns ChatGPT Codex structured auth.
/// Constructors use this marker to reject generic/static key adapters before
/// an HTTP client can be created.
pub(crate) fn require_openai_codex_auth_provider(
    provider: &SharedApiKeyProvider,
) -> Result<(), OpenAiCodexAuthError> {
    if provider.request_auth_provider_id() == Some(OPENAI_CODEX_PROVIDER_ID) {
        Ok(())
    } else {
        Err(OpenAiCodexAuthError::ProviderMismatch)
    }
}

/// Resolve and validate the provider-owned headers for one ChatGPT Codex tool
/// request. There is deliberately no API-key fallback: the provider must emit
/// an explicitly identified, generation-bound [`RequestAuth`] snapshot.
pub(crate) async fn resolve_openai_codex_request_auth(
    provider: &SharedApiKeyProvider,
) -> Result<ValidatedOpenAiCodexAuth, OpenAiCodexAuthError> {
    require_openai_codex_auth_provider(provider)?;
    let auth = provider
        .current_request_auth_async()
        .await
        .ok_or(OpenAiCodexAuthError::Unavailable)?;
    validate_openai_codex_request_auth(&auth)
}

/// Validate a ChatGPT Codex request-auth snapshot and project only the exact
/// credential headers accepted by Codex tool endpoints.
pub(crate) fn validate_openai_codex_request_auth(
    auth: &RequestAuth,
) -> Result<ValidatedOpenAiCodexAuth, OpenAiCodexAuthError> {
    if auth.provider() != Some(OPENAI_CODEX_PROVIDER_ID) {
        return Err(OpenAiCodexAuthError::ProviderMismatch);
    }
    let credential_snapshot = auth
        .credential_snapshot()
        .filter(|snapshot| !snapshot.opaque_id().trim().is_empty() && snapshot.generation() > 0)
        .cloned()
        .ok_or(OpenAiCodexAuthError::MissingCredentialGeneration)?;

    let mut headers = HeaderMap::new();
    let mut has_authorization = false;
    let mut has_account = false;
    let mut has_fedramp = false;
    for (name, value) in auth.headers() {
        let normalized = name.to_ascii_lowercase();
        match normalized.as_str() {
            "authorization" if has_authorization => {
                return Err(OpenAiCodexAuthError::DuplicateCredentialHeader);
            }
            "authorization"
                if value
                    .strip_prefix("Bearer ")
                    .is_none_or(|token| token.trim().is_empty()) =>
            {
                return Err(OpenAiCodexAuthError::InvalidAuthorization);
            }
            "chatgpt-account-id" if has_account => {
                return Err(OpenAiCodexAuthError::DuplicateCredentialHeader);
            }
            "chatgpt-account-id" if value.trim().is_empty() => {
                return Err(OpenAiCodexAuthError::InvalidAccount);
            }
            "x-openai-fedramp" if has_fedramp => {
                return Err(OpenAiCodexAuthError::DuplicateCredentialHeader);
            }
            "x-openai-fedramp" if value != "true" => {
                return Err(OpenAiCodexAuthError::InvalidFedRamp);
            }
            "authorization" | "chatgpt-account-id" | "x-openai-fedramp" => {}
            _ => return Err(OpenAiCodexAuthError::UnsupportedHeader),
        }

        let name = HeaderName::from_bytes(normalized.as_bytes())
            .map_err(|_| OpenAiCodexAuthError::InvalidHeader)?;
        let mut value =
            HeaderValue::from_str(value).map_err(|_| OpenAiCodexAuthError::InvalidHeader)?;
        value.set_sensitive(true);
        headers.insert(name, value);
        has_authorization |= normalized == "authorization";
        has_account |= normalized == "chatgpt-account-id";
        has_fedramp |= normalized == "x-openai-fedramp";
    }
    if !has_authorization || !has_account {
        return Err(OpenAiCodexAuthError::Incomplete);
    }

    Ok(ValidatedOpenAiCodexAuth {
        headers,
        credential_snapshot,
    })
}

/// Fixed-shape failures for provider-owned Kimi Code tool authentication.
/// No variant carries credential, header, or credential-record material.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum KimiCodeRequestAuthError {
    Unavailable,
    ProviderMismatch,
    MissingCredentialGeneration,
    DuplicateAuthorization,
    InvalidAuthorization,
    UnsupportedHeader,
    InvalidHeader,
}

impl std::fmt::Debug for KimiCodeRequestAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KimiCodeRequestAuthError")
    }
}

impl std::fmt::Display for KimiCodeRequestAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Kimi Code provider authentication is unavailable")
    }
}

impl std::error::Error for KimiCodeRequestAuthError {}

/// Validated Kimi Code bearer authentication for one hosted-tool request.
#[derive(Clone)]
pub(crate) struct ValidatedKimiCodeRequestAuth {
    headers: HeaderMap,
}

impl ValidatedKimiCodeRequestAuth {
    pub(crate) fn apply(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.headers(self.headers.clone())
    }

    #[cfg(test)]
    pub(crate) fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

impl std::fmt::Debug for ValidatedKimiCodeRequestAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidatedKimiCodeRequestAuth")
            .finish_non_exhaustive()
    }
}

pub(crate) async fn resolve_kimi_code_request_auth(
    provider: &SharedApiKeyProvider,
) -> Result<ValidatedKimiCodeRequestAuth, KimiCodeRequestAuthError> {
    if provider.request_auth_provider_id() != Some(KIMI_CODE_PROVIDER_ID) {
        return Err(KimiCodeRequestAuthError::ProviderMismatch);
    }
    let auth = provider
        .current_request_auth_async()
        .await
        .ok_or(KimiCodeRequestAuthError::Unavailable)?;
    validate_kimi_code_request_auth(&auth)
}

pub(crate) fn validate_kimi_code_request_auth(
    auth: &RequestAuth,
) -> Result<ValidatedKimiCodeRequestAuth, KimiCodeRequestAuthError> {
    if auth.provider() != Some(KIMI_CODE_PROVIDER_ID) {
        return Err(KimiCodeRequestAuthError::ProviderMismatch);
    }
    auth.credential_snapshot()
        .filter(|snapshot| !snapshot.opaque_id().trim().is_empty() && snapshot.generation() > 0)
        .ok_or(KimiCodeRequestAuthError::MissingCredentialGeneration)?;

    let mut headers = HeaderMap::new();
    let mut has_authorization = false;
    for (name, value) in auth.headers() {
        if !name.eq_ignore_ascii_case("authorization") {
            return Err(KimiCodeRequestAuthError::UnsupportedHeader);
        }
        if has_authorization {
            return Err(KimiCodeRequestAuthError::DuplicateAuthorization);
        }
        if value
            .strip_prefix("Bearer ")
            .is_none_or(|token| token.trim().is_empty())
        {
            return Err(KimiCodeRequestAuthError::InvalidAuthorization);
        }
        let mut value =
            HeaderValue::from_str(value).map_err(|_| KimiCodeRequestAuthError::InvalidHeader)?;
        value.set_sensitive(true);
        headers.insert(HeaderName::from_static("authorization"), value);
        has_authorization = true;
    }
    if !has_authorization {
        return Err(KimiCodeRequestAuthError::InvalidAuthorization);
    }
    Ok(ValidatedKimiCodeRequestAuth { headers })
}

/// Fixed-shape failure for provider-owned Z.AI Coding Plan bearer auth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ZaiCodingPlanRequestAuthError {
    Unavailable,
    ProviderMismatch,
    MissingCredentialGeneration,
    DuplicateAuthorization,
    InvalidAuthorization,
    UnsupportedHeader,
    InvalidHeader,
}

impl std::fmt::Display for ZaiCodingPlanRequestAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Z.AI Coding Plan provider authentication is unavailable")
    }
}

impl std::error::Error for ZaiCodingPlanRequestAuthError {}

#[derive(Clone)]
pub(crate) struct ValidatedZaiCodingPlanRequestAuth {
    headers: HeaderMap,
}

impl ValidatedZaiCodingPlanRequestAuth {
    pub(crate) fn apply(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.headers(self.headers.clone())
    }
}

impl std::fmt::Debug for ValidatedZaiCodingPlanRequestAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidatedZaiCodingPlanRequestAuth")
            .finish_non_exhaustive()
    }
}

pub(crate) async fn resolve_zai_coding_plan_request_auth(
    provider: &SharedApiKeyProvider,
) -> Result<ValidatedZaiCodingPlanRequestAuth, ZaiCodingPlanRequestAuthError> {
    if provider.request_auth_provider_id() != Some(ZAI_CODING_PLAN_PROVIDER_ID) {
        return Err(ZaiCodingPlanRequestAuthError::ProviderMismatch);
    }
    let auth = provider
        .current_request_auth_async()
        .await
        .ok_or(ZaiCodingPlanRequestAuthError::Unavailable)?;
    if auth.provider() != Some(ZAI_CODING_PLAN_PROVIDER_ID) {
        return Err(ZaiCodingPlanRequestAuthError::ProviderMismatch);
    }
    auth.credential_snapshot()
        .filter(|snapshot| !snapshot.opaque_id().trim().is_empty() && snapshot.generation() > 0)
        .ok_or(ZaiCodingPlanRequestAuthError::MissingCredentialGeneration)?;

    let mut headers = HeaderMap::new();
    let mut has_authorization = false;
    for (name, value) in auth.headers() {
        if !name.eq_ignore_ascii_case("authorization") {
            return Err(ZaiCodingPlanRequestAuthError::UnsupportedHeader);
        }
        if has_authorization {
            return Err(ZaiCodingPlanRequestAuthError::DuplicateAuthorization);
        }
        if value
            .strip_prefix("Bearer ")
            .is_none_or(|token| token.trim().is_empty())
        {
            return Err(ZaiCodingPlanRequestAuthError::InvalidAuthorization);
        }
        let mut value = HeaderValue::from_str(value)
            .map_err(|_| ZaiCodingPlanRequestAuthError::InvalidHeader)?;
        value.set_sensitive(true);
        headers.insert(HeaderName::from_static("authorization"), value);
        has_authorization = true;
    }
    if !has_authorization {
        return Err(ZaiCodingPlanRequestAuthError::InvalidAuthorization);
    }
    Ok(ValidatedZaiCodingPlanRequestAuth { headers })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_auth_debug_is_fixed_shape() {
        let auth = RequestAuth::for_provider_snapshot(
            "sentinel-provider",
            RequestCredentialSnapshot::new("sentinel-credential-id", 7),
            [
                (
                    "authorization".to_owned(),
                    "Bearer sentinel-secret".to_owned(),
                ),
                (
                    "chatgpt-account-id".to_owned(),
                    "sentinel-account".to_owned(),
                ),
            ],
        );
        assert_eq!(format!("{auth:?}"), "RequestAuth { .. }");
        assert_eq!(
            format!("{:?}", auth.credential_snapshot().unwrap()),
            "RequestCredentialSnapshot { .. }"
        );
    }

    fn valid_codex_auth() -> RequestAuth {
        RequestAuth::for_provider_snapshot(
            OPENAI_CODEX_PROVIDER_ID,
            RequestCredentialSnapshot::new("sentinel-credential-id", 7),
            [
                (
                    "authorization".to_owned(),
                    "Bearer sentinel-secret".to_owned(),
                ),
                (
                    "chatgpt-account-id".to_owned(),
                    "sentinel-account".to_owned(),
                ),
                ("x-openai-fedramp".to_owned(), "true".to_owned()),
            ],
        )
    }

    #[test]
    fn codex_auth_validation_marks_values_sensitive_and_debug_redacts_them() {
        let validated = validate_openai_codex_request_auth(&valid_codex_auth())
            .expect("provider-owned auth should validate");

        for name in ["authorization", "chatgpt-account-id", "x-openai-fedramp"] {
            assert!(validated.headers()[name].is_sensitive());
        }
        assert_eq!(format!("{validated:?}"), "ValidatedOpenAiCodexAuth { .. }");
    }

    #[test]
    fn codex_auth_validation_errors_do_not_reflect_provider_material() {
        let cases = [
            RequestAuth::for_provider_snapshot(
                "sentinel-other-provider",
                RequestCredentialSnapshot::new("sentinel-credential-id", 7),
                [
                    (
                        "authorization".to_owned(),
                        "Bearer sentinel-secret".to_owned(),
                    ),
                    (
                        "chatgpt-account-id".to_owned(),
                        "sentinel-account".to_owned(),
                    ),
                ],
            ),
            RequestAuth::for_provider_snapshot(
                OPENAI_CODEX_PROVIDER_ID,
                RequestCredentialSnapshot::new("sentinel-credential-id", 7),
                [
                    (
                        "authorization".to_owned(),
                        "Bearer sentinel-secret".to_owned(),
                    ),
                    (
                        "chatgpt-account-id".to_owned(),
                        "sentinel-account".to_owned(),
                    ),
                    (
                        "sentinel-unsupported-header".to_owned(),
                        "sentinel-header-value".to_owned(),
                    ),
                ],
            ),
        ];

        for auth in cases {
            let error = validate_openai_codex_request_auth(&auth)
                .expect_err("foreign or unsupported auth must fail")
                .to_string();
            for forbidden in [
                "sentinel-other-provider",
                "sentinel-secret",
                "sentinel-account",
                "sentinel-unsupported-header",
                "sentinel-header-value",
                "sentinel-credential-id",
            ] {
                assert!(!error.contains(forbidden));
            }
        }
    }

    fn valid_kimi_auth() -> RequestAuth {
        RequestAuth::for_provider_snapshot(
            KIMI_CODE_PROVIDER_ID,
            RequestCredentialSnapshot::new("sentinel-kimi-record", 3),
            [(
                "authorization".to_owned(),
                "Bearer sentinel-kimi-secret".to_owned(),
            )],
        )
    }

    #[test]
    fn kimi_auth_validation_marks_the_bearer_sensitive() {
        let validated = validate_kimi_code_request_auth(&valid_kimi_auth())
            .expect("provider-owned auth should validate");

        assert!(validated.headers()["authorization"].is_sensitive());
        assert_eq!(
            format!("{validated:?}"),
            "ValidatedKimiCodeRequestAuth { .. }"
        );
    }

    #[test]
    fn kimi_auth_validation_requires_a_credential_snapshot() {
        let auth = RequestAuth::for_provider(
            KIMI_CODE_PROVIDER_ID,
            [(
                "authorization".to_owned(),
                "Bearer sentinel-kimi-secret".to_owned(),
            )],
        );

        let error = validate_kimi_code_request_auth(&auth)
            .expect_err("unbound auth must fail")
            .to_string();

        assert_eq!(error, "Kimi Code provider authentication is unavailable");
        assert!(!error.contains("sentinel-kimi-secret"));
    }

    #[test]
    fn kimi_auth_validation_rejects_extra_headers() {
        let auth = RequestAuth::for_provider_snapshot(
            KIMI_CODE_PROVIDER_ID,
            RequestCredentialSnapshot::new("sentinel-kimi-record", 3),
            [
                (
                    "authorization".to_owned(),
                    "Bearer sentinel-kimi-secret".to_owned(),
                ),
                ("x-msh-device-id".to_owned(), "sentinel-device".to_owned()),
            ],
        );

        let error = validate_kimi_code_request_auth(&auth)
            .expect_err("official-client identity headers must fail")
            .to_string();

        assert_eq!(error, "Kimi Code provider authentication is unavailable");
        assert!(!error.contains("sentinel-device"));
    }

    struct StaticKeyProvider;

    impl ApiKeyProvider for StaticKeyProvider {
        fn current_api_key(&self) -> Option<String> {
            Some("sentinel-static-key".to_owned())
        }
    }

    struct ProviderMarkedKeyProvider;

    impl ApiKeyProvider for ProviderMarkedKeyProvider {
        fn current_api_key(&self) -> Option<String> {
            Some("sentinel-provider-key".to_owned())
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(KIMI_CODE_PROVIDER_ID)
        }
    }

    #[tokio::test]
    async fn generic_bearer_resolution_accepts_an_unmarked_static_provider() {
        let provider: SharedApiKeyProvider = Arc::new(StaticKeyProvider);

        assert_eq!(
            resolve_bearer(Some(&provider)).await.as_deref(),
            Some("sentinel-static-key")
        );
    }

    #[tokio::test]
    async fn generic_bearer_resolution_rejects_provider_marked_credentials() {
        let provider: SharedApiKeyProvider = Arc::new(ProviderMarkedKeyProvider);

        assert!(resolve_bearer(Some(&provider)).await.is_none());
    }

    #[tokio::test]
    async fn codex_auth_resolution_rejects_generic_static_key_fallback() {
        let provider: SharedApiKeyProvider = Arc::new(StaticKeyProvider);
        let error = resolve_openai_codex_request_auth(&provider)
            .await
            .expect_err("generic bearer auth must not become Codex auth")
            .to_string();
        assert_eq!(error, "Codex provider authentication is unavailable");
        assert!(!error.contains("sentinel-static-key"));
    }
}
