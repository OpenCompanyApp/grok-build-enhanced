//! Sampling error types.
//!
//! TODO: Move from xai-grok-shell/src/sampling/error.rs

use std::fmt;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::provider::{CredentialBinding, ProviderId};

pub type Result<T> = std::result::Result<T, SamplingError>;

/// Why the model's response was classified as "empty" by [`ConversationResponse::empty_reason`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmptyReason {
    /// The model emitted reasoning tokens but produced no visible content
    /// and no tool calls. The stream completed normally (has `finish_reason`).
    ReasoningOnly,
    /// The stream carried at least one `choice` but the final assistant
    /// message has empty `content` and no tool calls (and no reasoning).
    NoVisibleContent,
}

impl EmptyReason {
    pub fn as_str(self) -> &'static str {
        match self {
            EmptyReason::ReasoningOnly => "reasoning_only",
            EmptyReason::NoVisibleContent => "no_visible_content",
        }
    }
}

impl fmt::Display for EmptyReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Coarse transport classification retained when the underlying HTTP error
/// could contain a provider-controlled URL or other sensitive diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactedTransportKind {
    Timeout,
    Connect,
    Body,
    Request,
    Status,
    Other,
}

impl fmt::Display for RedactedTransportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Timeout => "timeout",
            Self::Connect => "connection",
            Self::Body => "request body",
            Self::Request => "request",
            Self::Status => "HTTP status",
            Self::Other => "transport",
        })
    }
}

/// Structured context captured at L2 stream completion time when the
/// response is classified as empty. Carries everything needed to
/// root-cause the issue from a single log line or error payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponseContext {
    pub reason: EmptyReason,
    /// Whether the response contained reasoning tokens.
    pub had_reasoning: bool,
    /// Byte length of the accumulated `content` string (0 for truly empty).
    pub content_len: usize,
    /// Number of tool calls in the final response.
    pub tool_call_count: usize,
    /// The `finish_reason` from the stream, if any.
    pub finish_reason: Option<String>,
    /// Token usage from the response (when available).
    pub completion_tokens: Option<u32>,
    pub reasoning_tokens: Option<u32>,
    pub prompt_tokens: Option<u32>,
    /// Model that produced the response.
    pub model: String,
    /// Whether at least one `choice` was seen in the stream.
    pub first_choice_seen: bool,
}

impl EmptyResponseContext {
    pub fn finish_reason_str(&self) -> &str {
        self.finish_reason.as_deref().unwrap_or("none")
    }
}

/// Model metadata from response headers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseModelMetadata {
    /// Provider that authenticated and issued the request producing these
    /// headers. This is captured at response time so a later model switch
    /// cannot route an ETag to a different provider's catalog.
    #[serde(default)]
    pub provider: ProviderId,
    /// Exact non-secret credential generation that authenticated the request.
    /// Codex catalog ETag renewal uses this to reject delayed responses from
    /// an older refresh-token generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_binding: Option<CredentialBinding>,
    pub context_window: Option<u64>,
    pub max_completion_tokens: Option<u32>,
    /// `x-models-etag` — triggers model catalog refresh when changed.
    pub models_etag: Option<String>,
}

/// Display prefix of [`SamplingError::Serialization`]. Shared with the
/// variant's `#[error(...)]` template so [`SamplingError::serialization_from_rendered`]
/// can never drift from what Display actually emits.
const SERIALIZATION_DISPLAY_PREFIX: &str = "serialization error: ";

