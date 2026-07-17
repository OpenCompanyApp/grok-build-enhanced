use std::fmt;

use blake3::Hasher;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use thiserror::Error;
use xai_grok_sampling_types::{CredentialBinding, CredentialSourceId, ProviderId};

use super::{CodexAuthError, CodexAuthManager, CodexCredentials};

const CHATGPT_ACCOUNT_ID: HeaderName = HeaderName::from_static("chatgpt-account-id");
const OPENAI_FEDRAMP: HeaderName = HeaderName::from_static("x-openai-fedramp");

/// A failure while obtaining or validating provider-scoped Codex request auth.
///
/// The variants deliberately carry no upstream text or credential metadata so
/// formatting an error cannot reveal token, account, FedRAMP, or record data.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum CodexAuthSnapshotError {
    #[error("OpenAI Codex authentication is unavailable")]
    Unavailable,
    #[error("OpenAI Codex authentication is temporarily unavailable")]
    Transient,
}

/// Exact provider credential binding plus the sensitive headers for one Codex
/// request generation.
///
/// The raw account and credential identities are used only while constructing
/// this value. Header values are marked sensitive immediately, the catalog
/// scope is one-way hashed, and `Debug` reveals no values, presence bits, or
/// binding fields. Cloning preserves the exact [`CredentialBinding`] so every
/// HTTP adapter can recover only the generation that actually signed a request.
#[derive(Clone)]
pub struct CodexAuthSnapshot {
    authorization: HeaderValue,
    account_id: HeaderValue,
    fedramp: Option<HeaderValue>,
    catalog_cache_identity: [u8; 32],
    credential_binding: CredentialBinding,
}

impl CodexAuthSnapshot {
    /// Construct a validated, fully redacted Codex request-auth snapshot.
    ///
    /// This signature intentionally remains compatible with the former catalog
    /// snapshot constructor; catalog callers may continue using the public
    /// compatibility alias exported from `crate::remote`.
    pub fn new(
        access_token: &str,
        chatgpt_account_id: &str,
        chatgpt_user_id: Option<&str>,
        credential_binding: CredentialBinding,
        fedramp: bool,
    ) -> Result<Self, CodexAuthSnapshotError> {
        let credential_id = credential_binding
            .record_id
            .as_deref()
            .filter(|record_id| !record_id.trim().is_empty());
        if access_token.trim().is_empty()
            || chatgpt_account_id.trim().is_empty()
            || credential_binding.provider != ProviderId::OpenAiCodex
            || credential_binding.source != CredentialSourceId::OpenAiCodexSubscription
            || credential_binding.generation == 0
            || credential_id.is_none()
        {
            return Err(CodexAuthSnapshotError::Unavailable);
        }

        let mut authorization = HeaderValue::from_str(&format!("Bearer {access_token}"))
            .map_err(|_| CodexAuthSnapshotError::Unavailable)?;
        authorization.set_sensitive(true);
        let mut account_id = HeaderValue::from_str(chatgpt_account_id)
            .map_err(|_| CodexAuthSnapshotError::Unavailable)?;
        account_id.set_sensitive(true);
        let fedramp = fedramp.then(|| {
            let mut value = HeaderValue::from_static("true");
            value.set_sensitive(true);
            value
        });

        let mut scope = Hasher::new_derive_key("grok-build-codex catalog credential scope v1");
        hash_len_prefixed(&mut scope, credential_id.unwrap_or_default().as_bytes());
        hash_len_prefixed(&mut scope, chatgpt_account_id.as_bytes());
        hash_len_prefixed(&mut scope, chatgpt_user_id.unwrap_or_default().as_bytes());
        scope.update(&[u8::from(fedramp.is_some())]);

        Ok(Self {
            authorization,
            account_id,
            fedramp,
            catalog_cache_identity: *scope.finalize().as_bytes(),
            credential_binding,
        })
    }

    pub(super) fn from_credentials(
        credentials: &CodexCredentials,
    ) -> Result<Self, CodexAuthSnapshotError> {
        Self::new(
            credentials.access_token(),
            credentials.account_id(),
            credentials.user_id(),
            credentials.credential_binding(),
            credentials.is_fedramp,
        )
    }

    /// Clone the complete approved Codex credential header set.
    ///
    /// The cloned values retain `HeaderValue::is_sensitive()`. No adapter may
    /// append provider-specific auth headers outside this common source.
    pub(crate) fn request_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::with_capacity(3);
        headers.insert(AUTHORIZATION, self.authorization.clone());
        headers.insert(CHATGPT_ACCOUNT_ID, self.account_id.clone());
        if let Some(fedramp) = &self.fedramp {
            headers.insert(OPENAI_FEDRAMP, fedramp.clone());
        }
        headers
    }

    pub fn credential_binding(&self) -> &CredentialBinding {
        &self.credential_binding
    }

    pub(crate) fn catalog_cache_identity(&self) -> &[u8; 32] {
        &self.catalog_cache_identity
    }
}

