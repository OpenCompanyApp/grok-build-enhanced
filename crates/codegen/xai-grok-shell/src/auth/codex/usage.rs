//! Provider-scoped ChatGPT Codex subscription usage.
//!
//! This follows the public OpenAI Codex client's current ChatGPT backend
//! contract. It is intentionally isolated from xAI billing and authentication.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use xai_grok_sampling_types::{CredentialBinding, CredentialSourceId, ProviderId};

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
    #[error("OpenAI Codex usage is temporarily unavailable")]
    TemporarilyUnavailable,
}

const USAGE_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(60);
const USAGE_BACKOFF_BASE: std::time::Duration = std::time::Duration::from_secs(60);
const USAGE_BACKOFF_MAX: std::time::Duration = std::time::Duration::from_secs(15 * 60);

#[derive(Clone)]
struct CachedUsage {
    /// Non-secret local credential-record binding retained only to prevent a
    /// cached account A snapshot from being served after switching to record B.
    /// This is deliberately a single slot, not an account-keyed cache.
    binding: CredentialBinding,
    fetched_at: std::time::Instant,
    snapshot: CodexUsageSnapshot,
}

#[derive(Default)]
struct CodexUsageCache {
    entry: Option<CachedUsage>,
    consecutive_failures: u32,
    retry_not_before: Option<std::time::Instant>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UsageFetchMode {
    /// Background refresh: obey backoff and use the last same-record snapshot
    /// while retrying. The cache never crosses credential-record boundaries.
    Silent,
    /// User-requested `/usage`: return a fresh cache hit, otherwise attempt the
    /// network immediately even if a background refresh is backing off.
    Explicit,
}

fn usage_cache() -> &'static tokio::sync::Mutex<CodexUsageCache> {
    static CACHE: std::sync::OnceLock<tokio::sync::Mutex<CodexUsageCache>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| tokio::sync::Mutex::new(CodexUsageCache::default()))
}

/// Dedicated credential-bearing client for the ChatGPT usage endpoint.
///
/// The process-wide shared client follows redirects. Reqwest strips the
/// standard Authorization header on a cross-origin redirect, but it cannot
/// know that ChatGPT-Account-ID and X-OpenAI-FedRAMP are equally sensitive.
/// Refusing redirects here keeps the complete provider-auth snapshot bound to
/// the one exact URL selected by this module.
fn codex_usage_http_client() -> reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(30))
                .user_agent(crate::http::process_user_agent_string())
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("failed to build OpenAI Codex usage HTTP client")
        })
        .clone()
}

fn validate_binding(binding: &CredentialBinding) -> Result<(), CodexUsageError> {
    if binding.provider != ProviderId::OpenAiCodex
        || binding.source != CredentialSourceId::OpenAiCodexSubscription
        || binding.generation == 0
        || binding
            .record_id
            .as_deref()
            .is_none_or(|record_id| record_id.trim().is_empty())
    {
        return Err(CodexAuthError::AccountChanged.into());
    }
    Ok(())
}

fn validate_auth_binding(
    auth: &xai_grok_tools::types::RequestAuth,
    expected: &CredentialBinding,
) -> Result<(), CodexUsageError> {
    validate_binding(expected)?;
    let snapshot = auth
        .credential_snapshot()
        .ok_or(CodexUsageError::InvalidAuth)?;
    let mut actual = CredentialBinding::openai_codex(Some(snapshot.opaque_id().to_owned()));
    actual.generation = snapshot.generation();
    validate_binding(&actual)?;
    if !expected.same_record(&actual) || actual.generation < expected.generation {
        return Err(CodexAuthError::AccountChanged.into());
    }
    Ok(())
}

