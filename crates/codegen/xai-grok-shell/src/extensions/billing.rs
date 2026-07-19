//! `x.ai/billing` extension handler.
//!
//! Fetches the authenticated user's Grok Build billing configuration
//! (credit limit, usage, on-demand cap, billing period, history) from
//! the backend. Used by the pager/desktop to display credits and usage.

use agent_client_protocol as acp;
use serde::{Deserialize, Serialize};

use super::{ExtResult, to_raw_response};
use crate::agent::MvpAgent;

/// Billing period cycle identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingCycle {
    pub year: i32,
    pub month: i32,
}

/// Cent value from the billing API (USD cents).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cent {
    /// proto3 JSON omits zero-valued scalars, so a `$0` Cent arrives as `{}`;
    /// default to 0 rather than failing the whole parse.
    #[serde(default)]
    pub val: i64,
}

/// A usage period (weekly or monthly) from the newer credits config.
///
/// `start`/`end` are RFC 3339 timestamps. `period_type` is the proto enum name
/// (e.g. `USAGE_PERIOD_TYPE_WEEKLY`); kept so callers can distinguish weekly
/// vs monthly cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsagePeriod {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub period_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// Usage summary for one past billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingPeriodUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_cycle: Option<BillingCycle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_used: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_demand_used: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_used: Option<Cent>,
}

/// Current billing configuration for Grok Build coding credits.
///
/// Carries both the newer credits-config fields (`credit_usage_percent`,
/// `current_period`) and the deprecated `GrokBuildBillingConfig` fields
/// (`monthly_limit`, `used`, `billing_period_*`). Consumers should prefer the
/// new fields and fall back to the deprecated ones, so the same struct works
/// against both the new `GetGrokCreditsConfig` and the legacy
/// `GetGrokBuildBillingConfig` backend responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingConfig {
    /// Included credit usage as a percentage of the allowance (0.0–100.0).
    /// Preferred over deriving from `monthly_limit`/`used`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_usage_percent: Option<f64>,
    /// Current usage period (weekly or monthly). Preferred over
    /// `billing_period_start`/`billing_period_end`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_period: Option<UsagePeriod>,
    /// Deprecated: included monthly credit budget. Use `credit_usage_percent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_limit: Option<Cent>,
    /// Deprecated: credits used this period. Use `credit_usage_percent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_demand_cap: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_demand_used: Option<Cent>,
    /// Remaining prepaid (purchased) credit balance, positive — the "bought
    /// credits" the user has topped up. Populated from the credits config
    /// (`GetGrokCreditsConfig.prepaid_balance`); absent in the legacy billing
    /// shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepaid_balance: Option<Cent>,
    /// Whether this user is on unified usage billing (shared weekly/monthly
    /// pool). From `GrokCreditsConfig.is_unified_billing_user`, which billing
    /// sets from remote settings `unified_consumer_billing_enabled`. `None` when
    /// absent (legacy `GetGrokBuildBillingConfig` shape or older servers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_unified_billing_user: Option<bool>,
    /// Deprecated: use `current_period.start`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_period_start: Option<String>,
    /// Deprecated: use `current_period.end`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_period_end: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<BillingPeriodUsage>,
}

/// Top-level response (primarily from `GET /rest/grok/credits` + auto-topup-rule).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfigResponse {
    pub config: Option<BillingConfig>,
    /// Authoritative ChatGPT Codex subscription limits for the selected Codex
    /// provider or a Codex-backed session. Never populated on the xAI path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_usage: Option<crate::auth::codex::CodexUsageSnapshot>,
    /// Authoritative Kimi Code plan limits for a Kimi-backed session. Never
    /// populated on xAI or Codex paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kimi_usage: Option<crate::auth::kimi_code::KimiCodeUsageSnapshot>,
    /// Authoritative Z.AI GLM Coding Plan quota and MCP usage for a Z.AI-backed
    /// session. Never populated on xAI, Codex, or Kimi paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zai_usage: Option<crate::auth::zai_coding_plan::ZaiCodingPlanUsageSnapshot>,
    /// Informational API-price comparison from actual session token counters.
    /// This is explicitly not ChatGPT subscription spend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_api_equivalent_cost: Option<crate::auth::codex::CodexApiEquivalentCostEstimate>,
    /// Whether on-demand credit usage is enabled. When `false`, the pager
    /// should hide on-demand controls. Populated from `RemoteSettings`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_demand_enabled: Option<bool>,
    /// User-friendly subscription tier name (e.g. "SuperGrok Heavy").
    /// Populated from `RemoteSettings` so the pager can update its cached
    /// tier on every billing fetch without an extra request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_tier: Option<String>,
}