#[derive(Debug, Error)]
pub enum SamplingError {
    #[error("{0}")]
    Auth(String),
    /// A provider-owned credential snapshot was rejected before any response
    /// body was accepted. The binding identifies exactly what signed that
    /// request, allowing a refresh waiter to adopt a newer generation instead
    /// of rotating it again. Its custom `Debug` redacts the opaque record ID.
    #[error("{provider} authentication was rejected")]
    ProviderAuthRejected {
        provider: ProviderId,
        credential: CredentialBinding,
    },
    #[error("invalid client configuration: {0}")]
    InvalidConfiguration(&'static str),
    #[error("request error: {0}")]
    Http(reqwest::Error),
    /// Payload-free replacement for [`SamplingError::Http`] at provider
    /// boundaries where `reqwest::Error` may retain a sensitive request URL.
    #[error("{provider} {kind} error")]
    RedactedTransport {
        provider: ProviderId,
        kind: RedactedTransportKind,
        retryable: bool,
        likely_body_rejected: bool,
    },
    #[error("{prefix}{0}", prefix = SERIALIZATION_DISPLAY_PREFIX)]
    Serialization(serde_json::Error),
    #[error("API error (status {status}): {message}")]
    Api {
        status: StatusCode,
        message: String,
        model_metadata: Option<ResponseModelMetadata>,
        /// Parsed from the `Retry-After` response header (seconds).
        retry_after_secs: Option<u64>,
        /// Parsed from the `x-should-retry` response header.
        /// `Some(true)` = transient, retry may help.
        /// `Some(false)` = request-content error, don't retry.
        /// `None` = header absent (old server or non-proxy origin).
        should_retry: Option<bool>,
    },
    #[error("reqwest error stream: {0}")]
    EventStreamError(String),
    /// Server-side stream error (sent as JSON within the SSE stream)
    #[error("stream error ({error_type}): {message}")]
    StreamError { error_type: String, message: String },
    /// Per-chunk idle timeout — no SSE chunk received from the model within the
    /// configured deadline. NOT retryable: the model (or network path) is stuck,
    /// and replaying the same request would likely stall again.
    #[error("inference idle timeout after {elapsed_secs}s with no chunks")]
    IdleTimeout { elapsed_secs: u64 },
    #[error("empty response from model ({})", context.reason)]
    EmptyResponse { context: EmptyResponseContext },
    #[error("response truncated by max_tokens")]
    MaxTokensTruncation,
    /// A confident server-reported doom loop on the attempt (mid-stream or
    /// on the completed response). Retryable on the recovery loop's own
    /// budget, separate from the transport budget. Carries the raw trigger
    /// labels (never generation content) plus, for telemetry only, the
    /// stream chunk index the mid-stream abort fired at (`None` when the
    /// signal was only seen on the completed response).
    #[error("doom loop detected: {}", triggers.join(", "))]
    DoomLoopDetected {
        triggers: Vec<String>,
        aborted_at_chunk: Option<u64>,
    },
}

impl SamplingError {
    /// Preserve retry/body-rejection behavior without retaining the raw
    /// `reqwest::Error`, whose rendered form may include a sensitive URL.
    pub fn redacted_transport(provider: ProviderId, error: &reqwest::Error) -> Self {
        let kind = if error.is_timeout() {
            RedactedTransportKind::Timeout
        } else if error.is_connect() {
            RedactedTransportKind::Connect
        } else if error.is_body() {
            RedactedTransportKind::Body
        } else if error.is_request() {
            RedactedTransportKind::Request
        } else if error.is_status() {
            RedactedTransportKind::Status
        } else {
            RedactedTransportKind::Other
        };
        let likely_body_rejected =
            (error.is_request() || error.is_body()) && !error.is_timeout() && !error.is_connect();
        Self::RedactedTransport {
            provider,
            kind,
            retryable: is_retryable_reqwest(error),
            likely_body_rejected,
        }
    }

    /// Rebuild a `Serialization` error from a rendered message for non-`Clone`
    /// contexts; it must stay `Serialization` so it remains non-retryable.
    pub fn serialization_message(msg: impl fmt::Display) -> Self {
        Self::Serialization(serde::de::Error::custom(msg))
    }

    /// Rebuild from this variant's full rendered Display (e.g. a round-tripped
    /// `SamplingErrorInfo` message), stripping the Display prefix so the
    /// rebuilt error does not render it twice.
    pub fn serialization_from_rendered(rendered: &str) -> Self {
        Self::serialization_message(
            rendered
                .strip_prefix(SERIALIZATION_DISPLAY_PREFIX)
                .unwrap_or(rendered),
        )
    }

