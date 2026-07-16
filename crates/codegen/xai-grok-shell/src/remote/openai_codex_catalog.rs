//! Authenticated model discovery for the ChatGPT Codex backend.
//!
//! This module deliberately owns neither login nor token refresh. Callers inject a
//! fresh, account-bound authentication snapshot for each operation. That keeps
//! catalog requests provider-scoped and prevents Codex authentication failures
//! from falling through to the xAI authentication machinery.

use std::collections::HashSet;
use std::fmt;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use blake3::Hasher;
use futures_util::StreamExt;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, ETAG, HeaderName, HeaderValue, IF_NONE_MATCH, USER_AGENT,
};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;
use xai_grok_sampling_types::{CredentialBinding, CredentialSourceId, ProviderId};

pub const OPENAI_CODEX_PROVIDER_ID: &str = "openai_codex";
pub const OPENAI_CODEX_MODEL_NAMESPACE: &str = "openai-codex";
pub const OPENAI_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
pub const OPENAI_CODEX_CATALOG_TIMEOUT: Duration = Duration::from_secs(5);
pub const OPENAI_CODEX_CATALOG_CACHE_TTL: Duration = Duration::from_secs(300);

const CACHE_SCHEMA_VERSION: u32 = 2;
const DEFAULT_MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_CACHE_BYTES: u64 = 16 * 1024 * 1024;
const CHATGPT_ACCOUNT_ID: HeaderName = HeaderName::from_static("chatgpt-account-id");
const OPENAI_FEDRAMP: HeaderName = HeaderName::from_static("x-openai-fedramp");
const ORIGINATOR: HeaderName = HeaderName::from_static("originator");
const CODEX_VERSION: HeaderName = HeaderName::from_static("version");

/// A failure while obtaining request authentication.
///
/// This error intentionally cannot carry provider error strings: token and
/// account material must not become printable through an error chain.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum CodexCatalogAuthError {
    #[error("OpenAI Codex authentication is unavailable")]
    Unavailable,
    #[error("OpenAI Codex authentication is temporarily unavailable")]
    Transient,
}

fn map_auth_error(error: CodexCatalogAuthError) -> CodexCatalogError {
    match error {
        CodexCatalogAuthError::Unavailable => CodexCatalogError::AuthenticationUnavailable,
        CodexCatalogAuthError::Transient => CodexCatalogError::AuthenticationTransient,
    }
}

/// Supplies a current provider-scoped authentication snapshot.
///
/// Implementations should reload or refresh credentials before returning the
/// snapshot when necessary. The catalog client asks once per operation and does
/// not retain an authentication provider.
#[async_trait]
pub trait CodexCatalogAuthProvider: Send + Sync {
    async fn catalog_auth(&self) -> Result<CodexCatalogAuthSnapshot, CodexCatalogAuthError>;

    /// Reload and, if appropriate, refresh this provider's Codex credentials
    /// after a rejected request. `rejected` is the exact non-secret record and
    /// generation that signed that request; implementations must not recover
    /// against whichever credential happens to be current when the 401 arrives.
    ///
    /// Returning `true` asks the catalog client to obtain a new snapshot and
    /// retry exactly once. The default is deliberately inert so a static
    /// snapshot can never turn a Codex 401 into an xAI authentication flow.
    async fn recover_unauthorized(
        &self,
        _rejected: CredentialBinding,
    ) -> Result<bool, CodexCatalogAuthError> {
        Ok(false)
    }
}

/// Sensitive request headers plus a non-secret, already-hashed cache identity.
///
/// Constructing a snapshot validates the header values and immediately hashes
/// the opaque credential identity together with the ChatGPT account ID. Raw
/// account and credential identifiers are never persisted in the model cache.
#[derive(Clone)]
pub struct CodexCatalogAuthSnapshot {
    authorization: HeaderValue,
    account_id: HeaderValue,
    fedramp: bool,
    cache_identity: [u8; 32],
    credential_binding: CredentialBinding,
}

impl CodexCatalogAuthSnapshot {
    pub fn new(
        access_token: &str,
        chatgpt_account_id: &str,
        chatgpt_user_id: Option<&str>,
        credential_binding: CredentialBinding,
        fedramp: bool,
    ) -> Result<Self, CodexCatalogAuthError> {
        let credential_id = credential_binding
            .record_id
            .as_deref()
            .filter(|record_id| !record_id.trim().is_empty());
        if access_token.trim().is_empty()
            || chatgpt_account_id.trim().is_empty()
            || credential_binding.provider != ProviderId::OpenAiCodex
            || credential_binding.source != CredentialSourceId::OpenAiCodexSubscription
            || credential_binding.generation == 0
            || credential_id.is_none()
        {
            return Err(CodexCatalogAuthError::Unavailable);
        }

        let mut authorization = HeaderValue::from_str(&format!("Bearer {access_token}"))
            .map_err(|_| CodexCatalogAuthError::Unavailable)?;
        authorization.set_sensitive(true);
        let mut account_id = HeaderValue::from_str(chatgpt_account_id)
            .map_err(|_| CodexCatalogAuthError::Unavailable)?;
        account_id.set_sensitive(true);

        let mut scope = Hasher::new_derive_key("grok-build-codex catalog credential scope v1");
        hash_len_prefixed(&mut scope, credential_id.unwrap_or_default().as_bytes());
        hash_len_prefixed(&mut scope, chatgpt_account_id.as_bytes());
        hash_len_prefixed(&mut scope, chatgpt_user_id.unwrap_or_default().as_bytes());
        scope.update(&[u8::from(fedramp)]);

        Ok(Self {
            authorization,
            account_id,
            fedramp,
            cache_identity: *scope.finalize().as_bytes(),
            credential_binding,
        })
    }
}

impl fmt::Debug for CodexCatalogAuthSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexCatalogAuthSnapshot")
            .field("authorization", &"<redacted>")
            .field("account_id", &"<redacted>")
            .field("fedramp", &self.fedramp)
            .field("cache_identity", &"<redacted>")
            .field("credential_binding", &self.credential_binding)
            .finish()
    }
}

#[async_trait]
impl CodexCatalogAuthProvider for CodexCatalogAuthSnapshot {
    async fn catalog_auth(&self) -> Result<CodexCatalogAuthSnapshot, CodexCatalogAuthError> {
        Ok(self.clone())
    }
}

/// Configuration for [`CodexCatalogClient`].
#[derive(Clone, Debug)]
pub struct CodexCatalogClientConfig {
    base_url: Url,
    cache_dir: PathBuf,
    client_version: String,
    timeout: Duration,
    cache_ttl: Duration,
    max_response_bytes: usize,
    user_agent: HeaderValue,
    originator: HeaderValue,
}

impl CodexCatalogClientConfig {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Result<Self, CodexCatalogError> {
        let base_url = Url::parse(OPENAI_CODEX_BASE_URL)
            .map_err(|_| CodexCatalogError::InvalidConfiguration)?;
        let user_agent =
            HeaderValue::from_str(&format!("grok-build-codex/{}", xai_grok_version::VERSION))
                .map_err(|_| CodexCatalogError::InvalidConfiguration)?;

        Ok(Self {
            base_url,
            cache_dir: cache_dir.into(),
            client_version: xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION.to_string(),
            timeout: OPENAI_CODEX_CATALOG_TIMEOUT,
            cache_ttl: OPENAI_CODEX_CATALOG_CACHE_TTL,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            user_agent,
            originator: HeaderValue::from_static("grok_build_codex"),
        })
    }

