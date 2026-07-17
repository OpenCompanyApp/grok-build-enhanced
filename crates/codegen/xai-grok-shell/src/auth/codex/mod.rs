//! Provider-scoped authentication for the experimental ChatGPT Codex backend.
//!
//! The endpoint and OAuth behavior here follow the public OpenAI Codex client
//! contract verified on 2026-07-16 at openai/codex commit
//! `f737605606c14e3aa59a4c17be80d338f164dff5`. They are not a stable,
//! general-purpose OpenAI API contract.

mod adapters;
mod credentials;
mod error;
mod flow;
mod manager;
mod oauth;
mod pricing;
mod request_auth;
mod storage;
mod usage;

pub use adapters::{
    CodexApiKeyProvider, CodexSamplerRequestAuth, shared_api_key_provider,
    shared_sampler_request_auth,
};
#[cfg(test)]
pub(crate) use credentials::credentials_for_test;
pub use credentials::{
    CODEX_CREDENTIAL_SCHEMA_VERSION, CodexCredentialProvider, CodexCredentials, SecretString,
};
pub use error::CodexAuthError;
pub use flow::{run_codex_cli_login, run_codex_cli_logout};
pub(crate) use manager::CodexCatalogIdentityLease;
pub use manager::{CodexAuthManager, CodexLogoutResult};
pub use oauth::{CodexDeviceAuthorization, CodexOAuthClient, PendingCodexBrowserLogin};
pub use pricing::{
    CodexApiEquivalentCostEstimate, CodexApiEquivalentCostLine, estimate_api_equivalent_cost,
};
pub use request_auth::{CodexAuthSnapshot, CodexAuthSnapshotError};
pub use storage::CodexCredentialStore;
pub use usage::{
    CodexAdditionalRateLimit, CodexCreditStatus, CodexRateLimit, CodexRateLimitReachedType,
    CodexResetCredits, CodexSpendControl, CodexSpendControlLimit, CodexUsageError,
    CodexUsageSnapshot, CodexUsageWindow, fetch_codex_usage, fetch_codex_usage_for_current,
    fetch_codex_usage_for_session,
};

pub use xai_grok_tools::types::OPENAI_CODEX_PROVIDER_ID;

/// Dedicated scope in Grok's provider-aware auth store.
pub const OPENAI_CODEX_SCOPE: &str = "openai::codex";
pub const OPENAI_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
pub const OPENAI_CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
pub const OPENAI_CODEX_ISSUER: &str = "https://auth.openai.com";
pub const OPENAI_CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const OPENAI_CODEX_ORIGINATOR: &str = "grok_build_codex";
pub const OPENAI_CODEX_SCOPES: &str =
    "openid profile email offline_access api.connectors.read api.connectors.invoke";
