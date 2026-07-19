//! Narrow Z.AI Coding Plan Streamable HTTP MCP compatibility client.
//!
//! The hosted Search, Reader, and Zread services currently negotiate MCP
//! `2024-11-05`. This client deliberately performs the complete initialize →
//! initialized notification → tools/list → tools/call sequence while keeping
//! bearer and session headers dynamic, sensitive, bounded, and out of logs.

use futures_util::StreamExt as _;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT};

use crate::types::{SharedApiKeyProvider, resolve_zai_coding_plan_request_auth};

pub(crate) const SEARCH_ENDPOINT: &str = "https://api.z.ai/api/mcp/web_search_prime/mcp";
pub(crate) const READER_ENDPOINT: &str = "https://api.z.ai/api/mcp/web_reader/mcp";
pub(crate) const ZREAD_ENDPOINT: &str = "https://api.z.ai/api/mcp/zread/mcp";
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const MCP_SESSION_HEADER: &str = "mcp-session-id";
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ZaiMcpError {
    Authentication,
    Quota,
    Protocol,
    InvalidResponse,
    Unavailable,
    Rejected,
}

impl std::fmt::Display for ZaiMcpError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Authentication => "Z.AI Coding Plan MCP authentication is unavailable",
            Self::Quota => "Z.AI Coding Plan MCP quota is exhausted",
            Self::Protocol => "Z.AI Coding Plan MCP protocol negotiation failed",
            Self::InvalidResponse => "Z.AI Coding Plan MCP returned an invalid response",
            Self::Unavailable => "Z.AI Coding Plan MCP is temporarily unavailable",
            Self::Rejected => "Z.AI Coding Plan MCP rejected the request",
        })
    }
}

impl std::error::Error for ZaiMcpError {}

#[derive(Clone)]
pub(crate) struct ZaiMcpClient {
    http: reqwest::Client,
    endpoint: String,
    auth_provider: SharedApiKeyProvider,
}

impl std::fmt::Debug for ZaiMcpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ZaiMcpClient")
            .finish_non_exhaustive()
    }
}

