//! Credit balance indicator for the agent status bar.
//!
//! Shows the user's coding credit usage as a compact status bar item.
//! Fetches real data from the `x.ai/billing` agent extension.

use chrono::TimeZone;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::theme::Theme;

/// Credit balance state from the billing API.
#[derive(Debug, Clone)]
pub struct CreditBalance {
    /// Usage as a percentage of the allowance (0.0–100.0).
    pub usage_pct: f64,
    /// Usage as a percentage of total budget (free + on-demand when enabled).
    pub effective_usage_pct: f64,
    /// Billing period end as a formatted local wall-clock string (no zone
    /// label), e.g. "Mar 31, 12:00".
    pub period_end_display: Option<String>,
    /// Whether pay-as-you-go (on-demand) billing is enabled.
    pub pay_as_you_go: bool,
    /// On-demand spending cap in USD cents (e.g. 500 = $5.00).
    pub on_demand_cap_cents: Option<i64>,
    /// On-demand usage this period in USD cents.
    pub on_demand_used_cents: Option<i64>,
    /// Remaining prepaid ("bought") credit balance in USD cents.
    pub prepaid_balance_cents: Option<i64>,
    /// Usage period type from the billing response (the proto enum name, e.g.
    /// `USAGE_PERIOD_TYPE_WEEKLY`). Drives the "Weekly/Monthly limit" label.
    pub period_type: Option<String>,
    /// From credits config `is_unified_billing_user` (`None` if absent).
    /// `Some(true)` = unified pool / buy-credits UX; `Some(false)` = legacy
    /// on-demand / PAYG UX.
    pub is_unified_billing_user: Option<bool>,
}

impl CreditBalance {
    /// Label for the percentage allowance, chosen from the period type:
    /// "Weekly limit" / "Monthly limit", falling back to "Usage" when unknown.
    pub fn usage_label(&self) -> &'static str {
        match self.period_type.as_deref() {
            Some(t) if t.contains("WEEKLY") => "Weekly limit",
            Some(t) if t.contains("MONTHLY") => "Monthly limit",
            _ => "Usage",
        }
    }
}

/// Auto top-up rule data used by the `/usage` summary.
#[derive(Debug, Clone)]
pub struct AutoTopupInfo {
    /// Whether auto top-up is enabled.
    pub enabled: bool,
    /// Per-trigger top-up amount in USD cents.
    pub topup_amount_cents: Option<i64>,
    /// Optional maximum monthly top-up amount in USD cents.
    pub max_amount_cents: Option<i64>,
}

impl AutoTopupInfo {
    /// A known "no / disabled auto top-up" state — distinct from an unresolved
    /// `None`, which means the rule hasn't been fetched yet.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            topup_amount_cents: None,
            max_amount_cents: None,
        }
    }
}

/// Outcome of an auto top-up rule fetch, so a transient failure doesn't clear a
/// previously cached rule.
#[derive(Debug, Clone)]
pub enum AutoTopupFetch {
    /// A definitive rule state (a real rule, or [`AutoTopupInfo::disabled`] when
    /// the backend reports none). Stored as the *known* auto top-up state.
    Resolved(AutoTopupInfo),
    /// Fetch failed — keep the cached value (last-known-good). A stored `None`
    /// therefore means "not yet known", not "no auto top-up".
    Unchanged,
    /// The rule is not applicable (no prepaid credits) — reset the cache to
    /// "unknown" so a later credits period doesn't read a stale rule.
    Cleared,
}

/// Format `cents` as a dollar string: whole dollars as `$N`, otherwise `$N.NN`.
fn fmt_dollars(cents: i64) -> String {
    let dollars = cents as f64 / 100.0;
    if dollars.fract() == 0.0 {
        format!("${dollars:.0}")
    } else {
        format!("${dollars:.2}")
    }
}

/// Build the `/usage` summary block shown in scrollback.
///
/// Always shows usage % and (when known) the next reset time. The credits
/// block is rendered only when the user has a positive prepaid balance:
/// - no prepaid balance       → credits block omitted entirely
/// - auto top-up off/unknown  → `Auto topup: disabled` (no max line)
/// - auto top-up on, no max   → `Auto topup: $N`
/// - auto top-up on, max set  → `Auto topup: $N` + `Max monthly topup: $M`
pub fn format_usage_summary(balance: &CreditBalance, autotopup: Option<&AutoTopupInfo>) -> String {
    // Floor to match the backend SpendingLimiter's `as u8` truncation
    // (99.994% → 99%, never 100% until truly exhausted).
    let mut lines = vec![format!(
        "{}: {}%",
        balance.usage_label(),
        balance.usage_pct.floor() as i64
    )];
    if let Some(reset) = &balance.period_end_display {
        lines.push(format!("Next reset: {reset}"));
    }

    // Billing stores credit / top-up amounts as negative cents (accounting
    // convention); display the absolute USD value, matching the web clients.
    if let Some(prepaid) = balance
        .prepaid_balance_cents
        .map(i64::abs)
        .filter(|c| *c > 0)
    {
        lines.push(String::new());
        lines.push(format!("Credits: {}", fmt_dollars(prepaid)));
        match autotopup {
            Some(at) if at.enabled && at.topup_amount_cents.is_some() => {
                lines.push(format!(
                    "Auto topup: {}",
                    fmt_dollars(at.topup_amount_cents.unwrap().abs())
                ));
                if let Some(max) = at.max_amount_cents {
                    lines.push(format!("Max monthly topup: {}", fmt_dollars(max.abs())));
                }
            }
            _ => lines.push("Auto topup: disabled".to_string()),
        }
    }

    // Legacy on-demand (pay-as-you-go) billing — shown only when enabled, for
    // users on the older monthly + on-demand model. Amounts always carry cents
    // (e.g. `$50.00`), matching the web client.
    if balance.pay_as_you_go {
        let used = balance.on_demand_used_cents.unwrap_or(0).abs() as f64 / 100.0;
        let cap = balance.on_demand_cap_cents.unwrap_or(0).abs() as f64 / 100.0;
        lines.push(String::new());
        lines.push(format!("Pay-as-you-go: ${used:.2} used of ${cap:.2} limit"));
    }

    lines.join("\n")
}

