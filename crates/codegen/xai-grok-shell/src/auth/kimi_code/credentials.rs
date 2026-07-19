use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::KimiCodeAuthError;

pub const KIMI_CODE_CREDENTIAL_SCHEMA_VERSION: u32 = 1;
pub(crate) const MAX_KIMI_CODE_API_KEY_BYTES: usize = 16 * 1024;

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct KimiCodeSecret(String);

impl KimiCodeSecret {
    pub fn new(value: impl Into<String>) -> Result<Self, KimiCodeAuthError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_KIMI_CODE_API_KEY_BYTES
            || value
                .as_bytes()
                .iter()
                .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
        {
            return Err(KimiCodeAuthError::InvalidCredential);
        }
        Ok(Self(value))
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for KimiCodeSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

impl<'de> Deserialize<'de> for KimiCodeSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

fn schema_version() -> u32 {
    KIMI_CODE_CREDENTIAL_SCHEMA_VERSION
}

fn provider() -> String {
    "kimi_code".to_owned()
}

fn credential_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Provider-owned Kimi Code API-key record.
///
/// This intentionally does not reuse xAI's `GrokAuth`: Kimi API keys have no
/// xAI session, refresh, user-profile, or account-routing semantics.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiCodeCredentials {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    #[serde(default = "provider")]
    pub provider: String,
    #[serde(default = "credential_id")]
    pub credential_id: String,
    pub api_key: KimiCodeSecret,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default = "default_revision")]
    pub revision: u64,
    #[serde(flatten)]
    pub additional_fields: BTreeMap<String, serde_json::Value>,
}

fn default_revision() -> u64 {
    1
}

impl fmt::Debug for KimiCodeCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KimiCodeCredentials")
            .finish_non_exhaustive()
    }
}

impl KimiCodeCredentials {
    pub fn new(api_key: impl Into<String>) -> Result<Self, KimiCodeAuthError> {
        let now = Utc::now();
        Ok(Self {
            schema_version: KIMI_CODE_CREDENTIAL_SCHEMA_VERSION,
            provider: provider(),
            credential_id: credential_id(),
            api_key: KimiCodeSecret::new(api_key)?,
            created_at: now,
            updated_at: now,
            revision: 1,
            additional_fields: BTreeMap::new(),
        })
    }

    pub(crate) fn validate_persisted(&self) -> Result<(), KimiCodeAuthError> {
        if self.schema_version != KIMI_CODE_CREDENTIAL_SCHEMA_VERSION
            || self.provider != "kimi_code"
            || uuid::Uuid::parse_str(&self.credential_id).is_err()
            || self.api_key.expose_secret().trim().is_empty()
            || self.revision == 0
        {
            return Err(KimiCodeAuthError::InvalidCredential);
        }
        Ok(())
    }

    pub(crate) fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    pub fn credential_binding(&self) -> xai_grok_sampling_types::CredentialBinding {
        xai_grok_sampling_types::CredentialBinding {
            provider: xai_grok_sampling_types::ProviderId::KimiCode,
            source: xai_grok_sampling_types::CredentialSourceId::KimiCodeApiKey,
            record_id: Some(self.credential_id.clone()),
            generation: self.revision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_rejects_ascii_whitespace_that_cannot_be_part_of_a_bearer_token() {
        let result = KimiCodeSecret::new("sentinel key");

        assert!(matches!(result, Err(KimiCodeAuthError::InvalidCredential)));
    }

    #[test]
    fn credential_rejects_values_above_the_input_limit() {
        let result = KimiCodeSecret::new("k".repeat(MAX_KIMI_CODE_API_KEY_BYTES + 1));

        assert!(matches!(result, Err(KimiCodeAuthError::InvalidCredential)));
    }
}
