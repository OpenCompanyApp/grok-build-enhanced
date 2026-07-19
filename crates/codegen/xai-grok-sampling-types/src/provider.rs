//! Provider and credential-source identity shared across the agent stack.
//!
//! These values describe *where* a request and its credentials belong. They
//! deliberately contain no credential material, account identifiers, or
//! provider-specific token state.

use serde::{Deserialize, Serialize};

/// Canonical public xAI inference API base URL.
pub const XAI_API_BASE_URL: &str = "https://api.x.ai/v1";
/// Canonical authenticated Grok CLI inference proxy base URL.
pub const XAI_CLI_CHAT_PROXY_BASE_URL: &str = "https://cli-chat-proxy.grok.com/v1";

/// Whether `candidate` is a credential-bearing xAI inference route.
///
/// This is intentionally narrower than a general first-party-host check: only
/// the two canonical HTTPS origins and their `/v1` route trees may receive an
/// xAI bearer. Userinfo, query, fragment, alternate ports, sibling xAI hosts,
/// and hostname-suffix lookalikes all fail closed.
pub fn is_trusted_xai_inference_url(candidate: &str) -> bool {
    [XAI_API_BASE_URL, XAI_CLI_CHAT_PROXY_BASE_URL]
        .into_iter()
        .any(|trusted| matches_trusted_inference_base(candidate, trusted))
}

fn matches_trusted_inference_base(candidate: &str, trusted: &str) -> bool {
    let (Ok(candidate), Ok(trusted)) =
        (reqwest::Url::parse(candidate), reqwest::Url::parse(trusted))
    else {
        return false;
    };
    if !candidate.username().is_empty()
        || candidate.password().is_some()
        || candidate.query().is_some()
        || candidate.fragment().is_some()
    {
        return false;
    }
    let trusted_path = trusted.path().trim_end_matches('/');
    let candidate_path = candidate.path().trim_end_matches('/');
    let path_matches = candidate_path == trusted_path
        || candidate_path
            .strip_prefix(trusted_path)
            .is_some_and(|suffix| suffix.starts_with('/'));
    candidate.scheme() == trusted.scheme()
        && candidate.host_str() == trusted.host_str()
        && candidate.port_or_known_default() == trusted.port_or_known_default()
        && path_matches
}

// Compatibility re-exports for callers that historically reached Codex
// protocol constants through `provider`.
pub use crate::openai_codex::{
    OPENAI_CODEX_BASE_URL, OPENAI_CODEX_COMPATIBILITY_VERSION,
    OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY, OPENAI_CODEX_FAST_SERVICE_TIER,
    OPENAI_CODEX_RESPONSES_LITE_HEADER, OPENAI_CODEX_RESPONSES_URL,
    OPENAI_CODEX_STANDARD_SERVICE_TIER,
};

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
    /// Kimi Code subscription provider. This identity remains Kimi Code even
    /// when an authenticated model catalog routes a model through the
    /// Anthropic-compatible Messages protocol.
    #[serde(rename = "kimi_code")]
    KimiCode,
    /// Global personal Z.AI GLM Coding Plan subscription. This must never
    /// share credentials or endpoints with Z.AI Open Platform or BigModel CN.
    #[serde(rename = "zai_coding_plan")]
    ZaiCodingPlan,
    /// Existing custom-model behavior. The provider-specific request policy
    /// remains caller-defined unless a more specific provider is selected.
    Custom,
}

impl ProviderId {
    pub const fn is_openai_codex(self) -> bool {
        matches!(self, Self::OpenAiCodex)
    }

    pub const fn is_kimi_code(self) -> bool {
        matches!(self, Self::KimiCode)
    }

    pub const fn is_zai_coding_plan(self) -> bool {
        matches!(self, Self::ZaiCodingPlan)
    }

