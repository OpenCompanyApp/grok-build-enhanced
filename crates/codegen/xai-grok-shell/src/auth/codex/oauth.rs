use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::{Query, Request, State};
use axum::http::header::{CACHE_CONTROL, HeaderName};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use url::Url;

use super::credentials::TokenResponse;
use super::{
    CodexAuthError, CodexCredentialStore, CodexCredentials, OPENAI_CODEX_CLIENT_ID,
    OPENAI_CODEX_ISSUER, OPENAI_CODEX_ORIGINATOR, OPENAI_CODEX_SCOPES,
};

const PRIMARY_CALLBACK_PORT: u16 = 1455;
const FALLBACK_CALLBACK_PORT: u16 = 1457;
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const AUTH_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const DEVICE_FLOW_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const REFERRER_POLICY: HeaderName = HeaderName::from_static("referrer-policy");

#[derive(Clone, Debug)]
struct OAuthEndpoints {
    issuer: Url,
}

impl OAuthEndpoints {
    fn production() -> Self {
        Self {
            issuer: Url::parse(OPENAI_CODEX_ISSUER).expect("OpenAI issuer must be a valid URL"),
        }
    }

    fn endpoint(&self, path: &str) -> Url {
        self.issuer
            .join(path)
            .expect("fixed OAuth endpoint path must be valid")
    }
}

#[derive(Clone)]
pub struct CodexOAuthClient {
    http: reqwest::Client,
    endpoints: OAuthEndpoints,
    client_id: String,
}

impl fmt::Debug for CodexOAuthClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexOAuthClient")
            .field("issuer", &self.endpoints.issuer)
            .field("client_id_configured", &!self.client_id.is_empty())
            .finish()
    }
}

