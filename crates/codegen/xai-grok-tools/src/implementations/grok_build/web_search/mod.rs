//! `web_search` tool — new architecture (`Tool` trait).
//!
//! Calls the Responses API with web search capability. Reads the
//! pre-constructed `WebSearchClient` from Resources (inserted by
//! `with_backend()` when the config is `Enabled`).

use crate::implementations::web_search::CodexWebSearchContext;
use crate::implementations::web_search::client::WebSearchClient;
use crate::types::output::WebSearchOutput;
use crate::types::requirements::{Expr, ToolRequirement};
use crate::types::tool::{ToolKind, ToolNamespace};

const LEGACY_WEB_SEARCH_DESCRIPTION: &str = "Search the web for up-to-date information, tailored for coding and software development tasks.";
const CODEX_WEB_SEARCH_DESCRIPTION: &str = r#"Access the internet for current information and source-backed answers.

Commands (combine independent commands in one call when useful):
- search_query: search with `q`, optional `domains`, and optional recency in days.
- image_query: find images with the same query shape.
- open: open a returned `ref_id` or a literal HTTP(S) URL; optional `lineno` jumps to a line.
- click: follow numbered link `id` from a previously opened `ref_id`.
- find: locate `pattern` inside a returned ref or literal URL.
- screenshot: capture zero-indexed PDF page `pageno` from a ref or URL.
- finance: look up equity, fund, crypto, or index tickers. `market` is an ISO
  3166-1 alpha-3 country code such as `USA`, `OTC`, or an empty string for
  cryptocurrency. Example: `{"finance":[{"ticker":"MSFT","type":"equity","market":"USA"}]}`.
- weather: forecast a location, with optional start date and duration.
- sports: request a schedule or standings for a supported league and optional team/date filters.
- time: resolve a UTC offset such as `+02:00`.
- response_length: request short, medium, or long output.

Reuse returned refs across calls: search, then open a result, find relevant text, and click numbered links as needed. `open` also fetches arbitrary public URLs and is preferred when local web_fetch rejects a domain. Use multiple queries/commands per call when independent, omit null or empty arrays, and send no more than four search queries. Four search queries require medium or long response_length. Preserve source URLs from results for citations. Use this tool whenever freshness, verification, precise sourcing, or changing facts matter, including current coding and software research."#;

/// Finalization-only provider switch for the exported web-search contract.
/// It is sealed by AgentBuilder after user tool params are merged.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WebSearchParams {
    #[serde(default)]
    pub codex_subscription: bool,
}

impl crate::types::resources::ResourceType for WebSearchParams {
    const ID: &'static str = "grok_build.WebSearch";
}

// ───────────────────────────────────────────────────────────────────────────
// Input
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchInput {
    /// Backward-compatible single-query shortcut. Leave empty when using one
    /// of the structured command arrays below.
    #[serde(default)]
    #[schemars(
        description = "A single search query. Leave empty when using structured commands such as search_query, open, click, or find."
    )]
    pub query: String,
    #[schemars(description = "Optional list of domains to restrict search to.")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Internet search queries. Codex subscription sessions accept up to four per call."
    )]
    pub search_query: Option<Vec<WebSearchQuery>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Image-search queries.")]
    pub image_query: Option<Vec<WebSearchQuery>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Open search references or literal HTTP(S) URLs, optionally at a line number."
    )]
    pub open: Option<Vec<WebSearchOpen>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Follow a numbered link from a previously opened search reference.")]
    pub click: Option<Vec<WebSearchClick>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Find a text pattern in a search reference or literal URL.")]
    pub find: Option<Vec<WebSearchFind>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Capture a zero-indexed page from a PDF search reference or URL.")]
    pub screenshot: Option<Vec<WebSearchScreenshot>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finance: Option<Vec<WebSearchFinance>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather: Option<Vec<WebSearchWeather>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sports: Option<Vec<WebSearchSports>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<Vec<WebSearchTime>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_length: Option<WebSearchResponseLength>,
    /// Shell-owned, dispatch-only context for ChatGPT Codex standalone
    /// search. The field is omitted from the exported JSON schema and is
    /// overwritten by the shell after model arguments are validated.
    #[doc(hidden)]
    #[serde(
        default,
        rename = "_grok_codex_context",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(skip)]
    pub codex_context: Option<CodexWebSearchContext>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct LegacyWebSearchInput {
    /// A single search query.
    pub query: String,
    /// Optional list of domains to restrict search to.
    pub allowed_domains: Option<Vec<String>>,
}

