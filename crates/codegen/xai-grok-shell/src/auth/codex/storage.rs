use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use fs2::FileExt;

use super::{CodexAuthError, CodexCredentials, OPENAI_CODEX_SCOPE};
use crate::auth::manager::lock::try_lock_auth_file_async;
use crate::auth::model::AuthStore;
use crate::auth::storage::{read_auth_json, read_auth_json_or_empty, write_auth_json};

const AUTH_FILE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const PROVIDER_LOCK_TIMEOUT: Duration = Duration::from_secs(45);
const PROVIDER_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Kernel-backed provider lock that serializes the complete refresh-token
/// rotation transaction across Grok Build processes. The file is deliberately
/// never unlinked: closing the descriptor releases the lock after crashes,
/// while unlinking could split contenders across two inodes.
pub(super) struct CodexProviderLock {
    _file: File,
}

/// Provider-scoped persistence for OpenAI Codex credentials in Grok's
/// existing `auth.json`. All writes retain xAI and unknown future scopes.
#[derive(Clone, Debug)]
pub struct CodexCredentialStore {
    auth_path: PathBuf,
}

impl CodexCredentialStore {
    pub fn new(grok_home: &Path) -> Self {
        let auth_path = crate::auth::resolved_auth_path(grok_home);
        Self { auth_path }
    }

    pub fn from_auth_path(auth_path: PathBuf) -> Self {
        Self { auth_path }
    }

    pub fn auth_path(&self) -> &Path {
        &self.auth_path
    }

    pub fn load(&self) -> Result<Option<CodexCredentials>, CodexAuthError> {
        match read_auth_json(&self.auth_path) {
            Ok(store) => {
                let credentials = store.get_codex().cloned();
                if credentials.is_none() && store.contains_key(OPENAI_CODEX_SCOPE) {
                    return Err(CodexAuthError::Storage(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "openai::codex credential record is invalid",
                    )));
                }
                Ok(credentials)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(CodexAuthError::Storage(error)),
        }
    }

    pub async fn save(&self, credentials: CodexCredentials) -> Result<(), CodexAuthError> {
        let _provider_lock = self.acquire_provider_lock().await?;
        self.save_locked(credentials).await
    }

    pub(super) async fn save_locked(
        &self,
        credentials: CodexCredentials,
    ) -> Result<(), CodexAuthError> {
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(CodexAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(CodexAuthError::LockTimeout);
        }
        let mut store = read_auth_json_or_empty(&self.auth_path)?;
        store.insert_codex(credentials);
        write_auth_json(&self.auth_path, &store)?;
        Ok(())
    }

    pub async fn remove(&self) -> Result<Option<CodexCredentials>, CodexAuthError> {
        let _provider_lock = self.acquire_provider_lock().await?;
        self.remove_locked().await
    }

    pub(super) async fn remove_locked(&self) -> Result<Option<CodexCredentials>, CodexAuthError> {
        let lock = try_lock_auth_file_async(&self.auth_path, AUTH_FILE_LOCK_TIMEOUT)
            .await
            .ok_or(CodexAuthError::LockTimeout)?;
        if !lock.still_live(&self.auth_path) {
            return Err(CodexAuthError::LockTimeout);
        }

        let mut store = match read_auth_json(&self.auth_path) {
            Ok(store) => store,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AuthStore::new(),
            Err(error) => return Err(CodexAuthError::Storage(error)),
        };
        let removed = store.remove_codex();
        if removed.is_none() {
            return Ok(None);
        }
        if store.is_empty() {
            match std::fs::remove_file(&self.auth_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(CodexAuthError::Storage(error)),
            }
        } else {
            write_auth_json(&self.auth_path, &store)?;
        }
        Ok(removed)
    }

    pub(super) async fn acquire_provider_lock(&self) -> Result<CodexProviderLock, CodexAuthError> {
        let lock_path = self.auth_path.with_file_name("auth.json.openai-codex.lock");
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = open_provider_lock_file(&lock_path)?;
        let deadline = tokio::time::Instant::now() + PROVIDER_LOCK_TIMEOUT;
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(CodexProviderLock { _file: file }),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(CodexAuthError::Storage(error)),
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(CodexAuthError::LockTimeout);
            }
            tokio::time::sleep(PROVIDER_LOCK_POLL_INTERVAL).await;
        }
    }
}

fn open_provider_lock_file(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o600);
        file.set_permissions(permissions)?;
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration as ChronoDuration, Utc};

    use super::*;
    use crate::auth::GrokAuth;
    use crate::auth::codex::credentials::credentials_for_test;

    #[tokio::test]
    async fn codex_save_preserves_xai_and_unknown_scopes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let xai = GrokAuth {
            key: "xai-secret".to_owned(),
            user_id: "xai-user".to_owned(),
            ..GrokAuth::test_default()
        };
        let initial = serde_json::json!({
            "xai::api_key": serde_json::to_value(&xai).unwrap(),
            "future::provider": {
                "opaque": [1, 2, 3],
                "new_field": { "nested": true }
            }
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&initial).unwrap()).unwrap();

        let store = CodexCredentialStore::from_auth_path(path.clone());
        store
            .save(credentials_for_test(
                "acct-one",
                "refresh-one",
                Utc::now() + ChronoDuration::hours(1),
            ))
            .await
            .unwrap();

        let raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(raw["xai::api_key"]["key"], "xai-secret");
        assert_eq!(raw["future::provider"], initial["future::provider"]);
        assert_eq!(raw[OPENAI_CODEX_SCOPE]["provider"], "openai_codex");
        assert_eq!(store.load().unwrap().unwrap().account_id(), "acct-one");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            let lock = path.with_file_name("auth.json.openai-codex.lock");
            assert_eq!(
                std::fs::metadata(lock).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[tokio::test]
    async fn codex_remove_does_not_log_out_xai() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let store = CodexCredentialStore::from_auth_path(path.clone());
        let mut auth_store = AuthStore::new();
        auth_store.insert("xai::api_key".to_owned(), GrokAuth::test_default());
        crate::auth::storage::write_auth_json(&path, &auth_store).unwrap();
        store
            .save(credentials_for_test(
                "acct-one",
                "refresh-one",
                Utc::now() + ChronoDuration::hours(1),
            ))
            .await
            .unwrap();

        assert!(store.remove().await.unwrap().is_some());
        assert!(store.load().unwrap().is_none());
        let remaining = crate::auth::storage::read_auth_json(&path).unwrap();
        assert!(remaining.get("xai::api_key").is_some());
        assert!(!remaining.contains_key(OPENAI_CODEX_SCOPE));
    }

    #[test]
    fn malformed_codex_record_is_not_silently_treated_as_logged_out() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        std::fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                (OPENAI_CODEX_SCOPE): { "provider": "openai_codex" }
            }))
            .unwrap(),
        )
        .unwrap();
        let store = CodexCredentialStore::from_auth_path(path);
        assert!(matches!(store.load(), Err(CodexAuthError::Storage(_))));
    }
}