    pub fn is_auth_error(&self) -> bool {
        // Only 401 Unauthorized means the credentials themselves were rejected
        // and warrant a token refresh / re-auth. 403 Forbidden means the
        // request was authenticated successfully but the action is not
        // permitted (e.g. content-safety blocks, ZDR-blocked operations,
        // or other policy denials unrelated to credentials). Treating 403
        // as an auth error triggers a pointless
        // OIDC refresh and then surfaces as acp::Error::auth_required on
        // the client, which in the desktop app tears down the session and
        // can race with invalid_grant_threshold to wipe auth.json.
        matches!(
            self,
            SamplingError::Auth(_)
                | SamplingError::ProviderAuthRejected { .. }
                | SamplingError::Api {
                    status: StatusCode::UNAUTHORIZED,
                    ..
                }
        )
    }

    pub fn is_rate_limited(&self) -> bool {
        matches!(
            self,
            SamplingError::Api {
                status: StatusCode::TOO_MANY_REQUESTS,
                ..
            }
        )
    }

    pub fn rejected_provider_credential(&self) -> Option<(ProviderId, &CredentialBinding)> {
        match self {
            Self::ProviderAuthRejected {
                provider,
                credential,
            } => Some((*provider, credential)),
            _ => None,
        }
    }

    pub fn is_payload_too_large(&self) -> bool {
        matches!(
            self,
            SamplingError::Api {
                status: StatusCode::PAYLOAD_TOO_LARGE,
                ..
            }
        )
    }

    /// `true` when the error looks like a connection reset or broken pipe
    /// during request upload — the pattern nginx produces when it rejects an
    /// oversized payload by closing the connection instead of responding 413.
    ///
    /// Timeouts and connect failures are excluded: those are unrelated to
    /// payload size and stripping images on them would lose context for no
    /// reason.
    pub fn is_likely_body_rejected(&self) -> bool {
        match self {
            SamplingError::Http(err) => {
                // `is_request()` covers broken-pipe / connection-reset during
                // body upload.  `is_body()` covers stream-write failures.
                // Exclude timeouts and connect errors — those are unrelated.
                (err.is_request() || err.is_body()) && !err.is_timeout() && !err.is_connect()
            }
            SamplingError::RedactedTransport {
                likely_body_rejected,
                ..
            } => *likely_body_rejected,
            _ => false,
        }
    }

    /// The server rejected the request because the conversation history
    /// contains `encrypted_content` from a different model family that the
    /// current model cannot decrypt. Never retryable — the user must start
    /// a new session.
    pub fn is_encrypted_content_error(&self) -> bool {
        matches!(
            self,
            SamplingError::Api {
                status: StatusCode::BAD_REQUEST,
                message,
                ..
            } if message.contains("encrypted_content")
        )
    }

    /// The API rejected the request because an inline image could not be
    /// processed. Matches both direct 400 and proxy-wrapped 500 responses.
    /// Exact-case match — consistent with `is_encrypted_content_error`.
    pub fn is_image_processing_error(&self) -> bool {
        matches!(
            self,
            SamplingError::Api {
                status,
                message,
                ..
            } if matches!(status.as_u16(), 400 | 500) && message.contains("Could not process image")
        )
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            SamplingError::Auth(_) => false,
            SamplingError::ProviderAuthRejected { .. } => false,
            SamplingError::InvalidConfiguration(_) => false,
            SamplingError::Http(err) => is_retryable_reqwest(err),
            SamplingError::RedactedTransport { retryable, .. } => *retryable,
            SamplingError::Serialization(_) => false,
            SamplingError::Api { status, .. } => {
                matches!(
                    status.as_u16(),
                    408 | 409 | 429 | 500 | 502 | 503 | 504 | 520 | 529
                )
            }
            SamplingError::EventStreamError(_) => true,
            SamplingError::StreamError { error_type, .. } => error_type != "usage_limit_reached",
            SamplingError::IdleTimeout { .. } => false,
            SamplingError::EmptyResponse { .. } => true,
            SamplingError::MaxTokensTruncation => false,
            SamplingError::DoomLoopDetected { .. } => true,
        }
    }

