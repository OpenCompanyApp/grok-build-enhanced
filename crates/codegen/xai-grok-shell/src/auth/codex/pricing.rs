//! Hypothetical API-equivalent cost for locally observed Codex session tokens.
//!
//! ChatGPT Codex subscription usage is not metered or billed by these values.
//! Rates are an informational comparison against published OpenAI API prices.

use serde::{Deserialize, Serialize};

/// Source date for the small, explicit rate table below. Pricing is not read
/// from the ChatGPT usage response and must never be presented as subscription
/// spend. Rates are the Standard tier from
/// <https://developers.openai.com/api/docs/pricing>.
pub const OPENAI_API_PRICING_VERIFIED_AT: &str = "2026-07-17";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexApiEquivalentCostLine {
    pub model_id: String,
    pub pricing_basis_model: String,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub uncached_input_tokens: u64,
    pub output_tokens: u64,
    pub input_usd_per_million: f64,
    pub cached_input_usd_per_million: f64,
    pub output_usd_per_million: f64,
    pub estimated_usd: f64,
}

/// A deliberately non-authoritative API comparison built from the session's
/// real provider-reported token ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexApiEquivalentCostEstimate {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub uncached_input_tokens: u64,
    pub output_tokens: u64,
    /// Present when every locally observed model has a published rate. When
    /// `usage_incomplete` is true this is an observed lower bound because
    /// auxiliary/compaction calls were not available to the ledger. Absence
    /// never means zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<CodexApiEquivalentCostLine>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unpriced_models: Vec<String>,
    pub usage_incomplete: bool,
    pub pricing_verified_at: String,
}

#[derive(Clone, Copy)]
struct PublishedRate {
    basis_model: &'static str,
    input: f64,
    cached_input: f64,
    output: f64,
}

/// Current published standard API token rates in USD per million tokens.
///
/// Exact matching is intentional. A future catalog alias is not assumed to
/// have the same price until OpenAI publishes that relationship. The one
/// exception is `gpt-5.6`: the official Sol model page explicitly documents it
/// as an alias of `gpt-5.6-sol`.
fn published_rate(model_id: &str) -> Option<PublishedRate> {
    let normalized = model_id.trim().to_ascii_lowercase();
    // Ledger identities are provider-qualified at record time. Requiring the
    // Codex namespace here prevents a custom/xAI model that reuses a published
    // OpenAI slug from being assigned OpenAI API prices after model switching.
    let normalized = normalized.strip_prefix("openai_codex::")?;
    // ACP catalog keys carry the explicit provider namespace while the
    // sampling ledger normally receives the provider's routing slug. They are
    // the same model identity, not a pricing alias.
    let model_id = normalized
        .strip_prefix("openai-codex/")
        .unwrap_or(normalized);
    match model_id {
        "gpt-5.6" | "gpt-5.6-sol" => Some(PublishedRate {
            basis_model: "gpt-5.6-sol",
            input: 5.0,
            cached_input: 0.5,
            output: 30.0,
        }),
        "gpt-5.6-terra" => Some(PublishedRate {
            basis_model: "gpt-5.6-terra",
            input: 2.5,
            cached_input: 0.25,
            output: 15.0,
        }),
        "gpt-5.6-luna" => Some(PublishedRate {
            basis_model: "gpt-5.6-luna",
            input: 1.0,
            cached_input: 0.1,
            output: 6.0,
        }),
        _ => None,
    }
}

fn estimate_line(
    model_id: &str,
    totals: &xai_chat_state::UsageTotals,
    rate: PublishedRate,
) -> CodexApiEquivalentCostLine {
    let display_model_id = model_id.strip_prefix("openai_codex::").unwrap_or(model_id);
    let display_model_id = display_model_id
        .strip_prefix("openai-codex/")
        .unwrap_or(display_model_id);
    let cached_input_tokens = totals.cached_read_tokens.min(totals.input_tokens);
    let uncached_input_tokens = totals.input_tokens.saturating_sub(cached_input_tokens);
    let estimated_usd = uncached_input_tokens as f64 / 1_000_000.0 * rate.input
        + cached_input_tokens as f64 / 1_000_000.0 * rate.cached_input
        + totals.output_tokens as f64 / 1_000_000.0 * rate.output;
    CodexApiEquivalentCostLine {
        model_id: display_model_id.to_owned(),
        pricing_basis_model: rate.basis_model.to_owned(),
        input_tokens: totals.input_tokens,
        cached_input_tokens,
        uncached_input_tokens,
        output_tokens: totals.output_tokens,
        input_usd_per_million: rate.input,
        cached_input_usd_per_million: rate.cached_input,
        output_usd_per_million: rate.output,
        estimated_usd,
    }
}

