use std::collections::HashSet;

use futures_util::StreamExt as _;
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};

use super::{
    BackendSearchResult, execution_error, validate_allowed_domains, validate_search_commands,
    validate_search_query,
};

const EXA_HOSTED_MCP_URL: &str = "https://mcp.exa.ai/mcp";
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const MAX_RENDERED_BYTES: usize = 256 * 1024;
const MAX_CITATIONS: usize = 64;

#[derive(Clone)]
pub(in crate::implementations::web_search) struct ExaHostedBackend {
    http: reqwest::Client,
    endpoint: String,
    configured_domain_policy: bool,
}

impl ExaHostedBackend {
    pub(in crate::implementations::web_search) fn new(
        base_url: &str,
        allowed_domains: Option<Vec<String>>,
        excluded_domains: Option<Vec<String>>,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        let endpoint = validate_endpoint(base_url)?;
        let http = xai_grok_provider_http::with_extra_root_certificates(reqwest::Client::builder())
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(25))
            .build()
            .map_err(|_| execution_error("Exa hosted search client could not be built"))?;
        Ok(Self {
            http,
            endpoint,
            configured_domain_policy: allowed_domains.is_some() || excluded_domains.is_some(),
        })
    }

    pub(in crate::implementations::web_search) async fn search(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        validate_search_query(query)?;
        self.reject_configured_domain_policy()?;
        reject_domain_filter(allowed_domains)?;
        self.execute(query).await
    }

    pub(in crate::implementations::web_search) async fn run_commands(
        &self,
        commands: &serde_json::Value,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        validate_search_commands(commands)?;
        if commands
            .as_object()
            .is_some_and(|commands| commands.keys().any(|key| key != "search_query"))
        {
            return Err(execution_error(
                "Exa hosted search supports search_query only",
            ));
        }
        let query = commands
            .get("search_query")
            .and_then(serde_json::Value::as_array)
            .filter(|queries| queries.len() == 1)
            .and_then(|queries| queries.first())
            .and_then(|query| query.get("q"))
            .and_then(serde_json::Value::as_str)
            .filter(|query| !query.trim().is_empty())
            .ok_or_else(|| {
                execution_error("Exa hosted search supports one search_query command")
            })?;
        validate_search_query(query)?;
        self.reject_configured_domain_policy()?;
        reject_domain_filter(allowed_domains)?;
        self.execute(query).await
    }

    fn reject_configured_domain_policy(&self) -> Result<(), xai_tool_runtime::ToolError> {
        if self.configured_domain_policy {
            return Err(execution_error(
                "Exa hosted search cannot enforce the configured domain policy; refusing an unfiltered search",
            ));
        }
        Ok(())
    }

    async fn execute(
        &self,
        query: &str,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "web_search_exa",
                "arguments": {
                    "query": query,
                    "type": "auto",
                    "numResults": 8,
                    "livecrawl": "fallback",
                    "contextMaxCharacters": 10_000
                }
            }
        });
        let response = self
            .http
            .post(&self.endpoint)
            .header(ACCEPT, "application/json, text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .header(
                USER_AGENT,
                format!("grok-agent/{}", env!("CARGO_PKG_VERSION")),
            )
            .json(&request)
            .send()
            .await
            .map_err(|_| execution_error("Exa hosted search could not be sent"))?;
        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(execution_error("Exa hosted search rate limit was reached"));
        }
        if !status.is_success() {
            return Err(execution_error(format!(
                "Exa hosted search failed with HTTP {status}"
            )));
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let bytes = read_response_body(response).await?;
        project_response(&bytes, content_type.contains("text/event-stream"))
    }
}

fn validate_endpoint(base_url: &str) -> Result<String, xai_tool_runtime::ToolError> {
    let normalized = base_url.trim_end_matches('/');
    #[cfg(any(test, feature = "test-support"))]
    let allowed = normalized == EXA_HOSTED_MCP_URL
        || reqwest::Url::parse(normalized).is_ok_and(|url| {
            url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost"))
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
        });
    #[cfg(not(any(test, feature = "test-support")))]
    let allowed = normalized == EXA_HOSTED_MCP_URL;
    allowed.then(|| normalized.to_owned()).ok_or_else(|| {
        execution_error("Exa hosted search requires the canonical keyless MCP endpoint")
    })
}

fn reject_domain_filter(
    allowed_domains: Option<Vec<String>>,
) -> Result<(), xai_tool_runtime::ToolError> {
    if validate_allowed_domains(allowed_domains)?.is_some() {
        return Err(execution_error(
            "Exa hosted search does not support allowed_domains; refusing an unfiltered search",
        ));
    }
    Ok(())
}