impl ZaiMcpClient {
    pub(crate) fn new(
        endpoint: &str,
        auth_provider: SharedApiKeyProvider,
    ) -> Result<Self, ZaiMcpError> {
        if auth_provider.request_auth_provider_id()
            != Some(crate::types::ZAI_CODING_PLAN_PROVIDER_ID)
        {
            return Err(ZaiMcpError::Authentication);
        }
        let endpoint = validate_endpoint(endpoint)?;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|_| ZaiMcpError::Unavailable)?;
        Ok(Self {
            http,
            endpoint,
            auth_provider,
        })
    }

    pub(crate) async fn call_tool(
        &self,
        expected_tool: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, ZaiMcpError> {
        self.call_tool_candidates(&[expected_tool], arguments).await
    }

    /// Call one tool from a small, caller-owned compatibility set.
    ///
    /// Z.AI's public Search documentation names `webSearchPrime`, while the
    /// live Coding Plan MCP catalog currently advertises `web_search_prime`.
    /// Callers must opt into each exact spelling; an arbitrary advertised tool
    /// is never selected or invoked.
    pub(crate) async fn call_tool_candidates(
        &self,
        expected_tools: &[&str],
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, ZaiMcpError> {
        if expected_tools.is_empty() || expected_tools.len() > 4 {
            return Err(ZaiMcpError::Protocol);
        }
        let initialize = self
            .send(
                None,
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": MCP_PROTOCOL_VERSION,
                        "capabilities": {},
                        "clientInfo": {
                            "name": "grok-build-enhanced",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    }
                }),
                false,
            )
            .await?;
        require_jsonrpc_response(&initialize.body, 1)?;
        let protocol = initialize
            .body
            .pointer("/result/protocolVersion")
            .and_then(serde_json::Value::as_str)
            .ok_or(ZaiMcpError::Protocol)?;
        if !matches!(protocol, "2024-11-05" | "2025-03-26" | "2025-06-18") {
            return Err(ZaiMcpError::Protocol);
        }

        self.send(
            initialize.session_id.as_deref(),
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            }),
            true,
        )
        .await?;

        let listed = self
            .send(
                initialize.session_id.as_deref(),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/list",
                    "params": {}
                }),
                false,
            )
            .await?;
        require_jsonrpc_response(&listed.body, 2)?;
        let advertised_tools = listed
            .body
            .pointer("/result/tools")
            .and_then(serde_json::Value::as_array)
            .ok_or(ZaiMcpError::Protocol)?;
        let selected_tool = select_advertised_tool(advertised_tools, expected_tools)?;

        let called = self
            .send(
                initialize.session_id.as_deref(),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "tools/call",
                    "params": {
                        "name": selected_tool,
                        "arguments": arguments
                    }
                }),
                false,
            )
            .await?;
        require_jsonrpc_response(&called.body, 3)?;
        let result = called
            .body
            .get("result")
            .cloned()
            .ok_or(ZaiMcpError::InvalidResponse)?;
        if result.get("isError").and_then(serde_json::Value::as_bool) == Some(true) {
            return Err(ZaiMcpError::Rejected);
        }
        Ok(result)
    }

    async fn send(
        &self,
        session_id: Option<&str>,
        body: serde_json::Value,
        allow_empty: bool,
    ) -> Result<McpResponse, ZaiMcpError> {
        let auth = resolve_zai_coding_plan_request_auth(&self.auth_provider)
            .await
            .map_err(|_| ZaiMcpError::Authentication)?;
        let mut request = auth
            .apply(self.http.post(&self.endpoint))
            .header(ACCEPT, "application/json, text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .header("mcp-protocol-version", MCP_PROTOCOL_VERSION)
            .header(
                USER_AGENT,
                format!("grok-agent/{}", env!("CARGO_PKG_VERSION")),
            );
        if let Some(session_id) = session_id {
            let mut value = HeaderValue::from_str(session_id).map_err(|_| ZaiMcpError::Protocol)?;
            value.set_sensitive(true);
            request = request.header(HeaderName::from_static(MCP_SESSION_HEADER), value);
        }
        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|_| ZaiMcpError::Unavailable)?;
        let status = response.status();
        let response_session = sensitive_session_id(response.headers())?;
        let bytes = read_limited_body(response).await?;
        if matches!(status.as_u16(), 401 | 403) {
            return Err(ZaiMcpError::Authentication);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            return Err(if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                ZaiMcpError::Quota
            } else {
                ZaiMcpError::Unavailable
            });
        }
        if !status.is_success() {
            return Err(ZaiMcpError::Rejected);
        }
        if bytes.is_empty() && allow_empty {
            return Ok(McpResponse {
                body: serde_json::Value::Null,
                session_id: response_session.or_else(|| session_id.map(str::to_owned)),
            });
        }
        let body = parse_response_body(&bytes)?;
        if let Some(code) = business_code(&body) {
            return Err(classify_business_code(code));
        }
        if body.get("error").is_some() {
            return Err(ZaiMcpError::Rejected);
        }
        Ok(McpResponse {
            body,
            session_id: response_session.or_else(|| session_id.map(str::to_owned)),
        })
    }
}

struct McpResponse {
    body: serde_json::Value,
    session_id: Option<String>,
}

fn select_advertised_tool<'a>(
    advertised_tools: &'a [serde_json::Value],
    expected_tools: &[&str],
) -> Result<&'a str, ZaiMcpError> {
    for candidate in expected_tools {
        if let Some(name) = advertised_tools.iter().find_map(|tool| {
            let name = tool.get("name")?.as_str()?;
            (name == *candidate).then_some(name)
        }) {
            return Ok(name);
        }
    }
    Err(ZaiMcpError::Protocol)
}

fn validate_endpoint(endpoint: &str) -> Result<String, ZaiMcpError> {
    let normalized = endpoint.trim_end_matches('/');
    let canonical = matches!(
        normalized,
        SEARCH_ENDPOINT | READER_ENDPOINT | ZREAD_ENDPOINT
    );
    #[cfg(any(test, feature = "test-support"))]
    let allowed = canonical
        || reqwest::Url::parse(normalized).is_ok_and(|url| {
            url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
        });
    #[cfg(not(any(test, feature = "test-support")))]
    let allowed = canonical;
    allowed
        .then(|| normalized.to_owned())
        .ok_or(ZaiMcpError::Rejected)
}

