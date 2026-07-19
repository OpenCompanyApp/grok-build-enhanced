use super::{
    BackendSearchResult, execution_error, validate_allowed_domains, validate_search_commands,
    validate_search_query,
};
use crate::attribution::{SharedAttributionCallback, ToolConsumer};
use crate::implementations::zai_mcp::{
    SEARCH_ENDPOINT, ZaiMcpClient, ZaiMcpError, text_content,
};
use crate::types::{SharedApiKeyProvider, ZAI_CODING_PLAN_PROVIDER_ID};

const MAX_RENDERED_BYTES: usize = 256 * 1024;
const MAX_CITATIONS: usize = 100;

#[derive(Clone)]
pub(in crate::implementations::web_search) struct ZaiCodingPlanBackend {
    client: ZaiMcpClient,
    attribution_callback: Option<SharedAttributionCallback>,
}

impl ZaiCodingPlanBackend {
    pub(in crate::implementations::web_search) fn new(
        endpoint: &str,
        auth_provider: SharedApiKeyProvider,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        if auth_provider.request_auth_provider_id() != Some(ZAI_CODING_PLAN_PROVIDER_ID) {
            return Err(execution_error(
                "Z.AI Coding Plan web search authentication is unavailable",
            ));
        }
        if endpoint.trim_end_matches('/') != SEARCH_ENDPOINT {
            #[cfg(not(any(test, feature = "test-support")))]
            return Err(execution_error(
                "Z.AI Coding Plan credentials may only be sent to the canonical Search MCP endpoint",
            ));
        }
        let client = ZaiMcpClient::new(endpoint, auth_provider)
            .map_err(|_| execution_error("Z.AI Coding Plan Search MCP could not be configured"))?;
        Ok(Self {
            client,
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
                execution_error("Z.AI Coding Plan Search MCP supports one search_query command")
            })?;
        if commands
            .as_object()
            .is_some_and(|commands| commands.keys().any(|key| key != "search_query"))
        {
            return Err(execution_error(
                "Z.AI Coding Plan Search MCP does not support navigation commands",
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
        let result = self
            .client
            .call_tool("webSearchPrime", serde_json::json!({"search_query": query}))
            .await
            .map_err(|error| self.map_error(error))?;
        let mut content = text_content(&result)
            .map_err(|_| execution_error("Z.AI Coding Plan Search MCP returned invalid content"))?;
        if content.len() > MAX_RENDERED_BYTES {
            content.truncate(MAX_RENDERED_BYTES);
            while !content.is_char_boundary(content.len()) {
                content.pop();
            }
        }
        let citation_pairs = collect_citations(&content, allowed_domains);
        Ok(BackendSearchResult {
            content,
            citation_pairs,
            references: Vec::new(),
        })
    }

    fn map_error(&self, error: ZaiMcpError) -> xai_tool_runtime::ToolError {
        match error {
            ZaiMcpError::Authentication => {
                if let Some(callback) = self.attribution_callback.as_ref() {
                    callback.record_401(ToolConsumer::WebSearch, true);
                }
                xai_tool_runtime::ToolError::unauthorized(
                    "Z.AI Coding Plan Search MCP API key was rejected".to_owned(),
                )
                .with_details(serde_json::json!({
                    "tool_id": "web_search",
                    "status": 401,
                    "auth_recovery_provider": ZAI_CODING_PLAN_PROVIDER_ID,
                    "auth_recovery_exhausted": true,
                }))
            }
            ZaiMcpError::Quota => execution_error(
                "Z.AI Coding Plan monthly Search/Reader/Zread quota was reached",
            ),
            ZaiMcpError::Unavailable => {
                execution_error("Z.AI Coding Plan Search MCP is temporarily unavailable")
            }
            ZaiMcpError::Protocol => execution_error(
                "Z.AI Coding Plan Search MCP protocol negotiation failed",
            ),
            ZaiMcpError::InvalidResponse | ZaiMcpError::Rejected => {
                execution_error("Z.AI Coding Plan Search MCP returned an invalid response")
            }
        }
    }
}

fn collect_citations(
    text: &str,
    allowed_domains: Option<&[String]>,
) -> Vec<(String, String)> {
    let mut citations = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for token in text.split_whitespace() {
        let candidate = token.trim_matches(|character: char| {
            matches!(
                character,
                '(' | ')' | '[' | ']' | '<' | '>' | '"' | '\'' | ',' | ';'
            )
        });
        let candidate = candidate.trim_end_matches(['.', ':', '!', '?']);
        let Ok(url) = reqwest::Url::parse(candidate) else {
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
        let url = url.to_string();
        if seen.insert(url.clone()) {
            citations.push(("Z.AI Search result".to_owned(), url));
            if citations.len() >= MAX_CITATIONS {
                break;
            }
        }
    }
    citations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citation_projection_deduplicates_and_filters_domains() {
        let citations = collect_citations(
            "https://example.com/a https://example.com/a https://other.example/b",
            Some(&["example.com".to_owned()]),
        );
        assert_eq!(citations.len(), 1);
    }
}
