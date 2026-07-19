//! Kimi Code subscription transport policy.
//!
//! This module owns the exact endpoint, request-body, reasoning, and diagnostic
//! boundary for Kimi Code. It deliberately contains no shell auth-store logic.

mod schema;
mod wire;

pub(crate) use wire::{deserialize_chat_chunk, deserialize_chat_response, stream_error};

use std::fmt::Write as _;

use reqwest::header::{HeaderMap, HeaderName};
use serde::Serialize;
use sha2::{Digest, Sha256};
use xai_grok_sampling_types::{
    ApiBackend, ChatCompletionRequest, CredentialBinding, CredentialSourceId, KIMI_CODE_BASE_URL,
    KIMI_CODE_MAX_REQUEST_BYTES, ProviderId, ReasoningEffort, Result, SamplingError,
};

const KIMI_MESSAGES_BETA_QUERY: &str = "beta=true";

pub(crate) fn is_valid_base_url(raw: &str) -> bool {
    let trimmed = raw.trim_end_matches('/');
    if trimmed == KIMI_CODE_BASE_URL {
        return true;
    }
    #[cfg(any(test, feature = "test-support"))]
    {
        let Ok(url) = reqwest::Url::parse(trimmed) else {
            return false;
        };
        return url.scheme() == "http"
            && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none();
    }
    #[cfg(not(any(test, feature = "test-support")))]
    false
}

pub(crate) fn validate_config(
    provider: ProviderId,
    base_url: &str,
    backend: ApiBackend,
) -> Result<()> {
    if !provider.is_kimi_code() {
        return Ok(());
    }
    if !is_valid_base_url(base_url) {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code credentials may only be sent to the canonical Kimi Code endpoint",
        ));
    }
    if !matches!(backend, ApiBackend::ChatCompletions | ApiBackend::Messages) {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code models require Chat Completions or Messages catalog routing",
        ));
    }
    Ok(())
}

pub(crate) fn endpoint(base_url: &str, backend: ApiBackend) -> Result<String> {
    validate_config(ProviderId::KimiCode, base_url, backend.clone())?;
    let base = base_url.trim_end_matches('/');
    Ok(match backend {
        ApiBackend::ChatCompletions => format!("{base}/chat/completions"),
        ApiBackend::Messages => format!("{base}/messages?{KIMI_MESSAGES_BETA_QUERY}"),
        ApiBackend::Responses => {
            return Err(SamplingError::InvalidConfiguration(
                "Kimi Code does not use the Responses backend",
            ));
        }
    })
}

/// Kimi inference uses a dedicated no-redirect client so provider credentials
/// can never be replayed to a redirect target. This mirrors the stronger
/// transport boundary already used by the ChatGPT Codex adapter.
pub(crate) fn http_client(force_http1: bool) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none());
    if force_http1 {
        builder = builder
            .pool_max_idle_per_host(0)
            .pool_idle_timeout(std::time::Duration::ZERO)
            .http1_only();
    }
    builder.build().map_err(SamplingError::Http)
}

pub(crate) fn is_protected_header(name: &HeaderName) -> bool {
    !is_allowed_kimi_header(name)
        || matches!(
            name.as_str(),
            "authorization" | "x-api-key" | "anthropic-version" | "anthropic-beta"
        )
}

fn is_allowed_kimi_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization"
            | "x-api-key"
            | "anthropic-version"
            | "anthropic-beta"
            | "content-type"
            | "accept"
            | "user-agent"
            | "traceparent"
            | "tracestate"
    )
}

fn has_exactly_one_sensitive_header(headers: &HeaderMap, name: &'static str) -> bool {
    let mut values = headers.get_all(name).iter();
    values
        .next()
        .is_some_and(reqwest::header::HeaderValue::is_sensitive)
        && values.next().is_none()
}

