use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use axum::Router;
use axum::extract::State;
use axum::http::header::{HeaderName, LOCATION};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use chrono::{Duration, Utc};
use serde_json::json;
use tokio::net::TcpListener;

use super::*;
use crate::auth::codex::CodexCredentialStore;
use crate::auth::codex::credentials::credentials_for_test;

#[derive(Clone)]
struct MockState {
    requests: Arc<parking_lot::Mutex<Vec<HeaderMap>>>,
    reject_first: bool,
    response_status: StatusCode,
}

async fn usage_fixture(State(state): State<MockState>, headers: HeaderMap) -> impl IntoResponse {
    let request_number = {
        let mut requests = state.requests.lock();
        requests.push(headers.clone());
        requests.len()
    };
    if state.reject_first && request_number == 1 {
        return (StatusCode::UNAUTHORIZED, "discarded-sensitive-error").into_response();
    }
    if !state.response_status.is_success() {
        return (state.response_status, "discarded-sensitive-error").into_response();
    }

    axum::Json(json!({
        "plan_type": "pro",
        "rate_limit": {
            "allowed": true,
            "limit_reached": false,
            "primary_window": {
                "used_percent": 42,
                "limit_window_seconds": 18000,
                "reset_after_seconds": 120,
                "reset_at": 1735689720
            },
            "secondary_window": {
                "used_percent": 5,
                "limit_window_seconds": 604800,
                "reset_after_seconds": 43200,
                "reset_at": 1735736400
            }
        },
        "credits": {
            "has_credits": true,
            "unlimited": false,
            "balance": "12.50",
            "approx_local_messages": ["must-not-survive"]
        },
        "spend_control": {
            "reached": false,
            "individual_limit": {
                "source": "workspace_spend_controls",
                "limit": "25000",
                "used": "8000",
                "remaining": "17000",
                "used_percent": 32,
                "remaining_percent": 68,
                "reset_after_seconds": 43200,
                "reset_at": 1735736400
            }
        },
        "additional_rate_limits": [{
            "limit_name": "codex_other",
            "metered_feature": "codex_other",
            "rate_limit": {
                "allowed": true,
                "limit_reached": false,
                "primary_window": {
                    "used_percent": 88,
                    "limit_window_seconds": 1800,
                    "reset_after_seconds": 600,
                    "reset_at": 1735693200
                }
            }
        }],
        "rate_limit_reached_type": {
            "type": "workspace_member_usage_limit_reached"
        },
        "rate_limit_reset_credits": { "available_count": 3 }
    }))
    .into_response()
}

struct FakeAuth {
    auth_calls: AtomicUsize,
    recoveries: AtomicUsize,
}

impl FakeAuth {
    fn new() -> Self {
        Self {
            auth_calls: AtomicUsize::new(0),
            recoveries: AtomicUsize::new(0),
        }
    }
}

fn binding(record_id: &str, generation: u64) -> CredentialBinding {
    let mut binding = CredentialBinding::openai_codex(Some(record_id.to_owned()));
    binding.generation = generation;
    binding
}

fn cached_usage(binding: CredentialBinding, age: std::time::Duration) -> CodexUsageCache {
    CodexUsageCache {
        entry: Some(CachedUsage {
            binding,
            fetched_at: std::time::Instant::now()
                .checked_sub(age)
                .expect("cache fixture age must fit"),
            snapshot: CodexUsageSnapshot {
                plan_type: Some("cached-plan".to_owned()),
                rate_limit: None,
                credits: None,
                spend_control: None,
                additional_rate_limits: Vec::new(),
                rate_limit_reached_type: None,
                rate_limit_reset_credits: None,
            },
        }),
        ..CodexUsageCache::default()
    }
}

#[async_trait]
impl UsageAuthProvider for FakeAuth {
    type IdentityLease = ();

    async fn auth_snapshot(&self) -> Result<CodexAuthSnapshot, CodexUsageError> {
        let generation = self.auth_calls.fetch_add(1, Ordering::SeqCst) as u64 + 1;
        CodexAuthSnapshot::new(
            &format!("token-{generation}"),
            "account-id",
            None,
            binding("record-id", generation),
            true,
        )
        .map_err(|_| CodexUsageError::InvalidAuth)
    }

