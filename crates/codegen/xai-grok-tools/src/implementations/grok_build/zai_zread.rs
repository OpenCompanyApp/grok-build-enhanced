//! Provider-scoped read-only tools for the Z.AI Coding Plan Zread MCP service.
//!
//! The public tool names are prefixed with `zread_` so they cannot collide with
//! local workspace tools such as `read_file`. Every call uses the session's
//! dynamically resolved Z.AI credential through [`ZaiMcpClient`].

use crate::{
    implementations::zai_mcp::{ZREAD_ENDPOINT, ZaiMcpClient, ZaiMcpError, text_content},
    types::{
        ToolInput,
        output::ToolOutput,
        requirements::{Expr, ToolRequirement},
        tool::{ToolKind, ToolNamespace},
        tool_metadata::{ToolMetadata, shared_resources},
    },
};

pub const ZREAD_SEARCH_DOC_TOOL_NAME: &str = "zread_search_doc";
pub const ZREAD_GET_REPO_STRUCTURE_TOOL_NAME: &str = "zread_get_repo_structure";
pub const ZREAD_READ_FILE_TOOL_NAME: &str = "zread_read_file";

const SEARCH_DOC_DESCRIPTION: &str = "Search documentation, issues, pull requests, news, and contributor knowledge for a public GitHub repository through Z.AI Coding Plan Zread. Use an owner/repository name, not a URL.";
const GET_REPO_STRUCTURE_DESCRIPTION: &str = "Get the directory structure and file list of a public GitHub repository through Z.AI Coding Plan Zread. Use this before reading unfamiliar repository files.";
const READ_FILE_DESCRIPTION: &str = "Read one complete file from a public GitHub repository through Z.AI Coding Plan Zread. This does not read local workspace files; use read_file for local paths.";
const MAX_QUERY_CHARS: usize = 8_192;
const MAX_FILE_PATH_CHARS: usize = 4_096;

#[derive(Clone, Debug)]
pub struct ZaiZreadClient {
    inner: ZaiMcpClient,
}

impl ZaiZreadClient {
    pub fn new(
        auth_provider: crate::types::SharedApiKeyProvider,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        Ok(Self {
            inner: ZaiMcpClient::new(ZREAD_ENDPOINT, auth_provider).map_err(map_zread_error)?,
        })
    }

