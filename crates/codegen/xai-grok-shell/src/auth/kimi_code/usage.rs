use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use xai_grok_sampling_types::KIMI_CODE_USAGE_URL;

use super::{KimiCodeAuthError, KimiCodeCredentials};

const MAX_USAGE_BYTES: usize = 1024 * 1024;
const FIXED_POINT_CENTS: i64 = 1_000_000;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiCodeUsageRow {
    pub label: String,
    pub used: i64,
    pub limit: i64,
    pub reset_hint: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiCodeExtraUsage {
    pub balance_cents: i64,
    pub total_cents: i64,
    pub monthly_charge_limit_enabled: bool,
    pub monthly_charge_limit_cents: i64,
    pub monthly_used_cents: i64,
    pub currency: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiCodeUsageSnapshot {
    pub summary: Option<KimiCodeUsageRow>,
    pub limits: Vec<KimiCodeUsageRow>,
    pub extra_usage: Option<KimiCodeExtraUsage>,
}

impl KimiCodeUsageSnapshot {
    pub fn highest_used_percent(&self) -> Option<f64> {
        self.summary
            .iter()
            .chain(self.limits.iter())
            .filter(|row| row.limit > 0)
            .map(|row| (row.used.max(0) as f64 / row.limit as f64) * 100.0)
            .max_by(f64::total_cmp)
    }
}

pub async fn fetch_usage(
    credentials: &KimiCodeCredentials,
) -> Result<KimiCodeUsageSnapshot, KimiCodeAuthError> {
    credentials.validate_persisted()?;
    let client = super::transport::http_client(std::time::Duration::from_secs(30))?;
    let mut authorization = HeaderValue::from_str(&format!("Bearer {}", credentials.api_key()))
        .map_err(|_| KimiCodeAuthError::InvalidCredential)?;
    authorization.set_sensitive(true);
    let response = client
        .get(KIMI_CODE_USAGE_URL)
        .header(AUTHORIZATION, authorization)
        .header(ACCEPT, "application/json")
        .header(
            USER_AGENT,
            format!("grok-agent/{}", xai_grok_version::VERSION),
        )
        .send()
        .await
        .map_err(|_| KimiCodeAuthError::InvalidResponse)?;
    let status = response.status();
    if !status.is_success() {
        return Err(KimiCodeAuthError::Http(status));
    }
    let bytes = super::transport::read_limited_body(response, MAX_USAGE_BYTES).await?;
    let payload: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| KimiCodeAuthError::InvalidResponse)?;
    Ok(parse_usage(&payload))
}

fn parse_usage(payload: &serde_json::Value) -> KimiCodeUsageSnapshot {
    let summary = payload
        .get("usage")
        .and_then(|value| usage_row(value, "Weekly limit"));
    let limits = payload
        .get("limits")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(index, item)| {
            let detail = item.get("detail").unwrap_or(item);
            usage_row(detail, &limit_label(item, detail, index))
        })
        .collect();
    let extra_usage = payload.get("boosterWallet").and_then(parse_extra_usage);
    KimiCodeUsageSnapshot {
        summary,
        limits,
        extra_usage,
    }
}

fn usage_row(value: &serde_json::Value, fallback: &str) -> Option<KimiCodeUsageRow> {
    let object = value.as_object()?;
    let limit = object.get("limit").and_then(number_as_i64);
    let used = object.get("used").and_then(number_as_i64).or_else(|| {
        let remaining = object.get("remaining").and_then(number_as_i64)?;
        Some(limit?.saturating_sub(remaining))
    });
    if limit.is_none() && used.is_none() {
        return None;
    }
    Some(KimiCodeUsageRow {
        // Provider-controlled labels are not needed to interpret usage and can
        // contain account identifiers. Keep the UI projection structural.
        label: fallback.to_owned(),
        used: used.unwrap_or(0),
        limit: limit.unwrap_or(0),
        reset_hint: reset_hint(object),
    })
}

