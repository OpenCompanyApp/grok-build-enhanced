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

/// Wire-credential provenance of a request that failed authentication.
///
/// A 401 for a request that went out with **no** credential header (a
/// fail-closed send while the bearer resolver had nothing wire-valid) is
/// not evidence against the credential itself; retry policies use this to
/// avoid charging credential-rejection budgets for such sends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SentCredential {
    /// The request carried a credential; the server rejected it.
    Sent,
    /// The request went out with no credential header at all.
    Missing,
    /// Provenance unknown (synthesized or legacy errors). Retry policies
    /// treat this like [`SentCredential::Sent`] — fail closed toward
    /// terminating rather than retrying forever.
    #[default]
    Unknown,
}

/// Hand-written so an unrecognized value from a newer peer degrades to
/// `Unknown` instead of failing the whole containing payload
/// (`#[serde(other)]` is not available on externally-tagged enums).
impl<'de> Deserialize<'de> for SentCredential {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        Ok(
            match std::borrow::Cow::<str>::deserialize(deserializer)?.as_ref() {
                "sent" => Self::Sent,
                "missing" => Self::Missing,
                _ => Self::Unknown,
            },
        )
    }
}

impl SentCredential {
    /// Classify from header presence without retaining any credential bytes.
    pub fn from_header_present(present: bool) -> Self {
        if present { Self::Sent } else { Self::Missing }
    }

    pub fn is_missing(self) -> bool {
        matches!(self, Self::Missing)
    }