    async fn call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolOutput, xai_tool_runtime::ToolError> {
        let result = self
            .inner
            .call_tool(tool_name, arguments)
            .await
            .map_err(map_zread_error)?;
        let text = text_content(&result).map_err(map_zread_error)?;
        Ok(ToolOutput::Text(text.into()))
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ZreadLanguage {
    En,
    Zh,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ZreadSearchDocInput {
    /// Public GitHub repository in `owner/repository` form.
    pub repo_name: String,
    /// Search term or natural-language question.
    pub query: String,
    /// Preferred result language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<ZreadLanguage>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ZreadRepoInput {
    /// Public GitHub repository in `owner/repository` form.
    pub repo_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ZreadReadFileInput {
    /// Public GitHub repository in `owner/repository` form.
    pub repo_name: String,
    /// Repository-relative file path using `/` separators.
    pub file_path: String,
}

impl From<ZreadSearchDocInput> for ToolInput {
    fn from(input: ZreadSearchDocInput) -> Self {
        Self::Dynamic(serde_json::to_value(input).expect("Zread search input serializes"))
    }
}

impl From<ZreadRepoInput> for ToolInput {
    fn from(input: ZreadRepoInput) -> Self {
        Self::Dynamic(serde_json::to_value(input).expect("Zread repository input serializes"))
    }
}

impl From<ZreadReadFileInput> for ToolInput {
    fn from(input: ZreadReadFileInput) -> Self {
        Self::Dynamic(serde_json::to_value(input).expect("Zread file input serializes"))
    }
}

#[derive(Debug, Default)]
pub struct ZreadSearchDocTool;

#[derive(Debug, Default)]
pub struct ZreadGetRepoStructureTool;

#[derive(Debug, Default)]
pub struct ZreadReadFileTool;

macro_rules! impl_zread_metadata {
    ($tool:ty, $kind:expr, $description:expr) => {
        impl ToolMetadata for $tool {
            fn kind(&self) -> ToolKind {
                $kind
            }

            fn tool_namespace(&self) -> ToolNamespace {
                ToolNamespace::GrokBuild
            }

            fn description_template(&self) -> &str {
                $description
            }

            fn requires_expr(&self) -> Expr<ToolRequirement> {
                Expr::True
            }
        }
    };
}

impl_zread_metadata!(ZreadSearchDocTool, ToolKind::Search, SEARCH_DOC_DESCRIPTION);
impl_zread_metadata!(
    ZreadGetRepoStructureTool,
    ToolKind::ListDir,
    GET_REPO_STRUCTURE_DESCRIPTION
);
impl_zread_metadata!(ZreadReadFileTool, ToolKind::Read, READ_FILE_DESCRIPTION);

macro_rules! zread_tool_description {
    ($name:expr, $description:expr) => {
        xai_tool_types::ToolDescription::new($name, $description)
    };
}

impl xai_tool_runtime::Tool for ZreadSearchDocTool {
    type Args = ZreadSearchDocInput;
    type Output = ToolOutput;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new(ZREAD_SEARCH_DOC_TOOL_NAME).expect("valid tool id")
    }

    fn description(
        &self,
        _ctx: &xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        zread_tool_description!(ZREAD_SEARCH_DOC_TOOL_NAME, SEARCH_DOC_DESCRIPTION)
    }

    fn capabilities(&self) -> xai_tool_protocol::ToolCapabilities {
        read_only_capabilities()
    }

    #[tracing::instrument(name = "tool.zread_search_doc", skip_all)]
    async fn run(
        &self,
        ctx: xai_tool_runtime::ToolCallContext,
        input: ZreadSearchDocInput,
    ) -> Result<ToolOutput, xai_tool_runtime::ToolError> {
        validate_repo_name(&input.repo_name)?;
        validate_nonempty_bounded("query", &input.query, MAX_QUERY_CHARS)?;
        let client = zread_client(&ctx).await?;
        client
            .call(
                "search_doc",
                serde_json::to_value(input).expect("Zread search input serializes"),
            )
            .await
    }
}

impl xai_tool_runtime::Tool for ZreadGetRepoStructureTool {
    type Args = ZreadRepoInput;
    type Output = ToolOutput;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new(ZREAD_GET_REPO_STRUCTURE_TOOL_NAME).expect("valid tool id")
    }

    fn description(
        &self,
        _ctx: &xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        zread_tool_description!(
            ZREAD_GET_REPO_STRUCTURE_TOOL_NAME,
            GET_REPO_STRUCTURE_DESCRIPTION
        )
    }

    fn capabilities(&self) -> xai_tool_protocol::ToolCapabilities {
        read_only_capabilities()
    }

    #[tracing::instrument(name = "tool.zread_get_repo_structure", skip_all)]
    async fn run(
        &self,
        ctx: xai_tool_runtime::ToolCallContext,
        input: ZreadRepoInput,
    ) -> Result<ToolOutput, xai_tool_runtime::ToolError> {
        validate_repo_name(&input.repo_name)?;
        let client = zread_client(&ctx).await?;
        client
            .call(
                "get_repo_structure",
                serde_json::to_value(input).expect("Zread repository input serializes"),
            )
            .await
    }
}

impl xai_tool_runtime::Tool for ZreadReadFileTool {
    type Args = ZreadReadFileInput;
    type Output = ToolOutput;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new(ZREAD_READ_FILE_TOOL_NAME).expect("valid tool id")
    }

    fn description(
        &self,
        _ctx: &xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        zread_tool_description!(ZREAD_READ_FILE_TOOL_NAME, READ_FILE_DESCRIPTION)
    }

    fn capabilities(&self) -> xai_tool_protocol::ToolCapabilities {
        read_only_capabilities()
    }

    #[tracing::instrument(name = "tool.zread_read_file", skip_all)]
    async fn run(
        &self,
        ctx: xai_tool_runtime::ToolCallContext,
        input: ZreadReadFileInput,
    ) -> Result<ToolOutput, xai_tool_runtime::ToolError> {
        validate_repo_name(&input.repo_name)?;
        validate_file_path(&input.file_path)?;
        let client = zread_client(&ctx).await?;
        client
            .call(
                "read_file",
                serde_json::to_value(input).expect("Zread file input serializes"),
            )
            .await
    }
}

fn read_only_capabilities() -> xai_tool_protocol::ToolCapabilities {
    xai_tool_protocol::ToolCapabilities {
        is_read_only: true,
        tool_scope: Some(xai_tool_protocol::ToolScope::Read),
        ..Default::default()
    }
}

async fn zread_client(
    ctx: &xai_tool_runtime::ToolCallContext,
) -> Result<ZaiZreadClient, xai_tool_runtime::ToolError> {
    let resources = shared_resources(ctx)?;
    let resources = resources.lock().await;
    resources.require::<ZaiZreadClient>().cloned()
}

fn validate_repo_name(repo_name: &str) -> Result<(), xai_tool_runtime::ToolError> {
    if repo_name != repo_name.trim() || repo_name.len() > 140 {
        return Err(invalid_repo_name());
    }
    let mut parts = repo_name.split('/');
    let Some(owner) = parts.next() else {
        return Err(invalid_repo_name());
    };
    let Some(repository) = parts.next() else {
        return Err(invalid_repo_name());
    };
    if parts.next().is_some()
        || !valid_repo_component(owner, 39)
        || !valid_repo_component(repository, 100)
    {
        return Err(invalid_repo_name());
    }
    Ok(())
}

fn valid_repo_component(component: &str, max_len: usize) -> bool {
    !component.is_empty()
        && component.len() <= max_len
        && !matches!(component, "." | "..")
        && component
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn invalid_repo_name() -> xai_tool_runtime::ToolError {
    xai_tool_runtime::ToolError::invalid_arguments(
        "repo_name must be a public GitHub repository in owner/repository form",
    )
}

fn validate_file_path(file_path: &str) -> Result<(), xai_tool_runtime::ToolError> {
    validate_nonempty_bounded("file_path", file_path, MAX_FILE_PATH_CHARS)?;
    if file_path != file_path.trim()
        || file_path.starts_with('/')
        || file_path.contains('\\')
        || file_path.contains(['\0', '?', '#'])
        || file_path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "file_path must be a safe repository-relative path using '/' separators",
        ));
    }
    Ok(())
}