impl Default for CodexOAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexOAuthClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(AUTH_HTTP_TIMEOUT)
            // OAuth requests carry bearer, refresh, or authorization-code
            // material. Never forward it through an unexpected redirect.
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(format!(
                "{OPENAI_CODEX_ORIGINATOR}/{}",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .expect("static OpenAI OAuth HTTP client configuration must build");
        Self {
            http,
            endpoints: OAuthEndpoints::production(),
            client_id: OPENAI_CODEX_CLIENT_ID.to_string(),
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(issuer: Url) -> Self {
        let mut client = Self::new();
        client.endpoints = OAuthEndpoints { issuer };
        client.client_id = "test-client".to_string();
        client
    }

    pub async fn begin_browser_login(
        &self,
        allowed_workspaces: Option<Vec<String>>,
    ) -> Result<PendingCodexBrowserLogin, CodexAuthError> {
        let listener = bind_callback_listener().await?;
        let port = listener
            .local_addr()
            .map_err(CodexAuthError::Storage)?
            .port();
        let redirect_uri = format!("http://localhost:{port}/auth/callback");
        let pkce = Pkce::generate();
        let state = random_urlsafe(32);
        let authorization_url = self.authorization_url(
            &redirect_uri,
            &pkce.challenge,
            &state,
            allowed_workspaces.as_deref(),
        );

        Ok(PendingCodexBrowserLogin {
            oauth: self.clone(),
            listener,
            redirect_uri,
            pkce,
            state,
            authorization_url,
            allowed_workspaces,
        })
    }

    pub async fn request_device_authorization(
        &self,
    ) -> Result<CodexDeviceAuthorization, CodexAuthError> {
        let endpoint = self.endpoints.endpoint("api/accounts/deviceauth/usercode");
        let response = self
            .http
            .post(endpoint)
            .json(&DeviceCodeRequest {
                client_id: &self.client_id,
            })
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CodexAuthError::DeviceFlowUnavailable);
        }
        if !response.status().is_success() {
            return Err(CodexAuthError::HttpStatus(response.status().as_u16()));
        }
        let response: DeviceCodeResponse = response.json().await?;
        let interval = response.interval.clamp(1, 30);
        Ok(CodexDeviceAuthorization {
            verification_url: self.endpoints.endpoint("codex/device"),
            user_code: response.user_code,
            device_auth_id: response.device_auth_id,
            interval: Duration::from_secs(interval),
        })
    }

    pub async fn complete_device_login(
        &self,
        authorization: CodexDeviceAuthorization,
        store: &CodexCredentialStore,
        allowed_workspaces: Option<&[String]>,
        cancellation: &CancellationToken,
    ) -> Result<CodexCredentials, CodexAuthError> {
        let deadline = tokio::time::Instant::now() + DEVICE_FLOW_TIMEOUT;
        let poll_endpoint = self.endpoints.endpoint("api/accounts/deviceauth/token");

        let code = loop {
            if tokio::time::Instant::now() >= deadline {
                return Err(CodexAuthError::DeviceCodeExpired);
            }
            let response = tokio::select! {
                () = cancellation.cancelled() => return Err(CodexAuthError::Cancelled),
                response = self.http.post(poll_endpoint.clone()).json(&DeviceTokenPoll {
                    device_auth_id: &authorization.device_auth_id,
                    user_code: &authorization.user_code,
                }).send() => response?,
            };

            if response.status().is_success() {
                let code: DeviceCodeSuccess = response.json().await?;
                validate_server_pkce(&code.code_verifier, &code.code_challenge)?;
                break code;
            }
            if response.status() == reqwest::StatusCode::FORBIDDEN
                || response.status() == reqwest::StatusCode::NOT_FOUND
            {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                let delay = authorization.interval.min(remaining);
                tokio::select! {
                    () = cancellation.cancelled() => return Err(CodexAuthError::Cancelled),
                    () = tokio::time::sleep(delay) => {}
                }
                continue;
            }
            let status = response.status();
            let error_code = response
                .json::<OAuthErrorBody>()
                .await
                .ok()
                .and_then(|body| body.code());
            return match error_code.as_deref() {
                Some("authorization_declined" | "access_denied") => {
                    Err(CodexAuthError::DeviceCodeDenied)
                }
                Some("expired_token" | "device_code_expired") => {
                    Err(CodexAuthError::DeviceCodeExpired)
                }
                _ => Err(CodexAuthError::HttpStatus(status.as_u16())),
            };
        };

        let redirect_uri = self.endpoints.endpoint("deviceauth/callback").to_string();
        let tokens = self
            .exchange_code(&code.authorization_code, &redirect_uri, &code.code_verifier)
            .await?;
        persist_login_tokens(tokens, store, allowed_workspaces).await
    }

    pub(crate) async fn refresh_tokens(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, CodexAuthError> {
        let response = self
            .http
            .post(self.endpoints.endpoint("oauth/token"))
            .json(&RefreshRequest {
                client_id: &self.client_id,
                grant_type: "refresh_token",
                refresh_token,
            })
            .send()
            .await?;
        if response.status().is_success() {
            return response.json().await.map_err(CodexAuthError::from);
        }
        let status = response.status();
        let code = response
            .json::<OAuthErrorBody>()
            .await
            .ok()
            .and_then(|body| body.code());
        if status == reqwest::StatusCode::UNAUTHORIZED
            || matches!(
                code.as_deref(),
                Some(
                    "invalid_grant"
                        | "refresh_token_expired"
                        | "refresh_token_reused"
                        | "refresh_token_invalidated"
                )
            )
        {
            Err(CodexAuthError::RefreshRejected)
        } else {
            Err(CodexAuthError::HttpStatus(status.as_u16()))
        }
    }

    pub(crate) async fn revoke(
        &self,
        credentials: &CodexCredentials,
    ) -> Result<(), CodexAuthError> {
        let refresh_result = self
            .revoke_token(
                credentials.refresh_token(),
                "refresh_token",
                Some(&self.client_id),
            )
            .await;
        // Attempt access-token invalidation even when refresh revocation fails;
        // local logout remains best-effort and always proceeds to deletion.
        let access_result = self
            .revoke_token(credentials.access_token(), "access_token", None)
            .await;
        refresh_result.and(access_result)
    }

    async fn revoke_token(
        &self,
        token: &str,
        token_type_hint: &'static str,
        client_id: Option<&str>,
    ) -> Result<(), CodexAuthError> {
        let response = self
            .http
            .post(self.endpoints.endpoint("oauth/revoke"))
            .json(&RevokeRequest {
                token,
                token_type_hint,
                client_id,
            })
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(CodexAuthError::HttpStatus(response.status().as_u16()))
        }
    }

    async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
        verifier: &str,
    ) -> Result<TokenResponse, CodexAuthError> {
        let response = self
            .http
            .post(self.endpoints.endpoint("oauth/token"))
            .form(&AuthorizationCodeExchange {
                grant_type: "authorization_code",
                code,
                redirect_uri,
                client_id: &self.client_id,
                code_verifier: verifier,
            })
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(CodexAuthError::HttpStatus(response.status().as_u16()));
        }
        response.json().await.map_err(CodexAuthError::from)
    }

    fn authorization_url(
        &self,
        redirect_uri: &str,
        challenge: &str,
        state: &str,
        allowed_workspaces: Option<&[String]>,
    ) -> Url {
        let mut url = self.endpoints.endpoint("oauth/authorize");
        {
            let mut query = url.query_pairs_mut();
            query
                .append_pair("response_type", "code")
                .append_pair("client_id", &self.client_id)
                .append_pair("redirect_uri", redirect_uri)
                .append_pair("scope", OPENAI_CODEX_SCOPES)
                .append_pair("code_challenge", challenge)
                .append_pair("code_challenge_method", "S256")
                .append_pair("id_token_add_organizations", "true")
                .append_pair("codex_cli_simplified_flow", "true")
                .append_pair("state", state)
                .append_pair("originator", OPENAI_CODEX_ORIGINATOR);
            if let Some(workspaces) = allowed_workspaces
                && !workspaces.is_empty()
            {
                query.append_pair("allowed_workspace_id", &workspaces.join(","));
            }
        }
        url
    }
}

