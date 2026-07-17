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

use std::collections::HashMap;

use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use reqwest::StatusCode;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
};
use serde::Serialize;

use xai_grok_sampling_types::error::{parse_error_bytes, try_parse_stream_error};
use xai_grok_sampling_types::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ConversationRequest,
    ConversationResponse, CreateResponseWrapper, CredentialBinding, CredentialSourceId,
    DOOM_LOOP_CHECK_HEADER, MessagesRequestWrapper, OPENAI_CODEX_COMPATIBILITY_VERSION,
    OPENAI_CODEX_RESPONSES_LITE_HEADER, ProviderId, ReasoningEffort, ResponseModelMetadata, Result,
    SamplingError, build_messages_request, is_check_event, messages, rs,
};

use crate::config::{AuthScheme, OriginClientInfo, SamplerConfig};

// Re-export ApiBackend from the shared types crate for downstream callers.
pub use xai_grok_sampling_types::ApiBackend;

/// Process-level fallback for the `x-grok-client-identifier` header.
const DEFAULT_CLIENT_IDENTIFIER: &str = "grok-shell";

/// Product identifier baked into User-Agent strings.
const AGENT_PRODUCT: &str = "grok-shell";
const ANTHROPIC_DEFAULT_MAX_TOKENS: u32 = 128_000;
const CHATGPT_ACCOUNT_ID_HEADER: &str = "chatgpt-account-id";
const OPENAI_FEDRAMP_HEADER: &str = "x-openai-fedramp";
const OPENAI_CODEX_ORIGINATOR: &str = "grok_build_codex";
const CODEX_SESSION_ID_HEADER: &str = "session-id";
const CODEX_THREAD_ID_HEADER: &str = "thread-id";
const CODEX_CLIENT_REQUEST_ID_HEADER: &str = "x-client-request-id";
const CODEX_TURN_STATE_HEADER: &str = "x-codex-turn-state";

/// Normalize provider-specific reasoning semantics before serialization.
/// Historically Grok Build accepted `max` as an alias for `xhigh`; Codex now
/// advertises a genuinely distinct Max tier, so only that provider preserves
/// the new value. Ultra has never been valid outside Codex and fails closed.
fn normalize_reasoning_effort_for_provider(
    provider: ProviderId,
    effort: ReasoningEffort,
) -> Result<ReasoningEffort> {
    if provider.is_openai_codex() {
        return Ok(effort);
    }
    match effort {
        ReasoningEffort::Max => Ok(ReasoningEffort::Xhigh),
        ReasoningEffort::Ultra => Err(SamplingError::InvalidConfiguration(
            "ultra reasoning effort requires the OpenAI Codex provider",
        )),
        effort => Ok(effort),
    }
}
const CODEX_DIAGNOSTIC_RESPONSE_HEADERS: &[&str] = &[
    "content-length",
    "content-type",
    "retry-after",
    "x-request-id",
    "x-should-retry",
];

/// Inject Codex reasoning tiers newer than the pinned `async-openai` request
/// type at the final JSON boundary. Current openai/codex treats `ultra` as a
/// client-side orchestration policy and sends `max` to the backend, so this
/// preserved Grok loop mirrors that wire contract without importing Codex's
/// app-server/multi-agent runtime. Keeping this provider-scoped prevents an
/// xAI/custom Responses request from receiving an unsupported tier.
fn apply_extended_codex_reasoning_effort(
    provider: ProviderId,
    effort: Option<ReasoningEffort>,
    body: &mut serde_json::Value,
) -> Result<()> {
    let Some(effort @ (ReasoningEffort::Max | ReasoningEffort::Ultra)) = effort else {
        return Ok(());
    };
    if !provider.is_openai_codex() && effort == ReasoningEffort::Ultra {
        return Err(SamplingError::InvalidConfiguration(
            "ultra Responses reasoning effort requires the OpenAI Codex provider",
        ));
    }

    let Some(root) = body.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Responses request body must be a JSON object",
        ));
    };
    let reasoning = root
        .entry("reasoning")
        .or_insert_with(|| serde_json::Value::Object(Default::default()));
    let Some(reasoning) = reasoning.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Responses reasoning configuration must be a JSON object",
        ));
    };
    let wire_effort = match (provider, effort) {
        (provider, ReasoningEffort::Max) if !provider.is_openai_codex() => ReasoningEffort::Xhigh,
        (_, ReasoningEffort::Ultra) => ReasoningEffort::Max,
        (_, effort) => effort,
    };
    reasoning.insert(
        "effort".to_string(),
        serde_json::Value::String(wire_effort.as_str().to_string()),
    );
    Ok(())
}

/// Apply the Codex Responses transport contract at the final provider JSON
/// boundary. The stable prompt cache key belongs to every Codex Responses
/// request; the remaining rewrite is gated by Responses Lite so Grok Build's
/// conversation, tool registry, persistence, and execution loop remain intact.
fn apply_codex_responses_lite_contract(
    provider: ProviderId,
    enabled: bool,
    prompt_cache_key: Option<&str>,
    body: &mut serde_json::Value,
) -> Result<()> {
    if !provider.is_openai_codex() {
        if !enabled {
            return Ok(());
        }
        return Err(SamplingError::InvalidConfiguration(
            "Responses Lite requires the OpenAI Codex provider",
        ));
    }

    let Some(root) = body.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Responses request body must be a JSON object",
        ));
    };

    // Responses Lite rejects sampling temperature for current Codex models.
    // The ordinary Grok defaults can populate it for auxiliary calls (notably
    // title generation), so omit it at this provider-specific wire boundary.
    root.remove("temperature");
    if let Some(prompt_cache_key) = prompt_cache_key.filter(|value| !value.is_empty()) {
        root.insert(
            "prompt_cache_key".to_string(),
            serde_json::Value::String(prompt_cache_key.to_string()),
        );
    }
    if !enabled {
        return Ok(());
    }

    let mut tools = match root.remove("tools") {
        None | Some(serde_json::Value::Null) => Vec::new(),
        Some(serde_json::Value::Array(tools)) => tools,
        Some(_) => {
            return Err(SamplingError::InvalidConfiguration(
                "Responses tools must be a JSON array",
            ));
        }
    };
    // The Codex client deliberately sends non-strict function definitions.
    // Preserve newer tool kinds verbatim, but normalize ordinary function
    // tools to the exact Responses Lite shape.
    for tool in &mut tools {
        if tool.get("type").and_then(serde_json::Value::as_str) == Some("function")
            && let Some(tool) = tool.as_object_mut()
        {
            tool.insert("strict".to_string(), serde_json::Value::Bool(false));
        }
    }
    let has_tools = !tools.is_empty();
    let instructions = match root.remove("instructions") {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(instructions)) if instructions.is_empty() => None,
        Some(serde_json::Value::String(instructions)) => Some(instructions),
        Some(_) => {
            return Err(SamplingError::InvalidConfiguration(
                "Responses instructions must be a string",
            ));
        }
    };

    let mut input = match root.remove("input") {
        None | Some(serde_json::Value::Null) => Vec::new(),
        Some(serde_json::Value::Array(input)) => input,
        Some(serde_json::Value::String(text)) => vec![serde_json::json!({
            "type": "message",
            "role": "user",
            "content": [{"type": "input_text", "text": text}],
        })],
        Some(_) => {
            return Err(SamplingError::InvalidConfiguration(
                "Responses Lite input must be text or an item array",
            ));
        }
    };
    for item in &mut input {
        if item.get("role").and_then(serde_json::Value::as_str) == Some("system")
            && let Some(item) = item.as_object_mut()
        {
            item.insert(
                "role".to_string(),
                serde_json::Value::String("developer".to_string()),
            );
        }
        strip_input_image_details(item);
    }

    let mut prefix = vec![serde_json::json!({
        "type": "additional_tools",
        "role": "developer",
        "tools": tools,
    })];
    if let Some(instructions) = instructions {
        prefix.push(serde_json::json!({
            "type": "message",
            "role": "developer",
            "content": [{"type": "input_text", "text": instructions}],
        }));
    }
    prefix.append(&mut input);
    root.insert("input".to_string(), serde_json::Value::Array(prefix));
    root.insert(
        "parallel_tool_calls".to_string(),
        serde_json::Value::Bool(false),
    );
    // Responses Lite expects the same explicit string used by the current
    // openai/codex client. Omitting this field can make `additional_tools`
    // advisory only, allowing the model to stop after a planning preamble
    // instead of entering Grok Build's function-tool loop.
    if has_tools {
        root.insert(
            "tool_choice".to_string(),
            serde_json::Value::String("auto".to_string()),
        );
    } else {
        // Auxiliary calls (for example title generation) intentionally have
        // no tool registry. Responses Lite rejects `auto` when no callable
        // tool exists, so leave the field absent for those requests.
        root.remove("tool_choice");
    }

    let reasoning = root
        .entry("reasoning")
        .or_insert_with(|| serde_json::Value::Object(Default::default()));
    let Some(reasoning) = reasoning.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Responses reasoning configuration must be a JSON object",
        ));
    };
    reasoning.insert(
        "context".to_string(),
        serde_json::Value::String("all_turns".to_string()),
    );
    Ok(())
}

fn codex_prompt_cache_key<'a>(conversation_id: &'a str, session_id: &'a str) -> Option<&'a str> {
    (!conversation_id.is_empty())
        .then_some(conversation_id)
        .or_else(|| (!session_id.is_empty()).then_some(session_id))
}

fn strip_input_image_details(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            if object.get("type").and_then(serde_json::Value::as_str) == Some("input_image") {
                object.remove("detail");
            }
            for value in object.values_mut() {
                strip_input_image_details(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                strip_input_image_details(value);
            }
        }
        _ => {}
    }
}

/// The pinned `async-openai` version cannot deserialize the newer Codex `max`/`ultra`
/// values echoed in `response.reasoning.effort`. Normalize only that typed
/// compatibility seam. Preserve the raw value in response metadata so the
/// conversation records true `max`/`ultra` provenance rather than falsely
/// claiming `xhigh`. `ultra` follows current Codex behavior and maps to `max`
/// on the request wire.
fn normalize_extended_codex_response_effort(value: &mut serde_json::Value) {
    fn normalize_response_object(object: &mut serde_json::Map<String, serde_json::Value>) {
        let extended = object
            .get("reasoning")
            .and_then(serde_json::Value::as_object)
            .and_then(|reasoning| reasoning.get("effort"))
            .and_then(serde_json::Value::as_str)
            .filter(|effort| matches!(*effort, "max" | "ultra"))
            .map(str::to_owned);
        let Some(extended) = extended else {
            return;
        };
        let metadata = object
            .entry("metadata")
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if !metadata.is_object() {
            *metadata = serde_json::Value::Object(serde_json::Map::new());
        }
        if let Some(metadata) = metadata.as_object_mut() {
            metadata.insert(
                xai_grok_sampling_types::OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY
                    .to_owned(),
                serde_json::Value::String(extended),
            );
        }
        if let Some(effort) = object
            .get_mut("reasoning")
            .and_then(serde_json::Value::as_object_mut)
            .and_then(|reasoning| reasoning.get_mut("effort"))
        {
            *effort = serde_json::Value::String("xhigh".to_string());
        }
    }

    if let Some(object) = value.as_object_mut() {
        normalize_response_object(object);
    }
    if let Some(object) = value
        .get_mut("response")
        .and_then(serde_json::Value::as_object_mut)
    {
        normalize_response_object(object);
    }
}

fn valid_openai_codex_base_url(base_url: &str) -> bool {
    let production =
        base_url.trim_end_matches('/') == xai_grok_sampling_types::OPENAI_CODEX_BASE_URL;
    #[cfg(any(test, feature = "test-support"))]
    let loopback = reqwest::Url::parse(base_url).ok().is_some_and(|url| {
        url.scheme() == "http"
            && url.query().is_none()
            && url.fragment().is_none()
            && url
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"))
    });
    #[cfg(not(any(test, feature = "test-support")))]
    let loopback = false;
    production || loopback
}

fn is_protected_credential_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization"
            | "proxy-authorization"
            | "x-api-key"
            | CHATGPT_ACCOUNT_ID_HEADER
            | "openai-organization"
            | "openai-project"
            | "x-openai-actor-authorization"
            | OPENAI_FEDRAMP_HEADER
            | "cookie"
            | "x-xai-token-auth"
            | "x-userid"
            | "x-email"
    )
}

