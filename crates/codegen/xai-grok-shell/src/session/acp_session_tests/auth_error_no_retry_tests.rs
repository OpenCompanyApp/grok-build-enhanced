use super::support::*;
use super::*;
use crate::auth::{AuthManager, AuthMode, GrokAuth, GrokComConfig};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

/// Test refresher that returns a fresh token and records that it
/// was invoked. Used to drive the auth-arm success path.
struct AlwaysSucceedRefresher {
    called: Arc<AtomicBool>,
}
#[async_trait::async_trait]
impl crate::auth::refresh::TokenRefresher for AlwaysSucceedRefresher {
    async fn refresh(
        &self,
        _reason: crate::auth::refresh::RefreshReason,
    ) -> crate::auth::refresh::RefreshOutcome {
        self.called.store(true, Ordering::SeqCst);
        crate::auth::refresh::RefreshOutcome::Success(Box::new(GrokAuth {
            key: "refreshed-test-token".to_string(),
            auth_mode: AuthMode::Oidc,
            refresh_token: Some("rt-new".into()),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            ..GrokAuth::test_default()
        }))
    }
}

struct AlwaysFailRefresher {
    calls: Arc<std::sync::atomic::AtomicU32>,
}

#[async_trait::async_trait]
impl crate::auth::refresh::TokenRefresher for AlwaysFailRefresher {
    async fn refresh(
        &self,
        _reason: crate::auth::refresh::RefreshReason,
    ) -> crate::auth::refresh::RefreshOutcome {
        self.calls.fetch_add(1, Ordering::SeqCst);
        crate::auth::refresh::RefreshOutcome::permanent(
            crate::auth::error::RefreshTokenFailedReason::RefreshTokenRejected,
            Some("initial-test-key".to_owned()),
        )
    }
}

/// `(tempdir, manager)` with an expired OIDC token loaded so
/// `unauthorized_recovery()` actually dispatches to the refresher.
/// Tempdir must outlive the manager (auth.json path).
fn auth_manager_with_refresher(
    refresher: Arc<dyn crate::auth::refresh::TokenRefresher>,
) -> (tempfile::TempDir, Arc<AuthManager>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let am = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
    am.hot_swap(GrokAuth {
        key: "initial-test-key".into(),
        auth_mode: AuthMode::Oidc,
        refresh_token: Some("rt".into()),
        expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        ..GrokAuth::test_default()
    });
    am.set_refresher(refresher);
    (dir, am)
}

/// Build a `SamplingErrorInfo` of kind Auth - the same shape the
/// inner `OaiCompatClient` emit surfaces after recording its own
/// attribution.
fn auth_error() -> xai_grok_sampler::SamplingErrorInfo {
    xai_grok_sampler::SamplingErrorInfo {
        kind: xai_grok_sampler::SamplingErrorKind::Auth,
        message: "Unauthorized (401)".to_string(),
        status_code: Some(401),
        is_retryable: false,
        retry_after_secs: None,
        should_retry: None,
        model_metadata: None,
        empty_response_context: None,
        doom_loop_triggers: None,
        doom_loop_aborted_at_chunk: None,
    }
}

fn rate_limited_error(message: &str) -> xai_grok_sampler::SamplingErrorInfo {
    xai_grok_sampler::SamplingErrorInfo {
        kind: xai_grok_sampler::SamplingErrorKind::RateLimited,
        message: message.to_owned(),
        status_code: Some(429),
        is_retryable: true,
        retry_after_secs: Some(7),
        should_retry: Some(false),
        model_metadata: None,
        empty_response_context: None,
        doom_loop_triggers: None,
        doom_loop_aborted_at_chunk: None,
    }
}

/// Construct a test actor with the supplied `auth_manager` and
/// session-token credentials wired in. Wraps the actor in `Arc`
/// ready for `handle_sampling_failure`.
async fn make_actor_with_auth_manager(
    auth_manager: Option<Arc<AuthManager>>,
) -> (Arc<SessionActor>, mpsc::UnboundedReceiver<PersistenceMsg>) {
    make_actor_with_auth_and_credentials(
        auth_manager,
        xai_chat_state::AuthType::SessionToken,
        "initial-test-key".to_string(),
    )
    .await
}

/// Variant that pins the credential `auth_type`; the `auth_method_id` is
/// derived from it. Use [`make_actor_with_method_and_credentials`] to pin the
/// two independently.
async fn make_actor_with_auth_and_credentials(
    auth_manager: Option<Arc<AuthManager>>,
    auth_type: xai_chat_state::AuthType,
    api_key: String,
) -> (Arc<SessionActor>, mpsc::UnboundedReceiver<PersistenceMsg>) {
    let method_id = match auth_type {
        xai_chat_state::AuthType::SessionToken => "cached_token",
        xai_chat_state::AuthType::ApiKey => "xai.api_key",
    };
    make_actor_with_method_and_credentials(auth_manager, method_id, auth_type, api_key).await
}

/// Pin the ACP `auth_method_id` and credential `auth_type` independently. The
/// gate keys off the stable `auth_method_id`, so this reproduces the regression:
/// a session method whose `creds.auth_type` has transiently collapsed to
/// `ApiKey` (session-token cache miss + `XAI_API_KEY`).
async fn make_actor_with_method_and_credentials(
    auth_manager: Option<Arc<AuthManager>>,
    auth_method_id: &str,
    auth_type: xai_chat_state::AuthType,
    api_key: String,
) -> (Arc<SessionActor>, mpsc::UnboundedReceiver<PersistenceMsg>) {
    let (gateway_tx, _) = mpsc::unbounded_channel();
    let (persistence_tx, persistence_rx) = mpsc::unbounded_channel();
    let mut actor = create_test_actor(50_000, 100_000, 85, gateway_tx, persistence_tx).await;
    actor.auth_manager = auth_manager;
    actor.auth_method_id = test_auth_method_id(auth_method_id);
    let mut sampling = actor
        .chat_state_handle
        .get_sampling_config()
        .await
        .expect("test actor sampling config");
    sampling.provider = xai_grok_sampling_types::ProviderId::Xai;
    sampling.base_url = xai_grok_sampling_types::XAI_API_BASE_URL.to_owned();
    actor.chat_state_handle.update_sampling_config(sampling);
    actor
        .chat_state_handle
        .update_credentials(xai_chat_state::Credentials {
            provider: Some(xai_grok_sampling_types::ProviderId::Xai),
            api_key: Some(api_key),
            auth_type,
            ..Default::default()
        });
    (Arc::new(actor), persistence_rx)
}

