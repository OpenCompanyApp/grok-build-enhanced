use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{KIMI_CODE_AUTH_SCOPE, KimiCodeAuthError, KimiCodeCredentials};
use crate::auth::manager::lock::try_lock_auth_file_async;
use crate::auth::model::AuthStore;
use crate::auth::storage::{read_auth_json, read_auth_json_or_empty, write_auth_json};

const AUTH_FILE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Provider-scoped persistence for Kimi Code API keys in Grok's existing
/// `auth.json`. Every write preserves xAI, Codex, and unknown future scopes.
#[derive(Clone, Debug)]
pub struct KimiCodeCredentialStore {
    auth_path: PathBuf,
}

impl KimiCodeCredentialStore {
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

    pub fn load(&self) -> Result<Option<KimiCodeCredentials>, KimiCodeAuthError> {
        match read_auth_json(&self.auth_path) {
            Ok(store) => {
                let credentials = store.get_kimi_code().cloned();
                if credentials.is_none() && store.contains_key(KIMI_CODE_AUTH_SCOPE) {
                    return Err(KimiCodeAuthError::Storage(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "kimi::code credential record is invalid",
                    )));
                }
                Ok(credentials)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(KimiCodeAuthError::Storage(error)),
        }
    }

    pub async fn save(&self, credentials: KimiCodeCredentials) -> Result<(), KimiCodeAuthError> {
        credentials.validate_persisted()?;
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(KimiCodeAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(KimiCodeAuthError::LockTimeout);
        }
        let mut store = read_auth_json_or_empty(&self.auth_path)?;
        store.insert_kimi_code(credentials);
        write_auth_json(&self.auth_path, &store)?;
        Ok(())
    }

    pub async fn remove(&self) -> Result<bool, KimiCodeAuthError> {
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(KimiCodeAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(KimiCodeAuthError::LockTimeout);
        }
        let mut store = match read_auth_json(&self.auth_path) {
            Ok(store) => store,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AuthStore::new(),
            Err(error) => return Err(KimiCodeAuthError::Storage(error)),
        };
        let removed = store.remove_kimi_code_record();
        if !removed {
            return Ok(false);
        }
        if store.is_empty() {
            match std::fs::remove_file(&self.auth_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(KimiCodeAuthError::Storage(error)),
            }
        } else {
            write_auth_json(&self.auth_path, &store)?;
        }
        Ok(removed)
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

        let store = KimiCodeCredentialStore::from_auth_path(path.clone());
        store
            .save(KimiCodeCredentials::new("sentinel-kimi-key").unwrap())
            .await
            .unwrap();
        assert!(store.load().unwrap().is_some());
        assert!(store.remove().await.unwrap());

        let remaining = crate::auth::storage::read_auth_json(&path).unwrap();
        assert!(remaining.get("xai::api_key").is_some());
        assert!(!remaining.contains_key(KIMI_CODE_AUTH_SCOPE));
    }

    #[tokio::test]
    async fn logout_repairs_a_malformed_kimi_scope_without_changing_xai_auth() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("auth.json");
        let payload = serde_json::json!({
            "xai::api_key": GrokAuth::test_default(),
            (KIMI_CODE_AUTH_SCOPE): {"provider": "kimi_code", "api_key": "invalid-record"}
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        let store = KimiCodeCredentialStore::from_auth_path(path.clone());
        assert!(store.load().is_err());

        let removed = store.remove().await.unwrap();

        assert!(removed);
        let remaining = crate::auth::storage::read_auth_json(&path).unwrap();
        assert!(remaining.get("xai::api_key").is_some());
        assert!(!remaining.contains_key(KIMI_CODE_AUTH_SCOPE));
    }
}
