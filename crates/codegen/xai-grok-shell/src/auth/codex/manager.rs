use std::fmt;
use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use parking_lot::RwLock;

use super::{
    CodexAuthError, CodexCredentialStore, CodexCredentials, CodexOAuthClient,
    OPENAI_CODEX_PROVIDER_ID,
};

/// Live, provider-scoped owner of one Grok Build Codex credential record.
///
/// The manager never consults xAI authentication and never imports
/// `~/.codex/auth.json`. All refreshes are serialized by both an in-process
/// mutex and the store's persistent kernel lock, then reload the latest record
/// before spending a rotating refresh token.
pub struct CodexAuthManager {
    store: CodexCredentialStore,
    oauth: CodexOAuthClient,
    current: RwLock<Option<CodexCredentials>>,
    refresh_gate: tokio::sync::Mutex<()>,
}

/// Cross-process lease proving that the auth record still belongs to the
/// catalog-producing identity. Holding it through in-memory publication makes
/// login/logout/account switching serialize with catalog installation.
pub(crate) struct CodexCatalogIdentityLease {
    _provider_lock: super::storage::CodexProviderLock,
}

impl fmt::Debug for CodexAuthManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexAuthManager")
            .field("provider", &OPENAI_CODEX_PROVIDER_ID)
            .field("configured", &self.current.read().is_some())
            .finish()
    }
}

impl CodexAuthManager {
    pub fn new(grok_home: &Path) -> Result<Self, CodexAuthError> {
        Self::from_store(CodexCredentialStore::new(grok_home))
    }

    pub fn from_store(store: CodexCredentialStore) -> Result<Self, CodexAuthError> {
        Self::with_oauth(store, CodexOAuthClient::new())
    }

    fn with_oauth(
        store: CodexCredentialStore,
        oauth: CodexOAuthClient,
    ) -> Result<Self, CodexAuthError> {
        let current = store.load()?;
        Ok(Self {
            store,
            oauth,
            current: RwLock::new(current),
            refresh_gate: tokio::sync::Mutex::new(()),
        })
    }

    #[cfg(test)]
    pub(super) fn for_test(
        store: CodexCredentialStore,
        oauth: CodexOAuthClient,
    ) -> Result<Self, CodexAuthError> {
        Self::with_oauth(store, oauth)
    }

    pub fn store(&self) -> &CodexCredentialStore {
        &self.store
    }

    /// Cached credential snapshot. Formatting the result remains fully
    /// redacted; secret access is explicit at request construction.
    pub fn current(&self) -> Option<CodexCredentials> {
        self.current.read().clone()
    }

    /// Verify the cached request credential against the provider-scoped disk
    /// record. Every outbound request uses this path so a local or sibling
    /// process logout/account switch fails closed immediately rather than
    /// continuing to send an old bearer token until it expires.
    pub fn current_verified(&self) -> Result<CodexCredentials, CodexAuthError> {
        let disk = self.store.load()?.ok_or(CodexAuthError::NotLoggedIn)?;
        if let Some(cached) = self.current() {
            if disk.credential_id() != cached.credential_id()
                || disk.revision < cached.revision
                || !disk.same_identity(&cached)
            {
                return Err(CodexAuthError::AccountChanged);
            }
        }
        *self.current.write() = Some(disk.clone());
        Ok(disk)
    }

    /// Reload credentials explicitly, allowing an intentional account switch.
    /// Active request recovery uses the identity-preserving refresh path below
    /// instead, so a sibling login cannot silently change a running session.
    pub fn reload(&self) -> Result<Option<CodexCredentials>, CodexAuthError> {
        let disk = self.store.load()?;
        *self.current.write() = disk.clone();
        Ok(disk)
    }

    pub(crate) async fn lock_catalog_identity(
        &self,
        expected: &CodexCredentials,
    ) -> Result<CodexCatalogIdentityLease, CodexAuthError> {
        let (disk, lease) = self
            .lock_catalog_binding(&expected.credential_binding())
            .await?;
        if !disk.same_identity(expected) {
            return Err(CodexAuthError::AccountChanged);
        }
        Ok(lease)
    }

