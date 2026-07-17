use std::fmt;
use std::path::Path;

use chrono::Utc;
use parking_lot::RwLock;

use super::{CodexAuthError, CodexCredentialStore, CodexCredentials, CodexOAuthClient};

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

/// Cross-process lease proving that one usage result is still bound to the
/// same credential record at a non-regressing generation. The usage cache
/// holds this through its final synchronous publication/return seam.
pub(super) struct CodexUsageIdentityLease {
    _provider_lock: super::storage::CodexProviderLock,
}

impl fmt::Debug for CodexAuthManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexAuthManager").finish_non_exhaustive()
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

    /// Lock and attest a usage result immediately before it is returned or
    /// published. Same-record refresh generations may advance, but logout,
    /// record replacement, identity change, and generation regression fail
    /// while the provider writer gate is held.
    pub(super) async fn lock_usage_binding(
        &self,
        expected: &xai_grok_sampling_types::CredentialBinding,
    ) -> Result<CodexUsageIdentityLease, CodexAuthError> {
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
        let actual = disk.credential_binding();
        if !expected.same_record(&actual) || actual.generation < expected.generation {
            return Err(CodexAuthError::AccountChanged);
        }
        if let Some(cached) = self.current()
            && (disk.credential_id() != cached.credential_id()
                || disk.revision < cached.revision
                || !disk.same_identity(&cached))
        {
            return Err(CodexAuthError::AccountChanged);
        }
        *self.current.write() = Some(disk);
        Ok(CodexUsageIdentityLease {
            _provider_lock: provider_lock,
        })
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodexLogoutResult {
    pub removed: bool,
    pub revoke_succeeded: bool,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use axum::Router;
    use axum::extract::{Json, State};
    use axum::routing::post;
    use chrono::{Duration, Utc};
    use tokio::net::TcpListener;
    use url::Url;

    use super::*;
    use crate::auth::GrokAuth;
    use crate::auth::codex::credentials::{credentials_for_test, jwt_for_test};

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
    async fn current_record_fails_closed_immediately_after_logout() {
        let (issuer, _state, server) = mock_oauth("acct-one").await;
        let original =
            credentials_for_test("acct-one", "refresh-old", Utc::now() + Duration::hours(1));
        let (_dir, manager) = manager_with_credentials(issuer, original).await;
        assert!(manager.store.remove().await.unwrap().is_some());

        assert!(matches!(
            manager.current_verified(),
            Err(CodexAuthError::NotLoggedIn)
        ));

        server.abort();
    }

    #[tokio::test]
    async fn current_record_rejects_account_switch_but_adopts_same_record_rotation() {
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
