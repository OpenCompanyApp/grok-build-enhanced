//! Explicitly opt-in live validation for ChatGPT Codex standalone search.
//!
//! This module uses only Grok Build's provider-scoped Codex auth manager. It
//! never reads `~/.codex/auth.json`, accepts a token override, or prints an
//! authenticated request/response. Normal test runs remain fully offline.

use std::sync::Arc;

use xai_grok_tools::implementations::web_search::{
    CodexWebSearchContext, CodexWebSearchContextSize, CodexWebSearchImageSettings,
    CodexWebSearchLocation, CodexWebSearchMessage, CodexWebSearchMode, CodexWebSearchSettings,
    CodexWebSearchTurnMetadata, WebSearchConfig, client::WebSearchClient,
};
use xai_grok_tools::types::output::WebToolErrorCode;

use super::{CodexAuthManager, shared_api_key_provider};
use crate::remote::{CodexCatalogClient, CodexCatalogClientConfig, CodexCatalogError};

fn safe_skip(category: &'static str) {
    eprintln!("live Codex web validation skipped: {category}");
}

fn explicit_live_opt_in() -> bool {
    std::env::var("GROK_RUN_LIVE_CODEX_TESTS").as_deref() == Ok("1")
}

fn tool_failure_code(error: &xai_tool_runtime::ToolError) -> Option<WebToolErrorCode> {
    error
        .details
        .as_ref()
        .and_then(|details| details.get("code"))
        .and_then(|code| serde_json::from_value(code.clone()).ok())
}

fn skip_allowed_tool_error(error: &xai_tool_runtime::ToolError) -> bool {
    matches!(
        tool_failure_code(error),
        Some(
            WebToolErrorCode::AuthenticationRequired
                | WebToolErrorCode::Timeout
                | WebToolErrorCode::RateLimited
        )
    )
}

async fn entitled_model(manager: &CodexAuthManager) -> Result<Option<String>, &'static str> {
    if let Ok(explicit) = std::env::var("GROK_LIVE_CODEX_MODEL")
        && !explicit.trim().is_empty()
    {
        let slug = explicit
            .trim()
            .strip_prefix("openai-codex/")
            .unwrap_or(explicit.trim());
        if slug.is_empty() || slug.chars().any(char::is_control) {
            return Err("invalid_explicit_model");
        }
        return Ok(Some(slug.to_owned()));
    }

    let grok_home = crate::util::grok_home::grok_home();
    let config = CodexCatalogClientConfig::new(grok_home.join("cache").join("openai-codex"))
        .map_err(|_| "catalog_configuration")?;
    let client = CodexCatalogClient::new(config).map_err(|_| "catalog_configuration")?;
    let fetched = match client.fetch(manager).await {
        Ok(fetched) => fetched,
        Err(
            CodexCatalogError::AuthenticationUnavailable
            | CodexCatalogError::AuthenticationTransient
            | CodexCatalogError::Unauthorized,
        ) => return Err("authentication_unavailable"),
        Err(CodexCatalogError::Timeout | CodexCatalogError::HttpStatus(429)) => {
            return Err("provider_limit");
        }
        Err(CodexCatalogError::HttpStatus(401 | 403)) => {
            return Err("account_ineligible");
        }
        Err(
            CodexCatalogError::InvalidConfiguration
            | CodexCatalogError::Transport
            | CodexCatalogError::HttpStatus(_)
            | CodexCatalogError::ResponseTooLarge
            | CodexCatalogError::InvalidResponse
            | CodexCatalogError::InvalidModel
            | CodexCatalogError::NotModifiedWithoutCache
            | CodexCatalogError::CacheUnavailable,
        ) => return Err("catalog_contract_failure"),
    };
    Ok(fetched
        .catalog
        .models
        .into_iter()
        .find(|model| model.visible_in_picker())
        .map(|model| model.slug))
}

fn context(model: &str, ordinal: usize) -> CodexWebSearchContext {
    let correlation = format!("grok-live-web-contract-{ordinal}");
    CodexWebSearchContext {
        input: vec![CodexWebSearchMessage::user(
            "Validate the official OpenAI website search contract.",
        )],
        turn_metadata: CodexWebSearchTurnMetadata {
            session_id: correlation.clone(),
            thread_id: correlation.clone(),
            turn_id: correlation,
            model: model.to_owned(),
            reasoning_effort: None,
        },
        // Exercise the adaptive short-response ceiling, not the provider max.
        max_output_tokens: 2_048,
    }
}