pub(crate) fn seal_headers(
    headers: &HeaderMap,
    backend: ApiBackend,
    credential_binding: Option<&CredentialBinding>,
) -> Result<()> {
    let binding = credential_binding.ok_or(SamplingError::InvalidConfiguration(
        "Kimi Code request authentication did not provide a credential binding",
    ))?;
    if binding.provider != ProviderId::KimiCode
        || binding.source != CredentialSourceId::KimiCodeApiKey
        || binding.generation == 0
        || binding
            .record_id
            .as_deref()
            .is_none_or(|record_id| record_id.trim().is_empty())
    {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code request authentication returned a foreign credential binding",
        ));
    }
    if headers.keys().any(|name| !is_allowed_kimi_header(name)) {
        return Err(SamplingError::InvalidConfiguration(
            "unapproved headers are forbidden on Kimi Code requests",
        ));
    }
    if headers
        .get("content-type")
        .is_some_and(|value| value != "application/json")
    {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code requests require application/json content",
        ));
    }
    match backend {
        ApiBackend::ChatCompletions => {
            if !has_exactly_one_sensitive_header(headers, "authorization")
                || headers.get("x-api-key").is_some()
                || headers.get("anthropic-version").is_some()
                || headers.get("anthropic-beta").is_some()
            {
                return Err(SamplingError::InvalidConfiguration(
                    "Kimi Code Chat Completions requires one sensitive bearer header",
                ));
            }
        }
        ApiBackend::Messages => {
            if !has_exactly_one_sensitive_header(headers, "x-api-key")
                || headers.get("authorization").is_some()
            {
                return Err(SamplingError::InvalidConfiguration(
                    "Kimi Code Messages requires one sensitive x-api-key header",
                ));
            }
            if headers
                .get("anthropic-version")
                .and_then(|value| value.to_str().ok())
                != Some(xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_VERSION)
                || headers
                    .get("anthropic-beta")
                    .and_then(|value| value.to_str().ok())
                    != Some(xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_BETA)
            {
                return Err(SamplingError::InvalidConfiguration(
                    "Kimi Code Messages requires the pinned Anthropic protocol and context-management beta",
                ));
            }
        }
        ApiBackend::Responses => {
            return Err(SamplingError::InvalidConfiguration(
                "Kimi Code does not use the Responses backend",
            ));
        }
    }
    Ok(())
}

fn opaque_cache_affinity(stable: &str, model: &str, effort: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"grok-kimi-cache-v1\0");
    for value in [model, effort, stable] {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value.as_bytes());
    }
    let digest = hasher.finalize();
    let mut affinity = String::with_capacity(45);
    affinity.push_str("grok-kimi-v1-");
    for byte in digest.iter().take(16) {
        let _ = write!(affinity, "{byte:02x}");
    }
    affinity
}

fn cache_affinity(request: &ChatCompletionRequest) -> Option<String> {
    let stable = request
        .x_grok_conv_id
        .as_deref()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            request
                .x_grok_session_id
                .as_deref()
                .filter(|value| !value.is_empty())
        })?;
    let model = request.model.as_deref().unwrap_or("unknown");
    let effort = request
        .reasoning_effort
        .map_or("default", ReasoningEffort::as_str);
    Some(opaque_cache_affinity(stable, model, effort))
}

fn thinking_value(effort: Option<ReasoningEffort>) -> serde_json::Value {
    match effort {
        Some(ReasoningEffort::None) => serde_json::json!({"type": "disabled"}),
        Some(effort) => serde_json::json!({
            "type": "enabled",
            "effort": kimi_effort(effort),
            "keep": "all"
        }),
        None => serde_json::json!({"type": "enabled", "keep": "all"}),
    }
}

pub(crate) fn kimi_effort(effort: ReasoningEffort) -> &'static str {
    // `/models` declares concrete effort strings. Preserve the selected value
    // exactly instead of silently collapsing a catalog-advertised tier into a
    // neighboring one.
    effort.as_str()
}