    /// Lock and attest the exact credential generation that authenticated a
    /// catalog/cache operation. This is stricter than stable account identity:
    /// a same-account refresh or relogin cannot publish an older in-flight
    /// result after a newer generation has reached disk.
    pub(crate) async fn lock_catalog_binding(
        &self,
        expected: &xai_grok_sampling_types::CredentialBinding,
    ) -> Result<(CodexCredentials, CodexCatalogIdentityLease), CodexAuthError> {
        if expected.provider != xai_grok_sampling_types::ProviderId::OpenAiCodex
            || expected.source
                != xai_grok_sampling_types::CredentialSourceId::OpenAiCodexSubscription
            || expected.generation == 0
            || expected
                .record_id
                .as_deref()
                .is_none_or(|record_id| record_id.trim().is_empty())
        {
            return Err(CodexAuthError::AccountChanged);
        }
        let provider_lock = self.store.acquire_provider_lock().await?;
        let disk = self.store.load()?.ok_or(CodexAuthError::NotLoggedIn)?;
        if disk.credential_binding() != *expected {
            return Err(CodexAuthError::AccountChanged);
        }
        Ok((
            disk,
            CodexCatalogIdentityLease {
                _provider_lock: provider_lock,
            },
        ))
    }

    /// Return a credential that remains valid beyond the early-refresh window.
    pub async fn ensure_fresh(&self) -> Result<CodexCredentials, CodexAuthError> {
        let current = self.current_verified()?;
        if !current.needs_refresh_at(Utc::now()) {
            return Ok(current);
        }
        self.refresh_expired(current).await
    }

    /// Provider-local 401 recovery. The latest on-disk generation is adopted
    /// first when a sibling already rotated it; otherwise exactly one refresh
    /// is performed under the provider refresh lock.
    pub async fn recover_after_unauthorized(
        &self,
        rejected: xai_grok_sampling_types::CredentialBinding,
    ) -> Result<bool, CodexAuthError> {
        if rejected.provider != xai_grok_sampling_types::ProviderId::OpenAiCodex
            || rejected.source
                != xai_grok_sampling_types::CredentialSourceId::OpenAiCodexSubscription
            || rejected
                .record_id
                .as_deref()
                .is_none_or(|record_id| record_id.trim().is_empty())
            || rejected.generation == 0
        {
            return Err(CodexAuthError::AccountChanged);
        }
        let rejected_record = rejected.record_id.as_deref().unwrap_or_default();

        let _in_process = self.refresh_gate.lock().await;
        let _cross_process = self.store.acquire_provider_lock().await?;
        let disk = self.store.load()?.ok_or(CodexAuthError::NotLoggedIn)?;

        if disk.credential_id() != rejected_record || disk.revision < rejected.generation {
            return Err(CodexAuthError::AccountChanged);
        }
        if let Some(current) = self.current()
            && (current.credential_id() != rejected_record
                || !current.same_identity(&disk)
                || current.revision > disk.revision)
        {
            return Err(CodexAuthError::AccountChanged);
        }

        // The 401 belongs to the exact generation in `rejected`. If any sibling
        // already advanced this same credential record, adopt it unconditionally
        // even when its unchanged access token remains near expiry. Spending the
        // replacement refresh token for a stale 401 would reintroduce the
        // staggered-response double-rotation race.
        if disk.revision > rejected.generation {
            *self.current.write() = Some(disk.clone());
            return Ok(true);
        }

        let tokens = self.oauth.refresh_tokens(disk.refresh_token()).await?;
        let refreshed = CodexCredentials::from_token_response(tokens, Some(&disk), None)?;
        self.store.save_locked(refreshed.clone()).await?;
        *self.current.write() = Some(refreshed.clone());
        Ok(true)
    }

