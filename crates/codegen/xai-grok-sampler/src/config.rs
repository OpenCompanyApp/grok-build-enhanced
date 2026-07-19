//! Sampler configuration types.
//!
//! [`SamplerConfig`] is the per-request configuration handed to the
//! sampler. It deliberately does **not** alias
//! `xai_grok_sampling_types::SamplingConfig` so that the sampler crate
//! avoids transitive dependencies on shell-specific types
//! (`xai-grok-tools`, etc.).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use xai_grok_sampling_types::{
    ApiBackend, CompactionAtTokens, CompactionsRemaining, CredentialBinding, CredentialSourceId,
    DoomLoopRecoveryPolicy, KIMI_CODE_BASE_URL, OPENAI_CODEX_BASE_URL, ProviderId, ReasoningEffort,
    ZAI_CODING_PLAN_BASE_URL,
};

use crate::attribution::SharedAttributionCallback;
use crate::retry::{DEFAULT_MAX_RETRIES, RATE_LIMIT_RETRY_THRESHOLD};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    #[default]
    Bearer,
    XApiKey,
}

/// All knobs that control a single sampling request.
///
/// The session typically owns one `SamplerConfig` per active model
/// and passes it (or a per-request override) to the actor on every
/// submit.
///
/// # Construction in `xai-grok-shell`
///
/// `SamplerConfig` is the single source of truth for sampler
/// configuration. The shell builds it directly (see
/// `agent::config::resolve_model_to_sampling_config` and
/// `session::acp_session::SessionActor::reconstruct_full_config`) by
/// composing chat-state's `xai_grok_sampling_types::SamplingConfig`
/// with `Credentials` (api key, client version).
///
/// URL-derived request headers (e.g. `X-XAI-Token-Auth` for the
/// cli-chat-proxy) are
/// folded into [`Self::extra_headers`] by
/// `agent::config::inject_url_derived_headers` before the
/// `SamplerConfig` is handed to the actor. Auth is selected separately
/// via `auth_scheme`, while `api_backend` controls only the request/response
/// protocol shape.
#[derive(Clone, Serialize, Deserialize)]
pub struct SamplerConfig {
    /// Explicit request provider. Defaults to xAI for serialized configs that
    /// predate provider-aware request policy.
    #[serde(default)]
    pub provider: ProviderId,
    /// Non-secret identity of the credential source selected for this model.
    #[serde(default)]
    pub credential_source: CredentialSourceId,
    /// Optional auth-store binding metadata. This contains no token or account
    /// information and must agree with `provider` when present.
    #[serde(default)]
    pub credential_binding: Option<CredentialBinding>,
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub max_completion_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub api_backend: ApiBackend,
    #[serde(default)]
    pub auth_scheme: AuthScheme,
    /// Extra request headers applied verbatim. The sampler never inspects
    /// the URL to derive headers; callers (the session) inject proxy auth
    /// and other access headers here before constructing the config.
    pub extra_headers: IndexMap<String, String>,
    /// Total context window size in tokens. The sampler does not enforce
    /// it; it is informational metadata used by the session for compaction
    /// decisions.
    pub context_window: u64,
    pub force_http1: bool,
    pub max_retries: Option<u32>,
    pub stream_tool_calls: bool,
    pub idle_timeout_secs: Option<u64>,

    // Reasoning effort
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Provider service-tier selection. Only authenticated Codex Responses
    /// requests consume this field; `default` is an explicit no-tier sentinel.
    #[serde(default)]
    pub service_tier: Option<String>,

    // Client identity
    pub origin_client: Option<OriginClientInfo>,
    pub client_identifier: Option<String>,
    pub deployment_id: Option<String>,
    pub user_id: Option<String>,
    pub client_version: Option<String>,

    /// Optional hook invoked at every UNAUTHORIZED (401) response site. The
    /// sampler reports only whether authentication was present; credential
    /// bytes, prefixes, account IDs, and header values never cross this
    /// boundary. `None` (default) is a no-op -- the 401 arm returns the same
    /// `SamplingError::Auth` it always did.
    ///
    /// `Arc<dyn Trait>` is not serializable, so the field is skipped
    /// in (de)serialization. Round-tripping a config through serde
    /// drops the callback; callers that deserialize a `SamplerConfig`
    /// from disk must re-attach the callback before passing it to
    /// [`crate::SamplingClient::new`] or 401 attribution will be
    /// silently disabled for the rebuilt client.
    #[serde(skip)]
    pub attribution_callback: Option<SharedAttributionCallback>,

    /// Live bearer resolve per request. `None` uses construction-time `api_key`.
    #[serde(skip)]
    pub bearer_resolver: Option<SharedBearerResolver>,

    /// Provider-owned dynamic authentication applied on every request. This is
    /// the only supported source of Codex `Authorization` and
    /// `ChatGPT-Account-ID` headers.
    #[serde(skip)]
    pub request_auth: Option<SharedRequestAuth>,

    #[serde(default)]
    pub supports_backend_search: bool,

