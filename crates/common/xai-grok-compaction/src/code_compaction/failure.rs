//! Deterministic-vs-transient failure classification for compaction
//! LLM calls.
//!
//! The *policy* lives here (shared across harnesses); the per-harness error
//! types and their wrapping (e.g. grok-build's `SamplingError` →
//! `CompactFailure(acp::Error)`) stay in thin host wrappers that delegate the
//! status/message decisions to these functions.

/// Whether a compaction-call failure is worth retrying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// Retrying the same payload will hit the same failure — the retry loop
    /// should bail without sleeping or re-issuing.
    Deterministic,
    /// Failure may resolve on retry (network blips, 5xx, rate limits).
    Transient,
}

impl FailureKind {
    /// `true` for [`FailureKind::Deterministic`].
    pub fn is_deterministic(self) -> bool {
        matches!(self, Self::Deterministic)
    }
}

/// True when an error message indicates a context-window overflow. Backends report
/// this inconsistently with no stable error code, so we match bounded message markers.
/// Broad token wording is accepted only after excluding rate-limit and throttling
/// responses; those remain transient even when they contain "too many tokens".
pub fn is_context_length_error(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    if m.contains("throttling error")
        || m.contains("service unavailable")
        || m.contains("rate limit")
        || m.contains("too many requests")
    {
        return false;
    }

    m.contains("too long for this model")
        || m.contains("prompt is too long")
        || m.contains("request_too_large")
        || m.contains("input is too long for requested model")
        || m.contains("exceeds the context window")
        || m.contains("maximum prompt length")
        || m.contains("maximum context length")
        || m.contains("maximum allowed input length")
        || m.contains("exceeds the available context size")
        || m.contains("greater than the context length")
        || m.contains("context window exceeds limit")
        || m.contains("exceeded model token limit")
        || m.contains("context_length_exceeded")
        || m.contains("model_context_window_exceeded")
        || m.contains("request entity too large")
        || m.contains("context length is only")
        || m.contains("configured context size")
        || m.contains("longer than the model's context length")
        || m.contains("longer than the model’s context length")
        || m.contains("tokens in request more than max tokens allowed")
        || m.contains("reduce the length of the messages")
        || m.contains("too many tokens")
        || m.contains("token limit exceeded")
        || (m.contains("input token count") && m.contains("exceeds the maximum"))
        || (m.contains("input length") && m.contains("exceeds") && m.contains("context length"))
        || (m.contains("prompt too long") && m.contains("context length"))
}

/// Classify an HTTP API failure (status + message) for the compaction retry
/// loop.
///
/// 4xx responses other than 408 (timeout) and 429 (rate limit) are
/// deterministic; a context-length overflow message is deterministic
/// regardless of status (backends sometimes dress it as a synthesized 500).
/// Everything else (5xx, 408, 429) is transient.
pub fn classify_http_status(status: u16, message: &str) -> FailureKind {
    if is_context_length_error(message)
        || ((400..500).contains(&status) && status != 408 && status != 429)
    {
        FailureKind::Deterministic
    } else {
        FailureKind::Transient
    }
}

/// Classify a provider-style stream error event (`ResponseError` /
/// `ResponseFailed.error`) for the compaction retry loop.
///
/// `code` is the structured `code` field on the event (typically a numeric
/// HTTP status as a string, but some providers also use error-type strings like
/// `"invalid_request_error"`). `message` is the human-readable detail.
///
/// Numeric codes are classified by HTTP-status range. The
/// `invalid_request_error` marker, which can appear in either field, always
/// maps to `Deterministic` (schema violations cannot be fixed by re-sending
/// the same payload). The check order is semantic — marker, then numeric
/// code, then context-length message, then default-to-transient.
pub fn classify_stream_event_error(code: Option<&str>, message: &str) -> FailureKind {
    if matches!(code, Some("invalid_request_error")) || message.contains("invalid_request_error") {
        return FailureKind::Deterministic;
    }

    if let Some(status_code) = code.and_then(|c| c.parse::<u16>().ok())
        && (400..500).contains(&status_code)
        && status_code != 408
        && status_code != 429
    {
        return FailureKind::Deterministic;
    }

    // Size overflow arrives here with no parseable code (`code="none"`); the
    // message is the only signal that re-sending cannot help.
    if is_context_length_error(message) {
        return FailureKind::Deterministic;
    }

    FailureKind::Transient
}