fn is_allowed_codex_credential_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization" | CHATGPT_ACCOUNT_ID_HEADER | OPENAI_FEDRAMP_HEADER
    )
}

/// Detect credential-bearing aliases that must never escape a provider auth
/// hook. Header names are case-insensitive already; removing punctuation also
/// catches variants such as `x-api-key`, `x-apikey`, and `x-api-key-backup`.
///
/// This intentionally does not treat request identity/correlation headers as
/// credentials. Codex protocol headers are sealed separately after auth and
/// ordinary transport headers such as `traceparent` remain available.
fn is_codex_credential_header_or_alias(name: &HeaderName) -> bool {
    if is_protected_credential_header(name) {
        return true;
    }

    let compact: String = name
        .as_str()
        .bytes()
        .filter(u8::is_ascii_alphanumeric)
        .map(char::from)
        .collect();

    compact.contains("authorization")
        || compact.contains("authentication")
        || compact.contains("apikey")
        || compact.contains("token")
        || compact.contains("secret")
        || compact.contains("credential")
        || compact.contains("cookie")
        || compact.contains("account")
        || compact.contains("organization")
        || compact.contains("project")
}

fn validate_codex_request_auth_headers(headers: &mut HeaderMap) -> Result<()> {
    if headers.keys().any(|name| {
        is_codex_credential_header_or_alias(name) && !is_allowed_codex_credential_header(name)
    }) {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted an unsupported credential header",
        ));
    }
    if headers.keys().any(is_xai_specific_header) {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted an xAI-specific header",
        ));
    }

    let authorization_count = headers.get_all(AUTHORIZATION).iter().count();
    if authorization_count == 0 {
        return Err(SamplingError::Auth(
            "OpenAI Codex authorization is unavailable".to_string(),
        ));
    }
    if authorization_count != 1 {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted duplicate credential headers",
        ));
    }
    let has_bearer = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| !token.is_empty());
    if !has_bearer {
        return Err(SamplingError::Auth(
            "OpenAI Codex authorization is unavailable".to_string(),
        ));
    }

    let account_count = headers.get_all(CHATGPT_ACCOUNT_ID_HEADER).iter().count();
    if account_count == 0 {
        return Err(SamplingError::Auth(
            "OpenAI Codex account selection is unavailable".to_string(),
        ));
    }
    if account_count != 1 {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted duplicate credential headers",
        ));
    }
    let has_account = headers
        .get(CHATGPT_ACCOUNT_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|account_id| !account_id.trim().is_empty());
    if !has_account {
        return Err(SamplingError::Auth(
            "OpenAI Codex account selection is unavailable".to_string(),
        ));
    }

    let fedramp_values = headers.get_all(OPENAI_FEDRAMP_HEADER);
    let fedramp_count = fedramp_values.iter().count();
    if fedramp_count > 1 {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted duplicate credential headers",
        ));
    }
    if fedramp_values
        .iter()
        .next()
        .is_some_and(|value| value.as_bytes() != b"true")
    {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex FedRAMP authentication state must be exactly true when present",
        ));
    }

    // These values are sensitive even when a RequestAuth implementation did
    // not mark them itself. This prevents future request-debug output from
    // exposing either the bearer token or selected ChatGPT account.
    if let Some(value) = headers.get_mut(AUTHORIZATION) {
        value.set_sensitive(true);
    }
    if let Some(value) = headers.get_mut(CHATGPT_ACCOUNT_ID_HEADER) {
        value.set_sensitive(true);
    }
    if let Some(value) = headers.get_mut(OPENAI_FEDRAMP_HEADER) {
        value.set_sensitive(true);
    }

    Ok(())
}

fn validate_codex_credential_binding(binding: &CredentialBinding) -> Result<()> {
    if binding.provider != ProviderId::OpenAiCodex
        || binding.source != CredentialSourceId::OpenAiCodexSubscription
        || binding
            .record_id
            .as_deref()
            .is_none_or(|record_id| record_id.trim().is_empty())
        || binding.generation == 0
    {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication omitted its credential generation",
        ));
    }
    Ok(())
}

fn is_xai_specific_header(name: &HeaderName) -> bool {
    let name = name.as_str();
    name.starts_with("x-grok-")
        || name.starts_with("x-xai-")
        || matches!(name, "x-api-key" | "x-userid" | "x-email")
}

fn is_codex_provider_identity_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "originator"
            | "version"
            | OPENAI_CODEX_RESPONSES_LITE_HEADER
            | CODEX_SESSION_ID_HEADER
            | CODEX_THREAD_ID_HEADER
            | CODEX_CLIENT_REQUEST_ID_HEADER
            | CODEX_TURN_STATE_HEADER
    )
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

    fn apply_codex(
        &self,
        builder: reqwest::RequestBuilder,
        responses_lite: bool,
    ) -> Result<reqwest::RequestBuilder> {
        let sensitive_value = |raw: &str| {
            HeaderValue::from_str(raw)
                .map(|mut value| {
                    value.set_sensitive(true);
                    value
                })
                .map_err(|_| {
                    SamplingError::InvalidConfiguration(
                        "OpenAI Codex request identity contains an invalid header value",
                    )
                })
        };
        let mut builder = builder;
        if !self.session_id.is_empty() {
            builder = builder.header(CODEX_SESSION_ID_HEADER, sensitive_value(self.session_id)?);
        }
        if !self.conv_id.is_empty() {
            builder = builder
                .header(CODEX_THREAD_ID_HEADER, sensitive_value(self.conv_id)?)
                .header(
                    CODEX_CLIENT_REQUEST_ID_HEADER,
                    sensitive_value(self.conv_id)?,
                );
        }
        if responses_lite {
            builder = builder.header(OPENAI_CODEX_RESPONSES_LITE_HEADER, "true");
        }
        Ok(builder)
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
#[cfg(test)]
fn deserialize_response_event(data: &str) -> Result<rs::ResponseStreamEvent> {
    deserialize_response_event_for_provider(data, false)
}

/// Stateful compatibility decoder for the deliberately sparse Responses SSE
/// frames emitted by the ChatGPT Codex backend.
///
/// The pinned `async-openai` stream types model the public API's fully
/// populated envelopes. The Codex client protocol instead omits bookkeeping
/// fields such as sequence/output indexes, and its terminal response commonly
/// contains only an id and usage. Keep that tolerance provider-scoped: xAI
/// continues through the strict decoder below.
#[derive(Debug)]
struct CodexSseDecoder {
    fallback_model: String,
    next_sequence_number: u64,
    next_output_index: u32,
    item_output_indexes: HashMap<String, u32>,
    active_output_index: Option<u32>,
    active_item_id: Option<String>,
}

/// Private typed-response metadata seam for the sparse Codex terminal flag.
/// The pinned Responses type has no `end_turn` field; the layer-2 stream
/// transformer removes this value before producing the public response.
pub(crate) const CODEX_END_TURN_METADATA_KEY: &str = "__grok_codex_end_turn";

impl CodexSseDecoder {
    fn new(fallback_model: impl Into<String>) -> Self {
        let fallback_model = fallback_model.into();
        Self {
            fallback_model: if fallback_model.is_empty() {
                "openai-codex".to_string()
            } else {
                fallback_model
            },
            next_sequence_number: 0,
            next_output_index: 0,
            item_output_indexes: HashMap::new(),
            active_output_index: None,
            active_item_id: None,
        }
    }

    /// Decode one raw Codex data frame. Unknown non-terminal frames (including
    /// `response.metadata`) are intentionally ignored, matching the official
    /// client's forward-compatible behavior. No provider-controlled value is
    /// included in an error or log message.
    fn decode(&mut self, data: &str) -> Result<Option<rs::ResponseStreamEvent>> {
        let mut value = serde_json::from_str::<serde_json::Value>(data)
            .map_err(|_| codex_invalid_stream_event())?;

        if value
            .get("error")
            .is_some_and(|error| error.is_object() || error.is_string())
        {
            return Err(codex_rejected_stream());
        }

        let kind = value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(codex_invalid_stream_event)?
            .to_owned();

        if kind == "error" {
            return Err(codex_rejected_stream());
        }
        if kind == "response.failed" {
            return Err(codex_failed_stream());
        }
        if kind == "response.incomplete" {
            return Err(codex_incomplete_stream());
        }

        let supported = matches!(
            kind.as_str(),
            "response.created"
                | "response.in_progress"
                | "response.queued"
                | "response.completed"
                | "response.output_item.added"
                | "response.output_item.done"
                | "response.output_text.delta"
                | "response.output_text.done"
                | "response.function_call_arguments.delta"
                | "response.function_call_arguments.done"
                | "response.reasoning_summary_text.delta"
                | "response.reasoning_summary_text.done"
                | "response.reasoning_text.delta"
                | "response.reasoning_text.done"
                | "response.custom_tool_call_input.delta"
                | "response.custom_tool_call_input.done"
                | "response.web_search_call.in_progress"
                | "response.web_search_call.searching"
                | "response.web_search_call.completed"
                | "response.image_generation_call.in_progress"
                | "response.image_generation_call.generating"
                | "response.image_generation_call.completed"
                | "response.image_generation_call.partial_image"
        );
        if !supported {
            return Ok(None);
        }

        let sequence_number = self.sequence_number(&value);
        let object = value
            .as_object_mut()
            .ok_or_else(codex_invalid_stream_event)?;
        object.insert(
            "sequence_number".to_string(),
            serde_json::Value::from(sequence_number),
        );

        match kind.as_str() {
            "response.created"
            | "response.in_progress"
            | "response.queued"
            | "response.completed" => {
                let status = match kind.as_str() {
                    "response.completed" => "completed",
                    "response.queued" => "queued",
                    _ => "in_progress",
                };
                let response = object
                    .get_mut("response")
                    .ok_or_else(codex_invalid_stream_event)?;
                normalize_codex_response(response, &self.fallback_model, status)?;
            }
            "response.output_item.added" | "response.output_item.done" => {
                let output_index = self.item_output_index(&value);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                object.insert(
                    "output_index".to_string(),
                    serde_json::Value::from(output_index),
                );
                let item = object
                    .get_mut("item")
                    .ok_or_else(codex_invalid_stream_event)?;
                let item_status = if kind == "response.output_item.done" {
                    "completed"
                } else {
                    "in_progress"
                };
                normalize_codex_output_item(item, output_index, item_status)?;
                self.remember_item_aliases(&value, output_index);
                self.active_output_index = Some(output_index);
                self.active_item_id = primary_codex_item_id(&value)
                    .or_else(|| Some(format!("codex-item-{output_index}")));
            }
            "response.output_text.delta" | "response.output_text.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "content_index", 0);
                insert_string(object, "item_id", item_id);
                if kind == "response.output_text.delta" {
                    insert_string_if_missing(object, "delta", "");
                    insert_null_if_missing(object, "logprobs");
                } else {
                    insert_string_if_missing(object, "text", "");
                    insert_null_if_missing(object, "logprobs");
                }
            }
            "response.function_call_arguments.delta" | "response.function_call_arguments.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_string(object, "item_id", item_id);
                if kind == "response.function_call_arguments.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "arguments", "");
                }
            }
            "response.reasoning_summary_text.delta" | "response.reasoning_summary_text.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "summary_index", 0);
                insert_string(object, "item_id", item_id);
                if kind == "response.reasoning_summary_text.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "text", "");
                }
            }
            "response.reasoning_text.delta" | "response.reasoning_text.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "content_index", 0);
                insert_string(object, "item_id", item_id);
                if kind == "response.reasoning_text.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "text", "");
                }
            }
            "response.custom_tool_call_input.delta" | "response.custom_tool_call_input.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_string(object, "item_id", item_id);
                if kind == "response.custom_tool_call_input.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "input", "");
                }
            }
            "response.web_search_call.in_progress"
            | "response.web_search_call.searching"
            | "response.web_search_call.completed"
            | "response.image_generation_call.in_progress"
            | "response.image_generation_call.generating"
            | "response.image_generation_call.completed" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_string(object, "item_id", item_id);
            }
            "response.image_generation_call.partial_image" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(codex_invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "partial_image_index", 0);
                insert_string(object, "item_id", item_id);
                insert_string_if_missing(object, "partial_image_b64", "");
            }
            _ => return Ok(None),
        }

        normalize_extended_codex_response_effort(&mut value);
        let mut event = serde_json::from_value::<rs::ResponseStreamEvent>(value)
            .map_err(|_| codex_invalid_stream_event())?;
        apply_terminal_event_overrides(&mut event, data);
        Ok(Some(event))
    }

    fn sequence_number(&mut self, value: &serde_json::Value) -> u64 {
        if let Some(sequence_number) = value
            .get("sequence_number")
            .and_then(serde_json::Value::as_u64)
        {
            self.next_sequence_number = self
                .next_sequence_number
                .max(sequence_number.saturating_add(1));
            sequence_number
        } else {
            let sequence_number = self.next_sequence_number;
            self.next_sequence_number = self.next_sequence_number.saturating_add(1);
            sequence_number
        }
    }

    fn item_output_index(&mut self, value: &serde_json::Value) -> u32 {
        let explicit = json_u32(value.get("output_index"));
        let aliases = codex_item_aliases(value);
        let output_index = explicit
            .or_else(|| {
                aliases
                    .iter()
                    .find_map(|alias| self.item_output_indexes.get(alias).copied())
            })
            .unwrap_or_else(|| self.allocate_output_index());
        self.advance_output_index(output_index);
        for alias in aliases {
            self.item_output_indexes.insert(alias, output_index);
        }
        output_index
    }

    fn delta_output_index(&mut self, value: &serde_json::Value) -> u32 {
        let explicit = json_u32(value.get("output_index"));
        let aliases = codex_item_aliases(value);
        let output_index = explicit
            .or_else(|| {
                aliases
                    .iter()
                    .find_map(|alias| self.item_output_indexes.get(alias).copied())
            })
            .or(self.active_output_index)
            .unwrap_or_else(|| self.allocate_output_index());
        self.advance_output_index(output_index);
        for alias in aliases {
            self.item_output_indexes.insert(alias, output_index);
        }
        self.active_output_index = Some(output_index);
        output_index
    }

    fn allocate_output_index(&mut self) -> u32 {
        let output_index = self.next_output_index;
        self.next_output_index = self.next_output_index.saturating_add(1);
        output_index
    }

    fn advance_output_index(&mut self, output_index: u32) {
        self.next_output_index = self.next_output_index.max(output_index.saturating_add(1));
    }

    fn remember_item_aliases(&mut self, value: &serde_json::Value, output_index: u32) {
        for alias in codex_item_aliases(value) {
            self.item_output_indexes.insert(alias, output_index);
        }
    }

    fn item_id(&mut self, value: &serde_json::Value, output_index: u32) -> String {
        let item_id = primary_codex_item_id(value)
            .or_else(|| self.active_item_id.clone())
            .unwrap_or_else(|| format!("codex-item-{output_index}"));
        self.item_output_indexes
            .insert(item_id.clone(), output_index);
        self.active_item_id = Some(item_id.clone());
        item_id
    }
}