    /// Per-model config for the `x-compactions-remaining` header; `None` disables it.
    #[serde(default)]
    pub compactions_remaining: Option<CompactionsRemaining>,

    /// Per-model config for the `x-compaction-at` header; `None` disables it.
    #[serde(default)]
    pub compaction_at_tokens: Option<CompactionAtTokens>,

    /// Server-side doom-loop check policy; `None` disables it. When set, the
    /// client itself sends the opt-in `x-grok-doom-loop-check` header on
    /// streaming Responses API requests and absorbs the reported trigger
    /// events (unlike the environment headers in [`Self::extra_headers`],
    /// this header gates the client's own decode behavior, so it lives with
    /// the decoder).
    #[serde(default)]
    pub doom_loop_recovery: Option<DoomLoopRecoveryPolicy>,

    /// Per-request header injector (e.g. OTel traceparent). Called in `post()`.
    #[serde(skip)]
    pub header_injector: Option<SharedHeaderInjector>,
}

impl std::fmt::Debug for SamplerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Sampling configuration can carry credential source/binding state,
        // custom endpoint userinfo, injected-header state, and provider-local
        // identifiers. Keep diagnostics fixed-shape instead of maintaining a
        // fragile field-by-field redaction allowlist.
        f.debug_struct("SamplerConfig").finish_non_exhaustive()
    }
}

impl Default for SamplerConfig {
    /// Empty defaults so callers can use `..Default::default()` and
    /// new fields don't ripple through every literal site.
    fn default() -> Self {
        Self {
            provider: ProviderId::default(),
            credential_source: CredentialSourceId::default(),
            credential_binding: None,
            api_key: None,
            base_url: String::new(),
            model: String::new(),
            max_completion_tokens: None,
            temperature: None,
            top_p: None,
            api_backend: ApiBackend::default(),
            auth_scheme: AuthScheme::default(),
            extra_headers: IndexMap::new(),
            context_window: 0,
            force_http1: false,
            max_retries: None,
            stream_tool_calls: false,
            idle_timeout_secs: None,
            reasoning_effort: None,
            service_tier: None,
            origin_client: None,
            client_identifier: None,
            deployment_id: None,
            user_id: None,
            client_version: None,
            attribution_callback: None,
            bearer_resolver: None,
            request_auth: None,
            supports_backend_search: false,
            compactions_remaining: None,
            compaction_at_tokens: None,
            doom_loop_recovery: None,
            header_injector: None,
        }
    }
}

impl SamplerConfig {
    /// Build the provider-safe baseline for ChatGPT Codex subscription
    /// requests. The caller must attach [`Self::request_auth`] before creating
    /// a [`crate::SamplingClient`].
    pub fn openai_codex(model: impl Into<String>) -> Self {
        Self {
            provider: ProviderId::OpenAiCodex,
            credential_source: CredentialSourceId::OpenAiCodexSubscription,
            credential_binding: Some(CredentialBinding::openai_codex(None)),
            base_url: OPENAI_CODEX_BASE_URL.to_string(),
            model: model.into(),
            api_backend: ApiBackend::Responses,
            ..Self::default()
        }
    }

    /// Build the provider-safe baseline for a catalog-resolved Kimi Code
    /// model. Authentication must be attached through `request_auth` before
    /// constructing a client.
    pub fn kimi_code(model: impl Into<String>, api_backend: ApiBackend) -> Self {
        let auth_scheme = if matches!(api_backend, ApiBackend::Messages) {
            AuthScheme::XApiKey
        } else {
            AuthScheme::Bearer
        };
        Self {
            provider: ProviderId::KimiCode,
            credential_source: CredentialSourceId::KimiCodeApiKey,
            credential_binding: Some(CredentialBinding::kimi_code(None)),
            base_url: KIMI_CODE_BASE_URL.to_string(),
            model: model.into(),
            api_backend,
            auth_scheme,
            ..Self::default()
        }
    }

    /// Build the provider-safe baseline for a global personal Z.AI Coding
    /// Plan model. Authentication must be attached through `request_auth`.
    pub fn zai_coding_plan(model: impl Into<String>) -> Self {
        Self {
            provider: ProviderId::ZaiCodingPlan,
            credential_source: CredentialSourceId::ZaiCodingPlanApiKey,
            credential_binding: Some(CredentialBinding::zai_coding_plan(None)),
            base_url: ZAI_CODING_PLAN_BASE_URL.to_string(),
            model: model.into(),
            api_backend: ApiBackend::ChatCompletions,
            auth_scheme: AuthScheme::Bearer,
            ..Self::default()
        }
    }
}

/// Cheap sync read of the current bearer for [`SamplerConfig::bearer_resolver`].
pub trait BearerResolver: Send + Sync + std::fmt::Debug {
    fn current_bearer(&self) -> Option<String>;
}

pub type SharedBearerResolver = std::sync::Arc<dyn BearerResolver>;

