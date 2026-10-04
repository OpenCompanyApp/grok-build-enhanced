use super::{
    BackendSearchResult, execution_error, validate_allowed_domains, validate_search_commands,
    validate_search_query,
};
use crate::attribution::{SharedAttributionCallback, ToolConsumer};
use crate::implementations::zai_mcp::{SEARCH_ENDPOINT, ZaiMcpClient, ZaiMcpError, text_content};
use crate::types::{SharedApiKeyProvider, ZAI_CODING_PLAN_PROVIDER_ID};

const MAX_RENDERED_BYTES: usize = 256 * 1024;
const MAX_JSON_STRING_LAYERS: usize = 2;

#[derive(Clone)]
pub(in crate::implementations::web_search) struct ZaiCodingPlanBackend {
    client: ZaiMcpClient,
    allowed_domains: Option<Vec<String>>,
    excluded_domains: Option<Vec<String>>,
    attribution_callback: Option<SharedAttributionCallback>,
}

impl ZaiCodingPlanBackend {
    pub(in crate::implementations::web_search) fn new(
        endpoint: &str,
        allowed_domains: Option<Vec<String>>,
        excluded_domains: Option<Vec<String>>,
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
            allowed_domains: validate_allowed_domains(allowed_domains)?,
            excluded_domains: validate_allowed_domains(excluded_domains)?,
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
        let allowed_domains =
            validate_allowed_domains(self.allowed_domains.clone().or_else(|| {
                if self.excluded_domains.is_some() {
                    None
                } else {
                    allowed_domains
                }
            }))?;
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
        let allowed_domains =
            validate_allowed_domains(self.allowed_domains.clone().or_else(|| {
                if self.excluded_domains.is_some() {
                    None
                } else {
                    allowed_domains
                }
            }))?;
        self.execute(query, allowed_domains.as_deref()).await
    }

    async fn execute(
        &self,
        query: &str,
        allowed_domains: Option<&[String]>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        let provider_query = scoped_search_query(query, allowed_domains);
        let result = self
            .client
            .call_tool_candidates(
                // Public docs use camelCase; the live Coding Plan catalog
                // currently advertises the snake_case compatibility spelling.
                &["webSearchPrime", "web_search_prime"],
                serde_json::json!({"search_query": provider_query}),
            )
            .await
            .map_err(|error| self.map_error(error))?;
        let content = text_content(&result)
            .map_err(|_| execution_error("Z.AI Search MCP returned invalid content"))?;
        project_content(&content, allowed_domains, self.excluded_domains.as_deref())
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
            ZaiMcpError::Quota => {
                execution_error("Z.AI Coding Plan monthly Search/Reader/Zread quota was reached")
            }
            ZaiMcpError::Unavailable => {
                execution_error("Z.AI Coding Plan Search MCP is temporarily unavailable")
            }
            ZaiMcpError::Protocol => {
                execution_error("Z.AI Coding Plan Search MCP protocol negotiation failed")
            }
            ZaiMcpError::InvalidResponse | ZaiMcpError::Rejected => {
                execution_error("Z.AI Coding Plan Search MCP returned an invalid response")
            }
        }
    }
}

fn scoped_search_query(query: &str, allowed_domains: Option<&[String]>) -> String {
    let Some(domains) = allowed_domains.filter(|domains| !domains.is_empty()) else {
        return query.to_owned();
    };
    if domains.len() == 1 {
        return format!("{query} site:{}", domains[0]);
    }
    let sites = domains
        .iter()
        .map(|domain| format!("site:{domain}"))
        .collect::<Vec<_>>()
        .join(" OR ");
    format!("{query} ({sites})")
}