    async fn refresh_expired(
        &self,
        expected: CodexCredentials,
    ) -> Result<CodexCredentials, CodexAuthError> {
        let _in_process = self.refresh_gate.lock().await;
        let _cross_process = self.store.acquire_provider_lock().await?;
        let disk = self.store.load()?.ok_or(CodexAuthError::NotLoggedIn)?;

        if disk.credential_id() != expected.credential_id()
            || disk.revision < expected.revision
            || !disk.same_identity(&expected)
        {
            return Err(CodexAuthError::AccountChanged);
        }
        if !disk.needs_refresh_at(Utc::now()) {
            *self.current.write() = Some(disk.clone());
            return Ok(disk);
        }

        let tokens = self.oauth.refresh_tokens(disk.refresh_token()).await?;
        let refreshed = CodexCredentials::from_token_response(tokens, Some(&disk), None)?;
        self.store.save_locked(refreshed.clone()).await?;
        *self.current.write() = Some(refreshed.clone());
        Ok(refreshed)
    }

    /// Remove only the `openai::codex` record, then revoke its tokens best
    /// effort. xAI credentials and unknown scopes remain. Network revocation
    /// deliberately runs after releasing the provider/file locks so a slow
    /// auth service cannot block concurrent login, refresh, or account-bound
    /// catalog invalidation.
    pub async fn logout(&self) -> Result<CodexLogoutResult, CodexAuthError> {
        let credentials = {
            let _in_process = self.refresh_gate.lock().await;
            let _cross_process = self.store.acquire_provider_lock().await?;
            let disk = self.store.load()?;
            let Some(credentials) = disk else {
                *self.current.write() = None;
                return Ok(CodexLogoutResult {
                    removed: false,
                    revoke_succeeded: false,
                });
            };

            let removed = self.store.remove_locked().await?.is_some();
            if removed {
                *self.current.write() = None;
                Some(credentials)
            } else {
                None
            }
        };

        let Some(credentials) = credentials else {
            return Ok(CodexLogoutResult {
                removed: false,
                revoke_succeeded: false,
            });
        };
        let revoke_succeeded = self.oauth.revoke(&credentials).await.is_ok();
        Ok(CodexLogoutResult {
            removed: true,
            revoke_succeeded,
        })
    }

    pub async fn request_auth(&self) -> Result<xai_grok_tools::types::RequestAuth, CodexAuthError> {
        let credentials = self.ensure_fresh().await?;
        let mut headers = vec![
            (
                "Authorization".to_owned(),
                format!("Bearer {}", credentials.access_token()),
            ),
            (
                "ChatGPT-Account-ID".to_owned(),
                credentials.account_id().to_owned(),
            ),
        ];
        if credentials.is_fedramp {
            headers.push(("X-OpenAI-Fedramp".to_owned(), "true".to_owned()));
        }
        let credential_snapshot = xai_grok_tools::types::RequestCredentialSnapshot::new(
            credentials.credential_id(),
            credentials.revision,
        );
        Ok(xai_grok_tools::types::RequestAuth::for_provider_snapshot(
            OPENAI_CODEX_PROVIDER_ID,
            credential_snapshot,
            headers,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodexLogoutResult {
    pub removed: bool,
    pub revoke_succeeded: bool,
}

/// Adapter used by Grok's existing tool registry. It supplies the complete
/// per-request Codex header set and routes a 401 exclusively through the
/// Codex manager, never through the xAI `AuthManager`.
#[derive(Clone)]
pub struct CodexApiKeyProvider(pub Arc<CodexAuthManager>);

impl fmt::Debug for CodexApiKeyProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexApiKeyProvider")
            .field("provider", &OPENAI_CODEX_PROVIDER_ID)
            .finish()
    }
}

impl xai_grok_tools::types::ApiKeyProvider for CodexApiKeyProvider {
    fn current_api_key(&self) -> Option<String> {
        None
    }

    fn current_api_key_async(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<String>> + Send + '_>> {
        Box::pin(std::future::ready(None))
    }

    fn current_request_auth_async(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Option<xai_grok_tools::types::RequestAuth>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move { self.0.request_auth().await.ok() })
    }

    fn recover_unauthorized_async(
        &self,
        rejected: Option<xai_grok_tools::types::RequestCredentialSnapshot>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            let Some(rejected) = rejected else {
                return false;
            };
            let mut binding = xai_grok_sampling_types::CredentialBinding::openai_codex(Some(
                rejected.opaque_id().to_owned(),
            ));
            binding.generation = rejected.generation();
            self.0
                .recover_after_unauthorized(binding)
                .await
                .unwrap_or(false)
        })
    }
}

