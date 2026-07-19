//! First-class global personal Z.AI GLM Coding Plan authentication, catalog,
//! usage, and provider-bound request authentication.

mod catalog;
mod credentials;
mod error;
mod request_auth;
mod storage;
mod transport;
mod usage;

pub use catalog::{ZaiCodingPlanModel, fetch_models, load_cached_model_entries, map_models};
pub use credentials::{
    ZAI_CODING_PLAN_CREDENTIAL_SCHEMA_VERSION, ZaiCodingPlanCredentials, ZaiCodingPlanSecret,
};
pub use error::ZaiCodingPlanAuthError;
pub use request_auth::{
    ZaiCodingPlanSamplerRequestAuth, ZaiCodingPlanToolAuthProvider, shared_sampler_request_auth,
    shared_tool_auth_provider,
};
pub use storage::ZaiCodingPlanCredentialStore;
pub use usage::{
    ZaiCodingPlanUsageDetail, ZaiCodingPlanUsageRow, ZaiCodingPlanUsageSnapshot, fetch_usage,
};

use std::io::{IsTerminal, Read};
use std::sync::LazyLock;

pub const ZAI_CODING_PLAN_AUTH_SCOPE: &str = xai_grok_sampling_types::ZAI_CODING_PLAN_AUTH_SCOPE;

static PROCESS_ENV_CREDENTIAL_RECORD_ID: LazyLock<String> =
    LazyLock::new(|| format!("env-process:{}", uuid::Uuid::new_v4()));

pub(crate) fn process_environment_binding() -> xai_grok_sampling_types::CredentialBinding {
    xai_grok_sampling_types::CredentialBinding {
        provider: xai_grok_sampling_types::ProviderId::ZaiCodingPlan,
        source: xai_grok_sampling_types::CredentialSourceId::ZaiCodingPlanApiKey,
        record_id: Some(PROCESS_ENV_CREDENTIAL_RECORD_ID.clone()),
        generation: 1,
    }
}

pub(crate) fn current_credentials_and_binding(
    grok_home: &std::path::Path,
) -> Result<
    (
        ZaiCodingPlanCredentials,
        xai_grok_sampling_types::CredentialBinding,
    ),
    ZaiCodingPlanAuthError,
> {
    if let Some(credentials) = ZaiCodingPlanCredentialStore::new(grok_home).load()? {
        let binding = credentials.credential_binding();
        return Ok((credentials, binding));
    }
    let credentials = credentials_from_env()?;
    Ok((credentials, process_environment_binding()))
}

pub async fn run_zai_coding_plan_cli_login() -> Result<(), ZaiCodingPlanAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let api_key = read_login_api_key()?;
    let credentials = ZaiCodingPlanCredentials::new(api_key)?;
    // Validate the dedicated Coding Plan catalog before any credential write.
    let models = catalog::fetch_models(&credentials).await?;
    // Publish the credential-bound cache first. If it fails, the existing
    // credential remains untouched; if the later auth-store write fails, the
    // new cache is unreachable because its opaque record ID does not match.
    catalog::save_cache(&grok_home, &credentials, &models)?;
    ZaiCodingPlanCredentialStore::new(&grok_home)
        .save(credentials)
        .await?;
    println!(
        "Z.AI Coding Plan login succeeded; discovered {} entitled model(s).",
        models.len()
    );
    Ok(())
}

pub async fn run_zai_coding_plan_cli_logout() -> Result<(), ZaiCodingPlanAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let removed = ZaiCodingPlanCredentialStore::new(&grok_home)
        .remove()
        .await?;
    catalog::remove_cache(&grok_home)?;
    if removed {
        println!("Z.AI Coding Plan credentials and model cache removed.");
    } else {
        println!("No stored Z.AI Coding Plan credentials were found.");
    }
    Ok(())
}

pub async fn run_zai_coding_plan_cli_models() -> Result<(), ZaiCodingPlanAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = ZaiCodingPlanCredentialStore::new(&grok_home);
    let stored = store.load()?;
    let credentials = match stored.as_ref() {
        Some(credentials) => credentials.clone(),
        None => credentials_from_env()?,
    };
    let models = catalog::fetch_models(&credentials).await?;
    if stored.is_some() {
        catalog::save_cache(&grok_home, &credentials, &models)?;
    }
    let entries = catalog::map_models(models);
    for (id, entry) in entries {
        println!(
            "{}\tchat_completions\tcontext={}\tmax_output={}\timage=false\tselectable={}",
            id,
            entry.info.context_window,
            entry
                .info
                .max_completion_tokens
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_owned()),
            entry.info.user_selectable,
        );
    }
    Ok(())
}

pub fn credentials_from_env() -> Result<ZaiCodingPlanCredentials, ZaiCodingPlanAuthError> {
    let api_key = std::env::var(xai_grok_sampling_types::ZAI_CODING_PLAN_API_KEY_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(ZaiCodingPlanAuthError::Unavailable)?;
    ZaiCodingPlanCredentials::new(api_key)
}

fn read_login_api_key() -> Result<String, ZaiCodingPlanAuthError> {
    if let Ok(api_key) = std::env::var(xai_grok_sampling_types::ZAI_CODING_PLAN_API_KEY_ENV)
        && !api_key.trim().is_empty()
    {
        return Ok(api_key);
    }
    let mut stdin = std::io::stdin();
    if stdin.is_terminal() {
        eprintln!(
            "Set Z_AI_API_KEY for this command or pipe the API key on standard input; interactive echo is deliberately not used."
        );
        return Err(ZaiCodingPlanAuthError::Unavailable);
    }
    read_login_api_key_from(&mut stdin)
}

fn read_login_api_key_from(reader: &mut impl Read) -> Result<String, ZaiCodingPlanAuthError> {
    let mut input = Vec::new();
    reader
        .take((credentials::MAX_ZAI_CODING_PLAN_API_KEY_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(ZaiCodingPlanAuthError::Storage)?;
    if input.len() > credentials::MAX_ZAI_CODING_PLAN_API_KEY_BYTES {
        return Err(ZaiCodingPlanAuthError::InvalidCredential);
    }
    let input = String::from_utf8(input).map_err(|_| ZaiCodingPlanAuthError::InvalidCredential)?;
    let api_key = input.trim().to_owned();
    if api_key.is_empty() {
        return Err(ZaiCodingPlanAuthError::Unavailable);
    }
    ZaiCodingPlanSecret::new(api_key.clone())?;
    Ok(api_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_input_is_bounded_and_trimmed() {
        let mut input = std::io::Cursor::new(b"sentinel-zai-key\n".to_vec());
        assert_eq!(
            read_login_api_key_from(&mut input).unwrap(),
            "sentinel-zai-key"
        );
    }
}
