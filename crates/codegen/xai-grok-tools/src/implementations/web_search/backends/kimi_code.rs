use futures_util::StreamExt as _;
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};

use super::{
    BackendSearchResult, execution_error, validate_allowed_domains, validate_search_commands,
    validate_search_query,
};
use crate::attribution::{SharedAttributionCallback, ToolConsumer};
use crate::types::{KIMI_CODE_PROVIDER_ID, SharedApiKeyProvider, resolve_kimi_code_request_auth};

const KIMI_CODE_BASE_URL: &str = "https://api.kimi.com/coding/v1";
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const MAX_RESULTS: usize = 50;
const MAX_RESULT_TEXT_BYTES: usize = 32 * 1024;
const MAX_URL_BYTES: usize = 4 * 1024;
const MAX_RENDERED_BYTES: usize = 256 * 1024;

#[derive(Clone)]
pub(in crate::implementations::web_search) struct KimiCodeBackend {
    http: reqwest::Client,
    endpoint: String,
    auth_provider: SharedApiKeyProvider,
    attribution_callback: Option<SharedAttributionCallback>,
}

struct SearchResult {
    site_name: String,
    title: String,
    url: String,
    snippet: String,
}

impl KimiCodeBackend {
    pub(in crate::implementations::web_search) fn new(
        base_url: &str,
        auth_provider: SharedApiKeyProvider,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        if auth_provider.request_auth_provider_id() != Some(KIMI_CODE_PROVIDER_ID) {
            return Err(execution_error(
                "Kimi Code web search authentication is unavailable",
            ));
        }
        let endpoint = validate_base_url(base_url)?;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .map_err(|_| execution_error("Kimi Code web search client could not be built"))?;
        Ok(Self {
            http,
            endpoint,
            auth_provider,
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
        self.execute(query, allowed_domains.as_deref()).await
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
            .filter(|queries| queries.len() == 1)
            .and_then(|queries| queries.first())
            .and_then(|query| query.get("q"))
            .and_then(serde_json::Value::as_str)
            .filter(|query| !query.trim().is_empty())
            .ok_or_else(|| {
                execution_error("Kimi Code hosted search supports one search_query command")
            })?;
        if commands
            .as_object()
            .is_some_and(|commands| commands.keys().any(|key| key != "search_query"))
        {
            return Err(execution_error(
                "Kimi Code hosted search does not support navigation commands",
            ));
        }
        validate_search_query(query)?;
        let allowed_domains = validate_allowed_domains(allowed_domains)?;
        self.execute(query, allowed_domains.as_deref()).await
    }

    async fn execute(
        &self,
        query: &str,
        allowed_domains: Option<&[String]>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        let auth = resolve_kimi_code_request_auth(&self.auth_provider)
            .await
            .map_err(|_| execution_error("Kimi Code web search authentication is unavailable"))?;
        let response = auth
            .apply(self.http.post(&self.endpoint))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(
                USER_AGENT,
                format!("grok-agent/{}", env!("CARGO_PKG_VERSION")),
            )
            .json(&serde_json::json!({"text_query": query}))
            .send()
            .await
            .map_err(|_| execution_error("Kimi Code hosted search could not be sent"))?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            self.record_401();
            return Err(xai_tool_runtime::ToolError::unauthorized(
                "Kimi Code hosted search API key was rejected".to_owned(),
            )
            .with_details(serde_json::json!({
                "tool_id": "web_search",
                "status": 401,
                "auth_recovery_provider": KIMI_CODE_PROVIDER_ID,
                "auth_recovery_exhausted": true,
            })));
        }
        if matches!(status.as_u16(), 402 | 403) {
            return Err(execution_error(
                "Kimi Code hosted search is not included in this membership",
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(execution_error(
                "Kimi Code hosted search quota or concurrency limit was reached",
            ));
        }
        if !status.is_success() {
            return Err(execution_error(format!(
                "Kimi Code hosted search failed with HTTP {status}"
            )));
        }
        let bytes = read_response_body(response, MAX_RESPONSE_BYTES).await?;
        let results = parse_search_results(&bytes)?;
        project_results(results, allowed_domains)
    }

    fn record_401(&self) {
        if let Some(callback) = self.attribution_callback.as_ref() {
            callback.record_401(ToolConsumer::WebSearch, true);
        }
    }
}

async fn read_response_body(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, xai_tool_runtime::ToolError> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(execution_error(
            "Kimi Code hosted search response exceeded the size limit",
        ));
    }

    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|_| execution_error("Kimi Code hosted search response could not be read"))?;
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(execution_error(
                "Kimi Code hosted search response exceeded the size limit",
            ));
        }
        body.try_reserve_exact(chunk.len())
            .map_err(|_| execution_error("Kimi Code hosted search response could not be read"))?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn validate_base_url(base_url: &str) -> Result<String, xai_tool_runtime::ToolError> {
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
        return Err(execution_error(
            "Kimi Code credentials may only be sent to the canonical hosted-search endpoint",
        ));
    }
    Ok(format!("{normalized}/search"))
}