    async fn recover_unauthorized(
        &self,
        rejected: CredentialBinding,
    ) -> Result<bool, CodexUsageError> {
        assert!(rejected == binding("record-id", 1));
        self.recoveries.fetch_add(1, Ordering::SeqCst);
        Ok(true)
    }

    async fn attest_binding(
        &self,
        expected: &CredentialBinding,
    ) -> Result<Self::IdentityLease, CodexUsageError> {
        validate_binding(expected)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum DiskTransition {
    Logout,
    Replace,
}

async fn apply_disk_transition(store: &CodexCredentialStore, transition: DiskTransition) {
    match transition {
        DiskTransition::Logout => {
            assert!(store.remove().await.unwrap().is_some());
        }
        DiskTransition::Replace => {
            store
                .save(credentials_for_test(
                    "replacement-account",
                    "replacement-refresh",
                    Utc::now() + Duration::hours(1),
                ))
                .await
                .unwrap();
        }
    }
}

#[derive(Clone)]
struct PausingManagerAuth {
    manager: Arc<CodexAuthManager>,
    attestation_reached: Arc<tokio::sync::Barrier>,
    resume_attestation: Arc<tokio::sync::Barrier>,
}

#[async_trait]
impl UsageAuthProvider for PausingManagerAuth {
    type IdentityLease = <CodexAuthManager as UsageAuthProvider>::IdentityLease;

    async fn auth_snapshot(&self) -> Result<CodexAuthSnapshot, CodexUsageError> {
        Ok(self.manager.auth_snapshot().await?)
    }

    async fn recover_unauthorized(
        &self,
        rejected: CredentialBinding,
    ) -> Result<bool, CodexUsageError> {
        Ok(self.manager.recover_after_unauthorized(rejected).await?)
    }

    async fn attest_binding(
        &self,
        expected: &CredentialBinding,
    ) -> Result<Self::IdentityLease, CodexUsageError> {
        self.attestation_reached.wait().await;
        self.resume_attestation.wait().await;
        Ok(self.manager.lock_usage_binding(expected).await?)
    }
}

#[derive(Clone, Copy)]
enum RecoveryFailure {
    Transport,
    Service,
}

struct RecoveryFailingAuth {
    failure: RecoveryFailure,
}

#[async_trait]
impl UsageAuthProvider for RecoveryFailingAuth {
    type IdentityLease = ();

    async fn auth_snapshot(&self) -> Result<CodexAuthSnapshot, CodexUsageError> {
        CodexAuthSnapshot::new(
            "rejected-token",
            "rejected-account",
            None,
            binding("record-id", 1),
            false,
        )
        .map_err(|_| CodexUsageError::InvalidAuth)
    }

    async fn recover_unauthorized(
        &self,
        _rejected: CredentialBinding,
    ) -> Result<bool, CodexUsageError> {
        match self.failure {
            RecoveryFailure::Transport => {
                let unavailable = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
                    .await
                    .unwrap();
                let address = unavailable.local_addr().unwrap();
                drop(unavailable);
                let source = reqwest::Client::new()
                    .get(format!("http://{address}/recovery"))
                    .send()
                    .await
                    .expect_err("released loopback port must reject recovery");
                Err(CodexAuthError::Transport(source).into())
            }
            RecoveryFailure::Service => Err(CodexAuthError::HttpStatus(503).into()),
        }
    }

    async fn attest_binding(
        &self,
        expected: &CredentialBinding,
    ) -> Result<Self::IdentityLease, CodexUsageError> {
        validate_binding(expected)?;
        Ok(())
    }
}

async fn mock_server(state: MockState) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/backend-api/wham/usage", get(usage_fixture))
        .with_state(state);
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{address}/backend-api/wham/usage"), task)
}

#[derive(Clone)]
struct PausedUsageState {
    requests: Arc<AtomicUsize>,
    request_reached: Arc<tokio::sync::Barrier>,
    release_response: Arc<tokio::sync::Barrier>,
}

async fn paused_usage_fixture(State(state): State<PausedUsageState>) -> impl IntoResponse {
    state.requests.fetch_add(1, Ordering::SeqCst);
    state.request_reached.wait().await;
    state.release_response.wait().await;
    axum::Json(json!({ "plan_type": "network-plan" }))
}

async fn paused_usage_server(state: PausedUsageState) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/backend-api/wham/usage", get(paused_usage_fixture))
        .with_state(state);
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{address}/backend-api/wham/usage"), task)
}

