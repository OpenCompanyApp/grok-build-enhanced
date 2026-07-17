use reqwest::StatusCode;
use reqwest::header::HeaderMap;

use xai_grok_sampling_types::{ProviderId, SamplingError};

const DIAGNOSTIC_RESPONSE_HEADERS: &[&str] = &[
    "content-length",
    "content-type",
    "retry-after",
    "x-request-id",
    "x-should-retry",
];

pub(crate) fn invalid_stream_event() -> SamplingError {
    SamplingError::serialization_message("ChatGPT Codex stream event was invalid")
}

pub(crate) fn rejected_stream() -> SamplingError {
    SamplingError::StreamError {
        error_type: "provider_error".to_string(),
        message: "ChatGPT Codex stream rejected".to_string(),
    }
}

pub(crate) fn failed_stream() -> SamplingError {
    SamplingError::StreamError {
        error_type: "provider_error".to_string(),
        message: "ChatGPT Codex response failed".to_string(),
    }
}

pub(crate) fn incomplete_stream() -> SamplingError {
    SamplingError::StreamError {
        error_type: "provider_error".to_string(),
        message: "ChatGPT Codex response was incomplete".to_string(),
    }
}

/// Return whether a Codex error envelope carries one of the small, fixed
/// subscription-exhaustion codes published by the open-source Codex client.
///
/// Provider-controlled messages are deliberately ignored: an error may echo
/// request text, bearer material, or account metadata. Matching only an
/// allowlist of structural codes lets callers distinguish a durable usage
/// block from a short rolling 429 without reflecting any response content.
pub(crate) fn is_usage_limit(value: &serde_json::Value) -> bool {
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

pub(crate) fn usage_limit_should_retry(
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
            .is_some_and(is_usage_limit)
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

/// Recognize the two server-error envelope shapes without logging or retaining
/// any provider-controlled string. The ChatGPT backend can reflect request
/// material, so even an error field is treated as sensitive.
pub(crate) fn try_parse_stream_error(data: &str) -> Option<SamplingError> {
    let value = serde_json::from_str::<serde_json::Value>(data).ok()?;
    let error = value.get("error")?;
    if !(error.is_object() || error.is_string()) {
        return None;
    }
    if is_usage_limit(&value) {
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

/// Convert a response body into one of a bounded set of safe diagnostics. No
/// provider-controlled value is retained or reflected.
pub(crate) fn response_message(body: &[u8]) -> String {
    if serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .as_ref()
        .is_some_and(is_usage_limit)
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
}

/// Expose only the presence of a tiny fixed response-header allowlist. Values
/// at this authenticated third-party boundary are always omitted.
pub(crate) fn response_header_diagnostics(headers: &HeaderMap) -> Vec<String> {
    headers
        .keys()
        .filter(|name| DIAGNOSTIC_RESPONSE_HEADERS.contains(&name.as_str()))
        .map(|name| format!("  {}: [value omitted]", name.as_str()))
        .collect()
}

/// Static, non-retryable failure for a Codex custom/freeform call. The message
/// deliberately excludes the tool name, call identifiers, and raw input.
pub(crate) fn unsupported_custom_tool_call() -> SamplingError {
    SamplingError::Api {
        status: StatusCode::BAD_REQUEST,
        message: "OpenAI Codex returned a custom/freeform tool call, but this build is using the Grok direct-function compatibility loop".to_string(),
        model_metadata: None,
        retry_after_secs: None,
        should_retry: Some(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_never_reflect_provider_payloads() {
        let reflected = "Bearer codex-access-token account-selected-by-auth-store";
        let stream = format!(r#"{{"error":{{"message":"{reflected}"}}}}"#);
        let rendered = try_parse_stream_error(&stream).unwrap().to_string();
        if rendered != "stream error (provider_error): ChatGPT Codex stream rejected" {
            panic!("provider stream diagnostics exposed unexpected detail");
        }
        assert!(!rendered.contains(reflected));

        let body = format!(r#"{{"error":{{"message":"{reflected}"}}}}"#);
        if response_message(body.as_bytes()) != "ChatGPT Codex request rejected" {
            panic!("provider response diagnostics exposed unexpected detail");
        }
    }

    #[test]
    fn usage_limit_uses_only_safe_structural_classification() {
        let reflected = "Bearer codex-access-token account-selected-by-auth-store";
        let data = format!(
            r#"{{"type":"error","status":429,"error":{{"type":"usage_limit_reached","message":"{reflected}","resets_at":1704067242}}}}"#
        );
        let error = try_parse_stream_error(&data).expect("usage-limit envelope");
        let rendered = format!("{error:?} {error}");
        assert!(rendered.contains("usage_limit_reached"));
        assert!(rendered.contains("usage limit reached"));
        assert!(!error.is_retryable());
        assert!(!rendered.contains(reflected));
        assert!(!rendered.contains("1704067242"));

        assert_eq!(
            usage_limit_should_retry(
                ProviderId::OpenAiCodex,
                StatusCode::TOO_MANY_REQUESTS,
                data.as_bytes(),
                None,
            ),
            Some(false)
        );
        assert_eq!(
            usage_limit_should_retry(
                ProviderId::Xai,
                StatusCode::TOO_MANY_REQUESTS,
                data.as_bytes(),
                None,
            ),
            None
        );
        if response_message(data.as_bytes())
            != "ChatGPT Codex request rejected (usage limit reached)"
        {
            panic!("usage-limit diagnostics exposed unexpected detail");
        }
    }

    #[test]
    fn response_headers_are_allowlisted_and_value_free() {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "text/plain; reflected=codex-access-token".parse().unwrap(),
        );
        headers.insert("x-error-detail", "Bearer reflected-secret".parse().unwrap());
        headers.insert("chatgpt-account-id", "reflected-account".parse().unwrap());

        let rendered = response_header_diagnostics(&headers).join("\n");
        assert!(rendered.contains("content-type: [value omitted]"));
        assert!(!rendered.contains("x-error-detail"));
        assert!(!rendered.contains("chatgpt-account-id"));
    }
}