/// `(tempdir, manager)` holding a valid OIDC token (so `get_valid_token()` is a
/// cache hit). The tempdir must outlive the manager (auth.json path).
fn auth_manager_with_valid_token(key: &str) -> (tempfile::TempDir, Arc<AuthManager>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let am = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
    am.hot_swap(GrokAuth {
        key: key.into(),
        auth_mode: AuthMode::Oidc,
        refresh_token: Some("rt".into()),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        ..GrokAuth::test_default()
    });
    (dir, am)
}

/// Sub-case 1: no auth_manager -> falls through, no emit.
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(attribution_emit_count)]
async fn no_emit_when_auth_manager_is_none() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_auth_manager(None).await;
            crate::auth::attribution::reset_test_emit_count();
            let _ = actor.handle_sampling_failure(auth_error()).await;
            assert_eq!(
                crate::auth::attribution::test_emit_count(),
                0,
                "auth arm must not emit attribution when no auth_manager is wired"
            );
        })
        .await;
}

/// Sub-case 2: no AuthManager → auth recovery is skipped entirely,
/// falls through to terminal error. Covers BYOK / API-key users
/// where no OIDC refresh is possible.
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(attribution_emit_count)]
async fn no_recovery_without_auth_manager() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_auth_and_credentials(
                None,
                xai_chat_state::AuthType::ApiKey,
                "xai-byok-key".to_string(),
            )
            .await;
            crate::auth::attribution::reset_test_emit_count();
            let result = actor.handle_sampling_failure(auth_error()).await;
            assert!(
                result.is_err(),
                "no auth manager must fall through to terminal error"
            );
            assert_eq!(
                crate::auth::attribution::test_emit_count(),
                0,
                "auth arm must not emit attribution without auth manager"
            );
        })
        .await;
}

/// Session-based auth + working refresher → RefreshAuthAndResubmit.
#[tokio::test(flavor = "current_thread")]
async fn sampler_401_recovery_returns_refresh_and_retry() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let result = actor.handle_sampling_failure(auth_error()).await;
            assert!(
                matches!(result, Ok(SamplerFailureRecovery::RefreshAuthAndResubmit)),
                "session-based auth with a working refresher must return RefreshAuthAndResubmit"
            );
            assert!(called.load(Ordering::SeqCst), "refresher must be invoked");
        })
        .await;
}

/// Regression: sampler 401 with API-key auth (BYOK `env_key` /
/// `XAI_API_KEY`) must NOT attempt an OIDC session-token refresh. The
/// bearer on the wire is the static API key, so refreshing the session
/// token reports success but the retry re-sends the same rejected key —
/// an invisible 401 loop that hangs the turn. Recovery is skipped and
/// the 401 surfaces as a terminal error.
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(attribution_emit_count)]
async fn sampler_401_with_api_key_auth_skips_refresh_and_surfaces_error() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_and_credentials(
                Some(am),
                xai_chat_state::AuthType::ApiKey,
                "xai-byok-key".to_string(),
            )
            .await;

            let result = actor.handle_sampling_failure(auth_error()).await;

            assert!(
                result.is_err(),
                "API-key 401 must surface a terminal error, not retry"
            );
            assert!(
                !called.load(Ordering::SeqCst),
                "API-key 401 must NOT trigger an OIDC session-token refresh"
            );
        })
        .await;
}

/// Per-turn pre-flight refresh must not fire when `creds.auth_type` is
/// `ApiKey` (a BYOK model): the model's own API key must not be overwritten
/// by the session JWT.
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(attribution_emit_count)]
async fn pre_flight_refresh_skips_api_key_auth_type() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_and_credentials(
                Some(am),
                xai_chat_state::AuthType::ApiKey,
                "byok-api-key".to_string(),
            )
            .await;
            actor
                .refresh_token_if_expired()
                .await
                .expect("preflight refresh should succeed");
            assert!(
                !called.load(Ordering::SeqCst),
                "pre-flight refresh must NOT fire for ApiKey auth_type"
            );
            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("byok-api-key"),
                "BYOK api_key must not be overwritten by session token refresh"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn managed_xai_refresh_failure_is_returned_without_credential_fallback() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let calls = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysFailRefresher {
                    calls: calls.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;

            let error = actor
                .refresh_token_if_expired()
                .await
                .expect_err("managed refresh failure must abort preflight");
            assert_eq!(
                error.data.as_ref().and_then(serde_json::Value::as_str),
                Some("managed xAI authentication refresh failed")
            );
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("initial-test-key"),
                "failed managed refresh must not adopt a generic/config credential"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn concurrent_managed_xai_refresh_waiters_observe_one_consistent_failure() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let calls = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysFailRefresher {
                    calls: calls.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;

            let (a, b, c, d) = tokio::join!(
                actor.refresh_token_if_expired(),
                actor.refresh_token_if_expired(),
                actor.refresh_token_if_expired(),
                actor.refresh_token_if_expired(),
            );
            for result in [a, b, c, d] {
                let error = result.expect_err("every managed refresh waiter must fail closed");
                assert_eq!(
                    error.data.as_ref().and_then(serde_json::Value::as_str),
                    Some("managed xAI authentication refresh failed")
                );
            }
            assert_eq!(
                calls.load(Ordering::SeqCst),
                1,
                "AuthManager must serialize and cache the permanent refresh failure"
            );
            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("initial-test-key")
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn xai_label_on_custom_origin_never_consults_managed_xai_auth() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::Xai;
            sampling.base_url = "https://custom.invalid/v1".to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            actor
                .refresh_token_if_expired()
                .await
                .expect("untrusted xAI-labelled origin should skip refresh");
            let config = actor
                .reconstruct_full_config()
                .await
                .expect("reconstruct untrusted xAI-labelled route");

            assert!(!called.load(Ordering::SeqCst));
            assert!(config.api_key.is_none());
            assert!(config.bearer_resolver.is_none());
            assert_eq!(
                config.credential_source,
                xai_grok_sampling_types::CredentialSourceId::Unspecified
            );
        })
        .await;
}

