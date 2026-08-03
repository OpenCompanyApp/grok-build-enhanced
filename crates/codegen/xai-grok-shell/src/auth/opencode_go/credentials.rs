use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::OpenCodeGoAuthError;

pub const OPENCODE_GO_CREDENTIAL_SCHEMA_VERSION: u32 = 1;
pub(crate) const MAX_OPENCODE_GO_API_KEY_BYTES: usize = 16 * 1024;

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct OpenCodeGoSecret(String);

impl OpenCodeGoSecret {
    pub fn new(value: impl Into<String>) -> Result<Self, OpenCodeGoAuthError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_OPENCODE_GO_API_KEY_BYTES
            || value
                .as_bytes()
                .iter()
                .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
        {
            return Err(OpenCodeGoAuthError::InvalidCredential);
        }
        Ok(Self(value))
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for OpenCodeGoSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl<'de> Deserialize<'de> for OpenCodeGoSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

fn schema_version() -> u32 {
    OPENCODE_GO_CREDENTIAL_SCHEMA_VERSION
}

fn provider() -> String {
    "open_code_go".to_owned()
}

fn credential_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_revision() -> u64 {
    1
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenCodeGoCredentials {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    #[serde(default = "provider")]
    pub provider: String,
    #[serde(default = "credential_id")]
    pub credential_id: String,
    pub api_key: OpenCodeGoSecret,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default = "default_revision")]
    pub revision: u64,
    #[serde(flatten)]
    pub additional_fields: BTreeMap<String, serde_json::Value>,
}

impl fmt::Debug for OpenCodeGoCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenCodeGoCredentials")
            .finish_non_exhaustive()
    }
}

impl OpenCodeGoCredentials {
    pub fn new(api_key: impl Into<String>) -> Result<Self, OpenCodeGoAuthError> {
        let now = Utc::now();
        Ok(Self {
            schema_version: OPENCODE_GO_CREDENTIAL_SCHEMA_VERSION,
            provider: provider(),
            credential_id: credential_id(),
            api_key: OpenCodeGoSecret::new(api_key)?,
            created_at: now,
            updated_at: now,
            revision: 1,
            additional_fields: BTreeMap::new(),
        })
    }

    pub(crate) fn validate_persisted(&self) -> Result<(), OpenCodeGoAuthError> {
        if self.schema_version != OPENCODE_GO_CREDENTIAL_SCHEMA_VERSION
            || self.provider != "open_code_go"
            || uuid::Uuid::parse_str(&self.credential_id).is_err()
            || self.revision == 0
        {
            return Err(OpenCodeGoAuthError::InvalidCredential);
        }
        OpenCodeGoSecret::new(self.api_key.expose_secret().to_owned())?;
        Ok(())
    }

    pub(crate) fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    pub fn credential_binding(&self) -> xai_grok_sampling_types::CredentialBinding {
        xai_grok_sampling_types::CredentialBinding {
            provider: xai_grok_sampling_types::ProviderId::OpenCodeGo,
            source: xai_grok_sampling_types::CredentialSourceId::OpenCodeGoApiKey,
            record_id: Some(self.credential_id.clone()),
            generation: self.revision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_is_bounded_and_header_safe() {
        assert!(OpenCodeGoSecret::new("sentinel key").is_err());
        assert!(OpenCodeGoSecret::new("k".repeat(MAX_OPENCODE_GO_API_KEY_BYTES + 1)).is_err());
        assert_eq!(
            format!("{:?}", OpenCodeGoSecret::new("sentinel").unwrap()),
            "<redacted>"
        );
    }
}
