#[cfg(test)]
use super::types::CodexWebSearchMessage;
use super::types::{
    CodexWebSearchContext, CodexWebSearchMessageRole, CodexWebSearchTurnMetadata, WebSearchConfig,
};
use crate::attribution::{SharedAttributionCallback, ToolConsumer};
use crate::types::SharedApiKeyProvider;
use crate::types::api_key_provider::{
    AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY, AUTH_RECOVERY_PROVIDER_DETAILS_KEY, RequestAuth,
};
use crate::types::output::WebSearchReference;
use async_openai::types::responses as rs;
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
};

const OPENAI_CODEX_PROVIDER: &str = "openai_codex";
const OPENAI_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const OPENAI_CODEX_ORIGINATOR: &str = "grok_build_codex";
const CODEX_TURN_METADATA_HEADER: &str = "x-codex-turn-metadata";
const MAX_CODEX_SEARCH_RESPONSE_BYTES: usize = 10 * 1024 * 1024;
const MAX_CODEX_SEARCH_INPUT_MESSAGES: usize = 8;
const MAX_CODEX_SEARCH_INPUT_BYTES: usize = 32 * 1024;
const MAX_CODEX_SEARCH_USER_MESSAGE_BYTES: usize = 16 * 1024;
const MAX_CODEX_SEARCH_ASSISTANT_BYTES: usize = 4 * 1024;
const MAX_CODEX_TURN_METADATA_BYTES: usize = 4 * 1024;
const MAX_CODEX_RESULT_REFERENCES: usize = 32;
const MAX_CODEX_RESULT_CITATIONS: usize = 64;
const MAX_CODEX_RESULT_NODES: usize = 4_096;
const MAX_CODEX_RESULT_DEPTH: usize = 16;
const MAX_CODEX_REF_ID_BYTES: usize = 512;
const MAX_CODEX_RESULT_TITLE_CHARS: usize = 512;
const MAX_CODEX_RESULT_URL_BYTES: usize = 4_096;

fn validate_codex_base_url(base_url: &str) -> Result<String, xai_tool_runtime::ToolError> {
    let production = base_url == OPENAI_CODEX_BASE_URL
        || base_url.strip_suffix('/') == Some(OPENAI_CODEX_BASE_URL);

    // Downstream crate tests need a narrow loopback seam for mock servers.
    // Production builds never accept a caller-selected Codex credential
    // destination.
    #[cfg(any(test, feature = "test-support"))]
    let loopback = reqwest::Url::parse(base_url).ok().is_some_and(|url| {
        url.scheme() == "http"
            && matches!(url.path(), "" | "/")
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"))
    });
    #[cfg(not(any(test, feature = "test-support")))]
    let loopback = false;

    if production {
        return Ok(OPENAI_CODEX_BASE_URL.to_owned());
    }
    if loopback {
        return Ok(base_url.trim_end_matches('/').to_owned());
    }
    Err(web_search_execution_error(
        "Codex web search credentials may only be sent to the ChatGPT Codex endpoint",
    ))
}

async fn read_codex_search_response_with_limit(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, xai_tool_runtime::ToolError> {
    use futures_util::StreamExt as _;

    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(web_search_execution_error(
            "Codex web search response exceeded the size limit",
        ));
    }

    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| {
            web_search_execution_error("Codex web search response could not be read")
        })?;
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(web_search_execution_error(
                "Codex web search response exceeded the size limit",
            ));
        }
        body.try_reserve_exact(chunk.len()).map_err(|_| {
            web_search_execution_error("Codex web search response could not be read")
        })?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[derive(Clone)]
enum WebSearchBackend {
    Responses,
    CodexSubscription { session_id: String },
}

/// A minimal, purpose-built HTTP client for calling the Responses API
/// with web search capability.
#[derive(Clone)]
pub struct WebSearchClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
    backend: WebSearchBackend,
    api_key_provider: Option<SharedApiKeyProvider>,
    /// Optional 401-attribution hook. Callers can wire this so a 401
    /// from the Responses API emits an `auth_401_attribution` event
    /// with `consumer == "WebSearch"`.
    attribution_callback: Option<SharedAttributionCallback>,
}

/// Provider-neutral result returned by the structured command entrypoint.
/// Standalone Codex search additionally supplies stable result references for
/// follow-up navigation; the xAI Responses path leaves `references` empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSearchCommandResult {
    pub content: String,
    pub citations: Vec<String>,
    pub references: Vec<WebSearchReference>,
}

