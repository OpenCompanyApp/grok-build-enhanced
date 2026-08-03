//! First-class OpenCode Go API-key authentication and model discovery.

mod catalog;
mod credentials;
mod error;
mod request_auth;
mod storage;

pub use catalog::{
    OpenCodeGoMediaSupport, OpenCodeGoModel, fetch_models, load_cached_model_entries, map_models,
    media_support, save_cache,
};
pub use credentials::{
    OPENCODE_GO_CREDENTIAL_SCHEMA_VERSION, OpenCodeGoCredentials, OpenCodeGoSecret,
};
pub use error::OpenCodeGoAuthError;
pub use request_auth::{OpenCodeGoRequestAuth, shared_request_auth};
pub use storage::OpenCodeGoCredentialStore;

use std::io::{IsTerminal, Read};
use std::sync::LazyLock;

pub const OPENCODE_GO_AUTH_SCOPE: &str = xai_grok_sampling_types::OPENCODE_GO_AUTH_SCOPE;

static PROCESS_ENV_CREDENTIAL_RECORD_ID: LazyLock<String> =
    LazyLock::new(|| format!("env-process:{}", uuid::Uuid::new_v4()));

pub(crate) fn process_environment_binding() -> xai_grok_sampling_types::CredentialBinding {
    xai_grok_sampling_types::CredentialBinding {
        provider: xai_grok_sampling_types::ProviderId::OpenCodeGo,
        source: xai_grok_sampling_types::CredentialSourceId::OpenCodeGoApiKey,
        record_id: Some(PROCESS_ENV_CREDENTIAL_RECORD_ID.clone()),
        generation: 1,
    }
}

pub(crate) fn current_credentials_and_binding(
    grok_home: &std::path::Path,
) -> Result<
    (
        OpenCodeGoCredentials,
        xai_grok_sampling_types::CredentialBinding,
    ),
    OpenCodeGoAuthError,
> {
    if let Some(credentials) = OpenCodeGoCredentialStore::new(grok_home).load()? {
        let binding = credentials.credential_binding();
        return Ok((credentials, binding));
    }
    let credentials = credentials_from_env()?;
    Ok((credentials, process_environment_binding()))
}

pub async fn run_cli_login() -> Result<(), OpenCodeGoAuthError> {
    let grok_home = crate::util::grok_home::grok_home();
    let credentials = OpenCodeGoCredentials::new(read_login_api_key()?)?;
    // The Go catalog is public and therefore is not authentication proof. Save
    // the key after local validation and describe first inference accurately.
    let models = catalog::fetch_models().await?;
    catalog::save_cache(&grok_home, &models)?;
    OpenCodeGoCredentialStore::new(&grok_home)
        .save(credentials)
        .await?;
    println!(
        "OpenCode Go API key stored; discovered {} supported model(s). The key will be validated on first inference.",
        models.iter().filter(|model| !model.deprecated).count()
    );
    Ok(())
}

pub async fn run_cli_logout() -> Result<(), OpenCodeGoAuthError> {
    let removed = OpenCodeGoCredentialStore::new(&crate::util::grok_home::grok_home())
        .remove()
        .await?;
    if removed {
        println!("OpenCode Go credentials removed.");
    } else {
        println!("No stored OpenCode Go credentials were found.");
    }
    Ok(())
}

pub async fn run_cli_models() -> Result<(), OpenCodeGoAuthError> {
    let models = catalog::fetch_models().await?;
    catalog::save_cache(&crate::util::grok_home::grok_home(), &models)?;
    for model in models.into_iter().filter(|model| !model.deprecated) {
        let protocol = match model.backend {
            xai_grok_sampling_types::ApiBackend::ChatCompletions => "chat_completions",
            xai_grok_sampling_types::ApiBackend::Responses => "responses",
            xai_grok_sampling_types::ApiBackend::Messages => "messages",
        };
        println!(
            "opencode-go/{}\t{}\tcontext={}\timage={}\tpdf={}\taudio={}\tvideo={}",
            model.id,
            protocol,
            model.context_length,
            model.media.image,
            model.media.pdf,
            model.media.audio,
            model.media.video,
        );
    }
    Ok(())
}

fn credentials_from_env() -> Result<OpenCodeGoCredentials, OpenCodeGoAuthError> {
    let api_key = [
        xai_grok_sampling_types::OPENCODE_GO_API_KEY_ENV,
        xai_grok_sampling_types::OPENCODE_COMPAT_API_KEY_ENV,
    ]
    .into_iter()
    .find_map(|name| {
        std::env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
    })
    .ok_or(OpenCodeGoAuthError::Unavailable)?;
    OpenCodeGoCredentials::new(api_key)
}

fn read_login_api_key() -> Result<String, OpenCodeGoAuthError> {
    for name in [
        xai_grok_sampling_types::OPENCODE_GO_API_KEY_ENV,
        xai_grok_sampling_types::OPENCODE_COMPAT_API_KEY_ENV,
    ] {
        if let Ok(api_key) = std::env::var(name)
            && !api_key.trim().is_empty()
        {
            return Ok(api_key);
        }
    }
    let mut stdin = std::io::stdin();
    if stdin.is_terminal() {
        eprintln!(
            "Set GROK_OPENCODE_GO_API_KEY (or OPENCODE_API_KEY for this explicit provider command), or pipe the API key on standard input."
        );
        return Err(OpenCodeGoAuthError::Unavailable);
    }
    let mut input = Vec::new();
    stdin
        .by_ref()
        .take((credentials::MAX_OPENCODE_GO_API_KEY_BYTES + 1) as u64)
        .read_to_end(&mut input)?;
    if input.len() > credentials::MAX_OPENCODE_GO_API_KEY_BYTES {
        return Err(OpenCodeGoAuthError::InvalidCredential);
    }
    let api_key = String::from_utf8(input)
        .map_err(|_| OpenCodeGoAuthError::InvalidCredential)?
        .trim()
        .to_owned();
    OpenCodeGoSecret::new(api_key.clone())?;
    Ok(api_key)
}