pub(crate) fn chat_body(
    request: &ChatCompletionRequest,
    stream: bool,
) -> Result<serde_json::Value> {
    let mut body = serde_json::to_value(request).map_err(SamplingError::Serialization)?;
    let Some(object) = body.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code Chat Completions body must be an object",
        ));
    };
    if let Some(max_tokens) = object.remove("max_tokens") {
        object.insert("max_completion_tokens".to_owned(), max_tokens);
    }
    // Kimi consumes effort inside its proprietary `thinking` object. The
    // generic OpenAI `reasoning_effort` field is not part of this route.
    object.remove("reasoning_effort");
    // xAI's hosted-search request extension is never part of the Kimi wire
    // contract; Kimi hosted web runs through its isolated tool endpoints.
    object.remove("search_parameters");
    object.insert(
        "thinking".to_owned(),
        thinking_value(request.reasoning_effort),
    );
    if let Some(cache_key) = cache_affinity(request) {
        object.insert(
            "prompt_cache_key".to_owned(),
            serde_json::Value::String(cache_key),
        );
    }
    if stream {
        object.insert("stream".to_owned(), serde_json::Value::Bool(true));
        object.insert(
            "stream_options".to_owned(),
            serde_json::json!({"include_usage": true}),
        );
    }
    wire::normalize_chat_history(
        &mut body,
        request.reasoning_effort != Some(ReasoningEffort::None),
    )?;
    schema::normalize_tool_schemas(&mut body, "chat_completions")?;
    wire::normalize_tool_call_ids(&mut body, "chat_completions")?;
    validate_body_size(&body)?;
    Ok(body)
}

fn set_messages_output_effort(
    object: &mut serde_json::Map<String, serde_json::Value>,
    effort: Option<ReasoningEffort>,
) -> Result<()> {
    if let Some(effort) = effort {
        let output_config = object
            .entry("output_config")
            .or_insert_with(|| serde_json::Value::Object(Default::default()));
        let output_config =
            output_config
                .as_object_mut()
                .ok_or(SamplingError::InvalidConfiguration(
                    "Kimi Code Messages output_config must be a JSON object",
                ))?;
        output_config.insert(
            "effort".to_owned(),
            serde_json::Value::String(kimi_effort(effort).to_owned()),
        );
        return Ok(());
    }

    let remove_empty = if let Some(output_config) = object.get_mut("output_config") {
        let output_config =
            output_config
                .as_object_mut()
                .ok_or(SamplingError::InvalidConfiguration(
                    "Kimi Code Messages output_config must be a JSON object",
                ))?;
        output_config.remove("effort");
        output_config.is_empty()
    } else {
        false
    };
    if remove_empty {
        object.remove("output_config");
    }
    Ok(())
}

pub(crate) fn shape_messages_body(
    body: &mut serde_json::Value,
    effort: Option<ReasoningEffort>,
    cache_affinity: Option<&str>,
) -> Result<()> {
    let Some(object) = body.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code Messages body must be an object",
        ));
    };
    match effort {
        Some(ReasoningEffort::None) => {
            object.insert(
                "thinking".to_owned(),
                serde_json::json!({"type": "disabled"}),
            );
            set_messages_output_effort(object, None)?;
            object.remove("context_management");
        }
        effort => {
            object.insert(
                "thinking".to_owned(),
                serde_json::json!({"type": "enabled"}),
            );
            set_messages_output_effort(object, effort)?;
            object.insert(
                "context_management".to_owned(),
                serde_json::json!({
                    "edits": [{"type": "clear_thinking_20251015", "keep": "all"}]
                }),
            );
        }
    }
    if let Some(cache_affinity) = cache_affinity.filter(|value| !value.is_empty()) {
        object.insert(
            "metadata".to_owned(),
            serde_json::json!({"user_id": cache_affinity}),
        );
    }
    wire::normalize_messages_history(body)?;
    schema::normalize_tool_schemas(body, "messages")?;
    wire::normalize_tool_call_ids(body, "messages")?;
    wire::inject_messages_cache_breakpoints(body)?;
    validate_body_size(body)
}

pub(crate) fn messages_cache_affinity(
    conversation_id: Option<&str>,
    session_id: Option<&str>,
    model: &str,
    effort: Option<ReasoningEffort>,
) -> Option<String> {
    let stable = conversation_id
        .filter(|value| !value.is_empty())
        .or_else(|| session_id.filter(|value| !value.is_empty()))?;
    Some(opaque_cache_affinity(
        stable,
        model,
        effort.map_or("default", ReasoningEffort::as_str),
    ))
}

pub(crate) fn validate_body_size<T: Serialize>(body: &T) -> Result<()> {
    let bytes = serde_json::to_vec(body).map_err(SamplingError::Serialization)?;
    if bytes.len() > KIMI_CODE_MAX_REQUEST_BYTES {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code request exceeds the 2 MiB JSON body limit",
        ));
    }
    Ok(())
}

