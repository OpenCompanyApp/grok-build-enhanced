//! HTTP client for the xAI sampling APIs.
//!
//! Owns the `reqwest::Client`, default request headers, and per-method
//! defaults. Talks to three backend shapes:
//!
//! * Chat Completions (`/chat/completions`)
//! * Responses API (`/responses`)
//! * Anthropic Messages API (`/messages`)
//!
//! All trace-upload and URL-based header injection is intentionally
//! *not* here. The session is responsible for putting any per-request
//! headers (proxy auth, OTel context, etc.)
//! into [`SamplerConfig::extra_headers`] before constructing the client.

use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
};
use serde::Serialize;

use xai_grok_sampling_types::error::{parse_error_bytes, try_parse_stream_error};
use xai_grok_sampling_types::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ConversationRequest,
    ConversationResponse, CreateResponseWrapper, CredentialBinding, DOOM_LOOP_CHECK_HEADER,
    MessagesRequestWrapper, OPENAI_CODEX_RESPONSES_LITE_HEADER, ProviderId, ReasoningEffort,
    ResponseModelMetadata, Result, SamplingError, build_messages_request, is_check_event, messages,
    rs,
};

use crate::config::{AuthScheme, OriginClientInfo, SamplerConfig};
pub(crate) use crate::provider::openai_codex::turn_state::CodexTurnStateStore;
use crate::provider::openai_codex::{
    endpoint as codex_endpoint, errors as codex_errors, headers as codex_headers,
    responses as codex_responses, sse::CodexSseDecoder,
};
use crate::provider::{kimi_code, zai_coding_plan};
use codex_headers::CODEX_TURN_STATE_HEADER;

// Re-export ApiBackend from the shared types crate for downstream callers.
pub use xai_grok_sampling_types::ApiBackend;

/// Process-level fallback for the `x-grok-client-identifier` header.
const DEFAULT_CLIENT_IDENTIFIER: &str = "grok-shell";

/// Product identifier baked into User-Agent strings.
const AGENT_PRODUCT: &str = "grok-shell";
const ANTHROPIC_DEFAULT_MAX_TOKENS: u32 = 128_000;
/// Normalize provider-specific reasoning semantics before serialization.
/// Historically Grok Build accepted `max` as an alias for `xhigh`; Codex now
/// advertises a genuinely distinct Max tier, so only that provider preserves
/// the new value. Ultra has never been valid outside Codex and fails closed.
fn normalize_reasoning_effort_for_provider(
    provider: ProviderId,
    effort: ReasoningEffort,
) -> Result<ReasoningEffort> {
    if provider.is_openai_codex() || provider.is_kimi_code() || provider.is_zai_coding_plan() {
        return Ok(effort);
    }
    match effort {
        ReasoningEffort::Max => Ok(ReasoningEffort::Xhigh),
        ReasoningEffort::Ultra => Err(SamplingError::InvalidConfiguration(
            "ultra reasoning effort requires a provider with an explicit extended-effort contract",
        )),
        effort => Ok(effort),
    }
}
/// Per-request `x-grok-*` headers. Optional fields are skipped when empty/`None`.
struct GrokRequestHeaders<'a> {
    conv_id: &'a str,
    req_id: &'a str,
    model_id: &'a str,
    session_id: &'a str,
    turn_idx: Option<&'a str>,
    agent_id: &'a str,
    deployment_id: Option<&'a str>,
    user_id: Option<&'a str>,
}

impl GrokRequestHeaders<'_> {
    fn apply(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut b = builder
            .header("x-grok-conv-id", self.conv_id)
            .header("x-grok-req-id", self.req_id)
            .header("x-grok-model-override", self.model_id)
            .header("x-grok-session-id", self.session_id)
            .header("x-grok-agent-id", self.agent_id);
        if let Some(idx) = self.turn_idx {
            b = b.header("x-grok-turn-idx", idx);
        }
        if let Some(id) = self.deployment_id.filter(|s| !s.is_empty()) {
            b = b.header("x-grok-deployment-id", id);
        }
        if let Some(id) = self.user_id.filter(|s| !s.is_empty()) {
            b = b.header("x-grok-user-id", id);
        }
        b
    }
}

/// Parse the `Retry-After` response header as delta-seconds.
/// Our inference backends only emit integer seconds (never HTTP-date),
/// so we only handle that form. HTTP-dates silently return `None` and
/// the caller falls back to exponential backoff.
/// Capped at 120s to prevent absurdly long sleeps from a misbehaving upstream.
/// Deserialize a Responses API SSE event, with a fallback for xAI-specific
/// tool types (e.g., `x_search`) that `async_openai` can't parse.
///
/// The API echoes the request's `tools` array in `ResponseCompleted` and
/// `ResponseCreated` events. If we sent `{"type": "x_search"}`, the response
/// includes it, and `rs::Tool` deserialization fails. On failure, we strip
/// unrecognized tools from the raw JSON and retry.
///
/// On `response.completed` / `response.incomplete`, this also rewrites
/// `response.usage.total_tokens` in place to the live context length
/// (`context_details.input_tokens + context_details.output_tokens`)
/// when the API emits the xAI-specific `context_details` field.
/// Async-openai's typed `ResponseUsage` doesn't model `context_details`,
/// so we peek the raw JSON for it. The cumulative `input_tokens` /
/// `output_tokens` / `cached_tokens` continue to flow from the typed
/// `ResponseUsage` unchanged so billing telemetry stays correct. When
/// the API doesn't emit `context_details` (older deployments) `total_tokens`
/// passes through unchanged.
fn deserialize_response_event(data: &str) -> Result<rs::ResponseStreamEvent> {
    let first_parse = serde_json::from_str::<rs::ResponseStreamEvent>(data);
    let mut event = match first_parse {
        Ok(event) => event,
        Err(first_err) => {
            // Try sanitizing: parse as Value, strip unknown tools, retry.
            if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(data) {
                // Strip tools that async_openai's rs::Tool can't deserialize
                // (e.g., xAI-specific "x_search"). Instead of maintaining a
                // hardcoded allowlist, try deserializing each tool entry —
                // if it fails, drop it.
                if let Some(tools) = value
                    .pointer_mut("/response/tools")
                    .and_then(|v| v.as_array_mut())
                {
                    tools.retain(|t| serde_json::from_value::<rs::Tool>(t.clone()).is_ok());
                }
                if let Ok(mut event) = serde_json::from_value::<rs::ResponseStreamEvent>(value) {
                    apply_terminal_event_overrides(&mut event, data);
                    return Ok(event);
                }
            }
            tracing::error!(
                error = %first_err,
                body_len = data.len(),
                "Failed to deserialize ResponseStreamEvent; body omitted"
            );
            return Err(SamplingError::Serialization(first_err));
        }
    };
    apply_terminal_event_overrides(&mut event, data);
    Ok(event)
}

fn deserialize_response_for_provider(bytes: &[u8], provider: ProviderId) -> Result<rs::Response> {
    if provider.is_openai_codex() {
        return codex_responses::deserialize_response(bytes);
    }

    serde_json::from_slice::<rs::Response>(bytes).map_err(|error| {
        tracing::error!(
            error = %error,
            body_len = bytes.len(),
            "Failed to deserialize rs::Response; body omitted"
        );
        SamplingError::Serialization(error)
    })
}

/// On terminal Responses API events (`response.completed` /
/// `response.incomplete`), rewrite `response.usage.total_tokens` to the
/// live context length when the wire includes
/// `response.usage.context_details.{input_tokens, output_tokens}`.
///
/// `total_tokens` drives the CLI's `/context` bar, the auto-compact
/// threshold, and `meta.totalTokens` on persisted sessions. Under
/// server-side multi-turn loops (e.g. `web_search`, `x_search`) the
/// wire's cumulative total inflates as the loop runs; `context_details`
/// reports the final turn's prompt + output tokens — the real live
/// context the model is sitting in. Billing fields
/// (`input_tokens`, `output_tokens`, `input_tokens_details.cached_tokens`,
/// `output_tokens_details.reasoning_tokens`) stay on the cumulative
/// wire values so telemetry is unaffected.
///
/// No-op when:
/// - the event is not terminal,
/// - `response.usage` is `None`,
/// - `context_details` is absent (older backends / non-loop responses),
/// - or either of `context_details.{input_tokens, output_tokens}` is
///   missing — we don't guess the missing half.
pub(crate) fn apply_terminal_event_overrides(event: &mut rs::ResponseStreamEvent, data: &str) {
    let response = match event {
        rs::ResponseStreamEvent::ResponseCompleted(e) => &mut e.response,
        rs::ResponseStreamEvent::ResponseIncomplete(e) => &mut e.response,
        _ => return,
    };
    // Re-parse for fields async_openai's types omit (context total, cost ticks).
    let Ok(value) = serde_json::from_str::<serde_json::Value>(data) else {
        return;
    };
    // Stash cost ticks in metadata for stream_responses.
    if let Some(ticks) = xai_grok_sampling_types::reported_cost_ticks(
        value
            .pointer("/response/usage/cost_in_usd_ticks")
            .and_then(|v| v.as_i64()),
    ) {
        response
            .metadata
            .get_or_insert_with(Default::default)
            .insert(COST_USD_TICKS_METADATA_KEY.to_owned(), ticks.to_string());
    }
    let Some(usage) = response.usage.as_mut() else {
        return;
    };
    let Some(total) = extract_context_total(&value) else {
        return;
    };
    usage.total_tokens = total;
}

/// Metadata key for cost ticks past typed Response events.
pub(crate) const COST_USD_TICKS_METADATA_KEY: &str = "xai.cost_usd_ticks";

/// Read `response.usage.context_details.{input_tokens, output_tokens}`
/// from the parsed terminal-event JSON and return their sum. Returns `None`
/// if either field is missing or out of `u32` range.
fn extract_context_total(value: &serde_json::Value) -> Option<u32> {
    let cd = value.pointer("/response/usage/context_details")?;
    let i = u32::try_from(cd.get("input_tokens")?.as_u64()?).ok()?;
    let o = u32::try_from(cd.get("output_tokens")?.as_u64()?).ok()?;
    Some(i.saturating_add(o))
}

/// Record `success=false` + `error` on the active inference span when a stream
/// request fails before any response (transport/connect/TLS errors). Without
/// this the `#[instrument]` span closes with both fields Empty, so an outage
/// shows zero `success=false` and error-rate alerts never fire.
fn record_stream_request_failure(err: &reqwest::Error) {
    let span = tracing::Span::current();
    span.record("success", false);
    span.record("error", err.to_string().as_str());
}

fn extract_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(|s| s.min(120))
}

fn extract_should_retry(headers: &reqwest::header::HeaderMap) -> Option<bool> {
    headers
        .get("x-should-retry")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            if s.eq_ignore_ascii_case("true") {
                Some(true)
            } else if s.eq_ignore_ascii_case("false") {
                Some(false)
            } else {
                None // unknown value — treat as absent
            }
        })
}

fn extract_model_metadata(
    headers: &reqwest::header::HeaderMap,
    provider: ProviderId,
    credential_binding: Option<&CredentialBinding>,
) -> Option<ResponseModelMetadata> {
    let context_window = headers
        .get("x-grok-context-window")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let max_completion_tokens = headers
        .get("x-grok-max-completion-tokens")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok());

    let models_etag = headers
        .get("x-models-etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if context_window.is_some() || max_completion_tokens.is_some() || models_etag.is_some() {
        Some(ResponseModelMetadata {
            provider,
            credential_binding: credential_binding.cloned(),
            context_window,
            max_completion_tokens,
            models_etag,
        })
    } else {
        None
    }
}

/// Wrapper for streaming chat completion requests that adds `stream` and
/// `stream_options` fields without modifying the original `ChatCompletionRequest`.
///
/// Uses `#[serde(flatten)]` to inline all fields from the inner request,
/// allowing single-pass serialization instead of the previous two-pass
/// approach (serialize to `Value`, mutate, serialize to bytes).
#[derive(Serialize)]
struct StreamingChatRequest<'a> {
    #[serde(flatten)]
    inner: &'a ChatCompletionRequest,
    stream: bool,
    stream_options: StreamOptions,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

/// HTTP client for sampling. Cheap to clone; carries an `Arc`-backed
/// `reqwest::Client` and the default headers/request-defaults computed
/// from a [`SamplerConfig`] at construction time.
struct PreparedRequest {
    builder: reqwest::RequestBuilder,
    credential_binding: Option<CredentialBinding>,
}

#[derive(Default)]
enum ProviderAuthRecoveryState {
    #[default]
    Idle,
    Recovering,
    Pending(CredentialBinding),
}

/// Cancellation-safe transition guard for provider-owned async recovery.
/// While it is alive, synchronous request preparation fails closed instead of
/// racing a credential rotation. Dropping an unfinished future restores Idle.
struct ProviderAuthRecoveryGuard<'a> {
    state: &'a std::sync::Mutex<ProviderAuthRecoveryState>,
    finished: bool,
}

impl<'a> ProviderAuthRecoveryGuard<'a> {
    fn begin(state: &'a std::sync::Mutex<ProviderAuthRecoveryState>) -> Option<Self> {
        let mut current = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(&*current, ProviderAuthRecoveryState::Idle) {
            return None;
        }
        *current = ProviderAuthRecoveryState::Recovering;
        drop(current);
        Some(Self {
            state,
            finished: false,
        })
    }

    fn finish(mut self, next: ProviderAuthRecoveryState) -> bool {
        let mut current = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let transitioned = matches!(&*current, ProviderAuthRecoveryState::Recovering);
        if transitioned {
            *current = next;
        }
        self.finished = true;
        transitioned
    }
}

impl Drop for ProviderAuthRecoveryGuard<'_> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let mut current = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(&*current, ProviderAuthRecoveryState::Recovering) {
            *current = ProviderAuthRecoveryState::Idle;
        }
    }
}

#[derive(Clone)]
pub struct SamplingClient {
    http: reqwest::Client,
    default_headers: HeaderMap,
    base_url: String,
    defaults: ClientDefaults,
    /// Optional 401-attribution hook. The shell wires this to emit a
    /// structured event at every UNAUTHORIZED arm so 401s can be
    /// bucketed by stale-snapshot vs. live-token-rejected. `None` for
    /// sampler-only callers and tests.
    attribution_callback: Option<crate::attribution::SharedAttributionCallback>,
    /// Per-request bearer override. See `SamplerConfig::bearer_resolver`.
    bearer_resolver: Option<crate::config::SharedBearerResolver>,
    /// Per-request header injection (OTel traceparent).
    header_injector: Option<crate::config::SharedHeaderInjector>,
    /// Provider-owned dynamic credentials applied after ordinary header
    /// injection on every request.
    request_auth: Option<crate::config::SharedRequestAuth>,
    /// Exact rejected binding that the next request on this client must
    /// strictly succeed. Shared across cheap clones so direct sampler callers
    /// cannot bypass the post-recovery check by cloning the transport.
    /// Deliberately omitted from `Debug` and tracing.
    provider_auth_recovery_state: std::sync::Arc<std::sync::Mutex<ProviderAuthRecoveryState>>,
    /// Per-turn ChatGPT sticky-routing state shared by cheap client clones.
    /// An empty request ID (for example an auxiliary title call) neither reads
    /// nor clears this state, so concurrent auxiliary work cannot disturb the
    /// active agent turn.
    codex_turn_state: CodexTurnStateStore,
}

impl std::fmt::Debug for SamplingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The client contains configured endpoint userinfo and credential
        // resolver state. Keep diagnostics independent of their values and
        // presence.
        f.debug_struct("SamplingClient").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Default)]
struct ClientDefaults {
    provider: ProviderId,
    /// True only for explicit xAI requests targeting a canonical inference URL.
    xai_trusted_origin: bool,
    model: String,
    max_completion_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    api_backend: ApiBackend,
    auth_scheme: AuthScheme,
    supports_reasoning_summary_parameter: bool,
    default_reasoning_summary: Option<String>,
    service_tier: Option<String>,
    stream_tool_calls: bool,
    responses_lite: bool,
    doom_loop_recovery: Option<xai_grok_sampling_types::DoomLoopRecoveryPolicy>,
}

// =============================================================================
// User-Agent helpers
// =============================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlatformInfo {
    os: String,
    arch: String,
}

impl PlatformInfo {
    fn current() -> Self {
        let os = match std::env::consts::OS {
            "macos" => "macos",
            "windows" => "windows",
            other => other,
        }
        .to_string();

        let arch = match std::env::consts::ARCH {
            "arm64" => "aarch64",
            "x86_64" => "x86_64",
            other => other,
        }
        .to_string();

        Self { os, arch }
    }
}

fn agent_version() -> String {
    xai_grok_version::VERSION.to_string()
}

/// Render a User-Agent string for the given origin client.
///
/// Mirrors the shell's `user_agent_string_for` but uses sampler-local
/// constants. The session typically owns the canonical User-Agent
/// rendering for process-wide HTTP clients; this helper is for
/// per-session sampling clients that want to override it.
pub fn user_agent_string_for(origin: &OriginClientInfo) -> String {
    let agent_version = agent_version();
    let platform = PlatformInfo::current();

    if origin.product == AGENT_PRODUCT && origin.version.as_deref() == Some(agent_version.as_str())
    {
        return format!(
            "{}/{} ({}; {})",
            AGENT_PRODUCT, agent_version, platform.os, platform.arch
        );
    }

    match origin.version.as_deref() {
        Some(origin_version) => format!(
            "{}/{} {}/{} ({}; {})",
            origin.product,
            origin_version,
            AGENT_PRODUCT,
            agent_version,
            platform.os,
            platform.arch
        ),
        None => format!(
            "{} {}/{} ({}; {})",
            origin.product, AGENT_PRODUCT, agent_version, platform.os, platform.arch
        ),
    }
}

// =============================================================================
// SamplingClient
// =============================================================================

impl SamplingClient {
    /// Construct a sampling client from a [`SamplerConfig`].
    ///
    /// Grabs the process-wide shared `reqwest::Client` (HTTP/2 by
    /// default, HTTP/1.1 when `config.force_http1` is set) and
    /// pre-computes the default request headers. This does not perform
    /// any network I/O.
    pub fn new(config: SamplerConfig) -> Result<Self> {
        Self::new_with_codex_turn_state(config, CodexTurnStateStore::default())
    }