    pub fn model_metadata(&self) -> Option<&ResponseModelMetadata> {
        match self {
            SamplingError::Api { model_metadata, .. } => model_metadata.as_ref(),
            _ => None,
        }
    }

    pub fn retry_after(&self) -> Option<u64> {
        match self {
            SamplingError::Api {
                retry_after_secs, ..
            } => *retry_after_secs,
            _ => None,
        }
    }

    /// Server hint on whether this error is worth retrying.
    pub fn should_retry_header(&self) -> Option<bool> {
        match self {
            SamplingError::Api { should_retry, .. } => *should_retry,
            _ => None,
        }
    }

    /// True when this error is a context-window/size overflow — deterministic,
    /// so retrying the same payload can't help. See [`is_context_length_error`].
    pub fn is_context_length_error(&self) -> bool {
        match self {
            SamplingError::Api { message, .. } | SamplingError::StreamError { message, .. } => {
                is_context_length_error(message)
            }
            _ => false,
        }
    }
}

impl From<reqwest::Error> for SamplingError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for SamplingError {
    fn from(value: serde_json::Error) -> Self {
        tracing::debug!("Serde deserialization error: {:?}", &value);
        Self::Serialization(value)
    }
}

/// OpenAI-standard provider error format: `{"error": {"message": "...", "type": "..."}}`.
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    message: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

/// Flat error from the Grok proxy/gateway: `{"code": "...", "error": "..."}`.
#[derive(Debug, Deserialize)]
struct FlatErrorResponse {
    error: String,
    #[serde(default)]
    code: Option<String>,
}

/// Extract `(error_type, message)` from either error format.
fn try_parse_error(data: &str) -> Option<(String, String)> {
    if let Ok(resp) = serde_json::from_str::<ErrorResponse>(data) {
        return Some((
            resp.error.kind.unwrap_or_else(|| "unknown".to_string()),
            resp.error
                .message
                .unwrap_or_else(|| "unknown error".to_string()),
        ));
    }
    if let Ok(flat) = serde_json::from_str::<FlatErrorResponse>(data) {
        return Some((
            flat.code.unwrap_or_else(|| "server_error".to_string()),
            flat.error,
        ));
    }
    None
}

const MAX_PROVIDER_ERROR_CHARS: usize = 2_048;
const EMPTY_PROVIDER_ERROR: &str = "provider returned an empty error response";
const HTML_PROVIDER_ERROR: &str = "provider returned an HTML error response";

fn sanitize_provider_error_text(raw: &str) -> String {
    let trimmed = raw.trim();
    let lower_prefix: String = trimmed
        .chars()
        .take(128)
        .collect::<String>()
        .to_ascii_lowercase();
    if lower_prefix.starts_with('<')
        && (lower_prefix.contains("<html")
            || lower_prefix.contains("<!doctype")
            || lower_prefix.contains("<head")
            || lower_prefix.contains("<body"))
    {
        return HTML_PROVIDER_ERROR.to_owned();
    }

    let mut output = String::with_capacity(trimmed.len().min(MAX_PROVIDER_ERROR_CHARS));
    let mut output_chars = 0;
    let mut pending_space = false;
    let mut truncated = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() || ch.is_control() {
            pending_space = !output.is_empty();
            continue;
        }
        if pending_space {
            if output_chars >= MAX_PROVIDER_ERROR_CHARS {
                truncated = true;
                break;
            }
            output.push(' ');
            output_chars += 1;
            pending_space = false;
        }
        if output_chars >= MAX_PROVIDER_ERROR_CHARS {
            truncated = true;
            break;
        }
        // Keep diagnostics plain-text even when a UI later renders them in an
        // HTML-capable surface.
        match ch {
            '<' => output.push('‹'),
            '>' => output.push('›'),
            _ => output.push(ch),
        }
        output_chars += 1;
    }
    if output.is_empty() {
        return EMPTY_PROVIDER_ERROR.to_owned();
    }
    if truncated {
        output.push('…');
    }
    output
}

