use std::fmt;

/// Fixed-shape failures for provider-owned Kimi Code authentication.
///
/// No variant carries credential material or provider response bodies.
#[derive(thiserror::Error)]
pub enum KimiCodeAuthError {
    #[error("Kimi Code API-key authentication is not configured")]
    Unavailable,
    #[error("Kimi Code API-key authentication is invalid")]
    InvalidCredential,
    #[error("Kimi Code credentials could not be stored: {0}")]
    Storage(#[from] std::io::Error),
    #[error("timed out while updating Kimi Code credentials")]
    LockTimeout,
    #[error("Kimi Code request failed with HTTP {0}")]
    Http(reqwest::StatusCode),
    #[error("Kimi Code returned an invalid response")]
    InvalidResponse,
    #[error("Kimi Code model discovery returned no usable models")]
    EmptyCatalog,
}

impl fmt::Debug for KimiCodeAuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => formatter
                .debug_tuple("KimiCodeAuthError::Storage")
                .field(&error.kind())
                .finish(),
            Self::Http(status) => formatter
                .debug_tuple("KimiCodeAuthError::Http")
                .field(&status.as_u16())
                .finish(),
            Self::Unavailable => formatter.write_str("KimiCodeAuthError::Unavailable"),
            Self::InvalidCredential => formatter.write_str("KimiCodeAuthError::InvalidCredential"),
            Self::LockTimeout => formatter.write_str("KimiCodeAuthError::LockTimeout"),
            Self::InvalidResponse => formatter.write_str("KimiCodeAuthError::InvalidResponse"),
            Self::EmptyCatalog => formatter.write_str("KimiCodeAuthError::EmptyCatalog"),
        }
    }
}