/// Proactive refresh keeps the cache hot so `refresh_token_if_expired`
/// (per-turn pre-flight) is a cache hit — the refresher fires once
/// (proactive), then the per-turn call sees the fresh token without
/// hitting the IdP again.
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(attribution_emit_count)]
async fn proactive_refresh_makes_per_turn_refresh_a_cache_hit() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> = Arc::new({
                struct Counting(Arc<std::sync::atomic::AtomicU32>);
                #[async_trait::async_trait]
                impl crate::auth::refresh::TokenRefresher for Counting {
                    async fn refresh(
                        &self,
                        _: crate::auth::refresh::RefreshReason,
                    ) -> crate::auth::refresh::RefreshOutcome {
                        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        crate::auth::refresh::RefreshOutcome::Success(Box::new(GrokAuth {
                            key: "proactive-fresh".into(),
                            auth_mode: AuthMode::Oidc,
                            refresh_token: Some("rt-new".into()),
                            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
                            ..GrokAuth::test_default()
                        }))
                    }
                }
                Counting(call_count.clone())
            });

            let (_dir, am) = auth_manager_with_refresher(refresher);
            let cancel = tokio_util::sync::CancellationToken::new();
            am.start_proactive_refresh(cancel.clone());

            // Wait for proactive task to fire.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            assert!(
                call_count.load(Ordering::SeqCst) >= 1,
                "proactive task must have fired"
            );
            let count_after_proactive = call_count.load(Ordering::SeqCst);

            // Now run refresh_token_if_expired (the per-turn pre-flight).
            // It should see the proactively-refreshed token and NOT invoke
            // the refresher again.
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            actor
                .refresh_token_if_expired()
                .await
                .expect("preflight refresh should succeed");

            assert_eq!(
                call_count.load(Ordering::SeqCst),
                count_after_proactive,
                "per-turn refresh must NOT call the refresher again (cache hit)"
            );
            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("proactive-fresh"),
                "per-turn refresh must pick up the proactively-refreshed token"
            );

            cancel.cancel();
        })
        .await;
}

fn model_not_found_error() -> xai_grok_sampler::SamplingErrorInfo {
    xai_grok_sampler::SamplingErrorInfo {
            kind: xai_grok_sampler::SamplingErrorKind::Api,
            message: "API error (status 404 Not Found): The model grok-build does not exist or your team does not have access".into(),
            status_code: Some(404),
            is_retryable: false,
            retry_after_secs: None,
            should_retry: None,
            model_metadata: None,
            empty_response_context: None,
            doom_loop_triggers: None,
            doom_loop_aborted_at_chunk: None,
        }
}

/// 404 model-not-found with a legacy WebLogin token appends a
/// "Legacy auth detected" hint to the error message.
#[tokio::test(flavor = "current_thread")]
async fn legacy_auth_hint_on_404_model_not_found() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().expect("tempdir");
            let am = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
            am.hot_swap(GrokAuth {
                key: "legacy-token".into(),
                auth_mode: AuthMode::WebLogin,
                ..GrokAuth::test_default()
            });

            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let result = actor.handle_sampling_failure(model_not_found_error()).await;
            let err = match result {
                Err(e) => e,
                Ok(_) => panic!("expected Err from handle_sampling_failure"),
            };
            let data = err.data.unwrap();
            let msg = data.as_str().unwrap();
            assert!(
                msg.contains("deprecated authentication method"),
                "404 with WebLogin must include deprecation message, got: {msg}"
            );
            assert!(
                msg.contains("grok logout"),
                "hint must mention `grok logout`, got: {msg}"
            );
            assert!(
                msg.contains("grok login"),
                "hint must mention `grok login`, got: {msg}"
            );
            assert!(
                msg.contains("Version:"),
                "must show client version, got: {msg}"
            );
        })
        .await;
}

/// Build a 401-shaped error that bypasses step 4b's auth recovery.
///
/// In production, 401s arrive as `SamplingErrorKind::Auth` with
/// `status_code: None`. Step 4b intercepts `Auth`-kind errors and
/// runs the full recovery chain — which succeeds on devbox/CI
/// environments via SA-token mint, masking the hint.
///
/// Using `Api` kind + `status_code: Some(401)` exercises the hint
/// condition (`status_code == Some(401)`) without triggering
/// recovery, making the test environment-independent.
fn unauthorized_401_error() -> xai_grok_sampler::SamplingErrorInfo {
    xai_grok_sampler::SamplingErrorInfo {
            kind: xai_grok_sampler::SamplingErrorKind::Api,
            message: "Unauthorized (401) from https://cli-chat-proxy.grok.com/v1/responses: {\"error\":\"Invalid or expired credentials (auth_kind=bearer, x_xai_token_auth=xai-grok-cli, upstream=Unauthenticated, reason=no auth context)\"}".into(),
            status_code: Some(401),
            is_retryable: false,
            retry_after_secs: None,
            should_retry: None,
            model_metadata: None,
            empty_response_context: None,
            doom_loop_triggers: None,
            doom_loop_aborted_at_chunk: None,
        }
}

/// 401 Unauthorized with a legacy WebLogin token appends a
/// "Legacy auth detected" hint to the error message.
#[tokio::test(flavor = "current_thread")]
async fn legacy_auth_hint_on_401_unauthorized() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().expect("tempdir");
            let am = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
            am.hot_swap(GrokAuth {
                key: "legacy-token".into(),
                auth_mode: AuthMode::WebLogin,
                ..GrokAuth::test_default()
            });

            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let result = actor
                .handle_sampling_failure(unauthorized_401_error())
                .await;
            let err = match result {
                Err(e) => e,
                Ok(_) => panic!("expected Err from handle_sampling_failure"),
            };
            let data = err.data.unwrap();
            let msg = data.as_str().unwrap();
            assert!(
                msg.contains("deprecated authentication method"),
                "401 with WebLogin must include deprecation message, got: {msg}"
            );
            assert!(
                msg.contains("grok logout"),
                "hint must mention `grok logout`, got: {msg}"
            );
            assert!(
                msg.contains("grok login"),
                "hint must mention `grok login`, got: {msg}"
            );
        })
        .await;
}

/// A simultaneously logged-in xAI account must not influence Codex failure
/// labeling or reauthentication advice.
#[tokio::test(flavor = "current_thread")]
async fn codex_401_uses_provider_scoped_auth_label_and_login_commands() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai_refresh_called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: xai_refresh_called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::OpenAiCodex;
            sampling.model = "gpt-5.6-luna".to_owned();
            sampling.base_url = xai_grok_sampling_types::OPENAI_CODEX_BASE_URL.to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            actor
                .refresh_token_if_expired()
                .await
                .expect("preflight refresh should succeed");
            assert!(
                !xai_refresh_called.load(Ordering::SeqCst),
                "Codex pre-flight must never call the xAI refresher"
            );

            let result = actor
                .handle_sampling_failure(unauthorized_401_error())
                .await;
            let err = match result {
                Err(error) => error,
                Ok(_) => panic!("expected terminal Codex 401"),
            };
            let data = err.data.expect("error details");
            let message = data
                .get("message")
                .and_then(|value| value.as_str())
                .or_else(|| data.as_str())
                .expect("user-facing message");
            assert!(message.contains("Auth:      openai_codex"), "{message}");
            assert!(
                message.contains("grok logout --provider openai-codex"),
                "{message}"
            );
            assert!(
                message.contains("grok login --provider openai-codex"),
                "{message}"
            );
            assert!(
                !message.contains("deprecated authentication method"),
                "{message}"
            );
            assert!(!message.contains("Auth:      WebLogin"), "{message}");
            assert!(
                !xai_refresh_called.load(Ordering::SeqCst),
                "Codex terminal auth failures must never call the xAI refresher"
            );
        })
        .await;
}