    pub fn with_base_url(mut self, base_url: Url) -> Result<Self, CodexCatalogError> {
        if !is_allowed_base_url(&base_url) {
            return Err(CodexCatalogError::InvalidConfiguration);
        }
        self.base_url = base_url;
        Ok(self)
    }

    pub fn with_client_version(
        mut self,
        client_version: impl Into<String>,
    ) -> Result<Self, CodexCatalogError> {
        let client_version = client_version.into();
        if client_version.trim().is_empty() || HeaderValue::from_str(&client_version).is_err() {
            return Err(CodexCatalogError::InvalidConfiguration);
        }
        self.client_version = client_version;
        Ok(self)
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, CodexCatalogError> {
        if timeout.is_zero() {
            return Err(CodexCatalogError::InvalidConfiguration);
        }
        self.timeout = timeout;
        Ok(self)
    }

    pub fn with_max_response_bytes(
        mut self,
        max_response_bytes: usize,
    ) -> Result<Self, CodexCatalogError> {
        if max_response_bytes == 0 {
            return Err(CodexCatalogError::InvalidConfiguration);
        }
        self.max_response_bytes = max_response_bytes;
        Ok(self)
    }

    pub fn with_client_headers(
        mut self,
        user_agent: &str,
        originator: &str,
    ) -> Result<Self, CodexCatalogError> {
        self.user_agent = HeaderValue::from_str(user_agent)
            .map_err(|_| CodexCatalogError::InvalidConfiguration)?;
        self.originator = HeaderValue::from_str(originator)
            .map_err(|_| CodexCatalogError::InvalidConfiguration)?;
        Ok(self)
    }
}

/// Authenticated ChatGPT Codex model catalog client.
#[derive(Clone)]
pub struct CodexCatalogClient {
    http: reqwest::Client,
    config: CodexCatalogClientConfig,
}

impl fmt::Debug for CodexCatalogClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexCatalogClient")
            .field("base_url", &self.config.base_url)
            .field("cache_dir", &self.config.cache_dir)
            .field("client_version", &self.config.client_version)
            .field("timeout", &self.config.timeout)
            .field("cache_ttl", &self.config.cache_ttl)
            .field("max_response_bytes", &self.config.max_response_bytes)
            .finish_non_exhaustive()
    }
}

