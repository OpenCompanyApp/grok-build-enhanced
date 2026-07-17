use async_openai::types::responses as rs;
use indexmap::IndexMap;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};

use super::{
    BackendSearchResult, execution_error, validate_allowed_domains, validate_search_commands,
    validate_search_query,
};
use crate::attribution::{SharedAttributionCallback, ToolConsumer};
use crate::types::SharedApiKeyProvider;

const MAX_RESPONSES_SEARCH_RESPONSE_BYTES: usize = 10 * 1024 * 1024;
const MAX_RESPONSES_CITATIONS: usize = 64;
const MAX_RESPONSES_CITATION_TITLE_CHARS: usize = 512;
const MAX_RESPONSES_CITATION_URL_BYTES: usize = 4 * 1024;

/// xAI/custom-provider web search implemented with the hosted Responses tool.
#[derive(Clone)]
pub(in crate::implementations::web_search) struct ResponsesBackend {
    http: reqwest::Client,
    base_url: String,
    model: String,
    api_key_provider: Option<SharedApiKeyProvider>,
    attribution_callback: Option<SharedAttributionCallback>,
}

impl ResponsesBackend {
    pub(in crate::implementations::web_search) fn new(
        api_key: &str,
        base_url: &str,
        model: &str,
        extra_headers: &IndexMap<String, String>,
        api_key_provider: Option<SharedApiKeyProvider>,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        if api_key_provider.as_ref().is_some_and(|provider| {
            provider.request_auth_provider_id() == Some(crate::types::OPENAI_CODEX_PROVIDER_ID)
        }) {
            return Err(execution_error(
                "Responses web search cannot use Codex provider authentication",
            ));
        }

        let base_url = validate_responses_base_url(base_url)?;
        validate_responses_identifier(model)?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|_| execution_error("Responses authentication is invalid"))?;
        authorization.set_sensitive(true);
        headers.insert(AUTHORIZATION, authorization);