impl fmt::Debug for CodexAuthSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not expose even presence bits or binding generations here. In
        // particular, FedRAMP state and local credential identifiers are not
        // diagnostic data.
        formatter
            .debug_struct("CodexAuthSnapshot")
            .finish_non_exhaustive()
    }
}

impl CodexAuthManager {
    /// Resolve a fresh provider-owned snapshot for an outbound async request.
    pub async fn auth_snapshot(&self) -> Result<CodexAuthSnapshot, CodexAuthError> {
        let credentials = self.ensure_fresh().await?;
        CodexAuthSnapshot::from_credentials(&credentials).map_err(snapshot_auth_error)
    }

    /// Resolve a disk-verified snapshot without performing async refresh.
    /// Sampler clients call `ensure_fresh` before construction and use this
    /// path per request to fail closed on cross-process logout/account change.
    pub(crate) fn current_auth_snapshot(&self) -> Result<CodexAuthSnapshot, CodexAuthError> {
        let credentials = self.current_verified()?;
        CodexAuthSnapshot::from_credentials(&credentials).map_err(snapshot_auth_error)
    }
}

fn snapshot_auth_error(_error: CodexAuthSnapshotError) -> CodexAuthError {
    CodexAuthError::InvalidTokenResponse("credential could not form request authentication")
}

fn hash_len_prefixed(hasher: &mut Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(record_id: Option<&str>, generation: u64) -> CredentialBinding {
        let mut binding = CredentialBinding::openai_codex(record_id.map(ToOwned::to_owned));
        binding.generation = generation;
        binding
    }

    #[test]
    fn snapshot_preserves_exact_binding_and_marks_every_header_sensitive() {
        let expected = binding(Some("sentinel-record-id"), 7);
        let snapshot = CodexAuthSnapshot::new(
            "sentinel-access-token",
            "sentinel-account-id",
            Some("sentinel-user-id"),
            expected.clone(),
            true,
        )
        .unwrap();

        assert_eq!(snapshot.credential_binding(), &expected);
        let headers = snapshot.request_headers();
        for name in [AUTHORIZATION, CHATGPT_ACCOUNT_ID, OPENAI_FEDRAMP] {
            assert!(headers[&name].is_sensitive(), "{name} must be sensitive");
        }
    }

    #[test]
    fn snapshot_debug_and_header_debug_are_strictly_redacted() {
        let snapshot = CodexAuthSnapshot::new(
            "sentinel-access-token",
            "sentinel-account-id",
            Some("sentinel-user-id"),
            binding(Some("sentinel-record-id"), 7),
            true,
        )
        .unwrap();

        let rendered = format!("{snapshot:?} {:?}", snapshot.request_headers());
        for forbidden in [
            "sentinel-access-token",
            "sentinel-account-id",
            "sentinel-user-id",
            "sentinel-record-id",
            "Bearer",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "debug output was not redacted"
            );
        }
        assert!(rendered.starts_with("CodexAuthSnapshot { .. }"));
        assert_eq!(rendered.matches("Sensitive").count(), 3);
        assert!(!rendered.contains("true"));
    }

    #[test]
    fn snapshot_rejects_non_codex_or_inexact_bindings_and_invalid_headers() {
        let mut wrong_provider = binding(Some("record"), 1);
        wrong_provider.provider = ProviderId::Xai;
        let mut wrong_source = binding(Some("record"), 1);
        wrong_source.source = CredentialSourceId::XaiApiKey;

        for rejected in [
            CodexAuthSnapshot::new("", "account", None, binding(Some("record"), 1), false),
            CodexAuthSnapshot::new("token", "", None, binding(Some("record"), 1), false),
            CodexAuthSnapshot::new("token", "account", None, binding(None, 1), false),
            CodexAuthSnapshot::new("token", "account", None, binding(Some(" "), 1), false),
            CodexAuthSnapshot::new("token", "account", None, binding(Some("record"), 0), false),
            CodexAuthSnapshot::new("token", "account", None, wrong_provider, false),
            CodexAuthSnapshot::new("token", "account", None, wrong_source, false),
            CodexAuthSnapshot::new(
                "token\nvalue",
                "account",
                None,
                binding(Some("record"), 1),
                false,
            ),
            CodexAuthSnapshot::new(
                "token",
                "account\nvalue",
                None,
                binding(Some("record"), 1),
                false,
            ),
        ] {
            assert_eq!(rejected.unwrap_err(), CodexAuthSnapshotError::Unavailable);
        }
    }
}
