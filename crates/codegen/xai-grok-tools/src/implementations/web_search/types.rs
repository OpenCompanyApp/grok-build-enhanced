use indexmap::IndexMap;

/// Reserved dispatch-only field used by the shell to pass a trusted snapshot
/// of the current Codex turn to the standalone search extension. It is never
/// advertised in the model-facing tool schema.
pub const CODEX_WEB_SEARCH_CONTEXT_FIELD: &str = "_grok_codex_context";

/// A visible conversation message supplied to the standalone Codex search
/// extension. Only user and assistant text is representable: images, tool
/// results, reasoning payloads, and provider-owned response state have no
/// representation in this type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexWebSearchMessage {
    pub role: CodexWebSearchMessageRole,
    pub text: String,
}

impl CodexWebSearchMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: CodexWebSearchMessageRole::User,
            text: text.into(),
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: CodexWebSearchMessageRole::Assistant,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodexWebSearchMessageRole {
    User,
    Assistant,
}

/// Grok-owned correlation metadata projected into the current public Codex
/// `x-codex-turn-metadata` shape. This deliberately contains no installation
/// identifier, credential/account data, arbitrary client metadata, or opaque
/// `x-codex-turn-state` returned by the provider.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexWebSearchTurnMetadata {
    pub session_id: String,
    pub thread_id: String,
    pub turn_id: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

/// Trusted per-call context attached by the shell after model arguments have
/// been parsed and validated. Keeping this snapshot on the call avoids shared
/// mutable state and preserves correctness for parallel web-search calls.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexWebSearchContext {
    pub input: Vec<CodexWebSearchMessage>,
    pub turn_metadata: CodexWebSearchTurnMetadata,
}

/// Configuration for the web search tool.
///
/// Use `Disabled` when no API key is available or web search should be turned off.
/// Use `Enabled { … }` to provide credentials and endpoint configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WebSearchConfig {
    #[default]
    Disabled,
    Enabled {
        api_key: String,
        base_url: String,
        model: String,
        #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
        extra_headers: IndexMap<String, String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alpha_test_key: Option<String>,
    },
    /// ChatGPT Codex subscription search through the provider-owned
    /// `alpha/search` endpoint. Authentication is resolved dynamically from
    /// the scoped [`crate::types::SharedApiKeyProvider`] for every request;
    /// no bearer or account identifier is stored in this configuration.
    CodexSubscription {
        base_url: String,
        model: String,
        session_id: String,
    },
}

impl WebSearchConfig {
    /// Returns `true` when the config is the `Enabled` variant.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Whether the main Responses request may advertise the provider-hosted
    /// `web_search` tool. Responses Lite subscription sessions instead use a
    /// client-executed function backed by `alpha/search`, matching the current
    /// public Codex client.
    pub fn allows_hosted_responses_tool(&self) -> bool {
        matches!(self, Self::Enabled { .. })
    }

    pub fn is_codex_subscription(&self) -> bool {
        matches!(self, Self::CodexSubscription { .. })
    }

    /// Return a copy safe for returning to clients.
    ///
    /// The `api_key` is replaced with `"***REDACTED***"` and the optional
    /// extra access key field is stripped.
    pub fn redacted(&self) -> Self {
        match self {
            Self::Disabled => Self::Disabled,
            Self::Enabled {
                base_url,
                model,
                extra_headers,
                ..
            } => Self::Enabled {
                api_key: "***REDACTED***".to_string(),
                base_url: base_url.clone(),
                model: model.clone(),
                extra_headers: extra_headers.clone(),
                alpha_test_key: None,
            },
            Self::CodexSubscription {
                base_url,
                model,
                session_id,
            } => Self::CodexSubscription {
                base_url: base_url.clone(),
                model: model.clone(),
                session_id: session_id.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_is_disabled() {
        let config = WebSearchConfig::default();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_config_enabled() {
        let config = WebSearchConfig::Enabled {
            api_key: "test-key".to_string(),
            base_url: "https://api.x.ai/v1".to_string(),
            model: "test-web-search-model".to_string(),
            extra_headers: IndexMap::new(),
            alpha_test_key: None,
        };
        assert!(config.is_enabled());
        assert!(config.allows_hosted_responses_tool());
        assert!(!config.is_codex_subscription());
    }

    #[test]
    fn codex_subscription_is_enabled_without_static_credentials() {
        let config = WebSearchConfig::CodexSubscription {
            base_url: "https://chatgpt.com/backend-api/codex".to_string(),
            model: "gpt-5.6-luna".to_string(),
            session_id: "session-public-id".to_string(),
        };

        assert!(config.is_enabled());
        assert!(!config.allows_hosted_responses_tool());
        assert!(config.is_codex_subscription());
    }

    #[test]
    fn test_config_redacted() {
        let mut headers = IndexMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());
        let config = WebSearchConfig::Enabled {
            api_key: "secret-key-12345".to_string(),
            base_url: "https://api.x.ai/v1".to_string(),
            model: "test-web-search-model".to_string(),
            extra_headers: headers,
            alpha_test_key: Some("alpha-secret".to_string()),
        };
        let redacted = config.redacted();
        match redacted {
            WebSearchConfig::Enabled {
                api_key,
                base_url,
                model,
                extra_headers,
                alpha_test_key,
            } => {
                assert_eq!(api_key, "***REDACTED***");
                assert_eq!(base_url, "https://api.x.ai/v1");
                assert_eq!(model, "test-web-search-model");
                assert_eq!(extra_headers.get("X-Custom").unwrap(), "value");
                assert!(alpha_test_key.is_none());
            }
            _ => panic!("Expected Enabled variant"),
        }
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = WebSearchConfig::Enabled {
            api_key: "key".to_string(),
            base_url: "https://api.x.ai/v1".to_string(),
            model: "test-web-search-model".to_string(),
            extra_headers: IndexMap::new(),
            alpha_test_key: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: WebSearchConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_enabled());
    }

    #[test]
    fn test_config_deserialize_from_set_options_payload() {
        let json = r#"{
            "status": "enabled",
            "api_key": "xai-abc123",
            "base_url": "https://api.x.ai/v1",
            "model": "test-web-search-model"
        }"#;
        let config: WebSearchConfig = serde_json::from_str(json).unwrap();
        assert!(config.is_enabled());
    }
}