/// Auto top-up configuration (from GetAutoTopupRule).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTopupRule {
    /// proto3 JSON omits `false`, so a disabled rule arrives without this field;
    /// default to `false` rather than failing the parse (which would otherwise
    /// keep a stale cached rule in the pager).
    #[serde(default)]
    pub enabled: bool,
    pub min_before_hitting_sl: Option<Cent>,
    pub topup_amount: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_amount_per_month: Option<Cent>,
}

/// Wrapper for the auto top-up rule response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAutoTopupRuleResponse {
    #[serde(default)]
    pub rule: Option<AutoTopupRule>,
}

#[tracing::instrument(skip_all, fields(method = %args.method))]
pub async fn handle(agent: &MvpAgent, args: &acp::ExtRequest) -> ExtResult {
    match args.method.as_ref() {
        "x.ai/billing" => {
            tracing::info!("handling billing config request");
            handle_get_billing(agent, args).await
        }
        "x.ai/auto-topup-rule" => {
            tracing::info!("handling auto top-up rule request");
            handle_get_auto_topup_rule(agent).await
        }
        _ => Err(acp::Error::method_not_found()),
    }
}

/// Structured context for unified-log entries from a successful billing fetch.
///
/// Keeps history to a count + the most recent period so `~/.grok/logs/unified.jsonl`
/// stays useful without dumping unbounded period arrays.
fn billing_unified_log_ctx(billing: &BillingConfigResponse) -> serde_json::Value {
    let history_len = billing
        .config
        .as_ref()
        .map(|c| c.history.len())
        .unwrap_or(0);
    let latest_history = billing
        .config
        .as_ref()
        .and_then(|c| c.history.last())
        .and_then(|p| serde_json::to_value(p).ok());

    let mut config_value = billing
        .config
        .as_ref()
        .and_then(|c| serde_json::to_value(c).ok())
        .unwrap_or(serde_json::Value::Null);
    if let Some(obj) = config_value.as_object_mut() {
        // Drop full history array; surface length + latest entry instead.
        obj.remove("history");
        obj.insert("historyLen".into(), serde_json::json!(history_len));
        if let Some(latest) = latest_history {
            obj.insert("latestHistory".into(), latest);
        }
    }

    serde_json::json!({
        "config": config_value,
        "onDemandEnabled": billing.on_demand_enabled,
        "subscriptionTier": billing.subscription_tier,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BillingRequest {
    #[serde(default)]
    session_id: Option<acp::SessionId>,
    /// Background callers may use stale same-record data while the usage
    /// endpoint is backing off. Explicit `/usage` bypasses backoff unless its
    /// one-minute cache entry is still fresh.
    #[serde(default)]
    silent: bool,
}

fn billing_provider(
    session_provider: Option<xai_grok_sampling_types::ProviderId>,
    configured_provider: xai_grok_sampling_types::ProviderId,
) -> xai_grok_sampling_types::ProviderId {
    session_provider.unwrap_or(configured_provider)
}

fn zai_usage_binding_is_current(
    expected: &xai_grok_sampling_types::CredentialBinding,
    current: &xai_grok_sampling_types::CredentialBinding,
) -> bool {
    expected.provider == xai_grok_sampling_types::ProviderId::ZaiCodingPlan
        && expected.source == xai_grok_sampling_types::CredentialSourceId::ZaiCodingPlanApiKey
        && expected.same_record(current)
        && current.generation >= expected.generation
}

async fn handle_get_billing(agent: &MvpAgent, args: &acp::ExtRequest) -> ExtResult {
    let request: BillingRequest = super::parse_params(args)?;
    let session = match request.session_id.as_ref() {
        Some(session_id) => Some(
            agent
                .sessions
                .borrow()
                .get(session_id)
                .cloned()
                .ok_or_else(|| acp::Error::invalid_params().data("unknown billing session"))?,
        ),
        None => None,
    };
    let session_sampling = match session.as_ref() {
        Some(session) => session.chat_state_handle.get_sampling_config().await,
        None => None,
    };
    let configured_provider = agent.sampling_config.borrow().provider;
    let provider = billing_provider(
        session_sampling.as_ref().map(|sampling| sampling.provider),
        configured_provider,
    );

    if provider == xai_grok_sampling_types::ProviderId::OpenAiCodex {
        let manager =
            crate::auth::codex::CodexAuthManager::new(&crate::util::grok_home::grok_home())
                .map_err(|error| {
                    acp::Error::auth_required()
                        .data(format!("OpenAI Codex usage unavailable: {error}"))
                })?;
        let codex_usage_result = match session_sampling.as_ref() {
            Some(sampling) => {
                let expected_binding = sampling.credential_binding.as_ref().ok_or_else(|| {
                    acp::Error::auth_required().data(
                        "OpenAI Codex session credential binding is unavailable; restart or select the model again",
                    )
                })?;
                crate::auth::codex::fetch_codex_usage_for_session(
                    &manager,
                    expected_binding,
                    request.silent,
                )
                .await
            }
            None => {
                crate::auth::codex::fetch_codex_usage_for_current(&manager, request.silent).await
            }
        };
        let codex_usage = codex_usage_result.map_err(|error| {
            // The usage client never includes an upstream body, token, account
            // ID, or credential-record ID in this display-safe error.
            let display = error.to_string();
            match error {
                crate::auth::codex::CodexUsageError::Auth(
                    crate::auth::codex::CodexAuthError::NotLoggedIn,
                ) => acp::Error::auth_required().data(display),
                crate::auth::codex::CodexUsageError::Auth(
                    crate::auth::codex::CodexAuthError::AccountChanged,
                ) => acp::Error::auth_required().data(format!(
                    "OpenAI Codex usage authentication changed: {display}"
                )),
                _ => acp::Error::internal_error()
                    .data(format!("Failed to fetch OpenAI Codex usage: {display}")),
            }
        })?;
        let codex_api_equivalent_cost = match (session.as_ref(), session_sampling.as_ref()) {
            (Some(session), Some(sampling)) => session
                .chat_state_handle
                .try_get_session_usage()
                .await
                .ok()
                .map(|ledger| {
                    crate::auth::codex::estimate_api_equivalent_cost(
                        &ledger,
                        &format!("openai_codex::{}", sampling.model),
                    )
                }),
            _ => None,
        };

        // Do not log the usage snapshot, account identity, or API-equivalent
        // estimate. The response is rendered only for the requesting client.
        tracing::info!("billing: fetched OpenAI Codex subscription limits");
        return to_raw_response(&BillingConfigResponse {
            config: None,
            codex_usage: Some(codex_usage),
            kimi_usage: None,
            zai_usage: None,
            codex_api_equivalent_cost,
            on_demand_enabled: None,
            subscription_tier: None,
        });
    }

    if provider == xai_grok_sampling_types::ProviderId::KimiCode {
        let grok_home = crate::util::grok_home::grok_home();
        let (credentials, current_binding) =
            crate::auth::kimi_code::current_credentials_and_binding(&grok_home).map_err(
                |error| {
                    acp::Error::auth_required()
                        .data(format!("Kimi Code usage unavailable: {error}"))
                },
            )?;
        if let Some(sampling) = session_sampling.as_ref() {
            let expected = sampling.credential_binding.as_ref().ok_or_else(|| {
                acp::Error::auth_required().data(
                    "Kimi Code session credential binding is unavailable; restart or select the model again",
                )
            })?;
            let valid = expected.provider == xai_grok_sampling_types::ProviderId::KimiCode
                && expected.source == xai_grok_sampling_types::CredentialSourceId::KimiCodeApiKey
                && expected.same_record(&current_binding)
                && current_binding.generation >= expected.generation;
            if !valid {
                return Err(acp::Error::auth_required().data(
                    "Kimi Code usage authentication changed; restart or select the model again",
                ));
            }
        }
        let kimi_usage = crate::auth::kimi_code::fetch_usage(&credentials)
            .await
            .map_err(|error| {
                let display = error.to_string();
                if matches!(
                    error,
                    crate::auth::kimi_code::KimiCodeAuthError::Unavailable
                        | crate::auth::kimi_code::KimiCodeAuthError::InvalidCredential
                        | crate::auth::kimi_code::KimiCodeAuthError::Http(
                            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN
                        )
                ) {
                    acp::Error::auth_required()
                        .data(format!("Kimi Code usage authentication failed: {display}"))
                } else {
                    acp::Error::internal_error()
                        .data(format!("Failed to fetch Kimi Code usage: {display}"))
                }
            })?;
        tracing::info!("billing: fetched Kimi Code plan limits");
        return to_raw_response(&BillingConfigResponse {
            config: None,
            codex_usage: None,
            kimi_usage: Some(kimi_usage),
            zai_usage: None,
            codex_api_equivalent_cost: None,
            on_demand_enabled: None,
            subscription_tier: None,
        });
    }

    if provider == xai_grok_sampling_types::ProviderId::ZaiCodingPlan {
        let grok_home = crate::util::grok_home::grok_home();
        let (credentials, current_binding) =
            crate::auth::zai_coding_plan::current_credentials_and_binding(&grok_home).map_err(
                |error| {
                    acp::Error::auth_required()
                        .data(format!("Z.AI Coding Plan usage unavailable: {error}"))
                },
            )?;
        if let Some(sampling) = session_sampling.as_ref() {
            let expected = sampling.credential_binding.as_ref().ok_or_else(|| {
                acp::Error::auth_required().data(
                    "Z.AI Coding Plan session credential binding is unavailable; restart or select the model again",
                )
            })?;
            if !zai_usage_binding_is_current(expected, &current_binding) {
                return Err(acp::Error::auth_required().data(
                    "Z.AI Coding Plan usage authentication changed; restart or select the model again",
                ));
            }
        }
        let zai_usage = crate::auth::zai_coding_plan::fetch_usage(&credentials)
            .await
            .map_err(|error| {
                let display = error.to_string();
                if matches!(
                    error,
                    crate::auth::zai_coding_plan::ZaiCodingPlanAuthError::Unavailable
                        | crate::auth::zai_coding_plan::ZaiCodingPlanAuthError::InvalidCredential
                        | crate::auth::zai_coding_plan::ZaiCodingPlanAuthError::Http(
                            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN
                        )
                        | crate::auth::zai_coding_plan::ZaiCodingPlanAuthError::Business(
                            1001..=1004
                        )
                ) {
                    acp::Error::auth_required().data(format!(
                        "Z.AI Coding Plan usage authentication failed: {display}"
                    ))
                } else {
                    acp::Error::internal_error()
                        .data(format!("Failed to fetch Z.AI Coding Plan usage: {display}"))
                }
            })?;
        tracing::info!("billing: fetched Z.AI Coding Plan limits");
        return to_raw_response(&BillingConfigResponse {
            config: None,
            codex_usage: None,
            kimi_usage: None,
            zai_usage: Some(zai_usage),
            codex_api_equivalent_cost: None,
            on_demand_enabled: None,
            subscription_tier: None,
        });
    }

    let auth = super::auth_gate::require_xai_auth(
        &agent.auth_manager,
        "Authentication required to fetch billing data",
        "Billing data requires auth with grok.com. Run `grok login` to authenticate.",
    )?;

    let proxy_base = agent.cli_chat_proxy_base_url();
    let base = proxy_base.trim_end_matches('/');

    // Credits balance / usage (new billing system) via the CLI proxy, which
    // forwards to the backend `GetGrokCreditsConfig`.
    let credits_url = format!("{}/billing?format=credits", base);
    let credits_resp = crate::http::shared_client()
        .get(&credits_url)
        .header("Authorization", format!("Bearer {}", &auth.key))
        .header(
            "X-XAI-Token-Auth",
            crate::auth::GrokComConfig::default().token_header,
        )
        .header("x-userid", &auth.user_id)
        .header("x-grok-client-version", xai_grok_version::VERSION)
        .header(
            crate::http::CLIENT_MODE_HEADER,
            crate::http::process_client_mode(),
        )
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "billing: upstream request failed");
            xai_grok_telemetry::unified_log::warn(
                "billing: upstream request failed",
                None,
                Some(serde_json::json!({ "error": e.to_string() })),
            );
            acp::Error::internal_error().data(format!("Failed to fetch billing data: {e}"))
        })?;

    if !credits_resp.status().is_success() {
        let status = credits_resp.status().as_u16();
        let body = credits_resp.text().await.unwrap_or_default();
        tracing::warn!(status, url = %credits_url, "billing: upstream error");

        let detail = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
            .unwrap_or_else(|| format!("HTTP {status}"));

        xai_grok_telemetry::unified_log::warn(
            "billing: upstream error",
            None,
            Some(serde_json::json!({
                "status": status,
                "detail": detail,
            })),
        );

        return Err(acp::Error::internal_error().data(format!("Billing service error: {detail}")));
    }

    let mut billing: BillingConfigResponse = credits_resp.json().await.map_err(|e| {
        tracing::error!(error = %e, "billing: failed to parse response");
        xai_grok_telemetry::unified_log::warn(
            "billing: failed to parse response",
            None,
            Some(serde_json::json!({ "error": e.to_string() })),
        );
        acp::Error::internal_error().data(format!("Failed to parse billing data: {e}"))
    })?;

    // Enrich with fields from remote settings.
    let rs = agent.cfg.borrow().remote_settings.clone();
    billing.on_demand_enabled = rs.as_ref().and_then(|rs| rs.on_demand_enabled);
    billing.subscription_tier = rs.as_ref().and_then(|rs| {
        rs.subscription_tier_display
            .clone()
            .or_else(|| rs.subscription_tier.clone())
    });

    // Every prompt / /usage / poll path hits `x.ai/billing`; log the fetched
    // credits snapshot so support can correlate limit UX with real balances.
    xai_grok_telemetry::unified_log::info(
        "billing: fetched credits config",
        None,
        Some(billing_unified_log_ctx(&billing)),
    );

    to_raw_response(&billing)
}

