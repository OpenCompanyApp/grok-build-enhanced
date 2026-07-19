use std::fmt;

/// Fixed-shape failures for provider-owned Z.AI Coding Plan authentication.
///
/// No variant carries credential material, account identifiers, provider
/// messages, or provider response bodies.
#[derive(thiserror::Error)]
pub enum ZaiCodingPlanAuthError {
    #[error("Z.AI Coding Plan API-key authentication is not configured")]
    Unavailable,
    #[error("Z.AI Coding Plan API-key authentication is invalid")]
    InvalidCredential,
    #[error("Z.AI Coding Plan credentials could not be stored: {0}")]
    Storage(#[from] std::io::Error),
    #[error("timed out while updating Z.AI Coding Plan credentials")]
    LockTimeout,
    #[error("Z.AI Coding Plan request failed with HTTP {0}")]
    Http(reqwest::StatusCode),
    #[error("Z.AI Coding Plan request failed with provider code {0}")]
    Business(i64),
    #[error("Z.AI Coding Plan returned an invalid response")]
    InvalidResponse,
    #[error("Z.AI Coding Plan model discovery returned no usable models")]
    EmptyCatalog,
}

impl fmt::Debug for ZaiCodingPlanAuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => formatter
                .debug_tuple("ZaiCodingPlanAuthError::Storage")
                .field(&error.kind())
                .finish(),
            Self::Http(status) => formatter
                .debug_tuple("ZaiCodingPlanAuthError::Http")
                .field(&status.as_u16())
                .finish(),
            Self::Business(code) => formatter
                .debug_tuple("ZaiCodingPlanAuthError::Business")
                .field(code)
                .finish(),
            Self::Unavailable => formatter.write_str("ZaiCodingPlanAuthError::Unavailable"),
            Self::InvalidCredential => {
                formatter.write_str("ZaiCodingPlanAuthError::InvalidCredential")
            }
            Self::LockTimeout => formatter.write_str("ZaiCodingPlanAuthError::LockTimeout"),
            Self::InvalidResponse => {
                formatter.write_str("ZaiCodingPlanAuthError::InvalidResponse")
            }
            Self::EmptyCatalog => formatter.write_str("ZaiCodingPlanAuthError::EmptyCatalog"),
        }
    }
}