#[derive(Clone)]
struct RedirectState {
    requests: Arc<parking_lot::Mutex<Vec<HeaderMap>>>,
    location: HeaderValue,
}

async fn redirect_fixture(
    State(state): State<RedirectState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    state.requests.lock().push(headers);
    let mut response = StatusCode::TEMPORARY_REDIRECT.into_response();
    response.headers_mut().insert(LOCATION, state.location);
    response
}

#[tokio::test]
async fn exact_usage_path_headers_schema_and_single_codex_retry() {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: true,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();

    let usage = fetch_from_url(&reqwest::Client::new(), &auth, &url)
        .await
        .unwrap();
    task.abort();

    assert_eq!(auth.recoveries.load(Ordering::SeqCst), 1);
    assert_eq!(auth.auth_calls.load(Ordering::SeqCst), 2);
    let requests = requests.lock();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["authorization"], "Bearer token-1");
    assert_eq!(requests[1]["authorization"], "Bearer token-2");
    assert_eq!(requests[1]["chatgpt-account-id"], "account-id");
    assert_eq!(requests[1]["x-openai-fedramp"], "true");
    for headers in requests.iter() {
        assert!(!headers.contains_key("x-xai-token-auth"));
        assert!(!headers.contains_key("x-userid"));
    }

    assert_eq!(usage.plan_type.as_deref(), Some("pro"));
    let limit = usage.rate_limit.as_ref().unwrap();
    assert_eq!(
        limit.primary_window.as_ref().unwrap().limit_window_seconds,
        18_000
    );
    assert_eq!(
        limit
            .secondary_window
            .as_ref()
            .unwrap()
            .limit_window_seconds,
        604_800
    );
    assert_eq!(
        limit.primary_window.as_ref().unwrap().reset_after_seconds,
        120
    );
    assert_eq!(
        limit.secondary_window.as_ref().unwrap().reset_at,
        1_735_736_400
    );
    assert_eq!(
        usage.credits.as_ref().unwrap().balance.as_deref(),
        Some("12.50")
    );
    assert_eq!(
        usage
            .rate_limit_reset_credits
            .as_ref()
            .unwrap()
            .available_count,
        3
    );
    assert_eq!(usage.additional_rate_limits.len(), 1);
    assert_eq!(usage.highest_used_percent(), Some(88.0));
    let sanitized = serde_json::to_string(&usage).unwrap();
    assert!(!sanitized.contains("must-not-survive"));
    assert!(!sanitized.contains("approx_local_messages"));
}