impl WebSearchClient {
    /// Create a new web search client from an enabled configuration.
    ///
    /// Returns `Err` if the config is `Disabled` or if header values are invalid.
    pub fn new(
        config: &WebSearchConfig,
        api_key_provider: Option<SharedApiKeyProvider>,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let (base_url, model, backend) = match config {
            WebSearchConfig::Disabled => {
                return Err(xai_tool_runtime::ToolError::execution(
                    xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                    "Cannot create WebSearchClient from disabled config".to_string(),
                ));
            }
            WebSearchConfig::Enabled {
                api_key,
                base_url,
                model,
                extra_headers,
                alpha_test_key,
            } => {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|e| {
                        xai_tool_runtime::ToolError::execution(
                            xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                            format!("Invalid API key for header: {e}"),
                        )
                    })?,
                );
                for (key, value) in extra_headers {
                    let header_name = HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                        xai_tool_runtime::ToolError::execution(
                            xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                            format!("Invalid header name '{key}': {e}"),
                        )
                    })?;
                    let header_value = HeaderValue::from_str(value).map_err(|e| {
                        xai_tool_runtime::ToolError::execution(
                            xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                            format!("Invalid header value for '{key}': {e}"),
                        )
                    })?;
                    headers.insert(header_name, header_value);
                }
                let _ = alpha_test_key;
                (base_url.clone(), model.clone(), WebSearchBackend::Responses)
            }
            WebSearchConfig::CodexSubscription {
                base_url,
                model,
                session_id,
            } => {
                // Match the current first-party Codex client contract: the
                // reqwest client supplies product identity, the model provider
                // supplies `version`, and provider auth supplies only dynamic
                // credentials. Identity headers are deliberately not accepted
                // through RequestAuth below.
                headers.insert(
                    USER_AGENT,
                    HeaderValue::from_str(&format!(
                        "grok-build-codex/{}",
                        xai_grok_version::VERSION
                    ))
                    .map_err(|_| {
                        web_search_execution_error("Invalid Codex web search client version")
                    })?,
                );
                headers.insert(
                    HeaderName::from_static("originator"),
                    HeaderValue::from_static(OPENAI_CODEX_ORIGINATOR),
                );
                headers.insert(
                    HeaderName::from_static("version"),
                    HeaderValue::from_static(xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION),
                );
                (
                    validate_codex_base_url(base_url)?,
                    model.clone(),
                    WebSearchBackend::CodexSubscription {
                        session_id: session_id.clone(),
                    },
                )
            }
        };
        let is_codex_subscription = matches!(&backend, WebSearchBackend::CodexSubscription { .. });
        let mut http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60));
        if is_codex_subscription {
            // Never replay bearer/account/FedRAMP headers to a redirect target.
            http = http.redirect(reqwest::redirect::Policy::none());
        }
        let http = http.build().map_err(|e| {
            xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("Failed to build HTTP client: {e}"),
            )
        })?;
        Ok(Self {
            http,
            base_url,
            model,
            backend,
            api_key_provider,
            attribution_callback: None,
        })
    }
    /// Wire a 401-attribution callback into this client. Idempotent;
    /// safe to call before or after the first request.
    pub fn with_attribution_callback(
        mut self,
        callback: Option<SharedAttributionCallback>,
    ) -> Self {
        self.attribution_callback = callback;
        self
    }
    async fn current_bearer(&self) -> Option<String> {
        crate::types::api_key_provider::resolve_bearer(self.api_key_provider.as_ref()).await
    }
    fn record_401_attribution(&self, sent_bearer: Option<&str>) {
        self.record_401_attribution_presence(sent_bearer.is_some());
    }
    fn record_401_attribution_presence(&self, bearer_was_sent: bool) {
        if let Some(callback) = self.attribution_callback.as_ref() {
            callback.record_401(ToolConsumer::WebSearch, bearer_was_sent);
        }
    }

    async fn search_codex_subscription(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<CodexSearchResult, xai_tool_runtime::ToolError> {
        if query.trim().is_empty() {
            return Err(web_search_execution_error(
                "A non-empty web search query is required",
            ));
        }

        self.search_codex_commands(
            query,
            serde_json::json!({
                "search_query": [{ "q": query }],
            }),
            allowed_domains,
            None,
        )
        .await
    }

    async fn search_codex_commands(
        &self,
        input_text: &str,
        commands: serde_json::Value,
        allowed_domains: Option<Vec<String>>,
        context: Option<&CodexWebSearchContext>,
    ) -> Result<CodexSearchResult, xai_tool_runtime::ToolError> {
        let WebSearchBackend::CodexSubscription { session_id } = &self.backend else {
            return Err(web_search_execution_error(
                "Codex web search was requested for a non-Codex client",
            ));
        };

        let mut settings = serde_json::Map::from_iter([
            ("allowed_callers".to_string(), serde_json::json!(["direct"])),
            (
                "external_web_access".to_string(),
                serde_json::Value::Bool(true),
            ),
        ]);
        if let Some(domains) = allowed_domains.filter(|domains| !domains.is_empty()) {
            settings.insert(
                "filters".to_string(),
                serde_json::json!({ "allowed_domains": domains }),
            );
        }
        let mut body = serde_json::Map::from_iter([
            ("id".to_string(), serde_json::json!(session_id)),
            ("model".to_string(), serde_json::json!(self.model)),
            ("commands".to_string(), commands),
            ("settings".to_string(), serde_json::Value::Object(settings)),
            ("max_output_tokens".to_string(), serde_json::json!(8192)),
        ]);
        let turn_metadata = if let Some(context) = context {
            let (input, metadata) = validate_codex_search_context(context)?;
            body.insert("input".to_string(), input);
            Some(metadata)
        } else if !input_text.trim().is_empty() {
            body.insert(
                "input".to_string(),
                serde_json::Value::String(input_text.to_string()),
            );
            None
        } else {
            None
        };
        self.execute_codex_search(serde_json::Value::Object(body), turn_metadata)
            .await
    }

    /// Execute the current Codex standalone-search command schema. This is
    /// intentionally separate from the xAI Responses-backed implementation:
    /// subscription calls use provider-scoped auth and `/alpha/search`, while
    /// xAI retains the existing hosted Responses tool behavior.
    pub async fn run_commands(
        &self,
        input_text: &str,
        commands: serde_json::Value,
        allowed_domains: Option<Vec<String>>,
        context: Option<&CodexWebSearchContext>,
    ) -> Result<WebSearchCommandResult, xai_tool_runtime::ToolError> {
        match &self.backend {
            WebSearchBackend::CodexSubscription { .. } => self
                .search_codex_commands(input_text, commands, allowed_domains, context)
                .await
                .map(|result| WebSearchCommandResult {
                    content: result.content,
                    citations: result.citations,
                    references: result.references,
                }),
            WebSearchBackend::Responses => {
                let query = commands
                    .get("search_query")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|queries| queries.first())
                    .and_then(|query| query.get("q"))
                    .and_then(serde_json::Value::as_str)
                    .filter(|query| !query.trim().is_empty())
                    .ok_or_else(|| {
                        web_search_execution_error(
                            "This provider supports query web search but not Codex navigation commands",
                        )
                    })?;
                if commands.as_object().is_some_and(|commands| {
                    commands.keys().any(|key| key != "search_query")
                        || commands
                            .get("search_query")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|queries| queries.len() != 1)
                }) {
                    return Err(web_search_execution_error(
                        "This provider supports query web search but not Codex navigation commands",
                    ));
                }
                self.search(query, allowed_domains)
                    .await
                    .map(|(content, citations)| WebSearchCommandResult {
                        content,
                        citations,
                        references: Vec::new(),
                    })
            }
        }
    }

    async fn execute_codex_search(
        &self,
        body: serde_json::Value,
        turn_metadata: Option<HeaderValue>,
    ) -> Result<CodexSearchResult, xai_tool_runtime::ToolError> {
        let provider = self.api_key_provider.as_ref().ok_or_else(|| {
            web_search_execution_error("Codex web search authentication is unavailable")
        })?;
        let url = format!("{}/alpha/search", self.base_url.trim_end_matches('/'));
        let mut recovered = false;

        loop {
            let auth = crate::types::api_key_provider::resolve_request_auth(Some(provider))
                .await
                .filter(|auth| auth.provider() == Some(OPENAI_CODEX_PROVIDER))
                .ok_or_else(|| {
                    web_search_execution_error(
                        "Codex web search provider authentication is unavailable",
                    )
                })?;
            let rejected = auth.credential_snapshot().cloned();
            let mut request = self.http.post(&url).json(&body);
            if let Some(value) = turn_metadata.as_ref() {
                request = request.header(
                    HeaderName::from_static(CODEX_TURN_METADATA_HEADER),
                    value.clone(),
                );
            }
            let request = apply_codex_request_auth(request, &auth)?;
            let response = request.send().await.map_err(|_| {
                web_search_execution_error("Codex web search request could not be sent")
            })?;
            let status = response.status();

            if status == reqwest::StatusCode::UNAUTHORIZED {
                // Provider auth was validated before dispatch, so a bearer was
                // definitely present. Report only that boolean; never pass the
                // dynamic credential into attribution or logs.
                self.record_401_attribution_presence(true);
                if !recovered && provider.recover_unauthorized_async(rejected).await {
                    recovered = true;
                    continue;
                }
                return Err(xai_tool_runtime::ToolError::unauthorized(
                    "ChatGPT Codex rejected web search authentication".to_string(),
                )
                .with_details(serde_json::json!({
                    "tool_id": "web_search",
                    "status": 401,
                    (AUTH_RECOVERY_PROVIDER_DETAILS_KEY): OPENAI_CODEX_PROVIDER,
                    (AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY): true,
                })));
            }
            if !status.is_success() {
                return Err(web_search_execution_error(format!(
                    "Codex web search request failed with HTTP {status}"
                )));
            }

            let bytes =
                read_codex_search_response_with_limit(response, MAX_CODEX_SEARCH_RESPONSE_BYTES)
                    .await?;
            let response: CodexSearchWireResponse =
                serde_json::from_slice(&bytes).map_err(|_| {
                    web_search_execution_error("Codex web search returned an invalid response")
                })?;
            let projection = codex_result_projection(response.results.as_deref());
            let citations = projection
                .citation_pairs
                .iter()
                .map(|(_, url)| url.clone())
                .collect();
            return Ok(CodexSearchResult {
                content: response.output,
                citations,
                citation_pairs: projection.citation_pairs,
                references: projection.references,
            });
        }
    }
    /// Perform a web search query using the Responses API.
    ///
    /// Returns `(content, citations)` where content is the assistant's text
    /// and citations are unique URLs found in the response annotations.
    pub async fn search(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<(String, Vec<String>), xai_tool_runtime::ToolError> {
        if matches!(&self.backend, WebSearchBackend::CodexSubscription { .. }) {
            return self
                .search_codex_subscription(query, allowed_domains)
                .await
                .map(|result| (result.content, result.citations));
        }
        let web_search = rs::WebSearchToolArgs::default()
            .filters(rs::WebSearchToolFilters { allowed_domains })
            .build()
            .map_err(|e| {
                xai_tool_runtime::ToolError::execution(
                    xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                    format!("Failed to build web search tool: {e}"),
                )
            })?;
        let request = rs::CreateResponseArgs::default()
            .model(self.model.clone())
            .input(query.to_string())
            .tools(vec![rs::Tool::WebSearch(web_search)])
            .store(false)
            .temperature(0.1)
            .top_p(0.95)
            .max_output_tokens(8192u32)
            .build()
            .map_err(|e| {
                xai_tool_runtime::ToolError::execution(
                    xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                    format!("Failed to build request: {e}"),
                )
            })?;
        let url = format!("{}/responses", self.base_url.trim_end_matches('/'));
        let sent_bearer = self.current_bearer().await;
        let mut req = self.http.post(&url).json(&request);
        if let Some(ref key) = sent_bearer {
            req = req.header(AUTHORIZATION, format!("Bearer {key}"));
        }
        let response = req.send().await.map_err(|e| {
            xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("HTTP request failed: {e}"),
            )
        })?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            self.record_401_attribution(sent_bearer.as_deref());
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(xai_tool_runtime::ToolError::unauthorized(format!(
                "Responses API returned 401 Unauthorized: {body}"
            ))
            .with_details(serde_json::json!({ "tool_id" : "web_search", "status" : 401, })));
        }
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("Responses API returned {status}: {body}"),
            ));
        }
        let bytes = response.bytes().await.map_err(|e| {
            xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("Failed to read response body: {e}"),
            )
        })?;
        let response_obj: rs::Response = serde_json::from_slice(&bytes).map_err(|e| {
            xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("Failed to parse response: {e}"),
            )
        })?;
        let content = response_obj
            .output_text()
            .unwrap_or_else(|| "No search results found.".to_string());
        let citations = extract_citations(&response_obj);
        Ok((content, citations))
    }
    /// Same as [`Self::search`] but also extracts per-citation titles when
    /// the Responses API surfaces them. Returns `(content, citations_with_titles)`
    /// where each citation is `(title, url)`. Empty `title` strings indicate
    /// the upstream didn't supply one for that URL.
    ///
    /// Used by the cursor-compat `WebSearch` adapter to render a
    /// `Links:\n1. [title](url)` list instead of the LLM synthesis text.
    pub async fn search_with_titles(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<(String, Vec<(String, String)>), xai_tool_runtime::ToolError> {
        if matches!(&self.backend, WebSearchBackend::CodexSubscription { .. }) {
            return self
                .search_codex_subscription(query, allowed_domains)
                .await
                .map(|result| (result.content, result.citation_pairs));
        }
        let web_search = rs::WebSearchToolArgs::default()
            .filters(rs::WebSearchToolFilters { allowed_domains })
            .build()
            .map_err(|e| {
                xai_tool_runtime::ToolError::execution(
                    xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                    format!("Failed to build web search tool: {e}"),
                )
            })?;
        let request = rs::CreateResponseArgs::default()
            .model(self.model.clone())
            .input(query.to_string())
            .tools(vec![rs::Tool::WebSearch(web_search)])
            .store(false)
            .temperature(0.1)
            .top_p(0.95)
            .max_output_tokens(8192u32)
            .build()
            .map_err(|e| {
                xai_tool_runtime::ToolError::execution(
                    xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                    format!("Failed to build request: {e}"),
                )
            })?;
        let url = format!("{}/responses", self.base_url.trim_end_matches('/'));
        let sent_bearer = self.current_bearer().await;
        let mut req = self.http.post(&url).json(&request);
        if let Some(ref key) = sent_bearer {
            req = req.header(AUTHORIZATION, format!("Bearer {key}"));
        }
        let response = req.send().await.map_err(|e| {
            xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("HTTP request failed: {e}"),
            )
        })?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            self.record_401_attribution(sent_bearer.as_deref());
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(xai_tool_runtime::ToolError::unauthorized(format!(
                "Responses API returned 401 Unauthorized: {body}"
            ))
            .with_details(serde_json::json!({ "tool_id" : "web_search", "status" : 401, })));
        }
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("Responses API returned {status}: {body}"),
            ));
        }
        let bytes = response.bytes().await.map_err(|e| {
            xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("Failed to read response body: {e}"),
            )
        })?;
        let response_obj: rs::Response = serde_json::from_slice(&bytes).map_err(|e| {
            xai_tool_runtime::ToolError::execution(
                xai_tool_protocol::ToolId::new("web_search").expect("valid"),
                format!("Failed to parse response: {e}"),
            )
        })?;
        let content = response_obj
            .output_text()
            .unwrap_or_else(|| "No search results found.".to_string());
        let pairs = extract_citation_pairs(&response_obj);
        Ok((content, pairs))
    }
}

