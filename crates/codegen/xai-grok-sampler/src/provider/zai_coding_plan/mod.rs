//! Global personal Z.AI GLM Coding Plan inference transport policy.
//!
//! This module owns the exact endpoint, request body, protected headers,
//! business-code handling, and redacted diagnostics for Coding Plan. It has no
//! auth-store logic and never falls through to Open Platform or BigModel CN.

use reqwest::header::{HeaderMap, HeaderName};
use xai_grok_sampling_types::{
    ApiBackend, ChatCompletionRequest, CredentialBinding, CredentialSourceId, ProviderId,
    ReasoningEffort, Result, SamplingError, ZAI_CODING_PLAN_BASE_URL,
    ZAI_CODING_PLAN_MAX_FUNCTION_TOOLS,
};

pub(crate) fn is_valid_base_url(raw: &str) -> bool {
    let trimmed = raw.trim_end_matches('/');
    if trimmed == ZAI_CODING_PLAN_BASE_URL {
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
    if !provider.is_zai_coding_plan() {
        return Ok(());
    }
    if !is_valid_base_url(base_url) {
        return Err(SamplingError::InvalidConfiguration(
            "Z.AI Coding Plan credentials may only be sent to the canonical global Coding Plan endpoint",
        ));
    }
    if backend != ApiBackend::ChatCompletions {
        return Err(SamplingError::InvalidConfiguration(
            "Z.AI Coding Plan models require the Chat Completions backend",
        ));
    }
    Ok(())
}

pub(crate) fn endpoint(base_url: &str) -> Result<String> {
    validate_config(
        ProviderId::ZaiCodingPlan,
        base_url,
        ApiBackend::ChatCompletions,
    )?;
    Ok(format!("{}/chat/completions", base_url.trim_end_matches('/')))
}

/// Never follow redirects while carrying a Coding Plan credential.
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

fn is_allowed_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization"
            | "content-type"
            | "accept"
            | "user-agent"
            | "traceparent"
            | "tracestate"
    )
}

pub(crate) fn is_protected_header(name: &HeaderName) -> bool {
    !is_allowed_header(name) || name.as_str() == "authorization"
}

fn has_exactly_one_sensitive_authorization(headers: &HeaderMap) -> bool {
    let mut values = headers.get_all("authorization").iter();
    values
        .next()
        .is_some_and(reqwest::header::HeaderValue::is_sensitive)
        && values.next().is_none()
}

pub(crate) fn seal_headers(
    headers: &HeaderMap,
    credential_binding: Option<&CredentialBinding>,
) -> Result<()> {
    let binding = credential_binding.ok_or(SamplingError::InvalidConfiguration(
        "Z.AI Coding Plan request authentication did not provide a credential binding",
    ))?;
    if binding.provider != ProviderId::ZaiCodingPlan
        || binding.source != CredentialSourceId::ZaiCodingPlanApiKey
        || binding.generation == 0
        || binding
            .record_id
            .as_deref()
            .is_none_or(|record_id| record_id.trim().is_empty())
    {
        return Err(SamplingError::InvalidConfiguration(
            "Z.AI Coding Plan request authentication returned a foreign credential binding",
        ));
    }
    if headers.keys().any(|name| !is_allowed_header(name)) {
        return Err(SamplingError::InvalidConfiguration(
            "unapproved headers are forbidden on Z.AI Coding Plan requests",
        ));
    }
    if !has_exactly_one_sensitive_authorization(headers)
        || headers.get("x-api-key").is_some()
        || headers.get("chatgpt-account-id").is_some()
        || headers.get("x-xai-token-auth").is_some()
    {
        return Err(SamplingError::InvalidConfiguration(
            "Z.AI Coding Plan requires exactly one sensitive bearer header",
        ));
    }
    if headers
        .get("content-type")
        .is_some_and(|value| value != "application/json")
    {
        return Err(SamplingError::InvalidConfiguration(
            "Z.AI Coding Plan requests require application/json content",
        ));
    }
    Ok(())
}

fn normalized_effort(effort: ReasoningEffort) -> Option<&'static str> {
    match effort {
        ReasoningEffort::None | ReasoningEffort::Minimal => None,
        ReasoningEffort::Low | ReasoningEffort::Medium | ReasoningEffort::High => Some("high"),
        ReasoningEffort::Xhigh | ReasoningEffort::Max | ReasoningEffort::Ultra => Some("max"),
    }
}