fn codex_invalid_stream_event() -> SamplingError {
    SamplingError::serialization_message("ChatGPT Codex stream event was invalid")
}

fn codex_rejected_stream() -> SamplingError {
    SamplingError::StreamError {
        error_type: "provider_error".to_string(),
        message: "ChatGPT Codex stream rejected".to_string(),
    }
}

fn codex_failed_stream() -> SamplingError {
    SamplingError::StreamError {
        error_type: "provider_error".to_string(),
        message: "ChatGPT Codex response failed".to_string(),
    }
}

fn codex_incomplete_stream() -> SamplingError {
    SamplingError::StreamError {
        error_type: "provider_error".to_string(),
        message: "ChatGPT Codex response was incomplete".to_string(),
    }
}

fn json_u32(value: Option<&serde_json::Value>) -> Option<u32> {
    value
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn codex_item_aliases(value: &serde_json::Value) -> Vec<String> {
    [
        value.pointer("/item/id"),
        value.pointer("/item/call_id"),
        value.get("item_id"),
        value.get("call_id"),
    ]
    .into_iter()
    .flatten()
    .filter_map(serde_json::Value::as_str)
    .filter(|value| !value.is_empty())
    .map(str::to_owned)
    .collect()
}

fn primary_codex_item_id(value: &serde_json::Value) -> Option<String> {
    codex_item_aliases(value).into_iter().next()
}

fn insert_u32(object: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: u32) {
    object.insert(key.to_string(), serde_json::Value::from(value));
}

fn insert_u32_if_missing(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: u32,
) {
    if object.get(key).is_none_or(serde_json::Value::is_null) {
        insert_u32(object, key, value);
    }
}

fn insert_string(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: String,
) {
    object.insert(key.to_string(), serde_json::Value::String(value));
}

fn insert_string_if_missing(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &str,
) {
    if object.get(key).is_none_or(serde_json::Value::is_null) {
        object.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }
}

fn insert_null_if_missing(object: &mut serde_json::Map<String, serde_json::Value>, key: &str) {
    if !object.contains_key(key) {
        object.insert(key.to_string(), serde_json::Value::Null);
    }
}

fn normalize_codex_response(
    response: &mut serde_json::Value,
    fallback_model: &str,
    status: &str,
) -> Result<()> {
    let response = response
        .as_object_mut()
        .ok_or_else(codex_invalid_stream_event)?;
    insert_u32_if_missing(response, "created_at", 0);
    insert_string_if_missing(response, "id", "codex-response");
    insert_string_if_missing(response, "model", fallback_model);
    insert_string_if_missing(response, "object", "response");
    insert_string_if_missing(response, "status", status);

    // Never forward arbitrary provider metadata through the compatibility
    // envelope. Preserve only the protocol bit the Grok loop needs, encoded
    // as a string because async-openai models metadata as `Map<String,
    // String>`. Extended reasoning provenance is added separately after this
    // normalization step.
    let end_turn = response
        .remove("end_turn")
        .and_then(|value| value.as_bool());
    response.remove("metadata");
    if let Some(end_turn) = end_turn {
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            CODEX_END_TURN_METADATA_KEY.to_string(),
            serde_json::Value::String(end_turn.to_string()),
        );
        response.insert("metadata".to_string(), serde_json::Value::Object(metadata));
    }

    if response
        .get("output")
        .is_none_or(serde_json::Value::is_null)
    {
        response.insert("output".to_string(), serde_json::Value::Array(Vec::new()));
    }
    let output = response
        .get_mut("output")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(codex_invalid_stream_event)?;
    for (output_index, item) in output.iter_mut().enumerate() {
        let output_index = u32::try_from(output_index).map_err(|_| codex_invalid_stream_event())?;
        normalize_codex_output_item(item, output_index, status)?;
    }

    if let Some(usage) = response.get_mut("usage")
        && !usage.is_null()
    {
        normalize_codex_usage(usage)?;
    }
    Ok(())
}

fn normalize_codex_usage(usage: &mut serde_json::Value) -> Result<()> {
    let usage = usage
        .as_object_mut()
        .ok_or_else(codex_invalid_stream_event)?;
    insert_u32_if_missing(usage, "input_tokens", 0);
    insert_u32_if_missing(usage, "output_tokens", 0);
    insert_u32_if_missing(usage, "total_tokens", 0);

    if usage
        .get("input_tokens_details")
        .is_none_or(serde_json::Value::is_null)
    {
        usage.insert(
            "input_tokens_details".to_string(),
            serde_json::json!({"cached_tokens": 0}),
        );
    } else {
        let details = usage
            .get_mut("input_tokens_details")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(codex_invalid_stream_event)?;
        insert_u32_if_missing(details, "cached_tokens", 0);
    }

    if usage
        .get("output_tokens_details")
        .is_none_or(serde_json::Value::is_null)
    {
        usage.insert(
            "output_tokens_details".to_string(),
            serde_json::json!({"reasoning_tokens": 0}),
        );
    } else {
        let details = usage
            .get_mut("output_tokens_details")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(codex_invalid_stream_event)?;
        insert_u32_if_missing(details, "reasoning_tokens", 0);
    }
    Ok(())
}

fn normalize_codex_output_item(
    item: &mut serde_json::Value,
    output_index: u32,
    status: &str,
) -> Result<()> {
    let item = item
        .as_object_mut()
        .ok_or_else(codex_invalid_stream_event)?;
    let kind = item
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(codex_invalid_stream_event)?
        .to_owned();
    let item_id = format!("codex-item-{output_index}");

    match kind.as_str() {
        "message" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "role", "assistant");
            insert_string_if_missing(item, "status", status);
            if item.get("content").is_none_or(serde_json::Value::is_null) {
                item.insert("content".to_string(), serde_json::Value::Array(Vec::new()));
            }
            let content = item
                .get_mut("content")
                .and_then(serde_json::Value::as_array_mut)
                .ok_or_else(codex_invalid_stream_event)?;
            for part in content {
                if part.get("type").and_then(serde_json::Value::as_str) == Some("output_text") {
                    let part = part
                        .as_object_mut()
                        .ok_or_else(codex_invalid_stream_event)?;
                    if part
                        .get("annotations")
                        .is_none_or(serde_json::Value::is_null)
                    {
                        part.insert(
                            "annotations".to_string(),
                            serde_json::Value::Array(Vec::new()),
                        );
                    }
                    insert_null_if_missing(part, "logprobs");
                    insert_string_if_missing(part, "text", "");
                }
            }
        }
        "reasoning" => {
            insert_string_if_missing(item, "id", &item_id);
            if item.get("summary").is_none_or(serde_json::Value::is_null) {
                item.insert("summary".to_string(), serde_json::Value::Array(Vec::new()));
            }
        }
        "function_call" => {
            insert_string_if_missing(item, "call_id", &format!("codex-call-{output_index}"));
            insert_string_if_missing(item, "name", "");
            insert_string_if_missing(item, "arguments", "");
        }
        "custom_tool_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "call_id", &format!("codex-call-{output_index}"));
            insert_string_if_missing(item, "name", "");
            insert_string_if_missing(item, "input", "");
        }
        "image_generation_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "status", status);
            insert_null_if_missing(item, "result");
        }
        "web_search_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "status", status);
            if item.get("action").is_none_or(serde_json::Value::is_null) {
                item.insert(
                    "action".to_string(),
                    serde_json::json!({"type": "search", "query": "", "sources": null}),
                );
            }
        }
        "file_search_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "status", status);
            if item.get("queries").is_none_or(serde_json::Value::is_null) {
                item.insert("queries".to_string(), serde_json::Value::Array(Vec::new()));
            }
        }
        _ => {}
    }
    Ok(())
}

fn deserialize_response_event_for_provider(
    data: &str,
    redact_provider_payload: bool,
) -> Result<rs::ResponseStreamEvent> {
    if redact_provider_payload {
        return CodexSseDecoder::new("openai-codex")
            .decode(data)?
            .ok_or_else(codex_invalid_stream_event);
    }

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
                raw_data = %data,
                "Failed to deserialize ResponseStreamEvent from stream"
            );
            return Err(SamplingError::Serialization(first_err));
        }
    };
    apply_terminal_event_overrides(&mut event, data);
    Ok(event)
}

fn deserialize_response_for_provider(bytes: &[u8], provider: ProviderId) -> Result<rs::Response> {
    let parsed = if provider.is_openai_codex() {
        serde_json::from_slice::<serde_json::Value>(bytes).and_then(|mut value| {
            normalize_extended_codex_response_effort(&mut value);
            serde_json::from_value::<rs::Response>(value)
        })
    } else {
        serde_json::from_slice::<rs::Response>(bytes)
    };

    parsed.map_err(|error| {
        if provider.is_openai_codex() {
            tracing::error!("Failed to deserialize Codex rs::Response");
            return SamplingError::serialization_message("ChatGPT Codex response was invalid");
        }
        let raw_body = String::from_utf8_lossy(bytes);
        tracing::error!(
            error = %error,
            raw_body = %raw_body,
            "Failed to deserialize rs::Response"
        );
        SamplingError::Serialization(error)
    })
}

