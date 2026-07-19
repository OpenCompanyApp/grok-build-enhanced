use super::backends::{
    KimiCodeBackend, OpenAiCodexBackend, ResponsesBackend, ZaiCodingPlanBackend,
};
use super::types::{CodexWebSearchContext, WebSearchConfig};
use crate::attribution::SharedAttributionCallback;
use crate::types::SharedApiKeyProvider;
use crate::types::output::WebSearchReference;

#[derive(Clone)]
enum WebSearchBackend {
    Responses(ResponsesBackend),
    OpenAiCodex(OpenAiCodexBackend),
    KimiCode(KimiCodeBackend),
    ZaiCodingPlan(ZaiCodingPlanBackend),
}

/// Provider-neutral web-search client facade.
///
/// Backend-specific endpoint, authentication, request, retry, parsing, and
/// projection contracts live under `backends/`; this wrapper preserves the
/// original externally used constructor and search methods.
#[derive(Clone)]
pub struct WebSearchClient {
    backend: WebSearchBackend,
}

/// Provider-neutral result returned by the structured command entrypoint.
/// Standalone Codex search additionally supplies stable result references for
/// follow-up navigation; Responses-backed search leaves `references` empty.
#[derive(Clone, PartialEq, Eq)]
pub struct WebSearchCommandResult {
    pub content: String,
    pub citations: Vec<String>,
    pub references: Vec<WebSearchReference>,
}

impl std::fmt::Debug for WebSearchCommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSearchCommandResult")
            .finish_non_exhaustive()
    }
}

impl WebSearchClient {
    /// Create a web-search client from an enabled configuration.
    ///
    /// ChatGPT Codex requires a non-optional provider-owned `RequestAuth`
    /// resolver. Static keys remain supported only by the Responses backend.
    pub fn new(
        config: &WebSearchConfig,
        api_key_provider: Option<SharedApiKeyProvider>,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        let backend = match config {
            WebSearchConfig::Disabled => {
                return Err(super::backends::execution_error(
                    "Cannot create WebSearchClient from disabled config",
                ));
            }
            WebSearchConfig::Enabled {
                api_key,
                base_url,
                model,
                extra_headers,
                alpha_test_key,
            } => {
                let _ = alpha_test_key;
                WebSearchBackend::Responses(ResponsesBackend::new(
                    api_key,
                    base_url,
                    model,
                    extra_headers,
                    api_key_provider,
                )?)
            }
            WebSearchConfig::CodexSubscription {
                base_url,
                model,
                session_id,
                settings,
            } => {
                if !settings.mode.is_enabled() {
                    return Err(super::backends::execution_error(
                        "Cannot create a Codex web search client in disabled mode",
                    ));
                }
                let request_auth_provider = api_key_provider.ok_or_else(|| {
                    super::backends::execution_error(
                        "Cannot create a Codex web search client without provider authentication",
                    )
                })?;
                WebSearchBackend::OpenAiCodex(OpenAiCodexBackend::new(
                    base_url,
                    model,
                    session_id,
                    settings.clone(),
                    request_auth_provider,
                )?)
            }
            WebSearchConfig::KimiCode { base_url } => {
                let request_auth_provider = api_key_provider.ok_or_else(|| {
                    super::backends::execution_error(
                        "Cannot create a Kimi Code web search client without provider authentication",
                    )
                })?;
                WebSearchBackend::KimiCode(KimiCodeBackend::new(base_url, request_auth_provider)?)
            }
            WebSearchConfig::ZaiCodingPlan { endpoint } => {
                let request_auth_provider = api_key_provider.ok_or_else(|| {
                    super::backends::execution_error(
                        "Cannot create a Z.AI Coding Plan web search client without provider authentication",
                    )
                })?;
                WebSearchBackend::ZaiCodingPlan(ZaiCodingPlanBackend::new(
                    endpoint,
                    request_auth_provider,
                )?)
            }
        };
        Ok(Self { backend })
    }

    /// Wire a 401-attribution callback into this client.
    pub fn with_attribution_callback(
        mut self,
        callback: Option<SharedAttributionCallback>,
    ) -> Self {
        match &mut self.backend {
            WebSearchBackend::Responses(backend) => {
                backend.set_attribution_callback(callback);
            }
            WebSearchBackend::OpenAiCodex(backend) => {
                backend.set_attribution_callback(callback);
            }
            WebSearchBackend::KimiCode(backend) => {
                backend.set_attribution_callback(callback);
            }
            WebSearchBackend::ZaiCodingPlan(backend) => {
                backend.set_attribution_callback(callback);
            }
        }
        self
    }