fn backoff_for_failures(failures: u32) -> std::time::Duration {
    let shift = failures.saturating_sub(1).min(8);
    USAGE_BACKOFF_BASE
        .saturating_mul(1u32 << shift)
        .min(USAGE_BACKOFF_MAX)
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
            let mut value = HeaderValue::from_static("true");
            value.set_sensitive(true);
            headers.insert(name, value);
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

#[cfg(test)]
async fn fetch_from_url<A: UsageAuthProvider + ?Sized>(
    client: &reqwest::Client,
    auth_provider: &A,
    url: &str,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    fetch_from_url_with_binding(client, auth_provider, url, None).await
}

async fn fetch_from_url_with_binding<A: UsageAuthProvider + ?Sized>(
    client: &reqwest::Client,
    auth_provider: &A,
    url: &str,
    expected: Option<&CredentialBinding>,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    for attempt in 0..=1 {
        let auth = auth_provider.request_auth().await?;
        if let Some(expected) = expected {
            validate_auth_binding(&auth, expected)?;
        }
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

async fn fetch_cached_from_url<A: UsageAuthProvider + ?Sized>(
    client: &reqwest::Client,
    auth_provider: &A,
    url: &str,
    expected: &CredentialBinding,
    mode: UsageFetchMode,
    cache: &tokio::sync::Mutex<CodexUsageCache>,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    validate_binding(expected)?;
    let now = std::time::Instant::now();
    let mut cache = cache.lock().await;

    if cache
        .entry
        .as_ref()
        .is_some_and(|entry| !entry.binding.same_record(expected))
    {
        // Account switching replaces the one cache slot. No account identity
        // or credential material is used as a map key or emitted to logs.
        *cache = CodexUsageCache::default();
    }

    if let Some(entry) = cache.entry.as_ref()
        && now.saturating_duration_since(entry.fetched_at) <= USAGE_CACHE_TTL
    {
        return Ok(entry.snapshot.clone());
    }

    if mode == UsageFetchMode::Silent
        && cache
            .retry_not_before
            .is_some_and(|retry_at| now < retry_at)
    {
        return cache
            .entry
            .as_ref()
            .map(|entry| entry.snapshot.clone())
            .ok_or(CodexUsageError::TemporarilyUnavailable);
    }

    match fetch_from_url_with_binding(client, auth_provider, url, Some(expected)).await {
        Ok(snapshot) => {
            cache.entry = Some(CachedUsage {
                binding: expected.clone(),
                fetched_at: std::time::Instant::now(),
                snapshot: snapshot.clone(),
            });
            cache.consecutive_failures = 0;
            cache.retry_not_before = None;
            Ok(snapshot)
        }
        Err(error) => {
            cache.consecutive_failures = cache.consecutive_failures.saturating_add(1);
            let backoff = backoff_for_failures(cache.consecutive_failures);
            cache.retry_not_before = Some(std::time::Instant::now() + backoff);
            if mode == UsageFetchMode::Silent
                && let Some(entry) = cache.entry.as_ref()
            {
                return Ok(entry.snapshot.clone());
            }
            Err(error)
        }
    }
}

fn snapshot_current_usage_binding(
    manager: &CodexAuthManager,
) -> Result<CredentialBinding, CodexUsageError> {
    let binding = manager.current_verified()?.credential_binding();
    validate_binding(&binding)?;
    Ok(binding)
}

async fn fetch_codex_usage_for_current_from_url(
    client: &reqwest::Client,
    manager: &CodexAuthManager,
    url: &str,
    silent: bool,
    cache: &tokio::sync::Mutex<CodexUsageCache>,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    // Snapshot the verified provider-scoped record before the first await.
    // The cache and every outbound request remain pinned to this record even
    // if another process logs out or changes accounts while this request runs.
    let expected = snapshot_current_usage_binding(manager)?;
    fetch_cached_from_url(
        client,
        manager,
        url,
        &expected,
        if silent {
            UsageFetchMode::Silent
        } else {
            UsageFetchMode::Explicit
        },
        cache,
    )
    .await
}

/// Fetch the selected ChatGPT account's current Codex subscription usage.
/// A 401 is recovered once through the Codex credential manager only. This
/// compatibility entry point is an explicit, account-bound refresh.
pub async fn fetch_codex_usage(
    manager: &CodexAuthManager,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    fetch_codex_usage_for_current(manager, false).await
}

/// Fetch usage before an ACP session exists by pinning the credential record
/// selected at request start. This uses the same account-safe cache, refresh,
/// and retry path as active-session usage and never consults xAI auth.
pub async fn fetch_codex_usage_for_current(
    manager: &CodexAuthManager,
    silent: bool,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    let client = codex_usage_http_client();
    fetch_codex_usage_for_current_from_url(
        &client,
        manager,
        OPENAI_CODEX_USAGE_URL,
        silent,
        usage_cache(),
    )
    .await
}

/// Fetch usage for the exact credential record pinned to an active session.
/// Same-record refresh generations may advance, but a logout/relogin or account
/// switch fails before any usage HTTP request is sent. Silent callers use a
/// one-minute cache plus 1m/2m/4m/.../15m capped retry backoff; explicit callers
/// use a fresh cache hit but otherwise attempt an immediate refresh.
pub async fn fetch_codex_usage_for_session(
    manager: &CodexAuthManager,
    expected: &CredentialBinding,
    silent: bool,
) -> Result<CodexUsageSnapshot, CodexUsageError> {
    validate_binding(expected)?;
    let current = manager.current().ok_or(CodexAuthError::NotLoggedIn)?;
    let actual = current.credential_binding();
    if !expected.same_record(&actual) || actual.generation < expected.generation {
        return Err(CodexAuthError::AccountChanged.into());
    }
    let client = codex_usage_http_client();
    fetch_cached_from_url(
        &client,
        manager,
        OPENAI_CODEX_USAGE_URL,
        expected,
        if silent {
            UsageFetchMode::Silent
        } else {
            UsageFetchMode::Explicit
        },
        usage_cache(),
    )
    .await
}

#[cfg(test)]
#[path = "usage_tests.rs"]
mod tests;