fn parse_search_results(bytes: &[u8]) -> Result<Vec<SearchResult>, xai_tool_runtime::ToolError> {
    let response: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| execution_error("Kimi Code hosted search returned an invalid response"))?;
    let Some(response) = response.as_object() else {
        return Err(execution_error(
            "Kimi Code hosted search returned an invalid response",
        ));
    };
    let values = match response.get("search_results") {
        None | Some(serde_json::Value::Null) => return Ok(Vec::new()),
        Some(serde_json::Value::Array(values)) => values,
        Some(_) => {
            return Err(execution_error(
                "Kimi Code hosted search returned an invalid response",
            ));
        }
    };

    Ok(values
        .iter()
        .filter_map(|value| {
            let object = value.as_object()?;
            let url = object.get("url")?.as_str()?.trim();
            if url.is_empty() {
                return None;
            }
            let optional_text = |field: &str| {
                object
                    .get(field)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            };
            Some(SearchResult {
                site_name: optional_text("site_name"),
                title: optional_text("title"),
                url: url.to_owned(),
                snippet: optional_text("snippet"),
            })
        })
        .collect())
}

fn sanitize_markdown_text(raw: &str) -> String {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut sanitized = String::with_capacity(collapsed.len());
    for ch in collapsed.chars() {
        if matches!(
            ch,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | '('
                | ')'
                | '#'
                | '+'
                | '-'
                | '.'
                | '!'
                | '|'
        ) {
            sanitized.push('\\');
        }
        sanitized.push(ch);
    }
    sanitized
}