#[tokio::test]
async fn usage_client_never_replays_credentials_to_a_redirect_target() {
    let target_requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let target_state = MockState {
        requests: target_requests.clone(),
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let target_listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let target_address = target_listener.local_addr().unwrap();
    let target = Router::new()
        .route("/credential-sink", get(usage_fixture))
        .with_state(target_state);
    let target_task = tokio::spawn(async move {
        axum::serve(target_listener, target).await.unwrap();
    });

    let source_requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let source_state = RedirectState {
        requests: source_requests.clone(),
        location: HeaderValue::from_str(&format!("http://{target_address}/credential-sink"))
            .unwrap(),
    };
    let source_listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let source_address = source_listener.local_addr().unwrap();
    let source = Router::new()
        .route("/backend-api/wham/usage", get(redirect_fixture))
        .with_state(source_state);
    let source_task = tokio::spawn(async move {
        axum::serve(source_listener, source).await.unwrap();
    });

    let auth = FakeAuth::new();
    let source_url = format!("http://{source_address}/backend-api/wham/usage");
    let error = fetch_from_url(&codex_usage_http_client(), &auth, &source_url)
        .await
        .expect_err("a Codex usage redirect must be terminal");
    source_task.abort();
    target_task.abort();

    assert!(matches!(error, CodexUsageError::HttpStatus(307)));
    assert_eq!(auth.auth_calls.load(Ordering::SeqCst), 1);
    assert_eq!(auth.recoveries.load(Ordering::SeqCst), 0);
    let source_requests = source_requests.lock();
    assert_eq!(source_requests.len(), 1);
    for name in ["authorization", "chatgpt-account-id", "x-openai-fedramp"] {
        assert!(
            source_requests[0].contains_key(name),
            "source request omitted {name}"
        );
    }
    assert!(
        target_requests.lock().is_empty(),
        "redirect target must receive no Authorization/account/FedRAMP request"
    );
}

#[tokio::test]
async fn upstream_error_body_is_never_exposed() {
    let state = MockState {
        requests: Arc::new(parking_lot::Mutex::new(Vec::new())),
        reject_first: false,
        response_status: StatusCode::INTERNAL_SERVER_ERROR,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();

    let error = fetch_from_url(&reqwest::Client::new(), &auth, &url)
        .await
        .unwrap_err();
    task.abort();

    let rendered = error.to_string();
    assert_eq!(rendered, "OpenAI Codex usage request returned HTTP 500");
    assert!(!rendered.contains("discarded-sensitive-error"));
}

#[test]
fn usage_headers_come_only_from_the_common_sensitive_snapshot() {
    let snapshot = CodexAuthSnapshot::new(
        "sentinel-access-token",
        "sentinel-account-id",
        None,
        binding("record-id", 1),
        true,
    )
    .unwrap();
    let headers = request_headers(&snapshot).unwrap();
    for name in [
        reqwest::header::AUTHORIZATION,
        HeaderName::from_static("chatgpt-account-id"),
        HeaderName::from_static("x-openai-fedramp"),
    ] {
        assert!(headers[&name].is_sensitive(), "{name} must be redacted");
    }
    assert!(headers.get("x-xai-token-auth").is_none());
    assert!(headers.get("x-userid").is_none());
    let rendered = format!("{headers:?}");
    assert!(!rendered.contains("sentinel-access-token"));
    assert!(!rendered.contains("sentinel-account-id"));
}

#[tokio::test]
async fn bound_usage_rejects_account_switch_before_http() {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();

    let error = fetch_from_url_with_binding(
        &reqwest::Client::new(),
        &auth,
        &url,
        Some(&binding("different-record", 1)),
    )
    .await
    .unwrap_err();
    task.abort();

    assert!(matches!(
        error,
        CodexUsageError::Auth(CodexAuthError::AccountChanged)
    ));
    assert_eq!(auth.auth_calls.load(Ordering::SeqCst), 1);
    assert!(
        requests.lock().is_empty(),
        "no switched-account HTTP request"
    );
}

#[tokio::test]
async fn no_session_usage_snapshots_current_record_and_rejects_account_switch() {
    let dir = tempfile::tempdir().unwrap();
    let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
    let first_credentials = credentials_for_test(
        "account-one",
        "refresh-one",
        Utc::now() + Duration::hours(1),
    );
    let first_binding = first_credentials.credential_binding();
    store.save(first_credentials).await.unwrap();
    let manager = CodexAuthManager::from_store(store.clone()).unwrap();

    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let cache = tokio::sync::Mutex::new(CodexUsageCache::default());
    let client = reqwest::Client::new();

    fetch_codex_usage_for_current_from_url(&client, &manager, &url, false, &cache)
        .await
        .unwrap();
    assert_eq!(
        cache.lock().await.entry.as_ref().unwrap().binding,
        first_binding
    );
    assert_eq!(requests.lock().len(), 1);

    store
        .save(credentials_for_test(
            "account-two",
            "refresh-two",
            Utc::now() + Duration::hours(1),
        ))
        .await
        .unwrap();
    let switched = fetch_codex_usage_for_current_from_url(&client, &manager, &url, false, &cache)
        .await
        .unwrap_err();
    task.abort();

    assert!(matches!(
        switched,
        CodexUsageError::Auth(CodexAuthError::AccountChanged)
    ));
    assert_eq!(
        requests.lock().len(),
        1,
        "a switched account must be rejected before cache or HTTP access"
    );
    assert!(
        cache.lock().await.entry.is_none(),
        "an account failure must evict prior-record usage"
    );
}

async fn assert_session_cache_rechecks_disk(stale: bool, replace_record: bool) {
    let dir = tempfile::tempdir().unwrap();
    let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
    let credentials = credentials_for_test(
        "session-account",
        "session-refresh",
        Utc::now() + Duration::hours(1),
    );
    let expected = credentials.credential_binding();
    store.save(credentials).await.unwrap();
    let manager = CodexAuthManager::from_store(store.clone()).unwrap();

    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let age = if stale {
        USAGE_CACHE_TTL + std::time::Duration::from_secs(1)
    } else {
        std::time::Duration::ZERO
    };
    let cache = tokio::sync::Mutex::new(cached_usage(expected.clone(), age));

    if replace_record {
        store
            .save(credentials_for_test(
                "replacement-account",
                "replacement-refresh",
                Utc::now() + Duration::hours(1),
            ))
            .await
            .unwrap();
    } else {
        assert!(store.remove().await.unwrap().is_some());
    }

    let error = fetch_codex_usage_for_session_from_url(
        &reqwest::Client::new(),
        &manager,
        &url,
        &expected,
        true,
        &cache,
    )
    .await
    .expect_err("a disk credential change must reject cached session usage");
    task.abort();

    if replace_record {
        assert!(matches!(
            error,
            CodexUsageError::Auth(CodexAuthError::AccountChanged)
        ));
    } else {
        assert!(matches!(
            error,
            CodexUsageError::Auth(CodexAuthError::NotLoggedIn)
        ));
    }
    assert!(
        requests.lock().is_empty(),
        "disk verification must fail before any usage request"
    );
    assert!(
        cache.lock().await.entry.is_none(),
        "prior-record usage must be evicted after disk verification fails"
    );
}

#[tokio::test]
async fn active_session_fresh_cache_rechecks_disk_before_returning() {
    assert_session_cache_rechecks_disk(false, false).await;
    assert_session_cache_rechecks_disk(false, true).await;
}

#[tokio::test]
async fn active_session_stale_cache_rechecks_disk_before_fallback() {
    assert_session_cache_rechecks_disk(true, false).await;
    assert_session_cache_rechecks_disk(true, true).await;
}

async fn assert_fresh_cache_transition_is_linearized(transition: DiskTransition) {
    let dir = tempfile::tempdir().unwrap();
    let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
    let credentials = credentials_for_test(
        "session-account",
        "session-refresh",
        Utc::now() + Duration::hours(1),
    );
    let expected = credentials.credential_binding();
    store.save(credentials).await.unwrap();
    let manager = Arc::new(CodexAuthManager::from_store(store.clone()).unwrap());

    // This is the active-session precheck. The test then pauses the final
    // cache-return attestation so the disk transition deterministically wins.
    assert!(manager.current_verified().is_ok());
    let attestation_reached = Arc::new(tokio::sync::Barrier::new(2));
    let resume_attestation = Arc::new(tokio::sync::Barrier::new(2));
    let auth = PausingManagerAuth {
        manager,
        attestation_reached: attestation_reached.clone(),
        resume_attestation: resume_attestation.clone(),
    };
    let cache = Arc::new(tokio::sync::Mutex::new(cached_usage(
        expected.clone(),
        std::time::Duration::ZERO,
    )));
    let fetch_cache = cache.clone();
    let fetch = tokio::spawn(async move {
        let client = reqwest::Client::new();
        fetch_cached_from_url(
            &client,
            &auth,
            "http://127.0.0.1:9/backend-api/wham/usage",
            &expected,
            UsageFetchMode::Silent,
            &fetch_cache,
        )
        .await
    });

    attestation_reached.wait().await;
    apply_disk_transition(&store, transition).await;
    resume_attestation.wait().await;
    let error = fetch
        .await
        .unwrap()
        .expect_err("a transition that wins before cache attestation must reject usage");

    match transition {
        DiskTransition::Logout => assert!(matches!(
            error,
            CodexUsageError::Auth(CodexAuthError::NotLoggedIn)
        )),
        DiskTransition::Replace => assert!(matches!(
            error,
            CodexUsageError::Auth(CodexAuthError::AccountChanged)
        )),
    }
    assert!(cache.lock().await.entry.is_none());
}

#[tokio::test]
async fn fresh_cache_return_is_linearized_against_logout_and_replacement() {
    for transition in [DiskTransition::Logout, DiskTransition::Replace] {
        assert_fresh_cache_transition_is_linearized(transition).await;
    }
}

async fn assert_in_flight_result_transition_is_linearized(transition: DiskTransition) {
    let dir = tempfile::tempdir().unwrap();
    let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
    let credentials = credentials_for_test(
        "session-account",
        "session-refresh",
        Utc::now() + Duration::hours(1),
    );
    let expected = credentials.credential_binding();
    store.save(credentials).await.unwrap();
    let manager = Arc::new(CodexAuthManager::from_store(store.clone()).unwrap());

    let state = PausedUsageState {
        requests: Arc::new(AtomicUsize::new(0)),
        request_reached: Arc::new(tokio::sync::Barrier::new(2)),
        release_response: Arc::new(tokio::sync::Barrier::new(2)),
    };
    let (url, server) = paused_usage_server(state.clone()).await;
    let cache = Arc::new(tokio::sync::Mutex::new(CodexUsageCache::default()));
    let fetch_cache = cache.clone();
    let fetch_manager = manager.clone();
    let fetch = tokio::spawn(async move {
        let client = reqwest::Client::new();
        fetch_codex_usage_for_session_from_url(
            &client,
            &fetch_manager,
            &url,
            &expected,
            false,
            &fetch_cache,
        )
        .await
    });

    state.request_reached.wait().await;
    apply_disk_transition(&store, transition).await;
    state.release_response.wait().await;
    let error = fetch
        .await
        .unwrap()
        .expect_err("a transition that wins during HTTP must reject the old result");
    server.abort();

    match transition {
        DiskTransition::Logout => assert!(matches!(
            error,
            CodexUsageError::Auth(CodexAuthError::NotLoggedIn)
        )),
        DiskTransition::Replace => assert!(matches!(
            error,
            CodexUsageError::Auth(CodexAuthError::AccountChanged)
        )),
    }
    assert_eq!(state.requests.load(Ordering::SeqCst), 1);
    assert!(cache.lock().await.entry.is_none());
}

#[tokio::test]
async fn in_flight_result_publication_is_linearized_against_logout_and_replacement() {
    for transition in [DiskTransition::Logout, DiskTransition::Replace] {
        assert_in_flight_result_transition_is_linearized(transition).await;
    }
}

#[tokio::test]
async fn fresh_usage_cache_deduplicates_concurrent_requests_for_same_record() {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();
    let cache = tokio::sync::Mutex::new(CodexUsageCache::default());
    let expected = binding("record-id", 1);
    let client = reqwest::Client::new();

    let first = fetch_cached_from_url(
        &client,
        &auth,
        &url,
        &expected,
        UsageFetchMode::Explicit,
        &cache,
    );
    let second = fetch_cached_from_url(
        &client,
        &auth,
        &url,
        &expected,
        UsageFetchMode::Silent,
        &cache,
    );
    let (first, second) = tokio::join!(first, second);
    let first = first.unwrap();
    let second = second.unwrap();
    task.abort();

    assert_eq!(first, second);
    assert_eq!(auth.auth_calls.load(Ordering::SeqCst), 1);
    assert_eq!(requests.lock().len(), 1);
}

#[tokio::test]
async fn usage_cache_never_crosses_credential_record_boundaries() {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();
    let cache = tokio::sync::Mutex::new(CodexUsageCache::default());
    let client = reqwest::Client::new();

    fetch_cached_from_url(
        &client,
        &auth,
        &url,
        &binding("record-id", 1),
        UsageFetchMode::Explicit,
        &cache,
    )
    .await
    .unwrap();
    let switched = fetch_cached_from_url(
        &client,
        &auth,
        &url,
        &binding("different-record", 1),
        UsageFetchMode::Silent,
        &cache,
    )
    .await
    .unwrap_err();
    task.abort();

    assert!(matches!(
        switched,
        CodexUsageError::Auth(CodexAuthError::AccountChanged)
    ));
    assert_eq!(
        requests.lock().len(),
        1,
        "the switched account must fail before another HTTP request"
    );
    assert!(
        cache.lock().await.entry.is_none(),
        "record A data must be evicted before record B is considered"
    );
}

#[tokio::test]
async fn invalid_binding_never_uses_or_retains_cached_usage() {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();
    let expected = binding("", 1);
    let cache = tokio::sync::Mutex::new(cached_usage(
        expected.clone(),
        USAGE_CACHE_TTL + std::time::Duration::from_secs(1),
    ));

    let error = fetch_cached_from_url(
        &reqwest::Client::new(),
        &auth,
        &url,
        &expected,
        UsageFetchMode::Silent,
        &cache,
    )
    .await
    .expect_err("an invalid binding must not receive stale usage");
    task.abort();

    assert!(matches!(
        error,
        CodexUsageError::Auth(CodexAuthError::AccountChanged)
    ));
    assert!(requests.lock().is_empty());
    assert!(cache.lock().await.entry.is_none());
}

#[tokio::test]
async fn transient_service_failure_can_use_same_record_stale_usage() {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::SERVICE_UNAVAILABLE,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();
    let expected = binding("record-id", 1);
    let cache = tokio::sync::Mutex::new(cached_usage(
        expected.clone(),
        USAGE_CACHE_TTL + std::time::Duration::from_secs(1),
    ));
    let expected_snapshot = cache.lock().await.entry.as_ref().unwrap().snapshot.clone();

    let stale = fetch_cached_from_url(
        &reqwest::Client::new(),
        &auth,
        &url,
        &expected,
        UsageFetchMode::Silent,
        &cache,
    )
    .await
    .unwrap();
    task.abort();

    assert_eq!(stale, expected_snapshot);
    assert_eq!(requests.lock().len(), 1);
    assert!(cache.lock().await.entry.is_some());
}

async fn assert_recovery_failure_preserves_rejection(failure: RecoveryFailure) {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests: requests.clone(),
        reject_first: false,
        response_status: StatusCode::UNAUTHORIZED,
    };
    let (url, task) = mock_server(state).await;
    let auth = RecoveryFailingAuth { failure };
    let expected = binding("record-id", 1);
    let cache = tokio::sync::Mutex::new(cached_usage(
        expected.clone(),
        USAGE_CACHE_TTL + std::time::Duration::from_secs(1),
    ));

    let error = fetch_cached_from_url(
        &reqwest::Client::new(),
        &auth,
        &url,
        &expected,
        UsageFetchMode::Silent,
        &cache,
    )
    .await
    .expect_err("failed recovery after rejection must never unlock stale usage");
    task.abort();

    assert!(matches!(error, CodexUsageError::CredentialRejected));
    assert!(!usage_error_allows_stale(&error));
    assert_eq!(requests.lock().len(), 1);
    assert!(cache.lock().await.entry.is_none());
}

#[tokio::test]
async fn usage_401_then_recovery_transport_never_returns_stale_usage() {
    assert_recovery_failure_preserves_rejection(RecoveryFailure::Transport).await;
}

#[tokio::test]
async fn usage_401_then_recovery_service_failure_never_returns_stale_usage() {
    assert_recovery_failure_preserves_rejection(RecoveryFailure::Service).await;
}

#[tokio::test]
async fn final_authorization_failures_never_use_or_retain_stale_usage() {
    for status in [StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN] {
        let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let state = MockState {
            requests: requests.clone(),
            reject_first: false,
            response_status: status,
        };
        let (url, task) = mock_server(state).await;
        let auth = FakeAuth::new();
        let expected = binding("record-id", 1);
        let cache = tokio::sync::Mutex::new(cached_usage(
            expected.clone(),
            USAGE_CACHE_TTL + std::time::Duration::from_secs(1),
        ));

        let error = fetch_cached_from_url(
            &reqwest::Client::new(),
            &auth,
            &url,
            &expected,
            UsageFetchMode::Silent,
            &cache,
        )
        .await
        .expect_err("an authorization failure must not receive stale usage");
        task.abort();

        if status == StatusCode::UNAUTHORIZED {
            assert!(matches!(error, CodexUsageError::CredentialRejected));
        } else {
            assert!(matches!(
                error,
                CodexUsageError::HttpStatus(code) if code == status.as_u16()
            ));
        }
        let expected_requests = if status == StatusCode::UNAUTHORIZED {
            2
        } else {
            1
        };
        assert_eq!(requests.lock().len(), expected_requests);
        assert!(cache.lock().await.entry.is_none());
    }
}

#[test]
fn invalid_auth_is_not_stale_eligible_and_invalidates_identity_cache() {
    let error = CodexUsageError::InvalidAuth;
    assert!(!usage_error_allows_stale(&error));
    assert!(usage_error_invalidates_cached_identity(&error));
}

#[tokio::test]
async fn silent_failure_uses_stale_snapshot_and_obeys_backoff() {
    let requests = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let state = MockState {
        requests,
        reject_first: false,
        response_status: StatusCode::OK,
    };
    let (url, task) = mock_server(state).await;
    let auth = FakeAuth::new();
    let cache = tokio::sync::Mutex::new(CodexUsageCache::default());
    let expected = binding("record-id", 1);
    let client = reqwest::Client::new();

    let good = fetch_cached_from_url(
        &client,
        &auth,
        &url,
        &expected,
        UsageFetchMode::Explicit,
        &cache,
    )
    .await
    .unwrap();
    {
        let mut state = cache.lock().await;
        state.entry.as_mut().unwrap().fetched_at =
            std::time::Instant::now() - USAGE_CACHE_TTL - std::time::Duration::from_secs(1);
    }
    // Reserve then release a loopback port so transport failure is
    // deterministic; aborting the working server alone races task shutdown.
    let unavailable = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let unavailable_url = format!(
        "http://{}/backend-api/wham/usage",
        unavailable.local_addr().unwrap()
    );
    drop(unavailable);

    let stale = fetch_cached_from_url(
        &client,
        &auth,
        &unavailable_url,
        &expected,
        UsageFetchMode::Silent,
        &cache,
    )
    .await
    .unwrap();
    assert_eq!(stale, good);
    let calls_after_failure = auth.auth_calls.load(Ordering::SeqCst);

    let backed_off = fetch_cached_from_url(
        &client,
        &auth,
        &unavailable_url,
        &expected,
        UsageFetchMode::Silent,
        &cache,
    )
    .await
    .unwrap();
    assert_eq!(backed_off, good);
    assert_eq!(
        auth.auth_calls.load(Ordering::SeqCst),
        calls_after_failure,
        "silent polling must not retry the network inside backoff"
    );

    let explicit = fetch_cached_from_url(
        &client,
        &auth,
        &unavailable_url,
        &expected,
        UsageFetchMode::Explicit,
        &cache,
    )
    .await;
    assert!(
        explicit.is_err(),
        "explicit refresh bypasses silent backoff"
    );
    assert!(auth.auth_calls.load(Ordering::SeqCst) > calls_after_failure);
    task.abort();
}

#[test]
fn usage_backoff_is_exponential_and_capped() {
    assert_eq!(backoff_for_failures(1), std::time::Duration::from_secs(60));
    assert_eq!(backoff_for_failures(2), std::time::Duration::from_secs(120));
    assert_eq!(backoff_for_failures(3), std::time::Duration::from_secs(240));
    assert_eq!(backoff_for_failures(99), USAGE_BACKOFF_MAX);
}
