//! `grok models` subcommand.

use anyhow::Result;
use tokio_util::sync::CancellationToken;
use xai_grok_shell::agent::config::Config as AgentConfig;
use xai_grok_shell::cli_models::{AuthStatus, list_codex_subscription_models, list_models};

use crate::client_identity::{PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION};

pub async fn list_available_models(agent_config: &AgentConfig) -> Result<()> {
    match AuthStatus::resolve(agent_config) {
        AuthStatus::ApiKey => println!("You are using XAI_API_KEY."),
        AuthStatus::LoggedIn(host) => println!("You are logged in with {}.", host),
        AuthStatus::ModelCredentials(model) => {
            println!("Model '{model}' is using its own API key.");
        }
        AuthStatus::DeploymentKey => println!("You are authenticated via deployment key."),
        AuthStatus::NotAuthenticated => println!("You are not authenticated."),
    }
    println!();

    let cancel = CancellationToken::new();
    let spawned = crate::acp::spawn::spawn_grok_shell(agent_config.clone(), &cancel, None).await?;

    let state = list_models(&spawned.channel.tx, PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION).await?;

    println!("Default model: {}", state.current_model_id.0);
    println!();
    println!("Available models:");
    for m in state.available_models {
        if m.model_id == state.current_model_id {
            println!("  * {} (default)", m.model_id.0);
        } else {
            println!("  - {}", m.model_id.0);
        }
    }

    cancel.cancel();
    Ok(())
}

/// List only models advertised to the authenticated ChatGPT Codex account.
///
/// Unlike the default xAI path this does not start an ACP agent just to obtain
/// model state: Codex discovery has its own authenticated catalog endpoint and
/// provider-scoped 401 recovery.
pub async fn list_available_codex_models() -> Result<()> {
    let state = list_codex_subscription_models().await?;

    println!("You are logged in with ChatGPT Codex.");
    println!();
    if let Some(default_model_id) = state.default_model_id.as_deref() {
        println!("Default model: {default_model_id}");
    } else {
        println!("Default model: (none)");
    }
    println!();
    println!("Available models:");
    if state.available_models.is_empty() {
        println!("  (none advertised for this account)");
    }
    for model in state.available_models {
        if state.default_model_id.as_deref() == Some(model.id.as_str()) {
            println!("  * {} (default)", model.id);
        } else {
            println!("  - {}", model.id);
        }
    }

    Ok(())
}