/// Return whether a Codex error envelope carries one of the small, fixed
/// subscription-exhaustion codes published by the open-source Codex client.
///
/// Provider-controlled messages are deliberately ignored: an error may echo
/// request text, bearer material, or account metadata. Matching only an
/// allowlist of structural codes lets callers distinguish a durable usage
/// block from a short rolling 429 without reflecting any response content.
fn is_codex_usage_limit_error(value: &serde_json::Value) -> bool {
    [
        value.pointer("/error/type"),
        value.pointer("/error/code"),
        value.get("code"),
    ]
    .into_iter()
    .flatten()
    .filter_map(serde_json::Value::as_str)
    .any(|kind| {
        matches!(
            kind,
            "usage_limit_reached"
                | "insufficient_quota"
                | "credits_depleted"
                | "workspace_owner_credits_depleted"
                | "workspace_member_credits_depleted"
                | "workspace_owner_usage_limit_reached"
                | "workspace_member_usage_limit_reached"
                | "spend_control_limit_reached"
                | "spending_limit_reached"
        )
    })
}

fn codex_usage_limit_should_retry(
    provider: ProviderId,
    status: StatusCode,
    body: &[u8],
    server_hint: Option<bool>,
) -> Option<bool> {
    if provider.is_openai_codex()
        && status == StatusCode::TOO_MANY_REQUESTS
        && serde_json::from_slice::<serde_json::Value>(body)
            .ok()
            .as_ref()
            .is_some_and(is_codex_usage_limit_error)
    {
        // Subscription exhaustion is durable until the provider's reset. The
        // current openai/codex client treats this structural error as terminal;
        // retrying it only burns another request and can leave the TUI appearing
        // to loop. Do not inspect or retain the provider-controlled message.
        Some(false)
    } else {
        server_hint
    }
}