pub(crate) fn chat_body(
    request: &ChatCompletionRequest,
    stream: bool,
) -> Result<serde_json::Value> {
    if request
        .tools
        .as_ref()
        .is_some_and(|tools| tools.len() > ZAI_CODING_PLAN_MAX_FUNCTION_TOOLS)
    {
        return Err(SamplingError::InvalidConfiguration(
            "Z.AI Coding Plan supports at most 128 function tools",
        ));
    }

    let mut body = serde_json::to_value(request).map_err(SamplingError::Serialization)?;
    let Some(object) = body.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Z.AI Coding Plan Chat Completions body must be an object",
        ));
    };
    object.remove("search_parameters");
    object.remove("stream_tool_calls");

    let model = request.model.as_deref().unwrap_or_default();
    match request.reasoning_effort {
        Some(ReasoningEffort::None | ReasoningEffort::Minimal) => {
            object.insert(
                "thinking".to_owned(),
                serde_json::json!({"type": "disabled"}),
            );
            object.remove("reasoning_effort");
        }
        effort => {
            object.insert(
                "thinking".to_owned(),
                serde_json::json!({"type": "enabled", "clear_thinking": false}),
            );
            // The documented compatibility effort field belongs to GLM-5.2.
            // Other catalog models retain preserved thinking but omit it.
            if model == "glm-5.2" {
                let effort = effort
                    .and_then(normalized_effort)
                    .unwrap_or("max")
                    .to_owned();
                object.insert(
                    "reasoning_effort".to_owned(),
                    serde_json::Value::String(effort),
                );
            } else {
                object.remove("reasoning_effort");
            }
        }
    }

    if request.tools.as_ref().is_some_and(|tools| !tools.is_empty()) {
        object.insert("tool_stream".to_owned(), serde_json::Value::Bool(true));
        if !object.contains_key("tool_choice") {
            object.insert(
                "tool_choice".to_owned(),
                serde_json::Value::String("auto".to_owned()),
            );
        }
    }
    if stream {
        object.insert("stream".to_owned(), serde_json::Value::Bool(true));
        object.insert(
            "stream_options".to_owned(),
            serde_json::json!({"include_usage": true}),
        );
    }
    Ok(body)
}

fn parse_code(value: &serde_json::Value) -> Option<i64> {
    value
        .get("code")
        .and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
                .or_else(|| value.as_str().and_then(|value| value.parse::<i64>().ok()))
        })
        .filter(|code| !matches!(code, 0 | 200))
}

fn business_classification(code: i64) -> (reqwest::StatusCode, Option<bool>, &'static str) {
    match code {
        1001..=1004 => (
            reqwest::StatusCode::UNAUTHORIZED,
            Some(false),
            "Z.AI Coding Plan API key was rejected; run `grok login --provider zai-coding-plan`",
        ),
        1113 => (
            reqwest::StatusCode::PAYMENT_REQUIRED,
            Some(false),
            "Z.AI reported insufficient pay-go balance; verify that the Coding Plan endpoint is selected",
        ),
        1211 => (
            reqwest::StatusCode::NOT_FOUND,
            Some(false),
            "The selected model is not present in the Z.AI Coding Plan catalog",
        ),
        1261 => (
            reqwest::StatusCode::BAD_REQUEST,
            Some(false),
            "The Z.AI Coding Plan prompt exceeds the selected model context window",
        ),
        1305 | 1312 => (
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            Some(true),
            "Z.AI Coding Plan is temporarily overloaded or rate limited",
        ),
        1308 | 1310 => (
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            Some(false),
            "Z.AI Coding Plan quota is exhausted; check `/usage` for reset information",
        ),
        1309 => (
            reqwest::StatusCode::FORBIDDEN,
            Some(false),
            "The Z.AI Coding Plan subscription is expired",
        ),
        1311 => (
            reqwest::StatusCode::FORBIDDEN,
            Some(false),
            "The Z.AI Coding Plan subscription does not include the selected model",
        ),
        1313..=1321 => (
            reqwest::StatusCode::FORBIDDEN,
            Some(false),
            "Z.AI Coding Plan rejected the request because of an account or plan restriction",
        ),
        _ => (
            reqwest::StatusCode::BAD_REQUEST,
            Some(false),
            "Z.AI Coding Plan rejected the request",
        ),
    }
}

pub(crate) fn business_error(body: &[u8]) -> Option<SamplingError> {
    let value = serde_json::from_slice::<serde_json::Value>(body).ok()?;
    let success = value.get("success").and_then(serde_json::Value::as_bool);
    let code = parse_code(&value).or_else(|| (success == Some(false)).then_some(-1))?;
    let (status, should_retry, message) = business_classification(code);
    Some(SamplingError::Api {
        status,
        message: message.to_owned(),
        model_metadata: None,
        retry_after_secs: None,
        should_retry,
    })
}