fn project_results(
    results: Vec<SearchResult>,
    allowed_domains: Option<&[String]>,
) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
    let mut rendered = String::new();
    let mut citations = Vec::new();
    for result in results.into_iter().take(MAX_RESULTS) {
        if result.title.len() > MAX_RESULT_TEXT_BYTES
            || result.snippet.len() > MAX_RESULT_TEXT_BYTES
            || result.site_name.len() > MAX_RESULT_TEXT_BYTES
            || result.url.len() > MAX_URL_BYTES
        {
            continue;
        }
        let Ok(url) = reqwest::Url::parse(&result.url) else {
            continue;
        };
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.host_str().is_none()
            || allowed_domains.is_some_and(|domains| {
                let host = url.host_str().unwrap_or_default();
                !domains
                    .iter()
                    .any(|domain| host == domain || host.ends_with(&format!(".{domain}")))
            })
        {
            continue;
        }
        let title = if result.title.trim().is_empty() {
            result.site_name.trim()
        } else {
            result.title.trim()
        };
        let title = sanitize_markdown_text(if title.is_empty() { "Result" } else { title });
        let snippet = sanitize_markdown_text(result.snippet.trim());
        let next = format!(
            "- [{title}](<{}>){}{}\n",
            url,
            if snippet.is_empty() { "" } else { ": " },
            snippet
        );
        if rendered.len().saturating_add(next.len()) > MAX_RENDERED_BYTES {
            break;
        }
        rendered.push_str(&next);
        citations.push((title.to_owned(), url.to_string()));
    }
    if rendered.is_empty() {
        rendered.push_str("No search results found.");
    }
    Ok(BackendSearchResult {
        content: rendered,
        citation_pairs: citations,
        references: Vec::new(),
    })
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

    fn test_backend(server: &MockServer) -> KimiCodeBackend {
        let provider: SharedApiKeyProvider = Arc::new(KimiTestProvider);
        KimiCodeBackend::new(&server.uri(), provider).unwrap()
    }

    #[tokio::test]
    async fn search_sends_the_kimi_hosted_contract_without_foreign_identity_headers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .and(body_json(
                serde_json::json!({"text_query": "rust ownership"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "search_results": [{
                    "site_name": "Rust",
                    "title": "Ownership",
                    "url": "https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html",
                    "snippet": "Ownership rules",
                    "date": "2026-07-19"
                }]
            })))
            .mount(&server)
            .await;

        let result = test_backend(&server)
            .search("rust ownership", None)
            .await
            .unwrap();
        assert_eq!(result.citation_pairs.len(), 1);

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0]
                .headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer sentinel-kimi-key")
        );
        for forbidden in [
            "x-grok-client-version",
            "x-xai-token-auth",
            "chatgpt-account-id",
            "x-msh-platform",
            "x-msh-device-id",
        ] {
            assert!(!requests[0].headers.contains_key(forbidden));
        }
    }

    #[tokio::test]
    async fn search_does_not_follow_a_redirect_to_a_credential_sink() {
        let sink = MockServer::start().await;
        let source = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(
                ResponseTemplate::new(307)
                    .insert_header("location", format!("{}/credential-sink", sink.uri())),
            )
            .mount(&source)
            .await;

        assert!(
            test_backend(&source)
                .search("redirect", None)
                .await
                .is_err()
        );
        assert!(sink.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn search_reports_an_authentication_failure_without_reflecting_the_response_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(
                ResponseTemplate::new(401).set_body_string("private prompt and key sentinel"),
            )
            .mount(&server)
            .await;

        let error = test_backend(&server)
            .search("authentication", None)
            .await
            .err()
            .expect("authentication rejection must remain explicit")
            .to_string();
        assert!(error.contains("API key was rejected"));
        assert!(!error.contains("private prompt"));
        assert!(!error.contains("sentinel"));
    }

    #[tokio::test]
    async fn search_keeps_a_valid_result_when_a_sibling_is_malformed() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "search_results": [
                    {"title": "Valid", "url": "https://example.com/page", "snippet": "text"},
                    "malformed sibling",
                    {"title": "Missing URL"},
                    {"title": "Wrong URL type", "url": 42}
                ]
            })))
            .mount(&server)
            .await;

        let result = test_backend(&server)
            .search("catalog drift", None)
            .await
            .unwrap();

        assert_eq!(result.citation_pairs.len(), 1);
        assert_eq!(
            result.content,
            "- [Valid](<https://example.com/page>): text\n"
        );
    }

    #[tokio::test]
    async fn search_escapes_provider_markdown_and_collapses_control_whitespace() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "search_results": [{
                    "title": "[Injected]\n# heading",
                    "url": "https://example.com/page",
                    "snippet": "line\n- item\t[x]"
                }]
            })))
            .mount(&server)
            .await;

        let result = test_backend(&server)
            .search("untrusted result", None)
            .await
            .unwrap();

        assert_eq!(
            result.content,
            "- [\\[Injected\\] \\# heading](<https://example.com/page>): line \\- item \\[x\\]\n"
        );
    }

    #[test]
    fn projection_filters_invalid_and_non_allowlisted_urls() {
        let result = project_results(
            vec![
                SearchResult {
                    site_name: String::new(),
                    title: "Allowed".to_owned(),
                    url: "https://docs.example.com/page".to_owned(),
                    snippet: "text".to_owned(),
                },
                SearchResult {
                    site_name: String::new(),
                    title: "Blocked".to_owned(),
                    url: "https://other.example/page".to_owned(),
                    snippet: "text".to_owned(),
                },
            ],
            Some(&["example.com".to_owned()]),
        )
        .unwrap();
        assert!(result.content.contains("Allowed"));
        assert!(!result.content.contains("Blocked"));
        assert_eq!(result.citation_pairs.len(), 1);
    }
}