/// Kimi authentication is provider-bound and must never enter the xAI token
/// refresh path during per-turn preflight.
#[tokio::test(flavor = "current_thread")]
async fn kimi_preflight_does_not_run_xai_credential_refresh() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai_refresh_called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: xai_refresh_called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::KimiCode;
            sampling.model = "k3".to_owned();
            sampling.base_url = xai_grok_sampling_types::KIMI_CODE_BASE_URL.to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            actor
                .refresh_token_if_expired()
                .await
                .expect("preflight refresh should succeed");

            assert!(
                !xai_refresh_called.load(Ordering::SeqCst),
                "Kimi preflight must never call the xAI refresher"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn non_xai_429_keeps_provider_detail_and_never_uses_xai_billing_code() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_auth_manager(None).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::KimiCode;
            sampling.model = "k3".to_owned();
            sampling.base_url = xai_grok_sampling_types::KIMI_CODE_BASE_URL.to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            let error = match actor
                .handle_sampling_failure(rate_limited_error(
                    "Kimi provider capacity is temporarily unavailable",
                ))
                .await
            {
                Err(error) => error,
                Ok(_) => panic!("non-xAI 429 should remain terminal"),
            };

            assert_ne!(
                i32::from(error.code),
                crate::sampling::error::RATE_LIMITED_ERROR_CODE,
                "xAI billing/upgrade rendering code must be xAI-only"
            );
            let details = error.data.expect("provider error details");
            let message = details
                .get("message")
                .and_then(serde_json::Value::as_str)
                .or_else(|| details.as_str())
                .expect("provider error message");
            assert!(message.contains("Kimi provider capacity"), "{message}");
            assert!(!message.contains("Upgrade your account"), "{message}");
            assert!(!message.contains("purchase more credits"), "{message}");
        })
        .await;
}

/// A terminal Kimi 401 must not be recovered with credentials from a
/// simultaneously logged-in xAI account.
#[tokio::test(flavor = "current_thread")]
async fn kimi_401_does_not_run_xai_auth_recovery() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai_refresh_called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: xai_refresh_called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::KimiCode;
            sampling.model = "k3".to_owned();
            sampling.base_url = xai_grok_sampling_types::KIMI_CODE_BASE_URL.to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            let result = actor.handle_sampling_failure(auth_error()).await;

            assert!(result.is_err(), "Kimi 401 must remain terminal");
            assert!(
                !xai_refresh_called.load(Ordering::SeqCst),
                "Kimi 401 must never call the xAI refresher"
            );
        })
        .await;
}

/// Kimi authentication failures should identify the selected provider and its
/// own login command rather than xAI or Codex recovery instructions.
#[tokio::test(flavor = "current_thread")]
async fn kimi_401_surfaces_provider_scoped_reauthentication_details() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_auth_manager(None).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::KimiCode;
            sampling.model = "k3".to_owned();
            sampling.base_url = xai_grok_sampling_types::KIMI_CODE_BASE_URL.to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            let error = match actor.handle_sampling_failure(auth_error()).await {
                Err(error) => error,
                Ok(_) => panic!("Kimi 401 must be terminal"),
            };
            let details = error.data.expect("error details");
            let message = details
                .get("message")
                .and_then(serde_json::Value::as_str)
                .or_else(|| details.as_str())
                .expect("user-facing message");

            assert!(message.contains("Auth:      kimi_code"), "{message}");
            assert!(
                message.contains("grok login --provider kimi-code"),
                "{message}"
            );
            assert!(!message.contains("openai-codex"), "{message}");
            assert!(!message.contains("Auth:      WebLogin"), "{message}");
        })
        .await;
}

/// A terminal Z.AI Coding Plan 401 must never invoke the global xAI token
/// refresher, even when an xAI account is logged in simultaneously.
#[tokio::test(flavor = "current_thread")]
async fn zai_401_does_not_run_xai_auth_recovery() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai_refresh_called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: xai_refresh_called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::ZaiCodingPlan;
            sampling.model = "glm-5.2".to_owned();
            sampling.base_url = xai_grok_sampling_types::ZAI_CODING_PLAN_BASE_URL.to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            let result = actor.handle_sampling_failure(auth_error()).await;

            assert!(result.is_err(), "Z.AI 401 must remain terminal");
            assert!(
                !xai_refresh_called.load(Ordering::SeqCst),
                "Z.AI 401 must never call the xAI refresher"
            );
        })
        .await;
}

/// Z.AI authentication failures must identify only the Coding Plan provider
/// and its own login command.
#[tokio::test(flavor = "current_thread")]
async fn zai_401_surfaces_provider_scoped_reauthentication_details() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_auth_manager(None).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor sampling config");
            sampling.provider = xai_grok_sampling_types::ProviderId::ZaiCodingPlan;
            sampling.model = "glm-5.2".to_owned();
            sampling.base_url = xai_grok_sampling_types::ZAI_CODING_PLAN_BASE_URL.to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            let error = match actor.handle_sampling_failure(auth_error()).await {
                Err(error) => error,
                Ok(_) => panic!("Z.AI 401 must be terminal"),
            };
            let details = error.data.expect("error details");
            let message = details
                .get("message")
                .and_then(serde_json::Value::as_str)
                .or_else(|| details.as_str())
                .expect("user-facing message");

            assert!(message.contains("Auth:      zai_coding_plan"), "{message}");
            assert!(
                message.contains("grok login --provider zai-coding-plan"),
                "{message}"
            );
            assert!(!message.contains("openai-codex"), "{message}");
            assert!(!message.contains("Auth:      WebLogin"), "{message}");
        })
        .await;
}

/// 401 with OIDC auth must NOT append the legacy hint.
#[tokio::test(flavor = "current_thread")]
async fn no_legacy_hint_on_401_for_oidc_auth() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().expect("tempdir");
            let am = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
            am.hot_swap(GrokAuth {
                key: "oidc-token".into(),
                auth_mode: AuthMode::Oidc,
                refresh_token: Some("rt".into()),
                expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
                ..GrokAuth::test_default()
            });

            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let result = actor
                .handle_sampling_failure(unauthorized_401_error())
                .await;
            let err = match result {
                Err(e) => e,
                Ok(_) => panic!("expected Err from handle_sampling_failure"),
            };
            let data = err.data.unwrap();
            let msg = data
                .get("message")
                .and_then(|v| v.as_str())
                .or_else(|| data.as_str())
                .unwrap();
            assert!(
                !msg.contains("deprecated authentication method"),
                "OIDC auth must NOT trigger WebLogin deprecation on 401, got: {msg}"
            );
            assert!(
                msg.contains("Auth:      Oidc"),
                "OIDC 401 must show auth mode in enriched message, got: {msg}"
            );
        })
        .await;
}