/// Failure to produce valid provider authentication for one request.
///
/// Variants intentionally carry no arbitrary strings so credential providers
/// cannot accidentally propagate token or account material into logs/errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestAuthError {
    CredentialsUnavailable,
    InvalidAuthorizationHeader,
    InvalidAccountIdHeader,
}

impl std::fmt::Display for RequestAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::CredentialsUnavailable => "provider credentials are unavailable",
            Self::InvalidAuthorizationHeader => "provider authorization header is invalid",
            Self::InvalidAccountIdHeader => "provider account header is invalid",
        })
    }
}

impl std::error::Error for RequestAuthError {}

/// Dynamic provider authentication applied to a fresh header map for every
/// outbound request. Implementations should reload their current credential
/// snapshot cheaply and replace protected headers rather than append them.
pub trait RequestAuth: Send + Sync {
    fn apply(
        &self,
        headers: &mut reqwest::header::HeaderMap,
    ) -> std::result::Result<CredentialBinding, RequestAuthError>;

    /// Provider-owned recovery after an authenticated request is rejected.
    /// The default is inert so custom/xAI auth can never be reached through a
    /// Codex failure. Callers must invoke this at most once per logical 401.
    fn recover_unauthorized(
        &self,
        _rejected: CredentialBinding,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async { false })
    }
}

pub type SharedRequestAuth = std::sync::Arc<dyn RequestAuth>;

/// Per-request header injection (e.g. OTel `traceparent`).
pub trait HeaderInjector: Send + Sync + std::fmt::Debug {
    fn inject(&self, headers: &mut reqwest::header::HeaderMap);
}

pub type SharedHeaderInjector = std::sync::Arc<dyn HeaderInjector>;

/// Retry knobs for the sampler's internal transport-error retry loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retries before giving up.
    pub max_retries: u32,
    /// After this many rate-limit (429) retries, escalate to the caller.
    /// Lower than `max_retries` because rate-limit waits can be long.
    pub rate_limit_retry_threshold: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            rate_limit_retry_threshold: RATE_LIMIT_RETRY_THRESHOLD,
        }
    }
}

/// Identity of the client that originated the request, used for
/// User-Agent rendering. The shell layer composes this with platform
/// info into a final UA string.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OriginClientInfo {
    pub product: String,
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(
            policy.rate_limit_retry_threshold,
            RATE_LIMIT_RETRY_THRESHOLD
        );
    }

    /// Configs serialized before the field existed must keep deserializing.
    #[test]
    fn config_without_doom_loop_recovery_deserializes_to_none() {
        let mut stripped = serde_json::to_value(SamplerConfig::default()).unwrap();
        stripped
            .as_object_mut()
            .unwrap()
            .remove("doom_loop_recovery");
        let config: SamplerConfig = serde_json::from_value(stripped).unwrap();
        assert!(config.doom_loop_recovery.is_none());

        let with_policy = SamplerConfig {
            doom_loop_recovery: Some(DoomLoopRecoveryPolicy {
                max_threshold: 8,
                max_retries: 2,
            }),
            ..Default::default()
        };
        let round_tripped: SamplerConfig =
            serde_json::from_value(serde_json::to_value(&with_policy).unwrap()).unwrap();
        assert_eq!(
            round_tripped.doom_loop_recovery,
            with_policy.doom_loop_recovery
        );
    }

    #[test]
    fn legacy_config_defaults_to_xai_provider() {
        let mut value = serde_json::to_value(SamplerConfig::default()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("provider");
        object.remove("credential_source");
        object.remove("credential_binding");

        let config: SamplerConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config.provider, ProviderId::Xai);
        assert_eq!(config.credential_source, CredentialSourceId::Unspecified);
        assert!(config.credential_binding.is_none());
    }

    #[test]
    fn codex_baseline_uses_current_chatgpt_endpoint_and_responses() {
        let config = SamplerConfig::openai_codex("gpt-5-codex");
        assert_eq!(config.provider, ProviderId::OpenAiCodex);
        assert_eq!(
            config.credential_source,
            CredentialSourceId::OpenAiCodexSubscription
        );
        assert_eq!(config.base_url, OPENAI_CODEX_BASE_URL);
        assert_eq!(config.api_backend, ApiBackend::Responses);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn config_debug_never_prints_credentials_or_identity_values() {
        let mut config = SamplerConfig {
            api_key: Some("secret-api-key".to_string()),
            user_id: Some("sensitive-user".to_string()),
            deployment_id: Some("sensitive-deployment".to_string()),
            client_identifier: Some("sensitive-client-id".to_string()),
            credential_binding: Some(CredentialBinding {
                provider: ProviderId::Xai,
                source: CredentialSourceId::XaiSession,
                record_id: Some("sensitive-record".to_string()),
                generation: 7,
            }),
            ..Default::default()
        };
        config.extra_headers.insert(
            "authorization".to_string(),
            "Bearer secret-extra-header".to_string(),
        );

        assert_eq!(format!("{config:?}"), "SamplerConfig { .. }");
    }
}