fn project_content(
    content: &str,
    allowed_domains: Option<&[String]>,
    excluded_domains: Option<&[String]>,
) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
    if content.len() > MAX_RENDERED_BYTES {
        return Err(execution_error(
            "Z.AI search response exceeded the projection limit",
        ));
    }
    let mut payload: serde_json::Value = serde_json::from_str(content)
        .map_err(|_| execution_error("Z.AI search returned an unsupported result format"))?;
    // Search MCP can JSON-encode its result JSON as a string. Unwrap only a
    // bounded number of layers before applying the same strict projection;
    // never render undecoded provider text or include it in parse errors.
    for _ in 0..MAX_JSON_STRING_LAYERS {
        let Some(encoded) = payload.as_str() else {
            break;
        };
        payload = serde_json::from_str(encoded)
            .map_err(|_| execution_error("Z.AI search returned an unsupported result format"))?;
    }
    let values = payload
        .as_array()
        .or_else(|| {
            payload
                .get("search_result")
                .and_then(serde_json::Value::as_array)
        })
        .or_else(|| payload.get("results").and_then(serde_json::Value::as_array))
        .ok_or_else(|| execution_error("Z.AI search returned an unsupported result format"))?;
    let results = values
        .iter()
        .filter_map(|value| {
            Some(super::kimi_code::SearchResult {
                url: value
                    .get("link")
                    .or_else(|| value.get("url"))?
                    .as_str()?
                    .to_owned(),
                title: value
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                site_name: value
                    .get("media")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                snippet: value
                    .get("content")
                    .or_else(|| value.get("snippet"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            })
        })
        .collect();
    // Shared provider-neutral Grok formatting filters both text and citations.
    super::kimi_code::project_results(results, allowed_domains, excluded_domains)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_content(payload: serde_json::Value, string_layers: usize) -> String {
        let mut text = payload.to_string();
        for _ in 0..string_layers {
            text = serde_json::to_string(&text).unwrap();
        }
        // Exercise the same MCP text extraction used by execute(). These are
        // synthetic results, never a capture of an authenticated response.
        text_content(&serde_json::json!({
            "content": [{"type": "text", "text": text}]
        }))
        .unwrap()
    }

    #[test]
    fn zai_search_decodes_string_wrapped_results_and_preserves_filters() {
        let results = serde_json::json!([
            {"title": "reference", "link": "https://example.com/doc", "content": "safe snippet"},
            {"title": "blocked", "url": "https://blocked.example/doc", "snippet": "excluded-content"}
        ]);
        let allow = vec!["example.com".to_owned()];
        let block = vec!["blocked.example".to_owned()];
        for payload in [
            results.clone(),
            serde_json::json!({"search_result": results, "request_id": "private-trace"}),
            serde_json::json!({"results": results}),
        ] {
            for layers in 0..=2 {
                let content = encoded_content(payload.clone(), layers);
                for (allowed, excluded) in [
                    (Some(allow.as_slice()), None),
                    (None, Some(block.as_slice())),
                ] {
                    let output = project_content(&content, allowed, excluded).unwrap();
                    assert!(output.content.contains("safe snippet"));
                    assert!(!output.content.contains("excluded-content"));
                    assert!(!output.content.contains("private-trace"));
                    assert_eq!(output.citations(), vec!["https://example.com/doc"]);
                }
            }
        }
    }

    #[test]
    fn zai_search_accepts_encoded_empty_results() {
        for layers in 0..=2 {
            let output =
                project_content(&encoded_content(serde_json::json!([]), layers), None, None)
                    .unwrap();
            assert!(output.citations().is_empty());
        }
    }

    #[test]
    fn zai_search_rejects_excessive_encoding_and_redacts_invalid_content() {
        for content in [
            encoded_content(serde_json::json!([]), 3),
            encoded_content(serde_json::json!({"error": "sentinel-secret"}), 1),
            encoded_content(serde_json::json!("sentinel-secret"), 1),
            "{sentinel-secret".to_owned(),
            "null".to_owned(),
        ] {
            let error = project_content(&content, None, None).err().unwrap();
            assert!(!error.to_string().contains("sentinel-secret"));
        }
        let oversized = encoded_content(
            serde_json::json!([{
                "link": "https://example.com", "content": "x".repeat(MAX_RENDERED_BYTES)
            }]),
            1,
        );
        assert!(project_content(&oversized, None, None).is_err());
    }

    #[test]
    fn domain_filters_are_sent_to_search_and_applied_to_citations() {
        assert_eq!(
            scoped_search_query("rust async", Some(&["docs.rs".to_owned()])),
            "rust async site:docs.rs"
        );
        assert_eq!(
            scoped_search_query(
                "rust async",
                Some(&["docs.rs".to_owned(), "rust-lang.org".to_owned()]),
            ),
            "rust async (site:docs.rs OR site:rust-lang.org)"
        );
    }

    #[test]
    fn zai_search_filters_result_text_and_citations_not_just_query_operators() {
        let content = serde_json::json!({"request_id": "private-trace", "search_result": [
            {"title": "allowed", "link": "https://example.com/doc", "content": "safe snippet"},
            {"title": "blocked", "link": "https://blocked.example/doc", "content": "excluded-content"}
        ]}).to_string();
        let allow = vec!["example.com".to_owned()];
        let output = project_content(&content, Some(&allow), None).unwrap();
        assert!(output.content.contains("safe snippet"));
        assert!(!output.content.contains("excluded-content"));
        assert!(!output.content.contains("private-trace"));
        assert_eq!(output.citations(), vec!["https://example.com/doc"]);
        let block = vec!["blocked.example".to_owned()];
        assert!(
            !project_content(&content, None, Some(&block))
                .unwrap()
                .content
                .contains("excluded-content")
        );
        assert!(project_content("unstructured provider text", Some(&allow), None).is_err());
        let error = project_content(r#"{"error": "sentinel-secret"}"#, None, None)
            .err()
            .unwrap();
        assert!(!error.to_string().contains("sentinel-secret"));
    }
}
