//! Provider and credential-source identity shared across the agent stack.
//!
//! These values describe *where* a request and its credentials belong. They
//! deliberately contain no credential material, account identifiers, or
//! provider-specific token state.

use serde::{Deserialize, Serialize};

/// Base URL used by the current public OpenAI Codex client for ChatGPT
/// subscription traffic.
///
/// This is an experimental client backend contract, not the stable OpenAI API.
pub const OPENAI_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";

/// Full Responses endpoint derived from [`OPENAI_CODEX_BASE_URL`].
pub const OPENAI_CODEX_RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex/responses";

/// Internal compatibility marker used by the public Codex client for models
/// whose authenticated catalog entry enables the Responses Lite contract.
pub const OPENAI_CODEX_RESPONSES_LITE_HEADER: &str = "x-openai-internal-codex-responses-lite";
/// Internal typed-compatibility side channel. The sampler stores a Codex
/// response's raw `max`/`ultra` effort here while deserializing through an
/// async-openai version whose enum stops at `xhigh`; conversation conversion
/// consumes it instead of recording a false `xhigh` provenance value.
pub const OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY: &str =
    "__grok_codex_reasoning_effort";

pub use xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION;

/// Provider that owns an outbound inference request.
///
/// `Xai` is the compatibility default so configs written before provider
/// identity was introduced retain their existing wire behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    #[default]
    Xai,
    #[serde(rename = "openai_codex")]
    OpenAiCodex,
    /// Existing custom-model behavior. The provider-specific request policy
    /// remains caller-defined unless a more specific provider is selected.
    Custom,
}

impl ProviderId {
    pub const fn is_openai_codex(self) -> bool {
        matches!(self, Self::OpenAiCodex)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Xai => "xai",
            Self::OpenAiCodex => "openai_codex",
            Self::Custom => "custom",
        }
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kind of credential source bound to a model/provider selection.
///
/// This is metadata only; it must never contain or derive from a token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialSourceId {
    #[default]
    Unspecified,
    XaiSession,
    XaiApiKey,
    #[serde(rename = "openai_codex_subscription")]
    OpenAiCodexSubscription,
    StaticApiKey,
    External,
}

/// Non-secret binding between a request provider and credential source.
///
/// `record_id` is an opaque local auth-store record key. It must not be an
/// access token, refresh token, account identifier, email address, or a value
/// derived from any of those fields.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialBinding {
    #[serde(default)]
    pub provider: ProviderId,
    #[serde(default)]
    pub source: CredentialSourceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    /// Monotonic credential generation used to pin a request/retry attempt to
    /// the snapshot it started with. This is not a token version or token
    /// derivative.
    #[serde(default)]
    pub generation: u64,
}

impl std::fmt::Debug for CredentialBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialBinding")
            .field("provider", &self.provider)
            .field("source", &self.source)
            .field("has_record_id", &self.record_id.is_some())
            .field("generation", &self.generation)
            .finish()
    }
}

impl CredentialBinding {
    pub fn openai_codex(record_id: Option<String>) -> Self {
        Self {
            provider: ProviderId::OpenAiCodex,
            source: CredentialSourceId::OpenAiCodexSubscription,
            record_id,
            generation: 0,
        }
    }

    /// Compare the stable credential record identity while allowing its
    /// monotonic generation to advance after a safe refresh rotation.
    pub fn same_record(&self, other: &Self) -> bool {
        self.provider == other.provider
            && self.source == other.source
            && self.record_id == other.record_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_provider_defaults_to_xai() {
        let provider: ProviderId = serde_json::from_str("null").unwrap_or_default();
        assert_eq!(provider, ProviderId::Xai);
        assert_eq!(CredentialBinding::default().provider, ProviderId::Xai);
    }

    #[test]
    fn codex_binding_has_stable_serde_identity() {
        let binding = CredentialBinding::openai_codex(Some("openai::codex".to_string()));
        let value = serde_json::to_value(&binding).unwrap();
        assert_eq!(value["provider"], "openai_codex");
        assert_eq!(value["source"], "openai_codex_subscription");
        assert_eq!(
            serde_json::from_value::<CredentialBinding>(value).unwrap(),
            binding
        );
        assert_eq!(ProviderId::OpenAiCodex.as_str(), "openai_codex");
    }

    #[test]
    fn old_binding_defaults_generation_to_zero_and_debug_redacts_record() {
        let binding: CredentialBinding = serde_json::from_value(serde_json::json!({
            "provider": "openai_codex",
            "source": "openai_codex_subscription",
            "record_id": "sensitive-record"
        }))
        .unwrap();
        assert_eq!(binding.generation, 0);
        let rendered = format!("{binding:?}");
        assert!(!rendered.contains("sensitive-record"));
        assert!(rendered.contains("has_record_id: true"));
    }

    #[test]
    fn codex_endpoint_matches_current_public_client_contract() {
        assert_eq!(
            format!("{OPENAI_CODEX_BASE_URL}/responses"),
            OPENAI_CODEX_RESPONSES_URL
        );
    }

    #[test]
    fn record_identity_survives_rotation_but_rejects_account_rebind() {
        let mut original = CredentialBinding::openai_codex(Some("record-a".to_owned()));
        original.generation = 1;
        let mut rotated = original.clone();
        rotated.generation = 2;
        assert!(original.same_record(&rotated));

        let switched = CredentialBinding::openai_codex(Some("record-b".to_owned()));
        assert!(!original.same_record(&switched));
    }
}