pub fn shared_api_key_provider(
    manager: Arc<CodexAuthManager>,
) -> xai_grok_tools::types::SharedApiKeyProvider {
    Arc::new(CodexApiKeyProvider(manager))
}

/// Synchronous sampler hook backed by a disk-verified manager snapshot.
/// [`CodexAuthManager::ensure_fresh`] is still called before constructing or
/// rebuilding a sampling client; the per-request verification here additionally
/// makes cross-process logout and account switching fail closed immediately.
#[derive(Clone)]
pub struct CodexSamplerRequestAuth(pub Arc<CodexAuthManager>);

impl fmt::Debug for CodexSamplerRequestAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexSamplerRequestAuth")
            .field("provider", &OPENAI_CODEX_PROVIDER_ID)
            .finish()
    }
}

impl xai_grok_sampler::RequestAuth for CodexSamplerRequestAuth {
    fn apply(
        &self,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<xai_grok_sampling_types::CredentialBinding, xai_grok_sampler::RequestAuthError>
    {
        use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};

        let credentials = self
            .0
            .current_verified()
            .map_err(|_| xai_grok_sampler::RequestAuthError::CredentialsUnavailable)?;
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {}", credentials.access_token()))
                .map_err(|_| xai_grok_sampler::RequestAuthError::InvalidAuthorizationHeader)?;
        authorization.set_sensitive(true);
        let mut account_id = HeaderValue::from_str(credentials.account_id())
            .map_err(|_| xai_grok_sampler::RequestAuthError::InvalidAccountIdHeader)?;
        account_id.set_sensitive(true);

        headers.insert(AUTHORIZATION, authorization);
        headers.insert(HeaderName::from_static("chatgpt-account-id"), account_id);
        if credentials.is_fedramp {
            let mut fedramp = HeaderValue::from_static("true");
            fedramp.set_sensitive(true);
            headers.insert(HeaderName::from_static("x-openai-fedramp"), fedramp);
        } else {
            headers.remove(HeaderName::from_static("x-openai-fedramp"));
        }
        Ok(credentials.credential_binding())
    }

    fn recover_unauthorized(
        &self,
        rejected: xai_grok_sampling_types::CredentialBinding,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            self.0
                .recover_after_unauthorized(rejected)
                .await
                .unwrap_or(false)
        })
    }
}

pub fn shared_sampler_request_auth(
    manager: Arc<CodexAuthManager>,
) -> xai_grok_sampler::SharedRequestAuth {
    Arc::new(CodexSamplerRequestAuth(manager))
}

fn catalog_auth_error(error: &CodexAuthError) -> crate::remote::CodexCatalogAuthError {
    let transient_storage = matches!(
        error,
        CodexAuthError::Storage(source)
            if matches!(
                source.kind(),
                std::io::ErrorKind::Interrupted
                    | std::io::ErrorKind::WouldBlock
                    | std::io::ErrorKind::TimedOut
            )
    );
    if transient_storage
        || matches!(
            error,
            CodexAuthError::Transport(_)
                | CodexAuthError::LockTimeout
                | CodexAuthError::HttpStatus(408 | 429 | 500..=599)
        )
    {
        crate::remote::CodexCatalogAuthError::Transient
    } else {
        crate::remote::CodexCatalogAuthError::Unavailable
    }
}