impl CodexCatalogClient {
    pub fn new(config: CodexCatalogClientConfig) -> Result<Self, CodexCatalogError> {
        // Auth and account headers must never be replayed to a redirect target.
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| CodexCatalogError::InvalidConfiguration)?;
        Ok(Self { http, config })
    }

    /// Fetch the authoritative catalog for the authenticated account.
    ///
    /// A successful `200` response containing zero models is authoritative and
    /// replaces any prior non-empty cache. Network and authentication failures
    /// never silently return stale models; callers may opt into [`Self::load_cached`]
    /// and label that result as cached/offline in their UI.
    pub async fn fetch(
        &self,
        auth_provider: &(dyn CodexCatalogAuthProvider + Send + Sync),
    ) -> Result<CodexCatalogFetch, CodexCatalogError> {
        let mut auth = auth_provider.catalog_auth().await.map_err(map_auth_error)?;
        match self.fetch_with_auth(&auth).await {
            Err(CodexCatalogError::Unauthorized) => {
                let rejected = auth.credential_binding.clone();
                let recovered = auth_provider
                    .recover_unauthorized(rejected)
                    .await
                    .map_err(map_auth_error)?;
                if !recovered {
                    return Err(CodexCatalogError::Unauthorized);
                }
                auth = auth_provider.catalog_auth().await.map_err(map_auth_error)?;
                // One recovery attempt is the hard limit. A second 401 is
                // surfaced without invoking recovery again.
                self.fetch_with_auth(&auth).await
            }
            result => result,
        }
    }

    pub async fn load_cached(
        &self,
        auth_provider: &(dyn CodexCatalogAuthProvider + Send + Sync),
    ) -> Result<CodexCachedCatalog, CodexCatalogError> {
        let auth = auth_provider.catalog_auth().await.map_err(map_auth_error)?;
        self.load_cached_with_auth(&auth)
    }

    /// Renew the account-scoped cache TTL after an authenticated response
    /// confirms the already-installed catalog ETag. Stale content is eligible
    /// because that matching live response is the freshness proof; this method
    /// never returns catalog data to the caller.
    pub async fn renew_cached(
        &self,
        auth_provider: &(dyn CodexCatalogAuthProvider + Send + Sync),
        confirmed_etag: &str,
    ) -> Result<CredentialBinding, CodexCatalogError> {
        let auth = auth_provider.catalog_auth().await.map_err(map_auth_error)?;
        let scope = self.cache_scope(&auth);
        let mut cached = self
            .read_cache(&scope)
            .map_err(|_| CodexCatalogError::CacheUnavailable)?
            .ok_or(CodexCatalogError::CacheUnavailable)?;
        if cached.etag.as_deref() != Some(confirmed_etag) {
            return Err(CodexCatalogError::CacheUnavailable);
        }
        cached.fetched_at_unix_ms = now_unix_ms();
        cached.credential_generation = auth.credential_binding.generation;
        self.write_cache(&scope, &cached)
            .map_err(|_| CodexCatalogError::CacheUnavailable)?;
        Ok(auth.credential_binding)
    }

    fn load_cached_with_auth(
        &self,
        auth: &CodexCatalogAuthSnapshot,
    ) -> Result<CodexCachedCatalog, CodexCatalogError> {
        let scope = self.cache_scope(auth);
        let cached = self
            .read_cache(&scope)
            .map_err(|_| CodexCatalogError::CacheUnavailable)?
            .ok_or(CodexCatalogError::CacheUnavailable)?;
        if !self.cache_is_fresh(&cached)
            || cached.credential_generation != auth.credential_binding.generation
        {
            return Err(CodexCatalogError::CacheUnavailable);
        }
        Ok(CodexCachedCatalog {
            catalog: cached.catalog,
            fetched_at_unix_ms: cached.fetched_at_unix_ms,
            etag: cached.etag,
            credential_binding: auth.credential_binding.clone(),
        })
    }

    async fn fetch_with_auth(
        &self,
        auth: &CodexCatalogAuthSnapshot,
    ) -> Result<CodexCatalogFetch, CodexCatalogError> {
        let scope = self.cache_scope(auth);
        // A malformed or partially-written cache is a miss for an online fetch.
        // A stale cache cannot be returned by `load_cached`, but its ETag remains
        // useful as a validator. Only an authenticated 304 response may make
        // that catalog fresh again.
        let cached = self.read_cache(&scope).ok().flatten();
        let request_url = self.models_url()?;
        let client_version = HeaderValue::from_str(&self.config.client_version)
            .map_err(|_| CodexCatalogError::InvalidConfiguration)?;
        let mut request = self
            .http
            .get(request_url)
            .timeout(self.config.timeout)
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .header(AUTHORIZATION, auth.authorization.clone())
            .header(CHATGPT_ACCOUNT_ID, auth.account_id.clone())
            .header(USER_AGENT, self.config.user_agent.clone())
            .header(ORIGINATOR, self.config.originator.clone())
            .header(CODEX_VERSION, client_version);
        if auth.fedramp {
            request = request.header(OPENAI_FEDRAMP, HeaderValue::from_static("true"));
        }
        if let Some(etag) = cached.as_ref().and_then(|cached| cached.etag.as_deref())
            && let Ok(value) = HeaderValue::from_str(etag)
        {
            request = request.header(IF_NONE_MATCH, value);
        }

        let response = request.send().await.map_err(map_reqwest_error)?;
        let status = response.status();
        if status == StatusCode::NOT_MODIFIED {
            let mut cached = cached.ok_or(CodexCatalogError::NotModifiedWithoutCache)?;
            cached.fetched_at_unix_ms = now_unix_ms();
            // The live authenticated 304 is the only operation allowed to
            // adopt cache bytes produced by an older same-account credential
            // generation. It attests that the ETag is still authoritative.
            cached.credential_generation = auth.credential_binding.generation;
            let cache_status = match self.write_cache(&scope, &cached) {
                Ok(()) => CodexCatalogCacheStatus::Updated,
                Err(_) => CodexCatalogCacheStatus::WriteFailed,
            };
            return Ok(CodexCatalogFetch {
                catalog: cached.catalog,
                fetched_at_unix_ms: cached.fetched_at_unix_ms,
                etag: cached.etag,
                source: CodexCatalogFetchSource::NotModified,
                cache_status,
                credential_binding: auth.credential_binding.clone(),
            });
        }
        if status == StatusCode::UNAUTHORIZED {
            return Err(CodexCatalogError::Unauthorized);
        }
        if !status.is_success() {
            return Err(CodexCatalogError::HttpStatus(status.as_u16()));
        }

        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let body = read_limited_body(response, self.config.max_response_bytes).await?;
        let wire: ModelsResponse =
            serde_json::from_slice(&body).map_err(|_| CodexCatalogError::InvalidResponse)?;
        let catalog = CodexModelCatalog::from_wire(wire.models)?;
        let fetched_at_unix_ms = now_unix_ms();
        let envelope = CacheEnvelope {
            schema_version: CACHE_SCHEMA_VERSION,
            scope: scope.digest.clone(),
            fetched_at_unix_ms,
            credential_generation: auth.credential_binding.generation,
            etag: etag.clone(),
            catalog: catalog.clone(),
        };
        let cache_status = match self.write_cache(&scope, &envelope) {
            Ok(()) => CodexCatalogCacheStatus::Updated,
            Err(_) => CodexCatalogCacheStatus::WriteFailed,
        };

        Ok(CodexCatalogFetch {
            catalog,
            fetched_at_unix_ms,
            etag,
            source: CodexCatalogFetchSource::Network,
            cache_status,
            credential_binding: auth.credential_binding.clone(),
        })
    }

    fn models_url(&self) -> Result<Url, CodexCatalogError> {
        let mut url = self.config.base_url.clone();
        let path = format!("{}/models", url.path().trim_end_matches('/'));
        url.set_path(&path);
        url.query_pairs_mut()
            .append_pair("client_version", &self.config.client_version);
        Ok(url)
    }

    fn cache_scope(&self, auth: &CodexCatalogAuthSnapshot) -> CacheScope {
        let mut hasher = Hasher::new_derive_key("grok-build-codex model catalog cache scope v1");
        hash_len_prefixed(&mut hasher, OPENAI_CODEX_PROVIDER_ID.as_bytes());
        hash_len_prefixed(&mut hasher, self.config.base_url.as_str().as_bytes());
        hash_len_prefixed(&mut hasher, self.config.client_version.as_bytes());
        hash_len_prefixed(&mut hasher, &auth.cache_identity);
        let digest = hasher.finalize().to_hex().to_string();
        let path = self
            .config
            .cache_dir
            .join(format!("openai-codex-models-{digest}.json"));
        CacheScope { digest, path }
    }

    fn read_cache(&self, scope: &CacheScope) -> io::Result<Option<CacheEnvelope>> {
        let metadata = match std::fs::metadata(&scope.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if metadata.len() > MAX_CACHE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "catalog cache exceeds size limit",
            ));
        }
        let bytes = std::fs::read(&scope.path)?;
        let mut envelope: CacheEnvelope = serde_json::from_slice(&bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid catalog cache"))?;
        if envelope.schema_version != CACHE_SCHEMA_VERSION || envelope.scope != scope.digest {
            return Ok(None);
        }
        envelope.catalog.normalize_and_validate().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid cached model catalog")
        })?;
        Ok(Some(envelope))
    }

    fn cache_is_fresh(&self, cached: &CacheEnvelope) -> bool {
        if self.config.cache_ttl.is_zero() {
            return false;
        }

        let age_ms = now_unix_ms().saturating_sub(cached.fetched_at_unix_ms);
        u128::from(age_ms) <= self.config.cache_ttl.as_millis()
    }

    fn write_cache(&self, scope: &CacheScope, envelope: &CacheEnvelope) -> io::Result<()> {
        std::fs::create_dir_all(&self.config.cache_dir)?;
        let body = serde_json::to_vec(envelope)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid catalog cache"))?;
        if body.len() as u64 > MAX_CACHE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "catalog cache exceeds size limit",
            ));
        }

        // A unique owner-only temp file in the destination directory plus
        // fsync and persist gives crash-safe atomic publication.
        let mut temporary = tempfile::NamedTempFile::new_in(&self.config.cache_dir)?;
        temporary.write_all(&body)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&scope.path)
            .map_err(|error| error.error)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CodexCatalogError {
    #[error("invalid OpenAI Codex catalog configuration")]
    InvalidConfiguration,
    #[error("OpenAI Codex authentication is unavailable")]
    AuthenticationUnavailable,
    #[error("OpenAI Codex authentication is temporarily unavailable")]
    AuthenticationTransient,
    #[error("OpenAI Codex authentication was rejected")]
    Unauthorized,
    #[error("OpenAI Codex catalog request timed out")]
    Timeout,
    #[error("OpenAI Codex catalog request failed")]
    Transport,
    #[error("OpenAI Codex catalog request returned HTTP {0}")]
    HttpStatus(u16),
    #[error("OpenAI Codex catalog response exceeded the size limit")]
    ResponseTooLarge,
    #[error("OpenAI Codex catalog response was invalid")]
    InvalidResponse,
    #[error("OpenAI Codex catalog contained invalid model metadata")]
    InvalidModel,
    #[error("OpenAI Codex catalog returned not-modified without a matching cache")]
    NotModifiedWithoutCache,
    #[error("OpenAI Codex catalog cache is unavailable")]
    CacheUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodexCatalogFetchSource {
    Network,
    NotModified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodexCatalogCacheStatus {
    Updated,
    WriteFailed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CodexCatalogFetch {
    pub catalog: CodexModelCatalog,
    pub fetched_at_unix_ms: u64,
    pub etag: Option<String>,
    pub source: CodexCatalogFetchSource,
    pub cache_status: CodexCatalogCacheStatus,
    /// Exact non-secret credential generation that authenticated this result.
    pub credential_binding: CredentialBinding,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CodexCachedCatalog {
    pub catalog: CodexModelCatalog,
    pub fetched_at_unix_ms: u64,
    pub etag: Option<String>,
    /// Exact non-secret credential generation that selected this cache scope.
    pub credential_binding: CredentialBinding,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CodexModelCatalog {
    pub models: Vec<CodexCatalogModel>,
}

impl CodexModelCatalog {
    fn from_wire(models: Vec<CodexCatalogModel>) -> Result<Self, CodexCatalogError> {
        let mut catalog = Self { models };
        catalog.normalize_and_validate()?;
        Ok(catalog)
    }

    fn normalize_and_validate(&mut self) -> Result<(), CodexCatalogError> {
        let mut slugs = HashSet::with_capacity(self.models.len());
        for model in &mut self.models {
            if model.slug.trim().is_empty()
                || model.slug.chars().any(char::is_control)
                || !slugs.insert(model.slug.clone())
            {
                return Err(CodexCatalogError::InvalidModel);
            }
            model.id = format!("{OPENAI_CODEX_MODEL_NAMESPACE}/{}", model.slug);
        }
        // Slice sorting is stable, preserving the service's relative order for
        // models with the same priority while matching Codex's ascending
        // priority picker order.
        self.models.sort_by_key(|model| model.priority);
        Ok(())
    }
}

/// Model metadata returned by the current ChatGPT Codex `/models` endpoint.
///
/// Extensible enum-like fields are strings and unknown fields are retained so a
/// newer backend does not make discovery fail merely because this fork has not
/// learned how to interpret a newly-added capability yet.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CodexCatalogModel {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub slug: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default_reasoning_level: Option<String>,
    #[serde(default)]
    pub supported_reasoning_levels: Vec<CodexReasoningEffortPreset>,
    #[serde(default)]
    pub shell_type: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub supported_in_api: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub additional_speed_tiers: Vec<String>,
    #[serde(default)]
    pub service_tiers: Vec<CodexModelServiceTier>,
    #[serde(default)]
    pub default_service_tier: Option<String>,
    #[serde(default)]
    pub availability_nux: Option<CodexModelAvailabilityNux>,
    #[serde(default)]
    pub upgrade: Option<CodexModelUpgrade>,
    #[serde(default)]
    pub base_instructions: String,
    #[serde(default)]
    pub model_messages: Option<Value>,
    #[serde(default)]
    pub include_skills_usage_instructions: bool,
    #[serde(default = "default_true")]
    pub supports_reasoning_summary_parameter: bool,
    #[serde(default)]
    pub default_reasoning_summary: Option<String>,
    #[serde(default)]
    pub support_verbosity: bool,
    #[serde(default)]
    pub default_verbosity: Option<String>,
    #[serde(default)]
    pub apply_patch_tool_type: Option<String>,
    #[serde(default)]
    pub web_search_tool_type: Option<String>,
    #[serde(default)]
    pub truncation_policy: Option<CodexTruncationPolicy>,
    #[serde(default)]
    pub supports_parallel_tool_calls: bool,
    #[serde(default)]
    pub supports_image_detail_original: bool,
    #[serde(default)]
    pub context_window: Option<i64>,
    #[serde(default)]
    pub max_context_window: Option<i64>,
    #[serde(default)]
    pub auto_compact_token_limit: Option<i64>,
    #[serde(default)]
    pub comp_hash: Option<String>,
    #[serde(default = "default_effective_context_window_percent")]
    pub effective_context_window_percent: i64,
    #[serde(default)]
    pub experimental_supported_tools: Vec<String>,
    #[serde(default = "default_input_modalities")]
    pub input_modalities: Vec<String>,
    #[serde(default)]
    pub supports_search_tool: bool,
    #[serde(default)]
    pub use_responses_lite: bool,
    #[serde(default)]
    pub auto_review_model_override: Option<String>,
    #[serde(default)]
    pub tool_mode: Option<Value>,
    #[serde(default)]
    pub multi_agent_version: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl CodexCatalogModel {
    pub fn visible_in_picker(&self) -> bool {
        self.visibility.as_deref() == Some("list")
    }

    pub fn accepts_images(&self) -> bool {
        self.input_modalities
            .iter()
            .any(|modality| modality == "image")
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CodexReasoningEffortPreset {
    pub effort: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CodexModelServiceTier {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CodexModelAvailabilityNux {
    #[serde(default)]
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CodexModelUpgrade {
    pub model: String,
    #[serde(default)]
    pub migration_markdown: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CodexTruncationPolicy {
    pub mode: String,
    pub limit: i64,
}

#[derive(Deserialize)]
struct ModelsResponse {
    models: Vec<CodexCatalogModel>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CacheEnvelope {
    schema_version: u32,
    scope: String,
    fetched_at_unix_ms: u64,
    /// Non-secret monotonic producer generation. A newer credential may use
    /// the ETag as a validator, but cannot relabel stale bytes as its own
    /// offline fallback without a live authenticated 304/200 response.
    credential_generation: u64,
    etag: Option<String>,
    catalog: CodexModelCatalog,
}

#[derive(Debug)]
struct CacheScope {
    digest: String,
    path: PathBuf,
}

fn default_true() -> bool {
    true
}

fn default_effective_context_window_percent() -> i64 {
    95
}

fn default_input_modalities() -> Vec<String> {
    vec!["text".to_string(), "image".to_string()]
}

fn is_allowed_base_url(base_url: &Url) -> bool {
    let official = Url::parse(OPENAI_CODEX_BASE_URL).expect("static Codex base URL is valid");
    if base_url == &official {
        return true;
    }

    // Mock endpoints are a unit-test-only seam. Shipping builds cannot direct
    // bearer/account headers to an arbitrary host, even if a caller constructs
    // a custom config.
    #[cfg(test)]
    {
        return base_url.scheme() == "http"
            && base_url
                .host_str()
                .and_then(|host| host.parse::<std::net::IpAddr>().ok())
                .is_some_and(|address| address.is_loopback())
            && base_url.username().is_empty()
            && base_url.password().is_none()
            && base_url.query().is_none()
            && base_url.fragment().is_none();
    }

    #[cfg(not(test))]
    false
}

fn hash_len_prefixed(hasher: &mut Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn map_reqwest_error(error: reqwest::Error) -> CodexCatalogError {
    if error.is_timeout() {
        CodexCatalogError::Timeout
    } else {
        CodexCatalogError::Transport
    }
}

async fn read_limited_body(
    response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, CodexCatalogError> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(CodexCatalogError::ResponseTooLarge);
    }

    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(map_reqwest_error)?;
        if body.len().saturating_add(chunk.len()) > limit {
            return Err(CodexCatalogError::ResponseTooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::Router;
    use axum::body::Body;
    use axum::extract::{Request, State};
    use axum::http::Response;
    use axum::routing::get;
    use reqwest::header::HeaderMap;
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;

    use super::*;

    #[derive(Clone)]
    struct MockState {
        requests: Arc<Mutex<Vec<CapturedRequest>>>,
        replies: Arc<Mutex<Vec<MockReply>>>,
        delay: Duration,
    }

    #[derive(Clone, Debug)]
    struct CapturedRequest {
        path_and_query: String,
        headers: HeaderMap,
    }

    #[derive(Clone)]
    struct MockReply {
        status: StatusCode,
        body: String,
        etag: Option<&'static str>,
    }

    async fn mock_models(State(state): State<MockState>, request: Request) -> Response<Body> {
        state.requests.lock().await.push(CapturedRequest {
            path_and_query: request
                .uri()
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_default(),
            headers: request.headers().clone(),
        });
        if !state.delay.is_zero() {
            tokio::time::sleep(state.delay).await;
        }
        let reply = state.replies.lock().await.remove(0);
        let mut response = Response::builder().status(reply.status);
        if let Some(etag) = reply.etag {
            response = response.header(ETAG, etag);
        }
        response.body(Body::from(reply.body)).unwrap()
    }

    async fn spawn_server(replies: Vec<MockReply>) -> (Url, MockState) {
        spawn_server_with_delay(replies, Duration::ZERO).await
    }

    async fn spawn_server_with_delay(replies: Vec<MockReply>, delay: Duration) -> (Url, MockState) {
        let state = MockState {
            requests: Arc::new(Mutex::new(Vec::new())),
            replies: Arc::new(Mutex::new(replies)),
            delay,
        };
        let app = Router::new()
            .route("/backend-api/codex/models", get(mock_models))
            .with_state(state.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (
            Url::parse(&format!("http://{address}/backend-api/codex")).unwrap(),
            state,
        )
    }

    fn reply(status: StatusCode, body: impl Into<String>) -> MockReply {
        MockReply {
            status,
            body: body.into(),
            etag: None,
        }
    }

    fn auth(
        token: &str,
        account: &str,
        credential: &str,
        fedramp: bool,
    ) -> CodexCatalogAuthSnapshot {
        auth_at_generation(token, account, credential, 1, fedramp)
    }

    fn auth_at_generation(
        token: &str,
        account: &str,
        credential: &str,
        generation: u64,
        fedramp: bool,
    ) -> CodexCatalogAuthSnapshot {
        let mut binding = CredentialBinding::openai_codex(Some(credential.to_owned()));
        binding.generation = generation;
        CodexCatalogAuthSnapshot::new(token, account, None, binding, fedramp).unwrap()
    }

    fn client(cache_dir: &Path, base_url: Url) -> Result<CodexCatalogClient, CodexCatalogError> {
        let config = CodexCatalogClientConfig::new(cache_dir)?
            .with_base_url(base_url)?
            .with_client_version("9.8.7")?
            .with_client_headers("catalog-test/1", "grok-build-codex-test")?;
        CodexCatalogClient::new(config)
    }

    fn rich_body(slug: &str) -> String {
        let mut body: Value = serde_json::from_str(
            r#"{
                "models": [{
                    "slug": "placeholder",
                    "display_name": "GPT Codex",
                    "description": "Current entitled model",
                    "default_reasoning_level": "medium",
                    "supported_reasoning_levels": [
                        {"effort": "low", "description": "Fast"},
                        {"effort": "xhigh", "description": "Deep"}
                    ],
                    "shell_type": "unified_exec",
                    "visibility": "list",
                    "supported_in_api": true,
                    "priority": 42,
                    "additional_speed_tiers": ["fast"],
                    "service_tiers": [{"id": "priority", "name": "Priority", "description": "Low latency"}],
                    "default_service_tier": "priority",
                    "availability_nux": {"message": "Available now"},
                    "upgrade": {"model": "next-codex", "migration_markdown": "Upgrade"},
                    "base_instructions": "Use tools carefully.",
                    "model_messages": {"instructions_template": "template"},
                    "include_skills_usage_instructions": true,
                    "supports_reasoning_summary_parameter": true,
                    "default_reasoning_summary": "auto",
                    "support_verbosity": true,
                    "default_verbosity": "medium",
                    "apply_patch_tool_type": "freeform",
                    "web_search_tool_type": "text_and_image",
                    "truncation_policy": {"mode": "tokens", "limit": 12000},
                    "supports_parallel_tool_calls": true,
                    "supports_image_detail_original": true,
                    "context_window": 200000,
                    "max_context_window": 300000,
                    "auto_compact_token_limit": 180000,
                    "comp_hash": "opaque-comp-hash",
                    "effective_context_window_percent": 95,
                    "experimental_supported_tools": ["image_generation"],
                    "input_modalities": ["text", "image"],
                    "supports_search_tool": true,
                    "use_responses_lite": false,
                    "auto_review_model_override": "review-codex",
                    "tool_mode": "code_mode",
                    "multi_agent_version": "v2",
                    "future_capability": {"enabled": true}
                }]
            }"#,
        )
        .unwrap();
        body["models"][0]["slug"] = Value::String(slug.to_string());
        body.to_string()
    }

    fn catalog_model(slug: &str, priority: i32) -> CodexCatalogModel {
        CodexCatalogModel {
            slug: slug.to_owned(),
            priority,
            ..CodexCatalogModel::default()
        }
    }

    fn set_cached_fetched_at(
        client: &CodexCatalogClient,
        auth: &CodexCatalogAuthSnapshot,
        fetched_at_unix_ms: u64,
    ) {
        let scope = client.cache_scope(auth);
        let mut envelope = client
            .read_cache(&scope)
            .expect("cache should be readable")
            .expect("cache should exist");
        envelope.fetched_at_unix_ms = fetched_at_unix_ms;
        client
            .write_cache(&scope, &envelope)
            .expect("cache timestamp rewrite should succeed");
    }

    #[tokio::test]
    async fn sends_exact_scoped_headers_and_parses_rich_catalog() {
        let (base_url, state) =
            spawn_server(vec![reply(StatusCode::OK, rich_body("gpt-codex"))]).await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let result = client
            .fetch(&auth(
                "top-secret-token",
                "acct-secret",
                "credential-a",
                true,
            ))
            .await
            .unwrap();

        assert_eq!(result.source, CodexCatalogFetchSource::Network);
        assert_eq!(result.cache_status, CodexCatalogCacheStatus::Updated);
        let model = &result.catalog.models[0];
        assert_eq!(model.id, "openai-codex/gpt-codex");
        assert!(model.visible_in_picker());
        assert!(model.accepts_images());
        assert!(model.supports_parallel_tool_calls);
        assert!(model.supports_image_detail_original);
        assert_eq!(model.truncation_policy.as_ref().unwrap().limit, 12000);
        assert_eq!(model.extra["future_capability"]["enabled"], true);

        let requests = state.requests.lock().await;
        let request = &requests[0];
        assert_eq!(
            request.path_and_query,
            "/backend-api/codex/models?client_version=9.8.7"
        );
        assert_eq!(request.headers[AUTHORIZATION], "Bearer top-secret-token");
        assert_eq!(request.headers[CHATGPT_ACCOUNT_ID], "acct-secret");
        assert_eq!(request.headers[OPENAI_FEDRAMP], "true");
        assert_eq!(request.headers.get_all(OPENAI_FEDRAMP).iter().count(), 1);
        assert_eq!(request.headers[ORIGINATOR], "grok-build-codex-test");
        assert_eq!(request.headers[CODEX_VERSION], "9.8.7");
        assert_eq!(request.headers[USER_AGENT], "catalog-test/1");
        assert!(!request.headers.contains_key("x-api-key"));
        assert!(!request.headers.contains_key("x-xai-api-key"));
    }

    #[tokio::test]
    async fn default_wire_query_uses_the_audited_codex_compatibility_version() {
        let (base_url, state) = spawn_server(vec![reply(StatusCode::OK, r#"{"models":[]}"#)]).await;
        let cache = tempfile::tempdir().unwrap();
        let config = CodexCatalogClientConfig::new(cache.path())
            .unwrap()
            .with_base_url(base_url)
            .unwrap()
            .with_client_headers("catalog-test/1", "grok-build-codex-test")
            .unwrap();
        CodexCatalogClient::new(config)
            .unwrap()
            .fetch(&auth("token", "account", "credential", false))
            .await
            .unwrap();

        assert_eq!(
            state.requests.lock().await[0].path_and_query,
            format!(
                "/backend-api/codex/models?client_version={}",
                xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION
            )
        );
        assert_eq!(
            state.requests.lock().await[0].headers[CODEX_VERSION],
            xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION
        );
    }

    #[test]
    fn default_cache_ttl_matches_current_codex_behavior() {
        let cache = tempfile::tempdir().unwrap();
        let config = CodexCatalogClientConfig::new(cache.path()).unwrap();
        assert_eq!(config.cache_ttl, Duration::from_secs(300));
        assert_eq!(config.cache_ttl, OPENAI_CODEX_CATALOG_CACHE_TTL);
    }

    #[test]
    fn auth_snapshot_rejects_unpinned_or_cross_provider_receipts() {
        let mut valid = CredentialBinding::openai_codex(Some("credential".to_owned()));
        valid.generation = 1;
        assert!(
            CodexCatalogAuthSnapshot::new("token", "account", None, valid.clone(), false).is_ok()
        );

        let mut missing_generation = valid.clone();
        missing_generation.generation = 0;
        let mut missing_record = valid.clone();
        missing_record.record_id = None;
        let mut wrong_provider = valid;
        wrong_provider.provider = ProviderId::Xai;
        for rejected in [missing_generation, missing_record, wrong_provider] {
            assert!(
                CodexCatalogAuthSnapshot::new("token", "account", None, rejected, false).is_err()
            );
        }
    }

    #[tokio::test]
    async fn omits_fedramp_header_when_not_required() {
        let (base_url, state) = spawn_server(vec![reply(StatusCode::OK, r#"{"models":[]}"#)]).await;
        let cache = tempfile::tempdir().unwrap();
        client(cache.path(), base_url)
            .unwrap()
            .fetch(&auth("token", "account", "credential", false))
            .await
            .unwrap();
        assert!(
            !state.requests.lock().await[0]
                .headers
                .contains_key(OPENAI_FEDRAMP)
        );
    }

    #[tokio::test]
    async fn etag_drives_conditional_fetch_and_304_cache_reuse() {
        let first = MockReply {
            status: StatusCode::OK,
            body: rich_body("etag-model"),
            etag: Some("\"catalog-v1\""),
        };
        let second = reply(StatusCode::NOT_MODIFIED, "");
        let (base_url, state) = spawn_server(vec![first, second]).await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let auth = auth("token", "account", "credential", false);

        client.fetch(&auth).await.unwrap();
        let result = client.fetch(&auth).await.unwrap();

        assert_eq!(result.source, CodexCatalogFetchSource::NotModified);
        assert_eq!(result.catalog.models[0].id, "openai-codex/etag-model");
        assert_eq!(
            state.requests.lock().await[1].headers[IF_NONE_MATCH],
            "\"catalog-v1\""
        );
    }

    #[tokio::test]
    async fn fallback_cache_accepts_fresh_entries_and_rejects_stale_entries() {
        let (base_url, _) =
            spawn_server(vec![reply(StatusCode::OK, rich_body("cached-model"))]).await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let auth = auth("token", "account", "credential", false);
        client.fetch(&auth).await.unwrap();

        set_cached_fetched_at(
            &client,
            &auth,
            now_unix_ms().saturating_sub(Duration::from_secs(240).as_millis() as u64),
        );
        assert_eq!(
            client.load_cached(&auth).await.unwrap().catalog.models[0].slug,
            "cached-model"
        );

        set_cached_fetched_at(
            &client,
            &auth,
            now_unix_ms().saturating_sub(Duration::from_secs(360).as_millis() as u64),
        );
        assert_eq!(
            client.load_cached(&auth).await.unwrap_err(),
            CodexCatalogError::CacheUnavailable
        );
    }

    #[tokio::test]
    async fn late_older_generation_cache_write_is_not_relabelled_by_newer_credentials() {
        let (base_url, _) = spawn_server(vec![
            reply(StatusCode::OK, rich_body("generation-two")),
            reply(StatusCode::OK, rich_body("late-generation-one")),
        ])
        .await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let generation_two = auth_at_generation("token-two", "account", "credential", 2, false);
        let generation_one = auth_at_generation("token-one", "account", "credential", 1, false);

        client.fetch(&generation_two).await.unwrap();
        client.fetch(&generation_one).await.unwrap();

        assert_eq!(
            client.load_cached(&generation_two).await.unwrap_err(),
            CodexCatalogError::CacheUnavailable,
            "a late generation-one response must not become generation-two fallback data"
        );
        assert_eq!(
            client
                .load_cached(&generation_one)
                .await
                .unwrap()
                .catalog
                .models[0]
                .slug,
            "late-generation-one"
        );
    }

    #[test]
    fn cache_scope_includes_chatgpt_user_and_fedramp_identity() {
        let cache = tempfile::tempdir().unwrap();
        let client = client(
            cache.path(),
            Url::parse("http://127.0.0.1:12345/backend-api/codex").unwrap(),
        )
        .unwrap();
        let mut binding = CredentialBinding::openai_codex(Some("credential".to_owned()));
        binding.generation = 1;
        let user_a = CodexCatalogAuthSnapshot::new(
            "token",
            "account",
            Some("user-a"),
            binding.clone(),
            false,
        )
        .unwrap();
        let user_b = CodexCatalogAuthSnapshot::new(
            "token",
            "account",
            Some("user-b"),
            binding.clone(),
            false,
        )
        .unwrap();
        let fedramp =
            CodexCatalogAuthSnapshot::new("token", "account", Some("user-a"), binding, true)
                .unwrap();

        assert_ne!(
            client.cache_scope(&user_a).digest,
            client.cache_scope(&user_b).digest
        );
        assert_ne!(
            client.cache_scope(&user_a).digest,
            client.cache_scope(&fedramp).digest
        );
    }

    #[tokio::test]
    async fn matching_live_etag_renews_only_the_matching_account_cache() {
        let response = MockReply {
            status: StatusCode::OK,
            body: rich_body("renewed-model"),
            etag: Some("\"catalog-v2\""),
        };
        let (base_url, _) = spawn_server(vec![response]).await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let auth = auth("token", "account", "credential", false);
        client.fetch(&auth).await.unwrap();
        set_cached_fetched_at(
            &client,
            &auth,
            now_unix_ms().saturating_sub(Duration::from_secs(360).as_millis() as u64),
        );

        assert_eq!(
            client.renew_cached(&auth, "\"catalog-v1\"").await,
            Err(CodexCatalogError::CacheUnavailable)
        );
        assert!(client.load_cached(&auth).await.is_err());

        client.renew_cached(&auth, "\"catalog-v2\"").await.unwrap();
        assert_eq!(
            client.load_cached(&auth).await.unwrap().catalog.models[0].slug,
            "renewed-model"
        );
    }

    #[tokio::test]
    async fn stale_cache_etag_can_be_revalidated_but_not_used_as_fallback() {
        let first = MockReply {
            status: StatusCode::OK,
            body: rich_body("revalidated-model"),
            etag: Some("\"catalog-v1\""),
        };
        let (base_url, state) =
            spawn_server(vec![first, reply(StatusCode::NOT_MODIFIED, "")]).await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let auth = auth("token", "account", "credential", false);
        client.fetch(&auth).await.unwrap();
        set_cached_fetched_at(
            &client,
            &auth,
            now_unix_ms().saturating_sub(Duration::from_secs(360).as_millis() as u64),
        );

        assert_eq!(
            client.load_cached(&auth).await.unwrap_err(),
            CodexCatalogError::CacheUnavailable
        );
        let revalidated = client.fetch(&auth).await.unwrap();
        assert_eq!(revalidated.source, CodexCatalogFetchSource::NotModified);
        assert_eq!(revalidated.catalog.models[0].slug, "revalidated-model");
        assert!(client.load_cached(&auth).await.is_ok());
        assert_eq!(
            state.requests.lock().await[1].headers[IF_NONE_MATCH],
            "\"catalog-v1\""
        );
    }

    #[tokio::test]
    async fn authenticated_304_adopts_cache_for_the_exact_new_generation() {
        let first = MockReply {
            status: StatusCode::OK,
            body: rich_body("generation-revalidated"),
            etag: Some("\"catalog-v1\""),
        };
        let (base_url, _) = spawn_server(vec![first, reply(StatusCode::NOT_MODIFIED, "")]).await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let generation_one = auth_at_generation("token-one", "account", "credential", 1, false);
        let generation_two = auth_at_generation("token-two", "account", "credential", 2, false);

        client.fetch(&generation_one).await.unwrap();
        assert_eq!(
            client.load_cached(&generation_two).await.unwrap_err(),
            CodexCatalogError::CacheUnavailable
        );

        let revalidated = client.fetch(&generation_two).await.unwrap();
        assert_eq!(revalidated.source, CodexCatalogFetchSource::NotModified);
        assert_eq!(
            client
                .load_cached(&generation_two)
                .await
                .unwrap()
                .catalog
                .models[0]
                .slug,
            "generation-revalidated"
        );
    }

    #[tokio::test]
    async fn authenticated_empty_catalog_replaces_non_empty_cache() {
        let (base_url, _) = spawn_server(vec![
            reply(StatusCode::OK, rich_body("previously-entitled")),
            reply(StatusCode::OK, r#"{"models":[]}"#),
        ])
        .await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let auth = auth("token", "account", "credential", false);

        assert_eq!(client.fetch(&auth).await.unwrap().catalog.models.len(), 1);
        assert!(client.fetch(&auth).await.unwrap().catalog.models.is_empty());
        assert!(
            client
                .load_cached(&auth)
                .await
                .unwrap()
                .catalog
                .models
                .is_empty()
        );
    }

    #[tokio::test]
    async fn cache_is_scoped_by_account_and_opaque_credential_identity() {
        let (base_url, _) = spawn_server(vec![
            reply(StatusCode::OK, rich_body("account-a-model")),
            reply(StatusCode::OK, rich_body("account-b-model")),
            reply(StatusCode::OK, rich_body("rotated-identity-model")),
        ])
        .await;
        let cache = tempfile::tempdir().unwrap();
        let client = client(cache.path(), base_url).unwrap();
        let account_a = auth("token-a", "account-a", "credential-shared", false);
        let account_b = auth("token-b", "account-b", "credential-shared", false);
        let rotated_identity = auth("token-c", "account-b", "credential-other", false);

        client.fetch(&account_a).await.unwrap();
        client.fetch(&account_b).await.unwrap();
        client.fetch(&rotated_identity).await.unwrap();
        assert_eq!(
            client.load_cached(&account_a).await.unwrap().catalog.models[0].slug,
            "account-a-model"
        );
        assert_eq!(
            client.load_cached(&account_b).await.unwrap().catalog.models[0].slug,
            "account-b-model"
        );
        assert_eq!(
            client
                .load_cached(&rotated_identity)
                .await
                .unwrap()
                .catalog
                .models[0]
                .slug,
            "rotated-identity-model"
        );
        assert_eq!(std::fs::read_dir(cache.path()).unwrap().count(), 3);
    }

    #[tokio::test]
    async fn request_timeout_is_bounded_and_redacted() {
        let (base_url, _) = spawn_server_with_delay(
            vec![reply(StatusCode::OK, r#"{"models":[]}"#)],
            Duration::from_millis(250),
        )
        .await;
        let cache = tempfile::tempdir().unwrap();
        let config = CodexCatalogClientConfig::new(cache.path())
            .unwrap()
            .with_base_url(base_url)
            .unwrap()
            .with_timeout(Duration::from_millis(20))
            .unwrap();
        let client = CodexCatalogClient::new(config).unwrap();
        let error = client
            .fetch(&auth(
                "timeout-secret-token",
                "timeout-secret-account",
                "timeout-secret-credential",
                false,
            ))
            .await
            .unwrap_err();
        assert_eq!(error, CodexCatalogError::Timeout);
        let printed = format!("{error:?} {error}");
        assert!(!printed.contains("timeout-secret"));
    }

    #[tokio::test]
    async fn errors_debug_and_cache_never_contain_credentials() {
        let token = "distinct-secret-token";
        let account = "distinct-secret-account";
        let credential = "distinct-secret-credential";
        let malicious_body = format!(r#"{{"models": ["{token}", "{account}", "{credential}"]}}"#);
        let (base_url, _) = spawn_server(vec![reply(StatusCode::OK, malicious_body)]).await;
        let cache = tempfile::tempdir().unwrap();
        let auth = auth(token, account, credential, false);
        let debug_auth = format!("{auth:?}");
        for secret in [token, account, credential] {
            assert!(!debug_auth.contains(secret));
        }

        let error = client(cache.path(), base_url)
            .unwrap()
            .fetch(&auth)
            .await
            .unwrap_err();
        let printed = format!("{error:?} {error}");
        for secret in [token, account, credential] {
            assert!(!printed.contains(secret));
        }

        // A successful fetch writes only hashed scope data and model metadata.
        let (base_url, _) =
            spawn_server(vec![reply(StatusCode::OK, rich_body("safe-model"))]).await;
        client(cache.path(), base_url)
            .unwrap()
            .fetch(&auth)
            .await
            .unwrap();
        let cache_text = std::fs::read_dir(cache.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
            .collect::<String>();
        for secret in [token, account, credential] {
            assert!(!cache_text.contains(secret));
        }
    }

    #[tokio::test]
    async fn not_modified_without_matching_cache_is_an_error() {
        let (base_url, _) = spawn_server(vec![reply(StatusCode::NOT_MODIFIED, "")]).await;
        let cache = tempfile::tempdir().unwrap();
        let error = client(cache.path(), base_url)
            .unwrap()
            .fetch(&auth("token", "account", "credential", false))
            .await
            .unwrap_err();
        assert_eq!(error, CodexCatalogError::NotModifiedWithoutCache);
    }

    #[test]
    fn production_base_url_is_sealed_and_test_seam_is_loopback_only() {
        let cache = tempfile::tempdir().unwrap();
        let config = CodexCatalogClientConfig::new(cache.path()).unwrap();
        assert_eq!(
            config.client_version,
            xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION
        );
        assert_eq!(
            config
                .clone()
                .with_client_version("invalid\nheader")
                .unwrap_err(),
            CodexCatalogError::InvalidConfiguration
        );
        assert!(
            config
                .clone()
                .with_base_url(Url::parse(OPENAI_CODEX_BASE_URL).unwrap())
                .is_ok()
        );
        assert!(
            config
                .clone()
                .with_base_url(Url::parse("http://127.0.0.1:43210/backend-api/codex").unwrap())
                .is_ok()
        );
        for forbidden in [
            "https://example.com/backend-api/codex",
            "http://chatgpt.com/backend-api/codex",
            "https://chatgpt.com/backend-api/codex/other",
            "http://localhost:43210/backend-api/codex",
            "http://user:password@127.0.0.1:43210/backend-api/codex",
        ] {
            assert_eq!(
                config
                    .clone()
                    .with_base_url(Url::parse(forbidden).unwrap())
                    .unwrap_err(),
                CodexCatalogError::InvalidConfiguration,
                "unexpectedly allowed {forbidden}"
            );
        }
    }

    #[tokio::test]
    async fn rejects_duplicate_slugs_and_oversized_responses() {
        let duplicate = r#"{"models":[{"slug":"same"},{"slug":"same"}]}"#;
        let (base_url, _) = spawn_server(vec![reply(StatusCode::OK, duplicate)]).await;
        let cache = tempfile::tempdir().unwrap();
        let error = client(cache.path(), base_url)
            .unwrap()
            .fetch(&auth("token", "account", "credential", false))
            .await
            .unwrap_err();
        assert_eq!(error, CodexCatalogError::InvalidModel);

        let (base_url, _) = spawn_server(vec![reply(StatusCode::OK, r#"{"models":[]}"#)]).await;
        let config = CodexCatalogClientConfig::new(cache.path())
            .unwrap()
            .with_base_url(base_url)
            .unwrap()
            .with_max_response_bytes(4)
            .unwrap();
        let error = CodexCatalogClient::new(config)
            .unwrap()
            .fetch(&auth("token", "account", "credential", false))
            .await
            .unwrap_err();
        assert_eq!(error, CodexCatalogError::ResponseTooLarge);
    }

    #[test]
    fn normalized_catalog_is_sorted_by_ascending_priority() {
        let catalog = CodexModelCatalog::from_wire(vec![
            catalog_model("third", 30),
            catalog_model("first", 10),
            catalog_model("second", 20),
        ])
        .unwrap();

        assert_eq!(
            catalog
                .models
                .iter()
                .map(|model| model.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "second", "third"]
        );
    }

    #[test]
    fn priority_sort_preserves_service_order_for_ties() {
        let catalog = CodexModelCatalog::from_wire(vec![
            catalog_model("tie-a", 20),
            catalog_model("first", 10),
            catalog_model("tie-b", 20),
            catalog_model("tie-c", 20),
            catalog_model("last", 30),
        ])
        .unwrap();

        assert_eq!(
            catalog
                .models
                .iter()
                .map(|model| model.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "tie-a", "tie-b", "tie-c", "last"]
        );
    }

    #[derive(Clone)]
    struct CountingAuth {
        calls: Arc<AtomicUsize>,
        auth: CodexCatalogAuthSnapshot,
    }

    #[derive(Clone)]
    struct RecoveringAuth {
        auth_calls: Arc<AtomicUsize>,
        recovery_calls: Arc<AtomicUsize>,
        rejected_bindings: Arc<Mutex<Vec<CredentialBinding>>>,
        rejected: CodexCatalogAuthSnapshot,
        recovered: CodexCatalogAuthSnapshot,
    }

    #[async_trait]
    impl CodexCatalogAuthProvider for RecoveringAuth {
        async fn catalog_auth(&self) -> Result<CodexCatalogAuthSnapshot, CodexCatalogAuthError> {
            self.auth_calls.fetch_add(1, Ordering::Relaxed);
            if self.recovery_calls.load(Ordering::Relaxed) == 0 {
                Ok(self.rejected.clone())
            } else {
                Ok(self.recovered.clone())
            }
        }

        async fn recover_unauthorized(
            &self,
            rejected: CredentialBinding,
        ) -> Result<bool, CodexCatalogAuthError> {
            self.recovery_calls.fetch_add(1, Ordering::Relaxed);
            self.rejected_bindings.lock().await.push(rejected);
            Ok(true)
        }
    }

    #[tokio::test]
    async fn provider_scoped_401_recovery_reloads_auth_and_retries_once() {
        let (base_url, state) = spawn_server(vec![
            reply(StatusCode::UNAUTHORIZED, "rejected body is never surfaced"),
            reply(StatusCode::OK, rich_body("recovered-model")),
        ])
        .await;
        let cache = tempfile::tempdir().unwrap();
        let auth_calls = Arc::new(AtomicUsize::new(0));
        let recovery_calls = Arc::new(AtomicUsize::new(0));
        let rejected_bindings = Arc::new(Mutex::new(Vec::new()));
        let auth = RecoveringAuth {
            auth_calls: auth_calls.clone(),
            recovery_calls: recovery_calls.clone(),
            rejected_bindings: rejected_bindings.clone(),
            rejected: auth_at_generation(
                "expired-token",
                "old-account",
                "old-credential",
                7,
                false,
            ),
            recovered: auth_at_generation(
                "refreshed-token",
                "new-account",
                "new-credential",
                8,
                true,
            ),
        };

        let result = client(cache.path(), base_url)
            .unwrap()
            .fetch(&auth)
            .await
            .unwrap();
        assert_eq!(result.catalog.models[0].slug, "recovered-model");
        assert_eq!(auth_calls.load(Ordering::Relaxed), 2);
        assert_eq!(recovery_calls.load(Ordering::Relaxed), 1);
        let rejected = rejected_bindings.lock().await;
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].record_id.as_deref(), Some("old-credential"));
        assert_eq!(rejected[0].generation, 7);
        drop(rejected);

        let requests = state.requests.lock().await;
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].headers[AUTHORIZATION], "Bearer expired-token");
        assert_eq!(requests[0].headers[CHATGPT_ACCOUNT_ID], "old-account");
        assert_eq!(requests[1].headers[AUTHORIZATION], "Bearer refreshed-token");
        assert_eq!(requests[1].headers[CHATGPT_ACCOUNT_ID], "new-account");
        assert_eq!(requests[1].headers[OPENAI_FEDRAMP], "true");
    }

    #[tokio::test]
    async fn second_401_is_returned_without_a_recovery_loop_or_stale_fallback() {
        let (base_url, _) = spawn_server(vec![
            reply(StatusCode::UNAUTHORIZED, "first rejection"),
            reply(StatusCode::UNAUTHORIZED, "second rejection"),
        ])
        .await;
        let cache = tempfile::tempdir().unwrap();
        let auth_calls = Arc::new(AtomicUsize::new(0));
        let recovery_calls = Arc::new(AtomicUsize::new(0));
        let auth = RecoveringAuth {
            auth_calls: auth_calls.clone(),
            recovery_calls: recovery_calls.clone(),
            rejected_bindings: Arc::new(Mutex::new(Vec::new())),
            rejected: auth("expired-token", "account", "credential", false),
            recovered: auth("still-rejected", "account", "credential", false),
        };

        let error = client(cache.path(), base_url)
            .unwrap()
            .fetch(&auth)
            .await
            .unwrap_err();
        assert_eq!(error, CodexCatalogError::Unauthorized);
        assert_eq!(auth_calls.load(Ordering::Relaxed), 2);
        assert_eq!(recovery_calls.load(Ordering::Relaxed), 1);
    }

    #[async_trait]
    impl CodexCatalogAuthProvider for CountingAuth {
        async fn catalog_auth(&self) -> Result<CodexCatalogAuthSnapshot, CodexCatalogAuthError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(self.auth.clone())
        }
    }

    #[tokio::test]
    async fn resolves_a_fresh_auth_snapshot_for_each_operation() {
        let (base_url, _) = spawn_server(vec![
            reply(StatusCode::OK, r#"{"models":[]}"#),
            reply(StatusCode::OK, r#"{"models":[]}"#),
        ])
        .await;
        let cache = tempfile::tempdir().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let auth = CountingAuth {
            calls: calls.clone(),
            auth: auth("token", "account", "credential", false),
        };
        let client = client(cache.path(), base_url).unwrap();
        client.fetch(&auth).await.unwrap();
        client.fetch(&auth).await.unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }
}