pub struct PendingCodexBrowserLogin {
    oauth: CodexOAuthClient,
    listener: TcpListener,
    redirect_uri: String,
    pkce: Pkce,
    state: String,
    authorization_url: Url,
    allowed_workspaces: Option<Vec<String>>,
}

impl fmt::Debug for PendingCodexBrowserLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingCodexBrowserLogin")
            .field("authorization_origin", &self.authorization_url.origin())
            .field("has_allowed_workspaces", &self.allowed_workspaces.is_some())
            .finish()
    }
}

impl PendingCodexBrowserLogin {
    pub fn authorization_url(&self) -> &Url {
        &self.authorization_url
    }

    pub fn open_browser(&self) -> std::io::Result<()> {
        webbrowser::open(self.authorization_url.as_str()).map(|_| ())
    }

    pub async fn complete(
        self,
        store: &CodexCredentialStore,
        cancellation: &CancellationToken,
    ) -> Result<CodexCredentials, CodexAuthError> {
        let Self {
            oauth,
            listener,
            redirect_uri,
            pkce,
            state,
            authorization_url: _,
            allowed_workspaces,
        } = self;
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let callback_state = CallbackState {
            expected_state: Arc::from(state),
            tx,
        };
        let app = Router::new()
            .route("/auth/callback", get(handle_browser_callback))
            .with_state(callback_state)
            // Extractor rejections and router-generated 404/405 responses never
            // enter `handle_browser_callback`; apply the privacy contract to
            // every response emitted by this loopback-only server.
            .layer(middleware::from_fn(add_browser_callback_privacy_headers));
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        let callback = tokio::select! {
            () = cancellation.cancelled() => Err(CodexAuthError::Cancelled),
            result = tokio::time::timeout(CALLBACK_TIMEOUT, rx.recv()) => match result {
                Ok(Some(result)) => result,
                Ok(None) => Err(CodexAuthError::AuthorizationDenied),
                Err(_) => Err(CodexAuthError::CallbackTimeout),
            }
        };
        let _ = shutdown_tx.send(());
        let _ = server.await;
        let code = callback?;

        let tokens = oauth
            .exchange_code(&code, &redirect_uri, &pkce.verifier)
            .await?;
        persist_login_tokens(tokens, store, allowed_workspaces.as_deref()).await
    }
}

#[derive(Clone)]
struct CallbackState {
    expected_state: Arc<str>,
    tx: tokio::sync::mpsc::Sender<Result<String, CodexAuthError>>,
}

fn set_browser_callback_privacy_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
}

fn browser_callback_response(status: StatusCode, message: &'static str) -> Response {
    let mut response = (status, Html(message)).into_response();
    set_browser_callback_privacy_headers(&mut response);
    response
}

