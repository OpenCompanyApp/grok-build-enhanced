//! Bearer resolution for voice STT requests.
//! The voice clients are long-lived: a single voice session opens many STT
//! WebSocket connections over its lifetime, and an OAuth/session bearer rotates
//! (~15 min). Capturing a token once at startup would 401 mid-session. So
//! instead of a static `String`, the clients hold a [`SharedVoiceAuth`] and
//! resolve a fresh bearer at the point of each connection.
//!
//! This crate stays dependency-light: it defines its own minimal async trait
//! rather than depending on the shell's `AuthManager` / tools' `ApiKeyProvider`.
//! The pager adapts the shell's refreshing provider onto this trait.

use std::future::{Future, ready};
use std::pin::Pin;
use std::sync::Arc;

use crate::config::VoiceConfig;
use crate::error::VoiceError;

pub trait VoiceAuthProvider: std::fmt::Debug + Send + Sync + 'static {
    /// Bind the credential lookup to the model receiving this dictation.
    ///
    /// Standalone providers may ignore the binding. Provider-aware adapters
    /// must fail closed for an absent or ineligible model id.
    fn bind_model(&self, _model_id: Option<&str>) {}

    fn bearer(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>>;
}

/// Shared provider handed to the voice pipeline.
pub type SharedVoiceAuth = Arc<dyn VoiceAuthProvider>;

pub(crate) async fn require_bearer(
    auth: &SharedVoiceAuth,
    config: &VoiceConfig,
) -> Result<String, VoiceError> {
    // Validate the destination before invoking the credential provider. This
    // ordering guarantees that even an in-memory key is never resolved for a
    // noncanonical speech endpoint.
    config.stt_ws_url()?;
    auth.bearer().await.ok_or_else(|| {
        VoiceError::Auth("voice requires an eligible xAI model and xAI session or API key".into())
    })
}

/// A fixed bearer that never refreshes.
///
/// Used by the standalone `voice-probe` binary and tests, where there is no
/// `AuthManager` — only a raw `XAI_API_KEY`.
pub struct StaticVoiceAuth(pub String);

impl std::fmt::Debug for StaticVoiceAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticVoiceAuth")
            .field(&"<redacted>")
            .finish()
    }
}

impl VoiceAuthProvider for StaticVoiceAuth {
    fn bearer(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        Box::pin(ready(Some(self.0.clone())))
    }
}

impl StaticVoiceAuth {
    /// Build a [`SharedVoiceAuth`] from a static key, trimming whitespace and
    /// rejecting an empty value.
    pub fn shared(key: impl Into<String>) -> Option<SharedVoiceAuth> {
        let key = key.into().trim().to_string();
        if key.is_empty() {
            return None;
        }
        Some(Arc::new(Self(key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn static_provider_resolves() {
        let provider = StaticVoiceAuth::shared("  sk-test  ").unwrap();
        assert_eq!(provider.bearer().await.as_deref(), Some("sk-test"));
    }

    #[test]
    fn static_provider_rejects_empty() {
        assert!(StaticVoiceAuth::shared("   ").is_none());
    }

    #[test]
    fn static_provider_debug_is_redacted() {
        let provider = StaticVoiceAuth("sentinel-voice-key".into());
        let rendered = format!("{provider:?}");
        assert!(!rendered.contains("sentinel-voice-key"));
        assert!(rendered.contains("redacted"));
    }

    #[derive(Debug)]
    struct CountingAuth(std::sync::atomic::AtomicUsize);

    impl VoiceAuthProvider for CountingAuth {
        fn bearer(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(ready(Some("must-not-resolve".into())))
        }
    }

    #[tokio::test]
    async fn noncanonical_endpoint_is_rejected_before_credential_resolution() {
        let auth = Arc::new(CountingAuth(std::sync::atomic::AtomicUsize::new(0)));
        let shared: SharedVoiceAuth = auth.clone();
        let config = VoiceConfig {
            api_base: "https://custom.example".into(),
            ..VoiceConfig::default()
        };

        let error = require_bearer(&shared, &config).await.unwrap_err();

        assert!(matches!(error, VoiceError::Config(_)));
        assert_eq!(auth.0.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
}