#[derive(Debug, serde::Deserialize)]
struct CodexSearchWireResponse {
    #[serde(default)]
    output: String,
    #[serde(default)]
    results: Option<Vec<serde_json::Value>>,
}

struct CodexSearchResult {
    content: String,
    citations: Vec<String>,
    citation_pairs: Vec<(String, String)>,
    references: Vec<WebSearchReference>,
}

fn web_search_execution_error(message: impl Into<String>) -> xai_tool_runtime::ToolError {
    xai_tool_runtime::ToolError::execution(
        xai_tool_protocol::ToolId::new("web_search").expect("valid"),
        message.into(),
    )
}

fn validate_codex_search_context(
    context: &CodexWebSearchContext,
) -> Result<(serde_json::Value, HeaderValue), xai_tool_runtime::ToolError> {
    if context.input.is_empty() || context.input.len() > MAX_CODEX_SEARCH_INPUT_MESSAGES {
        return Err(web_search_execution_error(
            "Codex web search context has an invalid message count",
        ));
    }
    if context.input.first().map(|message| message.role) != Some(CodexWebSearchMessageRole::User)
        || context.input.last().map(|message| message.role) != Some(CodexWebSearchMessageRole::User)
    {
        return Err(web_search_execution_error(
            "Codex web search context must start and end with visible user text",
        ));
    }

    let mut total_bytes = 0usize;
    let mut assistant_bytes = 0usize;
    let mut user_messages = 0usize;
    let mut wire_messages = Vec::with_capacity(context.input.len());
    for message in &context.input {
        if message.text.trim().is_empty() {
            return Err(web_search_execution_error(
                "Codex web search context contains an empty message",
            ));
        }
        total_bytes = total_bytes.saturating_add(message.text.len());
        let (role, content_type) = match message.role {
            CodexWebSearchMessageRole::User => {
                user_messages += 1;
                if message.text.len() > MAX_CODEX_SEARCH_USER_MESSAGE_BYTES {
                    return Err(web_search_execution_error(
                        "Codex web search user context exceeded the size limit",
                    ));
                }
                ("user", "input_text")
            }
            CodexWebSearchMessageRole::Assistant => {
                assistant_bytes = assistant_bytes.saturating_add(message.text.len());
                ("assistant", "output_text")
            }
        };
        wire_messages.push(serde_json::json!({
            "type": "message",
            "role": role,
            "content": [{
                "type": content_type,
                "text": message.text,
            }],
        }));
    }
    if !(1..=2).contains(&user_messages)
        || total_bytes > MAX_CODEX_SEARCH_INPUT_BYTES
        || assistant_bytes > MAX_CODEX_SEARCH_ASSISTANT_BYTES
    {
        return Err(web_search_execution_error(
            "Codex web search context exceeded its bounded visible-history window",
        ));
    }

    let metadata = codex_turn_metadata_header(&context.turn_metadata)?;
    Ok((serde_json::Value::Array(wire_messages), metadata))
}

fn codex_turn_metadata_header(
    metadata: &CodexWebSearchTurnMetadata,
) -> Result<HeaderValue, xai_tool_runtime::ToolError> {
    for (name, value, limit) in [
        ("session_id", metadata.session_id.as_str(), 256usize),
        ("thread_id", metadata.thread_id.as_str(), 256usize),
        ("turn_id", metadata.turn_id.as_str(), 256usize),
        ("model", metadata.model.as_str(), 256usize),
    ] {
        if value.trim().is_empty() || value.len() > limit || value.chars().any(char::is_control) {
            return Err(web_search_execution_error(format!(
                "Codex web search {name} metadata is invalid"
            )));
        }
    }
    if metadata.reasoning_effort.as_ref().is_some_and(|effort| {
        effort.is_empty()
            || effort.len() > 32
            || !effort
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    }) {
        return Err(web_search_execution_error(
            "Codex web search reasoning metadata is invalid",
        ));
    }

    let encoded = ascii_json_string(metadata)?;
    if encoded.len() > MAX_CODEX_TURN_METADATA_BYTES {
        return Err(web_search_execution_error(
            "Codex web search turn metadata exceeded the size limit",
        ));
    }
    debug_assert!(encoded.is_ascii());
    let mut header = HeaderValue::from_str(&encoded).map_err(|_| {
        web_search_execution_error("Codex web search turn metadata could not be encoded")
    })?;
    // Correlation identifiers are not credentials, but keeping the header
    // sensitive prevents HTTP debug formatters from exposing session IDs.
    header.set_sensitive(true);
    Ok(header)
}