fn config(model: &str, mode: CodexWebSearchMode, ordinal: usize) -> WebSearchConfig {
    WebSearchConfig::CodexSubscription {
        base_url: crate::auth::codex::OPENAI_CODEX_BASE_URL.to_owned(),
        model: model.to_owned(),
        session_id: format!("grok-live-search-{ordinal}"),
        settings: CodexWebSearchSettings {
            mode,
            search_context_size: Some(CodexWebSearchContextSize::Low),
            allowed_domains: Some(vec!["openai.com".to_owned()]),
            blocked_domains: vec!["example.org".to_owned()],
            user_location: Some(CodexWebSearchLocation {
                country: Some("US".to_owned()),
                region: None,
                city: None,
                timezone: Some("America/New_York".to_owned()),
            }),
            image_settings: Some(CodexWebSearchImageSettings {
                max_results: Some(2),
                caption: Some(true),
            }),
        },
    }
}

#[tokio::test]
#[ignore = "live ChatGPT Codex requests; also requires GROK_RUN_LIVE_CODEX_TESTS=1"]
async fn live_codex_search_modes_settings_and_reference_flow() {
    if !explicit_live_opt_in() {
        safe_skip("explicit_opt_in_missing");
        return;
    }

    let manager = match CodexAuthManager::new(&crate::util::grok_home::grok_home()) {
        Ok(manager) if manager.current().is_some() => Arc::new(manager),
        Ok(_) => {
            safe_skip("scoped_auth_missing");
            return;
        }
        Err(_) => {
            safe_skip("scoped_auth_unavailable");
            return;
        }
    };
    let model = match entitled_model(&manager).await {
        Ok(Some(model)) => model,
        Ok(None) => {
            safe_skip("no_entitled_model");
            return;
        }
        Err(
            category @ ("authentication_unavailable" | "provider_limit" | "account_ineligible"),
        ) => {
            safe_skip(category);
            return;
        }
        Err(category) => panic!("live Codex model discovery failed: {category}"),
    };
    let auth = shared_api_key_provider(Arc::clone(&manager));

    let mut live_result = None;
    for (ordinal, mode) in [
        CodexWebSearchMode::Cached,
        CodexWebSearchMode::Indexed,
        CodexWebSearchMode::Live,
    ]
    .into_iter()
    .enumerate()
    {
        let client = WebSearchClient::new(&config(&model, mode, ordinal), Some(auth.clone()))
            .expect("live Codex search client must satisfy the local contract");
        let result = client
            .run_commands(
                "official OpenAI website",
                serde_json::json!({
                    "search_query": [{"q": "official OpenAI website", "domains": ["openai.com"]}],
                    "response_length": "short"
                }),
                Some(vec!["openai.com".to_owned()]),
                Some(&context(&model, ordinal)),
            )
            .await;
        match result {
            Ok(result) => {
                assert!(
                    !result.content.trim().is_empty(),
                    "live search returned no content"
                );
                assert!(
                    result.content.len() <= 2_048 * 8,
                    "short output exceeded client defense"
                );
                if mode == CodexWebSearchMode::Live {
                    live_result = Some(result);
                }
            }
            Err(error) if skip_allowed_tool_error(&error) => {
                safe_skip(match tool_failure_code(&error) {
                    Some(WebToolErrorCode::AuthenticationRequired) => "authentication_unavailable",
                    Some(WebToolErrorCode::Timeout) => "provider_timeout",
                    Some(WebToolErrorCode::RateLimited) => "provider_limit",
                    _ => "provider_limit",
                });
                return;
            }
            Err(_) => panic!("live Codex search contract failed"),
        }
    }

    let live_result = live_result.expect("live-mode result must be retained");
    assert!(
        !live_result.references.is_empty() || !live_result.citations.is_empty(),
        "live search did not preserve any citation or reference"
    );
    let Some(reference) = live_result.references.first() else {
        safe_skip("provider_returned_citations_without_open_reference");
        return;
    };
    let open_client =
        WebSearchClient::new(&config(&model, CodexWebSearchMode::Live, 4), Some(auth))
            .expect("live Codex open client must satisfy the local contract");
    match open_client
        .run_commands(
            "",
            serde_json::json!({"open": [{"ref_id": reference.ref_id}]}),
            Some(vec!["openai.com".to_owned()]),
            Some(&context(&model, 4)),
        )
        .await
    {
        Ok(opened) => assert!(
            !opened.content.trim().is_empty(),
            "live open returned no content"
        ),
        Err(error) if skip_allowed_tool_error(&error) => {
            safe_skip("provider_limit");
        }
        Err(_) => panic!("live Codex reference-open contract failed"),
    }
}
