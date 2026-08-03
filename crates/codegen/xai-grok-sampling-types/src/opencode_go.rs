//! Public OpenCode Go endpoint and provider constants.
//!
//! These values contain no credentials. The fixed route tree prevents a Go
//! API key from being reused by xAI, Codex, Kimi Code, or custom providers.

pub const OPENCODE_GO_PROVIDER_NAME: &str = "opencode-go";
pub const OPENCODE_GO_MODEL_NAMESPACE: &str = "opencode-go";
pub const OPENCODE_GO_AUTH_SCOPE: &str = "opencode::go";
pub const OPENCODE_GO_API_KEY_ENV: &str = "GROK_OPENCODE_GO_API_KEY";
pub const OPENCODE_COMPAT_API_KEY_ENV: &str = "OPENCODE_API_KEY";
pub const OPENCODE_GO_BASE_URL: &str = "https://opencode.ai/zen/go/v1";
pub const OPENCODE_GO_CHAT_COMPLETIONS_URL: &str = "https://opencode.ai/zen/go/v1/chat/completions";
pub const OPENCODE_GO_RESPONSES_URL: &str = "https://opencode.ai/zen/go/v1/responses";
pub const OPENCODE_GO_MESSAGES_URL: &str = "https://opencode.ai/zen/go/v1/messages";
pub const OPENCODE_GO_MODELS_URL: &str = "https://opencode.ai/zen/go/v1/models";
pub const EXA_HOSTED_MCP_URL: &str = "https://mcp.exa.ai/mcp";
pub const OPENCODE_GO_ANTHROPIC_VERSION: &str = "2023-06-01";
pub const OPENCODE_GO_MAX_CATALOG_BYTES: usize = 1024 * 1024;
pub const OPENCODE_GO_MAX_CATALOG_MODELS: usize = 256;