    /// Whether requests cross a first-class subscription-provider boundary
    /// whose payloads and response diagnostics must remain redacted.
    pub const fn requires_redacted_provider_diagnostics(self) -> bool {
        matches!(
            self,
            Self::OpenAiCodex | Self::KimiCode | Self::ZaiCodingPlan
        )
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Xai => "xai",
            Self::OpenAiCodex => "openai_codex",
            Self::KimiCode => "kimi_code",
            Self::ZaiCodingPlan => "zai_coding_plan",
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
    #[serde(rename = "kimi_code_api_key")]
    KimiCodeApiKey,
    #[serde(rename = "zai_coding_plan_api_key")]
    ZaiCodingPlanApiKey,
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
        f.debug_struct("CredentialBinding").finish_non_exhaustive()
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

    pub fn kimi_code(record_id: Option<String>) -> Self {
        Self {
            provider: ProviderId::KimiCode,
            source: CredentialSourceId::KimiCodeApiKey,
            record_id,
            generation: 0,
        }
    }

    pub fn zai_coding_plan(record_id: Option<String>) -> Self {
        Self {
            provider: ProviderId::ZaiCodingPlan,
            source: CredentialSourceId::ZaiCodingPlanApiKey,
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

    /// Return whether this binding is a safe post-recovery successor to the
    /// exact binding rejected by a provider.
    ///
    /// Recovery may advance only the monotonic generation. Provider, source,
    /// and opaque local record identity must remain pinned, and merely
    /// reloading the same (or an older) generation is not a recovery.
    pub fn is_strict_successor_of(&self, rejected: &Self) -> bool {
        self.same_record(rejected) && self.generation > rejected.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_xai_inference_urls_are_exact_and_canonical() {
        assert!(is_trusted_xai_inference_url(XAI_API_BASE_URL));
        assert!(is_trusted_xai_inference_url(
            "https://api.x.ai/v1/responses"
        ));
        assert!(is_trusted_xai_inference_url(
            "https://cli-chat-proxy.grok.com/v1/chat/completions"
        ));

        for candidate in [
            "http://api.x.ai/v1",
            "https://api.x.ai:444/v1",
            "https://api.x.ai/v11",
            "https://other.x.ai/v1",
            "https://api.x.ai.evil.example/v1",
            "https://user@api.x.ai/v1",
            "https://api.x.ai/v1?redirect=1",
            "https://api.x.ai/v1#fragment",
            "https://custom.example/v1",
        ] {
            assert!(
                !is_trusted_xai_inference_url(candidate),
                "unexpected trusted xAI URL: {candidate}"
            );
        }
    }

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
        assert_eq!(format!("{binding:?}"), "CredentialBinding { .. }");
    }

    #[test]
    fn strict_successor_requires_same_record_and_higher_generation() {
        let mut rejected = CredentialBinding::openai_codex(Some("record-a".to_owned()));
        rejected.generation = 7;

        let mut successor = rejected.clone();
        successor.generation = 8;
        assert!(successor.is_strict_successor_of(&rejected));
        assert!(rejected.same_record(&successor));

        let mut same_generation = rejected.clone();
        same_generation.generation = rejected.generation;
        assert!(!same_generation.is_strict_successor_of(&rejected));

        let mut older = rejected.clone();
        older.generation = rejected.generation - 1;
        assert!(!older.is_strict_successor_of(&rejected));

        let mut switched_record = successor.clone();
        switched_record.record_id = Some("record-b".to_owned());
        assert!(!switched_record.is_strict_successor_of(&rejected));

        let mut switched_source = successor.clone();
        switched_source.source = CredentialSourceId::External;
        assert!(!switched_source.is_strict_successor_of(&rejected));

        let mut switched_provider = successor;
        switched_provider.provider = ProviderId::Custom;
        assert!(!switched_provider.is_strict_successor_of(&rejected));
    }

    #[test]
    fn legacy_zero_generation_can_advance_without_changing_serde_shape() {
        let rejected: CredentialBinding = serde_json::from_value(serde_json::json!({
            "provider": "openai_codex",
            "source": "openai_codex_subscription",
            "record_id": "legacy-record"
        }))
        .unwrap();
        let mut successor = rejected.clone();
        successor.generation = 1;

        assert!(successor.is_strict_successor_of(&rejected));
        assert_eq!(
            serde_json::to_value(&successor).unwrap()["generation"],
            serde_json::json!(1)
        );
    }
}