async fn read_response_body(
    response: reqwest::Response,
) -> Result<Vec<u8>, xai_tool_runtime::ToolError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(execution_error(
            "Exa hosted search response exceeded the size limit",
        ));
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|_| execution_error("Exa hosted search response could not be read"))?;
        if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(body.len()) {
            return Err(execution_error(
                "Exa hosted search response exceeded the size limit",
            ));
        }
        body.try_reserve_exact(chunk.len())
            .map_err(|_| execution_error("Exa hosted search response could not be read"))?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn project_response(
    bytes: &[u8],
    is_event_stream: bool,
) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
    let payload = if is_event_stream || bytes.starts_with(b"event:") || bytes.starts_with(b"data:")
    {
        parse_sse_payload(bytes)?
    } else {
        serde_json::from_slice(bytes)
            .map_err(|_| execution_error("Exa hosted search returned an invalid MCP response"))?
    };
    if payload.get("error").is_some() {
        return Err(execution_error("Exa hosted search returned an MCP error"));
    }
    let result = payload
        .get("result")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| execution_error("Exa hosted search returned no MCP result"))?;
    if result
        .get("isError")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Err(execution_error("Exa hosted search tool execution failed"));
    }
    let content = result
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| execution_error("Exa hosted search returned no text content"))?;
    let mut rendered = String::new();
    for text in content.iter().filter_map(|part| {
        (part.get("type").and_then(serde_json::Value::as_str) == Some("text"))
            .then(|| part.get("text").and_then(serde_json::Value::as_str))
            .flatten()
    }) {
        let separator = usize::from(!rendered.is_empty());
        if text.len() + separator > MAX_RENDERED_BYTES.saturating_sub(rendered.len()) {
            return Err(execution_error(
                "Exa hosted search result exceeded the rendered size limit",
            ));
        }
        if separator != 0 {
            rendered.push('\n');
        }
        rendered.push_str(text);
    }
    if rendered.trim().is_empty() {
        return Err(execution_error(
            "Exa hosted search returned no text content",
        ));
    }
    let citation_pairs = extract_https_urls(&rendered)
        .into_iter()
        .map(|url| (url.clone(), url))
        .collect();
    Ok(BackendSearchResult {
        content: rendered,
        citation_pairs,
        references: Vec::new(),
    })
}

fn parse_sse_payload(bytes: &[u8]) -> Result<serde_json::Value, xai_tool_runtime::ToolError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| execution_error("Exa hosted search returned an invalid event stream"))?;
    let mut latest = None;
    for event in text.split("\n\n") {
        let data = event
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n");
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let value = serde_json::from_str::<serde_json::Value>(&data)
            .map_err(|_| execution_error("Exa hosted search returned an invalid event"))?;
        if value.get("result").is_some() || value.get("error").is_some() {
            latest = Some(value);
        }
    }
    latest.ok_or_else(|| execution_error("Exa hosted search event stream contained no result"))
}

fn extract_https_urls(content: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    content
        .split_whitespace()
        .filter_map(|token| {
            let token = token.trim_matches(|character: char| {
                matches!(
                    character,
                    '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | ',' | ';' | '"' | '\''
                )
            });
            let token = token.trim_end_matches(['.', ':', '!', '?']);
            let url = reqwest::Url::parse(token).ok()?;
            (url.scheme() == "https" && url.host_str().is_some() && url.username().is_empty())
                .then(|| url.to_string())
        })
        .filter(|url| seen.insert(url.clone()))
        .take(MAX_CITATIONS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_json_and_sse_without_provider_metadata() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"content": [{"type": "text", "text": "Result https://example.com/a"}]}
        });
        let projected = project_response(&serde_json::to_vec(&response).unwrap(), false).unwrap();
        assert_eq!(
            projected.citations(),
            vec!["https://example.com/a".to_owned()]
        );

        let sse = format!("event: message\ndata: {response}\n\n");
        assert_eq!(
            project_response(sse.as_bytes(), true).unwrap().content,
            projected.content
        );
    }

    #[test]
    fn domain_filters_fail_closed() {
        assert!(reject_domain_filter(Some(vec!["example.com".to_owned()])).is_err());
        assert!(reject_domain_filter(None).is_ok());

        let configured = ExaHostedBackend::new(
            EXA_HOSTED_MCP_URL,
            None,
            Some(vec!["private.example.com".to_owned()]),
        )
        .unwrap();
        assert!(configured.reject_configured_domain_policy().is_err());
    }
}