fn limit_label(item: &serde_json::Value, detail: &serde_json::Value, index: usize) -> String {
    let window = item.get("window");
    let duration = window
        .and_then(|window| window.get("duration"))
        .or_else(|| item.get("duration"))
        .or_else(|| detail.get("duration"))
        .and_then(number_as_i64)
        .filter(|duration| *duration > 0);
    let unit = window
        .and_then(|window| window.get("timeUnit"))
        .or_else(|| item.get("timeUnit"))
        .or_else(|| detail.get("timeUnit"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_ascii_uppercase();
    match (duration, unit.as_str()) {
        (Some(duration), unit) if unit.contains("MINUTE") && duration % 60 == 0 => {
            format!("{}h limit", duration / 60)
        }
        (Some(duration), unit) if unit.contains("MINUTE") => format!("{duration}m limit"),
        (Some(duration), unit) if unit.contains("HOUR") => format!("{duration}h limit"),
        (Some(duration), unit) if unit.contains("DAY") => format!("{duration}d limit"),
        (Some(duration), _) => format!("{duration}s limit"),
        (None, _) => format!("Limit #{}", index + 1),
    }
}

fn reset_hint(object: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    for key in ["reset_at", "resetAt", "reset_time", "resetTime"] {
        if let Some(value) = object
            .get(key)
            .and_then(serde_json::Value::as_str)
            .and_then(safe_reset_text)
        {
            return Some(format!("resets at {value}"));
        }
    }
    for key in ["reset_in", "resetIn", "ttl", "window"] {
        if let Some(seconds) = object
            .get(key)
            .and_then(number_as_i64)
            .filter(|seconds| *seconds > 0)
        {
            return Some(format!("resets in {seconds}s"));
        }
    }
    None
}

fn parse_extra_usage(value: &serde_json::Value) -> Option<KimiCodeExtraUsage> {
    let object = value.as_object()?;
    let balance = object.get("balance")?.as_object()?;
    if balance.get("type").and_then(serde_json::Value::as_str) != Some("BOOSTER") {
        return None;
    }
    let total = balance.get("amount").and_then(number_as_i64)?;
    if total <= 0 {
        return None;
    }
    let monthly_limit = parse_money(object.get("monthlyChargeLimit"));
    let monthly_used = parse_money(object.get("monthlyUsed"));
    let currency = monthly_limit
        .as_ref()
        .or(monthly_used.as_ref())
        .map(|(_, currency)| currency.clone())
        .filter(|currency| !currency.is_empty())
        .unwrap_or_else(|| "USD".to_owned());
    Some(KimiCodeExtraUsage {
        balance_cents: fixed_point_to_cents(
            balance
                .get("amountLeft")
                .and_then(number_as_i64)
                .unwrap_or(0),
        ),
        total_cents: fixed_point_to_cents(total),
        monthly_charge_limit_enabled: object
            .get("monthlyChargeLimitEnabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        monthly_charge_limit_cents: monthly_limit.map(|(cents, _)| cents).unwrap_or(0),
        monthly_used_cents: monthly_used.map(|(cents, _)| cents).unwrap_or(0),
        currency,
    })
}

fn parse_money(value: Option<&serde_json::Value>) -> Option<(i64, String)> {
    let object = value?.as_object()?;
    let cents = object.get("priceInCents").and_then(number_as_i64)?;
    let currency = object
        .get("currency")
        .and_then(serde_json::Value::as_str)
        .and_then(safe_currency)
        .unwrap_or_default();
    Some((cents, currency))
}

fn fixed_point_to_cents(value: i64) -> i64 {
    if value > 0 && value < FIXED_POINT_CENTS {
        return 1;
    }
    let value = i128::from(value);
    let scale = i128::from(FIXED_POINT_CENTS);
    let rounded = if value >= 0 {
        (value + scale / 2) / scale
    } else {
        (value - scale / 2) / scale
    };
    i64::try_from(rounded).unwrap_or(if rounded.is_negative() {
        i64::MIN
    } else {
        i64::MAX
    })
}

fn safe_reset_text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_digit()
                || matches!(
                    character,
                    '-' | ':' | '+' | '.' | 'T' | 'Z' | 't' | 'z' | ' '
                )
        }))
    .then(|| value.to_owned())
}

fn safe_currency(value: &str) -> Option<String> {
    let value = value.trim();
    (value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .then(|| value.to_ascii_uppercase())
}

fn number_as_i64(value: &serde_json::Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| {
            value
                .as_f64()
                .filter(|value| value.is_finite())
                .map(|value| value.trunc() as i64)
        })
        .or_else(|| {
            value
                .as_str()
                .and_then(|value| value.trim().parse::<f64>().ok())
                .filter(|value| value.is_finite())
                .map(|value| value.trunc() as i64)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_usage_field_drift_and_derives_window_labels() {
        let snapshot = parse_usage(&serde_json::json!({
            "usage": {"name": "Weekly", "used": "40", "limit": 1000, "resetIn": 90},
            "limits": [{
                "detail": {"remaining": 99, "limit": "100"},
                "window": {"duration": 300, "timeUnit": "MINUTE"}
            }]
        }));
        assert_eq!(snapshot.summary.unwrap().used, 40);
        assert_eq!(snapshot.limits[0].used, 1);
        assert_eq!(snapshot.limits[0].label, "5h limit");
    }

    #[test]
    fn remaining_usage_derivation_saturates_untrusted_integer_extremes() {
        let snapshot = parse_usage(&serde_json::json!({
            "usage": {"limit": i64::MAX, "remaining": i64::MIN}
        }));

        assert_eq!(snapshot.summary.unwrap().used, i64::MAX);
    }

    #[test]
    fn parser_does_not_project_provider_controlled_identity_labels() {
        let snapshot = parse_usage(&serde_json::json!({
            "usage": {
                "name": "private-account-id",
                "used": 1,
                "limit": 10,
                "resetAt": "private-account-id"
            },
            "limits": [{
                "scope": "private-team-id",
                "detail": {"used": 2, "limit": 10}
            }],
            "boosterWallet": {
                "balance": {"type": "BOOSTER", "amount": 1000000, "amountLeft": 1000000},
                "monthlyChargeLimit": {"priceInCents": 100, "currency": "private-account-id"}
            }
        }));

        assert_eq!(snapshot.summary.as_ref().unwrap().label, "Weekly limit");
        assert!(snapshot.summary.as_ref().unwrap().reset_hint.is_none());
        assert_eq!(snapshot.limits[0].label, "Limit #1");
        assert_eq!(snapshot.extra_usage.as_ref().unwrap().currency, "USD");
        assert!(!format!("{snapshot:?}").contains("private"));
    }

    #[test]
    fn parser_projects_extra_usage_without_retaining_unknown_account_fields() {
        let snapshot = parse_usage(&serde_json::json!({
            "boosterWallet": {
                "balance": {
                    "type": "BOOSTER",
                    "amount": 12000000,
                    "amountLeft": 5000000,
                    "privateAccountField": "must not survive"
                },
                "monthlyChargeLimitEnabled": true,
                "monthlyChargeLimit": {"priceInCents": 5000, "currency": "USD"},
                "monthlyUsed": {"priceInCents": 1250, "currency": "USD"}
            }
        }));
        let extra = snapshot.extra_usage.unwrap();
        assert_eq!(extra.total_cents, 12);
        assert_eq!(extra.balance_cents, 5);
        assert_eq!(extra.monthly_used_cents, 1250);
        assert_eq!(extra.currency, "USD");
    }
}
