use super::super::{CodexAuthError, CodexAuthManager, CodexAuthSnapshot, CodexAuthSnapshotError};

fn catalog_auth_error(error: &CodexAuthError) -> CodexAuthSnapshotError {
    let transient_storage = matches!(
        error,
        CodexAuthError::Storage(source)
            if matches!(
                source.kind(),
                std::io::ErrorKind::Interrupted
                    | std::io::ErrorKind::WouldBlock
                    | std::io::ErrorKind::TimedOut
            )
    );
    if transient_storage
        || matches!(
            error,
            CodexAuthError::Transport(_)
                | CodexAuthError::LockTimeout
                | CodexAuthError::HttpStatus(408 | 429 | 500..=599)
        )
    {
        CodexAuthSnapshotError::Transient
    } else {
        CodexAuthSnapshotError::Unavailable
    }
}

#[async_trait::async_trait]
impl crate::remote::CodexCatalogAuthProvider for CodexAuthManager {
    async fn catalog_auth(&self) -> Result<CodexAuthSnapshot, CodexAuthSnapshotError> {
        self.auth_snapshot()
            .await
            .map_err(|error| catalog_auth_error(&error))
    }

    async fn recover_unauthorized(
        &self,
        rejected: xai_grok_sampling_types::CredentialBinding,
    ) -> Result<bool, CodexAuthSnapshotError> {
        self.recover_after_unauthorized(rejected)
            .await
            .map_err(|error| catalog_auth_error(&error))
    }
}

/// Static snapshots remain useful for tests and embedding callers. Their
/// default 401 recovery is inert, so a rejected Codex request can never fall
/// through to xAI or another credential source.
#[async_trait::async_trait]
impl crate::remote::CodexCatalogAuthProvider for CodexAuthSnapshot {
    async fn catalog_auth(&self) -> Result<CodexAuthSnapshot, CodexAuthSnapshotError> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, Utc};

    use super::*;
    use crate::auth::codex::CodexCredentialStore;
    use crate::auth::codex::credentials::credentials_for_test;
    use crate::remote::CodexCatalogAuthProvider as _;

    #[tokio::test]
    async fn catalog_adapter_returns_the_common_exact_redacted_snapshot() {
        let directory = tempfile::tempdir().unwrap();
        let store = CodexCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let credentials = credentials_for_test(
            "sentinel-account-id",
            "sentinel-refresh-token",
            Utc::now() + Duration::hours(1),
        );
        let expected = credentials.credential_binding();
        store.save(credentials).await.unwrap();
        let manager = Arc::new(CodexAuthManager::from_store(store).unwrap());

        let snapshot = manager.catalog_auth().await.unwrap();
        assert_eq!(snapshot.credential_binding(), &expected);
        let rendered = format!("{snapshot:?}");
        assert_eq!(rendered, "CodexAuthSnapshot { .. }");
        assert!(!rendered.contains("sentinel-account-id"));
        assert!(!rendered.contains("sentinel-refresh-token"));
    }
}