#[cfg(test)]
mod tests {
    use super::*;

    fn det_status(status: u16) -> bool {
        classify_http_status(status, "test").is_deterministic()
    }

    #[test]
    fn http_4xx_is_deterministic_except_408_and_429() {
        assert!(det_status(400));
        assert!(det_status(401));
        assert!(det_status(403));
        assert!(det_status(404));
        assert!(det_status(413));
        assert!(!det_status(408));
        assert!(!det_status(429));
        assert!(!det_status(500));
        assert!(!det_status(502));
        assert!(!det_status(503));
    }

    #[test]
    fn http_500_with_context_length_message_is_deterministic() {
        // The sampler synthesizes status=500 from a streamed size overflow, so
        // status alone reads transient; the message must still short-circuit.
        assert!(
            classify_http_status(
                500,
                "API error (status 500 Internal Server Error): \
                 The prompt is too long for this model's context window."
            )
            .is_deterministic()
        );
    }

    #[test]
    fn stream_event_invalid_request_error_marker_is_deterministic() {
        assert!(
            classify_stream_event_error(
                Some("invalid_request_error"),
                "messages.27.content.1: ..."
            )
            .is_deterministic()
        );
        assert!(
            classify_stream_event_error(
                Some("400"),
                "Provider returned invalid_request_error: messages.X..."
            )
            .is_deterministic()
        );
        assert!(
            classify_stream_event_error(None, "messages.X.content.Y: invalid_request_error: ...")
                .is_deterministic()
        );
    }

    #[test]
    fn stream_event_numeric_codes_match_http_classification() {
        let det = |c: &str| classify_stream_event_error(Some(c), "msg").is_deterministic();
        assert!(det("400"));
        assert!(det("401"));
        assert!(det("403"));
        assert!(det("404"));
        assert!(!det("408"));
        assert!(!det("429"));
        assert!(!det("500"));
        assert!(!det("503"));
    }

    #[test]
    fn stream_event_unknown_code_defaults_to_transient() {
        assert!(!classify_stream_event_error(None, "msg").is_deterministic());
        assert!(!classify_stream_event_error(Some("error"), "msg").is_deterministic());
        assert!(!classify_stream_event_error(Some("overloaded_error"), "msg").is_deterministic());
    }

    #[test]
    fn stream_event_context_length_message_is_deterministic() {
        assert!(
            classify_stream_event_error(
                None,
                "The prompt is too long for this model's context window."
            )
            .is_deterministic()
        );
    }

    #[test]
    fn context_length_error_matches_known_provider_messages() {
        for message in [
            "The prompt is too long for this model's context window.",
            "prompt is too long: 250000 tokens > 200000 maximum",
            "exceeds the maximum prompt length",
            "This model's maximum context length is 128000 tokens",
            "error code: context_length_exceeded",
            r#"{"error":{"type":"request_too_large","message":"Request exceeds the maximum size"}}"#,
            "Input is too long for requested model",
            "Input token count 300000 exceeds the maximum 262144",
            "tokens in request more than max tokens allowed",
            "Please reduce the length of the messages",
            "Input length 131393 exceeds the maximum allowed input length of 131040 tokens.",
            "The input (516368 tokens) is longer than the model's context length (262144 tokens).",
            "Prompt has 5,958,968 tokens, but the configured context size is 256,000 tokens",
            "model_context_window_exceeded",
            "Request entity too large",
            "Too many tokens",
            "Token limit exceeded",
        ] {
            assert!(is_context_length_error(message), "should match: {message}");
        }
    }

    #[test]
    fn context_length_error_excludes_rate_limits_and_transient_failures() {
        for message in [
            "internal server error",
            "rate limited",
            "Rate limit exceeded, please retry after 30 seconds.",
            "Too many requests. Please slow down.",
            "Throttling error: Too many tokens, please wait before trying again.",
            "API error (status 503): Service unavailable: token limit exceeded",
            "Stream failed: throttling error: too many tokens",
            "connection reset by peer",
        ] {
            assert!(
                !is_context_length_error(message),
                "should not match: {message}"
            );
        }
    }
}