fn ascii_json_string<T: serde::Serialize>(
    value: &T,
) -> Result<String, xai_tool_runtime::ToolError> {
    use std::fmt::Write as _;

    let json = serde_json::to_string(value).map_err(|_| {
        web_search_execution_error("Codex web search turn metadata could not be encoded")
    })?;
    let mut ascii = String::with_capacity(json.len());
    for character in json.chars() {
        if character.is_ascii() {
            ascii.push(character);
        } else {
            let mut code_units = [0u16; 2];
            for code_unit in character.encode_utf16(&mut code_units) {
                write!(&mut ascii, "\\u{:04x}", *code_unit).expect("writing to String cannot fail");
            }
        }
    }
    Ok(ascii)
}

fn apply_codex_request_auth(
    request: reqwest::RequestBuilder,
    auth: &RequestAuth,
) -> Result<reqwest::RequestBuilder, xai_tool_runtime::ToolError> {
    if auth.provider() != Some(OPENAI_CODEX_PROVIDER) {
        return Err(web_search_execution_error(
            "Codex web search received authentication for another provider",
        ));
    }
    if auth
        .credential_snapshot()
        .is_none_or(|snapshot| snapshot.opaque_id().trim().is_empty() || snapshot.generation() == 0)
    {
        return Err(web_search_execution_error(
            "Codex web search authentication omitted its credential generation",
        ));
    }
    let mut headers = HeaderMap::new();
    let mut has_authorization = false;
    let mut has_account = false;
    let mut has_fedramp = false;
    for (name, value) in auth.headers() {
        let normalized = name.to_ascii_lowercase();
        match normalized.as_str() {
            "authorization" if has_authorization => {
                return Err(web_search_execution_error(
                    "Codex web search authentication emitted duplicate credential headers",
                ));
            }
            "authorization"
                if value
                    .strip_prefix("Bearer ")
                    .is_none_or(|token| token.trim().is_empty()) =>
            {
                return Err(web_search_execution_error(
                    "Codex web search authorization is unavailable",
                ));
            }
            "chatgpt-account-id" if has_account => {
                return Err(web_search_execution_error(
                    "Codex web search authentication emitted duplicate credential headers",
                ));
            }
            "chatgpt-account-id" if value.trim().is_empty() => {
                return Err(web_search_execution_error(
                    "Codex web search account selection is unavailable",
                ));
            }
            "x-openai-fedramp" if has_fedramp => {
                return Err(web_search_execution_error(
                    "Codex web search authentication emitted duplicate credential headers",
                ));
            }
            "x-openai-fedramp" if value != "true" => {
                return Err(web_search_execution_error(
                    "Codex web search FedRAMP state must be exactly true when present",
                ));
            }
            "authorization" | "chatgpt-account-id" | "x-openai-fedramp" => {}
            _ => {
                return Err(web_search_execution_error(
                    "Codex web search received an unsupported provider header",
                ));
            }
        }
        let name = HeaderName::from_bytes(normalized.as_bytes()).map_err(|_| {
            web_search_execution_error("Codex web search received an invalid provider header")
        })?;
        let mut value = HeaderValue::from_str(value).map_err(|_| {
            web_search_execution_error("Codex web search received an invalid provider header")
        })?;
        if matches!(
            normalized.as_str(),
            "authorization" | "chatgpt-account-id" | "x-openai-fedramp"
        ) {
            value.set_sensitive(true);
        }
        headers.insert(name, value);
        has_authorization |= normalized == "authorization";
        has_account |= normalized == "chatgpt-account-id";
        has_fedramp |= normalized == "x-openai-fedramp";
    }
    if !has_authorization || !has_account {
        return Err(web_search_execution_error(
            "Codex web search provider authentication is incomplete",
        ));
    }
    Ok(request.headers(headers))
}

struct CodexResultProjection {
    citation_pairs: Vec<(String, String)>,
    references: Vec<WebSearchReference>,
}

#[derive(Default)]
struct CodexResultProjectionBuilder {
    citation_pairs: Vec<(String, String)>,
    references: Vec<WebSearchReference>,
    seen_urls: std::collections::HashSet<String>,
    seen_refs: std::collections::HashSet<String>,
    visited_nodes: usize,
}

fn codex_result_projection(results: Option<&[serde_json::Value]>) -> CodexResultProjection {
    let mut builder = CodexResultProjectionBuilder::default();
    for result in results.unwrap_or_default() {
        collect_codex_result_projection(result, 0, &mut builder);
        if builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
            break;
        }
    }
    CodexResultProjection {
        citation_pairs: builder.citation_pairs,
        references: builder.references,
    }
}