pub(crate) fn should_retry(
    status: reqwest::StatusCode,
    body: &[u8],
    header_hint: Option<bool>,
) -> Option<bool> {
    // An explicit provider veto always wins over heuristics in the body.
    if header_hint == Some(false) {
        return Some(false);
    }
    if status != reqwest::StatusCode::TOO_MANY_REQUESTS {
        return if matches!(
            status,
            reqwest::StatusCode::REQUEST_TIMEOUT | reqwest::StatusCode::CONFLICT
        ) || status.as_u16() == 529
        {
            Some(true)
        } else {
            header_hint
        };
    }
    let value = serde_json::from_slice::<serde_json::Value>(body).ok();
    let message = value
        .as_ref()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .or_else(|| value.get("message"))
        })
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if [
        "usage limit",
        "quota exhausted",
        "billing cycle",
        "monthly limit",
        "five-hour limit",
        "5-hour limit",
    ]
    .iter()
    .any(|needle| message.contains(needle))
    {
        Some(false)
    } else if [
        "engine is currently overloaded",
        "too many requests",
        "concurrency limit",
    ]
    .iter()
    .any(|needle| message.contains(needle))
    {
        Some(true)
    } else {
        header_hint
    }
}

pub(crate) fn messages_stream_error(error_type: &str, message: &str) -> SamplingError {
    wire::classify_stream_error(error_type, message)
}

pub(crate) fn response_message(status: reqwest::StatusCode, body: &[u8]) -> String {
    let value = serde_json::from_slice::<serde_json::Value>(body).ok();
    let message = value
        .as_ref()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .or_else(|| value.get("message"))
        })
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let canonical = canonical_error_message(message);
    if !matches!(
        canonical.as_str(),
        "Kimi Code request failed"
            | "Kimi Code request failed; use the provider trace ID when contacting support"
    ) {
        return canonical;
    }

    match status.as_u16() {
        401 => "Kimi Code API key was rejected; run `grok login --provider kimi-code`".to_owned(),
        402 | 403 => {
            "The selected Kimi Code model or context window is not included in this membership"
                .to_owned()
        }
        408 | 409 | 529 => {
            "Kimi Code is temporarily overloaded or at its concurrency limit".to_owned()
        }
        413 => "Kimi Code request exceeds the 2 MiB JSON body limit".to_owned(),
        429 => "Kimi Code quota or concurrency limit was reached".to_owned(),
        500..=599 => "Kimi Code is temporarily unavailable".to_owned(),
        400..=499 => "Kimi Code rejected the request contract".to_owned(),
        _ => canonical,
    }
}

pub(crate) fn canonical_error_message(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("api key")
        || lower.contains("invalid authentication")
        || lower.contains("unauthorized")
    {
        "Kimi Code API key was rejected; run `grok login --provider kimi-code`".to_owned()
    } else if lower.contains("does not have access")
        || lower.contains("permission denied")
        || lower.contains("membership")
        || lower.contains("forbidden")
    {
        "The selected Kimi Code model or context window is not included in this membership"
            .to_owned()
    } else if lower.contains("usage limit")
        || lower.contains("quota")
        || lower.contains("billing cycle")
        || lower.contains("monthly limit")
        || lower.contains("five-hour limit")
        || lower.contains("5-hour limit")
    {
        "Kimi Code usage quota is exhausted; check the Kimi Code Console for reset details"
            .to_owned()
    } else if lower.contains("engine is currently overloaded")
        || lower.contains("overload")
        || lower.contains("too many requests")
        || lower.contains("concurrency limit")
    {
        "Kimi Code is temporarily overloaded or at its concurrency limit".to_owned()
    } else if lower.contains("could not process image")
        || (lower.contains("image") && lower.contains("processing"))
    {
        "Could not process image attached to the Kimi Code request".to_owned()
    } else if lower.contains("reasoning_content is missing") {
        "Kimi Code rejected preserved-thinking history because reasoning content is missing"
            .to_owned()
    } else if lower.contains("total message size") || lower.contains("2097152") {
        "Kimi Code request exceeds the 2 MiB JSON body limit".to_owned()
    } else if lower.contains("token limit") || lower.contains("context") {
        "Kimi Code request exceeds the maximum context length".to_owned()
    } else if lower.contains("function name") && lower.contains("duplicated") {
        "Kimi Code rejected duplicate tool names".to_owned()
    } else if lower.contains("validation") || lower.contains("invalid_request") {
        "Kimi Code rejected the request contract".to_owned()
    } else if message.is_empty() {
        "Kimi Code request failed".to_owned()
    } else {
        "Kimi Code request failed; use the provider trace ID when contacting support".to_owned()
    }
}

