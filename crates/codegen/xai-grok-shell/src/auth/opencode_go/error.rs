use std::fmt;

/// Fixed-shape OpenCode Go failures. Provider response bodies and credentials
/// are deliberately never carried by an error variant.
#[derive(thiserror::Error)]
pub enum OpenCodeGoAuthError {
    #[error("OpenCode Go API-key authentication is not configured")]
    Unavailable,
    #[error("OpenCode Go API-key authentication is invalid")]
    InvalidCredential,
    #[error("OpenCode Go credentials could not be stored: {0}")]
    Storage(#[from] std::io::Error),
    #[error("timed out while updating OpenCode Go credentials")]
    LockTimeout,
    #[error("OpenCode Go request failed with HTTP {0}")]
    Http(reqwest::StatusCode),
    #[error("OpenCode Go returned an invalid response")]
    InvalidResponse,
    #[error("OpenCode Go model discovery returned no supported models")]
    EmptyCatalog,
}

impl fmt::Debug for OpenCodeGoAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => f
                .debug_tuple("OpenCodeGoAuthError::Storage")
                .field(&error.kind())
                .finish(),
            Self::Http(status) => f
                .debug_tuple("OpenCodeGoAuthError::Http")
                .field(&status.as_u16())
                .finish(),
            Self::Unavailable => f.write_str("OpenCodeGoAuthError::Unavailable"),
            Self::InvalidCredential => f.write_str("OpenCodeGoAuthError::InvalidCredential"),
            Self::LockTimeout => f.write_str("OpenCodeGoAuthError::LockTimeout"),
            Self::InvalidResponse => f.write_str("OpenCodeGoAuthError::InvalidResponse"),
            Self::EmptyCatalog => f.write_str("OpenCodeGoAuthError::EmptyCatalog"),
        }
    }
}