pub(crate) fn stream_error(data: &str) -> Option<SamplingError> {
    business_error(data.as_bytes()).or_else(|| {
        let value = serde_json::from_str::<serde_json::Value>(data).ok()?;
        let error = value.get("error")?;
        let code = error
            .get("code")
            .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
            .unwrap_or(-1);
        let (status, should_retry, message) = business_classification(code);
        Some(SamplingError::Api {
            status,
            message: message.to_owned(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry,
        })
    })
}

pub(crate) fn should_retry(
    status: reqwest::StatusCode,
    body: &[u8],
    header_hint: Option<bool>,
) -> Option<bool> {
    if let Some(SamplingError::Api { should_retry, .. }) = business_error(body) {
        return should_retry;
    }
    if header_hint == Some(false) {
        return Some(false);
    }
    if matches!(
        status,
        reqwest::StatusCode::REQUEST_TIMEOUT
            | reqwest::StatusCode::CONFLICT
            | reqwest::StatusCode::TOO_MANY_REQUESTS
    ) || status.is_server_error()
    {
        Some(true)
    } else {
        header_hint
    }
}

pub(crate) fn response_message(status: reqwest::StatusCode, body: &[u8]) -> String {
    if let Some(SamplingError::Api { message, .. }) = business_error(body) {
        return message;
    }
    match status.as_u16() {
        401 => "Z.AI Coding Plan API key was rejected; run `grok login --provider zai-coding-plan`",
        402 => "Z.AI Coding Plan billing or subscription access is unavailable",
        403 => "The Z.AI Coding Plan subscription does not permit this request",
        408 | 409 | 429 => "Z.AI Coding Plan is temporarily overloaded or rate limited",
        500..=599 => "Z.AI Coding Plan is temporarily unavailable",
        400..=499 => "Z.AI Coding Plan rejected the request contract",
        _ => "Z.AI Coding Plan request failed",
    }
    .to_owned()
}

pub(crate) fn response_header_diagnostics(headers: &HeaderMap) -> Vec<String> {
    // Keep diagnostics fixed-shape. Provider trace and session IDs can be used
    // internally for support, but must not enter ordinary logs or errors.
    let has_trace = ["x-request-id", "x-trace-id", "trace-id"]
        .iter()
        .any(|name| headers.contains_key(*name));
    has_trace
        .then(|| "Z.AI response included a provider trace ID".to_owned())
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampling_types::{ChatRequestMessage, ToolDefinition};

    #[test]
    fn glm_52_body_uses_singular_tool_stream_and_max_reasoning() {
        let mut request = ChatCompletionRequest::new(
            "glm-5.2",
            vec![ChatRequestMessage::user("test")],
        );
        request.reasoning_effort = Some(ReasoningEffort::Xhigh);
        request.tools = Some(vec![ToolDefinition {
            r#type: "function".to_owned(),
            function: xai_grok_sampling_types::FunctionDefinition {
                name: "test".to_owned(),
                description: None,
                parameters: serde_json::json!({"type": "object"}),
                strict: None,
            },
        }]);
        let body = chat_body(&request, true).unwrap();
        assert_eq!(body["tool_stream"], true);
        assert!(body.get("stream_tool_calls").is_none());
        assert_eq!(body["reasoning_effort"], "max");
        assert_eq!(body["thinking"]["clear_thinking"], false);
    }

    #[test]
    fn non_glm_52_omits_reasoning_effort_but_preserves_thinking() {
        let mut request = ChatCompletionRequest::new(
            "glm-4.7",
            vec![ChatRequestMessage::user("test")],
        );
        request.reasoning_effort = Some(ReasoningEffort::High);
        let body = chat_body(&request, true).unwrap();
        assert!(body.get("reasoning_effort").is_none());
        assert_eq!(body["thinking"]["type"], "enabled");
    }

    #[test]
    fn http_200_business_failure_is_canonical_and_retry_aware() {
        let body = br#"{"code":1308,"success":false,"msg":"private detail"}"#;
        let Some(SamplingError::Api {
            should_retry,
            message,
            ..
        }) = business_error(body)
        else {
            panic!("expected provider error");
        };
        assert_eq!(should_retry, Some(false));
        assert!(!message.contains("private detail"));
    }
}
