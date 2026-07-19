use indexmap::IndexMap;

/// Reserved dispatch-only field used by the shell to pass a trusted snapshot
/// of the current Codex turn to the standalone search extension. It is never
/// advertised in the model-facing tool schema.
pub const CODEX_WEB_SEARCH_CONTEXT_FIELD: &str = "_grok_codex_context";

/// A visible conversation message supplied to the standalone Codex search
/// extension. Only user and assistant text is representable: images, tool
/// results, reasoning payloads, and provider-owned response state have no
/// representation in this type.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

impl std::fmt::Debug for CodexWebSearchMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexWebSearchMessage")
            .finish_non_exhaustive()
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
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexWebSearchTurnMetadata {
    pub session_id: String,
    pub thread_id: String,
    pub turn_id: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

impl std::fmt::Debug for CodexWebSearchTurnMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexWebSearchTurnMetadata")
            .finish_non_exhaustive()
    }
}

/// Trusted per-call context attached by the shell after model arguments have
/// been parsed and validated. Keeping this snapshot on the call avoids shared
/// mutable state and preserves correctness for parallel web-search calls.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexWebSearchContext {
    pub input: Vec<CodexWebSearchMessage>,
    pub turn_metadata: CodexWebSearchTurnMetadata,
    /// Shell-computed output ceiling. This field is hidden from the tool schema
    /// and replaces any model-supplied reserved context before dispatch.
    pub max_output_tokens: u64,
}

impl std::fmt::Debug for CodexWebSearchContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexWebSearchContext")
            .finish_non_exhaustive()
    }
}

/// Provider-side external access mode for ChatGPT Codex standalone search.
/// `Disabled` is a registration policy and is never serialized on the wire.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CodexWebSearchMode {
    Disabled,
    #[default]
    Cached,
    Indexed,
    Live,
}

impl CodexWebSearchMode {
    pub fn is_enabled(self) -> bool {
        self != Self::Disabled
    }
}

impl std::fmt::Display for CodexWebSearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Disabled => "disabled",
            Self::Cached => "cached",
            Self::Indexed => "indexed",
            Self::Live => "live",
        })
    }
}

impl std::str::FromStr for CodexWebSearchMode {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disabled" => Ok(Self::Disabled),
            "cached" => Ok(Self::Cached),
            "indexed" => Ok(Self::Indexed),
            "live" => Ok(Self::Live),
            _ => Err("expected disabled, cached, indexed, or live"),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum CodexWebSearchContextSize {
    Low,
    Medium,
    High,
}

/// Explicit approximate location supplied by configuration. No field is
/// inferred from credentials, IP addresses, account metadata, or the host.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
pub struct CodexWebSearchLocation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
pub struct CodexWebSearchImageSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<bool>,
}

/// Fully resolved, provider-scoped standalone-search settings.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
pub struct CodexWebSearchSettings {
    pub mode: CodexWebSearchMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<CodexWebSearchContextSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_domains: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: Option<CodexWebSearchLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_settings: Option<CodexWebSearchImageSettings>,
}

/// Configuration for the web search tool.
///
/// Use `Disabled` when no API key is available or web search should be turned off.
/// Use `Enabled { … }` to provide credentials and endpoint configuration.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
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
        #[serde(default)]
        settings: CodexWebSearchSettings,
    },
    /// Kimi Code hosted search through the provider-owned `/search` service.
    /// Authentication is resolved per request from the Kimi-scoped provider;
    /// no API key is serialized into this configuration.
    KimiCode { base_url: String },
}