/// Recognize the two server-error envelope shapes without logging or
/// retaining any provider-controlled string. The ChatGPT backend can reflect
/// request material, so even an error field is treated as sensitive.
fn try_parse_codex_stream_error(data: &str) -> Option<SamplingError> {
    let value = serde_json::from_str::<serde_json::Value>(data).ok()?;
    let error = value.get("error")?;
    if !(error.is_object() || error.is_string()) {
        return None;
    }
    if is_codex_usage_limit_error(&value) {
        return Some(SamplingError::StreamError {
            error_type: "usage_limit_reached".to_owned(),
            message: "ChatGPT Codex stream rejected (usage limit reached)".to_owned(),
        });
    }
    Some(SamplingError::StreamError {
        error_type: "provider_error".to_owned(),
        message: "ChatGPT Codex stream rejected".to_owned(),
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
fn apply_terminal_event_overrides(event: &mut rs::ResponseStreamEvent, data: &str) {
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

/// Sticky-routing state returned by the ChatGPT Codex backend. The value is
/// opaque and credential-adjacent: it is kept only in memory, marked
/// sensitive, and scoped to the request ID shared by one Grok user/tool turn.
struct CodexTurnState {
    request_id: String,
    value: HeaderValue,
}

/// Actor-scoped owner for the opaque ChatGPT sticky-routing value.
///
/// A [`SamplingClient`] is rebuilt for every sampler submission (and may also
/// be rebuilt during transport recovery), while one user turn spans several
/// submissions when tools are called. Keeping the state in this shared owner
/// preserves the provider's per-turn contract without persisting or exposing
/// the value.
#[derive(Clone, Default)]
pub(crate) struct CodexTurnStateStore {
    inner: std::sync::Arc<std::sync::Mutex<Option<CodexTurnState>>>,
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
    /// Per-turn ChatGPT sticky-routing state shared by cheap client clones.
    /// An empty request ID (for example an auxiliary title call) neither reads
    /// nor clears this state, so concurrent auxiliary work cannot disturb the
    /// active agent turn.
    codex_turn_state: CodexTurnStateStore,
}

impl std::fmt::Debug for SamplingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SamplingClient")
            .field("base_url", &self.base_url)
            .field("defaults", &self.defaults)
            .field(
                "has_attribution_callback",
                &self.attribution_callback.is_some(),
            )
            .field("has_bearer_resolver", &self.bearer_resolver.is_some())
            .field("has_request_auth", &self.request_auth.is_some())
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
struct ClientDefaults {
    provider: ProviderId,
    model: String,
    max_completion_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    api_backend: ApiBackend,
    auth_scheme: AuthScheme,
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
        config: SamplerConfig,
        codex_turn_state: CodexTurnStateStore,
    ) -> Result<Self> {
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
            if !valid_openai_codex_base_url(&config.base_url) {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex credentials may only be sent to the ChatGPT Codex endpoint",
                ));
            }
            if config.api_backend != ApiBackend::Responses {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex requires the Responses API backend",
                ));
            }
            if config.api_key.is_some() || config.bearer_resolver.is_some() {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex credentials must use dynamic request authentication",
                ));
            }
            if config.request_auth.is_none() {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex dynamic request authentication is required",
                ));
            }
        }

        let mut headers = HeaderMap::new();
        let mut responses_lite = false;
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref api_key) = config.api_key {
            match config.auth_scheme {
                AuthScheme::XApiKey => {
                    let header_value = HeaderValue::from_str(api_key).map_err(|_| {
                        tracing::debug!(
                            "Invalid api_key: cannot be converted to a valid HTTP header"
                        );
                        SamplingError::Auth(
                            "Invalid api_key: cannot be converted to a valid HTTP header"
                                .to_string(),
                        )
                    })?;
                    headers.insert(HeaderName::from_static("x-api-key"), header_value);
                }
                AuthScheme::Bearer => {
                    let bearer = format!("Bearer {}", api_key);
                    let header_value = HeaderValue::from_str(&bearer).map_err(|_| {
                        tracing::debug!(
                            "Invalid api_key: cannot be converted to a valid HTTP Authorization header"
                        );
                        SamplingError::Auth(
                            "Invalid api_key: cannot be converted to a valid HTTP Authorization header"
                                .to_string(),
                        )
                    })?;
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
                && (is_codex_credential_header_or_alias(&header_name)
                    || is_xai_specific_header(&header_name)
                    || is_codex_provider_identity_header(&header_name))
            {
                return Err(SamplingError::InvalidConfiguration(
                    "OpenAI Codex protected/provider headers cannot be set via extra_headers",
                ));
            }
            let header_value = HeaderValue::from_str(value)
                .map_err(|_| SamplingError::InvalidConfiguration("Invalid extra header value"))?;
            headers.insert(header_name, header_value);
        }

        if !config.provider.is_openai_codex() {
            // Add x-grok-client-version header for version gating at the proxy.
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
        } else {
            headers.insert(
                HeaderName::from_static("originator"),
                HeaderValue::from_static(OPENAI_CODEX_ORIGINATOR),
            );
            headers.insert(
                HeaderName::from_static("version"),
                HeaderValue::from_static(OPENAI_CODEX_COMPATIBILITY_VERSION),
            );
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

        let http = if config.provider.is_openai_codex() {
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(SamplingError::Http)?
        } else if config.force_http1 {
            tracing::info!("Using HTTP/1.1 for sampling client (force_http1=true)");
            crate::shared_http::client_http1().map_err(SamplingError::Http)?
        } else {
            crate::shared_http::client().map_err(SamplingError::Http)?
        };

        tracing::info!(
            target: crate::sampling_log::TARGET,
            event = "client_new",
            base_url = %config.base_url,
            model = %config.model,
            provider = ?config.provider,
            api_backend = ?config.api_backend,
            auth_scheme = ?config.auth_scheme,
            // "unset" (not "none"): `ReasoningEffort::None` is a real wire value;
            // logging the absent Option as "none" looked like we were sending it.
            reasoning_effort = config.reasoning_effort.map_or("unset", |e| e.as_str()),
            has_api_key = config.api_key.is_some(),
            has_bearer_resolver = config.bearer_resolver.is_some(),
            has_request_auth = config.request_auth.is_some(),
            has_authorization_header = headers.get(AUTHORIZATION).is_some(),
            has_x_api_key_header = headers.get(HeaderName::from_static("x-api-key")).is_some(),
        );

        let defaults = ClientDefaults {
            provider: config.provider,
            model: config.model,
            max_completion_tokens: config.max_completion_tokens,
            temperature: config.temperature,
            top_p: config.top_p,
            api_backend: config.api_backend,
            auth_scheme: config.auth_scheme,
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
            codex_turn_state,
        })
    }

    /// Rebuild this HTTP client without dropping the actor-owned Codex turn
    /// state. Used only for transport fallback within the same submission.
    pub(crate) fn rebuild_with_config(&self, config: SamplerConfig) -> Result<Self> {
        Self::new_with_codex_turn_state(config, self.codex_turn_state.clone())
    }

    /// The configured API backend for this client.
    pub fn api_backend(&self) -> ApiBackend {
        self.defaults.api_backend.clone()
    }

    /// The explicit provider that owns requests made by this client.
    pub fn provider(&self) -> ProviderId {
        self.defaults.provider
    }

    /// POST with default headers. Legacy bearer resolution and provider-owned
    /// dynamic auth are applied to a fresh map on every call.
    fn post(&self, url: impl reqwest::IntoUrl) -> Result<PreparedRequest> {
        let mut headers = self.default_headers.clone();
        if let Some(resolver) = &self.bearer_resolver
            && let Some(fresh) = resolver.current_bearer()
        {
            match self.defaults.auth_scheme {
                AuthScheme::XApiKey => {
                    headers.remove(AUTHORIZATION);
                    if let Ok(v) = HeaderValue::from_str(&fresh) {
                        headers.insert(HeaderName::from_static("x-api-key"), v);
                    }
                }
                AuthScheme::Bearer => {
                    headers.remove(HeaderName::from_static("x-api-key"));
                    if let Ok(v) = HeaderValue::from_str(&format!("Bearer {fresh}")) {
                        headers.insert(AUTHORIZATION, v);
                    }
                }
            }
        }
        if let Some(injector) = &self.header_injector {
            injector.inject(&mut headers);
        }

        if self.defaults.provider.is_openai_codex() {
            // Ordinary injectors may add tracing/correlation headers, but they
            // never own provider credentials. Strip protected and xAI-specific
            // names before the provider hook gets exclusive ownership.
            let forbidden: Vec<HeaderName> = headers
                .keys()
                .filter(|name| {
                    is_codex_credential_header_or_alias(name)
                        || is_xai_specific_header(name)
                        || is_codex_provider_identity_header(name)
                })
                .cloned()
                .collect();
            for name in forbidden {
                headers.remove(name);
            }
            // Provider identity is fixed to this client implementation even
            // when a generic tracing injector attempted to replace it.
            headers.insert(
                HeaderName::from_static("originator"),
                HeaderValue::from_static(OPENAI_CODEX_ORIGINATOR),
            );
            headers.insert(
                HeaderName::from_static("version"),
                HeaderValue::from_static(OPENAI_CODEX_COMPATIBILITY_VERSION),
            );
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

        if self.defaults.provider.is_openai_codex() {
            // Request-auth implementations own credentials, not transport
            // identity or request correlation. Clear any attempted poisoning
            // after the hook, then seal the provider-owned values again.
            for name in [
                OPENAI_CODEX_RESPONSES_LITE_HEADER,
                CODEX_SESSION_ID_HEADER,
                CODEX_THREAD_ID_HEADER,
                CODEX_CLIENT_REQUEST_ID_HEADER,
                CODEX_TURN_STATE_HEADER,
                "originator",
                "version",
            ] {
                headers.remove(name);
            }
            headers.insert(
                HeaderName::from_static("originator"),
                HeaderValue::from_static(OPENAI_CODEX_ORIGINATOR),
            );
            headers.insert(
                HeaderName::from_static("version"),
                HeaderValue::from_static(OPENAI_CODEX_COMPATIBILITY_VERSION),
            );
            validate_codex_request_auth_headers(&mut headers)?;
            let binding =
                credential_binding
                    .as_ref()
                    .ok_or(SamplingError::InvalidConfiguration(
                        "OpenAI Codex request authentication omitted its credential generation",
                    ))?;
            validate_codex_credential_binding(binding)?;
        }

        tracing::info!(
            target: crate::sampling_log::TARGET,
            event = "client_post",
            base_url = %self.base_url,
            model = %self.defaults.model,
            provider = ?self.defaults.provider,
            api_backend = ?self.defaults.api_backend,
            auth_scheme = ?self.defaults.auth_scheme,
            has_bearer_resolver = self.bearer_resolver.is_some(),
            has_request_auth = self.request_auth.is_some(),
            has_authorization_header = headers.get(AUTHORIZATION).is_some(),
            has_x_api_key_header = headers.get(HeaderName::from_static("x-api-key")).is_some(),
            has_account_header = headers.get(CHATGPT_ACCOUNT_ID_HEADER).is_some(),
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
            let builder = grok_headers.apply_codex(builder, self.defaults.responses_lite)?;
            Ok(self.apply_codex_turn_state(builder, grok_headers.req_id))
        } else {
            Ok(grok_headers.apply(builder))
        }
    }

    fn apply_codex_turn_state(
        &self,
        builder: reqwest::RequestBuilder,
        request_id: &str,
    ) -> reqwest::RequestBuilder {
        if request_id.is_empty() {
            return builder;
        }
        let mut state = self
            .codex_turn_state
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.as_ref() {
            Some(turn_state) if turn_state.request_id == request_id => {
                builder.header(CODEX_TURN_STATE_HEADER, turn_state.value.clone())
            }
            Some(_) => {
                // A new Grok prompt gets a fresh Codex turn. Never replay the
                // sticky-routing value across that boundary.
                *state = None;
                builder
            }
            None => builder,
        }
    }

    fn capture_codex_turn_state(&self, headers: &HeaderMap, request_id: &str) {
        if !self.defaults.provider.is_openai_codex() || request_id.is_empty() {
            return;
        }
        let Some(value) = headers.get(CODEX_TURN_STATE_HEADER) else {
            return;
        };
        let mut value = value.clone();
        value.set_sensitive(true);
        let mut state = self
            .codex_turn_state
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.as_ref() {
            // Match the official once-per-turn behavior: the first value is
            // authoritative and remains unchanged throughout tool rounds and
            // retries in this request ID.
            Some(turn_state) if turn_state.request_id == request_id => {}
            _ => {
                *state = Some(CodexTurnState {
                    request_id: request_id.to_string(),
                    value,
                });
            }
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
        match &self.request_auth {
            Some(request_auth) => request_auth.recover_unauthorized(rejected).await,
            None => false,
        }
    }

    pub fn auth_info(&self) -> crate::sampling_log::AuthInfo {
        let has_auth = self.has_auth();
        let auth_type = match (self.defaults.provider, self.defaults.auth_scheme, has_auth) {
            (ProviderId::OpenAiCodex, _, true) => "openai-codex-subscription",
            (_, AuthScheme::XApiKey, true) => "x-api-key",
            (_, AuthScheme::Bearer, true) => "bearer",
            (_, _, false) => "none",
        };
        crate::sampling_log::AuthInfo { auth_type }
    }

    /// Check if a header name contains sensitive information that should be redacted.
    fn is_sensitive_header(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.contains("authorization")
            || lower.contains("api-key")
            || lower.contains("apikey")
            || lower.contains("token")
            || lower.contains("secret")
            || lower.contains("account")
            || lower.contains("user-id")
            || lower.contains("userid")
            || lower.contains("email")
            || lower.contains("organization")
            || lower.contains("org-id")
            || lower.contains("team-id")
            || lower.contains("deployment-id")
            || lower.contains("project-id")
            || lower == "openai-project"
            || lower == "x-openai-fedramp"
            || lower.contains("cookie")
            || lower.contains("session")
            || lower.contains("thread-id")
            || lower.contains("client-request-id")
            || lower == CODEX_TURN_STATE_HEADER
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

    /// Build request headers string for error messages (redacting sensitive values).
    fn format_request_headers(
        &self,
        x_grok_conv_id: &str,
        x_grok_req_id: &str,
        model_id: &str,
        include_accept: bool,
    ) -> Vec<String> {
        let mut req_headers: Vec<String> = self
            .default_headers
            .iter()
            .map(|(name, value)| {
                Self::format_header(name.as_str(), value.to_str().unwrap_or("[non-utf8]"))
            })
            .collect();

        if !self.defaults.provider.is_openai_codex() {
            req_headers.push(Self::format_header("x-grok-conv-id", x_grok_conv_id));
            req_headers.push(Self::format_header("x-grok-req-id", x_grok_req_id));
            req_headers.push(Self::format_header("x-grok-model-override", model_id));
        }
        if include_accept {
            req_headers.push(Self::format_header("accept", "text/event-stream"));
        }
        req_headers
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
            return headers
                .keys()
                .filter(|name| CODEX_DIAGNOSTIC_RESPONSE_HEADERS.contains(&name.as_str()))
                .map(|name| format!("  {}: [value omitted]", name.as_str()))
                .collect();
        }

        headers
            .iter()
            .map(|(name, value)| Self::format_header(name.as_str(), &format!("{:?}", value)))
            .collect()
    }

    /// Log all headers from a request at debug level (redacting sensitive values).
    fn log_request_headers(request: &reqwest::Request, endpoint_name: &str) {
        for (name, value) in request.headers().iter() {
            let value_str = if Self::is_sensitive_header(name.as_str()) {
                "[REDACTED]"
            } else {
                value.to_str().unwrap_or("[non-utf8]")
            };
            tracing::debug!(
                header_name = %name,
                header_value = %value_str,
                "Request header ({})",
                endpoint_name
            );
        }
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
    fn provider_error_message(&self, body: &[u8]) -> String {
        if self.defaults.provider.is_openai_codex() {
            if serde_json::from_slice::<serde_json::Value>(body)
                .ok()
                .as_ref()
                .is_some_and(is_codex_usage_limit_error)
            {
                "ChatGPT Codex request rejected (usage limit reached)".to_owned()
            } else if body
                .windows(b"encrypted_content".len())
                .any(|window| window == b"encrypted_content")
            {
                "ChatGPT Codex request rejected (encrypted_content)".to_owned()
            } else {
                "ChatGPT Codex request rejected".to_owned()
            }
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
        let bytes = response.bytes().await?;

        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.record_401_attribution(crate::attribution::SamplingConsumer::ChatCompletions);
                let server_message = parse_error_bytes(bytes.as_ref());
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401): {server_message}"
                )));
            }
            let message = parse_error_bytes(bytes.as_ref());
            return Err(SamplingError::Api {
                status,
                message,
                model_metadata,
                retry_after_secs,
                should_retry,
            });
        }

        let completion = serde_json::from_slice::<ChatCompletionResponse>(&bytes).map_err(|e| {
            let raw_body = String::from_utf8_lossy(&bytes);
            tracing::error!(
                error = %e,
                raw_body = %raw_body,
                "Failed to deserialize ChatCompletionResponse"
            );
            SamplingError::Serialization(e)
        })?;
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
        let prepared = self.post(self.endpoint("chat/completions"))?;
        let request_credential = prepared.credential_binding;
        let http_request = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .json(&payload);

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
        let prepared = self.post(self.endpoint("chat/completions"))?;
        let request_credential = prepared.credential_binding;
        let http_request = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .header(ACCEPT, HeaderValue::from_static("text/event-stream"))
            .json(&streaming_request);

        let built_request = http_request.build().map_err(|e| {
            tracing::error!("Failed to build HTTP request: {}", e);
            SamplingError::Http(e)
        })?;

        tracing::debug!(
            url = %built_request.url(),
            method = %built_request.method(),
            "Sending chat/completions request"
        );
        Self::log_request_headers(&built_request, "chat/completions");

        let response = self.http.execute(built_request).await.map_err(|e| {
            tracing::debug!("HTTP request failed: {}", e);
            record_stream_request_failure(&e);
            e
        })?;

        let status = response.status();
        let span = tracing::Span::current();
        span.record("status_code", status.as_u16() as i64);
        span.record("success", status.is_success());
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
                let endpoint = self.endpoint("chat/completions");
                let server_message = response.text().await.unwrap_or_default();
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401) from {endpoint}: {server_message}"
                )));
            }

            let req_headers =
                self.format_request_headers(x_grok_conv_id, x_grok_req_id, &model_id, true);
            let resp_headers =
                Self::format_response_headers(response.headers(), self.defaults.provider);
            let bytes = response.bytes().await?;
            let server_message = parse_error_bytes(bytes.as_ref());

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
        let chunks = event_stream
            .scan(false, |had_transport_error, event_res| {
                if *had_transport_error {
                    return std::future::ready(None);
                }
                let item = match event_res {
                    Ok(event) => {
                        let data = &event.data;
                        if data == "[DONE]" {
                            return std::future::ready(None);
                        }

                        tracing::info!(
                            target: crate::sampling_log::TARGET,
                            event = "sse_chunk",
                            backend = "chat_completions",
                            data = %data,
                        );

                        if let Some(stream_error) = try_parse_stream_error(data) {
                            Some(Err(stream_error))
                        } else {
                            Some(
                                serde_json::from_str::<ChatCompletionChunk>(data).map_err(|e| {
                                    tracing::error!(
                                        error = %e,
                                        raw_data = %data,
                                        "Failed to deserialize ChatCompletionChunk from stream"
                                    );
                                    SamplingError::Serialization(e)
                                }),
                            )
                        }
                    }
                    Err(e) => {
                        *had_transport_error = true;
                        Some(Err(SamplingError::EventStreamError(e.to_string())))
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
            request.inner.service_tier = match self.defaults.service_tier.as_deref() {
                Some("priority") => Some(rs::ServiceTier::Priority),
                Some("flex") => Some(rs::ServiceTier::Flex),
                Some("scale") => Some(rs::ServiceTier::Scale),
                Some("auto") => Some(rs::ServiceTier::Auto),
                Some(xai_grok_sampling_types::OPENAI_CODEX_STANDARD_SERVICE_TIER) | None => None,
                Some(tier) => {
                    tracing::warn!(
                        provider = "openai_codex",
                        service_tier = tier,
                        "catalog service tier is not supported by this client; omitting it"
                    );
                    None
                }
            };
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
        apply_extended_codex_reasoning_effort(
            self.defaults.provider,
            request.wire_reasoning_effort,
            &mut request_body,
        )?;
        apply_codex_responses_lite_contract(
            self.defaults.provider,
            self.defaults.responses_lite,
            codex_prompt_cache_key(
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
        let should_retry = codex_usage_limit_should_retry(
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
                            "OpenAI Codex request authentication omitted its credential generation",
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

            let req_headers =
                self.format_request_headers(x_grok_conv_id, x_grok_req_id, &model_id, false);
            let server_message = self.provider_error_message(bytes.as_ref());

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
    pub async fn create_response_stream(
        &self,
        mut request: CreateResponseWrapper,
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
        apply_extended_codex_reasoning_effort(
            self.defaults.provider,
            request.wire_reasoning_effort,
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
        apply_codex_responses_lite_contract(
            self.defaults.provider,
            self.defaults.responses_lite,
            codex_prompt_cache_key(
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
        let prepared = self.post(self.endpoint("responses"))?;
        let request_credential = prepared.credential_binding;
        let mut http_request = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .header(ACCEPT, HeaderValue::from_static("text/event-stream"));
        if doom_loop.is_some() {
            // Presence opts in; the server ignores the value.
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
        Self::log_request_headers(&built_request, "responses");

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
                            "OpenAI Codex request authentication omitted its credential generation",
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
            let req_headers =
                self.format_request_headers(x_grok_conv_id, x_grok_req_id, &model_id, true);
            let resp_headers =
                Self::format_response_headers(response.headers(), self.defaults.provider);
            let bytes = response.bytes().await?;
            let should_retry = codex_usage_limit_should_retry(
                self.defaults.provider,
                status,
                bytes.as_ref(),
                should_retry,
            );
            let server_message = self.provider_error_message(bytes.as_ref());

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
                                data = %data,
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
                            try_parse_codex_stream_error(data)
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
                            Some(Some(deserialize_response_event_for_provider(data, false)))
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
        let prepared = self.post(self.endpoint("messages"))?;
        let request_credential = prepared.credential_binding;
        let http_request = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .json(&request.inner);

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
                let endpoint = self.endpoint("messages");
                let server_message = parse_error_bytes(bytes.as_ref());
                return Err(SamplingError::Auth(format!(
                    "Unauthorized (401) from {endpoint}: {server_message}"
                )));
            }

            let req_headers =
                self.format_request_headers(x_grok_conv_id, x_grok_req_id, &model_id, false);
            let server_message = parse_error_bytes(bytes.as_ref());

            let message = self.build_api_error_message(
                status,
                &server_message,
                &self.endpoint("messages"),
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
                let raw_body = String::from_utf8_lossy(&bytes);
                tracing::error!(
                    error = %e,
                    raw_body = %raw_body,
                    "Failed to deserialize MessagesResponse"
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
        let prepared = self.post(self.endpoint("messages"))?;
        let request_credential = prepared.credential_binding;
        let http_request = self
            .apply_provider_request_headers(prepared.builder, &grok_headers)?
            .header(ACCEPT, HeaderValue::from_static("text/event-stream"))
            .json(&request.inner);

        let built_request = http_request.build().map_err(|e| {
            tracing::error!("Failed to build HTTP request: {}", e);
            SamplingError::Http(e)
        })?;

        tracing::debug!(
            url = %built_request.url(),
            method = %built_request.method(),
            "Sending messages API stream request"
        );
        Self::log_request_headers(&built_request, "messages");

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
                let endpoint = self.endpoint("messages");
                let server_message = response.text().await.unwrap_or_default();
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
            let req_headers =
                self.format_request_headers(x_grok_conv_id, x_grok_req_id, &model_id, true);
            let resp_headers =
                Self::format_response_headers(response.headers(), self.defaults.provider);
            let bytes = response.bytes().await?;
            let server_message = parse_error_bytes(bytes.as_ref());

            let message = self.build_api_error_message(
                status,
                &server_message,
                &self.endpoint("messages"),
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
        let events = event_stream
            .scan(false, |had_transport_error, event_res| {
                if *had_transport_error {
                    return std::future::ready(None);
                }
                let item = match event_res {
                    Ok(event) => {
                        let data = &event.data;
                        if data == "[DONE]" {
                            return std::future::ready(None);
                        }

                        tracing::info!(
                            target: crate::sampling_log::TARGET,
                            event = "sse_chunk",
                            backend = "messages",
                            data = %data,
                        );

                        if let Some(stream_error) = try_parse_stream_error(data) {
                            Some(Err(stream_error))
                        } else {
                            Some(
                                serde_json::from_str::<messages::MessageStreamEvent>(data).map_err(
                                    |e| {
                                        tracing::error!(
                                            error = %e,
                                            raw_data = %data,
                                            "Failed to deserialize MessageStreamEvent from stream"
                                        );
                                        SamplingError::Serialization(e)
                                    },
                                ),
                            )
                        }
                    }
                    Err(e) => {
                        *had_transport_error = true;
                        Some(Err(SamplingError::EventStreamError(e.to_string())))
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
        mut request: ConversationRequest,
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

        let responses_request: rs::CreateResponse = (&request).into();

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

        self.create_response_stream(wrapper).await
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

        let responses_request: rs::CreateResponse = (&request).into();

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
                let events = crate::stream::stream_messages(raw, meta, request_id, idle_timeout);
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
                should_retry: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use xai_grok_sampling_types::types::ChatRequestMessage;

    fn minimal_config() -> SamplerConfig {
        SamplerConfig {
            provider: Default::default(),
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
            let rendered = SamplingClient::format_header(name, "sensitive-value");
            assert!(rendered.contains("[REDACTED]"));
            assert!(!rendered.contains("sensitive-value"));
        }
        assert_eq!(
            SamplingClient::format_header("traceparent", "safe-value"),
            "  traceparent: safe-value"
        );
    }

    #[test]
    fn codex_response_header_diagnostics_never_render_provider_values() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "text/plain; reflected=codex-access-token".parse().unwrap(),
        );
        headers.insert(reqwest::header::RETRY_AFTER, "30".parse().unwrap());
        headers.insert(
            "x-error-detail",
            "Bearer codex-access-token".parse().unwrap(),
        );
        headers.insert(
            "chatgpt-account-id",
            "account-selected-by-auth-store".parse().unwrap(),
        );

        let rendered =
            SamplingClient::format_response_headers(&headers, ProviderId::OpenAiCodex).join("\n");

        assert!(rendered.contains("content-type: [value omitted]"));
        assert!(rendered.contains("retry-after: [value omitted]"));
        assert!(!rendered.contains("x-error-detail"));
        assert!(!rendered.contains("chatgpt-account-id"));
        assert!(!rendered.contains("codex-access-token"));
        assert!(!rendered.contains("account-selected-by-auth-store"));
    }

    #[test]
    fn codex_malformed_sse_diagnostics_do_not_reflect_payload() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let data = format!(r#"{{"type":"unknown.provider.event","value":"{reflected}"}}"#);
        let error = deserialize_response_event_for_provider(&data, true).unwrap_err();
        let rendered = error.to_string();
        assert_eq!(
            rendered,
            "serialization error: ChatGPT Codex stream event was invalid"
        );
        assert!(!rendered.contains(reflected));
    }

    #[test]
    fn codex_sparse_official_wire_fixtures_decode_statefully() {
        fn decode(
            decoder: &mut CodexSseDecoder,
            value: serde_json::Value,
        ) -> rs::ResponseStreamEvent {
            decoder
                .decode(&value.to_string())
                .expect("fixture should decode")
                .expect("fixture should produce an event")
        }

        let mut decoder = CodexSseDecoder::new("gpt-5.6");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.created",
                "response": {"id": "resp_1"}
            }),
        );
        let rs::ResponseStreamEvent::ResponseCreated(created) = event else {
            panic!("expected response.created");
        };
        assert_eq!(created.sequence_number, 0);
        assert_eq!(created.response.id, "resp_1");
        assert_eq!(created.response.model, "gpt-5.6");
        assert_eq!(created.response.status, rs::Status::InProgress);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.added",
                "item": {"type": "reasoning", "id": "reason_1", "summary": []}
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemAdded(reasoning) = event else {
            panic!("expected reasoning output_item.added");
        };
        assert_eq!(reasoning.sequence_number, 1);
        assert_eq!(reasoning.output_index, 0);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.reasoning_summary_text.delta",
                "delta": "summary ",
                "summary_index": 0
            }),
        );
        let rs::ResponseStreamEvent::ResponseReasoningSummaryTextDelta(summary) = event else {
            panic!("expected reasoning summary delta");
        };
        assert_eq!(summary.output_index, 0);
        assert_eq!(summary.item_id, "reason_1");
        assert_eq!(summary.delta, "summary ");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.reasoning_summary_text.done",
                "text": "summary complete",
                "summary_index": 0
            }),
        );
        let rs::ResponseStreamEvent::ResponseReasoningSummaryTextDone(summary) = event else {
            panic!("expected reasoning summary done");
        };
        assert_eq!(summary.output_index, 0);
        assert_eq!(summary.item_id, "reason_1");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.reasoning_text.delta",
                "delta": "private reasoning",
                "content_index": 0
            }),
        );
        let rs::ResponseStreamEvent::ResponseReasoningTextDelta(reasoning) = event else {
            panic!("expected reasoning text delta");
        };
        assert_eq!(reasoning.output_index, 0);
        assert_eq!(reasoning.item_id, "reason_1");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "shell",
                    "arguments": "{}"
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemDone(first_call) = event else {
            panic!("expected first function output_item.done");
        };
        assert_eq!(first_call.output_index, 1);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "function_call",
                    "call_id": "call_2",
                    "name": "read_file",
                    "arguments": "{\"path\":\"README.md\"}"
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemDone(second_call) = event else {
            panic!("expected second function output_item.done");
        };
        assert_eq!(second_call.output_index, 2);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.function_call_arguments.done",
                "item_id": "call_2",
                "arguments": "{\"path\":\"README.md\"}"
            }),
        );
        let rs::ResponseStreamEvent::ResponseFunctionCallArgumentsDone(arguments) = event else {
            panic!("expected function arguments done");
        };
        assert_eq!(arguments.output_index, 2);
        assert_eq!(arguments.item_id, "call_2");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.added",
                "item": {
                    "type": "message",
                    "role": "assistant",
                    "id": "msg_1",
                    "content": []
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemAdded(message_added) = event else {
            panic!("expected message output_item.added");
        };
        assert_eq!(message_added.output_index, 3);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_text.delta",
                "delta": "hello"
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputTextDelta(text) = event else {
            panic!("expected output text delta");
        };
        assert_eq!(text.output_index, 3);
        assert_eq!(text.item_id, "msg_1");
        assert_eq!(text.content_index, 0);
        assert_eq!(text.logprobs, None);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "message",
                    "role": "assistant",
                    "id": "msg_1",
                    "content": [{"type": "output_text", "text": "hello"}]
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemDone(message_done) = event else {
            panic!("expected message output_item.done");
        };
        assert_eq!(message_done.output_index, 3);
        let rs::OutputItem::Message(message) = message_done.item else {
            panic!("expected normalized message");
        };
        assert_eq!(message.status, rs::OutputStatus::Completed);
        let rs::OutputMessageContent::OutputText(text) = &message.content[0] else {
            panic!("expected output text content");
        };
        assert!(text.annotations.is_empty());
        assert_eq!(text.logprobs, None);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.completed",
                "response": {
                    "id": "resp_1",
                    "end_turn": false,
                    "metadata": {"provider_secret": "must-not-survive"},
                    "usage": {
                        "input_tokens": 7,
                        "input_tokens_details": null,
                        "output_tokens": 3,
                        "output_tokens_details": null,
                        "total_tokens": 10
                    }
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseCompleted(completed) = event else {
            panic!("expected sparse response.completed");
        };
        assert_eq!(completed.response.model, "gpt-5.6");
        assert_eq!(completed.response.status, rs::Status::Completed);
        assert!(completed.response.output.is_empty());
        let usage = completed.response.usage.expect("usage");
        assert_eq!(usage.input_tokens_details.cached_tokens, 0);
        assert_eq!(usage.output_tokens_details.reasoning_tokens, 0);
        let metadata = completed.response.metadata.expect("end_turn metadata");
        assert_eq!(
            metadata
                .get(CODEX_END_TURN_METADATA_KEY)
                .map(String::as_str),
            Some("false")
        );
        assert!(!metadata.contains_key("provider_secret"));
    }

    #[test]
    fn codex_decoder_ignores_unknown_nonterminal_metadata() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let mut decoder = CodexSseDecoder::new("gpt-5.6");
        let data =
            format!(r#"{{"type":"response.metadata","metadata":{{"reflected":"{reflected}"}}}}"#);
        assert!(
            decoder
                .decode(&data)
                .expect("metadata is ignored")
                .is_none()
        );

        let data = format!(r#"{{"type":"response.future_event","value":"{reflected}"}}"#);
        assert!(decoder.decode(&data).expect("unknown is ignored").is_none());
    }

    #[test]
    fn codex_decoder_preserves_both_end_turn_values_without_provider_metadata() {
        for (end_turn, expected) in [(true, "true"), (false, "false")] {
            let mut decoder = CodexSseDecoder::new("gpt-5.6");
            let event = decoder
                .decode(
                    &serde_json::json!({
                        "type": "response.completed",
                        "response": {
                            "id": "resp_1",
                            "end_turn": end_turn,
                            "metadata": {"provider_value": "must-not-survive"}
                        }
                    })
                    .to_string(),
                )
                .expect("terminal fixture should decode")
                .expect("terminal fixture should produce an event");
            let rs::ResponseStreamEvent::ResponseCompleted(completed) = event else {
                panic!("expected response.completed");
            };
            let metadata = completed.response.metadata.expect("private metadata");
            assert_eq!(
                metadata
                    .get(CODEX_END_TURN_METADATA_KEY)
                    .map(String::as_str),
                Some(expected)
            );
            assert!(!metadata.contains_key("provider_value"));
        }
    }

    #[test]
    fn codex_decoder_failures_never_reflect_provider_payload() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let mut decoder = CodexSseDecoder::new("gpt-5.6");

        for data in [
            format!(r#"{{"type":"response.output_text.delta","delta":{{"value":"{reflected}"}}}}"#),
            format!(
                r#"{{"type":"response.failed","response":{{"error":{{"message":"{reflected}"}}}}}}"#
            ),
            format!(r#"{{"type":"error","message":"{reflected}"}}"#),
            format!(r#"{{"error":{{"message":"{reflected}"}}}}"#),
        ] {
            let rendered = decoder.decode(&data).unwrap_err().to_string();
            assert!(!rendered.contains(reflected));
            assert!(!rendered.contains("access-token"));
            assert!(!rendered.contains("account-selected"));
        }
    }

    #[test]
    fn codex_malformed_response_diagnostics_do_not_reflect_payload() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let body = format!(
            r#"{{
                "id": "resp_1",
                "object": "response",
                "created_at": 0,
                "model": "gpt-5.6",
                "status": "{reflected}",
                "output": []
            }}"#
        );
        let error = deserialize_response_for_provider(body.as_bytes(), ProviderId::OpenAiCodex)
            .unwrap_err();
        let rendered = error.to_string();
        assert_eq!(
            rendered,
            "serialization error: ChatGPT Codex response was invalid"
        );
        assert!(!rendered.contains(reflected));
    }

    #[test]
    fn codex_stream_error_diagnostics_do_not_reflect_payload() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let data = format!(r#"{{"error":{{"message":"{reflected}"}}}}"#);
        let error = try_parse_codex_stream_error(&data).expect("error envelope");
        let rendered = error.to_string();
        assert!(rendered.contains("ChatGPT Codex stream rejected"));
        assert!(!rendered.contains(reflected));
    }

    #[test]
    fn codex_usage_limit_errors_keep_only_the_safe_structured_classification() {
        let reflected = "Bearer codex-access-token account-selected-by-auth-store";
        let data = format!(
            r#"{{"type":"error","status":429,"error":{{"type":"usage_limit_reached","message":"{reflected}","resets_at":1704067242}}}}"#
        );
        let error = try_parse_codex_stream_error(&data).expect("usage-limit envelope");
        let rendered = format!("{error:?} {error}");
        assert!(rendered.contains("usage_limit_reached"));
        assert!(rendered.contains("usage limit reached"));
        assert!(
            !error.is_retryable(),
            "a structural Codex SSE usage limit must stop after one request"
        );
        assert!(!rendered.contains(reflected));
        assert!(!rendered.contains("1704067242"));

        assert_eq!(
            codex_usage_limit_should_retry(
                ProviderId::OpenAiCodex,
                StatusCode::TOO_MANY_REQUESTS,
                data.as_bytes(),
                None,
            ),
            Some(false),
            "a structural Codex HTTP usage limit must stop after one request"
        );
        assert_eq!(
            codex_usage_limit_should_retry(
                ProviderId::Xai,
                StatusCode::TOO_MANY_REQUESTS,
                data.as_bytes(),
                None,
            ),
            None,
            "the Codex structural rule must not change xAI retry behavior"
        );

        let (config, _) = codex_config();
        let client = SamplingClient::new(config).unwrap();
        let message = client.provider_error_message(data.as_bytes());
        assert_eq!(
            message,
            "ChatGPT Codex request rejected (usage limit reached)"
        );
        assert!(!message.contains(reflected));
    }

    #[test]
    fn codex_extended_reasoning_efforts_match_current_codex_wire_contract() {
        for (effort, expected) in [
            (ReasoningEffort::Max, "max"),
            (ReasoningEffort::Ultra, "max"),
        ] {
            let mut body = serde_json::json!({
                "model": "gpt-5.6",
                "reasoning": {"summary": "auto"}
            });
            apply_extended_codex_reasoning_effort(ProviderId::OpenAiCodex, Some(effort), &mut body)
                .unwrap();
            assert_eq!(
                body.pointer("/reasoning/effort").and_then(|v| v.as_str()),
                Some(expected)
            );
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
        apply_extended_codex_reasoning_effort(
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
    fn extended_reasoning_efforts_are_not_injected_into_xai_responses() {
        let mut body = serde_json::json!({"model": "grok-4"});
        let error = apply_extended_codex_reasoning_effort(
            ProviderId::Xai,
            Some(ReasoningEffort::Ultra),
            &mut body,
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires the OpenAI Codex provider")
        );
        assert!(body.pointer("/reasoning/effort").is_none());
    }

    #[test]
    fn codex_responses_lite_rewrites_only_the_transport_contract() {
        let mut body = serde_json::json!({
            "model": "gpt-5.6",
            "instructions": "follow the repository instructions",
            "tools": [{"type": "function", "name": "read_file"}],
            "input": [
                {
                    "type": "message",
                    "role": "system",
                    "content": [{"type": "input_text", "text": "system context"}],
                },
                {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_image",
                        "image_url": "data:image/png;base64,AA==",
                        "detail": "high",
                    }],
                },
            ],
            "parallel_tool_calls": true,
            "temperature": 1.0,
            "reasoning": {"effort": "ultra"},
        });

        apply_codex_responses_lite_contract(
            ProviderId::OpenAiCodex,
            true,
            Some("conversation-cache-key"),
            &mut body,
        )
        .unwrap();

        assert!(body.get("tools").is_none());
        assert!(body.get("instructions").is_none());
        assert!(body.get("temperature").is_none());
        assert_eq!(body["prompt_cache_key"], "conversation-cache-key");
        assert_eq!(body["parallel_tool_calls"], false);
        assert_eq!(body["tool_choice"], "auto");
        assert_eq!(body["reasoning"]["effort"], "ultra");
        assert_eq!(body["reasoning"]["context"], "all_turns");
        assert_eq!(
            body["input"][0],
            serde_json::json!({
                "type": "additional_tools",
                "role": "developer",
                "tools": [{"type": "function", "name": "read_file", "strict": false}],
            })
        );
        assert_eq!(body["input"][1]["type"], "message");
        assert_eq!(body["input"][1]["role"], "developer");
        assert_eq!(
            body["input"][1]["content"][0]["text"],
            "follow the repository instructions"
        );
        assert_eq!(body["input"][2]["role"], "developer");
        assert_eq!(body["input"][3]["role"], "user");
        assert!(body["input"][3]["content"][0].get("detail").is_none());
    }

    #[test]
    fn codex_prompt_cache_key_prefers_conversation_then_session() {
        assert_eq!(
            codex_prompt_cache_key("conversation-id", "session-id"),
            Some("conversation-id")
        );
        assert_eq!(codex_prompt_cache_key("", "session-id"), Some("session-id"));
        assert_eq!(codex_prompt_cache_key("", ""), None);
    }

    #[test]
    fn responses_lite_is_provider_scoped_and_non_lite_keeps_normal_shape_with_cache_key() {
        let original = serde_json::json!({
            "model": "grok-4",
            "tools": [{"type": "function", "name": "read_file"}],
            "input": [],
        });
        let mut non_lite = original.clone();
        apply_codex_responses_lite_contract(ProviderId::OpenAiCodex, false, None, &mut non_lite)
            .unwrap();
        assert_eq!(non_lite, original);

        let mut cached_non_lite = original.clone();
        apply_codex_responses_lite_contract(
            ProviderId::OpenAiCodex,
            false,
            Some("stable-thread-key"),
            &mut cached_non_lite,
        )
        .unwrap();
        assert_eq!(cached_non_lite["prompt_cache_key"], "stable-thread-key");
        cached_non_lite
            .as_object_mut()
            .unwrap()
            .remove("prompt_cache_key");
        assert_eq!(cached_non_lite, original);

        let mut xai = original.clone();
        let error =
            apply_codex_responses_lite_contract(ProviderId::Xai, true, None, &mut xai).unwrap_err();
        assert!(error.to_string().contains("OpenAI Codex provider"));
        assert_eq!(xai, original);
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
            conv_id: "thread-1",
            req_id: "request-ignored",
            model_id: "gpt-5-codex",
            session_id: "session-1",
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
        assert_eq!(
            request
                .headers()
                .get(CODEX_SESSION_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("session-1")
        );
        assert_eq!(
            request
                .headers()
                .get(CODEX_THREAD_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("thread-1")
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
    fn codex_extended_reasoning_efforts_decode_from_response_and_sse() {
        for effort in ["max", "ultra"] {
            let response = format!(
                r#"{{
                    "id": "resp_1",
                    "object": "response",
                    "created_at": 0,
                    "model": "gpt-5.6",
                    "status": "completed",
                    "metadata": null,
                    "output": [],
                    "reasoning": {{"effort": "{effort}"}}
                }}"#
            );
            let response =
                deserialize_response_for_provider(response.as_bytes(), ProviderId::OpenAiCodex)
                    .expect("Codex response should normalize into async-openai");
            assert_eq!(
                response
                    .reasoning
                    .as_ref()
                    .and_then(|reasoning| reasoning.effort.clone()),
                Some(rs::ReasoningEffort::Xhigh)
            );
            assert_eq!(
                response
                    .metadata
                    .as_ref()
                    .and_then(|metadata| {
                        metadata.get(
                        xai_grok_sampling_types::OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY
                    )
                    })
                    .map(String::as_str),
                Some(effort)
            );
            let items =
                xai_grok_sampling_types::conversation::response_to_conversation_items(response);
            let assistant_effort = items.iter().find_map(|item| match item {
                xai_grok_sampling_types::conversation::ConversationItem::Assistant(assistant) => {
                    assistant.reasoning_effort
                }
                _ => None,
            });
            assert_eq!(assistant_effort, effort.parse().ok());

            let event = format!(
                r#"{{
                    "type": "response.completed",
                    "sequence_number": 0,
                    "response": {{
                        "id": "resp_1",
                        "object": "response",
                        "created_at": 0,
                        "model": "gpt-5.6",
                        "status": "completed",
                        "metadata": null,
                        "output": [],
                        "reasoning": {{"effort": "{effort}"}}
                    }}
                }}"#
            );
            let event = deserialize_response_event_for_provider(&event, true)
                .expect("Codex SSE should normalize into async-openai");
            let rs::ResponseStreamEvent::ResponseCompleted(event) = event else {
                panic!("expected response.completed");
            };
            assert_eq!(
                event
                    .response
                    .reasoning
                    .as_ref()
                    .and_then(|reasoning| reasoning.effort.clone()),
                Some(rs::ReasoningEffort::Xhigh)
            );
            assert_eq!(
                event
                    .response
                    .metadata
                    .as_ref()
                    .and_then(|metadata| {
                        metadata.get(
                        xai_grok_sampling_types::OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY
                    )
                    })
                    .map(String::as_str),
                Some(effort)
            );
        }
    }

    #[test]
    fn new_with_minimal_config_succeeds() {
        let client = SamplingClient::new(minimal_config()).expect("client should construct");
        assert_eq!(client.api_backend(), ApiBackend::ChatCompletions);
    }

    #[test]
    fn new_applies_extra_headers() {
        let mut cfg = minimal_config();
        cfg.extra_headers
            .insert("x-test-header".to_string(), "test-value".to_string());
        cfg.extra_headers
            .insert("x-XAI-token-auth".to_string(), "xai-grok-cli".to_string());
        let _client = SamplingClient::new(cfg).expect("client with extra headers should construct");
    }

    struct StaticCodexRequestAuth {
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    fn test_codex_binding(generation: u64) -> CredentialBinding {
        let mut binding = CredentialBinding::openai_codex(Some("test-credential".to_owned()));
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
                HeaderValue::from_static("Bearer codex-access-token"),
            );
            headers.insert(
                HeaderName::from_static(CHATGPT_ACCOUNT_ID_HEADER),
                HeaderValue::from_static("account-selected-by-auth-store"),
            );
            headers.insert(
                HeaderName::from_static("x-openai-fedramp"),
                HeaderValue::from_static("true"),
            );
            Ok(test_codex_binding(1))
        }
    }

    struct RecoveringCodexRequestAuth {
        recoveries: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl crate::config::RequestAuth for RecoveringCodexRequestAuth {
        fn apply(
            &self,
            headers: &mut HeaderMap,
        ) -> std::result::Result<CredentialBinding, crate::config::RequestAuthError> {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_static("Bearer codex-access-token"),
            );
            headers.insert(
                HeaderName::from_static(CHATGPT_ACCOUNT_ID_HEADER),
                HeaderValue::from_static("account-selected-by-auth-store"),
            );
            Ok(test_codex_binding(1))
        }

        fn recover_unauthorized(
            &self,
            rejected: CredentialBinding,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            Box::pin(async move {
                assert_eq!(rejected, test_codex_binding(1));
                self.recoveries
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                true
            })
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

    #[test]
    fn codex_turn_state_survives_client_rebuilds_only_within_same_request_id() {
        let (config, _) = codex_config();
        let turn_state = CodexTurnStateStore::default();
        let first_round =
            SamplingClient::new_with_codex_turn_state(config.clone(), turn_state.clone())
                .expect("first Codex client should build");
        fn headers_for(request_id: &str) -> GrokRequestHeaders<'_> {
            GrokRequestHeaders {
                conv_id: "thread-1",
                req_id: request_id,
                model_id: "gpt-5.6-luna",
                session_id: "session-1",
                turn_idx: None,
                agent_id: "agent-1",
                deployment_id: None,
                user_id: None,
            }
        }
        fn build(client: &SamplingClient, request_id: &str) -> reqwest::Request {
            client
                .apply_provider_request_headers(
                    client.http.post("https://example.test/responses"),
                    &headers_for(request_id),
                )
                .expect("Codex request identity should be valid")
                .build()
                .expect("request should build")
        }

        assert!(
            build(&first_round, "request-1")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );

        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            HeaderName::from_static(CODEX_TURN_STATE_HEADER),
            HeaderValue::from_static("opaque-sticky-state"),
        );
        first_round.capture_codex_turn_state(&response_headers, "request-1");

        // Tool results cause a fresh sampler submission and therefore a fresh
        // SamplingClient. It must inherit the actor-owned state from round one.
        let second_round =
            SamplingClient::new_with_codex_turn_state(config.clone(), turn_state.clone())
                .expect("second Codex client should build");
        let mut later_response_headers = HeaderMap::new();
        later_response_headers.insert(
            HeaderName::from_static(CODEX_TURN_STATE_HEADER),
            HeaderValue::from_static("must-not-replace-first-state"),
        );
        second_round.capture_codex_turn_state(&later_response_headers, "request-1");

        let same_turn = build(&second_round, "request-1");
        let replayed = same_turn
            .headers()
            .get(CODEX_TURN_STATE_HEADER)
            .expect("same turn should replay sticky state");
        assert_eq!(replayed.as_bytes(), b"opaque-sticky-state");
        assert!(replayed.is_sensitive());

        // Auxiliary calls have no Grok request ID and must not consume or
        // clear the active user turn's routing state.
        assert!(
            build(&second_round, "")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );
        assert!(
            build(&second_round, "request-1")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_some()
        );

        // A new user turn clears the old value before sending and cannot
        // regain it by presenting the previous request ID later.
        assert!(
            build(&second_round, "request-2")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );
        let third_round = SamplingClient::new_with_codex_turn_state(config, turn_state)
            .expect("third Codex client should build");
        assert!(
            build(&third_round, "request-1")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );
    }

    #[tokio::test]
    async fn provider_auth_recovery_is_scoped_to_codex_request_auth() {
        let recoveries = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut config = SamplerConfig::openai_codex("gpt-5-codex");
        config.request_auth = Some(std::sync::Arc::new(RecoveringCodexRequestAuth {
            recoveries: std::sync::Arc::clone(&recoveries),
        }));
        let client = SamplingClient::new(config).expect("Codex client should build");
        assert!(
            client
                .recover_provider_auth_after_unauthorized(test_codex_binding(1))
                .await
        );
        assert_eq!(recoveries.load(std::sync::atomic::Ordering::SeqCst), 1);

        let xai = SamplingClient::new(minimal_config()).expect("xAI client should build");
        assert!(
            !xai.recover_provider_auth_after_unauthorized(test_codex_binding(1))
                .await
        );
        assert_eq!(recoveries.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn codex_rejects_static_or_generic_header_credentials() {
        let (mut static_key, _) = codex_config();
        static_key.api_key = Some("must-not-be-a-static-token".to_string());
        let error = SamplingClient::new(static_key).unwrap_err();
        assert!(error.to_string().contains("dynamic request authentication"));

        let (mut generic_header, _) = codex_config();
        generic_header.extra_headers.insert(
            "Authorization".to_string(),
            "Bearer must-not-escape".to_string(),
        );
        let error = SamplingClient::new(generic_header).unwrap_err();
        assert!(error.to_string().contains("protected/provider headers"));
        assert!(!error.to_string().contains("must-not-escape"));
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
                Ok(_) => panic!("hostile credential header {header_name} was accepted"),
                Err(error) => error,
            };
            let rendered = error.to_string();
            assert!(
                rendered.contains("unsupported credential header"),
                "unexpected error for {header_name}: {rendered}"
            );
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
                Ok(_) => panic!("invalid FedRAMP value {invalid_value} was accepted"),
                Err(error) => error,
            };
            let rendered = error.to_string();
            assert!(rendered.contains("must be exactly true"));
            assert!(!rendered.contains(invalid_value));
        }

        let mut without_fedramp = SamplerConfig::openai_codex("gpt-5.6-terra");
        without_fedramp.request_auth = Some(std::sync::Arc::new(RecoveringCodexRequestAuth {
            recoveries: Default::default(),
        }));
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
                Ok(_) => panic!("duplicate credential header {header_name} was accepted"),
                Err(error) => error,
            };
            let rendered = error.to_string();
            assert!(
                rendered.contains("duplicate credential headers"),
                "unexpected error for duplicate {header_name}: {rendered}"
            );
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
        request.x_grok_conv_id = Some("codex-thread-123".to_string());
        request.x_grok_req_id = Some("must-not-be-sent".to_string());
        request.x_grok_session_id = Some("codex-session-123".to_string());
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
        assert_eq!(
            headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer codex-access-token")
        );
        assert_eq!(
            headers
                .get(CHATGPT_ACCOUNT_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("account-selected-by-auth-store")
        );
        assert_eq!(
            headers
                .get("x-openai-fedramp")
                .and_then(|value| value.to_str().ok()),
            Some("true")
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
        assert_eq!(
            headers
                .get(CODEX_SESSION_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("codex-session-123")
        );
        assert_eq!(
            headers
                .get(CODEX_THREAD_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("codex-thread-123")
        );
        assert_eq!(
            headers
                .get(CODEX_CLIENT_REQUEST_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("codex-thread-123")
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
            headers.keys().all(|name| !is_xai_specific_header(name)),
            "Codex requests must not contain any xAI-specific header"
        );
        assert_eq!(body["model"], "gpt-5.6-terra");
        assert_eq!(body["reasoning"]["effort"], "max");
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
        assert_eq!(credential, &test_codex_binding(1));
        let rendered = format!("{error:?} {error}");
        assert!(!rendered.contains("test-credential"));
        assert!(!rendered.contains("codex-access-token"));
        assert!(!rendered.contains("account-selected-by-auth-store"));
    }

    #[tokio::test]
    async fn codex_5xx_headers_cannot_reflect_credentials_into_diagnostics() {
        use axum::Router;
        use axum::http::{HeaderValue, StatusCode};
        use axum::response::IntoResponse;
        use axum::routing::post;

        let app = Router::new().route(
            "/responses",
            post(|| async {
                let mut response = (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Bearer reflected-body-secret",
                )
                    .into_response();
                response.headers_mut().insert(
                    "x-error-detail",
                    HeaderValue::from_static("Bearer reflected-header-secret"),
                );
                response.headers_mut().insert(
                    reqwest::header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; reflected=account-secret"),
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
        assert!(!rendered.contains("reflected-header-secret"));
        assert!(!rendered.contains("reflected-body-secret"));
        assert!(!rendered.contains("account-secret"));
        assert!(!rendered.contains("codex-access-token"));
        assert!(!rendered.contains("account-selected-by-auth-store"));
    }

    #[test]
    fn codex_error_bodies_cannot_reflect_credentials_into_diagnostics() {
        let (config, _) = codex_config();
        let client = SamplingClient::new(config).unwrap();
        let reflected = br#"{"error":{"message":"Bearer secret-token account-secret"}}"#;
        let message = client.provider_error_message(reflected);
        assert_eq!(message, "ChatGPT Codex request rejected");
        assert!(!message.contains("secret-token"));
        assert!(!message.contains("account-secret"));

        let encrypted = br#"{"error":{"code":"encrypted_content"}}"#;
        assert_eq!(
            client.provider_error_message(encrypted),
            "ChatGPT Codex request rejected (encrypted_content)"
        );

        let usage_limit = br#"{"error":{"type":"usage_limit_reached","message":"Bearer reflected-secret","resets_at":1704067242}}"#;
        let usage_message = client.provider_error_message(usage_limit);
        assert_eq!(
            usage_message,
            "ChatGPT Codex request rejected (usage limit reached)"
        );
        assert!(!usage_message.contains("reflected-secret"));
        assert!(!usage_message.contains("1704067242"));
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
        assert!(
            client
                .default_headers
                .get(HeaderName::from_static("x-api-key"))
                .is_some()
        );
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
        assert!(client.default_headers.get(AUTHORIZATION).is_some());
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

    impl crate::attribution::Auth401AttributionCallback for CountingCallback {
        fn record_401(&self, consumer: crate::attribution::SamplingConsumer, sent_bearer: bool) {
            self.invocations
                .lock()
                .unwrap()
                .push((consumer, sent_bearer));
        }
    }

    #[test]
    fn live_bearer_resolver_uses_authorization_for_messages_plus_bearer() {
        let cfg = SamplerConfig {
            api_key: Some("stale-bearer".to_string()),
            api_backend: ApiBackend::Messages,
            auth_scheme: AuthScheme::Bearer,
            bearer_resolver: Some(std::sync::Arc::new(StaticBearerResolver("fresh-bearer"))),
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        let request = client
            .post("https://example.test/v1/messages")
            .expect("apply request auth")
            .builder
            .build()
            .expect("request should build");
        let auth = request
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok());
        assert_eq!(auth, Some("Bearer fresh-bearer"));
        assert!(request.headers().get("x-api-key").is_none());
    }

    /// Regression: when `api_key` (which seeds `default_headers` with an
    /// `Authorization: Bearer ...`) AND a `bearer_resolver` are both set,
    /// `post()` must produce **exactly one** `Authorization` header on the
    /// wire. The pre-fix code used `RequestBuilder::header(AUTHORIZATION, ...)`
    /// which appends rather than replaces, causing two identical
    /// `Authorization` headers and a 400 from cli-chat-proxy.
    #[test]
    fn post_emits_single_authorization_with_api_key_and_bearer_resolver() {
        let cfg = SamplerConfig {
            api_key: Some("stale-bearer".to_string()),
            api_backend: ApiBackend::Responses,
            auth_scheme: AuthScheme::Bearer,
            bearer_resolver: Some(std::sync::Arc::new(StaticBearerResolver("fresh-bearer"))),
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");
        let request = client
            .post("https://example.test/v1/responses")
            .expect("apply request auth")
            .builder
            .build()
            .expect("request should build");
        let auth_count = request.headers().get_all(AUTHORIZATION).iter().count();
        assert_eq!(
            auth_count, 1,
            "expected exactly one Authorization header, got {auth_count}"
        );
        assert_eq!(
            request
                .headers()
                .get(AUTHORIZATION)
                .and_then(|v| v.to_str().ok()),
            Some("Bearer fresh-bearer"),
        );
    }

    #[test]
    fn live_bearer_resolver_uses_x_api_key_for_messages_plus_anthropic_api_key() {
        let cfg = SamplerConfig {
            api_key: Some("stale-anthropic".to_string()),
            api_backend: ApiBackend::Messages,
            auth_scheme: AuthScheme::XApiKey,
            bearer_resolver: Some(std::sync::Arc::new(StaticBearerResolver("fresh-anthropic"))),
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
            .and_then(|v| v.to_str().ok());
        assert_eq!(api_key, Some("fresh-anthropic"));
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
    fn bearer_resolver_replaces_authorization_header() {
        #[derive(Debug)]
        struct StaticResolver(String);
        impl crate::config::BearerResolver for StaticResolver {
            fn current_bearer(&self) -> Option<String> {
                Some(self.0.clone())
            }
        }

        let resolver: crate::config::SharedBearerResolver =
            std::sync::Arc::new(StaticResolver("fresh-token".to_string()));
        let cfg = SamplerConfig {
            api_key: Some("stale-token".to_string()),
            api_backend: ApiBackend::Responses,
            bearer_resolver: Some(resolver),
            ..minimal_config()
        };
        let client = SamplingClient::new(cfg).expect("client should build");

        // Build a request to inspect the final headers.
        let builder = client
            .post("https://example.test/v1/responses")
            .expect("apply request auth");
        let request = builder
            .builder
            .body("")
            .build()
            .expect("request should build");

        let auth_values: Vec<_> = request.headers().get_all(AUTHORIZATION).iter().collect();
        assert_eq!(
            auth_values.len(),
            1,
            "expected exactly one Authorization header, got {}: {:?}",
            auth_values.len(),
            auth_values
        );
        assert_eq!(
            auth_values[0].to_str().unwrap(),
            "Bearer fresh-token",
            "Authorization header should contain the resolver's fresh token"
        );
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
