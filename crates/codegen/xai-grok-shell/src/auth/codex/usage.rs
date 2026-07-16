//! Provider-scoped ChatGPT Codex subscription usage.
//!
//! This follows the public OpenAI Codex client's current ChatGPT backend
//! contract. It is intentionally isolated from xAI billing and authentication.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use super::{CodexAuthError, CodexAuthManager, OPENAI_CODEX_PROVIDER_ID, OPENAI_CODEX_USAGE_URL};

/// One server-advertised rate-limit window. Durations and reset timestamps are
/// kept exactly as returned so callers do not have to infer a five-hour or
/// weekly window from a stale model list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexUsageWindow {
    pub used_percent: f64,
    pub limit_window_seconds: i64,
    pub reset_after_seconds: i64,
    pub reset_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexRateLimit {
    pub allowed: bool,
    pub limit_reached: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_window: Option<CodexUsageWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_window: Option<CodexUsageWindow>,
}

/// Credit state exposed by `/wham/usage`. Approximate message projections in
/// the wire payload are deliberately not retained because their schema is
/// opaque and the official Codex client does not expose them as usage state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexCreditStatus {
    pub has_credits: bool,
    pub unlimited: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexSpendControlLimit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub limit: String,
    pub used: String,
    pub remaining: String,
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub reset_after_seconds: i64,
    pub reset_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexSpendControl {
    pub reached: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub individual_limit: Option<CodexSpendControlLimit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexAdditionalRateLimit {
    pub limit_name: String,
    pub metered_feature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<CodexRateLimit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexRateLimitReachedType {
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexResetCredits {
    pub available_count: i64,
}

/// Sanitized subset of the current public `/wham/usage` response.
///
/// The type contains no token, account identifier, or opaque reset-credit IDs
/// and is therefore safe to pass to the pager. It must still not be logged as a
/// raw upstream payload because balance and spend-control values are private.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexUsageSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<CodexRateLimit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credits: Option<CodexCreditStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spend_control: Option<CodexSpendControl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_rate_limits: Vec<CodexAdditionalRateLimit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_reached_type: Option<CodexRateLimitReachedType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_reset_credits: Option<CodexResetCredits>,
}

impl CodexUsageSnapshot {
    pub fn highest_used_percent(&self) -> Option<f64> {
        self.rate_limit
            .iter()
            .chain(
                self.additional_rate_limits
                    .iter()
                    .filter_map(|additional| additional.rate_limit.as_ref()),
            )
            .flat_map(|limit| {
                [
                    limit.primary_window.as_ref(),
                    limit.secondary_window.as_ref(),
                ]
                .into_iter()
                .flatten()
            })
            .map(|window| window.used_percent)
            .reduce(f64::max)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CodexUsageError {
    #[error(transparent)]
    Auth(#[from] CodexAuthError),
    #[error("OpenAI Codex usage authentication was invalid")]
    InvalidAuth,
    #[error("OpenAI Codex usage request returned HTTP {0}")]
    HttpStatus(u16),
    #[error("OpenAI Codex usage request failed")]
    Transport(#[source] reqwest::Error),
    #[error("OpenAI Codex usage response was invalid")]
    InvalidResponse,
}

#[async_trait]
trait UsageAuthProvider: Sync {
    async fn request_auth(&self) -> Result<xai_grok_tools::types::RequestAuth, CodexUsageError>;

    async fn recover_unauthorized(
        &self,
        rejected: xai_grok_tools::types::RequestCredentialSnapshot,
    ) -> Result<bool, CodexUsageError>;
}

#[async_trait]
impl UsageAuthProvider for CodexAuthManager {
    async fn request_auth(&self) -> Result<xai_grok_tools::types::RequestAuth, CodexUsageError> {
        Ok(CodexAuthManager::request_auth(self).await?)
    }

    async fn recover_unauthorized(
        &self,
        rejected: xai_grok_tools::types::RequestCredentialSnapshot,
    ) -> Result<bool, CodexUsageError> {
        let mut binding = xai_grok_sampling_types::CredentialBinding::openai_codex(Some(
            rejected.opaque_id().to_owned(),
        ));
        binding.generation = rejected.generation();
        Ok(self.recover_after_unauthorized(binding).await?)
    }
}

fn request_headers(
    auth: &xai_grok_tools::types::RequestAuth,
) -> Result<HeaderMap, CodexUsageError> {
    if auth.provider() != Some(OPENAI_CODEX_PROVIDER_ID) {
        return Err(CodexUsageError::InvalidAuth);
    }

    let mut headers = HeaderMap::new();
    let account_header = HeaderName::from_static("chatgpt-account-id");
    let fedramp_header = HeaderName::from_static("x-openai-fedramp");
    for (name, raw_value) in auth.headers() {
        let name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|_| CodexUsageError::InvalidAuth)?;
        if headers.contains_key(&name) {
            return Err(CodexUsageError::InvalidAuth);
        }

        if name == reqwest::header::AUTHORIZATION {
            if raw_value
                .strip_prefix("Bearer ")
                .is_none_or(|token| token.trim().is_empty())
            {
                return Err(CodexUsageError::InvalidAuth);
            }
            let mut value =
                HeaderValue::from_str(raw_value).map_err(|_| CodexUsageError::InvalidAuth)?;
            value.set_sensitive(true);
            headers.insert(name, value);
        } else if name == account_header {
            if raw_value.trim().is_empty() {
                return Err(CodexUsageError::InvalidAuth);
            }
            let mut value =
                HeaderValue::from_str(raw_value).map_err(|_| CodexUsageError::InvalidAuth)?;
            value.set_sensitive(true);
            headers.insert(name, value);
        } else if name == fedramp_header {
            if raw_value != "true" {
                return Err(CodexUsageError::InvalidAuth);
            }
            headers.insert(name, HeaderValue::from_static("true"));
        } else {
            // `RequestAuth` is a provider boundary. Never forward a newly
            // introduced or xAI-specific header to the ChatGPT backend until
            // this contract explicitly opts into it.
            return Err(CodexUsageError::InvalidAuth);
        }
    }
    if !headers.contains_key(reqwest::header::AUTHORIZATION)
        || !headers.contains_key(account_header)
    {
        return Err(CodexUsageError::InvalidAuth);
    }
    Ok(headers)
}

async fn fetch_from_url<A: UsageAuthProvider + ?Sized>(
    client: &reqwest::Client,
    auth_provider: &A,
    url: &str,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    for attempt in 0..=1 {
        let auth = auth_provider.request_auth().await?;
        let rejected = auth.credential_snapshot().cloned();
        let response = client
            .get(url)
            .headers(request_headers(&auth)?)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(CodexUsageError::Transport)?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED && attempt == 0 {
            let rejected = rejected.ok_or(CodexUsageError::InvalidAuth)?;
            if auth_provider.recover_unauthorized(rejected).await? {
                continue;
            }
        }

        if !response.status().is_success() {
            // Do not read or log the upstream body. Error payloads can echo
            // request metadata and are unnecessary for a status-only failure.
            return Err(CodexUsageError::HttpStatus(response.status().as_u16()));
        }

        return response
            .json::<CodexUsageSnapshot>()
            .await
            .map_err(|_| CodexUsageError::InvalidResponse);
    }

    unreachable!("the bounded Codex usage retry loop always returns")
}

/// Fetch the selected ChatGPT account's current Codex subscription usage.
/// A 401 is recovered once through the Codex credential manager only.
pub async fn fetch_codex_usage(
    manager: &CodexAuthManager,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    fetch_from_url(
        &crate::http::shared_client(),
        manager,
        OPENAI_CODEX_USAGE_URL,
    )
    .await
}

#[cfg(test)]
#[path = "usage_tests.rs"]
mod tests;
