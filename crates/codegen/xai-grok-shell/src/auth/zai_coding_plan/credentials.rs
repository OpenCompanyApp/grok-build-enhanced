use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ZaiCodingPlanAuthError;

pub const ZAI_CODING_PLAN_CREDENTIAL_SCHEMA_VERSION: u32 = 1;
pub(crate) const MAX_ZAI_CODING_PLAN_API_KEY_BYTES: usize = 16 * 1024;

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ZaiCodingPlanSecret(String);

impl ZaiCodingPlanSecret {
    pub fn new(value: impl Into<String>) -> Result<Self, ZaiCodingPlanAuthError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_ZAI_CODING_PLAN_API_KEY_BYTES
            || value
                .as_bytes()
                .iter()
                .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
        {
            return Err(ZaiCodingPlanAuthError::InvalidCredential);
        }
        Ok(Self(value))
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ZaiCodingPlanSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

impl<'de> Deserialize<'de> for ZaiCodingPlanSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

fn schema_version() -> u32 {
    ZAI_CODING_PLAN_CREDENTIAL_SCHEMA_VERSION
}

fn provider() -> String {
    "zai_coding_plan".to_owned()
}

fn credential_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Provider-owned global personal Z.AI Coding Plan API-key record.
///
/// It intentionally carries no Open Platform, BigModel CN, xAI, Kimi, or
/// Codex account semantics.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZaiCodingPlanCredentials {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    #[serde(default = "provider")]
    pub provider: String,
    #[serde(default = "credential_id")]
    pub credential_id: String,
    pub api_key: ZaiCodingPlanSecret,
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

impl fmt::Debug for ZaiCodingPlanCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZaiCodingPlanCredentials")
            .finish_non_exhaustive()
    }
}

impl ZaiCodingPlanCredentials {
    pub fn new(api_key: impl Into<String>) -> Result<Self, ZaiCodingPlanAuthError> {
        let now = Utc::now();
        Ok(Self {
            schema_version: ZAI_CODING_PLAN_CREDENTIAL_SCHEMA_VERSION,
            provider: provider(),
            credential_id: credential_id(),
            api_key: ZaiCodingPlanSecret::new(api_key)?,
            created_at: now,
            updated_at: now,
            revision: 1,
            additional_fields: BTreeMap::new(),
        })
    }

    pub(crate) fn validate_persisted(&self) -> Result<(), ZaiCodingPlanAuthError> {
        if self.schema_version != ZAI_CODING_PLAN_CREDENTIAL_SCHEMA_VERSION
            || self.provider != "zai_coding_plan"
            || uuid::Uuid::parse_str(&self.credential_id).is_err()
            || self.api_key.expose_secret().trim().is_empty()
            || self.revision == 0
        {
            return Err(ZaiCodingPlanAuthError::InvalidCredential);
        }
        Ok(())
    }

    pub(crate) fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    pub fn credential_binding(&self) -> xai_grok_sampling_types::CredentialBinding {
        xai_grok_sampling_types::CredentialBinding {
            provider: xai_grok_sampling_types::ProviderId::ZaiCodingPlan,
            source: xai_grok_sampling_types::CredentialSourceId::ZaiCodingPlanApiKey,
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
        let result = ZaiCodingPlanSecret::new("sentinel key");
        assert!(matches!(
            result,
            Err(ZaiCodingPlanAuthError::InvalidCredential)
        ));
    }

    #[test]
    fn credential_debug_is_redacted() {
        let credential = ZaiCodingPlanCredentials::new("sentinel-zai-key").unwrap();
        assert!(!format!("{credential:?}").contains("sentinel"));
        assert_eq!(format!("{:?}", credential.api_key), "<redacted>");
    }
}