    /// Perform a web-search query and return synthesized content plus unique
    /// citation URLs.
    pub async fn search(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<(String, Vec<String>), xai_tool_runtime::ToolError> {
        let result = match &self.backend {
            WebSearchBackend::Responses(backend) => backend.search(query, allowed_domains).await?,
            WebSearchBackend::OpenAiCodex(backend) => {
                backend.search(query, allowed_domains).await?
            }
            WebSearchBackend::KimiCode(backend) => backend.search(query, allowed_domains).await?,
            WebSearchBackend::ZaiCodingPlan(backend) => {
                backend.search(query, allowed_domains).await?
            }
        };
        let citations = result.citations();
        Ok((result.content, citations))
    }

    /// Perform a web-search query and preserve citation titles when supplied by
    /// the selected backend.
    pub async fn search_with_titles(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<(String, Vec<(String, String)>), xai_tool_runtime::ToolError> {
        let result = match &self.backend {
            WebSearchBackend::Responses(backend) => backend.search(query, allowed_domains).await?,
            WebSearchBackend::OpenAiCodex(backend) => {
                backend.search(query, allowed_domains).await?
            }
            WebSearchBackend::KimiCode(backend) => backend.search(query, allowed_domains).await?,
            WebSearchBackend::ZaiCodingPlan(backend) => {
                backend.search(query, allowed_domains).await?
            }
        };
        Ok((result.content, result.citation_pairs))
    }

    /// Execute the current structured standalone-search command schema.
    pub async fn run_commands(
        &self,
        input_text: &str,
        commands: serde_json::Value,
        allowed_domains: Option<Vec<String>>,
        context: Option<&CodexWebSearchContext>,
    ) -> Result<WebSearchCommandResult, xai_tool_runtime::ToolError> {
        let result = match &self.backend {
            WebSearchBackend::Responses(backend) => {
                backend.run_commands(&commands, allowed_domains).await?
            }
            WebSearchBackend::OpenAiCodex(backend) => {
                backend
                    .run_commands(input_text, commands, allowed_domains, context)
                    .await?
            }
            WebSearchBackend::KimiCode(backend) => {
                backend.run_commands(&commands, allowed_domains).await?
            }
            WebSearchBackend::ZaiCodingPlan(backend) => {
                backend.run_commands(&commands, allowed_domains).await?
            }
        };
        let citations = result.citations();
        Ok(WebSearchCommandResult {
            content: result.content,
            citations,
            references: result.references,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_config_is_rejected() {
        let error = WebSearchClient::new(&WebSearchConfig::Disabled, None)
            .err()
            .expect("disabled web search must not construct")
            .to_string();
        assert!(error.contains("disabled config"));
    }

    #[test]
    fn command_result_debug_does_not_expose_provider_projection_values() {
        let result = WebSearchCommandResult {
            content: "sentinel-provider-output".to_owned(),
            citations: vec!["https://sentinel.example/private".to_owned()],
            references: vec![WebSearchReference {
                ref_id: "sentinel-reference".to_owned(),
                title: "sentinel-title".to_owned(),
                url: "https://sentinel.example/reference".to_owned(),
            }],
        };
        let rendered = format!("{result:?}");
        for forbidden in [
            "sentinel-provider-output",
            "sentinel.example",
            "sentinel-reference",
            "sentinel-title",
        ] {
            assert!(!rendered.contains(forbidden));
        }
    }

    #[test]
    fn codex_constructor_requires_provider_authentication() {
        let config = WebSearchConfig::CodexSubscription {
            base_url: "https://chatgpt.com/backend-api/codex".to_owned(),
            model: "codex-search-model".to_owned(),
            session_id: "session-present".to_owned(),
            settings: Default::default(),
        };
        let error = WebSearchClient::new(&config, None)
            .err()
            .expect("Codex backend must require provider auth")
            .to_string();
        assert!(error.contains("provider authentication"));
    }

    struct StaticKeyProvider;

    impl crate::types::ApiKeyProvider for StaticKeyProvider {
        fn current_api_key(&self) -> Option<String> {
            Some("sentinel-static-key".to_owned())
        }
    }

    #[test]
    fn public_codex_constructor_rejects_generic_static_key_fallback() {
        let config = WebSearchConfig::CodexSubscription {
            base_url: "https://chatgpt.com/backend-api/codex".to_owned(),
            model: "codex-search-model".to_owned(),
            session_id: "session-present".to_owned(),
            settings: Default::default(),
        };
        let provider: SharedApiKeyProvider = std::sync::Arc::new(StaticKeyProvider);
        let error = WebSearchClient::new(&config, Some(provider))
            .err()
            .expect("static key adapters must not construct a Codex backend")
            .to_string();
        assert!(error.contains("authentication is unavailable"));
        assert!(!error.contains("sentinel-static-key"));
    }
}
