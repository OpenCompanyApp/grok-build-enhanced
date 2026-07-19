use futures_util::StreamExt as _;
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};

use super::error::WebFetchError;
use crate::types::{KIMI_CODE_PROVIDER_ID, SharedApiKeyProvider, resolve_kimi_code_request_auth};

pub(super) const KIMI_CODE_BASE_URL: &str = "https://api.kimi.com/coding/v1";

pub(super) enum HostedFetchResult {
    Content(String),
    Fallback,
}

#[derive(Clone)]
pub(super) struct KimiHostedFetch {
    http: reqwest::Client,
    endpoint: String,
    auth_provider: SharedApiKeyProvider,
    max_content_length: usize,
}

impl KimiHostedFetch {
    pub(super) fn new(
        base_url: &str,
        auth_provider: SharedApiKeyProvider,
        max_content_length: usize,
    ) -> Result<Self, WebFetchError> {
        if auth_provider.request_auth_provider_id() != Some(KIMI_CODE_PROVIDER_ID) {
            return Err(WebFetchError::HostedAuthentication);
        }
        let endpoint = validate_base_url(base_url)?;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .map_err(|_| WebFetchError::ClientBuildError)?;
        Ok(Self {
            http,
            endpoint,
            auth_provider,
            max_content_length,
        })
    }

    pub(super) async fn fetch(&self, url: &url::Url) -> Result<HostedFetchResult, WebFetchError> {
        let auth = resolve_kimi_code_request_auth(&self.auth_provider)
            .await
            .map_err(|_| WebFetchError::HostedAuthentication)?;
        let response = match auth
            .apply(self.http.post(&self.endpoint))
            .header(ACCEPT, "text/markdown")
            .header(CONTENT_TYPE, "application/json")
            .header(
                USER_AGENT,
                format!("grok-agent/{}", env!("CARGO_PKG_VERSION")),
            )
            .json(&serde_json::json!({"url": url.as_str()}))
            .send()
            .await
        {
            Ok(response) => response,
            // Hosted transport/service failures fall back to the existing
            // SSRF-safe local fetch. Credential and quota failures below do
            // not, because masking them would make provider testing lie.
            Err(_) => return Ok(HostedFetchResult::Fallback),
        };
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(WebFetchError::HostedAuthentication);
        }
        if matches!(status.as_u16(), 402 | 403) {
            return Err(WebFetchError::HostedMembership);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(WebFetchError::HostedQuota);
        }
        if status == reqwest::StatusCode::REQUEST_TIMEOUT {
            return Ok(HostedFetchResult::Fallback);
        }
        if status.is_client_error() {
            // A validated URL producing another 4xx indicates request-contract
            // rejection, not a transient hosted-service outage. Do not hide a
            // provider integration failure behind a successful local fetch.
            return Err(WebFetchError::HostedRequestRejected {
                status: status.as_u16(),
            });
        }
        if !status.is_success() {
            return Ok(HostedFetchResult::Fallback);
        }
        let bytes = match read_response_body(response, self.max_content_length).await {
            Some(bytes) => bytes,
            None => return Ok(HostedFetchResult::Fallback),
        };
        let content = match String::from_utf8(bytes) {
            Ok(content) => content,
            Err(_) => return Ok(HostedFetchResult::Fallback),
        };
        Ok(HostedFetchResult::Content(content))
    }
}

async fn read_response_body(response: reqwest::Response, max_bytes: usize) -> Option<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return None;
    }

    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ok()?;
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return None;
        }
        body.try_reserve_exact(chunk.len()).ok()?;
        body.extend_from_slice(&chunk);
    }
    Some(body)
}

