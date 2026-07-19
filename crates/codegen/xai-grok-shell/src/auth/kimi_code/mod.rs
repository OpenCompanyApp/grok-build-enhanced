//! First-class Kimi Code API-key authentication, catalog discovery, and
//! provider adapters.
//!
//! Kimi records are isolated from xAI sessions, ChatGPT Codex subscriptions,
//! and generic model API keys. The canonical managed endpoint is sealed in the
//! sampler and catalog clients.

mod catalog;
mod credentials;
mod error;
mod request_auth;
mod storage;
mod transport;
mod usage;

pub use catalog::{
    KimiCodeModel, KimiCodeThinkEfforts, fetch_models, load_cached_model_entries, map_models,
};
pub use credentials::{KIMI_CODE_CREDENTIAL_SCHEMA_VERSION, KimiCodeCredentials, KimiCodeSecret};
pub use error::KimiCodeAuthError;
pub use request_auth::{
    KimiCodeSamplerRequestAuth, KimiCodeToolAuthProvider, shared_sampler_request_auth,
    shared_tool_auth_provider,
};
use std::io::{IsTerminal, Read};
use std::sync::LazyLock;
pub use storage::KimiCodeCredentialStore;
pub use usage::{KimiCodeExtraUsage, KimiCodeUsageRow, KimiCodeUsageSnapshot, fetch_usage};

pub const KIMI_CODE_AUTH_SCOPE: &str = xai_grok_sampling_types::KIMI_CODE_AUTH_SCOPE;

// Environment credentials are intentionally ephemeral. The random process
// identity is opaque and unrelated to the secret/account; persisting it makes
// a later process fail closed instead of treating a different environment key
// as the same credential record.
static PROCESS_ENV_CREDENTIAL_RECORD_ID: LazyLock<String> =
    LazyLock::new(|| format!("env-process:{}", uuid::Uuid::new_v4()));

pub(crate) fn process_environment_binding() -> xai_grok_sampling_types::CredentialBinding {
    xai_grok_sampling_types::CredentialBinding {
        provider: xai_grok_sampling_types::ProviderId::KimiCode,
        source: xai_grok_sampling_types::CredentialSourceId::KimiCodeApiKey,
        record_id: Some(PROCESS_ENV_CREDENTIAL_RECORD_ID.clone()),
        generation: 1,
    }
}

pub(crate) fn current_credentials_and_binding(
    grok_home: &std::path::Path,
) -> Result<
    (
        KimiCodeCredentials,
        xai_grok_sampling_types::CredentialBinding,
    ),
    KimiCodeAuthError,
> {
    if let Some(credentials) = KimiCodeCredentialStore::new(grok_home).load()? {
        let binding = credentials.credential_binding();
        return Ok((credentials, binding));
    }
    let credentials = credentials_from_env()?;
    Ok((credentials, process_environment_binding()))
}

pub async fn run_kimi_code_cli_login() -> Result<(), KimiCodeAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let api_key = read_login_api_key()?;
    let credentials = KimiCodeCredentials::new(api_key)?;
    // Validate entitlement and capture the authenticated catalog before any
    // credential is committed. Provider response bodies are never reflected.
    let models = catalog::fetch_models(&credentials).await?;
    // Publish the credential-bound cache first. If that write fails, login
    // leaves the existing credential untouched. If the subsequent auth-store
    // write fails, the new cache remains unreachable because its random record
    // identity does not match the still-current credential.
    catalog::save_cache(&grok_home, &credentials, &models)?;
    KimiCodeCredentialStore::new(&grok_home)
        .save(credentials)
        .await?;
    println!(
        "Kimi Code login succeeded; discovered {} entitled model(s).",
        models.len()
    );
    Ok(())
}

pub async fn run_kimi_code_cli_logout() -> Result<(), KimiCodeAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let removed = KimiCodeCredentialStore::new(&grok_home).remove().await?;
    catalog::remove_cache(&grok_home)?;
    if removed {
        println!("Kimi Code credentials and model cache removed.");
    } else {
        println!("No stored Kimi Code credentials were found.");
    }
    Ok(())
}

pub async fn run_kimi_code_cli_models() -> Result<(), KimiCodeAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = KimiCodeCredentialStore::new(&grok_home);
    let stored = store.load()?;
    let credentials = match stored.as_ref() {
        Some(credentials) => credentials.clone(),
        None => credentials_from_env()?,
    };
    let models = catalog::fetch_models(&credentials).await?;
    if stored.is_some() {
        catalog::save_cache(&grok_home, &credentials, &models)?;
    }
    // Environment-only auth is deliberately process-local. Display discovery
    // results for this command, but do not persist a cache that a later process
    // with a different environment key could incorrectly adopt.
    for model in &models {
        let protocol = if model.protocol.as_deref() == Some("anthropic") {
            "messages"
        } else {
            "chat_completions"
        };
        println!(
            "kimi-code/{}\t{}\tcontext={}\timage={}\tvideo={}\treasoning={}",
            model.id,
            protocol,
            model.context_length,
            model.supports_image_in,
            model.supports_video_in,
            model.supports_reasoning,
        );
    }
    Ok(())
}

fn credentials_from_env() -> Result<KimiCodeCredentials, KimiCodeAuthError> {
    let api_key = std::env::var(xai_grok_sampling_types::KIMI_CODE_API_KEY_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(KimiCodeAuthError::Unavailable)?;
    KimiCodeCredentials::new(api_key)
}

fn read_login_api_key() -> Result<String, KimiCodeAuthError> {
    if let Ok(api_key) = std::env::var(xai_grok_sampling_types::KIMI_CODE_API_KEY_ENV)
        && !api_key.trim().is_empty()
    {
        return Ok(api_key);
    }
    let mut stdin = std::io::stdin();
    if stdin.is_terminal() {
        eprintln!(
            "Set KIMI_API_KEY for this command or pipe the API key on standard input; interactive echo is deliberately not used."
        );
        return Err(KimiCodeAuthError::Unavailable);
    }
    read_login_api_key_from(&mut stdin)
}

fn read_login_api_key_from(reader: &mut impl Read) -> Result<String, KimiCodeAuthError> {
    let mut input = Vec::new();
    reader
        .take((credentials::MAX_KIMI_CODE_API_KEY_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(KimiCodeAuthError::Storage)?;
    if input.len() > credentials::MAX_KIMI_CODE_API_KEY_BYTES {
        return Err(KimiCodeAuthError::InvalidCredential);
    }
    let input = String::from_utf8(input).map_err(|_| KimiCodeAuthError::InvalidCredential)?;
    let api_key = input.trim().to_owned();
    if api_key.is_empty() {
        return Err(KimiCodeAuthError::Unavailable);
    }
    // Apply the same bounded/header-safe validation used for environment and
    // persisted credentials before returning material to the login flow.
    KimiCodeSecret::new(api_key.clone())?;
    Ok(api_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_input_over_the_credential_limit_is_rejected_instead_of_truncated() {
        let mut input = std::io::Cursor::new(vec![b'k'; 16 * 1024 + 1]);

        let result = read_login_api_key_from(&mut input);

        assert!(matches!(result, Err(KimiCodeAuthError::InvalidCredential)));
    }

    #[test]
    fn login_input_trims_one_terminal_line_ending() {
        let mut input = std::io::Cursor::new(b"sentinel-kimi-key\n".to_vec());

        let api_key = read_login_api_key_from(&mut input).unwrap();

        assert_eq!(api_key, "sentinel-kimi-key");
    }
}