/// Provider-specific schema used when a web tool is re-registered during a
/// model-provider switch. Initial agent construction uses the same selection
/// through the built-in tool metadata finalizer.
pub fn provider_input_schema(codex_subscription: bool) -> serde_json::Value {
    if codex_subscription {
        serde_json::to_value(schemars::schema_for!(WebSearchInput))
            .expect("Codex web search schema serializes")
    } else {
        serde_json::to_value(schemars::schema_for!(LegacyWebSearchInput))
            .expect("legacy web search schema serializes")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchQuery {
    pub q: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recency: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchOpen {
    pub ref_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineno: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchClick {
    pub ref_id: String,
    pub id: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchFind {
    pub ref_id: String,
    pub pattern: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchScreenshot {
    pub ref_id: String,
    pub pageno: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchFinance {
    /// Ticker symbol to look up.
    pub ticker: String,
    /// Asset type to look up.
    #[serde(rename = "type")]
    pub asset_type: WebSearchFinanceAssetType,
    /// ISO 3166-1 alpha-3 country code, "OTC", or "" for cryptocurrency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchFinanceAssetType {
    Equity,
    Fund,
    Crypto,
    Index,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchWeather {
    pub location: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchSports {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<WebSearchSportsTool>,
    #[serde(rename = "fn")]
    pub operation: WebSearchSportsOperation,
    pub league: WebSearchSportsLeague,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opponent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_games: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchSportsTool {
    Sports,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchSportsOperation {
    Schedule,
    Standings,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchSportsLeague {
    Nba,
    Wnba,
    Nfl,
    Nhl,
    Mlb,
    Epl,
    Ncaamb,
    Ncaawb,
    Ipl,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchTime {
    pub utc_offset: String,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchResponseLength {
    Short,
    Medium,
    Long,
}

impl WebSearchInput {
    fn validate(&self) -> Result<(), xai_tool_runtime::ToolError> {
        for (name, queries) in [
            ("search_query", self.search_query.as_deref()),
            ("image_query", self.image_query.as_deref()),
        ] {
            if queries.is_some_and(|queries| queries.len() > 4) {
                return Err(xai_tool_runtime::ToolError::invalid_arguments(format!(
                    "{name} accepts at most four queries per call"
                )));
            }
            for (index, query) in queries.unwrap_or_default().iter().enumerate() {
                require_nonempty(&format!("{name}[{index}].q"), &query.q)?;
                for (domain_index, domain) in query
                    .domains
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                {
                    require_nonempty(&format!("{name}[{index}].domains[{domain_index}]"), domain)?;
                }
            }
        }
        if self
            .search_query
            .as_ref()
            .is_some_and(|queries| queries.len() > 3)
            && !matches!(
                self.response_length,
                Some(WebSearchResponseLength::Medium | WebSearchResponseLength::Long)
            )
        {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "search_query with four entries requires response_length to be medium or long",
            ));
        }
        for (index, domain) in self
            .allowed_domains
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            require_nonempty(&format!("allowed_domains[{index}]"), domain)?;
        }
        for (index, operation) in self.open.as_deref().unwrap_or_default().iter().enumerate() {
            require_nonempty(&format!("open[{index}].ref_id"), &operation.ref_id)?;
        }
        for (index, operation) in self.click.as_deref().unwrap_or_default().iter().enumerate() {
            require_nonempty(&format!("click[{index}].ref_id"), &operation.ref_id)?;
        }
        for (index, operation) in self.find.as_deref().unwrap_or_default().iter().enumerate() {
            require_nonempty(&format!("find[{index}].ref_id"), &operation.ref_id)?;
            require_nonempty(&format!("find[{index}].pattern"), &operation.pattern)?;
        }
        for (index, operation) in self
            .screenshot
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            require_nonempty(&format!("screenshot[{index}].ref_id"), &operation.ref_id)?;
        }
        for (index, operation) in self
            .finance
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            require_nonempty(&format!("finance[{index}].ticker"), &operation.ticker)?;
        }
        for (index, operation) in self
            .weather
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            require_nonempty(&format!("weather[{index}].location"), &operation.location)?;
        }
        for (index, operation) in self.time.as_deref().unwrap_or_default().iter().enumerate() {
            require_nonempty(&format!("time[{index}].utc_offset"), &operation.utc_offset)?;
        }
        if !self.has_operation() {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "web_search requires a query or at least one non-empty structured command",
            ));
        }
        Ok(())
    }

    fn has_operation(&self) -> bool {
        !self.query.trim().is_empty()
            || [
                self.search_query.as_ref().map(Vec::len),
                self.image_query.as_ref().map(Vec::len),
                self.open.as_ref().map(Vec::len),
                self.click.as_ref().map(Vec::len),
                self.find.as_ref().map(Vec::len),
                self.screenshot.as_ref().map(Vec::len),
                self.finance.as_ref().map(Vec::len),
                self.weather.as_ref().map(Vec::len),
                self.sports.as_ref().map(Vec::len),
                self.time.as_ref().map(Vec::len),
            ]
            .into_iter()
            .flatten()
            .any(|length| length > 0)
    }

    fn commands(&self) -> serde_json::Value {
        let mut commands = serde_json::Map::new();
        macro_rules! insert_command {
            ($name:literal, $value:expr) => {
                if let Some(value) = &$value
                    && !value.is_empty()
                {
                    commands.insert(
                        $name.to_string(),
                        serde_json::to_value(value).expect("web search command serializes"),
                    );
                }
            };
        }
        insert_command!("search_query", self.search_query);
        insert_command!("image_query", self.image_query);
        insert_command!("open", self.open);
        insert_command!("click", self.click);
        insert_command!("find", self.find);
        insert_command!("screenshot", self.screenshot);
        insert_command!("finance", self.finance);
        insert_command!("weather", self.weather);
        insert_command!("sports", self.sports);
        insert_command!("time", self.time);
        if let Some(response_length) = self.response_length {
            commands.insert(
                "response_length".to_string(),
                serde_json::to_value(response_length).expect("response length serializes"),
            );
        }
        if !self.query.trim().is_empty() && !commands.contains_key("search_query") {
            commands.insert(
                "search_query".to_string(),
                serde_json::json!([{ "q": self.query.trim() }]),
            );
        }
        serde_json::Value::Object(commands)
    }

    fn command_label(&self) -> String {
        if !self.query.trim().is_empty() {
            return self.query.clone();
        }
        if let Some(query) = self
            .search_query
            .as_deref()
            .and_then(|queries| queries.first())
            .or_else(|| {
                self.image_query
                    .as_deref()
                    .and_then(|queries| queries.first())
            })
        {
            return query.q.clone();
        }
        for (kind, reference) in [
            (
                "open",
                self.open
                    .as_deref()
                    .and_then(|operations| operations.first())
                    .map(|operation| operation.ref_id.as_str()),
            ),
            (
                "click",
                self.click
                    .as_deref()
                    .and_then(|operations| operations.first())
                    .map(|operation| operation.ref_id.as_str()),
            ),
            (
                "find",
                self.find
                    .as_deref()
                    .and_then(|operations| operations.first())
                    .map(|operation| operation.ref_id.as_str()),
            ),
        ] {
            if let Some(reference) = reference {
                return format!("{kind}: {reference}");
            }
        }
        "web lookup".to_string()
    }
}

fn require_nonempty(field: &str, value: &str) -> Result<(), xai_tool_runtime::ToolError> {
    if value.trim().is_empty() {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

// ───────────────────────────────────────────────────────────────────────────
// Tool implementation
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct WebSearchTool;

/// Runtime-only wrapper used when model switching installs the Codex search
/// contract on an already-built ToolBridge. The built-in registry continues
/// to use [`WebSearchTool`] plus sealed finalization params.
#[derive(Debug, Default)]
pub struct CodexWebSearchTool;

impl crate::types::tool_metadata::ToolMetadata for WebSearchTool {
    fn kind(&self) -> ToolKind {
        ToolKind::WebSearch
    }

    fn tool_namespace(&self) -> ToolNamespace {
        ToolNamespace::GrokBuild
    }

    fn description_template(&self) -> &str {
        LEGACY_WEB_SEARCH_DESCRIPTION
    }

    fn versioned_definition(
        &self,
        _contract_version: Option<&str>,
        client_name: &str,
        description_override: Option<&str>,
        renderer: &crate::types::template_renderer::TemplateRenderer,
        param_map: &std::collections::HashMap<String, String>,
        input_schema: &serde_json::Value,
        effective_params: &serde_json::Value,
    ) -> crate::types::definition::ToolDefinition {
        let params: WebSearchParams =
            serde_json::from_value(effective_params.clone()).unwrap_or_default();
        let description = description_override.unwrap_or(if params.codex_subscription {
            CODEX_WEB_SEARCH_DESCRIPTION
        } else {
            LEGACY_WEB_SEARCH_DESCRIPTION
        });
        let description = renderer
            .render(description)
            .unwrap_or_else(|_| description.to_string());
        let schema = if params.codex_subscription {
            input_schema.clone()
        } else {
            provider_input_schema(false)
        };
        let schema = if param_map.is_empty() {
            schema
        } else {
            crate::util::remap::remap_schema_properties(&schema, param_map)
        };
        crate::types::definition::ToolDefinition::function(client_name, Some(&description), schema)
    }

    fn requires_expr(&self) -> Expr<ToolRequirement> {
        Expr::True
    }
}

impl crate::types::tool_metadata::ToolMetadata for CodexWebSearchTool {
    fn kind(&self) -> ToolKind {
        ToolKind::WebSearch
    }

    fn tool_namespace(&self) -> ToolNamespace {
        ToolNamespace::GrokBuild
    }

    fn description_template(&self) -> &str {
        CODEX_WEB_SEARCH_DESCRIPTION
    }

    fn requires_expr(&self) -> Expr<ToolRequirement> {
        Expr::True
    }
}

impl xai_tool_runtime::Tool for WebSearchTool {
    type Args = WebSearchInput;
    type Output = WebSearchOutput;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new("web_search").expect("valid tool id")
    }

    fn description(
        &self,
        _ctx: &::xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        xai_tool_types::ToolDescription::new(
            "web_search",
            crate::types::tool_metadata::ToolMetadata::description_template(self),
        )
    }

    fn capabilities(&self) -> xai_tool_protocol::ToolCapabilities {
        xai_tool_protocol::ToolCapabilities {
            is_read_only: true,
            tool_scope: Some(xai_tool_protocol::ToolScope::Read),
            ..Default::default()
        }
    }

    #[tracing::instrument(name = "tool.web_search", skip_all)]
    async fn run(
        &self,
        ctx: xai_tool_runtime::ToolCallContext,
        input: WebSearchInput,
    ) -> Result<WebSearchOutput, xai_tool_runtime::ToolError> {
        run_web_search(ctx, input).await
    }
}

impl xai_tool_runtime::Tool for CodexWebSearchTool {
    type Args = WebSearchInput;
    type Output = WebSearchOutput;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new("web_search").expect("valid tool id")
    }

    fn description(
        &self,
        _ctx: &::xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        xai_tool_types::ToolDescription::new(
            "web_search",
            crate::types::tool_metadata::ToolMetadata::description_template(self),
        )
    }

    fn capabilities(&self) -> xai_tool_protocol::ToolCapabilities {
        xai_tool_protocol::ToolCapabilities {
            is_read_only: true,
            tool_scope: Some(xai_tool_protocol::ToolScope::Read),
            ..Default::default()
        }
    }

    #[tracing::instrument(name = "tool.web_search", skip_all)]
    async fn run(
        &self,
        ctx: xai_tool_runtime::ToolCallContext,
        input: WebSearchInput,
    ) -> Result<WebSearchOutput, xai_tool_runtime::ToolError> {
        run_web_search(ctx, input).await
    }
}

async fn run_web_search(
    ctx: xai_tool_runtime::ToolCallContext,
    input: WebSearchInput,
) -> Result<WebSearchOutput, xai_tool_runtime::ToolError> {
    use crate::types::tool_metadata::shared_resources;
    let resources = shared_resources(&ctx)?;

    let client;
    {
        let res = resources.lock().await;
        client = res.require::<WebSearchClient>()?.clone();
    }

    input.validate()?;
    let commands = input.commands();
    if commands.as_object().is_none_or(|commands| {
        commands.is_empty() || commands.keys().all(|command| command == "response_length")
    }) {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "web_search requires a query or at least one structured command",
        ));
    }
    let query = input.command_label();
    let result = client
        .run_commands(
            &query,
            commands,
            input.allowed_domains.clone(),
            input.codex_context.as_ref(),
        )
        .await?;

    Ok(WebSearchOutput {
        query,
        content: result.content,
        citations: result.citations,
        references: result.references,
        allowed_domains: input.allowed_domains.clone(),
        pre_formatted: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::resources::Resources;
    use crate::types::tool_metadata::test_ctx_with_call_id;

    #[test]
    fn tool_name_and_description() {
        let tool = WebSearchTool;
        assert_eq!(xai_tool_runtime::Tool::id(&tool).as_str(), "web_search");
        assert!(
            crate::types::tool_metadata::ToolMetadata::description_template(&tool)
                .contains("Search the web")
        );
        let codex = CodexWebSearchTool;
        let description = crate::types::tool_metadata::ToolMetadata::description_template(&codex);
        assert!(description.contains("search_query"));
        assert!(description.contains("Reuse returned refs"));
    }

    #[test]
    fn xai_contract_remains_query_only_while_codex_exports_structured_commands() {
        let legacy = provider_input_schema(false);
        let legacy_properties = legacy["properties"]
            .as_object()
            .expect("legacy schema properties");
        assert_eq!(
            legacy_properties
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            ["allowed_domains".to_string(), "query".to_string()]
                .into_iter()
                .collect()
        );
        assert_eq!(legacy["required"], serde_json::json!(["query"]));
        assert!(legacy_properties.get("open").is_none());

        let codex = provider_input_schema(true);
        let codex_properties = codex["properties"]
            .as_object()
            .expect("Codex schema properties");
        for command in [
            "search_query",
            "image_query",
            "open",
            "click",
            "find",
            "screenshot",
            "finance",
            "weather",
            "sports",
            "time",
            "response_length",
        ] {
            assert!(
                codex_properties.contains_key(command),
                "Codex schema omitted {command}"
            );
        }
        assert!(
            !codex_properties.contains_key("_grok_codex_context"),
            "dispatch-only context must never be model-visible"
        );
        assert_eq!(
            codex["$defs"]["WebSearchFinance"]["properties"]["market"]["description"],
            "ISO 3166-1 alpha-3 country code, \"OTC\", or \"\" for cryptocurrency."
        );
    }

    #[test]
    fn codex_web_commands_preserve_search_navigation_and_utility_shapes() {
        let input = WebSearchInput {
            search_query: Some(vec![WebSearchQuery {
                q: "Codex release notes".to_string(),
                recency: Some(7),
                domains: Some(vec!["openai.com".to_string()]),
            }]),
            image_query: Some(vec![WebSearchQuery {
                q: "Codex terminal UI".to_string(),
                recency: None,
                domains: None,
            }]),
            open: Some(vec![WebSearchOpen {
                ref_id: "turn0search0".to_string(),
                lineno: Some(12),
            }]),
            click: Some(vec![WebSearchClick {
                ref_id: "turn0view0".to_string(),
                id: 4,
            }]),
            find: Some(vec![WebSearchFind {
                ref_id: "https://example.com/docs".to_string(),
                pattern: "installation".to_string(),
            }]),
            screenshot: Some(vec![WebSearchScreenshot {
                ref_id: "turn0view1".to_string(),
                pageno: 2,
            }]),
            finance: Some(vec![WebSearchFinance {
                ticker: "BTC".to_string(),
                asset_type: WebSearchFinanceAssetType::Crypto,
                market: Some(String::new()),
            }]),
            weather: Some(vec![WebSearchWeather {
                location: "Belgium, Brussels".to_string(),
                start: Some("2026-07-17".to_string()),
                duration: Some(3),
            }]),
            sports: Some(vec![WebSearchSports {
                tool: Some(WebSearchSportsTool::Sports),
                operation: WebSearchSportsOperation::Schedule,
                league: WebSearchSportsLeague::Nba,
                team: Some("BOS".to_string()),
                opponent: None,
                date_from: Some("2026-10-01".to_string()),
                date_to: None,
                num_games: Some(2),
                locale: Some("en-US".to_string()),
            }]),
            time: Some(vec![WebSearchTime {
                utc_offset: "+02:00".to_string(),
            }]),
            response_length: Some(WebSearchResponseLength::Long),
            ..Default::default()
        };
        input.validate().expect("representative commands are valid");

        assert_eq!(
            input.commands(),
            serde_json::json!({
                "search_query": [{
                    "q": "Codex release notes",
                    "recency": 7,
                    "domains": ["openai.com"],
                }],
                "image_query": [{"q": "Codex terminal UI"}],
                "open": [{"ref_id": "turn0search0", "lineno": 12}],
                "click": [{"ref_id": "turn0view0", "id": 4}],
                "find": [{
                    "ref_id": "https://example.com/docs",
                    "pattern": "installation",
                }],
                "screenshot": [{"ref_id": "turn0view1", "pageno": 2}],
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
            })
        );
    }

    #[test]
    fn legacy_query_maps_to_one_search_query_and_query_count_is_bounded() {
        let input = WebSearchInput {
            query: "Rust 2026".to_string(),
            ..Default::default()
        };
        assert_eq!(
            input.commands(),
            serde_json::json!({"search_query": [{"q": "Rust 2026"}]})
        );
        assert!(input.validate().is_ok());

        let too_many = WebSearchInput {
            search_query: Some(
                (0..5)
                    .map(|index| WebSearchQuery {
                        q: format!("query {index}"),
                        recency: None,
                        domains: None,
                    })
                    .collect(),
            ),
            ..Default::default()
        };
        assert!(too_many.validate().is_err());
    }

    #[test]
    fn codex_web_commands_enforce_query_length_hint_and_required_text() {
        let four_queries = || {
            (0..4)
                .map(|index| WebSearchQuery {
                    q: format!("query {index}"),
                    recency: None,
                    domains: None,
                })
                .collect()
        };
        let short = WebSearchInput {
            search_query: Some(four_queries()),
            response_length: Some(WebSearchResponseLength::Short),
            ..Default::default()
        };
        assert!(
            short
                .validate()
                .unwrap_err()
                .to_string()
                .contains("requires response_length")
        );
        let medium = WebSearchInput {
            search_query: Some(four_queries()),
            response_length: Some(WebSearchResponseLength::Medium),
            ..Default::default()
        };
        assert!(medium.validate().is_ok());

        for invalid in [
            WebSearchInput {
                search_query: Some(vec![WebSearchQuery {
                    q: "  ".to_string(),
                    recency: None,
                    domains: None,
                }]),
                ..Default::default()
            },
            WebSearchInput {
                open: Some(vec![WebSearchOpen {
                    ref_id: String::new(),
                    lineno: None,
                }]),
                ..Default::default()
            },
            WebSearchInput {
                find: Some(vec![WebSearchFind {
                    ref_id: "turn0view0".to_string(),
                    pattern: "  ".to_string(),
                }]),
                ..Default::default()
            },
        ] {
            assert!(invalid.validate().is_err());
        }
    }

    #[test]
    fn codex_web_commands_omit_empty_arrays_and_require_an_operation() {
        let empty = WebSearchInput {
            search_query: Some(Vec::new()),
            open: Some(Vec::new()),
            response_length: Some(WebSearchResponseLength::Long),
            ..Default::default()
        };
        assert_eq!(
            empty.commands(),
            serde_json::json!({"response_length": "long"})
        );
        assert!(
            empty
                .validate()
                .unwrap_err()
                .to_string()
                .contains("non-empty structured command")
        );
    }

    #[tokio::test]
    async fn errors_when_client_not_in_resources() {
        let resources = Resources::new();
        let tool = WebSearchTool;
        let result = xai_tool_runtime::Tool::run(
            &tool,
            test_ctx_with_call_id(resources.into_shared(), "test-call"),
            WebSearchInput {
                query: "test".into(),
                allowed_domains: None,
                ..Default::default()
            },
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("missing required resource"),
            "Expected 'missing required resource' error, got: {err_msg}"
        );
    }
}