/// 404 model-not-found with OIDC auth must NOT append the legacy hint.
#[tokio::test(flavor = "current_thread")]
async fn no_legacy_hint_for_oidc_auth() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().expect("tempdir");
            let am = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
            am.hot_swap(GrokAuth {
                key: "oidc-token".into(),
                auth_mode: AuthMode::Oidc,
                refresh_token: Some("rt".into()),
                expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
                ..GrokAuth::test_default()
            });

            let (actor, _rx) = make_actor_with_auth_manager(Some(am)).await;
            let result = actor.handle_sampling_failure(model_not_found_error()).await;
            let err = match result {
                Err(e) => e,
                Ok(_) => panic!("expected Err from handle_sampling_failure"),
            };
            let data = err.data.unwrap();
            let msg = data
                .get("message")
                .and_then(|v| v.as_str())
                .or_else(|| data.as_str())
                .unwrap();
            assert!(
                !msg.contains("deprecated authentication method"),
                "OIDC auth must NOT trigger WebLogin deprecation, got: {msg}"
            );
            assert!(
                msg.contains("Auth:      Oidc"),
                "OIDC 404 must show auth mode in enriched message, got: {msg}"
            );
            assert!(
                msg.contains("Version:"),
                "OIDC 404 must show version in enriched message, got: {msg}"
            );
        })
        .await;
}

// Regression group: a live session whose `auth_type` transiently reads `ApiKey`
// must still recover, because the gate keys off the stable `auth_method_id`.
#[test]
fn session_token_auth_gate_truth_table() {
    use crate::agent::auth_method::{ModelByok, session_token_auth_gate as gate};
    // Non-session methods never refresh, regardless of BYOK status or endpoint.
    for fp in [false, true] {
        assert!(!gate(false, ModelByok::NotByok, fp));
        assert!(!gate(false, ModelByok::Byok, fp));
        assert!(!gate(false, ModelByok::Unknown, fp));
        // Session method: a definite classification ignores the endpoint —
        // NotByok always refreshes (only ever routes to the session endpoint),
        // a genuine per-model Byok never does.
        assert!(gate(true, ModelByok::NotByok, fp));
        assert!(!gate(true, ModelByok::Byok, fp));
    }
    // Session method + Unknown BYOK: refresh only against a first-party xAI
    // host, so a transiently-unclassifiable config can't demote a live session
    // (the stale-token 401 regression) yet the session token never leaks to a
    // third-party BYOK endpoint. This arm was unconditionally `false` pre-fix.
    assert!(gate(true, ModelByok::Unknown, true));
    assert!(!gate(true, ModelByok::Unknown, false));
}

/// Pre-fix, the gate read `auth_type` and skipped recovery here, 401'ing every
/// turn until restart.
#[tokio::test(flavor = "current_thread")]
async fn sampler_401_session_method_with_stale_api_key_auth_type_still_recovers() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(am),
                "cached_token",
                xai_chat_state::AuthType::ApiKey,
                "stale-session-jwt".to_string(),
            )
            .await;

            let result = actor.handle_sampling_failure(auth_error()).await;

            assert!(
                matches!(result, Ok(SamplerFailureRecovery::RefreshAuthAndResubmit)),
                "session-based method must recover even when auth_type transiently reads ApiKey"
            );
            assert!(
                called.load(Ordering::SeqCst),
                "the OIDC refresher must be invoked for a session-based method"
            );
        })
        .await;
}

/// Same regression via the `oidc` method id (the other session-based variant).
#[tokio::test(flavor = "current_thread")]
async fn sampler_401_oidc_method_with_stale_api_key_auth_type_still_recovers() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, am) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(am),
                "oidc",
                xai_chat_state::AuthType::ApiKey,
                "stale-session-jwt".to_string(),
            )
            .await;

            let result = actor.handle_sampling_failure(auth_error()).await;

            assert!(
                matches!(result, Ok(SamplerFailureRecovery::RefreshAuthAndResubmit)),
                "oidc method must recover even when auth_type transiently reads ApiKey"
            );
            assert!(
                called.load(Ordering::SeqCst),
                "the OIDC refresher must be invoked"
            );
        })
        .await;
}

/// Without the live bearer resolver here the sampler would sign requests with
/// the stale buffered token.
#[tokio::test(flavor = "current_thread")]
async fn reconstruct_full_config_wires_bearer_resolver_for_session_method_despite_api_key_auth_type()
 {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (_dir, am) = auth_manager_with_valid_token("fresh-session-token");
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(am),
                "cached_token",
                xai_chat_state::AuthType::ApiKey,
                "stale-session-jwt".to_string(),
            )
            .await;

            let cfg = actor
                .reconstruct_full_config()
                .await
                .expect("provider binding should succeed");

            assert!(
                cfg.bearer_resolver.is_some(),
                "session-based method must use the live bearer resolver, not the buffered key"
            );
        })
        .await;
}

/// Negative: a genuine `xai.api_key` method keeps its configured key on the
/// wire (no live resolver).
#[tokio::test(flavor = "current_thread")]
async fn reconstruct_full_config_no_bearer_resolver_for_api_key_method() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (_dir, am) = auth_manager_with_valid_token("session-token");
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(am),
                "xai.api_key",
                xai_chat_state::AuthType::ApiKey,
                "xai-static-key".to_string(),
            )
            .await;

            let cfg = actor
                .reconstruct_full_config()
                .await
                .expect("provider binding should succeed");

            assert!(
                cfg.bearer_resolver.is_none(),
                "api-key method must keep its configured bearer (no live resolver)"
            );
        })
        .await;
}

/// The pre-flight refresh heals a transiently-`ApiKey` session by writing the
/// fresh session token back into `creds.api_key`.
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(attribution_emit_count)]
async fn pre_flight_refresh_heals_session_method_with_stale_api_key_auth_type() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (_dir, am) = auth_manager_with_valid_token("fresh-session-token");
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(am),
                "cached_token",
                xai_chat_state::AuthType::ApiKey,
                "stale-session-jwt".to_string(),
            )
            .await;

            actor
                .refresh_token_if_expired()
                .await
                .expect("preflight refresh should succeed");

            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("fresh-session-token"),
                "session-based pre-flight refresh must heal a stale api_key with the live token"
            );
        })
        .await;
}

