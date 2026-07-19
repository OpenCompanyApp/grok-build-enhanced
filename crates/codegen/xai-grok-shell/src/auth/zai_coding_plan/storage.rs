use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{ZAI_CODING_PLAN_AUTH_SCOPE, ZaiCodingPlanAuthError, ZaiCodingPlanCredentials};
use crate::auth::manager::lock::try_lock_auth_file_async;
use crate::auth::model::AuthStore;
use crate::auth::storage::{read_auth_json, read_auth_json_or_empty, write_auth_json};

const AUTH_FILE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Provider-scoped persistence for a global personal Z.AI Coding Plan API key.
/// Every write preserves xAI, Codex, Kimi, and unknown future scopes.
#[derive(Clone, Debug)]
pub struct ZaiCodingPlanCredentialStore {
    auth_path: PathBuf,
}

impl ZaiCodingPlanCredentialStore {
    pub fn new(grok_home: &Path) -> Self {
        Self {
            auth_path: crate::auth::resolved_auth_path(grok_home),
        }
    }

    pub fn from_auth_path(auth_path: PathBuf) -> Self {
        Self { auth_path }
    }

    pub fn auth_path(&self) -> &Path {
        &self.auth_path
    }

    pub fn load(&self) -> Result<Option<ZaiCodingPlanCredentials>, ZaiCodingPlanAuthError> {
        match read_auth_json(&self.auth_path) {
            Ok(store) => {
                let credentials = store.get_zai_coding_plan().cloned();
                if credentials.is_none() && store.contains_key(ZAI_CODING_PLAN_AUTH_SCOPE) {
                    return Err(ZaiCodingPlanAuthError::Storage(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "zai::coding-plan::global credential record is invalid",
                    )));
                }
                Ok(credentials)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(ZaiCodingPlanAuthError::Storage(error)),
        }
    }

    pub async fn save(
        &self,
        credentials: ZaiCodingPlanCredentials,
    ) -> Result<(), ZaiCodingPlanAuthError> {
        credentials.validate_persisted()?;
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(ZaiCodingPlanAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(ZaiCodingPlanAuthError::LockTimeout);
        }
        let mut store = read_auth_json_or_empty(&self.auth_path)?;
        store.insert_zai_coding_plan(credentials);
        write_auth_json(&self.auth_path, &store)?;
        Ok(())
    }

    pub async fn remove(&self) -> Result<bool, ZaiCodingPlanAuthError> {
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(ZaiCodingPlanAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(ZaiCodingPlanAuthError::LockTimeout);
        }
        let mut store = match read_auth_json(&self.auth_path) {
            Ok(store) => store,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AuthStore::new(),
            Err(error) => return Err(ZaiCodingPlanAuthError::Storage(error)),
        };
        let removed = store.remove_zai_coding_plan_record();
        if !removed {
            return Ok(false);
        }
        if store.is_empty() {
            match std::fs::remove_file(&self.auth_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(ZaiCodingPlanAuthError::Storage(error)),
            }
        } else {
            write_auth_json(&self.auth_path, &store)?;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::GrokAuth;

    #[tokio::test]
    async fn save_and_remove_preserve_other_provider_scopes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("auth.json");
        let mut auth = AuthStore::new();
        auth.insert("xai::api_key".to_owned(), GrokAuth::test_default());
        crate::auth::storage::write_auth_json(&path, &auth).unwrap();

        let store = ZaiCodingPlanCredentialStore::from_auth_path(path.clone());
        store
            .save(ZaiCodingPlanCredentials::new("sentinel-zai-key").unwrap())
            .await
            .unwrap();
        assert!(store.load().unwrap().is_some());
        assert!(store.remove().await.unwrap());

        let remaining = crate::auth::storage::read_auth_json(&path).unwrap();
        assert!(remaining.get("xai::api_key").is_some());
        assert!(!remaining.contains_key(ZAI_CODING_PLAN_AUTH_SCOPE));
    }

    #[tokio::test]
    async fn logout_repairs_malformed_scope_without_changing_xai_auth() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("auth.json");
        let payload = serde_json::json!({
            "xai::api_key": GrokAuth::test_default(),
            (ZAI_CODING_PLAN_AUTH_SCOPE): {
                "provider": "zai_coding_plan",
                "api_key": "invalid-record"
            }
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        let store = ZaiCodingPlanCredentialStore::from_auth_path(path.clone());
        assert!(store.load().is_err());

        assert!(store.remove().await.unwrap());
        let remaining = crate::auth::storage::read_auth_json(&path).unwrap();
        assert!(remaining.get("xai::api_key").is_some());
        assert!(!remaining.contains_key(ZAI_CODING_PLAN_AUTH_SCOPE));
    }
}
