use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use serde_json::json;
use tokio::net::TcpListener;

use super::*;

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

#[async_trait]
impl UsageAuthProvider for FakeAuth {
    async fn request_auth(&self) -> Result<xai_grok_tools::types::RequestAuth, CodexUsageError> {
        let generation = self.auth_calls.fetch_add(1, Ordering::SeqCst) as u64 + 1;
        Ok(xai_grok_tools::types::RequestAuth::for_provider_snapshot(
            OPENAI_CODEX_PROVIDER_ID,
            xai_grok_tools::types::RequestCredentialSnapshot::new("record-id", generation),
            [
                (
                    "Authorization".to_owned(),
                    format!("Bearer token-{generation}"),
                ),
                ("ChatGPT-Account-ID".to_owned(), "account-id".to_owned()),
                ("X-OpenAI-Fedramp".to_owned(), "true".to_owned()),
            ],
        ))
    }

    async fn recover_unauthorized(
        &self,
        rejected: xai_grok_tools::types::RequestCredentialSnapshot,
    ) -> Result<bool, CodexUsageError> {
        assert_eq!(rejected.opaque_id(), "record-id");
        assert_eq!(rejected.generation(), 1);
        self.recoveries.fetch_add(1, Ordering::SeqCst);
        Ok(true)
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
fn request_headers_rejects_unexpected_and_xai_headers() {
    for (name, value) in [
        ("X-XAI-Token-Auth", "session"),
        ("x-userid", "user"),
        ("X-Future-Provider-Header", "value"),
    ] {
        let auth = xai_grok_tools::types::RequestAuth::for_provider_snapshot(
            OPENAI_CODEX_PROVIDER_ID,
            xai_grok_tools::types::RequestCredentialSnapshot::new("record-id", 1),
            [
                ("Authorization".to_owned(), "Bearer secret".to_owned()),
                ("ChatGPT-Account-ID".to_owned(), "account-id".to_owned()),
                (name.to_owned(), value.to_owned()),
            ],
        );

        assert!(matches!(
            request_headers(&auth),
            Err(CodexUsageError::InvalidAuth)
        ));
    }
}

#[test]
fn request_headers_accepts_only_exact_true_fedramp_value() {
    let auth = xai_grok_tools::types::RequestAuth::for_provider_snapshot(
        OPENAI_CODEX_PROVIDER_ID,
        xai_grok_tools::types::RequestCredentialSnapshot::new("record-id", 1),
        [
            ("Authorization".to_owned(), "Bearer secret".to_owned()),
            ("ChatGPT-Account-ID".to_owned(), "account-id".to_owned()),
            ("X-OpenAI-Fedramp".to_owned(), "false".to_owned()),
        ],
    );

    assert!(matches!(
        request_headers(&auth),
        Err(CodexUsageError::InvalidAuth)
    ));
}
