//! Public Kimi Code subscription endpoint and protocol constants.
//!
//! These values contain no credentials. Keeping them in the shared sampling
//! types crate lets the shell, sampler, tools, and catalog use one exact
//! provider boundary without URL-derived identity inference.

pub const KIMI_CODE_PROVIDER_NAME: &str = "kimi-code";
pub const KIMI_CODE_MODEL_NAMESPACE: &str = "kimi-code";
pub const KIMI_CODE_AUTH_SCOPE: &str = "kimi::code";
pub const KIMI_CODE_API_KEY_ENV: &str = "KIMI_API_KEY";
pub const KIMI_CODE_BASE_URL: &str = "https://api.kimi.com/coding/v1";
pub const KIMI_CODE_CHAT_COMPLETIONS_URL: &str = "https://api.kimi.com/coding/v1/chat/completions";
pub const KIMI_CODE_MESSAGES_URL: &str = "https://api.kimi.com/coding/v1/messages";
pub const KIMI_CODE_MODELS_URL: &str = "https://api.kimi.com/coding/v1/models";
pub const KIMI_CODE_USAGE_URL: &str = "https://api.kimi.com/coding/v1/usages";
pub const KIMI_CODE_SEARCH_URL: &str = "https://api.kimi.com/coding/v1/search";
pub const KIMI_CODE_FETCH_URL: &str = "https://api.kimi.com/coding/v1/fetch";
pub const KIMI_CODE_FILES_URL: &str = "https://api.kimi.com/coding/v1/files";
pub const KIMI_CODE_MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
pub const KIMI_CODE_REQUEST_TOO_LARGE_ERROR: &str =
    "Kimi Code request exceeds the 2 MiB JSON body limit";
pub const KIMI_CODE_ANTHROPIC_VERSION: &str = "2023-06-01";
pub const KIMI_CODE_ANTHROPIC_BETA: &str = "context-management-2025-06-27";

/// Whether a sanitized sampler error identifies Kimi Code's fixed JSON-body
/// ceiling. The canonical marker is provider-scoped so another provider's
/// generic HTTP 413 cannot trigger Kimi recovery policy.
pub fn is_kimi_code_request_too_large_error(message: &str) -> bool {
    message.contains(KIMI_CODE_REQUEST_TOO_LARGE_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_canonical_marker_identifies_kimi_request_size_error() {
        let message = format!("invalid client configuration: {KIMI_CODE_REQUEST_TOO_LARGE_ERROR}");

        assert!(is_kimi_code_request_too_large_error(&message));
    }

    #[test]
    fn unscoped_body_limit_does_not_identify_kimi_request_size_error() {
        assert!(!is_kimi_code_request_too_large_error(
            "request exceeds the 2 MiB JSON body limit"
        ));
    }

    #[test]
    fn foreign_provider_body_limit_does_not_identify_kimi_request_size_error() {
        assert!(!is_kimi_code_request_too_large_error(
            "OpenAI request exceeds the 2 MiB JSON body limit"
        ));
    }
}