pub fn parse_error_bytes(bytes: &[u8]) -> String {
    if let Some((error_type, message)) = std::str::from_utf8(bytes).ok().and_then(try_parse_error) {
        let message = sanitize_provider_error_text(&message);
        if error_type == "unknown" || error_type == "server_error" {
            return message;
        }
        return sanitize_provider_error_text(&format!(
            "{}: {message}",
            sanitize_provider_error_text(&error_type)
        ));
    }
    sanitize_provider_error_text(&String::from_utf8_lossy(bytes))
}

pub fn try_parse_stream_error(data: &str) -> Option<SamplingError> {
    let (error_type, message) = try_parse_error(data)?;
    let error_type = sanitize_provider_error_text(&error_type);
    let message = sanitize_provider_error_text(&message);
    tracing::warn!(error_type, message, "Server-side stream error");
    Some(SamplingError::StreamError {
        error_type,
        message,
    })
}

/// Detect a provider stream-error envelope without retaining, logging, or
/// returning its provider-controlled values.
pub fn try_parse_stream_error_redacted(
    data: &str,
    provider: crate::ProviderId,
) -> Option<SamplingError> {
    try_parse_error(data)?;
    tracing::warn!(
        provider = %provider,
        data_len = data.len(),
        "Provider-side stream error; details omitted"
    );
    Some(SamplingError::EventStreamError(format!(
        "{provider} event stream failed"
    )))
}

/// True when an error message indicates a context-window overflow. Backends report
/// this inconsistently with no stable error code, so we match the message text; it's
/// deterministic (re-sending the same payload always fails), so callers must not retry.
pub fn is_context_length_error(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    m.contains("too long for this model")
        || m.contains("prompt is too long")
        || m.contains("maximum prompt length")
        || m.contains("maximum context length")
        || m.contains("context_length_exceeded")
}