        for (key, value) in extra_headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|_| execution_error("Responses provider headers are invalid"))?;
            let mut header_value = HeaderValue::from_str(value)
                .map_err(|_| execution_error("Responses provider headers are invalid"))?;
            // Extra provider headers frequently carry API credentials. Treat
            // every configured value as sensitive rather than guessing from
            // a mutable list of provider-specific names.
            header_value.set_sensitive(true);
            headers.insert(header_name, header_value);
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            // Never replay configured credentials to a redirect target.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| execution_error("Responses web search HTTP client could not be built"))?;

        Ok(Self {
            http,
            base_url,
            model: model.to_owned(),
            api_key_provider,
            attribution_callback: None,
        })
    }

    pub(in crate::implementations::web_search) fn set_attribution_callback(
        &mut self,
        callback: Option<SharedAttributionCallback>,
    ) {
        self.attribution_callback = callback;
    }

    pub(in crate::implementations::web_search) async fn search(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        validate_search_query(query)?;
        let allowed_domains = validate_allowed_domains(allowed_domains)?;
        self.execute(query, allowed_domains).await
    }

    pub(in crate::implementations::web_search) async fn run_commands(
        &self,
        commands: &serde_json::Value,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        validate_search_commands(commands)?;
        let query = commands
            .get("search_query")
            .and_then(serde_json::Value::as_array)
            .and_then(|queries| queries.first())
            .and_then(|query| query.get("q"))
            .and_then(serde_json::Value::as_str)
            .filter(|query| !query.trim().is_empty())
            .ok_or_else(navigation_not_supported)?;

        if commands.as_object().is_some_and(|commands| {
            commands.keys().any(|key| key != "search_query")
                || commands
                    .get("search_query")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|queries| queries.len() != 1)
        }) {
            return Err(navigation_not_supported());
        }

        validate_search_query(query)?;
        let allowed_domains = validate_allowed_domains(allowed_domains)?;
        self.execute(query, allowed_domains).await
    }

    async fn execute(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        let web_search = rs::WebSearchToolArgs::default()
            .filters(rs::WebSearchToolFilters { allowed_domains })
            .build()
            .map_err(|_| execution_error("Responses web search request is invalid"))?;
        let request = rs::CreateResponseArgs::default()
            .model(self.model.clone())
            .input(query.to_owned())
            .tools(vec![rs::Tool::WebSearch(web_search)])
            .store(false)
            .temperature(0.1)
            .top_p(0.95)
            .max_output_tokens(8192u32)
            .build()
            .map_err(|_| execution_error("Responses web search request is invalid"))?;

        let url = format!("{}/responses", self.base_url.trim_end_matches('/'));
        let sent_bearer =
            crate::types::api_key_provider::resolve_bearer(self.api_key_provider.as_ref()).await;
        let mut request_builder = self.http.post(&url).json(&request);
        if let Some(key) = sent_bearer.as_ref() {
            let mut value = HeaderValue::from_str(&format!("Bearer {key}"))
                .map_err(|_| execution_error("Responses authentication is invalid"))?;
            value.set_sensitive(true);
            request_builder = request_builder.header(AUTHORIZATION, value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|_| execution_error("Responses web search request could not be sent"))?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            // A static configured bearer is always present even when no
            // dynamic override was resolved. Provider bodies are deliberately
            // neither read nor reflected across this error boundary.
            self.record_401_attribution(true);
            return Err(xai_tool_runtime::ToolError::unauthorized(
                "Responses web search authentication was rejected".to_owned(),
            )
            .with_details(serde_json::json!({
                "tool_id": "web_search",
                "status": 401,
            })));
        }
        if !status.is_success() {
            return Err(execution_error(format!(
                "Responses web search request failed with HTTP {status}"
            )));
        }

        let bytes = read_response_with_limit(response, MAX_RESPONSES_SEARCH_RESPONSE_BYTES).await?;
        let response: rs::Response = serde_json::from_slice(&bytes)
            .map_err(|_| execution_error("Responses web search returned an invalid response"))?;
        let content = response
            .output_text()
            .unwrap_or_else(|| "No search results found.".to_owned());

        Ok(BackendSearchResult {
            content,
            citation_pairs: extract_citation_pairs(&response),
            references: Vec::new(),
        })
    }

    fn record_401_attribution(&self, bearer_was_sent: bool) {
        if let Some(callback) = self.attribution_callback.as_ref() {
            callback.record_401(ToolConsumer::WebSearch, bearer_was_sent);
        }
    }
}

fn validate_responses_identifier(value: &str) -> Result<(), xai_tool_runtime::ToolError> {
    if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(execution_error(
            "Responses web search model configuration is invalid",
        ));
    }
    Ok(())
}

fn validate_responses_base_url(base_url: &str) -> Result<String, xai_tool_runtime::ToolError> {
    let url = reqwest::Url::parse(base_url)
        .map_err(|_| execution_error("Responses web search endpoint is invalid"))?;
    let safe_shape = url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none();
    if !safe_shape {
        return Err(execution_error("Responses web search endpoint is invalid"));
    }

    #[cfg(any(test, feature = "test-support"))]
    let test_loopback = url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"));
    #[cfg(not(any(test, feature = "test-support")))]
    let test_loopback = false;

    if url.scheme() != "https" && !test_loopback {
        return Err(execution_error(
            "Responses web search endpoint must use HTTPS",
        ));
    }
    if !test_loopback && url.host_str().is_some_and(is_local_or_private_host) {
        return Err(execution_error(
            "Responses web search endpoint is not permitted",
        ));
    }

    Ok(url.to_string().trim_end_matches('/').to_owned())
}

