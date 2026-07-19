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
        let tool_is_listed = listed
            .body
            .pointer("/result/tools")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|tools| {
                tools.iter().any(|tool| {
                    tool.get("name").and_then(serde_json::Value::as_str) == Some(expected_tool)
                })
            });
        if !tool_is_listed {
            return Err(ZaiMcpError::Protocol);
        }

        let called = self
            .send(
                initialize.session_id.as_deref(),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "tools/call",
                    "params": {
                        "name": expected_tool,
                        "arguments": arguments
                    }
                }),
                false,
            )
            .await?;
        let result = called
            .body
            .get("result")
            .cloned()
            .ok_or(ZaiMcpError::InvalidResponse)?;
        if result
            .get("isError")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
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

fn validate_endpoint(endpoint: &str) -> Result<String, ZaiMcpError> {
    let normalized = endpoint.trim_end_matches('/');
    let canonical = matches!(normalized, SEARCH_ENDPOINT | READER_ENDPOINT | ZREAD_ENDPOINT);
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
    use super::*;

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
}
