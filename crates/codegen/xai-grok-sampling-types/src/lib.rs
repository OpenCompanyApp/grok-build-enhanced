//! Pure data types for the xAI sampling / chat-completion API layer.
//!
//! This crate contains the API-agnostic conversation types, chat completion
//! request/response types, streaming types, and error types used across the
//! xAI agent stack.  It intentionally contains **no I/O** (no HTTP clients,
//! no file system access) so it can be depended on by downstream crates
//! (e.g., `xai-chat-state`) without pulling in the full `xai-grok-shell`.

pub mod conversation;
pub mod doom_loop;
pub mod error;
pub mod kimi_code;
pub mod messages;
pub mod openai_codex;
pub mod provider;
pub mod serde_helpers;
pub mod types;
pub mod zai_coding_plan;

pub use self::conversation::*;
pub use self::doom_loop::{
    DOOM_LOOP_CHECK_EVENT_TYPE, DOOM_LOOP_CHECK_HEADER, DoomLoopPeek, DoomLoopRecoveryPolicy,
    DoomLoopSignal, DoomLoopSignalKind, is_check_event, peek_doom_loop,
};
pub use self::error::{
    EmptyReason, EmptyResponseContext, ResponseModelMetadata, Result, SamplingError,
    is_context_length_error,
};
pub use self::kimi_code::{
    KIMI_CODE_ANTHROPIC_BETA, KIMI_CODE_ANTHROPIC_VERSION, KIMI_CODE_API_KEY_ENV,
    KIMI_CODE_AUTH_SCOPE, KIMI_CODE_BASE_URL, KIMI_CODE_CHAT_COMPLETIONS_URL, KIMI_CODE_FETCH_URL,
    KIMI_CODE_FILES_URL, KIMI_CODE_MAX_REQUEST_BYTES, KIMI_CODE_MESSAGES_URL,
    KIMI_CODE_MODEL_NAMESPACE, KIMI_CODE_MODELS_URL, KIMI_CODE_PROVIDER_NAME, KIMI_CODE_SEARCH_URL,
    KIMI_CODE_USAGE_URL,
};
pub use self::openai_codex::{
    OPENAI_CODEX_BASE_URL, OPENAI_CODEX_COMPATIBILITY_VERSION,
    OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY, OPENAI_CODEX_FAST_SERVICE_TIER,
    OPENAI_CODEX_RESPONSES_LITE_HEADER, OPENAI_CODEX_RESPONSES_URL,
    OPENAI_CODEX_SERVICE_TIER_METADATA_KEY, OPENAI_CODEX_STANDARD_SERVICE_TIER,
};
pub use self::provider::{
    CredentialBinding, CredentialSourceId, ProviderId, XAI_API_BASE_URL,
    XAI_CLI_CHAT_PROXY_BASE_URL, is_trusted_xai_inference_url,
};
pub use self::types::*;
pub use self::zai_coding_plan::*;

// Re-export async-openai crate Responses API types under `rs` namespace
pub use async_openai::types::responses as rs;