fn sensitive_session_id(headers: &HeaderMap) -> Result<Option<String>, ZaiMcpError> {
    headers
        .get(MCP_SESSION_HEADER)
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .map_err(|_| ZaiMcpError::Protocol)
        })
        .transpose()
}

async fn read_limited_body(response: reqwest::Response) -> Result<Vec<u8>, ZaiMcpError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(ZaiMcpError::InvalidResponse);
    }
    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ZaiMcpError::Unavailable)?;
        if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(body.len()) {
            return Err(ZaiMcpError::InvalidResponse);
        }
        body.try_reserve_exact(chunk.len())
            .map_err(|_| ZaiMcpError::InvalidResponse)?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn parse_response_body(bytes: &[u8]) -> Result<serde_json::Value, ZaiMcpError> {
    if let Ok(value) = serde_json::from_slice(bytes) {
        return Ok(value);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| ZaiMcpError::InvalidResponse)?;
    let mut data = String::new();
    for line in text.lines() {
        if let Some(fragment) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(fragment.trim_start());
        } else if line.is_empty() && !data.is_empty() {
            if let Ok(value) = serde_json::from_str(&data) {
                return Ok(value);
            }
            data.clear();
        }
    }
    if !data.is_empty() {
        return serde_json::from_str(&data).map_err(|_| ZaiMcpError::InvalidResponse);
    }
    Err(ZaiMcpError::InvalidResponse)
}

fn require_jsonrpc_response(
    value: &serde_json::Value,
    expected_id: i64,
) -> Result<(), ZaiMcpError> {
    if value.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0")
        || value.get("id").and_then(serde_json::Value::as_i64) != Some(expected_id)
        || value.get("result").is_none()
    {
        return Err(ZaiMcpError::Protocol);
    }
    Ok(())
}

fn business_code(value: &serde_json::Value) -> Option<i64> {
    let success = value.get("success").and_then(serde_json::Value::as_bool);
    let code = value.get("code").and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
            .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
    });
    match (success, code) {
        (Some(false), Some(code)) => Some(code),
        (Some(false), None) => Some(-1),
        (_, Some(code)) if !matches!(code, 0 | 200) => Some(code),
        _ => None,
    }
}

fn classify_business_code(code: i64) -> ZaiMcpError {
    match code {
        1001..=1004 => ZaiMcpError::Authentication,
        1305 | 1308 | 1310 | 1312 => ZaiMcpError::Quota,
        500..=599 => ZaiMcpError::Unavailable,
        _ => ZaiMcpError::Rejected,
    }
}

