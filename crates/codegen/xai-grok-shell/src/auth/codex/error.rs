use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CodexAuthError {
    #[error("not signed in to OpenAI Codex; run `grok login --provider openai-codex`")]
    NotLoggedIn,
    #[error("OpenAI Codex OAuth callback timed out")]
    CallbackTimeout,
    #[error("OpenAI Codex OAuth callback state did not match")]
    StateMismatch,
    #[error("OpenAI Codex OAuth callback was denied")]
    AuthorizationDenied,
    #[error("OpenAI Codex OAuth response was invalid: {0}")]
    InvalidTokenResponse(&'static str),
    #[error("OpenAI Codex token was not a valid JWT")]
    InvalidJwt,
    #[error("OpenAI Codex token did not identify a ChatGPT account")]
    MissingAccountId,
    #[error("the selected ChatGPT workspace is not allowed")]
    WorkspaceNotAllowed,
    #[error("the ChatGPT account changed; rebuild the provider session")]
    AccountChanged,
    #[error("OpenAI Codex device login is not available")]
    DeviceFlowUnavailable,
    #[error("OpenAI Codex device login expired")]
    DeviceCodeExpired,
    #[error("OpenAI Codex device login was denied")]
    DeviceCodeDenied,
    #[error("OpenAI Codex login was cancelled")]
    Cancelled,
    #[error("OpenAI Codex credential refresh is no longer possible; sign in again")]
    RefreshRejected,
    #[error("OpenAI Codex credential lock timed out")]
    LockTimeout,
    #[error("OpenAI Codex authentication service returned HTTP {0}")]
    HttpStatus(u16),
    #[error("OpenAI Codex authentication HTTP route setup failed; proxy details were omitted")]
    ProxyRoute,
    #[error("OpenAI Codex authentication transport failed")]
    Transport(#[source] reqwest::Error),
    #[error("failed to store OpenAI Codex credentials")]
    Storage(#[source] std::io::Error),
}

impl From<std::io::Error> for CodexAuthError {
    fn from(error: std::io::Error) -> Self {
        Self::Storage(error)
    }
}

impl From<reqwest::Error> for CodexAuthError {
    fn from(error: reqwest::Error) -> Self {
        Self::Transport(error)
    }
}