/// End-to-end for the frozen-gate bug: a session born on `xai.api_key` (gate
/// inactive) must adopt a later OIDC `/login` on the SAME actor -- the shared
/// `auth_method_id` handle is flipped in place (no re-spawn), so the next turn
/// wires the live bearer resolver and heals the stale key.
#[tokio::test(flavor = "current_thread")]
async fn session_born_on_api_key_recovers_after_oidc_login_without_restart() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (_dir, am) = auth_manager_with_valid_token("fresh-oidc-token");
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(am),
                "xai.api_key",
                xai_chat_state::AuthType::ApiKey,
                "stale-session-jwt".to_string(),
            )
            .await;

            // Born on api_key: the gate is inactive, so no live resolver.
            assert!(
                actor
                    .reconstruct_full_config()
                    .await
                    .expect("provider binding should succeed")
                    .bearer_resolver
                    .is_none(),
                "api-key session must not use the live resolver before login"
            );

            // Simulate the agent's `authenticate` publishing an OIDC method into
            // the shared handle this running actor already holds (no re-spawn).
            actor
                .auth_method_id
                .store(Some(std::sync::Arc::new(acp::AuthMethodId::new("oidc"))));

            // The gate is recomputed each turn from the shared handle, so the
            // flip alone activates the live resolver on the very next turn --
            // no re-spawn, before any token refresh runs.
            assert!(
                actor
                    .reconstruct_full_config()
                    .await
                    .expect("provider binding should succeed")
                    .bearer_resolver
                    .is_some(),
                "flipping the shared handle activates the resolver on the next turn"
            );

            // The pre-flight refresh then heals the stale api_key with the live token.
            actor
                .refresh_token_if_expired()
                .await
                .expect("preflight refresh should succeed");
            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("fresh-oidc-token"),
                "the stale api_key must be healed with the fresh OIDC token"
            );
        })
        .await;
}

// Per-model BYOK memo (`SessionActor::model_auth_memo`): a definite cached
// status is served without recomputing, and the memo keys on `model_id`.

/// The cache-hit branch is what lets a later config parse failure (`Unknown`)
/// fall back to the last-known-good status.
#[tokio::test(flavor = "current_thread")]
async fn model_auth_memo_serves_cached_status_and_keys_on_model() {
    use crate::agent::auth_method::ModelByok;
    use crate::agent::config::ModelAuthFacts;
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_method_and_credentials(
                None,
                "cached_token",
                xai_chat_state::AuthType::SessionToken,
                "k".to_string(),
            )
            .await;

            actor
                .model_auth_memo
                .replace(Some(crate::session::acp_session::ModelAuthMemo {
                    model_id: "model-a".to_string(),
                    facts: ModelAuthFacts {
                        byok: ModelByok::Byok,
                        auth_scheme: Default::default(),
                    },
                    provider: None,
                }));

            // Cache hit: served without consulting config.
            assert_eq!(actor.model_auth_facts("model-a").byok, ModelByok::Byok);

            // Different model re-resolves rather than serving the stale `Byok`.
            assert_ne!(actor.model_auth_facts("model-b").byok, ModelByok::Byok);
        })
        .await;
}

/// A restored custom session must not reinterpret an xAI-owned credential as
/// the custom model's own BYOK key merely because the model is configured BYOK.
#[tokio::test(flavor = "current_thread")]
async fn reconstruct_full_config_drops_cross_provider_custom_credential() {
    use crate::agent::auth_method::ModelByok;
    use crate::agent::config::ModelAuthFacts;
    use xai_grok_sampling_types::{CredentialSourceId, ProviderId};

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_method_and_credentials(
                None,
                "xai.api_key",
                xai_chat_state::AuthType::ApiKey,
                "xai-credential-sentinel".to_string(),
            )
            .await;

            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor has sampling config");
            sampling.provider = ProviderId::Custom;
            sampling.model = "custom-model".to_owned();
            sampling.base_url = "https://custom.example/v1".to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);
            actor
                .model_auth_memo
                .replace(Some(crate::session::acp_session::ModelAuthMemo {
                    model_id: "custom-model".to_owned(),
                    facts: ModelAuthFacts {
                        byok: ModelByok::Byok,
                        auth_scheme: Default::default(),
                    },
                    provider: None,
                }));

            let cfg = actor
                .reconstruct_full_config()
                .await
                .expect("custom provider binding should succeed");

            assert_eq!(cfg.provider, ProviderId::Custom);
            assert_eq!(cfg.credential_source, CredentialSourceId::Unspecified);
            assert!(cfg.api_key.is_none());
            assert!(cfg.bearer_resolver.is_none());
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn reconstruct_full_config_marks_a_cached_helper_key_as_rotating() {
    use xai_grok_sampling_types::CredentialSourceId;

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let provider = crate::auth::AuthProviderRef::new(
                "test-reconstruct-rotating".to_owned(),
                crate::auth::AuthProviderConfig {
                    command: "printf rotating-token".to_owned(),
                    args: None,
                    token_ttl_secs: Some(3600),
                    timeout_secs: None,
                },
            );
            let token = provider
                .ensure_fresh_token(None)
                .await
                .rotated()
                .expect("provider mints a token");
            let (actor, _rx) =
                make_actor_with_auth_and_credentials(None, xai_chat_state::AuthType::ApiKey, token)
                    .await;
            seed_provider_memo(&actor, provider).await;

            let cfg = actor
                .reconstruct_full_config()
                .await
                .expect("custom provider binding should succeed");

            assert_eq!(
                cfg.credential_source,
                CredentialSourceId::RotatingAuthProvider
            );
            assert_eq!(cfg.api_key.as_deref(), Some("rotating-token"));
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn reconstruct_full_config_keeps_rotating_source_for_a_short_lived_token() {
    use xai_grok_sampling_types::CredentialSourceId;

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let provider = crate::auth::AuthProviderRef::new(
                "test-reconstruct-short-lived".to_owned(),
                crate::auth::AuthProviderConfig {
                    command: "printf short-lived-token".to_owned(),
                    args: None,
                    token_ttl_secs: Some(1),
                    timeout_secs: None,
                },
            );
            let token = provider
                .ensure_fresh_token(None)
                .await
                .rotated()
                .expect("provider mints a token");
            assert_eq!(
                provider.cached_token(),
                None,
                "the cache read is stale immediately"
            );
            let (actor, _rx) =
                make_actor_with_auth_and_credentials(None, xai_chat_state::AuthType::ApiKey, token)
                    .await;
            seed_provider_memo(&actor, provider).await;

            let cfg = actor
                .reconstruct_full_config()
                .await
                .expect("custom provider binding should succeed");

            assert_eq!(
                cfg.credential_source,
                CredentialSourceId::RotatingAuthProvider
            );
        })
        .await;
}