pub(crate) fn response_header_diagnostics(headers: &HeaderMap) -> Vec<String> {
    let trace = headers
        .get("x-trace-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        });
    let retry_after = headers
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.min(120));
    let mut out = Vec::new();
    if let Some(trace) = trace {
        out.push(format!("  x-trace-id: {trace}"));
    }
    if let Some(retry_after) = retry_after {
        out.push(format!("  retry-after: {retry_after}"));
    }
    if out.is_empty() {
        out.push("  operational response details omitted".to_owned());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampling_types::{ChatRequestMessage, ToolCallRequest, ToolDefinition};

    #[tokio::test]
    async fn dedicated_client_never_reaches_a_redirect_sink() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        use axum::Router;
        use axum::extract::State;
        use axum::http::{StatusCode, header};
        use axum::routing::{get, post};
        use reqwest::header::HeaderValue;

        async fn sink(State(hits): State<Arc<AtomicUsize>>) -> StatusCode {
            hits.fetch_add(1, Ordering::SeqCst);
            StatusCode::OK
        }

        let sink_hits = Arc::new(AtomicUsize::new(0));
        let app = Router::new()
            .route(
                "/start",
                post(|| async {
                    (
                        StatusCode::TEMPORARY_REDIRECT,
                        [(header::LOCATION, "/credential-sink")],
                    )
                }),
            )
            .route("/credential-sink", get(sink).post(sink))
            .with_state(Arc::clone(&sink_hits));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let mut credential = HeaderValue::from_static("Bearer opaque-credential");
        credential.set_sensitive(true);
        let response = http_client(false)
            .unwrap()
            .post(format!("http://{address}/start"))
            .header(reqwest::header::AUTHORIZATION, credential)
            .send()
            .await
            .unwrap();
        server.abort();

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(response.url().path(), "/start");
        assert_eq!(sink_hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn chat_body_uses_kimi_tokens_thinking_cache_and_stream_usage() {
        let mut request = ChatCompletionRequest::new("k3", vec![ChatRequestMessage::user("hello")]);
        request.max_tokens = Some(123);
        request.reasoning_effort = Some(ReasoningEffort::Max);
        request.x_grok_session_id = Some("session-1".to_owned());
        let body = chat_body(&request, true).unwrap();
        assert_eq!(body["max_completion_tokens"], 123);
        assert!(body.get("max_tokens").is_none());
        assert_eq!(body["thinking"]["effort"], "max");
        assert_eq!(body["thinking"]["keep"], "all");
        assert_eq!(body["stream_options"]["include_usage"], true);
        let cache_key = body["prompt_cache_key"].as_str().unwrap();
        assert!(cache_key.starts_with("grok-kimi-v1-"));
        assert!(!cache_key.contains("session-1"));
    }

    #[test]
    fn chat_body_preserves_each_catalog_advertised_effort_tier() {
        assert_eq!(kimi_effort(ReasoningEffort::Minimal), "minimal");
        assert_eq!(kimi_effort(ReasoningEffort::Medium), "medium");
        assert_eq!(kimi_effort(ReasoningEffort::Xhigh), "xhigh");
        assert_eq!(kimi_effort(ReasoningEffort::Ultra), "ultra");
    }

    #[test]
    fn chat_body_omits_internal_message_model_identity() {
        let assistant = ChatRequestMessage::assistant("answer", "private-model-id", None);
        let request = ChatCompletionRequest::new("k3", vec![assistant]);

        let body = chat_body(&request, false).unwrap();

        assert!(body["messages"][0].get("model_id").is_none());
    }

    #[test]
    fn chat_body_inserts_empty_reasoning_history_when_thinking_is_preserved() {
        let assistant = ChatRequestMessage::assistant("answer", "k3", None);
        let mut request = ChatCompletionRequest::new("k3", vec![assistant]);
        request.reasoning_effort = Some(ReasoningEffort::High);

        let body = chat_body(&request, false).unwrap();

        assert_eq!(body["messages"][0]["reasoning_content"], "");
    }

    #[test]
    fn chat_body_does_not_inject_reasoning_history_when_thinking_is_disabled() {
        let assistant = ChatRequestMessage::assistant("answer", "k3", None);
        let mut request = ChatCompletionRequest::new("k3", vec![assistant]);
        request.reasoning_effort = Some(ReasoningEffort::None);

        let body = chat_body(&request, false).unwrap();

        assert!(body["messages"][0].get("reasoning_content").is_none());
    }

    #[test]
    fn chat_body_omits_empty_assistant_content_for_tool_call_history() {
        let mut assistant = ChatRequestMessage::assistant("", "k3", None);
        assistant.tool_calls = vec![
            ToolCallRequest::function("read_file", r#"{"path":"README.md"}"#).with_id("call_1"),
        ];
        let request = ChatCompletionRequest::new("k3", vec![assistant]);

        let body = chat_body(&request, false).unwrap();

        assert!(body["messages"][0].get("content").is_none());
    }

    #[test]
    fn chat_body_repairs_mcp_tool_schema_in_the_provider_wire_copy() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"mode": {"$ref": "#/$defs/Mode"}},
            "$defs": {"Mode": {"enum": ["fast", "safe"]}}
        });
        let mut request =
            ChatCompletionRequest::new("k3", vec![ChatRequestMessage::user("use the tool")]);
        request.tools = Some(vec![ToolDefinition::function(
            "configure",
            Some("Configure the project"),
            schema,
        )]);

        let body = chat_body(&request, true).unwrap();

        let parameters = &body["tools"][0]["function"]["parameters"];
        assert!(parameters.get("$defs").is_none());
        assert_eq!(parameters["properties"]["mode"]["type"], "string");
    }

    #[test]
    fn messages_shape_uses_disabled_or_preserved_thinking() {
        let mut disabled = serde_json::json!({"model": "k3"});
        shape_messages_body(&mut disabled, Some(ReasoningEffort::None), Some("cache")).unwrap();
        assert_eq!(disabled["thinking"]["type"], "disabled");
        assert!(disabled.get("context_management").is_none());

        let mut enabled = serde_json::json!({"model": "k3"});
        shape_messages_body(&mut enabled, Some(ReasoningEffort::Max), Some("cache")).unwrap();
        assert_eq!(enabled["thinking"]["type"], "enabled");
        assert_eq!(enabled["output_config"]["effort"], "max");
        assert_eq!(enabled["context_management"]["edits"][0]["keep"], "all");
        assert_eq!(enabled["metadata"]["user_id"], "cache");
    }

    #[test]
    fn messages_shape_preserves_structured_output_when_thinking_is_disabled() {
        let mut body = serde_json::json!({
            "model": "k3",
            "output_config": {
                "format": {"type": "json_schema", "schema": {"type": "object"}},
                "effort": "high"
            }
        });

        shape_messages_body(&mut body, Some(ReasoningEffort::None), None).unwrap();

        assert!(body["output_config"].get("effort").is_none());
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
    }

    #[test]
    fn messages_shape_merges_effort_into_structured_output() {
        let mut body = serde_json::json!({
            "model": "k3",
            "output_config": {
                "format": {"type": "json_schema", "schema": {"type": "object"}}
            }
        });

        shape_messages_body(&mut body, Some(ReasoningEffort::Medium), None).unwrap();

        assert_eq!(body["output_config"]["effort"], "medium");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
    }

    #[test]
    fn messages_shape_merges_user_steer_after_tool_results() {
        let mut body = serde_json::json!({
            "messages": [
                {"role": "assistant", "content": [{"type": "tool_use", "id": "call_1", "name": "read_file", "input": {}}]},
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "call_1", "content": "done"}]},
                {"role": "user", "content": [{"type": "text", "text": "also inspect tests"}]}
            ]
        });

        shape_messages_body(&mut body, Some(ReasoningEffort::High), None).unwrap();

        assert_eq!(body["messages"].as_array().unwrap().len(), 2);
        assert_eq!(body["messages"][1]["content"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn messages_shape_keeps_tool_result_after_text_user_turn_separate() {
        let mut body = serde_json::json!({
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "continue"}]},
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "call_1", "content": "done"}]}
            ]
        });

        shape_messages_body(&mut body, Some(ReasoningEffort::High), None).unwrap();

        assert_eq!(body["messages"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn messages_shape_merges_parallel_tool_results() {
        let mut body = serde_json::json!({
            "messages": [
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "call_1", "content": "one"}]},
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "call_2", "content": "two"}]}
            ]
        });

        shape_messages_body(&mut body, Some(ReasoningEffort::High), None).unwrap();

        assert_eq!(body["messages"].as_array().unwrap().len(), 1);
        assert_eq!(body["messages"][0]["content"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn messages_shape_marks_system_prompt_for_prompt_caching() {
        let mut body = serde_json::json!({
            "system": "You are a coding agent.",
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "inspect"}]}
            ]
        });

        shape_messages_body(&mut body, Some(ReasoningEffort::High), None).unwrap();

        assert_eq!(body["system"][0]["type"], "text");
        assert_eq!(body["system"][0]["text"], "You are a coding agent.");
        assert_eq!(body["system"][0]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn messages_shape_marks_final_message_block_and_tool_for_prompt_caching() {
        let mut body = serde_json::json!({
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "inspect"}]}
            ],
            "tools": [
                {"name": "read_file", "description": "Read", "input_schema": {"type": "object"}}
            ]
        });

        shape_messages_body(&mut body, Some(ReasoningEffort::High), None).unwrap();

        assert_eq!(
            body["messages"][0]["content"][0]["cache_control"]["type"],
            "ephemeral"
        );
        assert_eq!(body["tools"][0]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn cache_affinity_is_stable_without_exposing_local_session_identity() {
        let first = messages_cache_affinity(
            Some("private-conversation-id"),
            None,
            "k3",
            Some(ReasoningEffort::High),
        )
        .unwrap();
        let second = messages_cache_affinity(
            Some("private-conversation-id"),
            None,
            "k3",
            Some(ReasoningEffort::High),
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 45);
        assert!(!first.contains("private-conversation-id"));
    }

    #[test]
    fn messages_header_seal_accepts_only_kimi_binding_and_pinned_protocol() {
        let mut headers = HeaderMap::new();
        let mut api_key = reqwest::header::HeaderValue::from_static("sentinel");
        api_key.set_sensitive(true);
        headers.insert("x-api-key", api_key);
        headers.insert(
            "anthropic-version",
            xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_VERSION
                .parse()
                .unwrap(),
        );
        headers.insert(
            "anthropic-beta",
            xai_grok_sampling_types::KIMI_CODE_ANTHROPIC_BETA
                .parse()
                .unwrap(),
        );
        let binding = CredentialBinding {
            provider: ProviderId::KimiCode,
            source: CredentialSourceId::KimiCodeApiKey,
            record_id: Some("opaque-record-id".to_owned()),
            generation: 1,
        };
        seal_headers(&headers, ApiBackend::Messages, Some(&binding)).unwrap();
    }

    #[test]
    fn header_seal_rejects_foreign_provider_identity() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer sentinel".parse().unwrap());
        headers.insert("chatgpt-account-id", "foreign".parse().unwrap());
        let binding = CredentialBinding {
            provider: ProviderId::KimiCode,
            source: CredentialSourceId::KimiCodeApiKey,
            record_id: Some("opaque-record-id".to_owned()),
            generation: 1,
        };
        assert!(seal_headers(&headers, ApiBackend::ChatCompletions, Some(&binding)).is_err());
    }

    #[test]
    fn header_seal_rejects_foreign_credential_aliases() {
        let binding = CredentialBinding {
            provider: ProviderId::KimiCode,
            source: CredentialSourceId::KimiCodeApiKey,
            record_id: Some("opaque-record-id".to_owned()),
            generation: 1,
        };
        for foreign in [
            "cookie",
            "proxy-authorization",
            "x-goog-api-key",
            "openai-api-key",
            "anthropic-auth-token",
            "x-api-key-backup",
        ] {
            let mut headers = HeaderMap::new();
            let mut authorization = reqwest::header::HeaderValue::from_static("Bearer sentinel");
            authorization.set_sensitive(true);
            headers.insert("authorization", authorization);
            headers.insert(
                reqwest::header::HeaderName::from_bytes(foreign.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_static("foreign"),
            );
            assert!(
                seal_headers(&headers, ApiBackend::ChatCompletions, Some(&binding)).is_err(),
                "foreign header {foreign} must not cross the Kimi boundary"
            );
        }
    }

    #[test]
    fn header_seal_rejects_non_sensitive_kimi_credential() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer sentinel".parse().unwrap());
        let binding = CredentialBinding {
            provider: ProviderId::KimiCode,
            source: CredentialSourceId::KimiCodeApiKey,
            record_id: Some("opaque-record-id".to_owned()),
            generation: 1,
        };

        assert!(seal_headers(&headers, ApiBackend::ChatCompletions, Some(&binding)).is_err());
    }

    #[test]
    fn durable_usage_limit_disables_rate_limit_retry() {
        let body = serde_json::to_vec(&serde_json::json!({
            "error": {"message": "You've reached kimi monthly usage limit for this billing cycle."}
        }))
        .unwrap();
        assert_eq!(
            should_retry(reqwest::StatusCode::TOO_MANY_REQUESTS, &body, None),
            Some(false)
        );
    }

    #[test]
    fn transient_overload_allows_rate_limit_retry() {
        let body = serde_json::to_vec(&serde_json::json!({
            "error": {"message": "The engine is currently overloaded"}
        }))
        .unwrap();
        assert_eq!(
            should_retry(reqwest::StatusCode::TOO_MANY_REQUESTS, &body, None),
            Some(true)
        );
    }

    #[test]
    fn explicit_retry_veto_wins_over_overload_body() {
        let body = serde_json::to_vec(&serde_json::json!({
            "error": {"message": "The engine is currently overloaded"}
        }))
        .unwrap();

        assert_eq!(
            should_retry(reqwest::StatusCode::TOO_MANY_REQUESTS, &body, Some(false)),
            Some(false)
        );
    }

    #[test]
    fn provider_overload_status_is_retryable_without_header_hint() {
        let status = reqwest::StatusCode::from_u16(529).unwrap();

        assert_eq!(should_retry(status, b"{}", None), Some(true));
    }

    #[test]
    fn diagnostics_never_reflect_unknown_provider_text() {
        let message = canonical_error_message("secret prompt and bearer sentinel");
        assert!(!message.contains("secret"));
        assert!(!message.contains("bearer"));
    }

    #[test]
    fn status_only_authentication_rejection_remains_actionable() {
        assert_eq!(
            response_message(reqwest::StatusCode::UNAUTHORIZED, b"{}"),
            "Kimi Code API key was rejected; run `grok login --provider kimi-code`"
        );
    }

    #[test]
    fn status_only_membership_rejection_remains_explicit() {
        assert_eq!(
            response_message(reqwest::StatusCode::FORBIDDEN, b"{}"),
            "The selected Kimi Code model or context window is not included in this membership"
        );
    }

    #[test]
    fn canonical_image_rejection_activates_image_strip_recovery() {
        let error = SamplingError::Api {
            status: reqwest::StatusCode::BAD_REQUEST,
            message: canonical_error_message("Could not process image: private payload detail"),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: Some(false),
        };

        assert!(error.is_image_processing_error());
        assert!(!error.to_string().contains("private payload"));
    }

    #[test]
    fn canonical_context_rejection_stays_deterministic_even_when_wrapped_as_a_server_error() {
        let error = SamplingError::Api {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            message: canonical_error_message("context token limit contains private detail"),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };

        assert!(error.is_context_length_error());
        assert!(matches!(
            crate::retry::classify_error(&error, 0, 3, 2),
            crate::retry::RetryDecision::Fatal(_)
        ));
        assert!(!error.to_string().contains("private detail"));
    }
}