#[async_trait::async_trait]
impl crate::remote::CodexCatalogAuthProvider for CodexAuthManager {
    async fn catalog_auth(
        &self,
    ) -> Result<crate::remote::CodexCatalogAuthSnapshot, crate::remote::CodexCatalogAuthError> {
        let credentials = self
            .ensure_fresh()
            .await
            .map_err(|error| catalog_auth_error(&error))?;
        crate::remote::CodexCatalogAuthSnapshot::new(
            credentials.access_token(),
            credentials.account_id(),
            credentials.user_id(),
            credentials.credential_binding(),
            credentials.is_fedramp,
        )
    }

    async fn recover_unauthorized(
        &self,
        rejected: xai_grok_sampling_types::CredentialBinding,
    ) -> Result<bool, crate::remote::CodexCatalogAuthError> {
        self.recover_after_unauthorized(rejected)
            .await
            .map_err(|error| catalog_auth_error(&error))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::Router;
    use axum::extract::{Json, State};
    use axum::routing::post;
    use chrono::{Duration, Utc};
    use tokio::net::TcpListener;
    use url::Url;

    use super::*;
    use crate::auth::GrokAuth;
    use crate::auth::codex::credentials::{credentials_for_test, jwt_for_test};
    use crate::remote::CodexCatalogAuthProvider;
    use xai_grok_sampler::RequestAuth as _;
    use xai_grok_tools::types::ApiKeyProvider as _;

    struct OAuthState {
        token_calls: AtomicUsize,
        revoke_hints: Mutex<Vec<String>>,
        account_id: String,
    }

    async fn refresh_handler(
        State(state): State<Arc<OAuthState>>,
        Json(body): Json<HashMap<String, serde_json::Value>>,
    ) -> Json<serde_json::Value> {
        assert_eq!(body["client_id"], "test-client");
        assert_eq!(body["grant_type"], "refresh_token");
        assert!(body["refresh_token"].as_str().is_some());
        state.token_calls.fetch_add(1, Ordering::SeqCst);
        let access = jwt_for_test(serde_json::json!({
            "exp": (Utc::now() + Duration::hours(1)).timestamp(),
            "https://api.openai.com/auth": {
                "chatgpt_account_id": state.account_id.as_str()
            }
        }));
        let id = jwt_for_test(serde_json::json!({
            "https://api.openai.com/auth": {
                "chatgpt_account_id": state.account_id.as_str()
            }
        }));
        Json(serde_json::json!({
            "access_token": access,
            "id_token": id,
            "refresh_token": "refresh-rotated",
            "expires_in": 3600
        }))
    }

    async fn revoke_handler(
        State(state): State<Arc<OAuthState>>,
        Json(body): Json<HashMap<String, serde_json::Value>>,
    ) -> Json<serde_json::Value> {
        assert!(
            body["token"]
                .as_str()
                .is_some_and(|token| !token.is_empty())
        );
        let hint = body["token_type_hint"].as_str().unwrap().to_owned();
        if hint == "refresh_token" {
            assert_eq!(body["client_id"], "test-client");
        } else {
            assert!(body.get("client_id").is_none());
        }
        state.revoke_hints.lock().unwrap().push(hint);
        Json(serde_json::json!({}))
    }

    async fn mock_oauth(account_id: &str) -> (Url, Arc<OAuthState>, tokio::task::JoinHandle<()>) {
        let state = Arc::new(OAuthState {
            token_calls: AtomicUsize::new(0),
            revoke_hints: Mutex::new(Vec::new()),
            account_id: account_id.to_owned(),
        });
        let app = Router::new()
            .route("/oauth/token", post(refresh_handler))
            .route("/oauth/revoke", post(revoke_handler))
            .with_state(state.clone());
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (
            Url::parse(&format!("http://{address}/")).unwrap(),
            state,
            server,
        )
    }

    async fn manager_with_credentials(
        issuer: Url,
        credentials: CodexCredentials,
    ) -> (tempfile::TempDir, Arc<CodexAuthManager>) {
        let dir = tempfile::tempdir().unwrap();
        let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
        store.save(credentials).await.unwrap();
        let manager = Arc::new(
            CodexAuthManager::for_test(store, CodexOAuthClient::for_test(issuer)).unwrap(),
        );
        (dir, manager)
    }

    #[tokio::test]
    async fn concurrent_managers_rotate_refresh_token_once() {
        let (issuer, state, server) = mock_oauth("acct-one").await;
        let dir = tempfile::tempdir().unwrap();
        let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
        store
            .save(credentials_for_test(
                "acct-one",
                "refresh-old",
                Utc::now() - Duration::minutes(1),
            ))
            .await
            .unwrap();
        let first = Arc::new(
            CodexAuthManager::for_test(store.clone(), CodexOAuthClient::for_test(issuer.clone()))
                .unwrap(),
        );
        let second = Arc::new(
            CodexAuthManager::for_test(store.clone(), CodexOAuthClient::for_test(issuer)).unwrap(),
        );

        let (first_result, second_result) =
            tokio::join!(first.ensure_fresh(), second.ensure_fresh());
        let first_result = first_result.unwrap();
        let second_result = second_result.unwrap();
        assert_eq!(state.token_calls.load(Ordering::SeqCst), 1);
        assert_eq!(first_result.refresh_token(), "refresh-rotated");
        assert_eq!(second_result.refresh_token(), "refresh-rotated");
        assert_eq!(first_result.revision, 2);
        assert_eq!(second_result.revision, 2);
        assert_eq!(first_result.credential_id(), second_result.credential_id());

        server.abort();
    }

    #[tokio::test]
    async fn concurrent_unauthorized_recoveries_on_one_manager_rotate_once() {
        let (issuer, state, server) = mock_oauth("acct-one").await;
        let original =
            credentials_for_test("acct-one", "refresh-old", Utc::now() + Duration::hours(1));
        let rejected = original.credential_binding();
        let (_dir, manager) = manager_with_credentials(issuer, original).await;

        let (first, second) = tokio::join!(
            manager.recover_after_unauthorized(rejected.clone()),
            manager.recover_after_unauthorized(rejected)
        );

        assert!(first.unwrap());
        assert!(second.unwrap());
        assert_eq!(state.token_calls.load(Ordering::SeqCst), 1);
        let current = manager.current().unwrap();
        assert_eq!(current.refresh_token(), "refresh-rotated");
        assert_eq!(current.revision, 2);

        server.abort();
    }

    #[tokio::test]
    async fn stale_unauthorized_recovery_adopts_expired_sibling_without_rotating() {
        let (issuer, state, server) = mock_oauth("acct-one").await;
        let original =
            credentials_for_test("acct-one", "refresh-old", Utc::now() + Duration::hours(1));
        let original_binding = original.credential_binding();
        let (_dir, manager) = manager_with_credentials(issuer, original.clone()).await;

        let mut sibling = credentials_for_test(
            "acct-one",
            "refresh-sibling",
            Utc::now() - Duration::minutes(1),
        );
        sibling.revision = original.revision + 1;
        sibling.credential_id = original.credential_id.clone();
        manager.store.save(sibling.clone()).await.unwrap();

        assert!(
            manager
                .recover_after_unauthorized(original_binding.clone())
                .await
                .unwrap()
        );
        assert_eq!(state.token_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            manager.current().unwrap().refresh_token(),
            "refresh-sibling"
        );

        // A second late 401 from the old generation must remain adoption-only,
        // even though the adopted access token is already near expiry.
        assert!(
            manager
                .recover_after_unauthorized(original_binding)
                .await
                .unwrap()
        );
        assert_eq!(state.token_calls.load(Ordering::SeqCst), 0);

        // A request actually signed by the adopted generation may spend its
        // refresh token once when that generation itself receives a 401.
        assert!(
            manager
                .recover_after_unauthorized(sibling.credential_binding())
                .await
                .unwrap()
        );
        assert_eq!(state.token_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            manager.current().unwrap().refresh_token(),
            "refresh-rotated"
        );

        server.abort();
    }

    #[tokio::test]
    async fn unauthorized_recovery_fails_closed_after_account_switch() {
        let (issuer, state, server) = mock_oauth("acct-two").await;
        let original =
            credentials_for_test("acct-one", "refresh-old", Utc::now() + Duration::hours(1));
        let rejected = original.credential_binding();
        let (_dir, manager) = manager_with_credentials(issuer, original).await;

        let switched =
            credentials_for_test("acct-two", "refresh-new", Utc::now() + Duration::hours(1));
        manager.store.save(switched.clone()).await.unwrap();

        assert!(matches!(
            manager.recover_after_unauthorized(rejected).await,
            Err(CodexAuthError::AccountChanged)
        ));
        assert_eq!(state.token_calls.load(Ordering::SeqCst), 0);
        assert_eq!(manager.current().unwrap().account_id(), "acct-one");
        assert_eq!(manager.store.load().unwrap().unwrap(), switched);

        server.abort();
    }

    #[tokio::test]
    async fn active_request_hooks_fail_closed_immediately_after_logout() {
        let (issuer, _state, server) = mock_oauth("acct-one").await;
        let original =
            credentials_for_test("acct-one", "refresh-old", Utc::now() + Duration::hours(1));
        let (_dir, manager) = manager_with_credentials(issuer, original).await;
        assert!(manager.store.remove().await.unwrap().is_some());

        assert!(matches!(
            manager.current_verified(),
            Err(CodexAuthError::NotLoggedIn)
        ));
        let sampler = CodexSamplerRequestAuth(manager.clone());
        assert!(
            sampler
                .apply(&mut reqwest::header::HeaderMap::new())
                .is_err()
        );
        assert!(manager.request_auth().await.is_err());

        server.abort();
    }

    #[tokio::test]
    async fn active_request_hooks_reject_account_switch_but_adopt_same_record_rotation() {
        let (issuer, _state, server) = mock_oauth("acct-one").await;
        let original =
            credentials_for_test("acct-one", "refresh-old", Utc::now() + Duration::hours(1));
        let (_dir, manager) = manager_with_credentials(issuer, original.clone()).await;

        let mut rotated = original.clone();
        rotated.revision += 1;
        rotated.access_token = crate::auth::codex::SecretString::new("access-rotated").unwrap();
        manager.store.save(rotated.clone()).await.unwrap();
        assert_eq!(
            manager.current_verified().unwrap().revision,
            rotated.revision
        );

        let switched =
            credentials_for_test("acct-two", "refresh-new", Utc::now() + Duration::hours(1));
        manager.store.save(switched).await.unwrap();
        assert!(matches!(
            manager.current_verified(),
            Err(CodexAuthError::AccountChanged)
        ));
        let sampler = CodexSamplerRequestAuth(manager.clone());
        assert!(
            sampler
                .apply(&mut reqwest::header::HeaderMap::new())
                .is_err()
        );
        assert!(matches!(
            manager.request_auth().await,
            Err(CodexAuthError::AccountChanged)
        ));

        server.abort();
    }

    #[tokio::test]
    async fn tool_sampler_and_catalog_auth_are_scoped_and_redacted() {
        let (issuer, _state, server) = mock_oauth("acct-one").await;
        let mut credentials =
            credentials_for_test("acct-one", "refresh-old", Utc::now() + Duration::hours(1));
        credentials.is_fedramp = true;
        let expected_access = credentials.access_token().to_owned();
        let (_dir, manager) = manager_with_credentials(issuer, credentials).await;

        let provider = CodexApiKeyProvider(manager.clone());
        assert!(provider.current_api_key().is_none());
        assert!(provider.current_api_key_async().await.is_none());
        let request_auth = provider.current_request_auth_async().await.unwrap();
        assert_eq!(request_auth.provider(), Some(OPENAI_CODEX_PROVIDER_ID));
        let tool_snapshot = request_auth.credential_snapshot().unwrap();
        assert!(!tool_snapshot.opaque_id().is_empty());
        assert_eq!(tool_snapshot.generation(), 1);
        let headers: HashMap<_, _> = request_auth.headers().collect();
        let expected_authorization = format!("Bearer {expected_access}");
        assert_eq!(
            headers.get("Authorization"),
            Some(&expected_authorization.as_str())
        );
        assert_eq!(headers.get("ChatGPT-Account-ID"), Some(&"acct-one"));
        assert_eq!(headers.get("X-OpenAI-Fedramp"), Some(&"true"));
        assert!(
            !headers
                .keys()
                .any(|name| name.to_ascii_lowercase().starts_with("x-xai-"))
        );
        let tool_debug = format!("{request_auth:?}");
        assert!(!tool_debug.contains(&expected_access));
        assert!(!tool_debug.contains("acct-one"));

        let sampler = CodexSamplerRequestAuth(manager.clone());
        let mut sampler_headers = reqwest::header::HeaderMap::new();
        let sampler_binding = sampler.apply(&mut sampler_headers).unwrap();
        assert_eq!(
            sampler_binding.provider,
            xai_grok_sampling_types::ProviderId::OpenAiCodex
        );
        assert_eq!(
            sampler_binding.source,
            xai_grok_sampling_types::CredentialSourceId::OpenAiCodexSubscription
        );
        assert_eq!(sampler_binding.generation, 1);
        assert!(sampler_binding.record_id.is_some());
        assert_eq!(
            sampler_headers[reqwest::header::AUTHORIZATION],
            format!("Bearer {expected_access}")
        );
        assert_eq!(sampler_headers["chatgpt-account-id"], "acct-one");
        assert_eq!(sampler_headers["x-openai-fedramp"], "true");
        assert!(sampler_headers[reqwest::header::AUTHORIZATION].is_sensitive());
        assert!(sampler_headers["chatgpt-account-id"].is_sensitive());
        assert!(sampler_headers["x-openai-fedramp"].is_sensitive());
        assert!(sampler_headers.get("x-xai-token-auth").is_none());

        let catalog = manager.catalog_auth().await.unwrap();
        let catalog_debug = format!("{catalog:?}");
        assert!(!catalog_debug.contains(&expected_access));
        assert!(!catalog_debug.contains("acct-one"));

        server.abort();
    }

    #[tokio::test]
    async fn logout_revokes_refresh_and_access_then_removes_only_codex() {
        let (issuer, state, server) = mock_oauth("acct-one").await;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let store = CodexCredentialStore::from_auth_path(path.clone());
        let mut auth_store = crate::auth::model::AuthStore::new();
        auth_store.insert("xai::api_key".to_owned(), GrokAuth::test_default());
        crate::auth::storage::write_auth_json(&path, &auth_store).unwrap();
        store
            .save(credentials_for_test(
                "acct-one",
                "refresh-old",
                Utc::now() + Duration::hours(1),
            ))
            .await
            .unwrap();
        let manager =
            CodexAuthManager::for_test(store, CodexOAuthClient::for_test(issuer)).unwrap();

        let result = manager.logout().await.unwrap();
        assert_eq!(
            result,
            CodexLogoutResult {
                removed: true,
                revoke_succeeded: true
            }
        );
        assert_eq!(
            state.revoke_hints.lock().unwrap().clone(),
            vec!["refresh_token".to_owned(), "access_token".to_owned()]
        );
        let remaining = crate::auth::storage::read_auth_json(&path).unwrap();
        assert!(remaining.get("xai::api_key").is_some());
        assert!(remaining.get_codex().is_none());

        server.abort();
    }
}