/// A session method whose active model is a genuine per-model BYOK model keeps
/// the model's own key on the wire (no live resolver).
#[tokio::test(flavor = "current_thread")]
async fn reconstruct_full_config_no_bearer_resolver_for_byok_model_on_session_method() {
    use crate::agent::auth_method::ModelByok;
    use crate::agent::config::ModelAuthFacts;
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (_dir, am) = auth_manager_with_valid_token("session-token");
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(am),
                "cached_token",
                xai_chat_state::AuthType::SessionToken,
                "byok-key".to_string(),
            )
            .await;

            let model = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .map(|c| c.model)
                .unwrap_or_default();
            actor
                .model_auth_memo
                .replace(Some(crate::session::acp_session::ModelAuthMemo {
                    model_id: model,
                    facts: ModelAuthFacts {
                        byok: ModelByok::Byok,
                        auth_scheme: Default::default(),
                    },
                    provider: None,
                }));

            let cfg = actor
                .reconstruct_full_config()
                .await
                .expect("provider binding should succeed");

            assert!(
                cfg.bearer_resolver.is_none(),
                "a per-model BYOK model must keep its own key even on a session method"
            );
        })
        .await;
}

/// Regression: a model-switch chokepoint must invalidate
/// the memo even when `model_id` is unchanged. Otherwise a config edit that
/// turns the current model into a per-model BYOK model on a third-party
/// `base_url` keeps serving the stale `NotByok`, leaving the gate active and
/// leaking the OIDC token cross-host.
#[tokio::test(flavor = "current_thread")]
async fn set_session_model_invalidates_byok_memo_for_same_model_id() {
    use crate::agent::auth_method::ModelByok;
    use crate::agent::config::ModelAuthFacts;
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (actor, _rx) = make_actor_with_method_and_credentials(
                None,
                "cached_token",
                xai_chat_state::AuthType::SessionToken,
                "k".to_string(),
            )
            .await;

            let model = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .map(|c| c.model)
                .unwrap_or_default();

            actor
                .model_auth_memo
                .replace(Some(crate::session::acp_session::ModelAuthMemo {
                    model_id: model.clone(),
                    facts: ModelAuthFacts {
                        byok: ModelByok::NotByok,
                        auth_scheme: Default::default(),
                    },
                    provider: None,
                }));

            // Switch to the same model_id, now a per-model BYOK model on a
            // third-party endpoint.
            let cfg = xai_grok_sampler::SamplerConfig {
                provider: Default::default(),
                credential_source: Default::default(),
                credential_binding: None,
                api_key: Some("byok-key".to_string()),
                base_url: "https://third-party.example/v1".to_string(),
                model: model.clone(),
                service_tier: None,
                max_completion_tokens: None,
                temperature: None,
                top_p: None,
                api_backend: crate::sampling::ApiBackend::ChatCompletions,
                auth_scheme: Default::default(),
                extra_headers: Default::default(),
                context_window: 256_000,
                client_version: None,
                force_http1: false,
                max_retries: None,
                stream_tool_calls: false,
                idle_timeout_secs: None,
                client_identifier: None,
                reasoning_effort: None,
                comp_hash: None,
                supports_reasoning_summary_parameter: false,
                default_reasoning_summary: None,
                deployment_id: None,
                user_id: None,
                origin_client: None,
                attribution_callback: None,
                bearer_resolver: None,
                request_auth: None,
                supports_backend_search: false,
                compactions_remaining: None,
                compaction_at_tokens: None,
                doom_loop_recovery: None,
                header_injector: None,
            };
            let _ = actor
                .handle_set_session_model(cfg, false, false, true, 85)
                .await;

            assert!(
                actor.model_auth_memo.borrow().is_none(),
                "a model switch must invalidate the per-model BYOK memo so the next \
                 reconstruct recomputes under the current config"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn switching_to_xai_replaces_the_rotating_custom_provider_key() {
    use xai_grok_sampling_types::{CredentialSourceId, ProviderId};

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().unwrap();
            let provider =
                crate::auth::test_counting_provider("test-switch-provider-isolation", dir.path());
            let token = provider
                .ensure_fresh_token(None)
                .await
                .rotated()
                .expect("provider mints a token");
            let (actor, _rx) =
                make_actor_with_auth_and_credentials(None, xai_chat_state::AuthType::ApiKey, token)
                    .await;
            seed_provider_memo(&actor, provider).await;
            let xai = xai_grok_sampler::SamplerConfig {
                provider: ProviderId::Xai,
                credential_source: CredentialSourceId::StaticApiKey,
                api_key: Some("xai-api-key".to_owned()),
                base_url: "https://api.x.ai/v1".to_owned(),
                model: "grok-first-party".to_owned(),
                context_window: 256_000,
                ..Default::default()
            };

            actor
                .handle_set_session_model(xai, false, false, true, 85)
                .await
                .expect("model switch succeeds");

            let credentials = actor.chat_state_handle.get_credentials().await;
            assert_eq!(credentials.provider, Some(ProviderId::Xai));
            assert_eq!(credentials.api_key.as_deref(), Some("xai-api-key"));
        })
        .await;
}

use crate::auth::test_counting_provider as counting_provider;

/// Seed one exact-route-bound custom-provider model without loading process
/// config. Session tests use this to isolate the turn/auth recovery seam.
async fn seed_provider_memo(actor: &Arc<SessionActor>, provider: crate::auth::AuthProviderRef) {
    let mut sampling = actor
        .chat_state_handle
        .get_sampling_config()
        .await
        .expect("test actor has sampling config");
    sampling.provider = xai_grok_sampling_types::ProviderId::Custom;
    sampling.base_url = "https://auth-provider.test/v1".to_owned();
    let model = sampling.model.clone();
    actor.chat_state_handle.update_sampling_config(sampling);

    let mut credentials = actor.chat_state_handle.get_credentials().await;
    credentials.provider = Some(xai_grok_sampling_types::ProviderId::Custom);
    actor.chat_state_handle.update_credentials(credentials);
    actor
        .model_auth_memo
        .replace(Some(crate::session::acp_session::ModelAuthMemo {
            model_id: model,
            facts: crate::agent::config::ModelAuthFacts {
                byok: crate::agent::auth_method::ModelByok::Byok,
                auth_scheme: Default::default(),
            },
            provider: Some(provider),
        }));
}

