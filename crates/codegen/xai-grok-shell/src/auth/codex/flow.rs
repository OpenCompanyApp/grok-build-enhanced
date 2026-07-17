use tokio_util::sync::CancellationToken;

use super::{
    CodexAuthError, CodexAuthManager, CodexCredentialStore, CodexCredentials, CodexLogoutResult,
    CodexOAuthClient,
};

/// Run the interactive Codex subscription login used by the pager CLI.
///
/// `device_auth = false` uses browser OAuth with PKCE and callback-state
/// validation. `true` uses OpenAI's current device authorization flow.
pub async fn run_codex_cli_login(device_auth: bool) -> Result<CodexCredentials, CodexAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = CodexCredentialStore::new(&grok_home);
    let oauth = CodexOAuthClient::new();
    let cancellation = CancellationToken::new();

    if device_auth {
        let authorization = oauth.request_device_authorization().await?;
        println!(
            "Open {} and enter code: {}",
            authorization.verification_url(),
            authorization.user_code()
        );
        oauth
            .complete_device_login(authorization, &store, None, &cancellation)
            .await
    } else {
        let pending = oauth.begin_browser_login(None).await?;
        println!("Open this URL to sign in to ChatGPT:");
        println!("{}", pending.authorization_url());
        if pending.open_browser().is_err() {
            eprintln!("Could not open a browser automatically; use the URL above.");
        }
        pending.complete(&store, &cancellation).await
    }
}

/// Revoke best-effort and remove only Grok Build's `openai::codex` record.
pub async fn run_codex_cli_logout() -> Result<CodexLogoutResult, CodexAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    CodexAuthManager::new(&grok_home)?.logout().await
}