impl std::fmt::Debug for WebSearchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Configuration can contain credentials, configured credential-header
        // names, endpoint userinfo, model routing, and turn identity.
        f.debug_struct("WebSearchConfig").finish_non_exhaustive()
    }
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

    pub fn is_kimi_code(&self) -> bool {
        matches!(self, Self::KimiCode { .. })
    }

    /// Return a copy safe for returning to clients.
    ///
    /// Static keys, configured header values, and Codex turn/session identity
    /// are replaced or removed while non-sensitive endpoint/model shape is
    /// preserved.
    pub fn redacted(&self) -> Self {
        match self {
            Self::Disabled => Self::Disabled,
            Self::Enabled {
                base_url,
                model,
                extra_headers,
                ..
            } => {
                let _ = extra_headers;
                Self::Enabled {
                    api_key: "***REDACTED***".to_string(),
                    base_url: sanitize_redacted_base_url(base_url),
                    model: model.clone(),
                    // Header names can identify credential schemes and must not
                    // be returned merely because their values were removed.
                    extra_headers: IndexMap::new(),
                    alpha_test_key: None,
                }
            }
            Self::CodexSubscription {
                base_url,
                model,
                session_id: _,
                settings,
            } => Self::CodexSubscription {
                base_url: sanitize_redacted_base_url(base_url),
                model: model.clone(),
                session_id: "***REDACTED***".to_owned(),
                settings: settings.clone(),
            },
            Self::KimiCode { base_url } => Self::KimiCode {
                base_url: sanitize_redacted_base_url(base_url),
            },
        }
    }
}

fn sanitize_redacted_base_url(base_url: &str) -> String {
    let Ok(mut url) = url::Url::parse(base_url) else {
        return "***REDACTED***".to_owned();
    };
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return "***REDACTED***".to_owned();
    }
    if url.set_username("").is_err() || url.set_password(None).is_err() {
        return "***REDACTED***".to_owned();
    }
    // Query strings and fragments frequently carry proxy or deployment
    // secrets. Neither is needed to identify the configured endpoint shape.
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
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
            settings: CodexWebSearchSettings::default(),
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
            base_url: "https://sentinel-user:sentinel-pass@api.x.ai/v1?token=sentinel-query#sentinel-fragment".to_string(),
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
                assert!(extra_headers.is_empty());
                assert!(alpha_test_key.is_none());
            }
            _ => panic!("Expected Enabled variant"),
        }
    }

    #[test]
    fn codex_config_redaction_drops_session_identity() {
        let config = WebSearchConfig::CodexSubscription {
            base_url: "https://chatgpt.com/backend-api/codex".to_owned(),
            model: "codex-search-model".to_owned(),
            session_id: "sentinel-session".to_owned(),
            settings: CodexWebSearchSettings::default(),
        };
        let redacted = config.redacted();
        assert!(matches!(
            redacted,
            WebSearchConfig::CodexSubscription { session_id, .. }
                if session_id == "***REDACTED***"
        ));
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

    #[test]
    fn debug_redacts_credentials_headers_and_codex_turn_identity() {
        let mut headers = IndexMap::new();
        headers.insert(
            "x-provider-credential".to_owned(),
            "sentinel-header-value".to_owned(),
        );
        let responses = WebSearchConfig::Enabled {
            api_key: "sentinel-api-key".to_owned(),
            base_url: "https://sentinel-user:sentinel-pass@example.com".to_owned(),
            model: "test-model".to_owned(),
            extra_headers: headers,
            alpha_test_key: Some("sentinel-alpha-key".to_owned()),
        };
        let context = CodexWebSearchContext {
            input: vec![CodexWebSearchMessage::user("sentinel-visible-turn-text")],
            turn_metadata: CodexWebSearchTurnMetadata {
                session_id: "sentinel-session".to_owned(),
                thread_id: "sentinel-thread".to_owned(),
                turn_id: "sentinel-turn".to_owned(),
                model: "sentinel-turn-model".to_owned(),
                reasoning_effort: Some("sentinel-effort".to_owned()),
            },
            max_output_tokens: 8_192,
        };

        assert_eq!(format!("{responses:?}"), "WebSearchConfig { .. }");
        assert_eq!(format!("{context:?}"), "CodexWebSearchContext { .. }");
        assert_eq!(
            format!("{:?}", context.input[0]),
            "CodexWebSearchMessage { .. }"
        );
        assert_eq!(
            format!("{:?}", context.turn_metadata),
            "CodexWebSearchTurnMetadata { .. }"
        );
    }
}