fn collect_codex_result_projection(
    value: &serde_json::Value,
    depth: usize,
    builder: &mut CodexResultProjectionBuilder,
) {
    if depth > MAX_CODEX_RESULT_DEPTH || builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
        return;
    }
    builder.visited_nodes += 1;
    match value {
        serde_json::Value::Object(object) => {
            if let Some(url) = object
                .get("url")
                .and_then(serde_json::Value::as_str)
                .and_then(sanitize_codex_result_url)
            {
                let title = object
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .map(sanitize_codex_result_title)
                    .unwrap_or_default();
                if builder.citation_pairs.len() < MAX_CODEX_RESULT_CITATIONS
                    && builder.seen_urls.insert(url.clone())
                {
                    builder.citation_pairs.push((title.clone(), url.clone()));
                }
                if builder.references.len() < MAX_CODEX_RESULT_REFERENCES
                    && let Some(ref_id) = object
                        .get("ref_id")
                        .and_then(serde_json::Value::as_str)
                        .and_then(sanitize_codex_ref_id)
                    && builder.seen_refs.insert(ref_id.clone())
                {
                    builder
                        .references
                        .push(WebSearchReference { ref_id, title, url });
                }
            }
            for child in object.values() {
                collect_codex_result_projection(child, depth + 1, builder);
                if builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
                    break;
                }
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                collect_codex_result_projection(child, depth + 1, builder);
                if builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn sanitize_codex_ref_id(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.len() <= MAX_CODEX_REF_ID_BYTES
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')))
    .then(|| value.to_string())
}

fn sanitize_codex_result_title(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_CODEX_RESULT_TITLE_CHARS)
        .collect()
}

fn sanitize_codex_result_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_CODEX_RESULT_URL_BYTES {
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

/// Extract citation URLs from the Response output items.
/// The async-openai crate doesn't provide a helper for this, and the `url` field
/// in `UrlCitationBody` is private, so we serialize to JSON to extract it.
fn extract_citations(response: &rs::Response) -> Vec<String> {
    let mut citations = Vec::new();
    for output_item in &response.output {
        if let rs::OutputItem::Message(output_message) = output_item {
            for message_content in &output_message.content {
                if let rs::OutputMessageContent::OutputText(text_content) = message_content {
                    for annotation in &text_content.annotations {
                        if let rs::Annotation::UrlCitation(url_citation) = annotation
                            && let Ok(json) = serde_json::to_value(url_citation)
                            && let Some(url) = json.get("url").and_then(|v| v.as_str())
                        {
                            citations.push(url.to_string());
                        }
                    }
                }
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    citations.retain(|url| seen.insert(url.clone()));
    citations
}
/// Extract `(title, url)` pairs from the Responses API annotations.
///
/// `title` may be an empty string when upstream doesn't supply one. URLs
/// are deduplicated while preserving the first-seen order so the rendered
/// `Links:` list is stable and free of duplicates.
fn extract_citation_pairs(response: &rs::Response) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for output_item in &response.output {
        if let rs::OutputItem::Message(output_message) = output_item {
            for message_content in &output_message.content {
                if let rs::OutputMessageContent::OutputText(text_content) = message_content {
                    for annotation in &text_content.annotations {
                        if let rs::Annotation::UrlCitation(url_citation) = annotation
                            && let Ok(json) = serde_json::to_value(url_citation)
                        {
                            let url = json.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            if url.is_empty() {
                                continue;
                            }
                            let title = json
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            pairs.push((title, url.to_string()));
                        }
                    }
                }
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    pairs.retain(|(_t, url)| seen.insert(url.clone()));
    pairs
}
#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    /// Helper to create a Response from JSON for testing.
    fn response_from_json(json: serde_json::Value) -> rs::Response {
        serde_json::from_value(json).expect("Failed to parse test Response JSON")
    }
    #[test]
    fn test_new_client_uses_configured_model() {
        let config = WebSearchConfig::Enabled {
            api_key: "test-key".to_string(),
            base_url: "https://api.x.ai/v1".to_string(),
            model: "custom-enterprise-model".to_string(),
            extra_headers: IndexMap::new(),
            alpha_test_key: None,
        };
        let client = WebSearchClient::new(&config, None).expect("client should build");
        assert_eq!(client.model, "custom-enterprise-model");
    }

    #[tokio::test]
    async fn codex_chunked_response_without_content_length_is_stream_bounded() {
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
        let error = read_codex_search_response_with_limit(response, 5)
            .await
            .expect_err("six streamed bytes must exceed the five-byte cap")
            .to_string();
        assert!(error.contains("size limit"), "unexpected error: {error}");
        server.await.unwrap();
    }
    /// Counts attribution callback invocations for the test below.
    #[derive(Default, Debug)]
    struct CountingCallback {
        invocations: std::sync::Mutex<Vec<(ToolConsumer, bool)>>,
    }
    impl crate::attribution::Auth401AttributionCallback for CountingCallback {
        fn record_401(&self, consumer: ToolConsumer, bearer_was_sent: bool) {
            self.invocations
                .lock()
                .unwrap()
                .push((consumer, bearer_was_sent));
        }
    }
    /// `record_401_attribution` reports only bearer presence. No credential
    /// bytes cross the trait boundary.
    #[test]
    fn record_401_attribution_reports_presence_without_credential_material() {
        let cb = std::sync::Arc::new(CountingCallback::default());
        let cb_dyn: crate::attribution::SharedAttributionCallback = cb.clone();
        let config = WebSearchConfig::Enabled {
            api_key: "ignored".to_string(),
            base_url: "https://api.x.ai/v1".to_string(),
            model: "test-model".to_string(),
            extra_headers: IndexMap::new(),
            alpha_test_key: None,
        };
        let client = WebSearchClient::new(&config, None)
            .expect("client should build")
            .with_attribution_callback(Some(cb_dyn));
        client.record_401_attribution(Some("bearer-with-long-tail-aaaaaaaaaa"));
        let calls = cb.invocations.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, ToolConsumer::WebSearch);
        assert!(calls[0].1);
    }
    /// `record_401_attribution` is a no-op when no callback is wired
    /// -- the BYOK / standalone case must not panic or allocate.
    #[test]
    fn record_401_attribution_is_noop_without_callback() {
        let config = WebSearchConfig::Enabled {
            api_key: "test-key".to_string(),
            base_url: "https://api.x.ai/v1".to_string(),
            model: "test-model".to_string(),
            extra_headers: IndexMap::new(),
            alpha_test_key: None,
        };
        let client = WebSearchClient::new(&config, None).expect("client should build");
        client.record_401_attribution(Some("any-bearer"));
        client.record_401_attribution(None);
    }
    #[test]
    fn test_extract_citations_empty_response() {
        let response = response_from_json(serde_json::json!(
            { "id" : "resp_test", "object" : "response", "created_at" : 1234567890,
            "status" : "completed", "output" : [], "model" : "test-model" }
        ));
        let citations = extract_citations(&response);
        assert!(citations.is_empty());
    }
    #[test]
    fn test_extract_citations_with_url_citations() {
        let response = response_from_json(serde_json::json!(
            { "id" : "resp_test", "object" : "response", "created_at" : 1234567890,
            "status" : "completed", "model" : "test-model", "output" : [{ "type" :
            "message", "id" : "msg_1", "status" : "completed", "role" : "assistant",
            "content" : [{ "type" : "output_text", "text" :
            "Here is some info about Rust.", "annotations" : [{ "type" :
            "url_citation", "url" : "https://www.rust-lang.org/", "title" :
            "Rust Programming Language", "start_index" : 0, "end_index" : 10 }, {
            "type" : "url_citation", "url" : "https://docs.rs/", "title" : "Docs.rs",
            "start_index" : 11, "end_index" : 20 }] }] }] }
        ));
        let citations = extract_citations(&response);
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0], "https://www.rust-lang.org/");
        assert_eq!(citations[1], "https://docs.rs/");
    }
    #[test]
    fn test_extract_citations_deduplicates() {
        let response = response_from_json(serde_json::json!(
            { "id" : "resp_test", "object" : "response", "created_at" : 1234567890,
            "status" : "completed", "model" : "test-model", "output" : [{ "type" :
            "message", "id" : "msg_1", "status" : "completed", "role" : "assistant",
            "content" : [{ "type" : "output_text", "text" :
            "Info with duplicate citations.", "annotations" : [{ "type" :
            "url_citation", "url" : "https://example.com/page1", "title" : "Page 1",
            "start_index" : 0, "end_index" : 5 }, { "type" : "url_citation", "url" :
            "https://example.com/page2", "title" : "Page 2", "start_index" : 6,
            "end_index" : 10 }, { "type" : "url_citation", "url" :
            "https://example.com/page1", "title" : "Page 1 Again", "start_index" :
            11, "end_index" : 15 }] }] }] }
        ));
        let citations = extract_citations(&response);
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0], "https://example.com/page1");
        assert_eq!(citations[1], "https://example.com/page2");
    }
    #[test]
    fn test_extract_citations_multiple_messages() {
        let response = response_from_json(serde_json::json!(
            { "id" : "resp_test", "object" : "response", "created_at" : 1234567890,
            "status" : "completed", "model" : "test-model", "output" : [{ "type" :
            "message", "id" : "msg_1", "status" : "completed", "role" : "assistant",
            "content" : [{ "type" : "output_text", "text" : "First message",
            "annotations" : [{ "type" : "url_citation", "url" : "https://first.com/",
            "title" : "First", "start_index" : 0, "end_index" : 5 }] }] }, { "type" :
            "message", "id" : "msg_2", "status" : "completed", "role" : "assistant",
            "content" : [{ "type" : "output_text", "text" : "Second message",
            "annotations" : [{ "type" : "url_citation", "url" :
            "https://second.com/", "title" : "Second", "start_index" : 0, "end_index"
            : 6 }] }] }] }
        ));
        let citations = extract_citations(&response);
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0], "https://first.com/");
        assert_eq!(citations[1], "https://second.com/");
    }
    #[test]
    fn test_extract_citations_ignores_non_url_annotations() {
        let response = response_from_json(serde_json::json!(
            { "id" : "resp_test", "object" : "response", "created_at" : 1234567890,
            "status" : "completed", "model" : "test-model", "output" : [{ "type" :
            "message", "id" : "msg_1", "status" : "completed", "role" : "assistant",
            "content" : [{ "type" : "output_text", "text" : "Some text",
            "annotations" : [{ "type" : "url_citation", "url" : "https://valid.com/",
            "title" : "Valid", "start_index" : 0, "end_index" : 4 }] }] }] }
        ));
        let citations = extract_citations(&response);
        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0], "https://valid.com/");
    }
    /// A provider that always returns `None`, simulating an API-key user
    /// whose token has aged past the client-side TTL.
    struct NoneProvider;
    impl crate::types::ApiKeyProvider for NoneProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }
    }
    /// When the dynamic provider returns `None`, the static `api_key`
    /// from config must still be sent as the Authorization header.
    /// This is a regression scenario: API-key users
    /// past the 30-day client TTL saw 401 because no auth was sent.
    #[tokio::test]
    async fn static_api_key_is_fallback_when_provider_returns_none() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/responses"))
            .and(header("Authorization", "Bearer static-key-from-config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(
                { "id" : "resp_test", "object" : "response", "created_at" :
                1234567890, "status" : "completed", "model" : "test-model",
                "output" : [{ "type" : "message", "id" : "msg_1", "status" :
                "completed", "role" : "assistant", "content" : [{ "type" :
                "output_text", "text" : "search result", "annotations" : []
                }] }] }
            )))
            .mount(&server)
            .await;
        let config = WebSearchConfig::Enabled {
            api_key: "static-key-from-config".to_string(),
            base_url: server.uri(),
            model: "test-model".to_string(),
            extra_headers: IndexMap::new(),
            alpha_test_key: None,
        };
        let provider: SharedApiKeyProvider = std::sync::Arc::new(NoneProvider);
        let client = WebSearchClient::new(&config, Some(provider)).expect("client should build");
        let (content, _citations) = client
            .search("test query", None)
            .await
            .expect("search must succeed with static key fallback");
        assert_eq!(content, "search result");
    }
    /// When the provider returns a fresh key, it overrides the static one.
    #[tokio::test]
    async fn provider_key_overrides_static_key() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        struct FreshProvider;
        impl crate::types::ApiKeyProvider for FreshProvider {
            fn current_api_key(&self) -> Option<String> {
                Some("fresh-key-from-provider".to_string())
            }
        }
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/responses"))
            .and(header("Authorization", "Bearer fresh-key-from-provider"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(
                { "id" : "resp_test", "object" : "response", "created_at" :
                1234567890, "status" : "completed", "model" : "test-model",
                "output" : [{ "type" : "message", "id" : "msg_1", "status" :
                "completed", "role" : "assistant", "content" : [{ "type" :
                "output_text", "text" : "fresh result", "annotations" : [] }]
                }] }
            )))
            .mount(&server)
            .await;
        let config = WebSearchConfig::Enabled {
            api_key: "stale-static-key".to_string(),
            base_url: server.uri(),
            model: "test-model".to_string(),
            extra_headers: IndexMap::new(),
            alpha_test_key: None,
        };
        let provider: SharedApiKeyProvider = std::sync::Arc::new(FreshProvider);
        let client = WebSearchClient::new(&config, Some(provider)).expect("client should build");
        let (content, _citations) = client
            .search("test query", None)
            .await
            .expect("search must succeed with provider key");
        assert_eq!(content, "fresh result");
    }
    #[test]
    fn test_extract_citations_no_annotations() {
        let response = response_from_json(serde_json::json!(
            { "id" : "resp_test", "object" : "response", "created_at" : 1234567890,
            "status" : "completed", "model" : "test-model", "output" : [{ "type" :
            "message", "id" : "msg_1", "status" : "completed", "role" : "assistant",
            "content" : [{ "type" : "output_text", "text" :
            "Plain text with no annotations", "annotations" : [] }] }] }
        ));
        let citations = extract_citations(&response);
        assert!(citations.is_empty());
    }

    #[derive(Debug)]
    struct CodexRequestAuthProvider {
        generation: std::sync::atomic::AtomicUsize,
        recover: bool,
    }

    impl CodexRequestAuthProvider {
        fn new(recover: bool) -> Self {
            Self {
                generation: std::sync::atomic::AtomicUsize::new(1),
                recover,
            }
        }
    }

    impl crate::types::ApiKeyProvider for CodexRequestAuthProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn current_request_auth_async(
            &self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Option<crate::types::RequestAuth>> + Send + '_>,
        > {
            Box::pin(async move {
                let generation = self.generation.load(std::sync::atomic::Ordering::SeqCst);
                let token = if generation == 1 {
                    "codex-old-token"
                } else {
                    "codex-fresh-token"
                };
                Some(crate::types::RequestAuth::for_provider_snapshot(
                    OPENAI_CODEX_PROVIDER,
                    crate::types::RequestCredentialSnapshot::new(
                        "opaque-codex-credential",
                        generation as u64,
                    ),
                    [
                        ("Authorization".to_string(), format!("Bearer {token}")),
                        (
                            "ChatGPT-Account-ID".to_string(),
                            "account-sensitive".to_string(),
                        ),
                        ("X-OpenAI-Fedramp".to_string(), "true".to_string()),
                    ],
                ))
            })
        }

        fn recover_unauthorized_async(
            &self,
            rejected: Option<crate::types::RequestCredentialSnapshot>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            Box::pin(async move {
                if !self.recover || rejected.is_none_or(|snapshot| snapshot.generation() != 1) {
                    return false;
                }
                self.generation
                    .store(2, std::sync::atomic::Ordering::SeqCst);
                true
            })
        }
    }

    fn codex_search_config(base_url: String) -> WebSearchConfig {
        WebSearchConfig::CodexSubscription {
            base_url,
            model: "gpt-5.6-luna".to_string(),
            session_id: "session-stable".to_string(),
        }
    }

    #[test]
    fn codex_search_seals_its_production_base_url() {
        for rejected in [
            "https://foreign.example/backend-api/codex",
            "http://chatgpt.com/backend-api/codex",
            "https://chatgpt.com:444/backend-api/codex",
            "https://user@chatgpt.com/backend-api/codex",
            "https://chatgpt.com/backend-api/codex/alpha",
            "https://chatgpt.com/backend-api/codex?redirect=foreign",
            "https://chatgpt.com/backend-api/codex#fragment",
        ] {
            let error = WebSearchClient::new(&codex_search_config(rejected.to_owned()), None)
                .err()
                .expect("foreign or malformed Codex endpoints must be rejected")
                .to_string();
            assert!(error.contains("ChatGPT Codex endpoint"));
            assert!(
                !error.contains(rejected),
                "the rejected destination must not cross the error boundary"
            );
        }

        let client = WebSearchClient::new(
            &codex_search_config(format!("{OPENAI_CODEX_BASE_URL}/")),
            None,
        )
        .expect("one trailing slash should canonicalize to the official endpoint");
        assert_eq!(client.base_url, OPENAI_CODEX_BASE_URL);
    }

    #[tokio::test]
    async fn codex_search_never_replays_credentials_to_a_redirect_target() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let redirect_target = MockServer::start().await;
        let source = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .respond_with(ResponseTemplate::new(307).insert_header(
                "location",
                format!("{}/credential-sink", redirect_target.uri()),
            ))
            .expect(1)
            .mount(&source)
            .await;

        let provider: SharedApiKeyProvider =
            std::sync::Arc::new(CodexRequestAuthProvider::new(false));
        let client = WebSearchClient::new(&codex_search_config(source.uri()), Some(provider))
            .expect("loopback Codex client should build in tests");
        let error = client
            .search("redirect safety", None)
            .await
            .expect_err("a Codex redirect must be terminal")
            .to_string();
        assert!(error.contains("307"), "unexpected error: {error}");

        let source_requests = source.received_requests().await.expect("source request");
        let source_request = source_requests.first().expect("one source request");
        for name in ["authorization", "chatgpt-account-id", "x-openai-fedramp"] {
            assert!(
                source_request.headers.contains_key(name),
                "source request omitted {name}"
            );
        }
        assert!(
            redirect_target
                .received_requests()
                .await
                .expect("redirect target request log")
                .is_empty(),
            "the redirect target must receive no request or credential headers"
        );
    }

    #[tokio::test]
    async fn responses_backend_rejects_unadvertised_codex_navigation_without_http() {
        let client = WebSearchClient::new(
            &WebSearchConfig::Enabled {
                api_key: "xai-test-key".to_string(),
                base_url: "https://must-not-be-contacted.invalid/v1".to_string(),
                model: "xai-search-model".to_string(),
                extra_headers: Default::default(),
                alpha_test_key: None,
            },
            None,
        )
        .expect("xAI client should build");

        let error = client
            .run_commands(
                "open: turn0search0",
                serde_json::json!({"open": [{"ref_id": "turn0search0"}]}),
                None,
                None,
            )
            .await
            .expect_err("xAI backend must reject Codex-only commands")
            .to_string();
        assert!(error.contains("not Codex navigation commands"));
    }

    fn codex_request_auth(headers: impl IntoIterator<Item = (String, String)>) -> RequestAuth {
        RequestAuth::for_provider_snapshot(
            OPENAI_CODEX_PROVIDER,
            crate::types::RequestCredentialSnapshot::new("stable-credential", 1),
            headers,
        )
    }

    #[test]
    fn codex_search_auth_rejects_duplicate_or_malformed_credentials() {
        let cases = [
            (
                vec![
                    ("Authorization".to_string(), "Bearer first".to_string()),
                    ("authorization".to_string(), "Bearer second".to_string()),
                    ("ChatGPT-Account-ID".to_string(), "account".to_string()),
                ],
                "duplicate credential headers",
            ),
            (
                vec![
                    ("Authorization".to_string(), "Bearer   ".to_string()),
                    ("ChatGPT-Account-ID".to_string(), "account".to_string()),
                ],
                "authorization is unavailable",
            ),
            (
                vec![
                    ("Authorization".to_string(), "Bearer token".to_string()),
                    ("ChatGPT-Account-ID".to_string(), "  ".to_string()),
                ],
                "account selection is unavailable",
            ),
            (
                vec![
                    ("Authorization".to_string(), "Bearer token".to_string()),
                    ("ChatGPT-Account-ID".to_string(), "account".to_string()),
                    ("X-OpenAI-Fedramp".to_string(), "True".to_string()),
                ],
                "must be exactly true",
            ),
            (
                vec![
                    ("Authorization".to_string(), "Bearer token".to_string()),
                    ("ChatGPT-Account-ID".to_string(), "account".to_string()),
                    ("X-XAI-Token-Auth".to_string(), "wrong-provider".to_string()),
                ],
                "unsupported provider header",
            ),
        ];

        for (headers, expected) in cases {
            let auth = codex_request_auth(headers);
            let error =
                apply_codex_request_auth(reqwest::Client::new().get("https://example.com"), &auth)
                    .err()
                    .expect("malformed Codex auth must fail")
                    .to_string();
            assert!(error.contains(expected), "unexpected error: {error}");
            assert!(!error.contains("wrong-provider"));
        }
    }

    #[test]
    fn codex_search_auth_requires_binding_and_marks_only_secrets_sensitive() {
        let unbound = RequestAuth::for_provider(
            OPENAI_CODEX_PROVIDER,
            [
                ("Authorization".to_string(), "Bearer token".to_string()),
                ("ChatGPT-Account-ID".to_string(), "account".to_string()),
            ],
        );
        let error =
            apply_codex_request_auth(reqwest::Client::new().get("https://example.com"), &unbound)
                .err()
                .expect("unbound Codex auth must fail")
                .to_string();
        assert!(error.contains("credential generation"));

        let auth = codex_request_auth([
            ("Authorization".to_string(), "Bearer token".to_string()),
            ("ChatGPT-Account-ID".to_string(), "account".to_string()),
            ("X-OpenAI-Fedramp".to_string(), "true".to_string()),
        ]);
        let request =
            apply_codex_request_auth(reqwest::Client::new().get("https://example.com"), &auth)
                .expect("well-formed Codex auth should apply")
                .build()
                .expect("request should build");
        assert!(request.headers()["authorization"].is_sensitive());
        assert!(request.headers()["chatgpt-account-id"].is_sensitive());
        assert!(request.headers()["x-openai-fedramp"].is_sensitive());
    }

    #[test]
    fn codex_turn_metadata_header_is_ascii_bounded_and_sensitive() {
        let header = codex_turn_metadata_header(&CodexWebSearchTurnMetadata {
            session_id: "session-stable".to_string(),
            thread_id: "thread-stable".to_string(),
            turn_id: "türn-7".to_string(),
            model: "gpt-5.6-luna".to_string(),
            reasoning_effort: Some("xhigh".to_string()),
        })
        .expect("valid turn metadata");
        let wire = header.to_str().expect("ASCII header");
        assert!(wire.is_ascii());
        assert!(wire.contains(r#""turn_id":"t\u00fcrn-7""#));
        assert!(header.is_sensitive());

        let error = codex_turn_metadata_header(&CodexWebSearchTurnMetadata {
            session_id: "s".repeat(257),
            thread_id: "thread".to_string(),
            turn_id: "turn".to_string(),
            model: "gpt-5.6-luna".to_string(),
            reasoning_effort: None,
        })
        .expect_err("oversized metadata must fail")
        .to_string();
        assert!(error.contains("session_id metadata is invalid"));
    }

    #[test]
    fn codex_search_context_accepts_only_bounded_visible_dialogue() {
        let metadata = || CodexWebSearchTurnMetadata {
            session_id: "session-stable".to_string(),
            thread_id: "thread-stable".to_string(),
            turn_id: "turn-stable".to_string(),
            model: "gpt-5.6-luna".to_string(),
            reasoning_effort: Some("xhigh".to_string()),
        };
        let valid = CodexWebSearchContext {
            input: vec![
                CodexWebSearchMessage::user("previous user"),
                CodexWebSearchMessage::assistant("previous assistant"),
                CodexWebSearchMessage::user("current user"),
            ],
            turn_metadata: metadata(),
        };
        let (wire, header) =
            validate_codex_search_context(&valid).expect("visible dialogue should be accepted");
        assert_eq!(
            wire,
            serde_json::json!([
                {
                    "type": "message",
                    "role": "user",
                    "content": [{"type": "input_text", "text": "previous user"}],
                },
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": "previous assistant"}],
                },
                {
                    "type": "message",
                    "role": "user",
                    "content": [{"type": "input_text", "text": "current user"}],
                },
            ])
        );
        assert!(header.is_sensitive());

        for invalid in [
            CodexWebSearchContext {
                input: vec![
                    CodexWebSearchMessage::assistant("untrusted leading assistant"),
                    CodexWebSearchMessage::user("current user"),
                ],
                turn_metadata: metadata(),
            },
            CodexWebSearchContext {
                input: vec![
                    CodexWebSearchMessage::user("first user"),
                    CodexWebSearchMessage::user("second user"),
                    CodexWebSearchMessage::user("third user"),
                ],
                turn_metadata: metadata(),
            },
            CodexWebSearchContext {
                input: vec![
                    CodexWebSearchMessage::user("previous user"),
                    CodexWebSearchMessage::assistant(
                        "a".repeat(MAX_CODEX_SEARCH_ASSISTANT_BYTES + 1),
                    ),
                    CodexWebSearchMessage::user("current user"),
                ],
                turn_metadata: metadata(),
            },
        ] {
            assert!(
                validate_codex_search_context(&invalid).is_err(),
                "untrusted or oversized dialogue must not cross the provider boundary"
            );
        }
    }

    #[test]
    fn codex_result_projection_is_bounded_and_rejects_unsafe_fields() {
        let mut results = (0..80)
            .map(|index| {
                serde_json::json!({
                    "ref_id": format!("turn0search{index}"),
                    "title": format!("Result {index}"),
                    "url": format!("https://example.com/{index}"),
                    "opaque": {"provider_state": "must not be projected"},
                })
            })
            .collect::<Vec<_>>();
        results.extend([
            serde_json::json!({
                "ref_id": "unsafe",
                "title": "credential URL",
                "url": "https://user:password@example.com/private",
            }),
            serde_json::json!({
                "ref_id": "control\nid",
                "title": "control ref",
                "url": "https://safe.example/",
            }),
        ]);

        let projection = codex_result_projection(Some(&results));
        assert_eq!(projection.references.len(), MAX_CODEX_RESULT_REFERENCES);
        assert_eq!(projection.citation_pairs.len(), MAX_CODEX_RESULT_CITATIONS);
        assert!(projection.references.iter().all(|reference| {
            reference.ref_id.is_ascii()
                && !reference.ref_id.bytes().any(|byte| byte.is_ascii_control())
                && !reference.url.contains('@')
        }));
        let serialized = serde_json::to_string(&projection.references).unwrap();
        assert!(!serialized.contains("provider_state"));
        assert!(!serialized.contains("password"));
    }

    #[tokio::test]
    async fn codex_search_uses_alpha_endpoint_scoped_headers_and_structured_commands() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let turn_metadata = r#"{"session_id":"session-stable","thread_id":"session-stable","turn_id":"prompt-turn-7","model":"gpt-5.6-luna","reasoning_effort":"xhigh"}"#;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "output": "Search output with turn0search0",
                "results": [
                    {
                        "type": "text_result",
                        "ref_id": "turn0search0",
                        "url": "https://outside-allowlist.example/article",
                        "title": "Outside result"
                    },
                    {
                        "nested": {
                            "url": "https://outside-allowlist.example/article",
                            "title": "duplicate"
                        }
                    },
                    {
                        "type": "open_result",
                        "ref_id": "turn0view0",
                        "url": "https://openai.com/research/",
                        "title": "  OpenAI\n research  "
                    },
                    {
                        "ref_id": "credential-url-must-drop",
                        "url": "https://user:password@example.com/private",
                        "title": "secret"
                    },
                    {
                        "ref_id": "control\nid",
                        "url": "https://invalid-ref.example/",
                        "title": "invalid ref id"
                    },
                    {
                        "secret_provider_state": "opaque-state-must-not-surface"
                    }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;
        let provider: SharedApiKeyProvider =
            std::sync::Arc::new(CodexRequestAuthProvider::new(false));
        let client = WebSearchClient::new(&codex_search_config(server.uri()), Some(provider))
            .expect("Codex client should build");
        let context = CodexWebSearchContext {
            input: vec![
                super::CodexWebSearchMessage::user("previous user"),
                super::CodexWebSearchMessage::assistant("previous assistant"),
                super::CodexWebSearchMessage::user("current user"),
            ],
            turn_metadata: CodexWebSearchTurnMetadata {
                session_id: "session-stable".to_string(),
                thread_id: "session-stable".to_string(),
                turn_id: "prompt-turn-7".to_string(),
                model: "gpt-5.6-luna".to_string(),
                reasoning_effort: Some("xhigh".to_string()),
            },
        };
        let result = client
            .run_commands(
                "all standalone search commands",
                serde_json::json!({
                    "search_query": [{
                        "q": "current Codex release",
                        "recency": 7,
                        "domains": ["openai.com"],
                    }],
                    "image_query": [{"q": "Codex terminal UI"}],
                    "open": [
                        {"ref_id": "turn0search0", "lineno": 20},
                        {"ref_id": "https://outside.example/article"},
                    ],
                    "click": [{"ref_id": "turn0view0", "id": 3}],
                    "find": [{"ref_id": "turn0search0", "pattern": "install"}],
                    "screenshot": [{"ref_id": "turn0view1", "pageno": 0}],
                    "finance": [{"ticker": "BTC", "type": "crypto", "market": ""}],
                    "weather": [{
                        "location": "Belgium, Brussels",
                        "start": "2026-07-17",
                        "duration": 3,
                    }],
                    "sports": [{
                        "tool": "sports",
                        "fn": "schedule",
                        "league": "nba",
                        "team": "BOS",
                        "date_from": "2026-10-01",
                        "num_games": 2,
                        "locale": "en-US",
                    }],
                    "time": [{"utc_offset": "+02:00"}],
                    "response_length": "long",
                }),
                Some(vec!["openai.com".to_string()]),
                Some(&context),
            )
            .await
            .expect("Codex search should succeed");
        assert_eq!(result.content, "Search output with turn0search0");
        assert_eq!(
            result.citations,
            vec![
                "https://outside-allowlist.example/article".to_string(),
                "https://openai.com/research/".to_string(),
                "https://invalid-ref.example/".to_string(),
            ]
        );
        assert_eq!(
            result.references,
            vec![
                WebSearchReference {
                    ref_id: "turn0search0".to_string(),
                    title: "Outside result".to_string(),
                    url: "https://outside-allowlist.example/article".to_string(),
                },
                WebSearchReference {
                    ref_id: "turn0view0".to_string(),
                    title: "OpenAI research".to_string(),
                    url: "https://openai.com/research/".to_string(),
                },
            ]
        );

        let requests = server.received_requests().await.expect("recorded request");
        let request = requests.first().expect("one request");
        let expected_user_agent = format!("grok-build-codex/{}", xai_grok_version::VERSION);
        for (name, expected) in [
            ("authorization", "Bearer codex-old-token"),
            ("chatgpt-account-id", "account-sensitive"),
            ("x-openai-fedramp", "true"),
            ("originator", OPENAI_CODEX_ORIGINATOR),
            ("user-agent", expected_user_agent.as_str()),
            (
                "version",
                xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION,
            ),
            (CODEX_TURN_METADATA_HEADER, turn_metadata),
        ] {
            let values = request.headers.get_all(name);
            assert_eq!(values.iter().count(), 1, "expected one {name} header");
            assert_eq!(
                values.iter().next().and_then(|value| value.to_str().ok()),
                Some(expected),
                "unexpected {name} header"
            );
        }
        for forbidden in [
            "x-xai-token-auth",
            "x-grok-conv-id",
            "x-grok-req-id",
            "x-grok-model-override",
            "x-codex-turn-state",
        ] {
            assert!(
                !request.headers.contains_key(forbidden),
                "Codex search must not send {forbidden}"
            );
        }
        let body: serde_json::Value = serde_json::from_slice(&request.body).expect("JSON request");
        assert_eq!(
            body,
            serde_json::json!({
                "id": "session-stable",
                "model": "gpt-5.6-luna",
                "input": [
                    {
                        "type": "message",
                        "role": "user",
                        "content": [{"type": "input_text", "text": "previous user"}],
                    },
                    {
                        "type": "message",
                        "role": "assistant",
                        "content": [{"type": "output_text", "text": "previous assistant"}],
                    },
                    {
                        "type": "message",
                        "role": "user",
                        "content": [{"type": "input_text", "text": "current user"}],
                    },
                ],
                "commands": {
                    "search_query": [{
                        "q": "current Codex release",
                        "recency": 7,
                        "domains": ["openai.com"],
                    }],
                    "image_query": [{"q": "Codex terminal UI"}],
                    "open": [
                        {"ref_id": "turn0search0", "lineno": 20},
                        {"ref_id": "https://outside.example/article"},
                    ],
                    "click": [{"ref_id": "turn0view0", "id": 3}],
                    "find": [{"ref_id": "turn0search0", "pattern": "install"}],
                    "screenshot": [{"ref_id": "turn0view1", "pageno": 0}],
                    "finance": [{"ticker": "BTC", "type": "crypto", "market": ""}],
                    "weather": [{
                        "location": "Belgium, Brussels",
                        "start": "2026-07-17",
                        "duration": 3,
                    }],
                    "sports": [{
                        "tool": "sports",
                        "fn": "schedule",
                        "league": "nba",
                        "team": "BOS",
                        "date_from": "2026-10-01",
                        "num_games": 2,
                        "locale": "en-US",
                    }],
                    "time": [{"utc_offset": "+02:00"}],
                    "response_length": "long",
                },
                "settings": {
                    "allowed_callers": ["direct"],
                    "external_web_access": true,
                    "filters": {"allowed_domains": ["openai.com"]},
                },
                "max_output_tokens": 8192,
            })
        );
    }

    #[tokio::test]
    async fn codex_search_refreshes_once_with_provider_scoped_rotation() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .and(header("Authorization", "Bearer codex-old-token"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_string("sentinel-provider-body-must-not-surface"),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .and(header("Authorization", "Bearer codex-fresh-token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"output": "recovered"})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let provider = std::sync::Arc::new(CodexRequestAuthProvider::new(true));
        let provider_dyn: SharedApiKeyProvider = provider.clone();
        let callback = std::sync::Arc::new(CountingCallback::default());
        let callback_dyn: crate::attribution::SharedAttributionCallback = callback.clone();
        let client = WebSearchClient::new(&codex_search_config(server.uri()), Some(provider_dyn))
            .expect("Codex client should build")
            .with_attribution_callback(Some(callback_dyn));

        let (content, _) = client
            .search("current Codex behavior", None)
            .await
            .expect("provider recovery should replay once");
        assert_eq!(content, "recovered");
        assert_eq!(
            provider
                .generation
                .load(std::sync::atomic::Ordering::SeqCst),
            2
        );
        assert_eq!(
            callback.invocations.lock().unwrap().as_slice(),
            &[(ToolConsumer::WebSearch, true)],
            "Codex 401 attribution reports bearer presence without credential material"
        );
    }

    #[tokio::test]
    async fn codex_search_errors_never_reflect_provider_response_bodies() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .respond_with(
                ResponseTemplate::new(500)
                    .set_body_string("sentinel-token-and-account-provider-body"),
            )
            .mount(&server)
            .await;
        let provider: SharedApiKeyProvider =
            std::sync::Arc::new(CodexRequestAuthProvider::new(false));
        let client = WebSearchClient::new(&codex_search_config(server.uri()), Some(provider))
            .expect("Codex client should build");

        let error = client
            .search("will fail", None)
            .await
            .expect_err("500 should fail")
            .to_string();
        assert!(!error.contains("sentinel-token-and-account-provider-body"));
        assert!(error.contains("500"));
    }
}