fn is_local_or_private_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return true;
    }
    let Ok(address) = host.parse::<std::net::IpAddr>() else {
        return false;
    };
    match address {
        std::net::IpAddr::V4(address) => {
            let octets = address.octets();
            address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_unspecified()
                || address.is_broadcast()
                || address.is_multicast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || octets[0] >= 240
        }
        std::net::IpAddr::V6(address) => {
            let first = address.segments()[0];
            address.is_loopback()
                || address.is_unspecified()
                || address.is_multicast()
                || first & 0xfe00 == 0xfc00
                || first & 0xffc0 == 0xfe80
        }
    }
}

async fn read_response_with_limit(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, xai_tool_runtime::ToolError> {
    use futures_util::StreamExt as _;

    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(execution_error(
            "Responses web search response exceeded the size limit",
        ));
    }

    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|_| execution_error("Responses web search response could not be read"))?;
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(execution_error(
                "Responses web search response exceeded the size limit",
            ));
        }
        body.try_reserve_exact(chunk.len())
            .map_err(|_| execution_error("Responses web search response could not be read"))?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn navigation_not_supported() -> xai_tool_runtime::ToolError {
    execution_error("This provider supports query web search but not Codex navigation commands")
}

/// Extract `(title, url)` pairs from Responses API URL annotations. URLs are
/// deduplicated while preserving first-seen order.
fn extract_citation_pairs(response: &rs::Response) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for output_item in &response.output {
        if pairs.len() >= MAX_RESPONSES_CITATIONS {
            break;
        }
        if let rs::OutputItem::Message(output_message) = output_item {
            for message_content in &output_message.content {
                if let rs::OutputMessageContent::OutputText(text_content) = message_content {
                    for annotation in &text_content.annotations {
                        if pairs.len() >= MAX_RESPONSES_CITATIONS {
                            break;
                        }
                        if let rs::Annotation::UrlCitation(url_citation) = annotation
                            && let Ok(json) = serde_json::to_value(url_citation)
                            && let Some(url) = json
                                .get("url")
                                .and_then(serde_json::Value::as_str)
                                .and_then(sanitize_responses_citation_url)
                            && seen.insert(url.clone())
                        {
                            let title = json
                                .get("title")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default()
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .chars()
                                .take(MAX_RESPONSES_CITATION_TITLE_CHARS)
                                .collect();
                            pairs.push((title, url));
                        }
                    }
                }
            }
        }
    }
    pairs
}

