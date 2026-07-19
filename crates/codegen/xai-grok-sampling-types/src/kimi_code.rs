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
pub const KIMI_CODE_ANTHROPIC_VERSION: &str = "2023-06-01";
pub const KIMI_CODE_ANTHROPIC_BETA: &str = "context-management-2025-06-27";