async fn add_browser_callback_privacy_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    set_browser_callback_privacy_headers(&mut response);
    response
}

async fn handle_browser_callback(
    State(state): State<CallbackState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let received_state = query.get("state").map(String::as_str);
    if received_state != Some(state.expected_state.as_ref()) {
        // A stray or attacker-controlled request must not consume/cancel the
        // real pending login. Ignore it and keep waiting for the valid state.
        return browser_callback_response(
            StatusCode::BAD_REQUEST,
            "Sign-in could not be verified. Return to Grok Build and try again.",
        );
    }
    if query.get("error").is_some() {
        let _ = state.tx.try_send(Err(CodexAuthError::AuthorizationDenied));
        return browser_callback_response(
            StatusCode::FORBIDDEN,
            "Sign-in was not completed. Return to Grok Build.",
        );
    }
    let Some(code) = query.get("code").filter(|code| !code.is_empty()) else {
        let _ = state.tx.try_send(Err(CodexAuthError::AuthorizationDenied));
        return browser_callback_response(
            StatusCode::BAD_REQUEST,
            "Sign-in returned no authorization code. Return to Grok Build.",
        );
    };
    if state.tx.try_send(Ok(code.clone())).is_err() {
        return browser_callback_response(
            StatusCode::CONFLICT,
            "This sign-in callback was already used.",
        );
    }
    browser_callback_response(
        StatusCode::OK,
        "Signed in. You can close this window and return to Grok Build.",
    )
}

async fn bind_callback_listener() -> Result<TcpListener, CodexAuthError> {
    for port in [PRIMARY_CALLBACK_PORT, FALLBACK_CALLBACK_PORT] {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        match TcpListener::bind(address).await {
            Ok(listener) => return Ok(listener),
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(error) => return Err(CodexAuthError::Storage(error)),
        }
    }
    Err(CodexAuthError::Storage(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "OpenAI Codex callback ports 1455 and 1457 are both in use",
    )))
}

async fn persist_login_tokens(
    tokens: TokenResponse,
    store: &CodexCredentialStore,
    allowed_workspaces: Option<&[String]>,
) -> Result<CodexCredentials, CodexAuthError> {
    let _provider_lock = store.acquire_provider_lock().await?;
    let previous = store.load()?;
    let mut credentials = CodexCredentials::from_token_response(
        tokens,
        /*previous refresh family*/ None,
        allowed_workspaces,
    )?;
    if let Some(previous) = previous {
        let same_identity = credentials.same_identity(&previous);
        credentials.revision = previous.revision.saturating_add(1);
        if same_identity {
            credentials.created_at = previous.created_at;
            credentials
                .credential_id
                .clone_from(&previous.credential_id);
        }
        credentials.additional_fields = previous.additional_fields;
    }
    store.save_locked(credentials.clone()).await?;
    Ok(credentials)
}

struct Pkce {
    verifier: String,
    challenge: String,
}

impl Pkce {
    fn generate() -> Self {
        let verifier = random_urlsafe(64);
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Self {
            verifier,
            challenge,
        }
    }
}

fn random_urlsafe(bytes: usize) -> String {
    let mut random = vec![0_u8; bytes];
    rand::rng().fill_bytes(&mut random);
    URL_SAFE_NO_PAD.encode(random)
}

fn validate_server_pkce(verifier: &str, challenge: &str) -> Result<(), CodexAuthError> {
    let calculated = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    if calculated == challenge {
        Ok(())
    } else {
        Err(CodexAuthError::InvalidTokenResponse(
            "device authorization returned inconsistent PKCE values",
        ))
    }
}

#[derive(Serialize)]
struct AuthorizationCodeExchange<'a> {
    grant_type: &'static str,
    code: &'a str,
    redirect_uri: &'a str,
    client_id: &'a str,
    code_verifier: &'a str,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    client_id: &'a str,
    grant_type: &'static str,
    refresh_token: &'a str,
}

#[derive(Serialize)]
struct RevokeRequest<'a> {
    token: &'a str,
    token_type_hint: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<&'a str>,
}

#[derive(Serialize)]
struct DeviceCodeRequest<'a> {
    client_id: &'a str,
}