fn validate_nonempty_bounded(
    field: &str,
    value: &str,
    max_chars: usize,
) -> Result<(), xai_tool_runtime::ToolError> {
    if value.trim().is_empty() || value.chars().count() > max_chars {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(format!(
            "{field} must contain between 1 and {max_chars} characters"
        )));
    }
    Ok(())
}

fn map_zread_error(error: ZaiMcpError) -> xai_tool_runtime::ToolError {
    let (code, message) = match error {
        ZaiMcpError::Authentication => (
            "zai_zread_authentication",
            "Z.AI Coding Plan authentication is required for Zread",
        ),
        ZaiMcpError::Quota => ("zai_zread_quota", "Z.AI Coding Plan MCP quota is exhausted"),
        ZaiMcpError::Protocol | ZaiMcpError::InvalidResponse => (
            "zai_zread_protocol",
            "Z.AI Coding Plan Zread returned an invalid response",
        ),
        ZaiMcpError::Unavailable => (
            "zai_zread_unavailable",
            "Z.AI Coding Plan Zread is temporarily unavailable",
        ),
        ZaiMcpError::Rejected => (
            "zai_zread_rejected",
            "Z.AI Coding Plan Zread rejected the request",
        ),
    };
    xai_tool_runtime::ToolError::custom(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_public_repository_names() {
        assert!(validate_repo_name("openai/codex").is_ok());
        for invalid in [
            "openai",
            "https://github.com/openai/codex",
            "openai/codex/extra",
            "../codex",
            "openai/",
            " openai/codex",
        ] {
            assert!(validate_repo_name(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn rejects_unsafe_repository_file_paths() {
        assert!(validate_file_path("crates/core/src/lib.rs").is_ok());
        for invalid in [
            "",
            "/etc/passwd",
            "../Cargo.toml",
            "src/../Cargo.toml",
            "src\\lib.rs",
            "README.md#section",
        ] {
            assert!(validate_file_path(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn serializes_provider_wire_arguments() {
        let input = ZreadSearchDocInput {
            repo_name: "openai/codex".to_owned(),
            query: "streamed tool calls".to_owned(),
            language: Some(ZreadLanguage::En),
        };
        assert_eq!(
            serde_json::to_value(input).unwrap(),
            serde_json::json!({
                "repo_name": "openai/codex",
                "query": "streamed tool calls",
                "language": "en"
            })
        );
    }
}