#[tokio::test(flavor = "current_thread")]
async fn pre_turn_custom_provider_mints_without_refreshing_xai_session() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().unwrap();
            let provider = counting_provider("test-preturn-exclusive", dir.path());
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, manager) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(manager),
                "cached_token",
                xai_chat_state::AuthType::SessionToken,
                "foreign-session-token".to_owned(),
            )
            .await;
            seed_provider_memo(&actor, provider).await;
            let mut credentials = actor.chat_state_handle.get_credentials().await;
            credentials.api_key = None;
            actor.chat_state_handle.update_credentials(credentials);

            actor
                .refresh_token_if_expired()
                .await
                .expect("custom provider mint succeeds");

            let credentials = actor.chat_state_handle.get_credentials().await;
            assert_eq!(
                credentials.provider,
                Some(xai_grok_sampling_types::ProviderId::Custom)
            );
            assert_eq!(credentials.api_key.as_deref(), Some("tok-1"));
            assert_eq!(credentials.auth_type, xai_chat_state::AuthType::ApiKey);
            assert!(
                !called.load(Ordering::SeqCst),
                "xAI session refresh must never run for custom provider auth"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn failed_pre_turn_custom_provider_clears_stale_key_and_aborts() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let provider = crate::auth::AuthProviderRef::new(
                "test-preturn-failure".to_owned(),
                crate::auth::AuthProviderConfig {
                    command: "exit 1".to_owned(),
                    args: None,
                    token_ttl_secs: None,
                    timeout_secs: None,
                },
            );
            let (actor, _rx) = make_actor_with_auth_and_credentials(
                None,
                xai_chat_state::AuthType::ApiKey,
                "stale-custom-token".to_owned(),
            )
            .await;
            seed_provider_memo(&actor, provider).await;

            assert!(
                actor.refresh_token_if_expired().await.is_err(),
                "a failed helper must abort before sampling"
            );
            let credentials = actor.chat_state_handle.get_credentials().await;
            assert_eq!(
                credentials.provider,
                Some(xai_grok_sampling_types::ProviderId::Custom)
            );
            assert_eq!(
                credentials.api_key, None,
                "the stale bearer must be cleared"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn pre_turn_custom_provider_rejects_a_changed_endpoint() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let provider = crate::auth::AuthProviderRef::new(
                "test-preturn-route-mismatch".to_owned(),
                crate::auth::AuthProviderConfig {
                    command: "printf route-token".to_owned(),
                    args: None,
                    token_ttl_secs: Some(3600),
                    timeout_secs: None,
                },
            );
            let (actor, _rx) = make_actor_with_auth_and_credentials(
                None,
                xai_chat_state::AuthType::ApiKey,
                "stale-custom-token".to_owned(),
            )
            .await;
            seed_provider_memo(&actor, provider).await;
            let mut sampling = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor has sampling config");
            sampling.base_url = "https://other-route.test/v1".to_owned();
            actor.chat_state_handle.update_sampling_config(sampling);

            let error = actor
                .refresh_token_if_expired()
                .await
                .expect_err("the helper must not authenticate a different endpoint");

            assert_eq!(error.code, acp::Error::auth_required().code);
            assert_eq!(
                actor.chat_state_handle.get_credentials().await.api_key,
                None
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn custom_provider_401_with_no_key_mints_before_resubmission() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().unwrap();
            let provider = counting_provider("test-custom-401-no-key", dir.path());
            let (actor, _rx) = make_actor_with_auth_and_credentials(
                None,
                xai_chat_state::AuthType::ApiKey,
                "placeholder".to_owned(),
            )
            .await;
            seed_provider_memo(&actor, provider).await;
            let mut credentials = actor.chat_state_handle.get_credentials().await;
            credentials.api_key = None;
            actor.chat_state_handle.update_credentials(credentials);

            let result = actor.handle_sampling_failure(auth_error()).await;

            assert!(matches!(
                result,
                Ok(SamplerFailureRecovery::RefreshAuthAndResubmit)
            ));
            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("tok-1")
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn custom_provider_401_remints_once_without_xai_recovery() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().unwrap();
            let provider = counting_provider("test-custom-401", dir.path());
            let token = provider.ensure_fresh_token(None).await.rotated().unwrap();
            let called = Arc::new(AtomicBool::new(false));
            let refresher: Arc<dyn crate::auth::refresh::TokenRefresher> =
                Arc::new(AlwaysSucceedRefresher {
                    called: called.clone(),
                });
            let (_dir, manager) = auth_manager_with_refresher(refresher);
            let (actor, _rx) = make_actor_with_method_and_credentials(
                Some(manager),
                "cached_token",
                xai_chat_state::AuthType::SessionToken,
                token,
            )
            .await;
            seed_provider_memo(&actor, provider).await;
            crate::auth::test_backdate_provider_mint(
                "test-custom-401",
                std::time::Duration::from_secs(60),
            );

            let result = actor.handle_sampling_failure(auth_error()).await;
            assert!(matches!(
                result,
                Ok(SamplerFailureRecovery::RefreshAuthAndResubmit)
            ));
            assert!(!called.load(Ordering::SeqCst));
            let credentials = actor.chat_state_handle.get_credentials().await;
            assert_eq!(credentials.api_key.as_deref(), Some("tok-2"));
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn non_auth_kind_401_still_uses_custom_provider_recovery() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().unwrap();
            let provider = counting_provider("test-custom-non-auth-401", dir.path());
            let token = provider.ensure_fresh_token(None).await.rotated().unwrap();
            let (actor, _rx) =
                make_actor_with_auth_and_credentials(None, xai_chat_state::AuthType::ApiKey, token)
                    .await;
            seed_provider_memo(&actor, provider).await;
            crate::auth::test_backdate_provider_mint(
                "test-custom-non-auth-401",
                std::time::Duration::from_secs(60),
            );

            let mut error = auth_error();
            error.kind = xai_grok_sampler::SamplingErrorKind::Api;
            let result = actor.handle_sampling_failure(error).await;
            assert!(matches!(
                result,
                Ok(SamplerFailureRecovery::RefreshAuthAndResubmit)
            ));
            assert_eq!(
                actor
                    .chat_state_handle
                    .get_credentials()
                    .await
                    .api_key
                    .as_deref(),
                Some("tok-2")
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn fresh_rejected_custom_provider_token_surfaces_and_is_cleared() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let dir = tempfile::tempdir().unwrap();
            let provider = counting_provider("test-custom-fresh-guard", dir.path());
            let token = provider.ensure_fresh_token(None).await.rotated().unwrap();
            let (actor, _rx) =
                make_actor_with_auth_and_credentials(None, xai_chat_state::AuthType::ApiKey, token)
                    .await;
            seed_provider_memo(&actor, provider).await;

            assert!(actor.handle_sampling_failure(auth_error()).await.is_err());
            assert_eq!(
                actor.chat_state_handle.get_credentials().await.api_key,
                None,
                "a rejected bearer must not survive the terminal fresh-mint guard"
            );
        })
        .await;
}