#[derive(Serialize)]
struct DeviceTokenPoll<'a> {
    device_auth_id: &'a str,
    user_code: &'a str,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_auth_id: String,
    #[serde(alias = "usercode")]
    user_code: String,
    #[serde(default, deserialize_with = "deserialize_interval")]
    interval: u64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(u64),
}

fn deserialize_interval<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(value) => value.trim().parse().map_err(serde::de::Error::custom),
        StringOrNumber::Number(value) => Ok(value),
    }
}

#[derive(Deserialize)]
struct DeviceCodeSuccess {
    authorization_code: String,
    code_challenge: String,
    code_verifier: String,
}

#[derive(Deserialize)]
struct OAuthErrorBody {
    #[serde(default)]
    error: Option<serde_json::Value>,
}

impl OAuthErrorBody {
    fn code(self) -> Option<String> {
        match self.error? {
            serde_json::Value::String(code) => Some(code),
            serde_json::Value::Object(object) => object
                .get("code")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            _ => None,
        }
    }
}

pub struct CodexDeviceAuthorization {
    verification_url: Url,
    user_code: String,
    device_auth_id: String,
    interval: Duration,
}

impl CodexDeviceAuthorization {
    pub fn verification_url(&self) -> &Url {
        &self.verification_url
    }

    pub fn user_code(&self) -> &str {
        &self.user_code
    }
}