/// Decide whether a [`reqwest::Error`] is worth retrying.
pub fn is_retryable_reqwest(err: &reqwest::Error) -> bool {
    if err.is_timeout() || err.is_connect() {
        return true;
    }

    if err.is_status() {
        return matches!(
            err.status(),
            Some(status) if status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS
        );
    }

    if err.is_request() || err.is_body() {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_length_error_matches_backend_variants() {
        for msg in [
            "This model's maximum prompt length is 256000 but the request contains 1500000",
            "The prompt is too long for this model's context window.",
            "none: The prompt is too long for this model's context window.",
            "This model's maximum context length is 200000 tokens",
            "invalid_request_error: prompt is too long: 300000 tokens > 200000 maximum",
            "error type: context_length_exceeded",
        ] {
            assert!(is_context_length_error(msg), "should match: {msg}");
        }
        for msg in ["rate limited", "internal server error", "connection reset"] {
            assert!(!is_context_length_error(msg), "should not match: {msg}");
        }
        // The method delegates for the Api/StreamError variants.
        let api = SamplingError::Api {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "none: The prompt is too long for this model's context window.".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(api.is_context_length_error());
        assert!(
            SamplingError::StreamError {
                error_type: "overloaded_error".into(),
                message: "prompt is too long".into(),
            }
            .is_context_length_error()
        );
        assert!(!SamplingError::Auth("nope".into()).is_context_length_error());
    }

    #[test]
    fn serialization_message_stays_serialization_and_non_retryable() {
        let err = SamplingError::serialization_message("bad payload at line 1 column 7");
        assert!(matches!(err, SamplingError::Serialization(_)));
        assert!(!err.is_retryable());
        assert!(err.to_string().contains("bad payload at line 1 column 7"));
    }

    #[test]
    fn serialization_from_rendered_round_trips_display() {
        // Derived from a REAL error's Display so a template rewording cannot
        // silently desynchronize the strip from the prefix it mirrors.
        let original =
            SamplingError::Serialization(serde_json::from_str::<i32>("not a number").unwrap_err());
        let rendered = original.to_string();
        let rebuilt = SamplingError::serialization_from_rendered(&rendered);
        assert!(matches!(rebuilt, SamplingError::Serialization(_)));
        assert!(!rebuilt.is_retryable());
        assert_eq!(
            rebuilt.to_string(),
            rendered,
            "rendered Display must round-trip without double-prefixing"
        );
        // Bare (non-rendered) input gains the prefix exactly once.
        assert_eq!(
            SamplingError::serialization_from_rendered("bare message").to_string(),
            format!("{SERIALIZATION_DISPLAY_PREFIX}bare message"),
        );
    }

    #[test]
    fn idle_timeout_is_not_retryable() {
        let err = SamplingError::IdleTimeout { elapsed_secs: 300 };
        assert!(
            !err.is_retryable(),
            "IdleTimeout must not be retried — would cause 3× amplification"
        );
    }

    #[test]
    fn event_stream_error_is_retryable() {
        // Verify the existing contract hasn't changed — EventStreamError is retryable.
        let err = SamplingError::EventStreamError("connection reset".into());
        assert!(err.is_retryable());
    }

    #[test]
    fn usage_limit_stream_error_is_terminal_but_other_stream_errors_remain_retryable() {
        let usage_limit = SamplingError::StreamError {
            error_type: "usage_limit_reached".into(),
            message: "ChatGPT Codex stream rejected (usage limit reached)".into(),
        };
        assert!(!usage_limit.is_retryable());

        let overloaded = SamplingError::StreamError {
            error_type: "overloaded_error".into(),
            message: "try again".into(),
        };
        assert!(overloaded.is_retryable());
    }

    #[test]
    fn idle_timeout_display() {
        let err = SamplingError::IdleTimeout { elapsed_secs: 120 };
        let msg = err.to_string();
        assert!(
            msg.contains("120s"),
            "Display should include elapsed_secs: {msg}"
        );
    }

    #[test]
    fn try_parse_stream_error_flat_format() {
        let data = r#"{"code":"The service is currently unavailable","error":"Service temporarily unavailable. The model did not respond to this request."}"#;
        let err = try_parse_stream_error(data).expect("should parse flat error");
        match err {
            SamplingError::StreamError {
                error_type,
                message,
            } => {
                assert_eq!(error_type, "The service is currently unavailable");
                assert_eq!(
                    message,
                    "Service temporarily unavailable. The model did not respond to this request."
                );
            }
            other => panic!("expected StreamError, got {other:?}"),
        }
    }

    #[test]
    fn redacted_stream_error_omits_provider_controlled_values() {
        const SECRET: &str = "Bearer reflected-provider-secret";
        let data = format!(r#"{{"error":{{"type":"server_error","message":"{SECRET}"}}}}"#);

        let error = try_parse_stream_error_redacted(&data, crate::ProviderId::Custom)
            .expect("error envelope should be detected");
        let rendered = format!("{error:?} {error}");

        assert_eq!(
            error.to_string(),
            "reqwest error stream: custom event stream failed"
        );
        assert!(!rendered.contains(SECRET));
    }

    #[test]
    fn try_parse_stream_error_valid_chunk_returns_none() {
        let data = r#"{"id":"abc","object":"chat.completion.chunk","created":0,"model":"test","choices":[]}"#;
        assert!(
            try_parse_stream_error(data).is_none(),
            "valid chunk should not be parsed as error"
        );
    }

    #[test]
    fn parse_error_bytes_flat_format() {
        let bytes =
            br#"{"code":"The service is currently unavailable","error":"Service temporarily unavailable."}"#;
        let msg = parse_error_bytes(bytes);
        assert_eq!(
            msg,
            "The service is currently unavailable: Service temporarily unavailable."
        );
    }

    #[test]
    fn provider_error_text_is_bounded_and_control_neutral() {
        let body = format!(
            r#"{{"error":{{"type":"server_error","message":"{}\n\u0000tail"}}}}"#,
            "x".repeat(MAX_PROVIDER_ERROR_CHARS * 2)
        );
        let message = parse_error_bytes(body.as_bytes());
        assert!(message.chars().count() <= MAX_PROVIDER_ERROR_CHARS + 1);
        assert!(message.ends_with('…'));
        assert!(!message.contains('\n'));
        assert!(!message.contains('\0'));
    }

    #[test]
    fn html_provider_error_body_is_not_reflected() {
        let body = b"<!doctype html><html><body><script>synthetic-marker</script></body></html>";
        let message = parse_error_bytes(body);
        assert_eq!(message, HTML_PROVIDER_ERROR);
        assert!(!message.contains("synthetic-marker"));
        assert!(!message.contains('<'));
    }

    #[test]
    fn json_provider_error_message_is_html_neutral() {
        let body = br#"{"error":{"type":"bad_request","message":"bad <b>tag</b>"}}"#;
        let message = parse_error_bytes(body);
        assert_eq!(message, "bad_request: bad ‹b›tag‹/b›");
        assert!(!message.contains('<'));
        assert!(!message.contains('>'));
    }

    /// Regression test: 403 Forbidden must NOT be classified as an auth
    /// error. The proxy returns 403 for policy denials that are unrelated
    /// to the caller's credentials (content-safety blocks, ZDR-gated
    /// operations, or other usage-policy blocks). Misclassifying these as
    /// auth errors triggers a pointless OIDC
    /// refresh and surfaces as acp::Error::auth_required on the client,
    /// tearing down the session and risking an
    /// `invalid_grant_threshold`-triggered wipe of auth.json.
    #[test]
    fn forbidden_is_not_auth_error() {
        let err = SamplingError::Api {
            status: StatusCode::FORBIDDEN,
            message: "Content violates usage guidelines.".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(
            !err.is_auth_error(),
            "403 Forbidden must not be treated as an auth error"
        );
    }

    #[test]
    fn unauthorized_is_auth_error() {
        let err = SamplingError::Api {
            status: StatusCode::UNAUTHORIZED,
            message: "Invalid or expired credentials".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(
            err.is_auth_error(),
            "401 Unauthorized must be an auth error"
        );
    }

    #[test]
    fn auth_variant_is_auth_error() {
        let err = SamplingError::Auth("bad key".into());
        assert!(err.is_auth_error());
    }

    #[test]
    fn rate_limited_api_error_is_detected() {
        let err = SamplingError::Api {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: "Rate limit exceeded".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(err.is_rate_limited());
        assert!(err.is_retryable(), "429 should be retryable");
        assert!(!err.is_auth_error());
        assert!(!err.is_payload_too_large());
    }

    #[test]
    fn non_rate_limit_errors_are_not_rate_limited() {
        let server_error = SamplingError::Api {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(!server_error.is_rate_limited());

        let auth_error = SamplingError::Auth("bad key".into());
        assert!(!auth_error.is_rate_limited());

        let timeout = SamplingError::IdleTimeout { elapsed_secs: 30 };
        assert!(!timeout.is_rate_limited());
    }

    #[test]
    fn retry_after_returns_header_value() {
        let err = SamplingError::Api {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: "slow down".into(),
            model_metadata: None,
            retry_after_secs: Some(42),
            should_retry: None,
        };
        assert_eq!(err.retry_after(), Some(42));
    }

    #[test]
    fn retry_after_returns_none_when_absent() {
        let err = SamplingError::Api {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: "slow down".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn retry_after_returns_none_for_non_api_errors() {
        assert_eq!(SamplingError::Auth("x".into()).retry_after(), None);
        assert_eq!(
            SamplingError::IdleTimeout { elapsed_secs: 10 }.retry_after(),
            None
        );
    }

    #[test]
    fn encrypted_content_400_is_detected() {
        let err = SamplingError::Api {
            status: StatusCode::BAD_REQUEST,
            message: "Could not decrypt the provided encrypted_content. Ensure the value is the unmodified encrypted_content from a previous response.".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(err.is_encrypted_content_error());
        assert!(
            !err.is_retryable(),
            "encrypted_content errors must not be retried"
        );
    }

    #[test]
    fn encrypted_content_wrong_status_not_detected() {
        let err = SamplingError::Api {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "encrypted_content decryption failed".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(
            !err.is_encrypted_content_error(),
            "only 400 should match, not 500"
        );
    }

    #[test]
    fn encrypted_content_unrelated_400_not_detected() {
        let err = SamplingError::Api {
            status: StatusCode::BAD_REQUEST,
            message: "Invalid model parameter".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(
            !err.is_encrypted_content_error(),
            "unrelated 400 errors must not match"
        );
    }

    #[test]
    fn image_processing_error_direct_400_detected() {
        let err = SamplingError::Api {
            status: StatusCode::BAD_REQUEST,
            message: "Could not process image: unsupported format".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(err.is_image_processing_error());
        assert!(!err.is_encrypted_content_error());
    }

    #[test]
    fn image_processing_error_500_wrapped_detected() {
        let err = SamplingError::Api {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "upstream error: 400 Bad Request: Could not process image".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(err.is_image_processing_error());
    }

    #[test]
    fn image_processing_error_unrelated_400_not_detected() {
        let err = SamplingError::Api {
            status: StatusCode::BAD_REQUEST,
            message: "Invalid model parameter".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(!err.is_image_processing_error());
    }

    #[test]
    fn image_processing_error_unrelated_500_not_detected() {
        let err = SamplingError::Api {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal server error".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(!err.is_image_processing_error());
    }

    #[test]
    fn image_processing_error_wrong_status_not_detected() {
        let err = SamplingError::Api {
            status: StatusCode::BAD_GATEWAY,
            message: "Could not process image".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(
            !err.is_image_processing_error(),
            "only 400 and 500 should match"
        );
    }

    #[test]
    fn image_processing_error_400_is_not_retryable_standalone() {
        let err = SamplingError::Api {
            status: StatusCode::BAD_REQUEST,
            message: "Could not process image".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(
            !err.is_retryable(),
            "direct 400 must not be retryable by is_retryable()"
        );
    }

    #[test]
    fn provider_auth_rejection_is_typed_and_redacts_record_identity() {
        let mut credential =
            CredentialBinding::openai_codex(Some("sentinel-credential-id".to_owned()));
        credential.generation = 9;
        let err = SamplingError::ProviderAuthRejected {
            provider: ProviderId::OpenAiCodex,
            credential: credential.clone(),
        };

        assert!(err.is_auth_error());
        assert!(!err.is_retryable());
        assert_eq!(
            err.rejected_provider_credential(),
            Some((ProviderId::OpenAiCodex, &credential))
        );
        let rendered = format!("{err:?} {err}");
        assert!(!rendered.contains("sentinel-credential-id"));
        assert!(!rendered.contains("generation"));
        assert!(!rendered.contains('9'));
        assert!(rendered.contains("authentication was rejected"));
    }

    #[test]
    fn legacy_response_metadata_defaults_to_xai_provider() {
        let metadata: ResponseModelMetadata = serde_json::from_value(serde_json::json!({
            "context_window": 8192,
            "models_etag": "legacy-etag"
        }))
        .unwrap();

        assert_eq!(metadata.provider, ProviderId::Xai);
        assert!(metadata.credential_binding.is_none());
        assert_eq!(metadata.models_etag.as_deref(), Some("legacy-etag"));
    }
}
