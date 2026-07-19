//! OpenAI Codex subscription protocol constants shared across the agent stack.
//!
//! These values describe the experimental ChatGPT Codex transport contract.
//! Provider and credential-source identity remain in [`crate::provider`].

/// Base URL used by the current public OpenAI Codex client for ChatGPT
/// subscription traffic.
///
/// This is an experimental client backend contract, not the stable OpenAI API.
pub const OPENAI_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";

/// Full Responses endpoint derived from [`OPENAI_CODEX_BASE_URL`].
pub const OPENAI_CODEX_RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex/responses";

/// Catalog service-tier id used by the current public Codex client for Fast
/// mode. The user-facing config alias is `fast`; the Responses wire value is
/// `priority`.
pub const OPENAI_CODEX_FAST_SERVICE_TIER: &str = "priority";

/// Internal selection sentinel for explicit Standard mode. This value is
/// persisted with session/config state but is never sent to Responses.
pub const OPENAI_CODEX_STANDARD_SERVICE_TIER: &str = "default";

/// ACP model-metadata key carrying the effective Codex service tier into UI
/// clients. The value is session/config selection state, not catalog-default
/// capability metadata.
pub const OPENAI_CODEX_SERVICE_TIER_METADATA_KEY: &str = "serviceTier";

/// Internal compatibility marker used by the public Codex client for models
/// whose authenticated catalog entry enables the Responses Lite contract.
pub const OPENAI_CODEX_RESPONSES_LITE_HEADER: &str = "x-openai-internal-codex-responses-lite";

/// Internal typed-compatibility side channel. The sampler stores a Codex
/// response's raw `max`/`ultra` effort here while deserializing through an
/// async-openai version whose enum stops at `xhigh`; conversation conversion
/// consumes it instead of recording a false `xhigh` provenance value.
pub const OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY: &str =
    "__grok_codex_reasoning_effort";

pub use xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responses_endpoint_matches_current_public_client_contract() {
        assert_eq!(
            format!("{OPENAI_CODEX_BASE_URL}/responses"),
            OPENAI_CODEX_RESPONSES_URL
        );
    }
}