impl fmt::Debug for CodexDeviceAuthorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexDeviceAuthorization")
            .field("verification_origin", &self.verification_url.origin())
            .field("user_code", &"<redacted>")
            .field("device_auth_id", &"<redacted>")
            .field("interval", &self.interval)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::codex::credentials::jwt_for_test;
    use axum::body::Body;
    use axum::extract::{Form, Json};
    use axum::routing::post;
    use tower::ServiceExt as _;

    fn login_tokens(account: &str, refresh: &str) -> TokenResponse {
        TokenResponse {
            id_token: Some(jwt_for_test(serde_json::json!({
                "https://api.openai.com/auth": { "chatgpt_account_id": account }
            }))),
            access_token: Some(jwt_for_test(serde_json::json!({
                "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
                "https://api.openai.com/auth": { "chatgpt_account_id": account }
            }))),
            refresh_token: Some(refresh.to_owned()),
            expires_in: Some(3600),
        }
    }

    fn assert_private_callback_response(response: &Response, status: StatusCode) {
        assert_eq!(response.status(), status);
        assert_eq!(
            response
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("no-store")
        );
        assert_eq!(
            response
                .headers()
                .get(REFERRER_POLICY)
                .and_then(|value| value.to_str().ok()),
            Some("no-referrer")
        );
    }

    fn callback_router_for_test() -> Router {
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        Router::new()
            .route("/auth/callback", get(handle_browser_callback))
            .with_state(CallbackState {
                expected_state: Arc::from("expected"),
                tx,
            })
            .layer(middleware::from_fn(add_browser_callback_privacy_headers))
    }

    async fn callback_router_response(method: &str, uri: &str) -> Response {
        callback_router_for_test()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .expect("valid callback test request"),
            )
            .await
            .expect("callback router is infallible")
    }

    #[tokio::test]
    async fn router_generated_callback_errors_set_privacy_headers() {
        for (method, uri, status) in [
            ("GET", "/auth/callback?state=%FF", StatusCode::BAD_REQUEST),
            ("GET", "/not-the-callback", StatusCode::NOT_FOUND),
            ("POST", "/auth/callback", StatusCode::METHOD_NOT_ALLOWED),
        ] {
            let response = callback_router_response(method, uri).await;
            assert_private_callback_response(&response, status);
        }
    }

    #[test]
    fn authorization_url_matches_current_codex_contract() {
        let oauth = CodexOAuthClient::for_test(Url::parse("https://auth.example.test/").unwrap());
        let pkce = Pkce::generate();
        let url = oauth.authorization_url(
            "http://localhost:1455/auth/callback",
            &pkce.challenge,
            "state-value",
            Some(&["workspace-one".to_owned(), "workspace-two".to_owned()]),
        );
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();

        assert_eq!(
            url.as_str().split('?').next(),
            Some("https://auth.example.test/oauth/authorize")
        );
        assert_eq!(query.get("response_type").map(String::as_str), Some("code"));
        assert_eq!(
            query.get("client_id").map(String::as_str),
            Some("test-client")
        );
        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some("http://localhost:1455/auth/callback")
        );
        assert_eq!(
            query.get("scope").map(String::as_str),
            Some(OPENAI_CODEX_SCOPES)
        );
        assert_eq!(
            query.get("code_challenge_method").map(String::as_str),
            Some("S256")
        );
        assert_eq!(
            query.get("id_token_add_organizations").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            query.get("codex_cli_simplified_flow").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            query.get("originator").map(String::as_str),
            Some(OPENAI_CODEX_ORIGINATOR)
        );
        assert_eq!(query.get("state").map(String::as_str), Some("state-value"));
        assert_eq!(
            query.get("allowed_workspace_id").map(String::as_str),
            Some("workspace-one,workspace-two")
        );
        validate_server_pkce(&pkce.verifier, &pkce.challenge).unwrap();
        assert!(pkce.verifier.len() >= 43);
        assert_ne!(Pkce::generate().verifier, pkce.verifier);
    }

    #[tokio::test]
    async fn invalid_callback_state_is_ignored_then_valid_state_completes() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let state = CallbackState {
            expected_state: Arc::from("expected"),
            tx,
        };
        let response = handle_browser_callback(
            State(state.clone()),
            Query(HashMap::from([
                ("state".to_owned(), "attacker".to_owned()),
                ("code".to_owned(), "attacker-code".to_owned()),
            ])),
        )
        .await;
        assert_private_callback_response(&response, StatusCode::BAD_REQUEST);
        assert!(matches!(
            rx.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty)
        ));

        let response = handle_browser_callback(
            State(state),
            Query(HashMap::from([
                ("state".to_owned(), "expected".to_owned()),
                ("code".to_owned(), "valid-code".to_owned()),
            ])),
        )
        .await;
        assert_private_callback_response(&response, StatusCode::OK);
        assert_eq!(rx.recv().await.unwrap().unwrap(), "valid-code");
    }

    #[tokio::test]
    async fn every_terminal_callback_branch_sets_privacy_headers() {
        let (denied_tx, mut denied_rx) = tokio::sync::mpsc::channel(1);
        let denied = handle_browser_callback(
            State(CallbackState {
                expected_state: Arc::from("expected"),
                tx: denied_tx,
            }),
            Query(HashMap::from([
                ("state".to_owned(), "expected".to_owned()),
                ("error".to_owned(), "denied".to_owned()),
            ])),
        )
        .await;
        assert_private_callback_response(&denied, StatusCode::FORBIDDEN);
        assert!(matches!(
            denied_rx.try_recv(),
            Ok(Err(CodexAuthError::AuthorizationDenied))
        ));

        let (missing_tx, mut missing_rx) = tokio::sync::mpsc::channel(1);
        let missing = handle_browser_callback(
            State(CallbackState {
                expected_state: Arc::from("expected"),
                tx: missing_tx,
            }),
            Query(HashMap::from([("state".to_owned(), "expected".to_owned())])),
        )
        .await;
        assert_private_callback_response(&missing, StatusCode::BAD_REQUEST);
        assert!(matches!(
            missing_rx.try_recv(),
            Ok(Err(CodexAuthError::AuthorizationDenied))
        ));

        let (used_tx, used_rx) = tokio::sync::mpsc::channel(1);
        drop(used_rx);
        let used = handle_browser_callback(
            State(CallbackState {
                expected_state: Arc::from("expected"),
                tx: used_tx,
            }),
            Query(HashMap::from([
                ("state".to_owned(), "expected".to_owned()),
                ("code".to_owned(), "unused-code".to_owned()),
            ])),
        )
        .await;
        assert_private_callback_response(&used, StatusCode::CONFLICT);
    }

    #[test]
    fn device_interval_accepts_strings_and_numbers() {
        let from_string: DeviceCodeResponse = serde_json::from_value(serde_json::json!({
            "device_auth_id": "id",
            "usercode": "code",
            "interval": "7"
        }))
        .unwrap();
        let from_number: DeviceCodeResponse = serde_json::from_value(serde_json::json!({
            "device_auth_id": "id",
            "user_code": "code",
            "interval": 9
        }))
        .unwrap();
        let without_interval: DeviceCodeResponse = serde_json::from_value(serde_json::json!({
            "device_auth_id": "id",
            "user_code": "code"
        }))
        .unwrap();
        assert_eq!(from_string.interval, 7);
        assert_eq!(from_number.interval, 9);
        assert_eq!(without_interval.interval, 0);
    }

    #[tokio::test]
    async fn relogin_preserves_opaque_id_only_for_same_account() {
        let dir = tempfile::tempdir().unwrap();
        let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
        let first = persist_login_tokens(login_tokens("acct-one", "refresh-one"), &store, None)
            .await
            .unwrap();
        let same_account =
            persist_login_tokens(login_tokens("acct-one", "refresh-two"), &store, None)
                .await
                .unwrap();
        assert_eq!(same_account.credential_id(), first.credential_id());
        assert_eq!(same_account.revision, first.revision + 1);

        let switched =
            persist_login_tokens(login_tokens("acct-two", "refresh-three"), &store, None)
                .await
                .unwrap();
        assert_ne!(switched.credential_id(), same_account.credential_id());
        assert_eq!(switched.account_id(), "acct-two");
        assert_eq!(switched.revision, same_account.revision + 1);
    }

    #[tokio::test]
    async fn device_flow_validates_pkce_exchanges_code_and_persists() {
        let verifier = "device-verifier-value".to_owned();
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        let access = jwt_for_test(serde_json::json!({
            "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
            "https://api.openai.com/auth": { "chatgpt_account_id": "acct-device" }
        }));
        let id = jwt_for_test(serde_json::json!({
            "https://api.openai.com/auth": { "chatgpt_account_id": "acct-device" }
        }));
        let app = Router::new()
            .route(
                "/api/accounts/deviceauth/usercode",
                post(|| async {
                    Json(serde_json::json!({
                        "device_auth_id": "device-id",
                        "usercode": "USER-CODE",
                        "interval": "0"
                    }))
                }),
            )
            .route(
                "/api/accounts/deviceauth/token",
                post({
                    let verifier = verifier.clone();
                    let challenge = challenge.clone();
                    move |Json(body): Json<serde_json::Value>| {
                        let verifier = verifier.clone();
                        let challenge = challenge.clone();
                        async move {
                            assert_eq!(body["device_auth_id"], "device-id");
                            assert_eq!(body["user_code"], "USER-CODE");
                            Json(serde_json::json!({
                                "authorization_code": "authorization-code",
                                "code_verifier": verifier,
                                "code_challenge": challenge
                            }))
                        }
                    }
                }),
            )
            .route(
                "/oauth/token",
                post({
                    let access = access.clone();
                    let id = id.clone();
                    let verifier = verifier.clone();
                    move |Form(form): Form<HashMap<String, String>>| {
                        let access = access.clone();
                        let id = id.clone();
                        let verifier = verifier.clone();
                        async move {
                            assert_eq!(
                                form.get("grant_type").map(String::as_str),
                                Some("authorization_code")
                            );
                            assert_eq!(
                                form.get("code").map(String::as_str),
                                Some("authorization-code")
                            );
                            assert_eq!(
                                form.get("client_id").map(String::as_str),
                                Some("test-client")
                            );
                            assert_eq!(form.get("code_verifier"), Some(&verifier));
                            Json(serde_json::json!({
                                "access_token": access,
                                "id_token": id,
                                "refresh_token": "refresh-device",
                                "expires_in": 3600
                            }))
                        }
                    }
                }),
            );
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let oauth = CodexOAuthClient::for_test(Url::parse(&format!("http://{address}/")).unwrap());
        let authorization = oauth.request_device_authorization().await.unwrap();
        assert_eq!(authorization.user_code(), "USER-CODE");
        assert_eq!(authorization.interval, Duration::from_secs(1));
        let dir = tempfile::tempdir().unwrap();
        let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
        let credentials = oauth
            .complete_device_login(authorization, &store, None, &CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(credentials.account_id(), "acct-device");
        assert_eq!(store.load().unwrap().unwrap().account_id(), "acct-device");

        server.abort();
    }
}