fn sanitize_responses_citation_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_RESPONSES_CITATION_URL_BYTES {
        return None;
    }
    let parsed = reqwest::Url::parse(value).ok()?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return None;
    }
    Some(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribution::Auth401AttributionCallback;
    use crate::types::ApiKeyProvider;
    use std::sync::{Arc, Mutex};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    fn success_response(text: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "resp_test",
            "object": "response",
            "created_at": 1,
            "status": "completed",
            "model": "test-model",
            "output": [{
                "type": "message",
                "id": "msg_1",
                "status": "completed",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": text,
                    "annotations": [],
                }],
            }],
        })
    }

    fn backend(server: &MockServer, provider: Option<SharedApiKeyProvider>) -> ResponsesBackend {
        ResponsesBackend::new(
            "sentinel-static-key",
            &server.uri(),
            "configured-search-model",
            &IndexMap::new(),
            provider,
        )
        .expect("Responses backend should build")
    }

    fn request_header_matches(request: &Request, name: &str, expected: &str) -> bool {
        request
            .headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            == Some(expected)
    }

    struct NoneProvider;

    impl ApiKeyProvider for NoneProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }
    }

    struct CodexMarkerProvider;

    impl ApiKeyProvider for CodexMarkerProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(crate::types::OPENAI_CODEX_PROVIDER_ID)
        }
    }

    #[test]
    fn responses_constructor_rejects_codex_provider_fallthrough() {
        let provider: SharedApiKeyProvider = Arc::new(CodexMarkerProvider);
        let error = ResponsesBackend::new(
            "sentinel-static-key",
            "https://example.com/v1",
            "configured-search-model",
            &IndexMap::new(),
            Some(provider),
        )
        .err()
        .expect("Responses must reject Codex provider identity")
        .to_string();
        assert!(error.contains("cannot use Codex provider"));
        assert!(!error.contains("sentinel-static-key"));
    }

    #[test]
    fn responses_constructor_rejects_unsafe_endpoints_without_reflection() {
        for rejected in [
            "http://example.com/v1",
            "https://sentinel-user:sentinel-pass@example.com/v1",
            "https://example.com/v1?token=sentinel-query",
            "https://example.com/v1#sentinel-fragment",
            "https://localhost/v1",
            "https://127.0.0.1/v1",
            "file:///tmp/provider",
        ] {
            let error = ResponsesBackend::new(
                "sentinel-static-key",
                rejected,
                "configured-search-model",
                &IndexMap::new(),
                None,
            )
            .err()
            .unwrap_or_else(|| panic!("unsafe Responses endpoint was accepted: {rejected}"))
            .to_string();
            for forbidden in [
                rejected,
                "sentinel-user",
                "sentinel-pass",
                "sentinel-query",
                "sentinel-fragment",
                "sentinel-static-key",
            ] {
                assert!(!error.contains(forbidden), "unexpected error: {error}");
            }
        }
    }

    #[test]
    fn responses_constructor_does_not_reflect_custom_header_names() {
        let headers = IndexMap::from([(
            "sentinel-invalid-header\n".to_owned(),
            "sentinel-header-value".to_owned(),
        )]);
        let error = ResponsesBackend::new(
            "sentinel-static-key",
            "https://example.com/v1",
            "configured-search-model",
            &headers,
            None,
        )
        .err()
        .expect("invalid custom header must fail")
        .to_string();
        for forbidden in [
            "sentinel-invalid-header",
            "sentinel-header-value",
            "sentinel-static-key",
        ] {
            assert!(!error.contains(forbidden));
        }
    }

    #[tokio::test]
    async fn static_key_remains_the_responses_fallback_and_model_is_preserved() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/responses"))
            .and(body_partial_json(serde_json::json!({
                "model": "configured-search-model",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(success_response("result")))
            .mount(&server)
            .await;

        let provider: SharedApiKeyProvider = Arc::new(NoneProvider);
        let result = backend(&server, Some(provider))
            .search("query", None)
            .await
            .expect("static fallback should succeed");
        assert_eq!(result.content, "result");
        let requests = server.received_requests().await.unwrap();
        assert!(request_header_matches(
            &requests[0],
            "authorization",
            "Bearer sentinel-static-key"
        ));
    }

    struct FreshProvider;

    impl ApiKeyProvider for FreshProvider {
        fn current_api_key(&self) -> Option<String> {
            Some("sentinel-fresh-key".to_owned())
        }
    }

    #[tokio::test]
    async fn dynamic_responses_key_overrides_the_static_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/responses"))
            .respond_with(ResponseTemplate::new(200).set_body_json(success_response("fresh")))
            .mount(&server)
            .await;

        let provider: SharedApiKeyProvider = Arc::new(FreshProvider);
        let result = backend(&server, Some(provider))
            .search("query", None)
            .await
            .expect("dynamic override should succeed");
        assert_eq!(result.content, "fresh");
        let requests = server.received_requests().await.unwrap();
        assert!(request_header_matches(
            &requests[0],
            "authorization",
            "Bearer sentinel-fresh-key"
        ));
    }

    #[tokio::test]
    async fn responses_redirects_are_terminal_and_do_not_forward_credentials() {
        let target = MockServer::start().await;
        let source = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/responses"))
            .respond_with(
                ResponseTemplate::new(307)
                    .insert_header("location", format!("{}/credential-sink", target.uri())),
            )
            .mount(&source)
            .await;

        let error = backend(&source, None)
            .search("redirect contract", None)
            .await
            .err()
            .expect("redirect must be terminal")
            .to_string();
        assert!(error.contains("307"));
        assert_eq!(source.received_requests().await.unwrap().len(), 1);
        assert!(target.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn navigation_commands_are_rejected_before_http_dispatch() {
        let server = MockServer::start().await;
        let error = backend(&server, None)
            .run_commands(
                &serde_json::json!({"open": [{"ref_id": "turn0search0"}]}),
                None,
            )
            .await
            .err()
            .expect("Responses must reject Codex-only commands")
            .to_string();
        assert!(error.contains("not Codex navigation commands"));
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[test]
    fn citation_projection_deduplicates_urls_and_preserves_first_title() {
        let response: rs::Response = serde_json::from_value(serde_json::json!({
            "id": "resp_test",
            "object": "response",
            "created_at": 1,
            "status": "completed",
            "model": "test-model",
            "output": [{
                "type": "message",
                "id": "msg_1",
                "status": "completed",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": "result",
                    "annotations": [
                        {"type": "url_citation", "url": "https://example.com/a", "title": "First", "start_index": 0, "end_index": 1},
                        {"type": "url_citation", "url": "https://example.com/a", "title": "Duplicate", "start_index": 2, "end_index": 3},
                        {"type": "url_citation", "url": "https://example.com/b", "title": "Second", "start_index": 4, "end_index": 5},
                        {"type": "url_citation", "url": "https://sentinel-user:sentinel-pass@example.com/private", "title": "Unsafe", "start_index": 6, "end_index": 7}
                    ],
                }],
            }],
        }))
        .expect("valid Responses fixture");

        assert_eq!(
            extract_citation_pairs(&response),
            vec![
                ("First".to_owned(), "https://example.com/a".to_owned()),
                ("Second".to_owned(), "https://example.com/b".to_owned()),
            ]
        );
    }

    #[derive(Debug, Default)]
    struct CountingCallback {
        calls: Mutex<Vec<(ToolConsumer, bool)>>,
    }

    impl Auth401AttributionCallback for CountingCallback {
        fn record_401(&self, consumer: ToolConsumer, bearer_was_sent: bool) {
            self.calls.lock().unwrap().push((consumer, bearer_was_sent));
        }
    }

    #[tokio::test]
    async fn responses_401_attribution_reports_presence_without_key_material() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/responses"))
            .respond_with(
                ResponseTemplate::new(401).set_body_string("sentinel-provider-body-account-token"),
            )
            .mount(&server)
            .await;

        let callback = Arc::new(CountingCallback::default());
        let callback_dyn: SharedAttributionCallback = callback.clone();
        let mut backend = backend(&server, None);
        backend.set_attribution_callback(Some(callback_dyn));
        let error = backend
            .search("query", None)
            .await
            .err()
            .expect("401 should fail")
            .to_string();
        assert!(!error.contains("sentinel-provider-body-account-token"));
        assert!(!error.contains("sentinel-static-key"));

        assert_eq!(
            callback.calls.lock().unwrap().as_slice(),
            &[(ToolConsumer::WebSearch, true)]
        );
    }

    #[tokio::test]
    async fn responses_failures_do_not_reflect_provider_bodies() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/responses"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string("sentinel-provider-body-account-token"),
            )
            .mount(&server)
            .await;

        let error = backend(&server, None)
            .search("query", None)
            .await
            .err()
            .expect("500 should fail")
            .to_string();
        assert!(error.contains("500"));
        assert!(!error.contains("sentinel-provider-body-account-token"));
        assert!(!error.contains("sentinel-static-key"));
    }

    #[tokio::test]
    async fn responses_chunked_bodies_are_bounded_without_content_length() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n3\r\ndef\r\n0\r\n\r\n",
                )
                .await
                .unwrap();
        });

        let response = reqwest::get(format!("http://{address}/chunked"))
            .await
            .unwrap();
        assert_eq!(response.content_length(), None);
        let error = read_response_with_limit(response, 5)
            .await
            .expect_err("streamed response must obey the cap")
            .to_string();
        assert!(error.contains("size limit"));
        server.await.unwrap();
    }
}