pub fn estimate_api_equivalent_cost(
    ledger: &xai_chat_state::UsageLedger,
    current_model_id: &str,
) -> CodexApiEquivalentCostEstimate {
    let cached_input_tokens = ledger
        .totals
        .cached_read_tokens
        .min(ledger.totals.input_tokens);
    let mut lines = Vec::new();
    let mut unpriced_models = Vec::new();

    if ledger.by_model.is_empty() {
        if let Some(rate) = published_rate(current_model_id) {
            lines.push(estimate_line(current_model_id, &ledger.totals, rate));
        } else if ledger.totals.input_tokens > 0 || ledger.totals.output_tokens > 0 {
            unpriced_models.push(current_model_id.to_owned());
        }
    } else {
        for (model_id, totals) in &ledger.by_model {
            match published_rate(model_id) {
                Some(rate) => lines.push(estimate_line(model_id, totals, rate)),
                None => unpriced_models.push(model_id.clone()),
            }
        }
    }

    let estimated_usd = (unpriced_models.is_empty() && !lines.is_empty())
        .then(|| lines.iter().map(|line| line.estimated_usd).sum());
    CodexApiEquivalentCostEstimate {
        input_tokens: ledger.totals.input_tokens,
        cached_input_tokens,
        uncached_input_tokens: ledger
            .totals
            .input_tokens
            .saturating_sub(cached_input_tokens),
        output_tokens: ledger.totals.output_tokens,
        estimated_usd,
        lines,
        unpriced_models,
        usage_incomplete: ledger.incomplete,
        pricing_verified_at: OPENAI_API_PRICING_VERIFIED_AT.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn totals(input: u64, cached: u64, output: u64) -> xai_chat_state::UsageTotals {
        xai_chat_state::UsageTotals {
            input_tokens: input,
            cached_read_tokens: cached,
            output_tokens: output,
            ..Default::default()
        }
    }

    #[test]
    fn sol_estimate_charges_cached_tokens_at_published_cached_rate() {
        let mut ledger = xai_chat_state::UsageLedger::default();
        ledger.totals = totals(1_000_000, 400_000, 100_000);
        ledger
            .by_model
            .insert("openai_codex::gpt-5.6".to_owned(), ledger.totals.clone());

        let estimate = estimate_api_equivalent_cost(&ledger, "openai_codex::gpt-5.6");
        assert_eq!(estimate.uncached_input_tokens, 600_000);
        assert_eq!(estimate.cached_input_tokens, 400_000);
        assert_eq!(estimate.lines[0].pricing_basis_model, "gpt-5.6-sol");
        assert_eq!(estimate.lines[0].cached_input_usd_per_million, 0.5);
        assert!((estimate.estimated_usd.unwrap() - 6.2).abs() < 1e-12);
    }

    #[test]
    fn luna_and_terra_use_only_their_published_rates() {
        let mut ledger = xai_chat_state::UsageLedger::default();
        ledger.by_model.insert(
            "openai_codex::gpt-5.6-luna".to_owned(),
            totals(1_000_000, 0, 1_000_000),
        );
        ledger.by_model.insert(
            "openai_codex::gpt-5.6-terra".to_owned(),
            totals(1_000_000, 0, 1_000_000),
        );
        ledger.totals = totals(2_000_000, 0, 2_000_000);

        let estimate = estimate_api_equivalent_cost(&ledger, "openai_codex::gpt-5.6-terra");
        assert!((estimate.estimated_usd.unwrap() - 24.5).abs() < 1e-12);
    }

    #[test]
    fn provider_qualified_catalog_key_uses_the_exact_slug_rate() {
        let mut ledger = xai_chat_state::UsageLedger::default();
        ledger.totals = totals(1_000_000, 0, 0);
        ledger.by_model.insert(
            "openai_codex::openai-codex/gpt-5.6-terra".to_owned(),
            ledger.totals.clone(),
        );

        let estimate =
            estimate_api_equivalent_cost(&ledger, "openai_codex::openai-codex/gpt-5.6-terra");
        assert_eq!(estimate.estimated_usd, Some(2.5));
        assert_eq!(estimate.lines[0].pricing_basis_model, "gpt-5.6-terra");
    }

    #[test]
    fn unpublished_alias_and_incomplete_usage_never_produce_a_total() {
        let mut ledger = xai_chat_state::UsageLedger::default();
        ledger.totals = totals(1_000, 500, 100);
        ledger.by_model.insert(
            "openai_codex::gpt-5.6-codex-future".to_owned(),
            ledger.totals.clone(),
        );

        let estimate = estimate_api_equivalent_cost(&ledger, "openai_codex::gpt-5.6-codex-future");
        assert_eq!(estimate.estimated_usd, None);
        assert_eq!(
            estimate.unpriced_models,
            ["openai_codex::gpt-5.6-codex-future"]
        );

        ledger.by_model.clear();
        ledger.by_model.insert(
            "openai_codex::gpt-5.6-sol".to_owned(),
            ledger.totals.clone(),
        );
        ledger.incomplete = true;
        let incomplete = estimate_api_equivalent_cost(&ledger, "openai_codex::gpt-5.6-sol");
        assert!(incomplete.estimated_usd.is_some());
        assert!(incomplete.usage_incomplete);

        let empty = xai_chat_state::UsageLedger::default();
        let empty_estimate = estimate_api_equivalent_cost(&empty, "unpublished-future-model");
        assert_eq!(empty_estimate.estimated_usd, None);
        assert!(empty_estimate.lines.is_empty());
    }

    #[test]
    fn non_codex_provider_slug_collision_is_never_priced() {
        for identity in ["xai::gpt-5.6-luna", "custom::gpt-5.6-luna"] {
            let mut ledger = xai_chat_state::UsageLedger::default();
            ledger.totals = totals(1_000_000, 0, 1_000_000);
            ledger
                .by_model
                .insert(identity.to_owned(), ledger.totals.clone());

            let estimate = estimate_api_equivalent_cost(&ledger, "openai_codex::gpt-5.6-luna");
            assert_eq!(estimate.estimated_usd, None);
            assert_eq!(estimate.unpriced_models, [identity]);
        }
    }
}