pub(crate) fn text_content(result: &serde_json::Value) -> Result<String, ZaiMcpError> {
    let content = result
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or(ZaiMcpError::InvalidResponse)?;
    let mut output = String::new();
    for block in content {
        if block.get("type").and_then(serde_json::Value::as_str) == Some("text")
            && let Some(text) = block.get("text").and_then(serde_json::Value::as_str)
        {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(text);
        }
    }
    if output.is_empty() {
        return Err(ZaiMcpError::InvalidResponse);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::{future::Future, pin::Pin, sync::Arc};

    use super::*;
    use crate::types::{
        ApiKeyProvider, RequestAuth, RequestCredentialSnapshot, ZAI_CODING_PLAN_PROVIDER_ID,
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, method, path},
    };

    struct ZaiTestProvider;

    impl ApiKeyProvider for ZaiTestProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(ZAI_CODING_PLAN_PROVIDER_ID)
        }

        fn current_request_auth_async(
            &self,
        ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
            Box::pin(std::future::ready(Some(
                RequestAuth::for_provider_snapshot(
                    ZAI_CODING_PLAN_PROVIDER_ID,
                    RequestCredentialSnapshot::new("opaque-zai-record", 1),
                    [(
                        "authorization".to_owned(),
                        "Bearer sentinel-zai-key".to_owned(),
                    )],
                ),
            )))
        }
    }

    fn test_client(server: &MockServer) -> ZaiMcpClient {
        let provider: SharedApiKeyProvider = Arc::new(ZaiTestProvider);
        ZaiMcpClient::new(&server.uri(), provider).unwrap()
    }

    #[test]
    fn parses_json_and_sse_responses() {
        let json = br#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        assert!(parse_response_body(json).is_ok());
        let sse = b"event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n\n";
        assert!(parse_response_body(sse).is_ok());
    }

    #[test]
    fn rejects_cross_product_endpoint() {
        assert!(validate_endpoint("https://api.z.ai/api/paas/v4/web_search").is_err());
    }

    #[tokio::test]
    async fn performs_complete_session_bound_handshake_with_dynamic_auth() {
        let server = MockServer::start().await;
        let endpoint_path = "/";
        let session_id = "opaque-test-session";

        Mock::given(method("POST"))
            .and(path(endpoint_path))
            .and(body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": {
                        "name": "grok-build-enhanced",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            })))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header(MCP_SESSION_HEADER, session_id)
                    .set_body_json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": {
                            "protocolVersion": MCP_PROTOCOL_VERSION,
                            "capabilities": {},
                            "serverInfo": {"name": "mock-zai", "version": "1"}
                        }
                    })),
            )
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path(endpoint_path))
            .and(body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path(endpoint_path))
            .and(body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "result": {
                    "tools": [{"name": "search_doc", "inputSchema": {"type": "object"}}]
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path(endpoint_path))
            .and(body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "search_doc",
                    "arguments": {"repo_name": "openai/codex", "query": "streaming"}
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "result": {"content": [{"type": "text", "text": "result"}]}
            })))
            .mount(&server)
            .await;

        let result = test_client(&server)
            .call_tool(
                "search_doc",
                serde_json::json!({"repo_name": "openai/codex", "query": "streaming"}),
            )
            .await
            .unwrap();
        assert_eq!(text_content(&result).unwrap(), "result");

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 4);
        for request in &requests {
            assert_eq!(
                request
                    .headers
                    .get("authorization")
                    .and_then(|value| value.to_str().ok()),
                Some("Bearer sentinel-zai-key")
            );
            assert_eq!(
                request
                    .headers
                    .get("mcp-protocol-version")
                    .and_then(|value| value.to_str().ok()),
                Some(MCP_PROTOCOL_VERSION)
            );
        }
        for request in requests.iter().skip(1) {
            assert_eq!(
                request
                    .headers
                    .get(MCP_SESSION_HEADER)
                    .and_then(|value| value.to_str().ok()),
                Some(session_id)
            );
        }
    }

    #[tokio::test]
    async fn rejects_http_200_business_authentication_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 1001,
                "success": false,
                "msg": "private provider detail"
            })))
            .mount(&server)
            .await;

        let error = test_client(&server)
            .call_tool("search_doc", serde_json::json!({}))
            .await
            .unwrap_err();
        assert_eq!(error, ZaiMcpError::Authentication);
        assert!(!format!("{error:?}").contains("private provider detail"));
    }

    #[test]
    fn selects_only_explicitly_allowed_advertised_tool_spelling() {
        let tools = serde_json::json!([
            {"name": "web_search_prime"},
            {"name": "unrelated_tool"}
        ]);
        let tools = tools.as_array().unwrap();
        assert_eq!(
            select_advertised_tool(tools, &["webSearchPrime", "web_search_prime"]),
            Ok("web_search_prime")
        );
        assert_eq!(
            select_advertised_tool(tools, &["webSearchPrime"]),
            Err(ZaiMcpError::Protocol)
        );
        assert_eq!(
            select_advertised_tool(tools, &[]),
            Err(ZaiMcpError::Protocol)
        );
    }

    #[test]
    fn rejects_mismatched_jsonrpc_response_identity() {
        assert_eq!(
            require_jsonrpc_response(
                &serde_json::json!({"jsonrpc": "2.0", "id": 7, "result": {}}),
                1,
            ),
            Err(ZaiMcpError::Protocol)
        );
    }
}
