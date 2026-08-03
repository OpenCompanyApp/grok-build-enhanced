use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{OPENCODE_GO_AUTH_SCOPE, OpenCodeGoAuthError, OpenCodeGoCredentials};
use crate::auth::manager::lock::try_lock_auth_file_async;
use crate::auth::model::AuthStore;
use crate::auth::storage::{read_auth_json, read_auth_json_or_empty, write_auth_json};

const AUTH_FILE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub struct OpenCodeGoCredentialStore {
    auth_path: PathBuf,
}

impl OpenCodeGoCredentialStore {
    pub fn new(grok_home: &Path) -> Self {
        Self {
            auth_path: crate::auth::resolved_auth_path(grok_home),
        }
    }

    #[cfg(test)]
    pub fn from_auth_path(auth_path: PathBuf) -> Self {
        Self { auth_path }
    }

    pub fn load(&self) -> Result<Option<OpenCodeGoCredentials>, OpenCodeGoAuthError> {
        match read_auth_json(&self.auth_path) {
            Ok(store) => {
                let credentials = store.get_open_code_go().cloned();
                if credentials.is_none() && store.contains_key(OPENCODE_GO_AUTH_SCOPE) {
                    return Err(OpenCodeGoAuthError::Storage(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "opencode::go credential record is invalid",
                    )));
                }
                Ok(credentials)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(OpenCodeGoAuthError::Storage(error)),
        }
    }

    pub async fn save(
        &self,
        credentials: OpenCodeGoCredentials,
    ) -> Result<(), OpenCodeGoAuthError> {
        credentials.validate_persisted()?;
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(OpenCodeGoAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(OpenCodeGoAuthError::LockTimeout);
        }
        let mut store = read_auth_json_or_empty(&self.auth_path)?;
        store.insert_open_code_go(credentials);
        write_auth_json(&self.auth_path, &store)?;
        Ok(())
    }

    pub async fn remove(&self) -> Result<bool, OpenCodeGoAuthError> {
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(OpenCodeGoAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(OpenCodeGoAuthError::LockTimeout);
        }
        let mut store = match read_auth_json(&self.auth_path) {
            Ok(store) => store,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AuthStore::new(),
            Err(error) => return Err(OpenCodeGoAuthError::Storage(error)),
        };
        let removed = store.remove_open_code_go_record();
        if !removed {
            return Ok(false);
        }
        if store.is_empty() {
            match std::fs::remove_file(&self.auth_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(OpenCodeGoAuthError::Storage(error)),
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
    async fn save_and_logout_preserve_every_other_provider_scope() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("auth.json");
        let mut auth = AuthStore::new();
        auth.insert("xai::api_key".to_owned(), GrokAuth::test_default());
        crate::auth::storage::write_auth_json(&path, &auth).unwrap();

        let store = OpenCodeGoCredentialStore::from_auth_path(path.clone());
        store
            .save(OpenCodeGoCredentials::new("synthetic-go-key").unwrap())
            .await
            .unwrap();
        assert!(store.load().unwrap().is_some());
        assert!(store.remove().await.unwrap());

        let remaining = crate::auth::storage::read_auth_json(&path).unwrap();
        assert!(remaining.get("xai::api_key").is_some());
        assert!(!remaining.contains_key(OPENCODE_GO_AUTH_SCOPE));
    }
}