fn validate_base_url(base_url: &str) -> Result<String, WebFetchError> {
    let normalized = base_url.trim_end_matches('/');
    #[cfg(any(test, feature = "test-support"))]
    let allowed = normalized == KIMI_CODE_BASE_URL
        || reqwest::Url::parse(normalized).is_ok_and(|url| {
            url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost"))
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
        });
    #[cfg(not(any(test, feature = "test-support")))]
    let allowed = normalized == KIMI_CODE_BASE_URL;
    if !allowed {
        return Err(WebFetchError::HostedAuthentication);
    }
    Ok(format!("{normalized}/fetch"))
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    use super::*;
    use crate::types::{ApiKeyProvider, RequestAuth, RequestCredentialSnapshot};
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct KimiTestProvider;

    impl ApiKeyProvider for KimiTestProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(KIMI_CODE_PROVIDER_ID)
        }

        fn current_request_auth_async(
            &self,
        ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
            Box::pin(std::future::ready(Some(
                RequestAuth::for_provider_snapshot(
                    KIMI_CODE_PROVIDER_ID,
                    RequestCredentialSnapshot::new("opaque-kimi-record", 1),
                    [(
                        "authorization".to_owned(),
                        "Bearer sentinel-kimi-key".to_owned(),
                    )],
                ),
            )))
        }
    }

    fn test_client(server: &MockServer) -> KimiHostedFetch {
        let provider: SharedApiKeyProvider = Arc::new(KimiTestProvider);
        KimiHostedFetch::new(&server.uri(), provider, 1024).unwrap()
    }

    #[tokio::test]
    async fn fetch_returns_markdown_when_the_hosted_service_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .and(body_json(serde_json::json!({
                "url": "https://example.com/article"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Hosted markdown"))
            .mount(&server)
            .await;

        let outcome = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            HostedFetchResult::Content(content) if content == "# Hosted markdown"
        ));

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0]
                .headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer sentinel-kimi-key")
        );
        assert_eq!(
            requests[0]
                .headers
                .get("accept")
                .and_then(|value| value.to_str().ok()),
            Some("text/markdown")
        );
    }

    #[tokio::test]
    async fn fetch_requests_local_fallback_for_an_ordinary_service_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let outcome = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .unwrap();
        assert!(matches!(outcome, HostedFetchResult::Fallback));
    }

    #[tokio::test]
    async fn fetch_requests_local_fallback_when_the_hosted_service_times_out() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(ResponseTemplate::new(408))
            .mount(&server)
            .await;

        let outcome = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .unwrap();

        assert!(matches!(outcome, HostedFetchResult::Fallback));
    }

    #[tokio::test]
    async fn fetch_surfaces_request_rejection_without_local_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;

        let error = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .err()
            .expect("request-contract failures must remain explicit");

        assert!(matches!(
            error,
            WebFetchError::HostedRequestRejected { status: 400 }
        ));
    }

    #[tokio::test]
    async fn fetch_surfaces_authentication_failure_without_local_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let error = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .err()
            .expect("authentication failures must remain explicit");
        assert!(matches!(error, WebFetchError::HostedAuthentication));
    }

    #[tokio::test]
    async fn fetch_surfaces_membership_failure_without_local_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&server)
            .await;

        let error = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .err()
            .expect("membership failures must remain explicit");
        assert!(matches!(error, WebFetchError::HostedMembership));
    }

    #[tokio::test]
    async fn fetch_surfaces_payment_membership_failure_without_local_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(ResponseTemplate::new(402))
            .mount(&server)
            .await;

        let error = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .err()
            .expect("payment membership failures must remain explicit");
        assert!(matches!(error, WebFetchError::HostedMembership));
    }

    #[tokio::test]
    async fn fetch_surfaces_quota_failure_without_local_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let error = test_client(&server)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .err()
            .expect("quota failures must remain explicit");
        assert!(matches!(error, WebFetchError::HostedQuota));
    }

    #[tokio::test]
    async fn fetch_does_not_follow_a_redirect_to_a_credential_sink() {
        let sink = MockServer::start().await;
        let source = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/fetch"))
            .respond_with(
                ResponseTemplate::new(307)
                    .insert_header("location", format!("{}/credential-sink", sink.uri())),
            )
            .mount(&source)
            .await;

        let outcome = test_client(&source)
            .fetch(&url::Url::parse("https://example.com/article").unwrap())
            .await
            .unwrap();
        assert!(matches!(outcome, HostedFetchResult::Fallback));
        assert!(sink.received_requests().await.unwrap().is_empty());
    }
}