    pub(crate) fn new_with_codex_turn_state(
        mut config: SamplerConfig,
        codex_turn_state: CodexTurnStateStore,
    ) -> Result<Self> {
        let xai_trusted_origin = config.provider == ProviderId::Xai
            && xai_grok_sampling_types::is_trusted_xai_inference_url(&config.base_url);
        if config.provider == ProviderId::Xai && !xai_trusted_origin {
            // Legacy/restored state may label an arbitrary custom endpoint as
            // xAI. Preserve routing, but fail closed at the final wire seam:
            // no static, live, or provider-hook xAI credential may reach it.
            config.api_key = None;
            config.bearer_resolver = None;
            config.request_auth = None;
        }

        if let Some(binding) = config.credential_binding.as_ref()
            && (binding.provider != config.provider
                || (config.credential_source
                    != xai_grok_sampling_types::CredentialSourceId::Unspecified
                    && binding.source != config.credential_source))
        {
            return Err(SamplingError::InvalidConfiguration(
                "credential binding does not match the selected provider/source",
            ));
        }

        if config.provider.is_openai_codex() {
            if !codex_endpoint::is_valid_base_url(&config.base_url) {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex credentials may only be sent to the ChatGPT Codex endpoint",
                ));
            }
            if config.api_backend != ApiBackend::Responses {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex requires the Responses API backend",
                ));
            }
            if config.api_key.is_some()
                || config.bearer_resolver.is_some()
                || config.request_auth.is_none()
            {
                return Err(SamplingError::InvalidConfiguration(
                    codex_headers::CODEX_AUTH_CONFIGURATION_INVALID,
                ));
            }
        }
        if config.provider.is_kimi_code() {
            kimi_code::validate_config(
                config.provider,
                &config.base_url,
                config.api_backend.clone(),
            )?;
            if config.api_key.is_some()
                || config.bearer_resolver.is_some()
                || config.request_auth.is_none()
            {
                return Err(SamplingError::InvalidConfiguration(
                    "Kimi Code requires provider-scoped request authentication",
                ));
            }
            let expected_scheme = if matches!(config.api_backend, ApiBackend::Messages) {
                AuthScheme::XApiKey
            } else {
                AuthScheme::Bearer
            };
            if config.auth_scheme != expected_scheme {
                return Err(SamplingError::InvalidConfiguration(
                    "Kimi Code authentication scheme does not match the catalog protocol",
                ));
            }
        }
        if config.provider.is_zai_coding_plan() {
            zai_coding_plan::validate_config(
                config.provider,
                &config.base_url,
                config.api_backend.clone(),
            )?;
            if config.api_key.is_some()
                || config.bearer_resolver.is_some()
                || config.request_auth.is_none()
                || config.auth_scheme != AuthScheme::Bearer
            {
                return Err(SamplingError::InvalidConfiguration(
                    "Z.AI Coding Plan requires provider-scoped bearer authentication",
                ));
            }
        }

        let mut headers = HeaderMap::new();
        let mut responses_lite = false;
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref api_key) = config.api_key {
            match config.auth_scheme {
                AuthScheme::XApiKey => {
                    let mut header_value = HeaderValue::from_str(api_key).map_err(|_| {
                        tracing::debug!(
                            "Invalid api_key: cannot be converted to a valid HTTP header"
                        );
                        SamplingError::Auth(
                            "Invalid api_key: cannot be converted to a valid HTTP header"
                                .to_string(),
                        )
                    })?;
                    header_value.set_sensitive(true);
                    headers.insert(HeaderName::from_static("x-api-key"), header_value);
                }
                AuthScheme::Bearer => {
                    let bearer = format!("Bearer {}", api_key);
                    let mut header_value = HeaderValue::from_str(&bearer).map_err(|_| {
                        tracing::debug!(
                            "Invalid api_key: cannot be converted to a valid HTTP Authorization header"
                        );
                        SamplingError::Auth(
                            "Invalid api_key: cannot be converted to a valid HTTP Authorization header"
                                .to_string(),
                        )
                    })?;
                    header_value.set_sensitive(true);
                    headers.insert(AUTHORIZATION, header_value);
                }
            }
        }

        // Apply all extra headers verbatim. This is the single
        // injection point for proxy-auth headers and any other URL- or
        // environment-specific headers the session decides to set.
        for (key, value) in &config.extra_headers {
            let header_name = HeaderName::try_from(key.as_str())
                .map_err(|_| SamplingError::InvalidConfiguration("Invalid extra header name"))?;
            if config.provider == ProviderId::Xai
                && !xai_trusted_origin
                && (Self::is_xai_specific_header(&header_name)
                    || Self::is_credential_header(&header_name))
            {
                continue;
            }
            if config.provider != ProviderId::Xai && Self::is_xai_specific_header(&header_name) {
                return Err(SamplingError::InvalidConfiguration(
                    "xAI-specific headers require the xAI provider",
                ));
            }
            if header_name.as_str() == OPENAI_CODEX_RESPONSES_LITE_HEADER {
                if !config.provider.is_openai_codex() {
                    return Err(SamplingError::InvalidConfiguration(
                        "Responses Lite marker requires the OpenAI Codex provider",
                    ));
                }
                if value != "true" {
                    return Err(SamplingError::InvalidConfiguration(
                        "Responses Lite marker must be exactly true",
                    ));
                }
                responses_lite = true;
                continue;
            }
            if config.provider.is_openai_codex()
                && (codex_headers::is_credential_header_or_alias(&header_name)
                    || codex_headers::is_xai_specific_header(&header_name)
                    || codex_headers::is_provider_identity_header(&header_name))
            {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex protected/provider headers cannot be set via extra_headers",
                ));
            }
            if config.provider.is_kimi_code() && kimi_code::is_protected_header(&header_name) {
                return Err(SamplingError::InvalidConfiguration(
                    "Kimi Code protected/provider headers cannot be set via extra_headers",
                ));
            }
            if config.provider.is_zai_coding_plan()
                && zai_coding_plan::is_protected_header(&header_name)
            {
                return Err(SamplingError::InvalidConfiguration(
                    "Z.AI Coding Plan protected/provider headers cannot be set via extra_headers",
                ));
            }
            let mut header_value = HeaderValue::from_str(value)
                .map_err(|_| SamplingError::InvalidConfiguration("Invalid extra header value"))?;
            if Self::is_sensitive_header(header_name.as_str()) {
                header_value.set_sensitive(true);
            }
            headers.insert(header_name, header_value);
        }

        match config.provider {
            ProviderId::Xai if xai_trusted_origin => {
                // xAI request identity is meaningful only on the two
                // canonical xAI inference routes.
                if let Some(client_version) = config.client_version.as_ref()
                    && let Ok(header_value) = HeaderValue::from_str(client_version)
                {
                    headers.insert(
                        HeaderName::from_static("x-grok-client-version"),
                        header_value,
                    );
                }

                if let Some(deployment_id) = config.deployment_id.as_ref()
                    && let Ok(header_value) = HeaderValue::from_str(deployment_id)
                {
                    headers.insert(
                        HeaderName::from_static("x-grok-deployment-id"),
                        header_value,
                    );
                }

                if let Some(user_id) = config.user_id.as_ref()
                    && let Ok(header_value) = HeaderValue::from_str(user_id)
                {
                    headers.insert(HeaderName::from_static("x-grok-user-id"), header_value);
                }

                let client_id = config
                    .client_identifier
                    .clone()
                    .unwrap_or_else(|| DEFAULT_CLIENT_IDENTIFIER.to_string());
                if let Ok(header_value) = HeaderValue::from_str(&client_id) {
                    headers.insert(
                        HeaderName::from_static("x-grok-client-identifier"),
                        header_value,
                    );
                }
            }
            ProviderId::Xai | ProviderId::Custom => {}
            ProviderId::OpenAiCodex => codex_headers::insert_provider_identity(&mut headers),
            ProviderId::KimiCode => {
                if matches!(config.api_backend, ApiBackend::Messages) {
                    headers.insert(
                        HeaderName::from_static("anthropic-version"),
                        HeaderValue::from_static(
                            xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_VERSION,
                        ),
                    );
                    headers.insert(
                        HeaderName::from_static("anthropic-beta"),
                        HeaderValue::from_static(xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_BETA),
                    );
                }
            }
            ProviderId::ZaiCodingPlan => {}
        }

        // Always set User-Agent: per-session origin if available, else fallback.
        {
            let ua_string = match config.origin_client.as_ref() {
                Some(origin) => user_agent_string_for(origin),
                None => user_agent_string_for(&OriginClientInfo {
                    product: AGENT_PRODUCT.to_string(),
                    version: Some(agent_version()),
                }),
            };
            if let Ok(v) = HeaderValue::from_str(&ua_string) {
                headers.insert(USER_AGENT, v);
            }
        }
        Self::mark_sensitive_headers(&mut headers);

        let http = if config.provider.is_openai_codex() {
            codex_endpoint::http_client()?
        } else if config.provider.is_kimi_code() {
            kimi_code::http_client(config.force_http1)?
        } else if config.provider.is_zai_coding_plan() {
            zai_coding_plan::http_client(config.force_http1)?
        } else if config.force_http1 {
            tracing::info!("Using HTTP/1.1 for sampling client (force_http1=true)");
            crate::shared_http::client_http1().map_err(SamplingError::Http)?
        } else {
            crate::shared_http::client().map_err(SamplingError::Http)?
        };

        tracing::info!(
            target: crate::sampling_log::TARGET,
            event = "client_new",
            model = %config.model,
            provider = ?config.provider,
            api_backend = ?config.api_backend,
            // "unset" (not "none"): `ReasoningEffort::None` is a real wire value.
            reasoning_effort = config.reasoning_effort.map_or("unset", |e| e.as_str()),
        );

        let defaults = ClientDefaults {
            provider: config.provider,
            xai_trusted_origin,
            model: config.model,
            max_completion_tokens: config.max_completion_tokens,
            temperature: config.temperature,
            top_p: config.top_p,
            api_backend: config.api_backend,
            auth_scheme: config.auth_scheme,
            supports_reasoning_summary_parameter: config.supports_reasoning_summary_parameter,
            default_reasoning_summary: config.default_reasoning_summary,
            service_tier: config.service_tier,
            stream_tool_calls: config.stream_tool_calls,
            responses_lite,
            doom_loop_recovery: config.doom_loop_recovery,
        };

        Ok(Self {
            http,
            default_headers: headers,
            base_url: config.base_url,
            defaults,
            attribution_callback: config.attribution_callback,
            bearer_resolver: config.bearer_resolver,
            header_injector: config.header_injector,
            request_auth: config.request_auth,
            provider_auth_recovery_state: Default::default(),
            codex_turn_state,
        })
    }

    /// Rebuild this HTTP client without dropping the actor-owned Codex turn
    /// state. Used only for transport fallback within the same submission.
    pub(crate) fn rebuild_with_config(&self, config: SamplerConfig) -> Result<Self> {
        let mut rebuilt = Self::new_with_codex_turn_state(config, self.codex_turn_state.clone())?;
        rebuilt.provider_auth_recovery_state =
            std::sync::Arc::clone(&self.provider_auth_recovery_state);
        Ok(rebuilt)
    }

    /// The configured API backend for this client.
    pub fn api_backend(&self) -> ApiBackend {
        self.defaults.api_backend.clone()
    }

    /// The explicit provider that owns requests made by this client.
    pub fn provider(&self) -> ProviderId {
        self.defaults.provider
    }

    /// Build a fresh request-header map and bind it to the exact credential
    /// snapshot that supplied provider-owned authentication.
    ///
    /// When `rejected` is present, this is the first request preparation after
    /// a provider reported 401. The actual signing binding must be a strict
    /// successor of that exact rejected binding; a preflight alone is not
    /// sufficient because another refresh or account switch could race it.
    fn prepare_request_headers(
        &self,
        rejected: Option<&CredentialBinding>,
    ) -> Result<(HeaderMap, Option<CredentialBinding>)> {
        let mut headers = self.default_headers.clone();
        if let Some(resolver) = &self.bearer_resolver
            && let Some(fresh) = resolver.current_bearer()
        {
            match self.defaults.auth_scheme {
                AuthScheme::XApiKey => {
                    headers.remove(AUTHORIZATION);
                    if let Ok(mut value) = HeaderValue::from_str(&fresh) {
                        value.set_sensitive(true);
                        headers.insert(HeaderName::from_static("x-api-key"), value);
                    }
                }
                AuthScheme::Bearer => {
                    headers.remove(HeaderName::from_static("x-api-key"));
                    if let Ok(mut value) = HeaderValue::from_str(&format!("Bearer {fresh}")) {
                        value.set_sensitive(true);
                        headers.insert(AUTHORIZATION, value);
                    }
                }
            }
        }
        if let Some(injector) = &self.header_injector {
            injector.inject(&mut headers);
        }

        if self.defaults.provider.is_openai_codex() {
            codex_headers::prepare_for_request_auth(&mut headers);
        }

        let credential_binding = self
            .request_auth
            .as_ref()
            .map(|request_auth| {
                request_auth
                    .apply(&mut headers)
                    .map_err(|error| SamplingError::Auth(error.to_string()))
            })
            .transpose()?;

        if self.defaults.provider == ProviderId::Xai && !self.defaults.xai_trusted_origin {
            Self::retain_wire_headers(&mut headers, |name| {
                !Self::is_credential_header(name) && !Self::is_xai_specific_header(name)
            });
        } else if self.defaults.provider == ProviderId::Custom {
            Self::retain_wire_headers(&mut headers, |name| !Self::is_xai_specific_header(name));
        }

        if self.defaults.provider.is_openai_codex() {
            codex_headers::seal_after_request_auth(&mut headers, credential_binding.as_ref())?;
        }
        if self.defaults.provider.is_kimi_code() {
            kimi_code::seal_headers(
                &headers,
                self.defaults.api_backend.clone(),
                credential_binding.as_ref(),
            )?;
        }
        if self.defaults.provider.is_zai_coding_plan() {
            zai_coding_plan::seal_headers(&headers, credential_binding.as_ref())?;
        }
        if let Some(rejected) = rejected
            && !credential_binding
                .as_ref()
                .is_some_and(|current| current.is_strict_successor_of(rejected))
        {
            return Err(SamplingError::InvalidConfiguration(
                "provider authentication recovery did not advance the rejected credential binding",
            ));
        }

        // Do this after every extension point. Static credentials and live
        // bearer replacement are marked at insertion time as well, while this
        // final pass catches credential-like extra/injected headers and custom
        // RequestAuth implementations.
        Self::mark_sensitive_headers(&mut headers);
        Ok((headers, credential_binding))
    }

    /// POST with default headers. Legacy bearer resolution and provider-owned
    /// dynamic auth are applied to a fresh map on every call.
    fn post(&self, url: impl reqwest::IntoUrl) -> Result<PreparedRequest> {
        self.post_after_provider_auth_recovery(url, None)
    }

    fn post_after_provider_auth_recovery(
        &self,
        url: impl reqwest::IntoUrl,
        rejected: Option<&CredentialBinding>,
    ) -> Result<PreparedRequest> {
        // Serialize the one-shot successor check across cheap client clones.
        // Holding this small sync mutex through `RequestAuth::apply` is safe:
        // that trait method is deliberately synchronous and required to be a
        // cheap snapshot read.
        let mut recovery_state = self
            .provider_auth_recovery_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending_binding = match &*recovery_state {
            ProviderAuthRecoveryState::Idle => None,
            ProviderAuthRecoveryState::Recovering => {
                return Err(SamplingError::InvalidConfiguration(
                    "provider authentication recovery is still in progress",
                ));
            }
            ProviderAuthRecoveryState::Pending(binding) => Some(binding),
        };
        if let (Some(explicit), Some(pending_binding)) = (rejected, pending_binding)
            && explicit != pending_binding
        {
            return Err(SamplingError::InvalidConfiguration(
                "provider authentication retry does not match the pending rejected binding",
            ));
        }
        let required_successor = rejected.or(pending_binding);
        let prepared_headers = self.prepare_request_headers(required_successor);
        if prepared_headers.is_ok()
            && matches!(&*recovery_state, ProviderAuthRecoveryState::Pending(_))
        {
            *recovery_state = ProviderAuthRecoveryState::Idle;
        }
        drop(recovery_state);
        let (headers, credential_binding) = prepared_headers?;

        tracing::info!(
            target: crate::sampling_log::TARGET,
            event = "client_post",
            model = %self.defaults.model,
            provider = ?self.defaults.provider,
            api_backend = ?self.defaults.api_backend,
        );

        Ok(PreparedRequest {
            builder: self.http.post(url).headers(headers),
            credential_binding,
        })
    }

    fn apply_provider_request_headers(
        &self,
        builder: reqwest::RequestBuilder,
        grok_headers: &GrokRequestHeaders<'_>,
    ) -> Result<reqwest::RequestBuilder> {
        if self.defaults.provider.is_openai_codex() {
            let builder = codex_headers::apply_request_identity(
                builder,
                grok_headers.session_id,
                grok_headers.conv_id,
                self.defaults.responses_lite,
            )?;
            Ok(self.codex_turn_state.apply(builder, grok_headers.req_id))
        } else if self.defaults.provider == ProviderId::Xai && self.defaults.xai_trusted_origin {
            Ok(grok_headers.apply(builder))
        } else {
            // Non-xAI providers and legacy xAI-labelled custom endpoints keep
            // Grok/xAI conversation identity local.
            Ok(builder)
        }
    }

    fn capture_codex_turn_state(&self, headers: &HeaderMap, request_id: &str) {
        if self.defaults.provider.is_openai_codex() {
            self.codex_turn_state.capture(headers, request_id);
        }
    }

    /// Invoke the optional 401 attribution callback for one logical
    /// 401 response. Each of the six UNAUTHORIZED arms in this file
    /// calls this helper immediately before returning
    /// `SamplingError::Auth(...)`. Emit happens at the lowest layer
    /// that saw the status, so higher layers that react to a 401 must
    /// not emit a duplicate event.
    ///
    /// Credential material is intentionally never passed to the callback.
    fn record_401_attribution(&self, consumer: crate::attribution::SamplingConsumer) {
        if let Some(cb) = self.attribution_callback.as_ref() {
            cb.record_401(consumer, self.has_auth());
        }
    }

    fn has_auth(&self) -> bool {
        self.request_auth.is_some()
            || self.bearer_resolver.is_some()
            || self.default_headers.get(AUTHORIZATION).is_some()
            || self.default_headers.get("x-api-key").is_some()
    }

    /// Ask the provider-owned request authenticator to recover one rejected
    /// credential snapshot. This is deliberately unavailable to xAI/custom
    /// clients: their authentication lifecycle remains owned by the shell's
    /// existing `AuthManager`.
    pub async fn recover_provider_auth_after_unauthorized(
        &self,
        rejected: CredentialBinding,
    ) -> bool {
        if !self.defaults.provider.is_openai_codex() {
            return false;
        }
        let Some(request_auth) = &self.request_auth else {
            return false;
        };
        let Some(recovery_guard) =
            ProviderAuthRecoveryGuard::begin(self.provider_auth_recovery_state.as_ref())
        else {
            return false;
        };
        if !request_auth.recover_unauthorized(rejected.clone()).await {
            return false;
        }

        // A provider reporting recovery success is only advisory. Reload the
        // post-recovery snapshot and require a same-record generation advance
        // before any caller is told to retry.
        if self.prepare_request_headers(Some(&rejected)).is_err() {
            return false;
        }

        // Pin the exact rejection into the next actual request on this client.
        // The actor passes the same binding explicitly as an additional guard;
        // direct sampler callers (such as compaction) are protected by this
        // shared one-shot state without changing their public call shape.
        recovery_guard.finish(ProviderAuthRecoveryState::Pending(rejected))
    }

    pub fn auth_info(&self) -> crate::sampling_log::AuthInfo {
        let has_auth = self.has_auth();
        let auth_type = match (self.defaults.provider, self.defaults.auth_scheme, has_auth) {
            (ProviderId::OpenAiCodex, _, true) => "openai-codex-subscription",
            (ProviderId::KimiCode, _, true) => "kimi-code-api-key",
            (ProviderId::ZaiCodingPlan, _, true) => "zai-coding-plan-api-key",
            (_, AuthScheme::XApiKey, true) => "x-api-key",
            (_, AuthScheme::Bearer, true) => "bearer",
            (_, _, false) => "none",
        };
        crate::sampling_log::AuthInfo { auth_type }
    }

    fn is_xai_specific_header(name: &HeaderName) -> bool {
        let name = name.as_str();
        name.starts_with("x-grok-")
            || name.starts_with("x-xai-")
            || matches!(
                name,
                "x-authenticateresponse"
                    | "x-userid"
                    | "x-email"
                    | "x-compactions-remaining"
                    | "x-compaction-at"
            )
    }

    fn is_credential_header(name: &HeaderName) -> bool {
        let lower = name.as_str();
        lower == AUTHORIZATION.as_str()
            || lower == "proxy-authorization"
            || lower == "x-api-key"
            || Self::is_sensitive_header(lower)
    }

    fn retain_wire_headers(headers: &mut HeaderMap, mut keep: impl FnMut(&HeaderName) -> bool) {
        let remove: Vec<HeaderName> = headers.keys().filter(|name| !keep(name)).cloned().collect();
        for name in remove {
            headers.remove(name);
        }
    }

    /// Check if a header name contains sensitive information that should be redacted.
    fn is_sensitive_header(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        let compact: String = lower
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect();
        compact.contains("authorization")
            || compact.contains("authentication")
            || compact.contains("apikey")
            || compact.contains("token")
            || compact.contains("secret")
            || compact.contains("credential")
            || compact.contains("account")
            || compact.contains("userid")
            || compact.contains("email")
            || compact.contains("organization")
            || compact.contains("orgid")
            || compact.contains("teamid")
            || compact.contains("deploymentid")
            || compact.contains("projectid")
            || compact.contains("cookie")
            || compact.contains("session")
            || compact.contains("threadid")
            || compact.contains("clientrequestid")
            || compact.contains("clientidentifier")
            || lower == "x-openai-fedramp"
            || lower == CODEX_TURN_STATE_HEADER
    }

    /// Mark every credential- or identity-like value sensitive at the HTTP
    /// layer. This is intentionally name-based and provider-neutral so custom
    /// proxy access headers receive the same `HeaderValue` Debug redaction as
    /// standard Authorization and API-key headers.
    fn mark_sensitive_headers(headers: &mut HeaderMap) {
        for (name, value) in headers.iter_mut() {
            if Self::is_sensitive_header(name.as_str()) {
                value.set_sensitive(true);
            }
        }
    }

    /// Format a single header for error messages, redacting sensitive values.
    fn format_header(name: &str, value: &str) -> String {
        let display_value = if Self::is_sensitive_header(name) {
            "[REDACTED]"
        } else {
            value
        };
        format!("  {}: {}", name, display_value)
    }

    /// Fixed/coarse request-header context for user-visible API errors. Never
    /// enumerate names, values, or counts: even a redacted optional name (or a
    /// changing count) exposes provider security state through diagnostics.
    fn request_header_diagnostics(&self, streaming: bool) -> Vec<String> {
        vec![format!(
            "  details omitted (provider={}, streaming={streaming})",
            self.defaults.provider
        )]
    }

    /// Build response headers for error messages.
    ///
    /// The experimental ChatGPT backend is an authenticated third-party
    /// boundary. Its response values must never be able to reflect bearer or
    /// account material into local logs, spans, or user-visible errors. For
    /// Codex, expose only the presence of a tiny fixed allowlist of operational
    /// headers. Other providers retain the existing value-bearing diagnostics.
    fn format_response_headers(headers: &HeaderMap, provider: ProviderId) -> Vec<String> {
        if provider.is_openai_codex() {
            return codex_errors::response_header_diagnostics(headers);
        }
        if provider.is_kimi_code() {
            return kimi_code::response_header_diagnostics(headers);
        }
        if provider.is_zai_coding_plan() {
            return zai_coding_plan::response_header_diagnostics(headers);
        }

        headers
            .iter()
            .map(|(name, value)| Self::format_header(name.as_str(), &format!("{:?}", value)))
            .collect()
    }

    /// Emit fixed request diagnostics without enumerating header names, values,
    /// or counts. Even a redacted per-header event exposes optional security
    /// state through its name (and a count exposes it through cardinality).
    fn log_request_diagnostics(request: &reqwest::Request, endpoint_name: &str) {
        tracing::debug!(
            endpoint = endpoint_name,
            method = %request.method(),
            url = %request.url(),
            has_body = request.body().is_some(),
            "HTTP request prepared; header diagnostics suppressed"
        );
    }

    /// Build error context message based on error type and status code.
    /// Includes relevant request/response details depending on what the error is about.
    fn build_api_error_message(
        &self,
        status: reqwest::StatusCode,
        server_message: &str,
        endpoint: &str,
        req_headers: &[String],
        resp_headers: Option<&[String]>,
    ) -> String {
        let server_message_lower = server_message.to_lowercase();

        let mut context_parts = vec![server_message.to_string()];
        context_parts.push(format!("\nRequest URL: {}", endpoint));

        // Show headers if error mentions headers
        if server_message_lower.contains("header") {
            context_parts.push(format!("Request headers:\n{}", req_headers.join("\n")));
        }

        // Always show response headers for server errors
        if status.is_server_error()
            && let Some(resp_hdrs) = resp_headers
        {
            context_parts.push(format!("Response headers:\n{}", resp_hdrs.join("\n")));
        }

        context_parts.join("\n")
    }

    /// Convert an error body into a diagnostic message without allowing the
    /// experimental ChatGPT backend to reflect credential or account material
    /// into logs and user-visible errors.
    fn provider_error_message(&self, status: reqwest::StatusCode, body: &[u8]) -> String {
        if self.defaults.provider.is_openai_codex() {
            codex_errors::response_message(body)
        } else if self.defaults.provider.is_kimi_code() {
            kimi_code::response_message(status, body)
        } else if self.defaults.provider.is_zai_coding_plan() {
            zai_coding_plan::response_message(status, body)
        } else {
            parse_error_bytes(body)
        }
    }

    fn endpoint(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }

    fn apply_defaults(&self, mut request: ChatCompletionRequest) -> Result<ChatCompletionRequest> {
        if request.model.is_none() {
            request.model = Some(self.defaults.model.clone());
        }

        if request.max_tokens.is_none() {
            request.max_tokens = self.defaults.max_completion_tokens;
        }

        if request.temperature.is_none() {
            request.temperature = self.defaults.temperature;
        }

        if request.top_p.is_none() {
            request.top_p = self.defaults.top_p;
        }

        if let Some(effort) = request.reasoning_effort {
            request.reasoning_effort = Some(normalize_reasoning_effort_for_provider(
                self.defaults.provider,
                effort,
            )?);
        }

        Ok(request)
    }

    async fn handle_response(
        &self,
        response: reqwest::Response,
        credential_binding: Option<CredentialBinding>,
    ) -> Result<ChatCompletionResponse> {
        let status = response.status();
        let model_metadata = extract_model_metadata(
            response.headers(),
            self.defaults.provider,
            credential_binding.as_ref(),
        );
        let retry_after_secs = extract_retry_after(response.headers());
        let should_retry = extract_should_retry(response.headers());
        let bytes = if self.defaults.provider.is_zai_coding_plan() {
            zai_coding_plan::read_limited_response_body(response).await?
        } else {
            response.bytes().await?
        };

        if self.defaults.provider.is_zai_coding_plan()
            && let Some(error) = zai_coding_plan::business_error(bytes.as_ref())
        {
            if matches!(
                error,
                SamplingError::Api {
                    status: reqwest::StatusCode::UNAUTHORIZED,
                    ..
                }
            ) {
                self.record_401_attribution(crate::attribution::SamplingConsumer::ChatCompletions);
            }
            return Err(error);
        }

        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.record_401_attribution(crate::attribution::SamplingConsumer::ChatCompletions);
                let server_message = self.provider_error_message(status, bytes.as_ref());
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401): {server_message}"
                )));
            }
            let message = self.provider_error_message(status, bytes.as_ref());
            let should_retry = if self.defaults.provider.is_kimi_code() {
                kimi_code::should_retry(status, bytes.as_ref(), should_retry)
            } else if self.defaults.provider.is_zai_coding_plan() {
                zai_coding_plan::should_retry(status, bytes.as_ref(), should_retry)
            } else {
                should_retry
            };
            return Err(SamplingError::Api {
                status,
                message,
                model_metadata,
                retry_after_secs,
                should_retry,
            });
        }

        let completion = if self.defaults.provider.is_kimi_code() {
            kimi_code::deserialize_chat_response(&bytes).map_err(|error| {
                tracing::error!(
                    error_kind = %error,
                    provider = %self.defaults.provider,
                    body_len = bytes.len(),
                    "Failed to deserialize provider ChatCompletionResponse; body omitted"
                );
                error
            })?
        } else {
            serde_json::from_slice::<ChatCompletionResponse>(&bytes).map_err(|error| {
                tracing::error!(
                    error = %error,
                    provider = %self.defaults.provider,
                    body_len = bytes.len(),
                    "Failed to deserialize ChatCompletionResponse; body omitted"
                );
                SamplingError::Serialization(error)
            })?
        };
        Ok(completion)
    }

    // =========================================================================
    // Chat Completions API
    // =========================================================================

    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let payload = self.apply_defaults(request)?;
        let x_grok_conv_id = &payload.x_grok_conv_id.clone().unwrap_or_default();
        let x_grok_req_id = &payload.x_grok_req_id.clone().unwrap_or_default();
        let model_id = payload.model.clone().unwrap_or_default();

        tracing::debug!(
            base_url = %self.base_url,
            model_id = %model_id,
            "Sending chat completion request"
        );

        let grok_headers = GrokRequestHeaders {
            conv_id: x_grok_conv_id,
            req_id: x_grok_req_id,
            model_id: &model_id,
            session_id: payload.x_grok_session_id.as_deref().unwrap_or_default(),
            turn_idx: payload.x_grok_turn_idx.as_deref(),
            agent_id: payload.x_grok_agent_id.as_deref().unwrap_or_default(),
            deployment_id: payload.x_grok_deployment_id.as_deref(),
            user_id: payload.x_grok_user_id.as_deref(),
        };
        let endpoint = if self.defaults.provider.is_kimi_code() {
            kimi_code::endpoint(&self.base_url, ApiBackend::ChatCompletions)?
        } else if self.defaults.provider.is_zai_coding_plan() {
            zai_coding_plan::endpoint(&self.base_url)?
        } else {
            self.endpoint("chat/completions")
        };
        let prepared = self.post(endpoint)?;
        let request_credential = prepared.credential_binding;
        let request_builder =
            self.apply_provider_request_headers(prepared.builder, &grok_headers)?;
        let http_request = if self.defaults.provider.is_kimi_code() {
            request_builder.json(&kimi_code::chat_body(&payload, false)?)
        } else if self.defaults.provider.is_zai_coding_plan() {
            request_builder.json(&zai_coding_plan::chat_body(&payload, false)?)
        } else {
            request_builder.json(&payload)
        };

        let response = http_request.send().await.map_err(|e| {
            // Log at debug level; errors are surfaced to the caller.
            tracing::debug!("HTTP request failed: {}", e);
            e
        })?;

        self.handle_response(response, request_credential).await
    }

    /// Start a streaming chat completion request. Returns a stream of typed chunks.
    #[tracing::instrument(
        name = "http.chat_completion_stream",
        skip_all,
        fields(
            endpoint = %self.endpoint("chat/completions"),
            model_id = request.model.as_deref().unwrap_or(""),
            status_code = tracing::field::Empty,
            success = tracing::field::Empty,
            error = tracing::field::Empty,
        )
    )]
    pub async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<(
        BoxStream<'static, Result<ChatCompletionChunk>>,
        Option<ResponseModelMetadata>,
    )> {
        let payload = self.apply_defaults(request)?;
        let x_grok_conv_id = &payload.x_grok_conv_id.clone().unwrap_or_default();
        let x_grok_req_id = &payload.x_grok_req_id.clone().unwrap_or_default();
        let model_id = payload.model.clone().unwrap_or_default();

        // Wrap the request with streaming fields and serialize once.
        // Previously this path serialized twice: first to serde_json::Value
        // (to inject `stream` and `stream_options`), then to HTTP body bytes.
        let streaming_request = StreamingChatRequest {
            inner: &payload,
            stream: true,
            stream_options: StreamOptions {
                include_usage: true,
            },
        };

        let grok_headers = GrokRequestHeaders {
            conv_id: x_grok_conv_id,
            req_id: x_grok_req_id,
            model_id: &model_id,
            session_id: payload.x_grok_session_id.as_deref().unwrap_or_default(),
            turn_idx: payload.x_grok_turn_idx.as_deref(),
            agent_id: payload.x_grok_agent_id.as_deref().unwrap_or_default(),
            deployment_id: payload.x_grok_deployment_id.as_deref(),
            user_id: payload.x_grok_user_id.as_deref(),
        };
        let endpoint = if self.defaults.provider.is_kimi_code() {
            kimi_code::endpoint(&self.base_url, ApiBackend::ChatCompletions)?
        } else if self.defaults.provider.is_zai_coding_plan() {
            zai_coding_plan::endpoint(&self.base_url)?
        } else {
            self.endpoint("chat/completions")
        };
        let prepared = self.post(endpoint)?;
        let request_credential = prepared.credential_binding;
        let request_builder = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .header(ACCEPT, HeaderValue::from_static("text/event-stream"));
        let http_request = if self.defaults.provider.is_kimi_code() {
            request_builder.json(&kimi_code::chat_body(&payload, true)?)
        } else if self.defaults.provider.is_zai_coding_plan() {
            request_builder.json(&zai_coding_plan::chat_body(&payload, true)?)
        } else {
            request_builder.json(&streaming_request)
        };

        let built_request = http_request.build().map_err(|e| {
            tracing::error!("Failed to build HTTP request: {}", e);
            SamplingError::Http(e)
        })?;

        tracing::debug!(
            url = %built_request.url(),
            method = %built_request.method(),
            "Sending chat/completions request"
        );
        Self::log_request_diagnostics(&built_request, "chat/completions");

        let response = self.http.execute(built_request).await.map_err(|e| {
            tracing::debug!("HTTP request failed: {}", e);
            record_stream_request_failure(&e);
            e
        })?;

        let status = response.status();
        let span = tracing::Span::current();
        span.record("status_code", status.as_u16() as i64);
        span.record("success", status.is_success());
        if self.defaults.provider.is_zai_coding_plan()
            && status.is_success()
            && response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_none_or(|value| !value.to_ascii_lowercase().contains("text/event-stream"))
        {
            let bytes = zai_coding_plan::read_limited_response_body(response).await?;
            if let Some(error) = zai_coding_plan::business_error(bytes.as_ref()) {
                return Err(error);
            }
            return Err(SamplingError::InvalidConfiguration(
                "Z.AI Coding Plan streaming response was not an event stream",
            ));
        }
        let model_metadata = extract_model_metadata(
            response.headers(),
            self.defaults.provider,
            request_credential.as_ref(),
        );
        let retry_after_secs = extract_retry_after(response.headers());
        let should_retry = extract_should_retry(response.headers());
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                span.record("error", "unauthorized (401)");
                self.record_401_attribution(
                    crate::attribution::SamplingConsumer::ChatCompletionsStream,
                );
                let endpoint = if self.defaults.provider.is_kimi_code() {
                    kimi_code::endpoint(&self.base_url, ApiBackend::ChatCompletions)?
                } else {
                    self.endpoint("chat/completions")
                };
                let bytes = if self.defaults.provider.is_zai_coding_plan() {
                    zai_coding_plan::read_limited_response_body(response)
                        .await
                        .unwrap_or_default()
                } else {
                    response.bytes().await.unwrap_or_default()
                };
                let server_message = self.provider_error_message(status, bytes.as_ref());
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401) from {endpoint}: {server_message}"
                )));
            }

            let req_headers = self.request_header_diagnostics(true);
            let resp_headers =
                Self::format_response_headers(response.headers(), self.defaults.provider);
            let bytes = if self.defaults.provider.is_zai_coding_plan() {
                zai_coding_plan::read_limited_response_body(response).await?
            } else {
                response.bytes().await?
            };
            let should_retry = if self.defaults.provider.is_kimi_code() {
                kimi_code::should_retry(status, bytes.as_ref(), should_retry)
            } else if self.defaults.provider.is_zai_coding_plan() {
                zai_coding_plan::should_retry(status, bytes.as_ref(), should_retry)
            } else {
                should_retry
            };
            let server_message = self.provider_error_message(status, bytes.as_ref());

            let message = self.build_api_error_message(
                status,
                &server_message,
                &self.endpoint("chat/completions"),
                &req_headers,
                Some(&resp_headers),
            );

            span.record("error", message.as_str());
            tracing::error!(
                status = %status,
                error_message = %message,
                model_id = %model_id,
                "chat/completions API error"
            );
            return Err(SamplingError::Api {
                status,
                message,
                model_metadata,
                retry_after_secs,
                should_retry,
            });
        }

        // Strip UTF-8 BOM if present: eventsource-stream 0.2.3 incorrectly slices BOM at byte 1 instead of 3.
        const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
        let mut is_first = true;
        let byte_stream = response.bytes_stream().map(move |result| {
            result.map(|bytes| {
                if is_first {
                    is_first = false;
                    if bytes.starts_with(UTF8_BOM) {
                        return bytes.slice(UTF8_BOM.len()..);
                    }
                }
                bytes
            })
        });

        // Turn raw bytes into SSE events
        let event_stream = byte_stream.eventsource();

        // Map SSE events into ChatCompletionChunk.
        // Uses `scan` so that `[DONE]` and transport errors both terminate the
        // stream (`None`). The first transport error is emitted to the consumer,
        // then subsequent polls return `None` -- preventing an infinite busy-loop
        // when the HTTP/2 connection drops and h2 keeps producing errors.
        let stream_provider = self.defaults.provider;
        let chunks = event_stream
            .scan(false, move |had_transport_error, event_res| {
                if *had_transport_error {
                    return std::future::ready(None);
                }
                let item = match event_res {
                    Ok(event) => {
                        let data = &event.data;
                        if data == "[DONE]" {
                            return std::future::ready(None);
                        }

                        if stream_provider.requires_redacted_provider_diagnostics() {
                            tracing::info!(
                                target: crate::sampling_log::TARGET,
                                event = "sse_chunk",
                                backend = "chat_completions",
                                provider = %stream_provider,
                                data_len = data.len(),
                                "provider SSE payload omitted"
                            );
                        } else {
                            tracing::info!(
                                target: crate::sampling_log::TARGET,
                                event = "sse_chunk",
                                backend = "chat_completions",
                                body_len = data.len(),
                            );
                        }

                        if stream_provider.is_zai_coding_plan()
                            && let Some(stream_error) = zai_coding_plan::stream_error(data)
                        {
                            Some(Err(stream_error))
                        } else if stream_provider.is_kimi_code()
                            && let Some(stream_error) = kimi_code::stream_error(data)
                        {
                            Some(Err(stream_error))
                        } else if let Some(stream_error) = try_parse_stream_error(data) {
                            if stream_provider.requires_redacted_provider_diagnostics() {
                                Some(Err(SamplingError::EventStreamError(format!(
                                    "{stream_provider} event stream failed"
                                ))))
                            } else {
                                Some(Err(stream_error))
                            }
                        } else if stream_provider.is_kimi_code() {
                            Some(kimi_code::deserialize_chat_chunk(data).map_err(|error| {
                                tracing::error!(
                                    error_kind = %error,
                                    provider = %stream_provider,
                                    data_len = data.len(),
                                    "Failed to deserialize provider ChatCompletionChunk; payload omitted"
                                );
                                error
                            }))
                        } else {
                            Some(
                                serde_json::from_str::<ChatCompletionChunk>(data).map_err(|e| {
                                    if stream_provider.requires_redacted_provider_diagnostics() {
                                        tracing::error!(
                                            error = %e,
                                            provider = %stream_provider,
                                            data_len = data.len(),
                                            "Failed to deserialize provider ChatCompletionChunk; payload omitted"
                                        );
                                    } else {
                                        tracing::error!(
                                            error = %e,
                                            body_len = data.len(),
                                            "Failed to deserialize ChatCompletionChunk; body omitted"
                                        );
                                    }
                                    SamplingError::Serialization(e)
                                }),
                            )
                        }
                    }
                    Err(e) => {
                        *had_transport_error = true;
                        let message = if stream_provider.requires_redacted_provider_diagnostics() {
                            format!("{stream_provider} event stream transport failed")
                        } else {
                            e.to_string()
                        };
                        Some(Err(SamplingError::EventStreamError(message)))
                    }
                };
                std::future::ready(item)
            })
            .boxed();

        Ok((chunks, model_metadata))
    }

    // =========================================================================
    // Responses API
    // =========================================================================

    /// Apply default configuration to a Responses API request.
    fn apply_response_defaults(&self, request: &mut CreateResponseWrapper) -> Result<()> {
        // Apply model default if not specified
        if request.inner.model.is_none() {
            request.inner.model = Some(self.defaults.model.clone());
        }

        // Apply temperature default if not specified
        if request.inner.temperature.is_none() {
            request.inner.temperature = self.defaults.temperature;
        }

        // Apply top_p default if not specified
        if request.inner.top_p.is_none() {
            request.inner.top_p = self.defaults.top_p;
        }

        // Apply max_output_tokens default if not specified
        if request.inner.max_output_tokens.is_none() {
            request.inner.max_output_tokens = self.defaults.max_completion_tokens;
        }

        // Fast mode is a provider-scoped Codex service tier. `default` is the
        // persisted explicit-Standard sentinel used by the shell and must not
        // appear in a Responses payload. Never project this setting onto xAI
        // or custom-provider traffic.
        if request.inner.service_tier.is_none() && self.defaults.provider == ProviderId::OpenAiCodex
        {
            request.inner.service_tier =
                codex_responses::service_tier(self.defaults.service_tier.as_deref());
        }

        // Set store to false if not specified (default is true, but that breaks ZDR compliance)
        if request.inner.store.is_none() {
            request.inner.store = Some(false);
        }

        // Include encrypted reasoning content if not specified
        let includes = request.inner.include.get_or_insert_with(Vec::new);
        if !includes.contains(&rs::IncludeEnum::ReasoningEncryptedContent) {
            includes.push(rs::IncludeEnum::ReasoningEncryptedContent);
        }

        Ok(())
    }

    /// Create a response using the Responses API (non-streaming).
    ///
    /// This uses the Responses API format which provides a simpler interface
    /// for multi-turn conversations and tool calling.
    pub async fn create_response(
        &self,
        mut request: CreateResponseWrapper,
    ) -> Result<rs::Response> {
        self.apply_response_defaults(&mut request)?;

        let x_grok_conv_id = request.x_grok_conv_id.as_deref().unwrap_or_default();
        let x_grok_req_id = request.x_grok_req_id.as_deref().unwrap_or_default();
        let model_id = request.inner.model.clone().unwrap_or_default();

        // The trace field is process-local: it is consumed by upstream
        // session code (which may upload a payload artifact) and is not
        // forwarded by the sampler. Drop it before we send.
        request.trace.take();

        tracing::debug!(
            provider = ?self.defaults.provider,
            model = %model_id,
            endpoint = %self.endpoint("responses"),
            "responses request prepared"
        );

        let grok_headers = GrokRequestHeaders {
            conv_id: x_grok_conv_id,
            req_id: x_grok_req_id,
            model_id: &model_id,
            session_id: request.x_grok_session_id.as_deref().unwrap_or_default(),
            turn_idx: request.x_grok_turn_idx.as_deref(),
            agent_id: request.x_grok_agent_id.as_deref().unwrap_or_default(),
            deployment_id: request.x_grok_deployment_id.as_deref(),
            user_id: request.x_grok_user_id.as_deref(),
        };
        let mut request_body = serde_json::to_value(&request.inner).map_err(|e| {
            tracing::error!("Failed to serialize responses request: {}", e);
            SamplingError::Serialization(e)
        })?;
        codex_responses::apply_extended_codex_reasoning_effort(
            self.defaults.provider,
            request.wire_reasoning_effort,
            &mut request_body,
        )?;
        codex_responses::apply_codex_reasoning_summary(
            self.defaults.provider,
            self.defaults.supports_reasoning_summary_parameter,
            self.defaults.default_reasoning_summary.as_deref(),
            &mut request_body,
        )?;
        codex_responses::apply_codex_responses_lite_contract(
            self.defaults.provider,
            self.defaults.responses_lite,
            codex_responses::codex_prompt_cache_key(
                x_grok_conv_id,
                request.x_grok_session_id.as_deref().unwrap_or_default(),
            ),
            &mut request_body,
        )?;
        // async-openai's ReasoningTextContent struct omits the `type`
        // discriminator that the Responses API requires on input. Patch
        // it in post-serialize. This is the last surviving piece of the
        // old raw_output machinery.
        xai_grok_sampling_types::patch_reasoning_text_types(&mut request_body);
        let prepared = self.post(self.endpoint("responses"))?;
        let request_credential = prepared.credential_binding;
        let http_request = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .json(&request_body);

        let response = http_request.send().await.map_err(|e| {
            tracing::debug!("HTTP request failed: {}", e);
            e
        })?;

        let status = response.status();
        let model_metadata = extract_model_metadata(
            response.headers(),
            self.defaults.provider,
            request_credential.as_ref(),
        );
        let retry_after_secs = extract_retry_after(response.headers());
        let should_retry = extract_should_retry(response.headers());
        if status.is_success() {
            self.capture_codex_turn_state(response.headers(), x_grok_req_id);
        }
        let bytes = response.bytes().await?;
        let should_retry = codex_errors::usage_limit_should_retry(
            self.defaults.provider,
            status,
            bytes.as_ref(),
            should_retry,
        );

        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.record_401_attribution(crate::attribution::SamplingConsumer::Responses);
                if self.defaults.provider.is_openai_codex() {
                    let credential =
                        request_credential.ok_or(SamplingError::InvalidConfiguration(
                            codex_headers::CODEX_AUTH_CONFIGURATION_INVALID,
                        ))?;
                    return Err(SamplingError::ProviderAuthRejected {
                        provider: ProviderId::OpenAiCodex,
                        credential,
                    });
                }
                let endpoint = self.endpoint("responses");
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401) from {endpoint}"
                )));
            }

            let req_headers = self.request_header_diagnostics(false);
            let server_message = self.provider_error_message(status, bytes.as_ref());

            let message = self.build_api_error_message(
                status,
                &server_message,
                &self.endpoint("responses"),
                &req_headers,
                None,
            );
            tracing::warn!(
                status = %status,
                error_message = %message,
                model_id = %model_id,
                "responses API error"
            );
            return Err(SamplingError::Api {
                status,
                message,
                model_metadata,
                retry_after_secs,
                should_retry,
            });
        }

        let response_obj = deserialize_response_for_provider(&bytes, self.defaults.provider)?;
        Ok(response_obj)
    }

    /// Create a streaming response using the Responses API.
    ///
    /// Returns a stream of `rs::ResponseStreamEvent` which includes events like:
    /// - `response.created` - Initial response object
    /// - `response.output_text.delta` - Text content deltas
    /// - `response.function_call_arguments.delta` - Function call argument deltas
    /// - `response.completed` - Final response with all output
    ///
    /// The third tuple element is a per-request doom-loop signal collector,
    /// `Some` only when `SamplerConfig::doom_loop_recovery` is set — the same
    /// gate that adds the opt-in `x-grok-doom-loop-check` request header, so
    /// header and parse protection cannot drift apart. It is filled by the
    /// SSE decoder as the server reports triggers and is meant to be handed
    /// to `stream_responses` so the signals land on the final
    /// `ConversationResponse`.
    #[allow(clippy::type_complexity)]
    pub async fn create_response_stream(
        &self,
        request: CreateResponseWrapper,
    ) -> Result<(
        BoxStream<'static, Result<rs::ResponseStreamEvent>>,
        Option<ResponseModelMetadata>,
        Option<crate::doom_loop::DoomLoopSignalCollector>,
    )> {
        self.create_response_stream_after_provider_auth_recovery(request, None)
            .await
    }

    #[tracing::instrument(
        name = "http.create_response_stream",
        skip_all,
        fields(
            endpoint = %self.endpoint("responses"),
            model_id = request.inner.model.as_deref().unwrap_or(""),
            status_code = tracing::field::Empty,
            success = tracing::field::Empty,
            error = tracing::field::Empty,
        )
    )]
    #[allow(clippy::type_complexity)]
    async fn create_response_stream_after_provider_auth_recovery(
        &self,
        mut request: CreateResponseWrapper,
        rejected: Option<&CredentialBinding>,
    ) -> Result<(
        BoxStream<'static, Result<rs::ResponseStreamEvent>>,
        Option<ResponseModelMetadata>,
        Option<crate::doom_loop::DoomLoopSignalCollector>,
    )> {
        self.apply_response_defaults(&mut request)?;

        // Enable streaming
        request.inner.stream = Some(true);

        let x_grok_conv_id = request.x_grok_conv_id.as_deref().unwrap_or_default();
        let x_grok_req_id = request.x_grok_req_id.as_deref().unwrap_or_default();
        let model_id = request.inner.model.clone().unwrap_or_default();

        // Drop process-local trace data (see note in `create_response`).
        request.trace.take();

        tracing::debug!(
            base_url = %self.base_url,
            model_id = model_id.as_str(),
            "Sending responses API stream request"
        );

        let grok_headers = GrokRequestHeaders {
            conv_id: x_grok_conv_id,
            req_id: x_grok_req_id,
            model_id: &model_id,
            session_id: request.x_grok_session_id.as_deref().unwrap_or_default(),
            turn_idx: request.x_grok_turn_idx.as_deref(),
            agent_id: request.x_grok_agent_id.as_deref().unwrap_or_default(),
            deployment_id: request.x_grok_deployment_id.as_deref(),
            user_id: request.x_grok_user_id.as_deref(),
        };
        let extra_raw_tools = std::mem::take(&mut request.extra_raw_tools);
        let mut request_body = serde_json::to_value(&request.inner).map_err(|e| {
            tracing::error!("Failed to serialize responses request: {}", e);
            SamplingError::Serialization(e)
        })?;
        codex_responses::apply_extended_codex_reasoning_effort(
            self.defaults.provider,
            request.wire_reasoning_effort,
            &mut request_body,
        )?;
        codex_responses::apply_codex_reasoning_summary(
            self.defaults.provider,
            self.defaults.supports_reasoning_summary_parameter,
            self.defaults.default_reasoning_summary.as_deref(),
            &mut request_body,
        )?;
        // Inject xAI-specific fields not in async-openai's CreateResponse type.
        if !self.defaults.provider.is_openai_codex() && self.defaults.stream_tool_calls {
            request_body["stream_tool_calls"] = serde_json::json!(true);
        }
        // Inject xAI-specific tools (e.g., x_search) that can't be expressed
        // via async_openai's rs::Tool enum.
        if !self.defaults.provider.is_openai_codex() && !extra_raw_tools.is_empty() {
            if let Some(tools) = request_body.get_mut("tools").and_then(|v| v.as_array_mut()) {
                tools.extend(extra_raw_tools);
            } else {
                request_body["tools"] = serde_json::Value::Array(extra_raw_tools);
            }
        }
        codex_responses::apply_codex_responses_lite_contract(
            self.defaults.provider,
            self.defaults.responses_lite,
            codex_responses::codex_prompt_cache_key(
                x_grok_conv_id,
                request.x_grok_session_id.as_deref().unwrap_or_default(),
            ),
            &mut request_body,
        )?;
        xai_grok_sampling_types::patch_reasoning_text_types(&mut request_body);
        // Fresh per attempt so signals never leak across retries; `None`
        // (check disabled) sends no header and does no peek work per event.
        let doom_loop = self
            .defaults
            .doom_loop_recovery
            .filter(|_| !self.defaults.provider.is_openai_codex())
            .map(crate::doom_loop::DoomLoopSignalCollector::new);
        let prepared =
            self.post_after_provider_auth_recovery(self.endpoint("responses"), rejected)?;
        let request_credential = prepared.credential_binding;
        let mut http_request = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .header(ACCEPT, HeaderValue::from_static("text/event-stream"));
        if doom_loop.is_some()
            && self.defaults.provider == ProviderId::Xai
            && self.defaults.xai_trusted_origin
        {
            // Presence opts in; the server ignores the value. This xAI header
            // is never projected onto custom or subscription providers.
            http_request = http_request.header(DOOM_LOOP_CHECK_HEADER, "true");
        }
        let http_request = http_request.json(&request_body);

        let built_request = http_request.build().map_err(|e| {
            tracing::error!("Failed to build HTTP request: {}", e);
            SamplingError::Http(e)
        })?;

        tracing::debug!(
            url = %built_request.url(),
            method = %built_request.method(),
            "Sending responses API stream request"
        );
        Self::log_request_diagnostics(&built_request, "responses");

        let response = self.http.execute(built_request).await.map_err(|e| {
            tracing::debug!("HTTP request failed: {}", e);
            record_stream_request_failure(&e);
            e
        })?;

        let status = response.status();
        let span = tracing::Span::current();
        span.record("status_code", status.as_u16() as i64);
        span.record("success", status.is_success());
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                span.record("error", "unauthorized (401)");
                self.record_401_attribution(crate::attribution::SamplingConsumer::ResponsesStream);
                if self.defaults.provider.is_openai_codex() {
                    let credential =
                        request_credential.ok_or(SamplingError::InvalidConfiguration(
                            codex_headers::CODEX_AUTH_CONFIGURATION_INVALID,
                        ))?;
                    return Err(SamplingError::ProviderAuthRejected {
                        provider: ProviderId::OpenAiCodex,
                        credential,
                    });
                }
                let endpoint = self.endpoint("responses");
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401) from {endpoint}"
                )));
            }
            let model_metadata = extract_model_metadata(
                response.headers(),
                self.defaults.provider,
                request_credential.as_ref(),
            );
            let retry_after_secs = extract_retry_after(response.headers());
            let should_retry = extract_should_retry(response.headers());
            let req_headers = self.request_header_diagnostics(true);
            let resp_headers =
                Self::format_response_headers(response.headers(), self.defaults.provider);
            let bytes = response.bytes().await?;
            let should_retry = codex_errors::usage_limit_should_retry(
                self.defaults.provider,
                status,
                bytes.as_ref(),
                should_retry,
            );
            let server_message = self.provider_error_message(status, bytes.as_ref());

            let message = self.build_api_error_message(
                status,
                &server_message,
                &self.endpoint("responses"),
                &req_headers,
                Some(&resp_headers),
            );

            span.record("error", message.as_str());
            tracing::error!(
                status = %status,
                error_message = %message,
                model_id = %model_id,
                "responses API error"
            );
            return Err(SamplingError::Api {
                status,
                message,
                model_metadata,
                retry_after_secs,
                should_retry,
            });
        }

        self.capture_codex_turn_state(response.headers(), x_grok_req_id);

        let model_metadata = extract_model_metadata(
            response.headers(),
            self.defaults.provider,
            request_credential.as_ref(),
        );

        // Strip UTF-8 BOM if present
        const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
        let mut is_first = true;
        let byte_stream = response.bytes_stream().map(move |result| {
            result.map(|bytes| {
                if is_first {
                    is_first = false;
                    if bytes.starts_with(UTF8_BOM) {
                        return bytes.slice(UTF8_BOM.len()..);
                    }
                }
                bytes
            })
        });

        // Turn raw bytes into SSE events
        let event_stream = byte_stream.eventsource();

        let doom_loop_for_stream = doom_loop.clone();
        let redact_provider_payload = self.defaults.provider.is_openai_codex();
        let codex_decoder = redact_provider_payload.then(|| CodexSseDecoder::new(model_id.clone()));

        // The scan item is an `Option`: `Some(None)` skips an absorbed
        // doom-loop event without terminating the stream (`filter_map`
        // below), while an outer `None` still ends it.
        let events = event_stream
            .scan((false, codex_decoder), move |state, event_res| {
                if state.0 {
                    return std::future::ready(None);
                }
                let item = match event_res {
                    Ok(event) => {
                        let data = &event.data;
                        if data == "[DONE]" {
                            return std::future::ready(None);
                        }

                        if redact_provider_payload {
                            tracing::info!(
                                target: crate::sampling_log::TARGET,
                                event = "sse_chunk",
                                backend = "responses",
                                provider = "openai_codex",
                            );
                        } else {
                            tracing::info!(
                                target: crate::sampling_log::TARGET,
                                event = "sse_chunk",
                                backend = "responses",
                                body_len = data.len(),
                            );
                        }

                        // Intercept the non-standard doom-loop event before
                        // typed deserialization; async-openai's event enum
                        // does not know it and would fail to parse it. With
                        // the check disabled, the shared name-or-payload-type
                        // predicate guards against a server emitting it
                        // despite no opt-in (rollout skew), named or not.
                        let swallow = match &doom_loop_for_stream {
                            Some(collector) => collector.absorb(&event.event, data),
                            None => is_check_event(&event.event, data),
                        };
                        if swallow {
                            Some(None)
                        } else if redact_provider_payload && data.trim().is_empty() {
                            // The official Codex fixture stream occasionally
                            // carries type-only SSE heartbeats with no data
                            // payload. They are forward-compatible liveness
                            // frames, not malformed model output.
                            Some(None)
                        } else if let Some(stream_error) = if redact_provider_payload {
                            codex_errors::try_parse_stream_error(data)
                        } else {
                            try_parse_stream_error(data)
                        } {
                            Some(Some(Err(stream_error)))
                        } else if let Some(decoder) = state.1.as_mut() {
                            match decoder.decode(data) {
                                Ok(Some(event)) => Some(Some(Ok(event))),
                                Ok(None) => Some(None),
                                Err(error) => Some(Some(Err(error))),
                            }
                        } else {
                            Some(Some(deserialize_response_event(data)))
                        }
                    }
                    Err(e) => {
                        state.0 = true;
                        let message = if redact_provider_payload {
                            "ChatGPT Codex event stream failed".to_string()
                        } else {
                            e.to_string()
                        };
                        Some(Some(Err(SamplingError::EventStreamError(message))))
                    }
                };
                std::future::ready(item)
            })
            .filter_map(std::future::ready)
            .boxed();

        Ok((events, model_metadata, doom_loop))
    }

    // =========================================================================
    // Anthropic Messages API
    // =========================================================================

    /// Apply default configuration to a Messages API request.
    fn apply_message_defaults(&self, request: &mut MessagesRequestWrapper) -> Result<()> {
        // Apply model default if not specified
        if request.inner.model.is_empty() {
            request.inner.model = self.defaults.model.clone();
        }

        if request.inner.max_tokens == 0 {
            request.inner.max_tokens = self
                .defaults
                .max_completion_tokens
                .unwrap_or(ANTHROPIC_DEFAULT_MAX_TOKENS);
        }

        // Apply temperature default if not specified
        if request.inner.temperature.is_none() {
            request.inner.temperature = self.defaults.temperature;
        }

        // Apply top_p default if not specified
        if request.inner.top_p.is_none() {
            request.inner.top_p = self.defaults.top_p;
        }

        Ok(())
    }

    /// Create a message using the Anthropic Messages API (non-streaming).
    pub async fn create_message(
        &self,
        mut request: MessagesRequestWrapper,
    ) -> Result<messages::MessagesResponse> {
        self.apply_message_defaults(&mut request)?;

        let x_grok_conv_id = request.x_grok_conv_id.as_deref().unwrap_or_default();
        let x_grok_req_id = request.x_grok_req_id.as_deref().unwrap_or_default();
        let model_id = request.inner.model.clone();

        // Drop process-local trace data.
        request.trace.take();

        tracing::debug!(
            provider = ?self.defaults.provider,
            model = %model_id,
            endpoint = %self.endpoint("messages"),
            "messages request prepared"
        );

        let grok_headers = GrokRequestHeaders {
            conv_id: x_grok_conv_id,
            req_id: x_grok_req_id,
            model_id: &model_id,
            session_id: request.x_grok_session_id.as_deref().unwrap_or_default(),
            turn_idx: request.x_grok_turn_idx.as_deref(),
            agent_id: request.x_grok_agent_id.as_deref().unwrap_or_default(),
            deployment_id: request.x_grok_deployment_id.as_deref(),
            user_id: request.x_grok_user_id.as_deref(),
        };
        let endpoint = if self.defaults.provider.is_kimi_code() {
            kimi_code::endpoint(&self.base_url, ApiBackend::Messages)?
        } else {
            self.endpoint("messages")
        };
        let prepared = self.post(endpoint.clone())?;
        let request_credential = prepared.credential_binding;
        let request_builder =
            self.apply_provider_request_headers(prepared.builder, &grok_headers)?;
        let http_request = if self.defaults.provider.is_kimi_code() {
            let mut body =
                serde_json::to_value(&request.inner).map_err(SamplingError::Serialization)?;
            let cache_affinity = kimi_code::messages_cache_affinity(
                Some(x_grok_conv_id),
                request.x_grok_session_id.as_deref(),
                &model_id,
                request.wire_reasoning_effort,
            );
            kimi_code::shape_messages_body(
                &mut body,
                request.wire_reasoning_effort,
                cache_affinity.as_deref(),
            )?;
            request_builder.json(&body)
        } else {
            request_builder.json(&request.inner)
        };

        let response = http_request.send().await.map_err(|e| {
            tracing::debug!("HTTP request failed: {}", e);
            e
        })?;

        let status = response.status();
        let model_metadata = extract_model_metadata(
            response.headers(),
            self.defaults.provider,
            request_credential.as_ref(),
        );
        let retry_after_secs = extract_retry_after(response.headers());
        let should_retry = extract_should_retry(response.headers());
        let bytes = response.bytes().await?;

        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.record_401_attribution(crate::attribution::SamplingConsumer::Messages);
                let server_message = self.provider_error_message(status, bytes.as_ref());
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401) from {endpoint}: {server_message}"
                )));
            }

            let req_headers = self.request_header_diagnostics(false);
            let should_retry = if self.defaults.provider.is_kimi_code() {
                kimi_code::should_retry(status, bytes.as_ref(), should_retry)
            } else if self.defaults.provider.is_zai_coding_plan() {
                zai_coding_plan::should_retry(status, bytes.as_ref(), should_retry)
            } else {
                should_retry
            };
            let server_message = self.provider_error_message(status, bytes.as_ref());

            let message = self.build_api_error_message(
                status,
                &server_message,
                &endpoint,
                &req_headers,
                None,
            );
            tracing::warn!(
                status = %status,
                error_message = %message,
                model_id = %model_id,
                "messages API error"
            );
            return Err(SamplingError::Api {
                status,
                message,
                model_metadata,
                retry_after_secs,
                should_retry,
            });
        }

        let response_obj =
            serde_json::from_slice::<messages::MessagesResponse>(&bytes).map_err(|e| {
                tracing::error!(
                    error = %e,
                    provider = %self.defaults.provider,
                    body_len = bytes.len(),
                    "Failed to deserialize MessagesResponse; body omitted"
                );
                SamplingError::Serialization(e)
            })?;
        Ok(response_obj)
    }

    /// Create a streaming message using the Anthropic Messages API.
    ///
    /// Returns a stream of `MessageStreamEvent` which includes events like:
    /// - `message_start` - Initial message object
    /// - `content_block_start` / `content_block_delta` / `content_block_stop` - Content blocks
    /// - `message_delta` / `message_stop` - Final message with stop reason
    #[tracing::instrument(
        name = "http.create_message_stream",
        skip_all,
        fields(
            endpoint = %self.endpoint("messages"),
            model_id = request.inner.model.as_str(),
            status_code = tracing::field::Empty,
            success = tracing::field::Empty,
            error = tracing::field::Empty,
        )
    )]
    pub async fn create_message_stream(
        &self,
        mut request: MessagesRequestWrapper,
    ) -> Result<(
        BoxStream<'static, Result<messages::MessageStreamEvent>>,
        Option<ResponseModelMetadata>,
    )> {
        self.apply_message_defaults(&mut request)?;

        // Enable streaming
        request.inner.stream = Some(true);

        let x_grok_conv_id = request.x_grok_conv_id.as_deref().unwrap_or_default();
        let x_grok_req_id = request.x_grok_req_id.as_deref().unwrap_or_default();
        let model_id = request.inner.model.clone();

        // Drop process-local trace data.
        request.trace.take();

        tracing::debug!(
            base_url = %self.base_url,
            model_id = model_id.as_str(),
            "Sending Messages API stream request"
        );

        let grok_headers = GrokRequestHeaders {
            conv_id: x_grok_conv_id,
            req_id: x_grok_req_id,
            model_id: &model_id,
            session_id: request.x_grok_session_id.as_deref().unwrap_or_default(),
            turn_idx: request.x_grok_turn_idx.as_deref(),
            agent_id: request.x_grok_agent_id.as_deref().unwrap_or_default(),
            deployment_id: request.x_grok_deployment_id.as_deref(),
            user_id: request.x_grok_user_id.as_deref(),
        };
        let endpoint = if self.defaults.provider.is_kimi_code() {
            kimi_code::endpoint(&self.base_url, ApiBackend::Messages)?
        } else {
            self.endpoint("messages")
        };
        let prepared = self.post(endpoint.clone())?;
        let request_credential = prepared.credential_binding;
        let request_builder = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .header(ACCEPT, HeaderValue::from_static("text/event-stream"));
        let http_request = if self.defaults.provider.is_kimi_code() {
            let mut body =
                serde_json::to_value(&request.inner).map_err(SamplingError::Serialization)?;
            let cache_affinity = kimi_code::messages_cache_affinity(
                Some(x_grok_conv_id),
                request.x_grok_session_id.as_deref(),
                &model_id,
                request.wire_reasoning_effort,
            );
            kimi_code::shape_messages_body(
                &mut body,
                request.wire_reasoning_effort,
                cache_affinity.as_deref(),
            )?;
            request_builder.json(&body)
        } else {
            request_builder.json(&request.inner)
        };

        let built_request = http_request.build().map_err(|e| {
            tracing::error!("Failed to build HTTP request: {}", e);
            SamplingError::Http(e)
        })?;

        tracing::debug!(
            url = %built_request.url(),
            method = %built_request.method(),
            "Sending messages API stream request"
        );
        Self::log_request_diagnostics(&built_request, "messages");

        let response = self.http.execute(built_request).await.map_err(|e| {
            tracing::debug!("HTTP request failed: {}", e);
            record_stream_request_failure(&e);
            e
        })?;

        let status = response.status();
        let span = tracing::Span::current();
        span.record("status_code", status.as_u16() as i64);
        span.record("success", status.is_success());
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                span.record("error", "unauthorized (401)");
                self.record_401_attribution(crate::attribution::SamplingConsumer::MessagesStream);
                let bytes = response.bytes().await?;
                let server_message = self.provider_error_message(status, bytes.as_ref());
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401) from {endpoint}: {server_message}"
                )));
            }
            let model_metadata = extract_model_metadata(
                response.headers(),
                self.defaults.provider,
                request_credential.as_ref(),
            );
            let retry_after_secs = extract_retry_after(response.headers());
            let should_retry = extract_should_retry(response.headers());
            let req_headers = self.request_header_diagnostics(true);
            let resp_headers =
                Self::format_response_headers(response.headers(), self.defaults.provider);
            let bytes = response.bytes().await?;
            let should_retry = if self.defaults.provider.is_kimi_code() {
                kimi_code::should_retry(status, bytes.as_ref(), should_retry)
            } else if self.defaults.provider.is_zai_coding_plan() {
                zai_coding_plan::should_retry(status, bytes.as_ref(), should_retry)
            } else {
                should_retry
            };
            let server_message = self.provider_error_message(status, bytes.as_ref());

            let message = self.build_api_error_message(
                status,
                &server_message,
                &endpoint,
                &req_headers,
                Some(&resp_headers),
            );

            span.record("error", message.as_str());
            tracing::error!(
                status = %status,
                error_message = %message,
                model_id = %model_id,
                "messages API error"
            );
            return Err(SamplingError::Api {
                status,
                message,
                model_metadata,
                retry_after_secs,
                should_retry,
            });
        }

        let model_metadata = extract_model_metadata(
            response.headers(),
            self.defaults.provider,
            request_credential.as_ref(),
        );

        // Strip UTF-8 BOM if present
        const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
        let mut is_first = true;
        let byte_stream = response.bytes_stream().map(move |result| {
            result.map(|bytes| {
                if is_first {
                    is_first = false;
                    if bytes.starts_with(UTF8_BOM) {
                        return bytes.slice(UTF8_BOM.len()..);
                    }
                }
                bytes
            })
        });

        // Turn raw bytes into SSE events
        let event_stream = byte_stream.eventsource();

        // Map SSE events into MessageStreamEvent.
        // Uses `scan` so transport errors terminate the stream after the first
        // error (same pattern as `chat_completion_stream`).
        let stream_provider = self.defaults.provider;
        let events = event_stream
            .scan(false, move |had_transport_error, event_res| {
                if *had_transport_error {
                    return std::future::ready(None);
                }
                let item = match event_res {
                    Ok(event) => {
                        let data = &event.data;
                        if data == "[DONE]" {
                            return std::future::ready(None);
                        }

                        if !stream_provider.requires_redacted_provider_diagnostics() {
                            tracing::info!(
                                target: crate::sampling_log::TARGET,
                                event = "sse_chunk",
                                backend = "messages",
                                body_len = data.len(),
                            );
                        }

                        if stream_provider.is_kimi_code()
                            && let Some(stream_error) = kimi_code::stream_error(data)
                        {
                            Some(Err(stream_error))
                        } else if let Some(stream_error) = try_parse_stream_error(data) {
                            if stream_provider.requires_redacted_provider_diagnostics() {
                                Some(Err(SamplingError::EventStreamError(format!(
                                    "{stream_provider} event stream failed"
                                ))))
                            } else {
                                Some(Err(stream_error))
                            }
                        } else {
                            Some(
                                serde_json::from_str::<messages::MessageStreamEvent>(data).map_err(
                                    |e| {
                                        tracing::error!(
                                            error = %e,
                                            provider = %stream_provider,
                                            body_len = data.len(),
                                            "Failed to deserialize MessageStreamEvent; body omitted"
                                        );
                                        SamplingError::Serialization(e)
                                    },
                                ),
                            )
                        }
                    }
                    Err(e) => {
                        *had_transport_error = true;
                        let message = if stream_provider.requires_redacted_provider_diagnostics() {
                            format!("{stream_provider} event stream transport failed")
                        } else {
                            e.to_string()
                        };
                        Some(Err(SamplingError::EventStreamError(message)))
                    }
                };
                std::future::ready(item)
            })
            .boxed();

        Ok((events, model_metadata))
    }

    // =========================================================================
    // Unified Conversation API
    // =========================================================================

    /// Apply default configuration to a ConversationRequest.
    fn apply_conversation_defaults(&self, request: &mut ConversationRequest) -> Result<()> {
        if request.model.is_none() {
            request.model = Some(self.defaults.model.clone());
        }

        if request.temperature.is_none() {
            request.temperature = self.defaults.temperature;
        }

        if request.top_p.is_none() {
            request.top_p = self.defaults.top_p;
        }

        if request.max_output_tokens.is_none() {
            request.max_output_tokens = self.defaults.max_completion_tokens;
        }

        if let Some(effort) = request.reasoning_effort {
            request.reasoning_effort = Some(normalize_reasoning_effort_for_provider(
                self.defaults.provider,
                effort,
            )?);
        }

        Ok(())
    }

    /// Send a conversation request using the Chat Completions API (streaming).
    ///
    /// Converts the `ConversationRequest` to `ChatCompletionRequest` internally.
    /// Returns the stream and any model metadata extracted from response headers.
    pub async fn conversation_stream(
        &self,
        mut request: ConversationRequest,
    ) -> Result<(
        BoxStream<'static, Result<ChatCompletionChunk>>,
        Option<ResponseModelMetadata>,
    )> {
        self.apply_conversation_defaults(&mut request)?;

        let trace = request.trace.take();
        let mut chat_request: ChatCompletionRequest = request.into();
        if let Some(trace) = trace {
            chat_request.trace = Some(trace);
        }

        self.chat_completion_stream(chat_request).await
    }

    /// Send a conversation request using the Chat Completions API (non-streaming).
    ///
    /// Converts the `ConversationRequest` to `ChatCompletionRequest` internally.
    pub async fn conversation(
        &self,
        mut request: ConversationRequest,
    ) -> Result<ChatCompletionResponse> {
        self.apply_conversation_defaults(&mut request)?;

        let trace = request.trace.take();
        let mut chat_request: ChatCompletionRequest = request.into();
        if let Some(trace) = trace {
            chat_request.trace = Some(trace);
        }

        self.chat_completion(chat_request).await
    }

    /// Send a conversation request using the Responses API (streaming).
    ///
    /// Converts the `ConversationRequest` to Responses API format internally.
    /// The third tuple element is the per-request doom-loop signal collector
    /// (see [`Self::create_response_stream`]); callers that don't consume the
    /// signals can ignore it.
    #[allow(clippy::type_complexity)]
    pub async fn conversation_stream_responses(
        &self,
        request: ConversationRequest,
    ) -> Result<(
        BoxStream<'static, Result<rs::ResponseStreamEvent>>,
        Option<ResponseModelMetadata>,
        Option<crate::doom_loop::DoomLoopSignalCollector>,
    )> {
        self.conversation_stream_responses_with_rejected_binding(request, None)
            .await
    }

    #[allow(clippy::type_complexity)]
    pub(crate) async fn conversation_stream_responses_after_provider_auth_recovery(
        &self,
        request: ConversationRequest,
        rejected: &CredentialBinding,
    ) -> Result<(
        BoxStream<'static, Result<rs::ResponseStreamEvent>>,
        Option<ResponseModelMetadata>,
        Option<crate::doom_loop::DoomLoopSignalCollector>,
    )> {
        self.conversation_stream_responses_with_rejected_binding(request, Some(rejected))
            .await
    }

    #[allow(clippy::type_complexity)]
    async fn conversation_stream_responses_with_rejected_binding(
        &self,
        mut request: ConversationRequest,
        rejected: Option<&CredentialBinding>,
    ) -> Result<(
        BoxStream<'static, Result<rs::ResponseStreamEvent>>,
        Option<ResponseModelMetadata>,
        Option<crate::doom_loop::DoomLoopSignalCollector>,
    )> {
        self.apply_conversation_defaults(&mut request)?;

        let trace = request.trace.take();
        let x_grok_conv_id = request.x_grok_conv_id.clone();
        let x_grok_req_id = request.x_grok_req_id.clone();
        let x_grok_session_id = request.x_grok_session_id.clone();
        let x_grok_turn_idx = request.x_grok_turn_idx.clone();
        let x_grok_agent_id = request.x_grok_agent_id.clone();
        let wire_reasoning_effort = request.reasoning_effort;

        // Collect xAI-specific tools that can't be expressed via rs::Tool
        // (e.g., x_search). These are injected as raw JSON after serialization.
        let extra_tools = xai_grok_sampling_types::extra_raw_tools(&request.hosted_tools);

        // The shell resolves the authenticated provider/model capability for
        // each turn. Normalize only this cheap Arc-backed prompt clone at the
        // final Responses conversion seam so persisted Grok history remains
        // authoritative and can be re-evaluated after a provider/model switch.
        let serialization_prompt = request.clone_for_responses_serialization();
        let responses_request: rs::CreateResponse = (&serialization_prompt).into();

        let mut wrapper = CreateResponseWrapper::new(responses_request);
        wrapper.x_grok_conv_id = x_grok_conv_id;
        wrapper.x_grok_req_id = x_grok_req_id;
        wrapper.x_grok_session_id = x_grok_session_id;
        wrapper.x_grok_turn_idx = x_grok_turn_idx;
        wrapper.x_grok_agent_id = x_grok_agent_id;
        wrapper.extra_raw_tools = extra_tools;
        wrapper.wire_reasoning_effort = wire_reasoning_effort;

        if let Some(trace) = trace {
            wrapper.trace = Some(trace);
        }

        self.create_response_stream_after_provider_auth_recovery(wrapper, rejected)
            .await
    }

    /// Send a conversation request using the Responses API (non-streaming).
    ///
    /// Converts the `ConversationRequest` to Responses API format internally.
    pub async fn conversation_responses(
        &self,
        mut request: ConversationRequest,
    ) -> Result<rs::Response> {
        self.apply_conversation_defaults(&mut request)?;

        let trace = request.trace.take();
        let x_grok_conv_id = request.x_grok_conv_id.clone();
        let x_grok_req_id = request.x_grok_req_id.clone();
        let x_grok_session_id = request.x_grok_session_id.clone();
        let x_grok_turn_idx = request.x_grok_turn_idx.clone();
        let x_grok_agent_id = request.x_grok_agent_id.clone();
        let wire_reasoning_effort = request.reasoning_effort;

        // Keep non-streaming Responses on the same request-copy capability
        // contract as the interactive streaming path.
        let serialization_prompt = request.clone_for_responses_serialization();
        let responses_request: rs::CreateResponse = (&serialization_prompt).into();

        let mut wrapper = CreateResponseWrapper::new(responses_request);
        wrapper.x_grok_conv_id = x_grok_conv_id;
        wrapper.x_grok_req_id = x_grok_req_id;
        wrapper.x_grok_session_id = x_grok_session_id;
        wrapper.x_grok_turn_idx = x_grok_turn_idx;
        wrapper.x_grok_agent_id = x_grok_agent_id;
        wrapper.wire_reasoning_effort = wire_reasoning_effort;

        if let Some(trace) = trace {
            wrapper.trace = Some(trace);
        }

        self.create_response(wrapper).await
    }

    /// Send a conversation request using the Anthropic Messages API (streaming).
    ///
    /// Converts the `ConversationRequest` to Messages API format internally.
    pub async fn conversation_stream_messages(
        &self,
        mut request: ConversationRequest,
    ) -> Result<(
        BoxStream<'static, Result<messages::MessageStreamEvent>>,
        Option<ResponseModelMetadata>,
    )> {
        self.apply_conversation_defaults(&mut request)?;

        let trace = request.trace.take();
        let x_grok_conv_id = request.x_grok_conv_id.clone();
        let x_grok_req_id = request.x_grok_req_id.clone();
        let x_grok_session_id = request.x_grok_session_id.clone();
        let x_grok_turn_idx = request.x_grok_turn_idx.clone();
        let x_grok_agent_id = request.x_grok_agent_id.clone();

        let messages_request = build_messages_request(&request);

        let mut wrapper = MessagesRequestWrapper::new(messages_request);
        wrapper.x_grok_conv_id = x_grok_conv_id;
        wrapper.x_grok_req_id = x_grok_req_id;
        wrapper.x_grok_session_id = x_grok_session_id;
        wrapper.x_grok_turn_idx = x_grok_turn_idx;
        wrapper.x_grok_agent_id = x_grok_agent_id;
        wrapper.wire_reasoning_effort = request.reasoning_effort;

        if let Some(trace) = trace {
            wrapper.trace = Some(trace);
        }

        self.create_message_stream(wrapper).await
    }

    /// Send a conversation request using the Anthropic Messages API (non-streaming).
    ///
    /// Converts the `ConversationRequest` to Messages API format internally.
    pub async fn conversation_messages(
        &self,
        mut request: ConversationRequest,
    ) -> Result<messages::MessagesResponse> {
        self.apply_conversation_defaults(&mut request)?;

        let trace = request.trace.take();
        let x_grok_conv_id = request.x_grok_conv_id.clone();
        let x_grok_req_id = request.x_grok_req_id.clone();
        let x_grok_session_id = request.x_grok_session_id.clone();
        let x_grok_turn_idx = request.x_grok_turn_idx.clone();
        let x_grok_agent_id = request.x_grok_agent_id.clone();

        let messages_request = build_messages_request(&request);

        let mut wrapper = MessagesRequestWrapper::new(messages_request);
        wrapper.x_grok_conv_id = x_grok_conv_id;
        wrapper.x_grok_req_id = x_grok_req_id;
        wrapper.x_grok_session_id = x_grok_session_id;
        wrapper.x_grok_turn_idx = x_grok_turn_idx;
        wrapper.x_grok_agent_id = x_grok_agent_id;
        wrapper.wire_reasoning_effort = request.reasoning_effort;

        if let Some(trace) = trace {
            wrapper.trace = Some(trace);
        }

        self.create_message(wrapper).await
    }

    /// Backend-aware streaming call that collects the full response.
    pub async fn conversation_collect(
        &self,
        request: ConversationRequest,
    ) -> Result<ConversationResponse> {
        let request_id = crate::types::RequestId::random();
        let idle_timeout = std::time::Duration::from_secs(300);
        let result = match self.api_backend() {
            ApiBackend::ChatCompletions => {
                let (raw, meta) = self.conversation_stream(request).await?;
                let events =
                    crate::stream::stream_chat_completions(raw, meta, request_id, idle_timeout);
                crate::stream::collect_response(events).await
            }
            ApiBackend::Responses => {
                let (raw, meta, doom_loop) = self.conversation_stream_responses(request).await?;
                let events = crate::stream::stream_responses(
                    raw,
                    meta,
                    request_id,
                    idle_timeout,
                    doom_loop,
                    self.provider(),
                );
                crate::stream::collect_response(events).await
            }
            ApiBackend::Messages => {
                let (raw, meta) = self.conversation_stream_messages(request).await?;
                let events = crate::stream::messages::stream_messages_for_provider(
                    raw,
                    meta,
                    request_id,
                    idle_timeout,
                    self.provider(),
                );
                crate::stream::collect_response(events).await
            }
        };
        result
            .map(|(response, _metrics)| response)
            .map_err(|info| SamplingError::Api {
                status: info
                    .status_code
                    .and_then(|c| reqwest::StatusCode::from_u16(c).ok())
                    .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
                message: info.message,
                model_metadata: info.model_metadata,
                retry_after_secs: info.retry_after_secs,
                should_retry: info.should_retry,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::openai_codex::headers::{
        CHATGPT_ACCOUNT_ID_HEADER, CODEX_CLIENT_REQUEST_ID_HEADER, CODEX_SESSION_ID_HEADER,
        CODEX_THREAD_ID_HEADER, OPENAI_CODEX_ORIGINATOR, OPENAI_FEDRAMP_HEADER,
    };
    use indexmap::IndexMap;
    use xai_grok_sampling_types::OPENAI_CODEX_COMPATIBILITY_VERSION;
    use xai_grok_sampling_types::types::ChatRequestMessage;

    const CODEX_AUTHORIZATION_FIXTURE: &str = "Bearer opaque-credential";
    const CODEX_ACCOUNT_FIXTURE: &str = "opaque-account";
    const CODEX_FEDRAMP_FIXTURE: &str = "true";
    const CODEX_SESSION_FIXTURE: &str = "opaque-session";
    const CODEX_THREAD_FIXTURE: &str = "opaque-thread";
    const CODEX_RECORD_FIXTURE: &str = "local-credential-record";
    const SWITCHED_RECORD_FIXTURE: &str = "other-local-record";
    const REDACTION_VALUE_FIXTURE: &str = "opaque-sensitive-value";

    #[derive(Clone, Default)]
    struct DebugEventCapture {
        events: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    struct DebugFieldCapture(String);

    impl tracing::field::Visit for DebugFieldCapture {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            use std::fmt::Write as _;
            let _ = write!(&mut self.0, "{}={value:?};", field.name());
        }
    }

    impl tracing::Subscriber for DebugEventCapture {
        fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
            *metadata.level() == tracing::Level::DEBUG
        }

        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }

        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

        fn event(&self, event: &tracing::Event<'_>) {
            let mut fields = DebugFieldCapture(String::new());
            event.record(&mut fields);
            let rendered = format!(
                "target={};name={};{}",
                event.metadata().target(),
                event.metadata().name(),
                fields.0
            );
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(rendered);
        }

        fn enter(&self, _: &tracing::span::Id) {}

        fn exit(&self, _: &tracing::span::Id) {}
    }

    fn capture_request_diagnostics(request: &reqwest::Request) -> Vec<String> {
        let capture = DebugEventCapture::default();
        {
            let _guard = tracing::subscriber::set_default(capture.clone());
            SamplingClient::log_request_diagnostics(request, "responses");
        }
        let events = capture
            .events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        events
    }

    fn minimal_config() -> SamplerConfig {
        SamplerConfig {
            provider: ProviderId::Custom,
            credential_source: Default::default(),
            credential_binding: None,
            api_key: Some("test-key".to_string()),
            base_url: "https://example.test".to_string(),
            model: "test-model".to_string(),
            max_completion_tokens: None,
            temperature: None,
            top_p: None,
            api_backend: ApiBackend::ChatCompletions,
            auth_scheme: AuthScheme::Bearer,
            service_tier: None,
            extra_headers: IndexMap::new(),
            context_window: 8192,
            force_http1: false,
            max_retries: None,
            stream_tool_calls: false,
            idle_timeout_secs: None,
            reasoning_effort: None,
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

    /// Verify the serialized shape of StreamingChatRequest matches the
    /// expected wire format: all ChatCompletionRequest fields flattened at
    /// top level, plus `stream: true` and `stream_options.include_usage: true`.
    #[test]
    fn streaming_chat_request_serializes_correctly() {
        let request = ChatCompletionRequest {
            model: Some("test-model".into()),
            messages: vec![ChatRequestMessage::user("hello")],
            temperature: Some(0.7),
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            user: None,
            tools: None,
            tool_choice: None,
            search_parameters: None,
            response_format: None,
            reasoning_effort: None,
            x_grok_conv_id: None,
            x_grok_req_id: None,
            x_grok_session_id: None,
            x_grok_turn_idx: None,
            x_grok_agent_id: None,
            x_grok_deployment_id: None,
            x_grok_user_id: None,
            trace: None,
        };

        let wrapper = StreamingChatRequest {
            inner: &request,
            stream: true,
            stream_options: StreamOptions {
                include_usage: true,
            },
        };

        let json: serde_json::Value = serde_json::to_value(&wrapper).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.get("stream").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            obj.get("stream_options")
                .and_then(|v| v.get("include_usage"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        assert!(
            obj.get("inner").is_none(),
            "inner field should be flattened"
        );
        assert_eq!(
            obj.get("model").and_then(|v| v.as_str()),
            Some("test-model")
        );
        assert!(obj.get("messages").is_some());
        let temp = obj.get("temperature").and_then(|v| v.as_f64()).unwrap();
        assert!((temp - 0.7).abs() < 0.001, "temperature should be ~0.7");

        assert!(obj.get("max_tokens").is_none());
        assert!(obj.get("tools").is_none());
    }

    #[test]
    fn extract_retry_after_parses_seconds() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "30".parse().unwrap());
        assert_eq!(extract_retry_after(&headers), Some(30));
    }

    #[test]
    fn extract_retry_after_caps_at_120() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "3600".parse().unwrap());
        assert_eq!(extract_retry_after(&headers), Some(120));
    }

    #[test]
    fn extract_retry_after_zero_is_valid() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "0".parse().unwrap());
        assert_eq!(extract_retry_after(&headers), Some(0));
    }

    #[test]
    fn extract_retry_after_ignores_http_date() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::RETRY_AFTER,
            "Fri, 31 Dec 2025 23:59:59 GMT".parse().unwrap(),
        );
        assert_eq!(extract_retry_after(&headers), None);
    }

    #[test]
    fn extract_retry_after_none_when_missing() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(extract_retry_after(&headers), None);
    }

    #[test]
    fn model_metadata_captures_request_provider_and_credential_binding() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-models-etag", "codex-etag".parse().unwrap());
        let mut binding = CredentialBinding::openai_codex(Some("credential-record".to_owned()));
        binding.generation = 7;

        let metadata = extract_model_metadata(&headers, ProviderId::OpenAiCodex, Some(&binding))
            .expect("etag should produce response metadata");

        assert_eq!(metadata.provider, ProviderId::OpenAiCodex);
        assert_eq!(metadata.credential_binding.as_ref(), Some(&binding));
        assert_eq!(metadata.models_etag.as_deref(), Some("codex-etag"));
    }

    #[test]
    fn extract_should_retry_true() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-should-retry", "true".parse().unwrap());
        assert_eq!(extract_should_retry(&headers), Some(true));
    }

    #[test]
    fn extract_should_retry_true_case_insensitive() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-should-retry", "TRUE".parse().unwrap());
        assert_eq!(extract_should_retry(&headers), Some(true));
    }

    #[test]
    fn extract_should_retry_false() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-should-retry", "false".parse().unwrap());
        assert_eq!(extract_should_retry(&headers), Some(false));
    }

    #[test]
    fn extract_should_retry_unknown_value_is_none() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-should-retry", "banana".parse().unwrap());
        assert_eq!(extract_should_retry(&headers), None);
    }

    #[test]
    fn extract_should_retry_absent_is_none() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(extract_should_retry(&headers), None);
    }

    #[test]
    fn credential_and_account_headers_are_completely_redacted() {
        for name in [
            "authorization",
            "x-api-key",
            "chatgpt-account-id",
            "x-xai-token-auth",
            "x-grok-user-id",
            "x-grok-deployment-id",
            "openai-organization",
            "x-openai-fedramp",
            "cookie",
        ] {
            let rendered = SamplingClient::format_header(name, REDACTION_VALUE_FIXTURE);
            assert!(rendered.contains("[REDACTED]"));
            assert!(!rendered.contains(REDACTION_VALUE_FIXTURE));
        }
        assert_eq!(
            SamplingClient::format_header("traceparent", "safe-value"),
            "  traceparent: safe-value"
        );
    }

    #[test]
    fn generic_debug_diagnostics_hide_optional_security_header_presence() {
        let url = reqwest::Url::parse("https://example.test/responses").unwrap();
        let baseline = reqwest::Request::new(reqwest::Method::POST, url.clone());
        let mut protected = reqwest::Request::new(reqwest::Method::POST, url);

        let mut fedramp = HeaderValue::from_static("true");
        fedramp.set_sensitive(true);
        protected
            .headers_mut()
            .insert(HeaderName::from_static(OPENAI_FEDRAMP_HEADER), fedramp);
        let mut turn_state = HeaderValue::from_static("opaque-state-fixture");
        turn_state.set_sensitive(true);
        protected
            .headers_mut()
            .insert(HeaderName::from_static(CODEX_TURN_STATE_HEADER), turn_state);

        let baseline_debug = capture_request_diagnostics(&baseline);
        let protected_debug = capture_request_diagnostics(&protected);
        if baseline_debug.len() != 1 {
            panic!("generic request debug diagnostics were not emitted exactly once");
        }
        if baseline_debug != protected_debug {
            panic!("optional request security state changed generic debug diagnostics");
        }
        let captured = protected_debug.join("\n");
        for forbidden in [
            OPENAI_FEDRAMP_HEADER,
            CODEX_TURN_STATE_HEADER,
            "opaque-state-fixture",
        ] {
            if captured.contains(forbidden) {
                panic!("request security state leaked into generic debug diagnostics");
            }
        }
    }

    #[test]
    fn legacy_xai_max_alias_still_serializes_as_xhigh() {
        assert_eq!(
            normalize_reasoning_effort_for_provider(ProviderId::Xai, ReasoningEffort::Max).unwrap(),
            ReasoningEffort::Xhigh,
        );
        assert_eq!(
            normalize_reasoning_effort_for_provider(ProviderId::OpenAiCodex, ReasoningEffort::Max,)
                .unwrap(),
            ReasoningEffort::Max,
        );

        let mut body = serde_json::json!({
            "model": "grok-4",
            "reasoning": {"summary": "auto"}
        });
        codex_responses::apply_extended_codex_reasoning_effort(
            ProviderId::Xai,
            Some(ReasoningEffort::Max),
            &mut body,
        )
        .unwrap();
        assert_eq!(
            body.pointer("/reasoning/effort")
                .and_then(|value| value.as_str()),
            Some("xhigh"),
        );
    }

    #[test]
    fn responses_lite_marker_is_rejected_outside_codex_and_when_not_exact() {
        let mut xai = minimal_config();
        xai.extra_headers.insert(
            OPENAI_CODEX_RESPONSES_LITE_HEADER.to_string(),
            "true".to_string(),
        );
        let error = SamplingClient::new(xai).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires the OpenAI Codex provider")
        );

        let (mut codex, _) = codex_config();
        codex.extra_headers.insert(
            OPENAI_CODEX_RESPONSES_LITE_HEADER.to_string(),
            "TRUE".to_string(),
        );
        let error = SamplingClient::new(codex).unwrap_err();
        assert!(error.to_string().contains("must be exactly true"));
    }

    #[test]
    fn ordinary_codex_requests_do_not_enable_responses_lite() {
        let (config, _) = codex_config();
        let client = SamplingClient::new(config).unwrap();
        let grok_headers = GrokRequestHeaders {
            conv_id: CODEX_THREAD_FIXTURE,
            req_id: "request-ignored",
            model_id: "gpt-5-codex",
            session_id: CODEX_SESSION_FIXTURE,
            turn_idx: None,
            agent_id: "agent-ignored",
            deployment_id: None,
            user_id: None,
        };
        let prepared = client.post("https://example.test/responses").unwrap();
        let request = client
            .apply_provider_request_headers(prepared.builder, &grok_headers)
            .unwrap()
            .body("")
            .build()
            .unwrap();
        assert!(
            request
                .headers()
                .get(OPENAI_CODEX_RESPONSES_LITE_HEADER)
                .is_none()
        );
        assert!(
            request
                .headers()
                .get(CODEX_SESSION_ID_HEADER)
                .is_some_and(|value| value.as_bytes() == CODEX_SESSION_FIXTURE.as_bytes())
        );
        assert!(
            request
                .headers()
                .get(CODEX_THREAD_ID_HEADER)
                .is_some_and(|value| value.as_bytes() == CODEX_THREAD_FIXTURE.as_bytes())
        );
        for name in [
            CODEX_SESSION_ID_HEADER,
            CODEX_THREAD_ID_HEADER,
            CODEX_CLIENT_REQUEST_ID_HEADER,
        ] {
            assert!(
                request.headers()[name].is_sensitive(),
                "{name} must be sensitive at the HTTP layer"
            );
        }
    }

    #[test]
    fn new_with_minimal_config_succeeds() {
        let client = SamplingClient::new(minimal_config()).expect("client should construct");
        assert_eq!(client.api_backend(), ApiBackend::ChatCompletions);
    }

    #[test]
    fn credential_like_extra_headers_are_sensitive() {
        let mut cfg = minimal_config();
        cfg.extra_headers
            .insert("x-test-header".to_string(), "ordinary-value".to_string());
        for name in [
            "authorization",
            "proxy-authorization",
            "x-api-key",
            "x-custom-token",
            "x-api-key-backup",
            "x-client-secret",
        ] {
            cfg.extra_headers
                .insert(name.to_string(), "opaque-value".to_string());
        }

        let client = SamplingClient::new(cfg).expect("extra headers should construct");
        assert!(!client.default_headers["x-test-header"].is_sensitive());
        for name in [
            "authorization",
            "proxy-authorization",
            "x-api-key",
            "x-custom-token",
            "x-api-key-backup",
            "x-client-secret",
        ] {
            assert!(client.default_headers[name].is_sensitive());
        }
    }

    #[derive(Debug, Clone)]
    struct StaticKimiRequestAuth {
        backend: ApiBackend,
    }

    impl crate::config::RequestAuth for StaticKimiRequestAuth {
        fn apply(
            &self,
            headers: &mut HeaderMap,
        ) -> std::result::Result<CredentialBinding, crate::config::RequestAuthError> {
            let (name, value) = if matches!(self.backend, ApiBackend::Messages) {
                (
                    HeaderName::from_static("x-api-key"),
                    HeaderValue::from_static("opaque-kimi-key"),
                )
            } else {
                (
                    AUTHORIZATION,
                    HeaderValue::from_static("Bearer opaque-kimi-key"),
                )
            };
            let mut value = value;
            value.set_sensitive(true);
            headers.insert(name, value);
            let mut binding = CredentialBinding::kimi_code(Some("kimi-record".to_owned()));
            binding.generation = 1;
            Ok(binding)
        }
    }

    fn kimi_config(backend: ApiBackend) -> SamplerConfig {
        let mut config = SamplerConfig::kimi_code("k3", backend.clone());
        config.request_auth = Some(std::sync::Arc::new(StaticKimiRequestAuth { backend }));
        config
    }

    #[derive(Debug)]
    struct StaticZaiRequestAuth;

    impl crate::config::RequestAuth for StaticZaiRequestAuth {
        fn apply(
            &self,
            headers: &mut HeaderMap,
        ) -> std::result::Result<CredentialBinding, crate::config::RequestAuthError> {
            let mut value = HeaderValue::from_static("Bearer opaque-zai-key");
            value.set_sensitive(true);
            headers.insert(AUTHORIZATION, value);
            let mut binding = CredentialBinding::zai_coding_plan(Some("zai-record".to_owned()));
            binding.generation = 1;
            Ok(binding)
        }
    }

    fn zai_config() -> SamplerConfig {
        let mut config = SamplerConfig::zai_coding_plan("glm-5.2");
        config.request_auth = Some(std::sync::Arc::new(StaticZaiRequestAuth));
        config
    }

    struct StaticCodexRequestAuth {
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    fn test_codex_binding(generation: u64) -> CredentialBinding {
        let mut binding = CredentialBinding::openai_codex(Some(CODEX_RECORD_FIXTURE.to_owned()));
        binding.generation = generation;
        binding
    }

    impl crate::config::RequestAuth for StaticCodexRequestAuth {
        fn apply(
            &self,
            headers: &mut HeaderMap,
        ) -> std::result::Result<CredentialBinding, crate::config::RequestAuthError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_static(CODEX_AUTHORIZATION_FIXTURE),
            );
            headers.insert(
                HeaderName::from_static(CHATGPT_ACCOUNT_ID_HEADER),
                HeaderValue::from_static(CODEX_ACCOUNT_FIXTURE),
            );
            headers.insert(
                HeaderName::from_static(OPENAI_FEDRAMP_HEADER),
                HeaderValue::from_static(CODEX_FEDRAMP_FIXTURE),
            );
            Ok(test_codex_binding(1))
        }
    }

    struct RecoveringCodexRequestAuth {
        recoveries: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        generation: std::sync::atomic::AtomicU64,
    }

    impl RecoveringCodexRequestAuth {
        fn new(recoveries: std::sync::Arc<std::sync::atomic::AtomicUsize>) -> Self {
            Self {
                recoveries,
                generation: std::sync::atomic::AtomicU64::new(1),
            }
        }
    }

    impl crate::config::RequestAuth for RecoveringCodexRequestAuth {
        fn apply(
            &self,
            headers: &mut HeaderMap,
        ) -> std::result::Result<CredentialBinding, crate::config::RequestAuthError> {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_static("Bearer opaque-credential"),
            );
            headers.insert(
                HeaderName::from_static(CHATGPT_ACCOUNT_ID_HEADER),
                HeaderValue::from_static("opaque-account"),
            );
            Ok(test_codex_binding(
                self.generation.load(std::sync::atomic::Ordering::SeqCst),
            ))
        }

        fn recover_unauthorized(
            &self,
            rejected: CredentialBinding,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            Box::pin(async move {
                assert_eq!(rejected, test_codex_binding(1));
                self.recoveries
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.generation
                    .store(2, std::sync::atomic::Ordering::SeqCst);
                true
            })
        }
    }

    struct RacingCodexRequestAuth {
        applications: std::sync::atomic::AtomicUsize,
        actual_binding: CredentialBinding,
    }

    impl crate::config::RequestAuth for RacingCodexRequestAuth {
        fn apply(
            &self,
            headers: &mut HeaderMap,
        ) -> std::result::Result<CredentialBinding, crate::config::RequestAuthError> {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_static(CODEX_AUTHORIZATION_FIXTURE),
            );
            headers.insert(
                HeaderName::from_static(CHATGPT_ACCOUNT_ID_HEADER),
                HeaderValue::from_static(CODEX_ACCOUNT_FIXTURE),
            );
            if self
                .applications
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                == 0
            {
                Ok(test_codex_binding(2))
            } else {
                Ok(self.actual_binding.clone())
            }
        }

        fn recover_unauthorized(
            &self,
            _rejected: CredentialBinding,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            Box::pin(async { true })
        }
    }

    struct HeaderMutatingCodexRequestAuth {
        header_name: &'static str,
        header_value: &'static str,
        append: bool,
    }

    impl crate::config::RequestAuth for HeaderMutatingCodexRequestAuth {
        fn apply(
            &self,
            headers: &mut HeaderMap,
        ) -> std::result::Result<CredentialBinding, crate::config::RequestAuthError> {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_static("Bearer valid-codex-access-token"),
            );
            headers.insert(
                HeaderName::from_static(CHATGPT_ACCOUNT_ID_HEADER),
                HeaderValue::from_static("valid-codex-account"),
            );
            headers.insert(
                HeaderName::from_static(OPENAI_FEDRAMP_HEADER),
                HeaderValue::from_static("true"),
            );

            let name = HeaderName::from_static(self.header_name);
            let value = HeaderValue::from_static(self.header_value);
            if self.append {
                headers.append(name, value);
            } else {
                headers.insert(name, value);
            }
            Ok(test_codex_binding(1))
        }
    }

    fn codex_config_with_header_mutation(
        header_name: &'static str,
        header_value: &'static str,
        append: bool,
    ) -> SamplerConfig {
        let mut config = SamplerConfig::openai_codex("gpt-5.6-terra");
        config.request_auth = Some(std::sync::Arc::new(HeaderMutatingCodexRequestAuth {
            header_name,
            header_value,
            append,
        }));
        config
    }

    fn codex_config() -> (
        SamplerConfig,
        std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut config = SamplerConfig::openai_codex("gpt-5-codex");
        config.request_auth = Some(std::sync::Arc::new(StaticCodexRequestAuth {
            calls: std::sync::Arc::clone(&calls),
        }));
        (config, calls)
    }

    #[tokio::test]
    async fn fake_server_provider_matrix_never_receives_xai_headers_off_trusted_xai_routes() {
        use axum::Router;
        use axum::extract::State;
        use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
        use axum::routing::post;

        type Capture = std::sync::Arc<tokio::sync::Mutex<Vec<AxumHeaderMap>>>;
        async fn capture_request(
            State(capture): State<Capture>,
            headers: AxumHeaderMap,
        ) -> StatusCode {
            capture.lock().await.push(headers);
            StatusCode::NO_CONTENT
        }

        let capture: Capture = Default::default();
        let app = Router::new()
            .route("/capture", post(capture_request))
            .with_state(std::sync::Arc::clone(&capture));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let base_url = format!("http://{address}");

        let mut xai_labelled_custom = minimal_config();
        xai_labelled_custom.provider = ProviderId::Xai;
        xai_labelled_custom.base_url = base_url.clone();
        xai_labelled_custom.api_key = Some("synthetic-xai-key".to_owned());
        xai_labelled_custom.extra_headers.insert(
            "x-xai-token-auth".to_owned(),
            "synthetic-xai-marker".to_owned(),
        );

        let mut custom = minimal_config();
        custom.base_url = base_url.clone();
        custom.api_key = Some("synthetic-custom-key".to_owned());

        let (mut codex, _) = codex_config();
        codex.base_url = base_url.clone();
        let mut kimi = kimi_config(ApiBackend::ChatCompletions);
        kimi.base_url = base_url.clone();
        let mut zai = zai_config();
        zai.base_url = base_url.clone();

        for config in [xai_labelled_custom, custom, codex, kimi, zai] {
            let client = SamplingClient::new(config).expect("matrix client should build");
            let request = client
                .post(format!("{base_url}/capture"))
                .expect("matrix auth should prepare")
                .builder
                .body("{}")
                .build()
                .expect("matrix request should build");
            let response = client
                .http
                .execute(request)
                .await
                .expect("matrix request should reach fake server");
            assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
        }

        let captures = capture.lock().await.clone();
        server.abort();
        assert_eq!(captures.len(), 5);

        assert!(captures[0].get(AUTHORIZATION).is_none());
        assert!(captures[0].get("x-api-key").is_none());
        assert!(captures[1].get(AUTHORIZATION).is_some());
        assert!(captures[2].get(AUTHORIZATION).is_some());
        assert!(captures[2].get(CHATGPT_ACCOUNT_ID_HEADER).is_some());
        assert!(captures[3].get(AUTHORIZATION).is_some());
        assert!(captures[4].get(AUTHORIZATION).is_some());
        for headers in captures {
            assert!(headers.keys().all(|name| {
                !name.as_str().starts_with("x-xai-")
                    && !name.as_str().starts_with("x-grok-")
                    && name.as_str() != "x-authenticateresponse"
            }));
        }
    }

    #[tokio::test]
    async fn authenticated_cross_origin_redirect_is_not_followed() {
        use axum::Router;
        use axum::http::{HeaderMap as AxumHeaderMap, HeaderValue as AxumHeaderValue, StatusCode};
        use axum::response::IntoResponse;
        use axum::routing::post;

        let redirected_requests = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let redirected_counter = std::sync::Arc::clone(&redirected_requests);
        let target_app = Router::new().route(
            "/target",
            post(move |headers: AxumHeaderMap| {
                let redirected_counter = std::sync::Arc::clone(&redirected_counter);
                async move {
                    redirected_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    assert!(headers.get(AUTHORIZATION).is_none());
                    StatusCode::NO_CONTENT
                }
            }),
        );
        let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_address = target_listener.local_addr().unwrap();
        let target_server =
            tokio::spawn(async move { axum::serve(target_listener, target_app).await.unwrap() });

        let location = format!("http://{target_address}/target");
        let redirect_app = Router::new().route(
            "/start",
            post(move || {
                let location = location.clone();
                async move {
                    let mut response = StatusCode::TEMPORARY_REDIRECT.into_response();
                    response.headers_mut().insert(
                        axum::http::header::LOCATION,
                        AxumHeaderValue::from_str(&location).unwrap(),
                    );
                    response
                }
            }),
        );
        let redirect_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let redirect_address = redirect_listener.local_addr().unwrap();
        let redirect_server =
            tokio::spawn(
                async move { axum::serve(redirect_listener, redirect_app).await.unwrap() },
            );

        let mut config = minimal_config();
        config.base_url = format!("http://{redirect_address}");
        config.api_key = Some("synthetic-custom-bearer".to_owned());
        let client = SamplingClient::new(config).expect("redirect test client");
        let request = client
            .post(format!("http://{redirect_address}/start"))
            .expect("prepare authenticated redirect request")
            .builder
            .body("{}")
            .build()
            .expect("build authenticated redirect request");
        let response = client
            .http
            .execute(request)
            .await
            .expect("redirect response");

        assert_eq!(response.status(), reqwest::StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            redirected_requests.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "cross-origin redirect target must not receive any request"
        );
        redirect_server.abort();
        target_server.abort();
    }

    #[test]
    fn kimi_chat_request_has_only_sensitive_bearer_and_transport_headers() {
        let request = SamplingClient::new(kimi_config(ApiBackend::ChatCompletions))
            .unwrap()
            .post("http://localhost/test")
            .unwrap()
            .builder
            .build()
            .unwrap();

        let names: std::collections::BTreeSet<_> =
            request.headers().keys().map(|name| name.as_str()).collect();
        assert_eq!(
            names,
            std::collections::BTreeSet::from(["authorization", "content-type", "user-agent"])
        );
        assert!(request.headers()[AUTHORIZATION].is_sensitive());
    }

    #[test]
    fn kimi_messages_request_has_pinned_protocol_and_sensitive_api_key() {
        let request = SamplingClient::new(kimi_config(ApiBackend::Messages))
            .unwrap()
            .post("http://localhost/test")
            .unwrap()
            .builder
            .build()
            .unwrap();

        assert_eq!(
            request.headers()["anthropic-version"],
            xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_VERSION
        );
        assert_eq!(
            request.headers()["anthropic-beta"],
            xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_BETA
        );
        assert!(request.headers()["x-api-key"].is_sensitive());
        assert!(!request.headers().contains_key(AUTHORIZATION));
    }

    #[tokio::test]
    async fn kimi_chat_client_projects_provider_owned_history_at_the_http_boundary() {
        use axum::extract::State;
        use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
        use axum::routing::post;
        use axum::{Json, Router};

        type Capture =
            std::sync::Arc<tokio::sync::Mutex<Option<(AxumHeaderMap, serde_json::Value)>>>;
        async fn capture_request(
            State(capture): State<Capture>,
            headers: AxumHeaderMap,
            Json(body): Json<serde_json::Value>,
        ) -> (StatusCode, Json<serde_json::Value>) {
            *capture.lock().await = Some((headers, body));
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": {"message": "captured"}})),
            )
        }

        let capture: Capture = Default::default();
        let app = Router::new()
            .route("/chat/completions", post(capture_request))
            .with_state(std::sync::Arc::clone(&capture));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let mut config = kimi_config(ApiBackend::ChatCompletions);
        config.base_url = format!("http://{address}");
        let client = SamplingClient::new(config).unwrap();
        let mut assistant = ChatRequestMessage::assistant("", "local-model-id", None);
        assistant.tool_calls = vec![
            xai_grok_sampling_types::ToolCallRequest::function(
                "read_file",
                r#"{"path":"README.md"}"#,
            )
            .with_id("call|unsafe"),
        ];
        let mut request = ChatCompletionRequest::new("k3", vec![assistant]);
        request.reasoning_effort = Some(ReasoningEffort::Medium);
        request.search_parameters = Some(xai_grok_sampling_types::SearchParameters {
            mode: Some("on".to_owned()),
            sources: None,
            from_date: None,
            to_date: None,
            return_citations: Some(true),
            max_search_results: Some(5),
        });

        let error = client
            .chat_completion(request)
            .await
            .expect_err("capture endpoint deliberately rejects the request");
        assert!(!error.to_string().contains("captured"));
        let (headers, body) = capture
            .lock()
            .await
            .take()
            .expect("request must reach the capture boundary");
        server.abort();

        assert!(headers.get(AUTHORIZATION).is_some());
        assert!(headers.keys().all(|name| {
            !name.as_str().starts_with("x-grok-")
                && !name.as_str().starts_with("x-xai-")
                && !name.as_str().starts_with("x-msh-")
        }));
        assert_eq!(body["thinking"]["effort"], "medium");
        assert_eq!(body["thinking"]["keep"], "all");
        assert_eq!(body["messages"][0]["reasoning_content"], "");
        assert!(body["messages"][0].get("model_id").is_none());
        assert!(body["messages"][0].get("content").is_none());
        assert_eq!(body["messages"][0]["tool_calls"][0]["id"], "call_unsafe");
        assert!(body.get("search_parameters").is_none());
    }

    #[test]
    fn kimi_request_rejects_foreign_header_added_by_injector() {
        #[derive(Debug)]
        struct ForeignInjector;
        impl crate::config::HeaderInjector for ForeignInjector {
            fn inject(&self, headers: &mut HeaderMap) {
                headers.insert(
                    HeaderName::from_static("cookie"),
                    HeaderValue::from_static("foreign-cookie"),
                );
            }
        }

        let mut config = kimi_config(ApiBackend::ChatCompletions);
        config.header_injector = Some(std::sync::Arc::new(ForeignInjector));
        let client = SamplingClient::new(config).unwrap();

        let error = client
            .post("http://localhost/test")
            .err()
            .expect("foreign injected header must fail closed");
        assert!(error.to_string().contains("unapproved headers"));
        assert!(!error.to_string().contains("foreign-cookie"));
    }

    #[test]
    fn codex_uses_current_chatgpt_responses_endpoint() {
        let (config, _) = codex_config();
        let client = SamplingClient::new(config).expect("Codex client should build");
        assert_eq!(
            client.endpoint("responses"),
            xai_grok_sampling_types::OPENAI_CODEX_RESPONSES_URL
        );
    }

    #[test]
    fn codex_fast_service_tier_maps_to_exact_priority_wire_value() {
        let (mut config, _) = codex_config();
        config.service_tier =
            Some(xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER.to_owned());
        let client = SamplingClient::new(config).expect("Codex client should build");
        let mut request = CreateResponseWrapper::default();

        client
            .apply_response_defaults(&mut request)
            .expect("Codex response defaults should apply");

        assert!(matches!(
            request.inner.service_tier,
            Some(rs::ServiceTier::Priority)
        ));
        let body = serde_json::to_value(&request.inner).expect("request should serialize");
        assert_eq!(
            body.get("service_tier").and_then(serde_json::Value::as_str),
            Some("priority")
        );
    }

    #[test]
    fn codex_explicit_standard_service_tier_is_omitted_from_wire() {
        let (mut config, _) = codex_config();
        config.service_tier =
            Some(xai_grok_sampling_types::OPENAI_CODEX_STANDARD_SERVICE_TIER.to_owned());
        let client = SamplingClient::new(config).expect("Codex client should build");
        let mut request = CreateResponseWrapper::default();

        client
            .apply_response_defaults(&mut request)
            .expect("Codex response defaults should apply");

        assert!(request.inner.service_tier.is_none());
        let body = serde_json::to_value(&request.inner).expect("request should serialize");
        assert!(body.get("service_tier").is_none());
    }

    #[test]
    fn xai_does_not_inherit_codex_fast_service_tier() {
        let mut config = minimal_config();
        config.provider = ProviderId::Xai;
        config.base_url = xai_grok_sampling_types::XAI_API_BASE_URL.to_owned();
        config.api_backend = ApiBackend::Responses;
        config.service_tier =
            Some(xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER.to_owned());
        let client = SamplingClient::new(config).expect("xAI client should build");
        let mut request = CreateResponseWrapper::default();

        client
            .apply_response_defaults(&mut request)
            .expect("xAI response defaults should apply");

        assert!(request.inner.service_tier.is_none());
        let body = serde_json::to_value(&request.inner).expect("request should serialize");
        assert!(body.get("service_tier").is_none());
    }

    #[test]
    fn codex_unknown_service_tier_is_omitted_from_wire() {
        let (mut config, _) = codex_config();
        config.service_tier = Some("unknown-provider-tier".to_owned());
        let client = SamplingClient::new(config).expect("Codex client should build");
        let mut request = CreateResponseWrapper::default();

        client
            .apply_response_defaults(&mut request)
            .expect("unsupported tier should be omitted without failing the request");

        assert!(request.inner.service_tier.is_none());
        let body = serde_json::to_value(&request.inner).expect("request should serialize");
        assert!(body.get("service_tier").is_none());
    }

    #[tokio::test]
    async fn provider_auth_recovery_is_scoped_to_codex_request_auth() {
        let recoveries = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut config = SamplerConfig::openai_codex("gpt-5-codex");
        config.request_auth = Some(std::sync::Arc::new(RecoveringCodexRequestAuth::new(
            std::sync::Arc::clone(&recoveries),
        )));
        let client = SamplingClient::new(config).expect("Codex client should build");
        let rejected = test_codex_binding(1);
        assert!(
            client
                .recover_provider_auth_after_unauthorized(rejected.clone())
                .await
        );
        assert_eq!(recoveries.load(std::sync::atomic::Ordering::SeqCst), 1);
        let prepared = client
            .post_after_provider_auth_recovery("https://example.test/responses", Some(&rejected))
            .expect("same-record generation advance should bind the retry");
        assert_eq!(
            prepared
                .credential_binding
                .expect("Codex request must retain a binding")
                .generation,
            2
        );

        let xai = SamplingClient::new(minimal_config()).expect("xAI client should build");
        assert!(
            !xai.recover_provider_auth_after_unauthorized(test_codex_binding(1))
                .await
        );
        assert_eq!(recoveries.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn requests_fail_closed_while_provider_recovery_is_in_progress() {
        let (config, auth_calls) = codex_config();
        let client = SamplingClient::new(config).expect("Codex client should build");
        let recovery_guard =
            ProviderAuthRecoveryGuard::begin(client.provider_auth_recovery_state.as_ref())
                .expect("idle recovery state should be reservable");

        let error = match client.post("https://example.test/responses") {
            Err(error) => error,
            Ok(_) => panic!("a request escaped while provider recovery was in progress"),
        };
        assert!(error.to_string().contains("still in progress"));
        assert_eq!(auth_calls.load(std::sync::atomic::Ordering::SeqCst), 0);

        drop(recovery_guard);
        client
            .post("https://example.test/responses")
            .expect("dropping an unfinished recovery restores idle state");
        assert_eq!(auth_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn actual_request_binding_cannot_race_recovery_preflight() {
        let mut switched = test_codex_binding(3);
        switched.record_id = Some(SWITCHED_RECORD_FIXTURE.to_owned());
        let mut config = SamplerConfig::openai_codex("gpt-5-codex");
        config.request_auth = Some(std::sync::Arc::new(RacingCodexRequestAuth {
            applications: std::sync::atomic::AtomicUsize::new(0),
            actual_binding: switched,
        }));
        let client = SamplingClient::new(config).expect("Codex client should build");
        let retry_client = client.clone();
        assert!(
            client
                .recover_provider_auth_after_unauthorized(test_codex_binding(1))
                .await
        );

        let error = match retry_client.post("https://example.test/responses") {
            Err(error) => error,
            Ok(_) => panic!("a raced credential rebind was accepted for retry"),
        };
        let rendered = error.to_string();
        assert!(rendered.contains("did not advance the rejected credential binding"));
        assert!(!rendered.contains(SWITCHED_RECORD_FIXTURE));
    }

    #[test]
    fn codex_rejects_static_or_generic_header_credentials() {
        const UNSAFE_VALUE: &str = "opaque-credential-material";

        let (mut static_key, _) = codex_config();
        static_key.api_key = Some(UNSAFE_VALUE.to_string());
        let error = SamplingClient::new(static_key).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(codex_headers::CODEX_AUTH_CONFIGURATION_INVALID)
        );

        for name in [
            "authorization",
            "proxy-authorization",
            "x-api-key-backup",
            "x-auth-token",
            "x-client-secret",
            "cookie",
        ] {
            let (mut generic_header, _) = codex_config();
            generic_header
                .extra_headers
                .insert(name.to_string(), UNSAFE_VALUE.to_string());
            let Err(error) = SamplingClient::new(generic_header) else {
                panic!("Codex accepted a protected extra-header alias");
            };
            let rendered = error.to_string();
            assert!(rendered.contains("protected/provider headers"));
            if rendered.contains(UNSAFE_VALUE) {
                panic!("protected extra-header credential material leaked into an error");
            }
        }
    }

    #[test]
    fn codex_rejects_credential_aliases_emitted_by_request_auth() {
        const HOSTILE_VALUE: &str = "credential-material-must-never-leak";
        for header_name in [
            "proxy-authorization",
            "cookie",
            "set-cookie",
            "x-api-key",
            "x-api-key-backup",
            "x-apikey",
            "api-key",
            "openai-api-key",
            "x-openai-api-key",
            "openai-organization",
            "openai-project",
            "x-openai-actor-authorization",
            "x-auth-token",
            "x-client-secret",
        ] {
            let client = SamplingClient::new(codex_config_with_header_mutation(
                header_name,
                HOSTILE_VALUE,
                false,
            ))
            .expect("the dynamic auth boundary is validated per request");
            let error = match client.post("http://localhost/test") {
                Ok(_) => panic!("a hostile credential header was accepted"),
                Err(error) => error,
            };
            let rendered = error.to_string();
            if !rendered.contains(codex_headers::CODEX_AUTH_CONFIGURATION_INVALID) {
                panic!("credential-header rejection used an unexpected diagnostic");
            }
            assert!(!rendered.contains(header_name));
            assert!(!rendered.contains(HOSTILE_VALUE));
        }
    }

    #[test]
    fn codex_fedramp_auth_state_is_optional_but_must_be_exactly_true() {
        for invalid_value in ["false", "TRUE", "1"] {
            let client = SamplingClient::new(codex_config_with_header_mutation(
                OPENAI_FEDRAMP_HEADER,
                invalid_value,
                false,
            ))
            .expect("the dynamic auth boundary is validated per request");
            let error = match client.post("http://localhost/test") {
                Ok(_) => panic!("invalid provider security state was accepted"),
                Err(error) => error,
            };
            let rendered = error.to_string();
            if !rendered.contains(codex_headers::CODEX_AUTH_CONFIGURATION_INVALID) {
                panic!("provider security-state rejection used an unexpected diagnostic");
            }
            assert!(!rendered.contains("FedRAMP"));
            assert!(!rendered.contains(invalid_value));
        }

        let mut without_fedramp = SamplerConfig::openai_codex("gpt-5.6-terra");
        without_fedramp.request_auth = Some(std::sync::Arc::new(RecoveringCodexRequestAuth::new(
            Default::default(),
        )));
        let client = SamplingClient::new(without_fedramp).unwrap();
        let _ = client
            .post("http://localhost/test")
            .expect("FedRAMP state is optional");

        let client = SamplingClient::new(codex_config_with_header_mutation(
            OPENAI_FEDRAMP_HEADER,
            "true",
            false,
        ))
        .unwrap();
        let _ = client
            .post("http://localhost/test")
            .expect("exact lowercase true is allowed");
    }

    #[test]
    fn codex_rejects_duplicate_allowed_credential_headers() {
        for (header_name, header_value) in [
            ("authorization", "Bearer second-token"),
            (CHATGPT_ACCOUNT_ID_HEADER, "second-account"),
            (OPENAI_FEDRAMP_HEADER, "true"),
        ] {
            let client = SamplingClient::new(codex_config_with_header_mutation(
                header_name,
                header_value,
                true,
            ))
            .expect("the dynamic auth boundary is validated per request");
            let error = match client.post("http://localhost/test") {
                Ok(_) => panic!("duplicate credential headers were accepted"),
                Err(error) => error,
            };
            let rendered = error.to_string();
            if !rendered.contains(codex_headers::CODEX_AUTH_CONFIGURATION_INVALID) {
                panic!("duplicate credential rejection used an unexpected diagnostic");
            }
            assert!(!rendered.contains(header_value));
        }
    }

    #[test]
    fn codex_auth_validation_preserves_noncredential_transport_headers() {
        let client = SamplingClient::new(codex_config_with_header_mutation(
            "traceparent",
            "00-safe-trace-context-00",
            false,
        ))
        .unwrap();
        let request = client
            .post("http://localhost/test")
            .expect("noncredential transport header should survive auth validation")
            .builder
            .build()
            .unwrap();
        assert_eq!(
            request
                .headers()
                .get("traceparent")
                .and_then(|value| value.to_str().ok()),
            Some("00-safe-trace-context-00")
        );
        assert!(request.headers().contains_key(CONTENT_TYPE));
        assert!(request.headers().contains_key(USER_AGENT));
        assert!(request.headers().contains_key("originator"));
        assert!(request.headers().contains_key("version"));
        assert!(request.headers()[AUTHORIZATION].is_sensitive());
        assert!(request.headers()[CHATGPT_ACCOUNT_ID_HEADER].is_sensitive());
        assert!(request.headers()[OPENAI_FEDRAMP_HEADER].is_sensitive());
    }

    #[tokio::test]
    async fn codex_responses_wire_request_has_exact_auth_and_no_xai_extensions() {
        use axum::extract::State;
        use axum::http::{HeaderMap as AxumHeaderMap, StatusCode, Uri};
        use axum::routing::post;
        use axum::{Json, Router};

        type CapturedRequest = (Uri, AxumHeaderMap, serde_json::Value);
        type Capture = std::sync::Arc<tokio::sync::Mutex<Option<CapturedRequest>>>;

        async fn capture_request(
            State(capture): State<Capture>,
            uri: Uri,
            headers: AxumHeaderMap,
            Json(body): Json<serde_json::Value>,
        ) -> (StatusCode, Json<serde_json::Value>) {
            *capture.lock().await = Some((uri, headers, body));
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": {"message": "captured"}})),
            )
        }

        #[derive(Debug)]
        struct PoisoningInjector;
        impl crate::config::HeaderInjector for PoisoningInjector {
            fn inject(&self, headers: &mut HeaderMap) {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_static("Bearer generic-injector-token"),
                );
                headers.insert(
                    HeaderName::from_static(CHATGPT_ACCOUNT_ID_HEADER),
                    HeaderValue::from_static("generic-injector-account"),
                );
                headers.insert(
                    HeaderName::from_static("x-xai-token-auth"),
                    HeaderValue::from_static("xai-grok-cli"),
                );
                headers.insert(
                    HeaderName::from_static("x-openai-fedramp"),
                    HeaderValue::from_static("false"),
                );
                headers.insert(
                    HeaderName::from_static("x-grok-req-id"),
                    HeaderValue::from_static("must-be-removed"),
                );
                headers.insert(
                    HeaderName::from_static("originator"),
                    HeaderValue::from_static("must-be-replaced"),
                );
                headers.insert(
                    HeaderName::from_static("version"),
                    HeaderValue::from_static("must-be-replaced"),
                );
                headers.insert(
                    HeaderName::from_static(OPENAI_CODEX_RESPONSES_LITE_HEADER),
                    HeaderValue::from_static("false"),
                );
                headers.insert(
                    HeaderName::from_static(CODEX_SESSION_ID_HEADER),
                    HeaderValue::from_static("poisoned-session"),
                );
                headers.insert(
                    HeaderName::from_static(CODEX_THREAD_ID_HEADER),
                    HeaderValue::from_static("poisoned-thread"),
                );
                headers.insert(
                    HeaderName::from_static(CODEX_CLIENT_REQUEST_ID_HEADER),
                    HeaderValue::from_static("poisoned-request"),
                );
                headers.insert(
                    HeaderName::from_static("traceparent"),
                    HeaderValue::from_static("00-safe-trace-context-00"),
                );
            }
        }

        let capture: Capture = Default::default();
        let app = Router::new()
            .route("/responses", post(capture_request))
            .with_state(std::sync::Arc::clone(&capture));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let (mut config, auth_calls) = codex_config();
        config.base_url = format!("http://{addr}");
        config.model = "gpt-5.6-terra".to_string();
        config.comp_hash = Some("opaque-session-only-hash".to_string());
        config.supports_reasoning_summary_parameter = true;
        config.default_reasoning_summary = Some("auto".to_string());
        config.client_version = Some("must-not-be-sent".to_string());
        config.client_identifier = Some("must-not-be-sent".to_string());
        config.deployment_id = Some("must-not-be-sent".to_string());
        config.user_id = Some("must-not-be-sent".to_string());
        config.stream_tool_calls = true;
        config.extra_headers.insert(
            OPENAI_CODEX_RESPONSES_LITE_HEADER.to_string(),
            "true".to_string(),
        );
        config.doom_loop_recovery = Some(Default::default());
        config.header_injector = Some(std::sync::Arc::new(PoisoningInjector));
        let client = SamplingClient::new(config).expect("Codex client should build");

        let mut request = CreateResponseWrapper::default();
        request.inner.tools = Some(vec![rs::Tool::Function(rs::FunctionTool {
            name: "read_file".to_string(),
            description: Some("Read a file".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
            })),
            strict: None,
        })]);
        request.wire_reasoning_effort = Some(ReasoningEffort::Ultra);
        request.x_grok_conv_id = Some(CODEX_THREAD_FIXTURE.to_string());
        request.x_grok_req_id = Some("must-not-be-sent".to_string());
        request.x_grok_session_id = Some(CODEX_SESSION_FIXTURE.to_string());
        request.x_grok_agent_id = Some("must-not-be-sent".to_string());
        request.extra_raw_tools = vec![serde_json::json!({"type": "x_search"})];

        let error = match client.create_response_stream(request).await {
            Err(error) => error,
            Ok(_) => panic!("mock deliberately returns 400"),
        };
        assert!(error.to_string().contains("ChatGPT Codex request rejected"));
        assert!(!error.to_string().contains("captured"));
        assert_eq!(auth_calls.load(std::sync::atomic::Ordering::SeqCst), 1);

        let (uri, headers, body) = capture
            .lock()
            .await
            .take()
            .expect("mock captured the request");
        server.abort();

        assert_eq!(uri.path(), "/responses");
        assert!(
            headers.get(AUTHORIZATION).is_some_and(|value| {
                value.as_bytes() == CODEX_AUTHORIZATION_FIXTURE.as_bytes()
            })
        );
        assert!(
            headers
                .get(CHATGPT_ACCOUNT_ID_HEADER)
                .is_some_and(|value| value.as_bytes() == CODEX_ACCOUNT_FIXTURE.as_bytes())
        );
        assert!(
            headers
                .get(OPENAI_FEDRAMP_HEADER)
                .is_some_and(|value| value.as_bytes() == CODEX_FEDRAMP_FIXTURE.as_bytes())
        );
        assert_eq!(
            headers
                .get("originator")
                .and_then(|value| value.to_str().ok()),
            Some(OPENAI_CODEX_ORIGINATOR)
        );
        assert_eq!(
            headers.get("version").and_then(|value| value.to_str().ok()),
            Some(OPENAI_CODEX_COMPATIBILITY_VERSION)
        );
        assert!(
            headers
                .get(CODEX_SESSION_ID_HEADER)
                .is_some_and(|value| value.as_bytes() == CODEX_SESSION_FIXTURE.as_bytes())
        );
        assert!(
            headers
                .get(CODEX_THREAD_ID_HEADER)
                .is_some_and(|value| value.as_bytes() == CODEX_THREAD_FIXTURE.as_bytes())
        );
        assert!(
            headers
                .get(CODEX_CLIENT_REQUEST_ID_HEADER)
                .is_some_and(|value| value.as_bytes() == CODEX_THREAD_FIXTURE.as_bytes())
        );
        assert_eq!(
            headers
                .get_all(OPENAI_CODEX_RESPONSES_LITE_HEADER)
                .iter()
                .count(),
            1
        );
        assert_eq!(
            headers
                .get(OPENAI_CODEX_RESPONSES_LITE_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("true")
        );
        assert_eq!(
            headers
                .get("traceparent")
                .and_then(|value| value.to_str().ok()),
            Some("00-safe-trace-context-00")
        );
        assert!(
            headers
                .keys()
                .all(|name| !codex_headers::is_xai_specific_header(name)),
            "Codex requests must not contain any xAI-specific header"
        );
        assert_eq!(body["model"], "gpt-5.6-terra");
        assert!(body.get("comp_hash").is_none());
        assert_eq!(body["reasoning"]["effort"], "max");
        assert_eq!(body["reasoning"]["summary"], "auto");
        assert_eq!(body["reasoning"]["context"], "all_turns");
        assert_eq!(body["parallel_tool_calls"], false);
        assert_eq!(body["tool_choice"], "auto");
        assert_eq!(body["input"][0]["type"], "additional_tools");
        assert_eq!(body["input"][0]["role"], "developer");
        assert_eq!(body["input"][0]["tools"][0]["strict"], false);
        assert!(body.get("tools").is_none());
        assert!(body.get("instructions").is_none());
        assert_eq!(body["stream"], true);
        assert_eq!(body["store"], false);
        assert!(body.get("stream_tool_calls").is_none());
        assert!(
            body.get("tools")
                .and_then(|tools| tools.as_array())
                .is_none_or(|tools| tools.iter().all(|tool| tool["type"] != "x_search"))
        );
        assert!(body["include"].as_array().is_some_and(|include| {
            include
                .iter()
                .any(|item| item.as_str() == Some("reasoning.encrypted_content"))
        }));
    }

    #[tokio::test]
    async fn codex_401_carries_exact_request_credential_binding() {
        use axum::Router;
        use axum::http::StatusCode;
        use axum::routing::post;

        let app = Router::new().route("/responses", post(|| async { StatusCode::UNAUTHORIZED }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let (mut config, _) = codex_config();
        config.base_url = format!("http://{address}");
        let client = SamplingClient::new(config).unwrap();
        let error = match client
            .create_response_stream(CreateResponseWrapper::default())
            .await
        {
            Err(error) => error,
            Ok(_) => panic!("mock deliberately returns 401"),
        };
        server.abort();

        let (provider, credential) = error
            .rejected_provider_credential()
            .expect("Codex 401 must retain the signing receipt");
        assert_eq!(provider, ProviderId::OpenAiCodex);
        assert!(credential == &test_codex_binding(1));
        let rendered = format!("{error:?} {error}");
        assert!(!rendered.contains(CODEX_RECORD_FIXTURE));
        assert!(!rendered.contains(CODEX_AUTHORIZATION_FIXTURE));
        assert!(!rendered.contains(CODEX_ACCOUNT_FIXTURE));
    }

    #[tokio::test]
    async fn codex_5xx_headers_cannot_reflect_credentials_into_diagnostics() {
        use axum::Router;
        use axum::http::{HeaderValue, StatusCode};
        use axum::response::IntoResponse;
        use axum::routing::post;

        const REFLECTED_BODY: &str = "Bearer reflected-body-secret";
        const REFLECTED_HEADER: &str = "Bearer reflected-header-secret";
        const REFLECTED_CONTENT_TYPE: &str = "text/plain; reflected=account-secret";

        let app = Router::new().route(
            "/responses",
            post(|| async {
                let mut response =
                    (StatusCode::INTERNAL_SERVER_ERROR, REFLECTED_BODY).into_response();
                response
                    .headers_mut()
                    .insert("x-error-detail", HeaderValue::from_static(REFLECTED_HEADER));
                response.headers_mut().insert(
                    reqwest::header::CONTENT_TYPE,
                    HeaderValue::from_static(REFLECTED_CONTENT_TYPE),
                );
                response
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let (mut config, _) = codex_config();
        config.base_url = format!("http://{address}");
        let client = SamplingClient::new(config).unwrap();
        let error = match client
            .create_response_stream(CreateResponseWrapper::default())
            .await
        {
            Err(error) => error,
            Ok(_) => panic!("mock deliberately returns 500"),
        };
        server.abort();

        let rendered = format!("{error:?} {error}");
        assert!(rendered.contains("content-type: [value omitted]"));
        assert!(!rendered.contains("x-error-detail"));
        assert!(!rendered.contains(REFLECTED_HEADER));
        assert!(!rendered.contains(REFLECTED_BODY));
        assert!(!rendered.contains(REFLECTED_CONTENT_TYPE));
        assert!(!rendered.contains(CODEX_AUTHORIZATION_FIXTURE));
        assert!(!rendered.contains(CODEX_ACCOUNT_FIXTURE));
    }

    #[test]
    fn codex_error_bodies_cannot_reflect_credentials_into_diagnostics() {
        const REFLECTED_BODY: &[u8] =
            br#"{"error":{"message":"Bearer secret-token account-secret"}}"#;
        const TOKEN_MARKER: &str = "secret-token";
        const ACCOUNT_MARKER: &str = "account-secret";
        const USAGE_BODY: &[u8] = br#"{"error":{"type":"usage_limit_reached","message":"Bearer reflected-secret","resets_at":1704067242}}"#;
        const USAGE_MARKER: &str = "reflected-secret";
        const RESET_MARKER: &str = "1704067242";

        let (config, _) = codex_config();
        let client = SamplingClient::new(config).unwrap();
        let message =
            client.provider_error_message(reqwest::StatusCode::BAD_REQUEST, REFLECTED_BODY);
        if message != "ChatGPT Codex request rejected" {
            panic!("provider error diagnostics exposed unexpected detail");
        }
        assert!(!message.contains(TOKEN_MARKER));
        assert!(!message.contains(ACCOUNT_MARKER));

        let encrypted = br#"{"error":{"code":"encrypted_content"}}"#;
        if client.provider_error_message(reqwest::StatusCode::BAD_REQUEST, encrypted)
            != "ChatGPT Codex request rejected (encrypted_content)"
        {
            panic!("provider encrypted-content diagnostics exposed unexpected detail");
        }

        let usage_message =
            client.provider_error_message(reqwest::StatusCode::TOO_MANY_REQUESTS, USAGE_BODY);
        if usage_message != "ChatGPT Codex request rejected (usage limit reached)" {
            panic!("provider usage-limit diagnostics exposed unexpected detail");
        }
        assert!(!usage_message.contains(USAGE_MARKER));
        assert!(!usage_message.contains(RESET_MARKER));
    }

    #[test]
    fn messages_plus_anthropic_api_key_uses_x_api_key_and_not_authorization() {
        let cfg = SamplerConfig {
            api_key: Some("anthropic-key-abc123".to_string()),
            api_backend: ApiBackend::Messages,
            auth_scheme: AuthScheme::XApiKey,
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        let api_key = client
            .default_headers
            .get(HeaderName::from_static("x-api-key"))
            .expect("API-key authentication should be configured");
        assert!(api_key.is_sensitive());
        assert!(client.default_headers.get(AUTHORIZATION).is_none());
    }

    #[test]
    fn messages_plus_bearer_uses_authorization_and_not_x_api_key() {
        let cfg = SamplerConfig {
            api_key: Some("bearer-key-abc123".to_string()),
            api_backend: ApiBackend::Messages,
            auth_scheme: AuthScheme::Bearer,
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        let authorization = client
            .default_headers
            .get(AUTHORIZATION)
            .expect("bearer authentication should be configured");
        assert!(authorization.is_sensitive());
        assert!(
            client
                .default_headers
                .get(HeaderName::from_static("x-api-key"))
                .is_none()
        );
    }

    // Regression: a past change dropped User-Agent from sampling requests.
    #[test]
    fn sampling_client_always_has_user_agent() {
        let client = SamplingClient::new(minimal_config()).expect("build");
        assert!(client.default_headers.contains_key(USER_AGENT));
    }

    // Regression: a past change dropped HeaderInjector (traceparent) from sampling requests.
    #[test]
    fn header_injector_is_called_in_post() {
        #[derive(Debug)]
        struct TestInjector;
        impl crate::config::HeaderInjector for TestInjector {
            fn inject(&self, headers: &mut HeaderMap) {
                headers.insert(
                    HeaderName::from_static("traceparent"),
                    HeaderValue::from_static("00-test-trace-id-00"),
                );
            }
        }

        let mut config = minimal_config();
        config.header_injector = Some(std::sync::Arc::new(TestInjector));
        let client = SamplingClient::new(config).expect("build");
        let req = client
            .post("http://localhost/test")
            .expect("apply request auth")
            .builder
            .build()
            .expect("build request");
        assert!(
            req.headers().contains_key("traceparent"),
            "HeaderInjector should inject traceparent into post() requests"
        );
    }

    #[test]
    fn credential_header_replacement_by_injector_is_sensitive() {
        #[derive(Debug)]
        struct CredentialInjector;
        impl crate::config::HeaderInjector for CredentialInjector {
            fn inject(&self, headers: &mut HeaderMap) {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_static("Bearer opaque-injected-credential"),
                );
                headers.insert(
                    HeaderName::from_static("x-api-key-backup"),
                    HeaderValue::from_static("opaque-injected-credential"),
                );
            }
        }

        let mut config = minimal_config();
        config.header_injector = Some(std::sync::Arc::new(CredentialInjector));
        let request = SamplingClient::new(config)
            .expect("client should build")
            .post("http://localhost/test")
            .expect("injected headers should be accepted")
            .builder
            .build()
            .expect("request should build");

        assert!(request.headers()[AUTHORIZATION].is_sensitive());
        assert!(request.headers()["x-api-key-backup"].is_sensitive());
    }

    #[test]
    fn user_agent_includes_origin_and_agent_product() {
        let origin = OriginClientInfo {
            product: "my-client".to_string(),
            version: Some("1.2.3".to_string()),
        };
        let ua = user_agent_string_for(&origin);
        assert!(ua.contains("my-client/1.2.3"));
        assert!(ua.contains(AGENT_PRODUCT));
    }

    #[test]
    fn user_agent_omits_origin_version_when_absent() {
        let origin = OriginClientInfo {
            product: "my-client".to_string(),
            version: None,
        };
        let ua = user_agent_string_for(&origin);
        // No slash between product and the grok-shell agent product.
        assert!(ua.starts_with("my-client grok-shell/"));
    }

    #[test]
    fn user_agent_collapses_when_origin_matches_agent() {
        let agent_version = xai_grok_version::VERSION.to_string();
        let origin = OriginClientInfo {
            product: AGENT_PRODUCT.to_string(),
            version: Some(agent_version.clone()),
        };
        let ua = user_agent_string_for(&origin);
        // Single product/version slot when the origin and agent match.
        assert!(ua.starts_with(&format!("{}/{}", AGENT_PRODUCT, agent_version)));
    }

    /// Counts callbacks for assertions in the tests below.
    #[derive(Default, Debug)]
    struct CountingCallback {
        invocations: std::sync::Mutex<Vec<(crate::attribution::SamplingConsumer, bool)>>,
    }

    #[derive(Debug)]
    struct StaticBearerResolver(&'static str);

    impl crate::config::BearerResolver for StaticBearerResolver {
        fn current_bearer(&self) -> Option<String> {
            Some(self.0.to_string())
        }
    }

    #[test]
    fn xai_auth_is_kept_only_for_a_canonical_trusted_origin() {
        let mut config = minimal_config();
        config.provider = ProviderId::Xai;
        config.base_url = xai_grok_sampling_types::XAI_API_BASE_URL.to_owned();
        config.bearer_resolver = Some(std::sync::Arc::new(StaticBearerResolver(
            "synthetic-live-xai",
        )));
        let client = SamplingClient::new(config).expect("trusted xAI client");
        let request = client
            .post("https://api.x.ai/v1/responses")
            .expect("prepare trusted xAI request")
            .builder
            .build()
            .expect("build trusted xAI request");

        let authorization = request
            .headers()
            .get(AUTHORIZATION)
            .expect("trusted xAI request should carry its bearer");
        assert_eq!(authorization.as_bytes(), b"Bearer synthetic-live-xai");
        assert!(authorization.is_sensitive());
    }

    #[test]
    fn xai_label_on_custom_origin_strips_static_live_and_extra_auth() {
        let mut config = minimal_config();
        config.provider = ProviderId::Xai;
        config.base_url = "https://custom.invalid/v1".to_owned();
        config.api_key = Some("synthetic-static-xai".to_owned());
        config.bearer_resolver = Some(std::sync::Arc::new(StaticBearerResolver(
            "synthetic-live-xai",
        )));
        config.extra_headers.insert(
            "Authorization".to_owned(),
            "Bearer synthetic-extra-xai".to_owned(),
        );
        config
            .extra_headers
            .insert("X-XAI-Token-Auth".to_owned(), "synthetic-marker".to_owned());
        config
            .extra_headers
            .insert("x-grok-user-id".to_owned(), "synthetic-user".to_owned());
        let client = SamplingClient::new(config).expect("untrusted xAI-labelled client");
        let prepared = client
            .prepare_request_headers(None)
            .expect("prepare untrusted xAI-labelled headers")
            .0;

        assert!(prepared.get(AUTHORIZATION).is_none());
        assert!(prepared.get("x-api-key").is_none());
        assert!(prepared.get("x-xai-token-auth").is_none());
        assert!(
            prepared
                .keys()
                .all(|name| !name.as_str().starts_with("x-grok-"))
        );
        assert_eq!(client.auth_info().auth_type, "none");
    }

    #[test]
    fn custom_provider_rejects_xai_specific_extra_headers() {
        let mut config = minimal_config();
        config
            .extra_headers
            .insert("x-xai-token-auth".to_owned(), "synthetic-marker".to_owned());
        let error = SamplingClient::new(config).expect_err("custom must reject xAI headers");
        assert!(error.to_string().contains("require the xAI provider"));
    }

    impl crate::attribution::Auth401AttributionCallback for CountingCallback {
        fn record_401(&self, consumer: crate::attribution::SamplingConsumer, sent_bearer: bool) {
            self.invocations
                .lock()
                .unwrap()
                .push((consumer, sent_bearer));
        }
    }

    #[test]
    fn live_bearer_resolver_uses_sensitive_authorization_for_messages() {
        const LIVE_INPUT: &str = "opaque-live-credential";
        const LIVE_AUTHORIZATION: &[u8] = b"Bearer opaque-live-credential";

        let cfg = SamplerConfig {
            api_key: Some("opaque-static-credential".to_string()),
            api_backend: ApiBackend::Messages,
            auth_scheme: AuthScheme::Bearer,
            bearer_resolver: Some(std::sync::Arc::new(StaticBearerResolver(LIVE_INPUT))),
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        let request = client
            .post("https://example.test/v1/messages")
            .expect("apply request auth")
            .builder
            .build()
            .expect("request should build");
        let authorization = request
            .headers()
            .get(AUTHORIZATION)
            .expect("live authorization should be present");
        assert!(authorization.is_sensitive());
        assert!(authorization.as_bytes() == LIVE_AUTHORIZATION);
        assert!(request.headers().get("x-api-key").is_none());
    }

    /// Regression: when `api_key` (which seeds `default_headers` with an
    /// `Authorization: Bearer ...`) AND a `bearer_resolver` are both set,
    /// `post()` must produce **exactly one** `Authorization` header on the
    /// wire. The pre-fix code used `RequestBuilder::header(AUTHORIZATION, ...)`
    /// which appends rather than replaces, causing two identical
    /// `Authorization` headers and a 400 from cli-chat-proxy.
    #[test]
    fn post_emits_one_sensitive_authorization_with_live_bearer() {
        const LIVE_INPUT: &str = "opaque-live-credential";
        const LIVE_AUTHORIZATION: &[u8] = b"Bearer opaque-live-credential";

        let cfg = SamplerConfig {
            api_key: Some("opaque-static-credential".to_string()),
            api_backend: ApiBackend::Responses,
            auth_scheme: AuthScheme::Bearer,
            bearer_resolver: Some(std::sync::Arc::new(StaticBearerResolver(LIVE_INPUT))),
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        let request = client
            .post("https://example.test/v1/responses")
            .expect("apply request auth")
            .builder
            .build()
            .expect("request should build");
        assert_eq!(request.headers().get_all(AUTHORIZATION).iter().count(), 1);
        let authorization = request
            .headers()
            .get(AUTHORIZATION)
            .expect("live authorization should be present");
        assert!(authorization.is_sensitive());
        assert!(authorization.as_bytes() == LIVE_AUTHORIZATION);
    }

    #[test]
    fn live_bearer_resolver_uses_sensitive_x_api_key_for_messages() {
        const LIVE_INPUT: &str = "opaque-live-credential";
        const LIVE_API_KEY: &[u8] = b"opaque-live-credential";

        let cfg = SamplerConfig {
            api_key: Some("opaque-static-credential".to_string()),
            api_backend: ApiBackend::Messages,
            auth_scheme: AuthScheme::XApiKey,
            bearer_resolver: Some(std::sync::Arc::new(StaticBearerResolver(LIVE_INPUT))),
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        let request = client
            .post("https://example.test/v1/messages")
            .expect("apply request auth")
            .builder
            .build()
            .expect("request should build");
        let api_key = request
            .headers()
            .get("x-api-key")
            .expect("live API-key authentication should be present");
        assert!(api_key.is_sensitive());
        assert!(api_key.as_bytes() == LIVE_API_KEY);
        assert!(request.headers().get(AUTHORIZATION).is_none());
    }

    /// 401 attribution retains the endpoint signal without crossing any
    /// credential material into telemetry callbacks.
    #[test]
    fn record_401_attribution_never_exposes_credential_material() {
        let cb = std::sync::Arc::new(CountingCallback::default());
        let cb_dyn: crate::attribution::SharedAttributionCallback = cb.clone();
        let cfg = SamplerConfig {
            api_key: Some("the-bearer-1234567890-extra-tail".to_string()),
            api_backend: ApiBackend::ChatCompletions,
            attribution_callback: Some(cb_dyn),
            bearer_resolver: None,
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        client.record_401_attribution(crate::attribution::SamplingConsumer::ChatCompletionsStream);
        let calls = cb.invocations.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0].0,
            crate::attribution::SamplingConsumer::ChatCompletionsStream
        );
        assert!(calls[0].1);
    }

    /// Regression test: when a bearer_resolver is wired, `post()` must
    /// *replace* the Authorization header from `default_headers`, not
    /// append a second one. Duplicate Authorization headers cause
    /// Cloudflare to return 400 Bad Request.
    #[test]
    fn bearer_resolver_replacement_remains_sensitive() {
        const LIVE_INPUT: &str = "opaque-live-credential";
        const LIVE_AUTHORIZATION: &[u8] = b"Bearer opaque-live-credential";

        #[derive(Debug)]
        struct StaticResolver(String);
        impl crate::config::BearerResolver for StaticResolver {
            fn current_bearer(&self) -> Option<String> {
                Some(self.0.clone())
            }
        }

        let resolver: crate::config::SharedBearerResolver =
            std::sync::Arc::new(StaticResolver(LIVE_INPUT.to_string()));
        let cfg = SamplerConfig {
            api_key: Some("opaque-static-credential".to_string()),
            api_backend: ApiBackend::Responses,
            bearer_resolver: Some(resolver),
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");

        let request = client
            .post("https://example.test/v1/responses")
            .expect("apply request auth")
            .builder
            .body("")
            .build()
            .expect("request should build");

        assert_eq!(request.headers().get_all(AUTHORIZATION).iter().count(), 1);
        let authorization = request
            .headers()
            .get(AUTHORIZATION)
            .expect("live authorization should be present");
        assert!(authorization.is_sensitive());
        assert!(authorization.as_bytes() == LIVE_AUTHORIZATION);
    }

    /// `record_401_attribution` is a no-op when `attribution_callback`
    /// is `None` (the BYOK / sampler-only path). The previous tests
    /// in this module construct clients without a callback and rely
    /// on this property holding.
    #[test]
    fn record_401_attribution_is_noop_without_callback() {
        let cfg = SamplerConfig {
            api_key: Some("bearer".to_string()),
            api_backend: ApiBackend::ChatCompletions,
            attribution_callback: None,
            bearer_resolver: None,
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        // Must not panic.
        client.record_401_attribution(crate::attribution::SamplingConsumer::ChatCompletions);
    }

    /// `response.completed` carrying
    /// `usage.context_details.{input_tokens, output_tokens}` rewrites
    /// `usage.total_tokens` in place to the live context length
    /// (`ctx.input + ctx.output`). Billing fields stay on the wire's
    /// cumulative values.
    #[test]
    fn deserialize_response_event_overrides_total_tokens_from_context_details() {
        let sse = r#"{
            "type": "response.completed",
            "sequence_number": 0,
            "response": {
                "id": "resp_1",
                "object": "response",
                "created_at": 0,
                "model": "grok-build",
                "status": "completed",
                "output": [],
                "usage": {
                    "input_tokens": 6003,
                    "input_tokens_details": { "cached_tokens": 1984 },
                    "output_tokens": 711,
                    "output_tokens_details": { "reasoning_tokens": 388 },
                    "total_tokens": 6714,
                    "context_details": {
                        "input_tokens": 5022,
                        "output_tokens": 571
                    }
                }
            }
        }"#;
        let event = deserialize_response_event(sse).expect("parse");
        let rs::ResponseStreamEvent::ResponseCompleted(e) = event else {
            panic!("expected ResponseCompleted");
        };
        let usage = e.response.usage.expect("usage present");
        // Billing fields stay cumulative — unchanged by context_details.
        assert_eq!(usage.input_tokens, 6003);
        assert_eq!(usage.output_tokens, 711);
        assert_eq!(usage.input_tokens_details.cached_tokens, 1984);
        assert_eq!(usage.output_tokens_details.reasoning_tokens, 388);
        // total_tokens rewritten to ctx.input + ctx.output (5022 + 571).
        // NOT the wire's cumulative total (6714).
        assert_eq!(usage.total_tokens, 5_593);
    }

    #[test]
    fn deserialize_response_event_stashes_cost_in_metadata() {
        let make = |ticks: i64| {
            format!(
                r#"{{
                "type": "response.completed",
                "sequence_number": 0,
                "response": {{
                    "id": "resp_1", "object": "response", "created_at": 0,
                    "model": "grok-build", "status": "completed", "output": [],
                    "usage": {{
                        "input_tokens": 10,
                        "input_tokens_details": {{ "cached_tokens": 0 }},
                        "output_tokens": 5,
                        "output_tokens_details": {{ "reasoning_tokens": 0 }},
                        "total_tokens": 15,
                        "cost_in_usd_ticks": {ticks}
                    }}
                }}
            }}"#
            )
        };

        let event = deserialize_response_event(&make(78)).expect("parse");
        let rs::ResponseStreamEvent::ResponseCompleted(e) = event else {
            panic!("expected ResponseCompleted");
        };
        assert_eq!(
            e.response
                .metadata
                .as_ref()
                .and_then(|m| m.get(COST_USD_TICKS_METADATA_KEY))
                .map(String::as_str),
            Some("78")
        );

        // The REST mapper backfills 0 for unbilled requests: no stash.
        let event = deserialize_response_event(&make(0)).expect("parse");
        let rs::ResponseStreamEvent::ResponseCompleted(e) = event else {
            panic!("expected ResponseCompleted");
        };
        assert!(e.response.metadata.is_none());
    }

    #[test]
    fn deserialize_response_event_total_tokens_unchanged_when_context_details_absent() {
        // Older / non-Responses backends omit `context_details`.
        // `total_tokens` passes through from the wire unchanged.
        let sse = r#"{
            "type": "response.completed",
            "sequence_number": 0,
            "response": {
                "id": "resp_1",
                "object": "response",
                "created_at": 0,
                "model": "grok-build",
                "status": "completed",
                "output": [],
                "usage": {
                    "input_tokens": 10000,
                    "input_tokens_details": { "cached_tokens": 0 },
                    "output_tokens": 100,
                    "output_tokens_details": { "reasoning_tokens": 0 },
                    "total_tokens": 10100
                }
            }
        }"#;
        let event = deserialize_response_event(sse).expect("parse");
        let rs::ResponseStreamEvent::ResponseCompleted(e) = event else {
            panic!("expected ResponseCompleted");
        };
        let usage = e.response.usage.expect("usage present");
        assert_eq!(usage.total_tokens, 10_100);
    }

    #[test]
    fn deserialize_response_event_total_tokens_unchanged_when_context_details_partial() {
        // Defensive: if the backend ever ships only one of the two
        // context_details fields, we don't have a complete picture of
        // the live context size, so leave `total_tokens` on the wire's
        // cumulative value instead of guessing (treating the missing
        // half as 0 would silently under-report).
        let sse = r#"{
            "type": "response.completed",
            "sequence_number": 0,
            "response": {
                "id": "resp_1",
                "object": "response",
                "created_at": 0,
                "model": "grok-build",
                "status": "completed",
                "output": [],
                "usage": {
                    "input_tokens": 6003,
                    "input_tokens_details": { "cached_tokens": 1984 },
                    "output_tokens": 711,
                    "output_tokens_details": { "reasoning_tokens": 388 },
                    "total_tokens": 6714,
                    "context_details": {
                        "input_tokens": 5022
                    }
                }
            }
        }"#;
        let event = deserialize_response_event(sse).expect("parse");
        let rs::ResponseStreamEvent::ResponseCompleted(e) = event else {
            panic!("expected ResponseCompleted");
        };
        let usage = e.response.usage.expect("usage present");
        assert_eq!(usage.total_tokens, 6_714);
    }

    #[test]
    fn deserialize_response_event_ignores_context_details_on_non_terminal_events() {
        // Non-terminal events don't carry final usage; even if the backend ever
        // echoed `context_details` on one, we don't touch it.
        let sse = r#"{
            "type": "response.output_text.delta",
            "sequence_number": 0,
            "item_id": "item-1",
            "output_index": 0,
            "content_index": 0,
            "delta": "hello",
            "logprobs": []
        }"#;
        let event = deserialize_response_event(sse).expect("non-terminal event parses");
        assert!(matches!(
            event,
            rs::ResponseStreamEvent::ResponseOutputTextDelta(_)
        ));
    }
}