    /// By reference so it can serve as a serde `skip_serializing_if`.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// Display prefix of [`SamplingError::Serialization`]. Shared with the
/// variant's `#[error(...)]` template so [`SamplingError::serialization_from_rendered`]
/// can never drift from what Display actually emits.
const SERIALIZATION_DISPLAY_PREFIX: &str = "serialization error: ";

#[derive(Debug, Error)]
pub enum SamplingError {
    #[error("{message}")]
    Auth {
        message: String,
        credential: SentCredential,
    },
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
    /// Construct an authentication failure whose wire provenance is not known.
    pub fn auth_unknown(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
            credential: SentCredential::Unknown,
        }
    }

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
            SamplingError::Auth { .. }
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
                should_retry,
                ..
            } if *should_retry != Some(false)
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
            SamplingError::Auth { .. } | SamplingError::ProviderAuthRejected { .. } => false,
            SamplingError::InvalidConfiguration(_) => false,
            SamplingError::Http(err) => is_retryable_reqwest(err),
            SamplingError::RedactedTransport { retryable, .. } => *retryable,
            SamplingError::Serialization(_) => false,
            SamplingError::Api {
                status,
                should_retry,
                ..
            } => {
                if *should_retry == Some(false) {
                    return false;
                }
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

    /// Capacity / overload: HTTP 529, a 5xx whose message clearly says
    /// overloaded (proxies wrap stream overloads in a 500), or a stream
    /// error whose parsed `error_type` is a capacity type (`overloaded_error`
    /// / `service_unavailable_error`). Never reachable from a 4xx or a
    /// request-shaped stream error, whatever the message text. Transient —
    /// worth a short, bounded retry at the call site.
    pub fn is_overloaded(&self) -> bool {
        match self {
            SamplingError::Api {
                status, message, ..
            } => {
                status.as_u16() == 529
                    || (status.is_server_error() && message_looks_overloaded(message))
            }
            // `error_type` is already parsed from the stream payload — trust
            // it alone; matching message text here would let a request-shaped
            // error that merely mentions "overloaded" retry.
            SamplingError::StreamError { error_type, .. } => {
                error_type.eq_ignore_ascii_case("overloaded_error")
                    || error_type.eq_ignore_ascii_case("service_unavailable_error")
            }
            _ => false,
        }
    }

    /// Retry vetoes shared by every retry loop — the sampler actor's
    /// `classify_error` and one-shot callers like `/btw`. One definition so
    /// a new veto lands everywhere at once:
    /// - `x-should-retry: false` — the server says the failure is
    ///   request-content-caused, not transient.
    /// - Context-length overflow — deterministic; re-sending the same
    ///   payload always fails.
    pub fn is_retry_vetoed(&self) -> bool {
        self.should_retry_header() == Some(false) || self.is_context_length_error()
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

fn structured_error_message(bytes: &[u8]) -> Option<String> {
    if let Some((error_type, message)) = std::str::from_utf8(bytes).ok().and_then(try_parse_error) {
        let message = sanitize_provider_error_text(&message);
        if error_type == "unknown" || error_type == "server_error" {
            return Some(message);
        }
        return Some(sanitize_provider_error_text(&format!(
            "{}: {message}",
            sanitize_provider_error_text(&error_type)
        )));
    }
    None
}

/// Parse only a structured provider error envelope. Arbitrary HTML or text
/// bodies are never reflected into diagnostics or terminal output.
pub fn parse_error_bytes(bytes: &[u8]) -> String {
    structured_error_message(bytes).unwrap_or_else(|| "upstream error".to_owned())
}

pub fn status_user_message(status: StatusCode) -> String {
    match status.as_u16() {
        code @ 502..=504 => {
            format!("Grok is temporarily unavailable. Please try again in a moment. (HTTP {code}).")
        }
        code @ 520..=524 => format!(
            "Connection to Grok timed out or was interrupted. Please try again. (HTTP {code})."
        ),
        code if status.is_server_error() => {
            format!("Something went wrong on the server (HTTP {code}).")
        }
        code => format!("Request failed (HTTP {code})."),
    }
}

pub fn user_facing_api_error_message(status: StatusCode, bytes: &[u8]) -> String {
    structured_error_message(bytes).unwrap_or_else(|| status_user_message(status))
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

/// True when an error message indicates a context-window overflow. Keep sampling
/// and compaction retry policy on the same shared classifier so broad provider
/// wording cannot drift into conflicting retry decisions.
pub fn is_context_length_error(message: &str) -> bool {
    if xai_grok_compaction::is_context_length_error(message) {
        return true;
    }
    let message = message.to_ascii_lowercase();
    message.contains("current message") && message.contains("exceeds budget")
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

/// Capacity-style provider text: "Overloaded" / `overloaded_error` (possibly
/// proxy-wrapped) or `service_unavailable_error` (503-shaped capacity).
fn message_looks_overloaded(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    m.contains("overloaded") || m.contains("service_unavailable_error")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overloaded_detects_stream_and_api_shapes() {
        assert!(
            SamplingError::StreamError {
                error_type: "overloaded_error".into(),
                message: "Overloaded".into(),
            }
            .is_overloaded()
        );
        assert!(
            SamplingError::Api {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "stream error (overloaded_error): Overloaded".into(),
                model_metadata: None,
                retry_after_secs: None,
                should_retry: None,
            }
            .is_overloaded()
        );
        assert!(
            SamplingError::Api {
                status: StatusCode::from_u16(529).unwrap(),
                message: "capacity".into(),
                model_metadata: None,
                retry_after_secs: None,
                should_retry: None,
            }
            .is_overloaded()
        );
        assert!(
            SamplingError::Api {
                status: StatusCode::from_u16(529).unwrap(),
                message: "capacity".into(),
                model_metadata: None,
                retry_after_secs: None,
                should_retry: None,
            }
            .is_retryable()
        );
        assert!(!SamplingError::auth_unknown("nope").is_overloaded());
        assert!(
            !SamplingError::Api {
                status: StatusCode::BAD_REQUEST,
                message: "invalid json".into(),
                model_metadata: None,
                retry_after_secs: None,
                should_retry: None,
            }
            .is_overloaded()
        );
        // Only server errors classify on message text — a 4xx that merely
        // mentions "overloaded" is a request error, not capacity.
        assert!(
            !SamplingError::Api {
                status: StatusCode::BAD_REQUEST,
                message: "field `overloaded` is not a valid parameter".into(),
                model_metadata: None,
                retry_after_secs: None,
                should_retry: None,
            }
            .is_overloaded()
        );
        // Stream errors classify on the parsed error_type only — a
        // request-shaped stream error mentioning "overloaded" is not capacity.
        assert!(
            !SamplingError::StreamError {
                error_type: "invalid_request_error".into(),
                message: "tool result mentions overloaded".into(),
            }
            .is_overloaded()
        );
        assert!(
            SamplingError::StreamError {
                error_type: "service_unavailable_error".into(),
                message: "upstream capacity".into(),
            }
            .is_overloaded()
        );
    }

    #[test]
    fn overloaded_message_matches_backend_variants() {
        // 5xx messages that classify as capacity.
        for msg in [
            "Overloaded",
            "stream error (overloaded_error): Overloaded",
            "overloaded_error",
            "service_unavailable_error: try again",
        ] {
            assert!(
                SamplingError::Api {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: msg.into(),
                    model_metadata: None,
                    retry_after_secs: None,
                    should_retry: None,
                }
                .is_overloaded(),
                "expected overloaded for message: {msg}"
            );
        }
        // 5xx messages that do not.
        for msg in ["upstream connect timeout", "internal error"] {
            assert!(
                !SamplingError::Api {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: msg.into(),
                    model_metadata: None,
                    retry_after_secs: None,
                    should_retry: None,
                }
                .is_overloaded(),
                "expected not overloaded for message: {msg}"
            );
        }
    }

    #[test]
    fn retry_veto_covers_header_and_context_length() {
        let vetoed_by_header = SamplingError::Api {
            status: StatusCode::from_u16(529).unwrap(),
            message: "capacity".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: Some(false),
        };
        assert!(vetoed_by_header.is_retry_vetoed());

        let vetoed_by_context = SamplingError::Api {
            status: StatusCode::from_u16(529).unwrap(),
            message: "prompt is too long: 300000 tokens > 200000 maximum".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(vetoed_by_context.is_retry_vetoed());

        let not_vetoed = SamplingError::Api {
            status: StatusCode::from_u16(529).unwrap(),
            message: "capacity".into(),
            model_metadata: None,
            retry_after_secs: None,
            should_retry: None,
        };
        assert!(!not_vetoed.is_retry_vetoed());
    }

    #[test]
    fn context_length_error_matches_backend_variants() {
        for message in [
            "This model's maximum prompt length is 256000 but the request contains 1500000",
            "The prompt is too long for this model's context window.",
            "none: The prompt is too long for this model's context window.",
            "This model's maximum context length is 200000 tokens",
            "invalid_request_error: prompt is too long: 300000 tokens > 200000 maximum",
            "error type: context_length_exceeded",
            "request_too_large",
            "Input length 131393 exceeds the maximum allowed input length of 131040 tokens.",
            "Prompt has 5,958,968 tokens, but the configured context size is 256,000 tokens",
            "Too many tokens",
            "Token limit exceeded",
        ] {
            assert!(is_context_length_error(message), "should match: {message}");
        }
        for message in [
            "rate limited",
            "Rate limit exceeded: too many tokens",
            "Too many requests",
            "Throttling error: Too many tokens, please wait",
            "Service unavailable: token limit exceeded",
            "internal server error",
            "connection reset",
        ] {
            assert!(
                !is_context_length_error(message),
                "should not match: {message}"
            );
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
        assert!(!SamplingError::auth_unknown("nope").is_context_length_error());
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
        assert_eq!(message, "upstream error");
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
        let err = SamplingError::auth_unknown("bad key");
        assert!(err.is_auth_error());
    }

    /// Known values round-trip; an unrecognized value from a newer peer
    /// degrades to `Unknown` instead of failing the containing payload.
    #[test]
    fn sent_credential_wire_compat() {
        for (json, expected) in [
            ("\"sent\"", SentCredential::Sent),
            ("\"missing\"", SentCredential::Missing),
            ("\"unknown\"", SentCredential::Unknown),
            ("\"some-future-variant\"", SentCredential::Unknown),
        ] {
            assert_eq!(
                serde_json::from_str::<SentCredential>(json).unwrap(),
                expected
            );
        }
        assert_eq!(
            serde_json::to_string(&SentCredential::Missing).unwrap(),
            "\"missing\""
        );
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
    fn provider_retry_veto_makes_quota_429_non_retryable() {
        let err = SamplingError::Api {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: "provider quota exhausted".into(),
            model_metadata: None,
            retry_after_secs: Some(60),
            should_retry: Some(false),
        };
        assert!(!err.is_rate_limited());
        assert!(!err.is_retryable());
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

        let auth_error = SamplingError::auth_unknown("bad key");
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
        assert_eq!(SamplingError::auth_unknown("x").retry_after(), None);
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