fn codex_window_label(seconds: i64) -> String {
    match seconds {
        18_000 => "5-hour limit".to_owned(),
        604_800 => "Weekly limit".to_owned(),
        seconds if seconds > 0 && seconds % 86_400 == 0 => {
            format!("{}-day limit", seconds / 86_400)
        }
        seconds if seconds > 0 && seconds % 3_600 == 0 => {
            format!("{}-hour limit", seconds / 3_600)
        }
        seconds => format!("{}-second limit", seconds.max(0)),
    }
}

fn codex_reset_display(window: &xai_grok_shell::auth::codex::CodexUsageWindow) -> String {
    chrono::Utc
        .timestamp_opt(window.reset_at, 0)
        .single()
        .map(|reset| {
            reset
                .with_timezone(&chrono::Local)
                .format("%B %-d, %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| format!("in {}s", window.reset_after_seconds.max(0)))
}

fn push_codex_window(
    lines: &mut Vec<String>,
    window: &xai_grok_shell::auth::codex::CodexUsageWindow,
) {
    lines.push(format!(
        "{}: {}% used",
        codex_window_label(window.limit_window_seconds),
        window.used_percent.floor() as i64
    ));
    lines.push(format!("Resets: {}", codex_reset_display(window)));
}

fn format_token_count(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(ch);
    }
    formatted
}

fn format_hypothetical_api_cost(
    estimate: Option<&xai_grok_shell::auth::codex::CodexApiEquivalentCostEstimate>,
) -> Vec<String> {
    let mut lines = vec!["Hypothetical API-equivalent cost (not subscription spend):".to_owned()];
    let Some(estimate) = estimate else {
        lines.push("Unavailable: session token counters could not be read.".to_owned());
        return lines;
    };

    lines.push(format!(
        "Observed session tokens: {} uncached input + {} cached input + {} output",
        format_token_count(estimate.uncached_input_tokens),
        format_token_count(estimate.cached_input_tokens),
        format_token_count(estimate.output_tokens),
    ));
    match estimate.estimated_usd {
        Some(cost) if estimate.usage_incomplete => {
            lines.push(format!("Observed API-cost lower bound: ${cost:.6}"));
            lines.push(
                "Incomplete ledger: unobserved auxiliary and compaction model calls are excluded."
                    .to_owned(),
            );
        }
        Some(cost) => lines.push(format!("Standard API comparison: ${cost:.6}")),
        None if !estimate.unpriced_models.is_empty() => lines.push(format!(
            "Unavailable: no published API rate is mapped for {}.",
            estimate.unpriced_models.join(", ")
        )),
        None => lines.push("Unavailable: no published API pricing basis.".to_owned()),
    }
    for line in &estimate.lines {
        lines.push(format!(
            "Pricing basis {}: ${:.2} input / ${:.2} cached / ${:.2} output per 1M tokens",
            line.pricing_basis_model,
            line.input_usd_per_million,
            line.cached_input_usd_per_million,
            line.output_usd_per_million,
        ));
    }
    lines.push(format!(
        "Rates verified {}. Excludes tool fees, cache-write premiums, and long-context multipliers.",
        estimate.pricing_verified_at
    ));
    lines
}

/// Format authoritative ChatGPT Codex subscription limits plus a separately
/// labeled, non-authoritative standard-API price comparison.
pub fn format_codex_usage_summary(
    usage: &xai_grok_shell::auth::codex::CodexUsageSnapshot,
    estimate: Option<&xai_grok_shell::auth::codex::CodexApiEquivalentCostEstimate>,
) -> String {
    let mut lines = vec!["OpenAI Codex subscription usage".to_owned()];
    if let Some(plan) = usage.plan_type.as_deref().filter(|plan| !plan.is_empty()) {
        lines.push(format!("Plan: {plan}"));
    }
    if let Some(limit) = &usage.rate_limit {
        if let Some(primary) = &limit.primary_window {
            push_codex_window(&mut lines, primary);
        }
        if let Some(secondary) = &limit.secondary_window {
            push_codex_window(&mut lines, secondary);
        }
        if limit.limit_reached {
            let reached = usage
                .rate_limit_reached_type
                .as_ref()
                .map(|value| value.kind.as_str())
                .unwrap_or("rate limit reached");
            lines.push(format!("Status: {reached}"));
        }
    } else {
        lines.push("Subscription windows: unavailable".to_owned());
    }

    if let Some(credits) = &usage.credits {
        if credits.unlimited {
            lines.push("Purchased credits: unlimited".to_owned());
        } else if let Some(balance) = credits.balance.as_deref() {
            lines.push(format!("Purchased credits balance: {balance}"));
        } else if credits.has_credits {
            lines.push("Purchased credits: available (balance unavailable)".to_owned());
        }
    }
    if let Some(reset_credits) = &usage.rate_limit_reset_credits {
        lines.push(format!(
            "Rate-limit reset credits available: {}",
            reset_credits.available_count.max(0)
        ));
    }
    if let Some(spend) = &usage.spend_control
        && let Some(limit) = &spend.individual_limit
    {
        lines.push(format!(
            "Workspace spend control: {}% used; {} remaining (provider units)",
            limit.used_percent.floor() as i64,
            limit.remaining
        ));
    }
    for additional in &usage.additional_rate_limits {
        if let Some(limit) = &additional.rate_limit
            && let Some(window) = &limit.primary_window
        {
            lines.push(format!(
                "Additional limit {}: {}% used ({})",
                additional.limit_name,
                window.used_percent.floor() as i64,
                codex_window_label(window.limit_window_seconds),
            ));
        }
    }

    lines.push(String::new());
    lines.extend(format_hypothetical_api_cost(estimate));
    lines.join("\n")
}

/// Format authoritative Kimi Code plan windows and optional Extra Usage wallet
/// data. API-equivalent costs remain intentionally absent because Kimi's
/// hosted-search pricing sources currently conflict.
pub fn format_kimi_usage_summary(
    usage: &xai_grok_shell::auth::kimi_code::KimiCodeUsageSnapshot,
) -> String {
    fn push_row(lines: &mut Vec<String>, row: &xai_grok_shell::auth::kimi_code::KimiCodeUsageRow) {
        let amount = if row.limit > 0 {
            format!(
                "{} / {} ({:.0}% used)",
                row.used,
                row.limit,
                (row.used.max(0) as f64 / row.limit as f64) * 100.0
            )
        } else {
            format!("{} used", row.used)
        };
        let reset = row
            .reset_hint
            .as_deref()
            .map(|hint| format!("; {hint}"))
            .unwrap_or_default();
        lines.push(format!("{}: {amount}{reset}", row.label));
    }

    let mut lines = vec!["Kimi Code plan usage".to_owned()];
    if let Some(summary) = &usage.summary {
        push_row(&mut lines, summary);
    }
    for limit in &usage.limits {
        push_row(&mut lines, limit);
    }
    if usage.summary.is_none() && usage.limits.is_empty() {
        lines.push("Plan windows: unavailable".to_owned());
    }
    if let Some(extra) = &usage.extra_usage {
        lines.push(format!(
            "Extra Usage balance: {} {:.2} / {:.2}",
            extra.currency,
            extra.balance_cents as f64 / 100.0,
            extra.total_cents as f64 / 100.0,
        ));
        if extra.monthly_charge_limit_enabled {
            lines.push(format!(
                "Extra Usage monthly spend: {} {:.2} / {:.2}",
                extra.currency,
                extra.monthly_used_cents as f64 / 100.0,
                extra.monthly_charge_limit_cents as f64 / 100.0,
            ));
        }
    }
    lines.push(String::new());
    lines.push(
        "API-equivalent cost: unavailable; Kimi plan usage and hosted-tool fees are separate."
            .to_owned(),
    );
    lines.join("\n")
}

/// Low-balance ($10) and pay-as-you-go critical ($5) warning thresholds, in cents.
const LOW_BALANCE_CENTS: i64 = 1000;
const PAY_AS_YOU_GO_CRITICAL_CENTS: i64 = 500;

/// The prompt's usage/credits warning as `(text, critical)`, or `None`
/// (`critical` = yellow, else grey; team users with `usage_visible = false`
/// never warn). Behaviour splits by billing model — prepaid credits,
/// pay-as-you-go on-demand, or the included-allowance percentage — with exact
/// thresholds and copy pinned by the unit tests.
///
/// Gateway light-frontend (`kind: "chat"`) sessions must not surface Build
/// coding-credit warnings — use [`usage_warning_for_session`] with
/// `gateway_chat = true` so the prompt shows no fake local sampler telemetry.
pub fn usage_warning(
    balance: &CreditBalance,
    autotopup: Option<&AutoTopupInfo>,
    usage_visible: bool,
) -> Option<(String, bool)> {
    usage_warning_for_session(balance, autotopup, usage_visible, false)
}

/// Like [`usage_warning`], but suppresses output for gateway/chat-kind sessions.
pub fn usage_warning_for_session(
    balance: &CreditBalance,
    autotopup: Option<&AutoTopupInfo>,
    usage_visible: bool,
    gateway_chat: bool,
) -> Option<(String, bool)> {
    if gateway_chat || !usage_visible {
        return None;
    }

    // A non-zero prepaid balance (stored as signed cents) means the credits model.
    let credits = balance
        .prepaid_balance_cents
        .map(i64::abs)
        .filter(|c| *c > 0);

    let Some(credits_cents) = credits else {
        // Pay-as-you-go (legacy on-demand): warn on dollars left in the cap once
        // the included allowance is spent.
        if balance.pay_as_you_go {
            if balance.usage_pct >= 100.0 {
                let cap = balance.on_demand_cap_cents.unwrap_or(0).abs();
                let used = balance.on_demand_used_cents.unwrap_or(0).abs();
                let remaining = (cap - used).max(0);
                if remaining <= LOW_BALANCE_CENTS {
                    let text = format!("Pay-as-you-go limit left: {}", fmt_dollars(remaining));
                    return Some((text, remaining <= PAY_AS_YOU_GO_CRITICAL_CENTS));
                }
            }
            return None;
        }

        let pct = balance.effective_usage_pct;
        if pct > 90.0 {
            // "Left" = complement of floored usage, so it agrees with the
            // floored summary (99.994% → "1% left", not "0%").
            let remaining = (100 - pct.floor() as i64).max(0);
            let label = balance.usage_label();
            return Some((format!("{label} left: {remaining}%"), pct > 95.0));
        }
        return None;
    };

    // Credits are only drawn down at 100% usage; don't warn before then.
    if balance.usage_pct < 100.0 {
        return None;
    }

    let credits_warning = || {
        (
            format!("Credits left: {}", fmt_dollars(credits_cents)),
            true,
        )
    };

    // Auto top-up gates the warning: unknown → silent; disabled → warn when low;
    // enabled w/o max → never; enabled w/ max → warn below one top-up amount.
    match autotopup {
        None => None,
        Some(at) if !at.enabled => (credits_cents <= LOW_BALANCE_CENTS).then(credits_warning),
        Some(at) if at.max_amount_cents.is_none() => None,
        Some(at) => at
            .topup_amount_cents
            .map(i64::abs)
            .and_then(|amt| (credits_cents < amt).then(credits_warning)),
    }
}

/// Build the credit balance indicator as a `Line<'static>`.
///
/// Shows `Credits used: XX%` in the status bar.
///
/// Gateway light-frontend (`kind: "chat"`) sessions must not show Build coding
/// credits — use [`credit_bar_line_for_session`] with `gateway_chat = true`
/// (returns `None`). remote settings / managed opt-in for chat entry can share the
/// same gate later; for now it only zeros/suppresses misleading local telemetry.
pub fn credit_bar_line(balance: &CreditBalance, hovered: bool, theme: &Theme) -> Line<'static> {
    credit_bar_line_for_session(balance, hovered, theme, false)
        .expect("non-chat credit_bar_line always renders")
}

/// Like [`credit_bar_line`], but returns `None` for gateway/chat-kind sessions
/// so the status bar never implies Build sampler / coding-credit usage.
pub fn credit_bar_line_for_session(
    balance: &CreditBalance,
    _hovered: bool,
    theme: &Theme,
    gateway_chat: bool,
) -> Option<Line<'static>> {
    if gateway_chat {
        return None;
    }
    let pct = balance.usage_pct;
    let color = if pct >= 100.0 {
        theme.accent_error
    } else if pct >= 80.0 {
        theme.warning
    } else {
        theme.accent_success
    };

    let text = format!("Credits used: {pct:.0}%");

    let style = Style::default().fg(color).bg(theme.bg_base);
    Some(Line::from(Span::styled(text, style)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bal(pct: f64) -> CreditBalance {
        CreditBalance {
            usage_pct: pct,
            effective_usage_pct: pct,
            period_end_display: None,
            pay_as_you_go: false,
            on_demand_cap_cents: None,
            on_demand_used_cents: None,
            prepaid_balance_cents: None,
            period_type: None,
            is_unified_billing_user: None,
        }
    }

    fn topup(enabled: bool, amount: Option<i64>, max: Option<i64>) -> AutoTopupInfo {
        AutoTopupInfo {
            enabled,
            topup_amount_cents: amount,
            max_amount_cents: max,
        }
    }

    fn codex_usage() -> xai_grok_shell::auth::codex::CodexUsageSnapshot {
        use xai_grok_shell::auth::codex::{
            CodexCreditStatus, CodexRateLimit, CodexResetCredits, CodexUsageSnapshot,
            CodexUsageWindow,
        };
        CodexUsageSnapshot {
            plan_type: Some("pro".to_owned()),
            rate_limit: Some(CodexRateLimit {
                allowed: true,
                limit_reached: false,
                primary_window: Some(CodexUsageWindow {
                    used_percent: 42.0,
                    limit_window_seconds: 18_000,
                    reset_after_seconds: 120,
                    reset_at: 1_735_689_720,
                }),
                secondary_window: Some(CodexUsageWindow {
                    used_percent: 5.0,
                    limit_window_seconds: 604_800,
                    reset_after_seconds: 43_200,
                    reset_at: 1_735_736_400,
                }),
            }),
            credits: Some(CodexCreditStatus {
                has_credits: true,
                unlimited: false,
                balance: Some("12.50".to_owned()),
            }),
            spend_control: None,
            additional_rate_limits: Vec::new(),
            rate_limit_reached_type: None,
            rate_limit_reset_credits: Some(CodexResetCredits { available_count: 3 }),
        }
    }

    #[test]
    fn codex_summary_separates_authoritative_limits_from_api_comparison() {
        use xai_grok_shell::auth::codex::{
            CodexApiEquivalentCostEstimate, CodexApiEquivalentCostLine,
        };
        let estimate = CodexApiEquivalentCostEstimate {
            input_tokens: 1_000_000,
            cached_input_tokens: 400_000,
            uncached_input_tokens: 600_000,
            output_tokens: 100_000,
            estimated_usd: Some(6.2),
            lines: vec![CodexApiEquivalentCostLine {
                model_id: "gpt-5.6".to_owned(),
                pricing_basis_model: "gpt-5.6-sol".to_owned(),
                input_tokens: 1_000_000,
                cached_input_tokens: 400_000,
                uncached_input_tokens: 600_000,
                output_tokens: 100_000,
                input_usd_per_million: 5.0,
                cached_input_usd_per_million: 0.5,
                output_usd_per_million: 30.0,
                estimated_usd: 6.2,
            }],
            unpriced_models: Vec::new(),
            usage_incomplete: false,
            pricing_verified_at: "2026-07-16".to_owned(),
        };

        let summary = format_codex_usage_summary(&codex_usage(), Some(&estimate));
        assert!(summary.contains("OpenAI Codex subscription usage"));
        assert!(summary.contains("5-hour limit: 42% used"));
        assert!(summary.contains("Weekly limit: 5% used"));
        assert!(summary.contains("Purchased credits balance: 12.50"));
        assert!(summary.contains("Rate-limit reset credits available: 3"));
        assert!(summary.contains("Hypothetical API-equivalent cost (not subscription spend):"));
        assert!(summary.contains("600,000 uncached input + 400,000 cached input + 100,000 output"));
        assert!(summary.contains("Standard API comparison: $6.200000"));
        assert!(summary.contains("Pricing basis gpt-5.6-sol"));
        assert!(!summary.contains("Subscription spend: $"));
    }

    #[test]
    fn kimi_summary_reports_plan_windows_and_extra_usage_without_cost_guessing() {
        use xai_grok_shell::auth::kimi_code::{
            KimiCodeExtraUsage, KimiCodeUsageRow, KimiCodeUsageSnapshot,
        };
        let usage = KimiCodeUsageSnapshot {
            summary: Some(KimiCodeUsageRow {
                label: "Weekly limit".to_owned(),
                used: 420,
                limit: 1000,
                reset_hint: Some("resets in 3600s".to_owned()),
            }),
            limits: vec![KimiCodeUsageRow {
                label: "5h limit".to_owned(),
                used: 90,
                limit: 100,
                reset_hint: None,
            }],
            extra_usage: Some(KimiCodeExtraUsage {
                balance_cents: 500,
                total_cents: 1000,
                monthly_charge_limit_enabled: true,
                monthly_charge_limit_cents: 5000,
                monthly_used_cents: 1250,
                currency: "USD".to_owned(),
            }),
        };

        let summary = format_kimi_usage_summary(&usage);
        assert!(summary.contains("Kimi Code plan usage"));
        assert!(summary.contains("Weekly limit: 420 / 1000 (42% used); resets in 3600s"));
        assert!(summary.contains("5h limit: 90 / 100 (90% used)"));
        assert!(summary.contains("Extra Usage balance: USD 5.00 / 10.00"));
        assert!(summary.contains("Extra Usage monthly spend: USD 12.50 / 50.00"));
        assert!(summary.contains("API-equivalent cost: unavailable"));
    }

    #[test]
    fn codex_summary_shows_incomplete_observed_cost_as_lower_bound() {
        let estimate = xai_grok_shell::auth::codex::CodexApiEquivalentCostEstimate {
            input_tokens: 1_000,
            cached_input_tokens: 500,
            uncached_input_tokens: 500,
            output_tokens: 100,
            estimated_usd: Some(0.00575),
            lines: Vec::new(),
            unpriced_models: Vec::new(),
            usage_incomplete: true,
            pricing_verified_at: "2026-07-16".to_owned(),
        };
        let summary = format_codex_usage_summary(&codex_usage(), Some(&estimate));
        assert!(summary.contains("Observed API-cost lower bound: $0.005750"));
        assert!(summary.contains("unobserved auxiliary and compaction model calls are excluded"));
        assert!(!summary.contains("Standard API comparison: $"));
    }

    #[test]
    fn codex_summary_refuses_to_price_unpublished_model_alias() {
        let estimate = xai_grok_shell::auth::codex::CodexApiEquivalentCostEstimate {
            input_tokens: 1_000,
            cached_input_tokens: 500,
            uncached_input_tokens: 500,
            output_tokens: 100,
            estimated_usd: None,
            lines: Vec::new(),
            unpriced_models: vec!["gpt-5.6-codex-future".to_owned()],
            usage_incomplete: false,
            pricing_verified_at: "2026-07-16".to_owned(),
        };
        let summary = format_codex_usage_summary(&codex_usage(), Some(&estimate));
        assert!(
            summary
                .contains("Unavailable: no published API rate is mapped for gpt-5.6-codex-future.")
        );
        assert!(!summary.contains("Standard API comparison: $"));
    }

    #[test]
    fn summary_no_credits_omits_credits_block() {
        let b = CreditBalance {
            period_end_display: Some("June 14, 16:00".into()),
            prepaid_balance_cents: Some(0),
            ..bal(25.0)
        };
        // Even with an auto-topup rule present, zero prepaid → no credits block.
        let out = format_usage_summary(&b, Some(&topup(true, Some(2000), Some(10000))));
        assert_eq!(out, "Usage: 25%\nNext reset: June 14, 16:00");
    }

    #[test]
    fn summary_credits_without_autotopup_shows_disabled() {
        let b = CreditBalance {
            prepaid_balance_cents: Some(10000),
            ..bal(25.0)
        };
        assert_eq!(
            format_usage_summary(&b, None),
            "Usage: 25%\n\nCredits: $100\nAuto topup: disabled"
        );
        // A disabled rule renders the same.
        assert_eq!(
            format_usage_summary(&b, Some(&topup(false, Some(2000), Some(10000)))),
            "Usage: 25%\n\nCredits: $100\nAuto topup: disabled"
        );
    }

    #[test]
    fn summary_autotopup_enabled_without_max_omits_max() {
        let b = CreditBalance {
            prepaid_balance_cents: Some(10000),
            ..bal(25.0)
        };
        assert_eq!(
            format_usage_summary(&b, Some(&topup(true, Some(2000), None))),
            "Usage: 25%\n\nCredits: $100\nAuto topup: $20"
        );
    }

    #[test]
    fn summary_autotopup_enabled_with_max_renders_all() {
        let b = CreditBalance {
            period_end_display: Some("June 14, 16:00".into()),
            prepaid_balance_cents: Some(10000),
            ..bal(25.0)
        };
        assert_eq!(
            format_usage_summary(&b, Some(&topup(true, Some(2000), Some(10000)))),
            "Usage: 25%\nNext reset: June 14, 16:00\n\nCredits: $100\nAuto topup: $20\nMax monthly topup: $100"
        );
    }

    #[test]
    fn summary_formats_fractional_dollars() {
        let b = CreditBalance {
            prepaid_balance_cents: Some(1250),
            ..bal(25.0)
        };
        assert_eq!(
            format_usage_summary(&b, Some(&topup(true, Some(550), None))),
            "Usage: 25%\n\nCredits: $12.50\nAuto topup: $5.50"
        );
    }

    #[test]
    fn summary_abs_negative_billing_amounts() {
        // Billing returns credit / top-up amounts as negative cents; the
        // summary must render them as positive USD (matching the web).
        let b = CreditBalance {
            prepaid_balance_cents: Some(-500),
            ..bal(100.0)
        };
        assert_eq!(
            format_usage_summary(&b, Some(&topup(true, Some(-500), Some(-1000)))),
            "Usage: 100%\n\nCredits: $5\nAuto topup: $5\nMax monthly topup: $10"
        );
    }

    #[test]
    fn summary_pay_as_you_go_enabled_renders_used_of_limit() {
        let b = CreditBalance {
            pay_as_you_go: true,
            on_demand_used_cents: Some(355),
            on_demand_cap_cents: Some(5000),
            period_type: Some("USAGE_PERIOD_TYPE_MONTHLY".into()),
            period_end_display: Some("June 30, 16:00".into()),
            ..bal(91.0)
        };
        assert_eq!(
            format_usage_summary(&b, None),
            "Monthly limit: 91%\nNext reset: June 30, 16:00\n\nPay-as-you-go: $3.55 used of $50.00 limit"
        );
    }

    #[test]
    fn summary_pay_as_you_go_disabled_omits_line() {
        let b = CreditBalance {
            pay_as_you_go: false,
            period_type: Some("USAGE_PERIOD_TYPE_MONTHLY".into()),
            period_end_display: Some("June 30, 16:00".into()),
            ..bal(91.0)
        };
        assert_eq!(
            format_usage_summary(&b, None),
            "Monthly limit: 91%\nNext reset: June 30, 16:00"
        );
    }

    // ── usage_label / period type ────────────────────────────────────

    fn bal_period(pct: f64, period_type: &str) -> CreditBalance {
        CreditBalance {
            period_type: Some(period_type.to_string()),
            ..bal(pct)
        }
    }

    #[test]
    fn usage_label_from_period_type() {
        assert_eq!(
            bal_period(0.0, "USAGE_PERIOD_TYPE_WEEKLY").usage_label(),
            "Weekly limit"
        );
        assert_eq!(
            bal_period(0.0, "USAGE_PERIOD_TYPE_MONTHLY").usage_label(),
            "Monthly limit"
        );
        // Unknown / unspecified / absent → falls back to "Usage".
        assert_eq!(
            bal_period(0.0, "USAGE_PERIOD_TYPE_UNSPECIFIED").usage_label(),
            "Usage"
        );
        assert_eq!(bal(0.0).usage_label(), "Usage");
    }

    #[test]
    fn summary_uses_period_label() {
        let weekly = bal_period(25.0, "USAGE_PERIOD_TYPE_WEEKLY");
        assert_eq!(format_usage_summary(&weekly, None), "Weekly limit: 25%");
        let monthly = bal_period(25.0, "USAGE_PERIOD_TYPE_MONTHLY");
        assert_eq!(format_usage_summary(&monthly, None), "Monthly limit: 25%");
    }

    #[test]
    fn warning_uses_period_label() {
        let weekly = bal_period(92.0, "USAGE_PERIOD_TYPE_WEEKLY");
        assert_eq!(
            usage_warning(&weekly, None, true),
            Some(("Weekly limit left: 8%".to_string(), false))
        );
    }

    #[test]
    fn summary_floors_usage_percent() {
        // Match the backend SpendingLimiter (`as u8` truncation): 99.994% must
        // render as 99%, not round up to 100%.
        let almost = bal_period(99.994, "USAGE_PERIOD_TYPE_WEEKLY");
        assert_eq!(format_usage_summary(&almost, None), "Weekly limit: 99%");
        // A true 100% still shows 100%.
        let full = bal_period(100.0, "USAGE_PERIOD_TYPE_WEEKLY");
        assert_eq!(format_usage_summary(&full, None), "Weekly limit: 100%");
    }

    #[test]
    fn warning_percent_left_is_floor_complement() {
        // 99.994% used → floored to 99% → "1% left" (not "0% left"), so the
        // warning and the floored summary always sum to 100.
        let almost = bal_period(99.994, "USAGE_PERIOD_TYPE_WEEKLY");
        assert_eq!(
            usage_warning(&almost, None, true),
            Some(("Weekly limit left: 1%".to_string(), true))
        );
        // A true 100% (no credits) → "0% left".
        let full = bal_period(100.0, "USAGE_PERIOD_TYPE_WEEKLY");
        assert_eq!(
            usage_warning(&full, None, true),
            Some(("Weekly limit left: 0%".to_string(), true))
        );
    }

    // ── usage_warning (prompt info row) ──────────────────────────────

    #[test]
    fn warning_usage_model_thresholds() {
        assert_eq!(usage_warning(&bal(50.0), None, true), None);
        assert_eq!(
            usage_warning(&bal(92.0), None, true),
            Some(("Usage left: 8%".to_string(), false))
        );
        assert_eq!(
            usage_warning(&bal(97.0), None, true),
            Some(("Usage left: 3%".to_string(), true))
        );
    }

    #[test]
    fn warning_hidden_for_team_users() {
        assert_eq!(usage_warning(&bal(99.0), None, false), None);
        let credits = CreditBalance {
            prepaid_balance_cents: Some(100),
            ..bal(0.0)
        };
        assert_eq!(usage_warning(&credits, None, false), None);
    }

    #[test]
    fn warning_credits_unknown_topup_is_suppressed() {
        // At 100% usage with prepaid credits, but the rule isn't known yet
        // (None) — never warn; it resolves on the next billing fetch.
        let b = CreditBalance {
            prepaid_balance_cents: Some(100),
            ..bal(100.0)
        };
        assert_eq!(usage_warning(&b, None, true), None);
    }

    #[test]
    fn warning_credits_suppressed_below_full_usage() {
        // Low credits + no auto top-up, but the included allowance still has
        // room (usage < 100%) → no warning (credits aren't being spent yet).
        let disabled = topup(false, None, None);
        let low = CreditBalance {
            prepaid_balance_cents: Some(453),
            ..bal(0.0)
        };
        assert_eq!(usage_warning(&low, Some(&disabled), true), None);
        // Same balance once the allowance is exhausted → warn.
        let exhausted = CreditBalance {
            prepaid_balance_cents: Some(453),
            ..bal(100.0)
        };
        assert_eq!(
            usage_warning(&exhausted, Some(&disabled), true),
            Some(("Credits left: $4.53".to_string(), true))
        );
    }

    #[test]
    fn warning_credits_no_topup_low_shows_dollars() {
        // "No auto top-up" is a known, disabled rule (not an unresolved None).
        let b = CreditBalance {
            prepaid_balance_cents: Some(453),
            ..bal(100.0)
        };
        let disabled = topup(false, None, None);
        assert_eq!(
            usage_warning(&b, Some(&disabled), true),
            Some(("Credits left: $4.53".to_string(), true))
        );
    }

    #[test]
    fn warning_credits_no_topup_above_threshold_silent() {
        let disabled = topup(false, None, None);
        let b = CreditBalance {
            prepaid_balance_cents: Some(1500),
            ..bal(100.0)
        };
        assert_eq!(usage_warning(&b, Some(&disabled), true), None);
        // Exactly $10 is still "low".
        let at_ten = CreditBalance {
            prepaid_balance_cents: Some(1000),
            ..bal(100.0)
        };
        assert_eq!(
            usage_warning(&at_ten, Some(&disabled), true),
            Some(("Credits left: $10".to_string(), true))
        );
    }

    #[test]
    fn warning_credits_topup_no_max_never_warns() {
        let b = CreditBalance {
            prepaid_balance_cents: Some(1),
            ..bal(100.0)
        };
        assert_eq!(
            usage_warning(&b, Some(&topup(true, Some(2000), None)), true),
            None
        );
    }

    #[test]
    fn warning_credits_topup_with_max_below_topup_amount() {
        // $15 balance, $20 top-up amount, $100 max → below one top-up → warn.
        let b = CreditBalance {
            prepaid_balance_cents: Some(1500),
            ..bal(100.0)
        };
        assert_eq!(
            usage_warning(&b, Some(&topup(true, Some(2000), Some(10000))), true),
            Some(("Credits left: $15".to_string(), true))
        );
        let plenty = CreditBalance {
            prepaid_balance_cents: Some(2500),
            ..bal(100.0)
        };
        assert_eq!(
            usage_warning(&plenty, Some(&topup(true, Some(2000), Some(10000))), true),
            None
        );
    }

    #[test]
    fn warning_credits_handles_negative_cents() {
        let b = CreditBalance {
            prepaid_balance_cents: Some(-453),
            ..bal(100.0)
        };
        assert_eq!(
            usage_warning(&b, Some(&topup(true, Some(-2000), Some(-10000))), true),
            Some(("Credits left: $4.53".to_string(), true))
        );
    }

    #[test]
    fn warning_credits_take_precedence_over_usage() {
        // A credits user below 100% usage gets no warning at all (no usage-%
        // warning, and credits aren't being spent yet) — unlike a non-credits
        // user, who would see "Usage left: 1%" at 99%.
        let b = CreditBalance {
            prepaid_balance_cents: Some(5000),
            ..bal(99.0)
        };
        assert_eq!(
            usage_warning(&b, Some(&topup(false, None, None)), true),
            None
        );
        // Zero prepaid falls back to the usage model.
        let zero = CreditBalance {
            prepaid_balance_cents: Some(0),
            ..bal(99.0)
        };
        assert_eq!(
            usage_warning(&zero, None, true),
            Some(("Usage left: 1%".to_string(), true))
        );
    }

    // ── usage_warning: pay-as-you-go (monthly on-demand) ─────────────

    fn pay_as_you_go(usage_pct: f64, cap_cents: i64, used_cents: i64) -> CreditBalance {
        CreditBalance {
            pay_as_you_go: true,
            on_demand_cap_cents: Some(cap_cents),
            on_demand_used_cents: Some(used_cents),
            period_type: Some("USAGE_PERIOD_TYPE_MONTHLY".into()),
            ..bal(usage_pct)
        }
    }

    #[test]
    fn warning_pay_as_you_go_low_dollars_shows_remaining() {
        // $50 cap, $42 used → $8 left → grey (above $5).
        let grey = pay_as_you_go(100.0, 5000, 4200);
        assert_eq!(
            usage_warning(&grey, None, true),
            Some(("Pay-as-you-go limit left: $8".to_string(), false))
        );
        // $50 cap, $46 used → $4 left → critical (yellow).
        let yellow = pay_as_you_go(100.0, 5000, 4600);
        assert_eq!(
            usage_warning(&yellow, None, true),
            Some(("Pay-as-you-go limit left: $4".to_string(), true))
        );
    }

    #[test]
    fn warning_pay_as_you_go_boundaries() {
        // Exactly $10 left → show, grey.
        let at_ten = pay_as_you_go(100.0, 5000, 4000);
        assert_eq!(
            usage_warning(&at_ten, None, true),
            Some(("Pay-as-you-go limit left: $10".to_string(), false))
        );
        // Exactly $5 left → critical (yellow).
        let at_five = pay_as_you_go(100.0, 5000, 4500);
        assert_eq!(
            usage_warning(&at_five, None, true),
            Some(("Pay-as-you-go limit left: $5".to_string(), true))
        );
    }

    #[test]
    fn warning_pay_as_you_go_above_threshold_silent() {
        // $20 left (> $10) → no warning.
        let b = pay_as_you_go(100.0, 5000, 3000);
        assert_eq!(usage_warning(&b, None, true), None);
    }

    #[test]
    fn warning_pay_as_you_go_suppressed_below_full_usage() {
        // Pay-as-you-go users get NO percentage warning before the included
        // allowance is exhausted, even with low on-demand room remaining.
        let b = pay_as_you_go(95.0, 5000, 4800);
        assert_eq!(usage_warning(&b, None, true), None);
    }

    #[test]
    fn warning_pay_as_you_go_fractional_dollars() {
        // $50 cap, $46.50 used → $3.50 left → critical, fractional formatting.
        let b = pay_as_you_go(100.0, 5000, 4650);
        assert_eq!(
            usage_warning(&b, None, true),
            Some(("Pay-as-you-go limit left: $3.50".to_string(), true))
        );
    }

    #[test]
    fn test_credit_bar_line_shows_percentage() {
        let theme = Theme::default();
        let line = credit_bar_line(&bal(24.0), false, &theme);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Credits used: 24%");
    }

    #[test]
    fn test_color_thresholds() {
        let theme = Theme::default();

        let low = credit_bar_line(&bal(50.0), false, &theme);
        assert_eq!(low.spans[0].style.fg, Some(theme.accent_success));

        let high = credit_bar_line(&bal(85.0), false, &theme);
        assert_eq!(high.spans[0].style.fg, Some(theme.warning));

        let over = credit_bar_line(&bal(100.0), false, &theme);
        assert_eq!(over.spans[0].style.fg, Some(theme.accent_error));
    }

    #[test]
    fn test_zero_percent() {
        let theme = Theme::default();
        let line = credit_bar_line(&bal(0.0), false, &theme);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Credits used: 0%");
        assert_eq!(line.spans[0].style.fg, Some(theme.accent_success));
    }

    #[test]
    fn test_boundary_at_80_percent() {
        let theme = Theme::default();
        // Exactly 80% should be warning (yellow).
        let at_80 = credit_bar_line(&bal(80.0), false, &theme);
        assert_eq!(at_80.spans[0].style.fg, Some(theme.warning));

        // Just below 80% should be success (green).
        let below_80 = credit_bar_line(&bal(79.9), false, &theme);
        assert_eq!(below_80.spans[0].style.fg, Some(theme.accent_success));
    }

    #[test]
    fn test_boundary_at_100_percent() {
        let theme = Theme::default();
        // Exactly 100% should be error (red).
        let at_100 = credit_bar_line(&bal(100.0), false, &theme);
        assert_eq!(at_100.spans[0].style.fg, Some(theme.accent_error));

        // Just below 100% should be warning (yellow).
        let below_100 = credit_bar_line(&bal(99.9), false, &theme);
        assert_eq!(below_100.spans[0].style.fg, Some(theme.warning));
    }

    #[test]
    fn test_over_100_percent() {
        let theme = Theme::default();
        let line = credit_bar_line(&bal(150.0), false, &theme);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Credits used: 150%");
        assert_eq!(line.spans[0].style.fg, Some(theme.accent_error));
    }

    #[test]
    fn test_fractional_percentage_rounds_display() {
        let theme = Theme::default();
        let line = credit_bar_line(&bal(33.7), false, &theme);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Credits used: 34%");
    }

    #[test]
    fn test_credit_balance_with_on_demand_fields() {
        let balance = CreditBalance {
            effective_usage_pct: 25.0,
            period_end_display: Some("Jun 1, 00:00".into()),
            pay_as_you_go: true,
            on_demand_cap_cents: Some(2000),
            on_demand_used_cents: Some(500),
            ..bal(50.0)
        };
        let theme = Theme::default();
        // The credit bar uses usage_pct (not effective_usage_pct).
        let line = credit_bar_line(&balance, false, &theme);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Credits used: 50%");
    }

    #[test]
    fn gateway_chat_suppresses_credit_bar_and_usage_warning() {
        let theme = Theme::default();
        let b = bal(90.0);
        assert!(credit_bar_line_for_session(&b, false, &theme, true).is_none());
        assert!(usage_warning_for_session(&b, None, true, true).is_none());
        // Build path still renders.
        assert!(credit_bar_line_for_session(&b, false, &theme, false).is_some());
    }
}