async fn handle_get_auto_topup_rule(agent: &MvpAgent) -> ExtResult {
    let auth = super::auth_gate::require_xai_auth(
        &agent.auth_manager,
        "Authentication required to fetch auto top-up rule",
        "Auto top-up data requires auth with grok.com. Run `grok login` to authenticate.",
    )?;

    let proxy_base = agent.cli_chat_proxy_base_url();
    let base = proxy_base.trim_end_matches('/');

    // Auto top-up rule via the CLI proxy, which forwards to the backend
    // `GetAutoTopupRule`.
    let url = format!("{}/auto-topup-rule", base);
    let response = crate::http::shared_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", &auth.key))
        .header(
            "X-XAI-Token-Auth",
            crate::auth::GrokComConfig::default().token_header,
        )
        .header("x-userid", &auth.user_id)
        .header("x-grok-client-version", xai_grok_version::VERSION)
        .header(
            crate::http::CLIENT_MODE_HEADER,
            crate::http::process_client_mode(),
        )
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "auto-topup: upstream request failed");
            acp::Error::internal_error().data(format!("Failed to fetch auto top-up rule: {e}"))
        })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        tracing::warn!(status, url = %url, "auto-topup: upstream error");

        let detail = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
            .unwrap_or_else(|| format!("HTTP {status}"));

        return Err(
            acp::Error::internal_error().data(format!("Auto top-up service error: {detail}"))
        );
    }

    // Return the upstream response body verbatim (as a JSON value) so /usage
    // can print the exact data from this request unformatted.
    let body_text = response.text().await.unwrap_or_default();
    let value: serde_json::Value =
        serde_json::from_str(&body_text).unwrap_or(serde_json::json!({"raw": body_text}));
    to_raw_response(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zai_usage_binding_accepts_only_the_pinned_record_and_generation() {
        use xai_grok_sampling_types::{CredentialBinding, CredentialSourceId, ProviderId};

        let mut expected = CredentialBinding::zai_coding_plan(Some("zai-record".to_owned()));
        expected.generation = 3;
        let mut current = expected.clone();
        assert!(zai_usage_binding_is_current(&expected, &current));

        current.generation = 4;
        assert!(zai_usage_binding_is_current(&expected, &current));

        current.record_id = Some("different-record".to_owned());
        assert!(!zai_usage_binding_is_current(&expected, &current));

        let mut stale = expected.clone();
        stale.generation = 2;
        assert!(!zai_usage_binding_is_current(&expected, &stale));

        let foreign = CredentialBinding {
            provider: ProviderId::KimiCode,
            source: CredentialSourceId::KimiCodeApiKey,
            record_id: expected.record_id.clone(),
            generation: expected.generation,
        };
        assert!(!zai_usage_binding_is_current(&expected, &foreign));
    }

    #[test]
    fn billing_request_defaults_to_explicit_and_accepts_silent_polling() {
        let explicit: BillingRequest = serde_json::from_value(serde_json::json!({
            "sessionId": "session-1"
        }))
        .unwrap();
        assert!(!explicit.silent);
        assert!(explicit.session_id.is_some());

        let silent: BillingRequest = serde_json::from_value(serde_json::json!({
            "sessionId": "session-1",
            "silent": true
        }))
        .unwrap();
        assert!(silent.silent);
    }

    #[test]
    fn no_session_billing_keeps_the_explicit_agent_provider() {
        assert_eq!(
            billing_provider(None, xai_grok_sampling_types::ProviderId::OpenAiCodex),
            xai_grok_sampling_types::ProviderId::OpenAiCodex,
            "Codex startup billing must not fall through to xAI"
        );
        assert_eq!(
            billing_provider(None, xai_grok_sampling_types::ProviderId::Xai),
            xai_grok_sampling_types::ProviderId::Xai
        );
    }

    #[test]
    fn auto_topup_disabled_rule_omits_enabled_field() {
        // proto3 JSON omits `false` / `0`, so a disabled rule arrives without
        // `enabled` (and zero Cents as `{}`). It must still deserialize (as
        // disabled) rather than erroring — otherwise the pager keeps a stale
        // cached rule.
        let json = serde_json::json!({
            "rule": { "topupAmount": {"val": 500}, "minBeforeHittingSl": {} }
        });
        let resp: GetAutoTopupRuleResponse = serde_json::from_value(json).unwrap();
        let rule = resp.rule.expect("rule present");
        assert!(!rule.enabled);
        assert_eq!(rule.topup_amount.unwrap().val, 500);
        assert_eq!(rule.min_before_hitting_sl.unwrap().val, 0);
    }

    #[test]
    fn billing_config_response_deserializes_from_backend_json() {
        let json = serde_json::json!({
            "config": {
                "monthlyLimit": {"val": 2000},
                "used": {"val": 1234},
                "onDemandCap": {"val": 500},
                "billingPeriodStart": "2025-04-01T00:00:00Z",
                "billingPeriodEnd": "2025-05-01T00:00:00Z",
                "history": [
                    {
                        "billingCycle": {"year": 2025, "month": 3},
                        "includedUsed": {"val": 1800},
                        "onDemandUsed": {"val": 0},
                        "totalUsed": {"val": 1800}
                    }
                ]
            }
        });
        let resp: BillingConfigResponse = serde_json::from_value(json).unwrap();
        let config = resp.config.unwrap();
        assert_eq!(config.monthly_limit.unwrap().val, 2000);
        assert_eq!(config.used.unwrap().val, 1234);
        assert_eq!(config.on_demand_cap.unwrap().val, 500);
        assert_eq!(
            config.billing_period_start.as_deref(),
            Some("2025-04-01T00:00:00Z")
        );
        assert_eq!(config.history.len(), 1);
        let period = &config.history[0];
        let cycle = period.billing_cycle.as_ref().unwrap();
        assert_eq!(cycle.year, 2025);
        assert_eq!(cycle.month, 3);
        assert_eq!(period.included_used.as_ref().unwrap().val, 1800);
        assert_eq!(period.total_used.as_ref().unwrap().val, 1800);
    }

    #[test]
    fn billing_unified_log_ctx_includes_credits_and_collapses_history() {
        let resp = BillingConfigResponse {
            config: Some(BillingConfig {
                credit_usage_percent: Some(42.5),
                current_period: Some(UsagePeriod {
                    period_type: Some("USAGE_PERIOD_TYPE_WEEKLY".into()),
                    start: Some("2025-04-01T00:00:00Z".into()),
                    end: Some("2025-04-08T00:00:00Z".into()),
                }),
                monthly_limit: Some(Cent { val: 2000 }),
                used: Some(Cent { val: 850 }),
                on_demand_cap: Some(Cent { val: 500 }),
                on_demand_used: Some(Cent { val: 0 }),
                prepaid_balance: Some(Cent { val: 100 }),
                is_unified_billing_user: Some(true),
                billing_period_start: None,
                billing_period_end: None,
                history: vec![
                    BillingPeriodUsage {
                        billing_cycle: Some(BillingCycle {
                            year: 2025,
                            month: 2,
                        }),
                        included_used: Some(Cent { val: 1000 }),
                        on_demand_used: Some(Cent { val: 0 }),
                        total_used: Some(Cent { val: 1000 }),
                    },
                    BillingPeriodUsage {
                        billing_cycle: Some(BillingCycle {
                            year: 2025,
                            month: 3,
                        }),
                        included_used: Some(Cent { val: 1800 }),
                        on_demand_used: Some(Cent { val: 0 }),
                        total_used: Some(Cent { val: 1800 }),
                    },
                ],
            }),
            codex_usage: None,
            kimi_usage: None,
            zai_usage: None,
            codex_api_equivalent_cost: None,
            on_demand_enabled: Some(true),
            subscription_tier: Some("SuperGrok".into()),
        };
        let ctx = billing_unified_log_ctx(&resp);
        assert_eq!(ctx["onDemandEnabled"], true);
        assert_eq!(ctx["subscriptionTier"], "SuperGrok");
        let config = ctx["config"].as_object().expect("config object");
        assert!(
            config.get("history").is_none(),
            "full history must be collapsed"
        );
        assert_eq!(config["historyLen"], 2);
        assert_eq!(
            config["latestHistory"]["billingCycle"]["month"], 3,
            "latest history period retained"
        );
        assert_eq!(config["creditUsagePercent"], 42.5);
        assert_eq!(config["prepaidBalance"]["val"], 100);
    }

    #[test]
    fn billing_config_response_roundtrips_through_json() {
        let config = BillingConfig {
            credit_usage_percent: None,
            current_period: None,
            monthly_limit: Some(Cent { val: 5000 }),
            used: Some(Cent { val: 123 }),
            on_demand_cap: Some(Cent { val: 0 }),
            on_demand_used: Some(Cent { val: 50 }),
            prepaid_balance: Some(Cent { val: 750 }),
            is_unified_billing_user: None,
            billing_period_start: Some("2025-04-01T00:00:00Z".to_string()),
            billing_period_end: Some("2025-05-01T00:00:00Z".to_string()),
            history: vec![BillingPeriodUsage {
                billing_cycle: Some(BillingCycle {
                    year: 2025,
                    month: 3,
                }),
                included_used: Some(Cent { val: 4500 }),
                on_demand_used: Some(Cent { val: 100 }),
                total_used: Some(Cent { val: 4600 }),
            }],
        };
        let resp = BillingConfigResponse {
            config: Some(config),
            codex_usage: None,
            kimi_usage: None,
            zai_usage: None,
            codex_api_equivalent_cost: None,
            on_demand_enabled: None,
            subscription_tier: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        let roundtripped: BillingConfigResponse = serde_json::from_value(json).unwrap();
        let rt_config = roundtripped.config.unwrap();
        assert_eq!(rt_config.monthly_limit.unwrap().val, 5000);
        assert_eq!(rt_config.used.unwrap().val, 123);
        assert_eq!(rt_config.prepaid_balance.unwrap().val, 750);
        assert_eq!(rt_config.history.len(), 1);
    }

    #[test]
    fn billing_config_response_handles_null_config() {
        let json = serde_json::json!({"config": null});
        let resp: BillingConfigResponse = serde_json::from_value(json).unwrap();
        assert!(resp.config.is_none());
    }

    #[test]
    fn billing_config_response_handles_empty_history() {
        let json = serde_json::json!({
            "config": {
                "monthlyLimit": {"val": 1000},
                "used": {"val": 0}
            }
        });
        let resp: BillingConfigResponse = serde_json::from_value(json).unwrap();
        let config = resp.config.unwrap();
        assert_eq!(config.monthly_limit.unwrap().val, 1000);
        assert!(config.history.is_empty());
    }

    #[test]
    fn billing_config_serializes_camel_case() {
        let config = BillingConfig {
            credit_usage_percent: None,
            current_period: None,
            monthly_limit: Some(Cent { val: 100 }),
            used: None,
            on_demand_cap: None,
            on_demand_used: None,
            prepaid_balance: None,
            is_unified_billing_user: None,
            billing_period_start: None,
            billing_period_end: None,
            history: vec![],
        };
        let json = serde_json::to_value(&config).unwrap();
        assert!(json.get("monthlyLimit").is_some());
        // Fields with None are skipped
        assert!(json.get("creditUsagePercent").is_none());
        assert!(json.get("currentPeriod").is_none());
        assert!(json.get("used").is_none());
        assert!(json.get("onDemandCap").is_none());
        assert!(json.get("onDemandUsed").is_none());
        assert!(json.get("prepaidBalance").is_none());
        assert!(json.get("billingPeriodStart").is_none());
        // Empty history is skipped
        assert!(json.get("history").is_none());
    }

    #[test]
    fn billing_config_deserializes_credits_config_shape() {
        // Newer `GetGrokCreditsConfig` response: percentage-based usage,
        // a typed current period, and history keyed by `period`.
        let json = serde_json::json!({
            "config": {
                "creditUsagePercent": 42.5,
                "currentPeriod": {
                    "type": "USAGE_PERIOD_TYPE_WEEKLY",
                    "start": "2026-06-01T00:00:00Z",
                    "end": "2026-06-08T00:00:00Z"
                },
                "onDemandCap": {"val": 5000},
                "onDemandUsed": {"val": 300},
                "prepaidBalance": {"val": 1250},
                "isUnifiedBillingUser": true,
                "productUsage": [
                    {"product": "PRODUCT_GROK_BUILD", "usagePercent": 61.2}
                ],
                "history": [
                    {
                        "period": {
                            "type": "USAGE_PERIOD_TYPE_WEEKLY",
                            "start": "2026-05-25T00:00:00Z",
                            "end": "2026-06-01T00:00:00Z"
                        },
                        "onDemandUsed": {"val": 120}
                    }
                ]
            }
        });
        let resp: BillingConfigResponse = serde_json::from_value(json).unwrap();
        let config = resp.config.unwrap();
        assert_eq!(config.credit_usage_percent, Some(42.5));
        let period = config.current_period.as_ref().unwrap();
        assert_eq!(
            period.period_type.as_deref(),
            Some("USAGE_PERIOD_TYPE_WEEKLY")
        );
        assert_eq!(period.end.as_deref(), Some("2026-06-08T00:00:00Z"));
        // Deprecated fields are absent in the credits shape.
        assert!(config.monthly_limit.is_none());
        assert!(config.billing_period_end.is_none());
        assert_eq!(config.on_demand_cap.unwrap().val, 5000);
        assert_eq!(config.on_demand_used.unwrap().val, 300);
        // Bought (prepaid) credit balance is parsed from the credits config.
        assert_eq!(config.prepaid_balance.unwrap().val, 1250);
        assert_eq!(config.is_unified_billing_user, Some(true));
        // productUsage is still unused by the CLI billing surface.
        assert_eq!(config.history.len(), 1);
        assert_eq!(config.history[0].on_demand_used.as_ref().unwrap().val, 120);
    }

    #[test]
    fn cent_serializes_as_val_field() {
        let c = Cent { val: 4299 };
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json, serde_json::json!({"val": 4299}));
    }
}
