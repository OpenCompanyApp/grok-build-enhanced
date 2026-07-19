use chrono::{DateTime, TimeZone, Utc};
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use xai_grok_sampling_types::{
    ZAI_CODING_PLAN_MAX_RESPONSE_BYTES, ZAI_CODING_PLAN_MODEL_USAGE_URL, ZAI_CODING_PLAN_QUOTA_URL,
    ZAI_CODING_PLAN_TOOL_USAGE_URL,
};

use super::{ZaiCodingPlanAuthError, ZaiCodingPlanCredentials};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZaiCodingPlanUsageDetail {
    pub code: String,
    pub usage: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZaiCodingPlanUsageRow {
    pub label: String,
    pub used: i64,
    pub limit: i64,
    pub percentage: f64,
    pub reset_at: Option<DateTime<Utc>>,
    pub details: Vec<ZaiCodingPlanUsageDetail>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZaiCodingPlanUsageSnapshot {
    pub plan_name: Option<String>,
    pub five_hour: Option<ZaiCodingPlanUsageRow>,
    pub weekly: Option<ZaiCodingPlanUsageRow>,
    pub monthly_mcp: Option<ZaiCodingPlanUsageRow>,
    pub other_limits: Vec<ZaiCodingPlanUsageRow>,
    pub recent_model_tokens: Vec<ZaiCodingPlanUsageDetail>,
    pub recent_tool_calls: Vec<ZaiCodingPlanUsageDetail>,
}

impl ZaiCodingPlanUsageSnapshot {
    pub fn highest_used_percent(&self) -> Option<f64> {
        self.five_hour
            .iter()
            .chain(self.weekly.iter())
            .chain(self.monthly_mcp.iter())
            .chain(self.other_limits.iter())
            .map(|row| row.percentage)
            .filter(|value| value.is_finite())
            .max_by(f64::total_cmp)
    }

    pub fn rows(&self) -> impl Iterator<Item = &ZaiCodingPlanUsageRow> {
        self.five_hour
            .iter()
            .chain(self.weekly.iter())
            .chain(self.monthly_mcp.iter())
            .chain(self.other_limits.iter())
    }
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    code: Option<serde_json::Value>,
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

pub async fn fetch_usage(
    credentials: &ZaiCodingPlanCredentials,
) -> Result<ZaiCodingPlanUsageSnapshot, ZaiCodingPlanAuthError> {
    credentials.validate_persisted()?;
    let client = super::transport::http_client(std::time::Duration::from_secs(30))?;
    let quota = fetch_json(&client, credentials, ZAI_CODING_PLAN_QUOTA_URL).await?;
    let mut snapshot = parse_quota(&quota)?;

    // The detail endpoints are undocumented and optional. Their failure must
    // not hide the authoritative quota windows.
    if let Ok(value) = fetch_dated_json(&client, credentials, ZAI_CODING_PLAN_MODEL_USAGE_URL).await
    {
        snapshot.recent_model_tokens =
            parse_series_details(&value, "modelDataList", "modelName", "tokensUsage");
    }
    if let Ok(value) = fetch_dated_json(&client, credentials, ZAI_CODING_PLAN_TOOL_USAGE_URL).await
    {
        snapshot.recent_tool_calls = parse_generic_details(&value);
    }
    Ok(snapshot)
}

async fn fetch_dated_json(
    client: &reqwest::Client,
    credentials: &ZaiCodingPlanCredentials,
    base: &str,
) -> Result<serde_json::Value, ZaiCodingPlanAuthError> {
    let end = Utc::now();
    let start = end - chrono::Duration::hours(24);
    let url = reqwest::Url::parse_with_params(
        base,
        [
            ("startTime", start.format("%Y-%m-%d %H:00:00").to_string()),
            ("endTime", end.format("%Y-%m-%d %H:59:59").to_string()),
        ],
    )
    .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    fetch_json(client, credentials, url.as_str()).await
}

async fn fetch_json(
    client: &reqwest::Client,
    credentials: &ZaiCodingPlanCredentials,
    url: &str,
) -> Result<serde_json::Value, ZaiCodingPlanAuthError> {
    let mut authorization = HeaderValue::from_str(&format!("Bearer {}", credentials.api_key()))
        .map_err(|_| ZaiCodingPlanAuthError::InvalidCredential)?;
    authorization.set_sensitive(true);
    let response = client
        .get(url)
        .header(AUTHORIZATION, authorization)
        .header(ACCEPT, "application/json")
        .header(ACCEPT_LANGUAGE, "en-US,en")
        .header(
            USER_AGENT,
            format!("grok-agent/{}", xai_grok_version::VERSION),
        )
        .send()
        .await
        .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    let status = response.status();
    let bytes =
        super::transport::read_limited_body(response, ZAI_CODING_PLAN_MAX_RESPONSE_BYTES).await?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    if let Some(code) = super::transport::business_code(&value) {
        return Err(ZaiCodingPlanAuthError::Business(code));
    }
    if !status.is_success() {
        return Err(ZaiCodingPlanAuthError::Http(status));
    }
    Ok(value)
}

fn parse_quota(
    value: &serde_json::Value,
) -> Result<ZaiCodingPlanUsageSnapshot, ZaiCodingPlanAuthError> {
    let envelope: Envelope = serde_json::from_value(value.clone())
        .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    if envelope.success == Some(false) {
        let code = envelope
            .code
            .as_ref()
            .and_then(number_as_i64)
            .unwrap_or_default();
        return Err(ZaiCodingPlanAuthError::Business(code));
    }
    let data = envelope.data.as_ref().unwrap_or(value);
    let plan_name = data
        .get("planName")
        .or_else(|| data.get("plan_name"))
        .and_then(serde_json::Value::as_str)
        .and_then(safe_label);
    let mut token_rows = Vec::new();
    let mut mcp_rows = Vec::new();
    let mut other = Vec::new();
    for (index, raw) in data
        .get("limits")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let kind = raw
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let mut row = parse_limit(raw, format!("Provider limit #{}", index + 1));
        match kind {
            "TOKENS_LIMIT" => token_rows.push((window_minutes(raw), row)),
            "TIME_LIMIT" if is_mcp_limit(raw, &row) => {
                row.label = "Monthly MCP".to_owned();
                mcp_rows.push(row);
            }
            _ => other.push(row),
        }
    }
    token_rows.sort_by_key(|(window, _)| window.unwrap_or(i64::MAX));
    let (five_hour, weekly) = match token_rows.len() {
        0 => (None, None),
        1 => {
            let mut row = token_rows.remove(0).1;
            row.label = window_label_for_single(&row.label);
            (Some(row), None)
        }
        _ => {
            let mut first = token_rows.remove(0).1;
            let mut last = token_rows.pop().expect("at least one token row").1;
            first.label = "5-hour pool".to_owned();
            last.label = "Weekly quota".to_owned();
            for (_, row) in token_rows {
                other.push(row);
            }
            (Some(first), Some(last))
        }
    };
    Ok(ZaiCodingPlanUsageSnapshot {
        plan_name,
        five_hour,
        weekly,
        monthly_mcp: mcp_rows.into_iter().next(),
        other_limits: other,
        recent_model_tokens: Vec::new(),
        recent_tool_calls: Vec::new(),
    })
}

fn parse_limit(raw: &serde_json::Value, fallback: String) -> ZaiCodingPlanUsageRow {
    let limit = raw.get("usage").and_then(number_as_i64).unwrap_or(0).max(0);
    let current = raw
        .get("currentValue")
        .or_else(|| raw.get("current_value"))
        .and_then(number_as_i64);
    let remaining = raw.get("remaining").and_then(number_as_i64);
    let used = current
        .or_else(|| remaining.map(|remaining| limit.saturating_sub(remaining)))
        .unwrap_or(0)
        .clamp(0, limit.max(0));
    let percentage = if limit > 0 {
        (used as f64 / limit as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        raw.get("percentage")
            .and_then(number_as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 100.0)
    };
    let details = raw
        .get("usageDetails")
        .or_else(|| raw.get("usage_details"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|detail| {
            let code = detail
                .get("modelCode")
                .or_else(|| detail.get("model_code"))
                .and_then(serde_json::Value::as_str)
                .and_then(safe_code)?;
            let usage = detail.get("usage").and_then(number_as_i64)?.max(0);
            Some(ZaiCodingPlanUsageDetail { code, usage })
        })
        .collect();
    let reset_at = raw
        .get("nextResetTime")
        .or_else(|| raw.get("next_reset_time"))
        .and_then(number_as_i64)
        .and_then(|millis| Utc.timestamp_millis_opt(millis).single());
    ZaiCodingPlanUsageRow {
        label: window_description(raw).unwrap_or(fallback),
        used,
        limit,
        percentage,
        reset_at,
        details,
    }
}

fn is_mcp_limit(raw: &serde_json::Value, row: &ZaiCodingPlanUsageRow) -> bool {
    row.details.iter().any(|detail| {
        let code = detail.code.to_ascii_lowercase();
        code.contains("search") || code.contains("reader") || code.contains("zread")
    }) || matches!(
        (
            raw.get("unit").and_then(number_as_i64),
            raw.get("number").and_then(number_as_i64)
        ),
        (Some(5), Some(1))
    )
}

fn window_minutes(raw: &serde_json::Value) -> Option<i64> {
    let number = raw.get("number").and_then(number_as_i64)?.checked_abs()?;
    if number == 0 {
        return None;
    }
    match raw.get("unit").and_then(number_as_i64)? {
        1 => number.checked_mul(24 * 60),
        3 => number.checked_mul(60),
        5 => Some(number),
        6 => number.checked_mul(7 * 24 * 60),
        _ => None,
    }
}

fn window_description(raw: &serde_json::Value) -> Option<String> {
    let number = raw.get("number").and_then(number_as_i64)?;
    if number <= 0 {
        return None;
    }
    let unit = match raw.get("unit").and_then(number_as_i64)? {
        1 => "day",
        3 => "hour",
        5 => "minute",
        6 => "week",
        _ => return None,
    };
    Some(format!(
        "{number} {unit}{} window",
        if number == 1 { "" } else { "s" }
    ))
}

fn window_label_for_single(label: &str) -> String {
    if label.contains("5 hour") || label.contains("300 minute") {
        "5-hour pool".to_owned()
    } else if label.contains("7 day") || label.contains("1 week") {
        "Weekly quota".to_owned()
    } else {
        label.to_owned()
    }
}

fn parse_series_details(
    value: &serde_json::Value,
    list_key: &str,
    name_key: &str,
    values_key: &str,
) -> Vec<ZaiCodingPlanUsageDetail> {
    let data = value.get("data").unwrap_or(value);
    data.get(list_key)
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let code = item
                .get(name_key)
                .and_then(serde_json::Value::as_str)
                .and_then(safe_code)?;
            let usage = item
                .get(values_key)
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(number_as_i64)
                .fold(0i64, i64::saturating_add)
                .max(0);
            (usage > 0).then_some(ZaiCodingPlanUsageDetail { code, usage })
        })
        .collect()
}

fn parse_generic_details(value: &serde_json::Value) -> Vec<ZaiCodingPlanUsageDetail> {
    let data = value.get("data").unwrap_or(value);
    let arrays = data
        .get("usageDetails")
        .or_else(|| data.get("toolDataList"))
        .and_then(serde_json::Value::as_array);
    arrays
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let code = item
                .get("modelCode")
                .or_else(|| item.get("toolCode"))
                .or_else(|| item.get("name"))
                .and_then(serde_json::Value::as_str)
                .and_then(safe_code)?;
            let usage = item
                .get("usage")
                .or_else(|| item.get("count"))
                .and_then(number_as_i64)?
                .max(0);
            Some(ZaiCodingPlanUsageDetail { code, usage })
        })
        .collect()
}

fn safe_label(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_')
        }))
    .then(|| value.to_owned())
}

fn safe_code(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.len() <= 128
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
        }))
    .then(|| value.to_owned())
}

fn number_as_i64(value: &serde_json::Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| value.as_str()?.trim().parse::<i64>().ok())
}

fn number_as_f64(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_classifies_short_long_and_mcp_windows_without_identity_fields() {
        let snapshot = parse_quota(&serde_json::json!({
            "code": 200,
            "success": true,
            "data": {
                "planName": "Pro",
                "limits": [
                    {"type":"TOKENS_LIMIT","unit":3,"number":5,"usage":100,"currentValue":25,"percentage":25,"nextResetTime":1700000000000_i64},
                    {"type":"TOKENS_LIMIT","unit":1,"number":7,"usage":1000,"remaining":400,"percentage":60},
                    {"type":"TIME_LIMIT","unit":5,"number":1,"usage":1000,"currentValue":10,"usageDetails":[{"modelCode":"web_search_prime","usage":8},{"modelCode":"zread","usage":2}]}
                ]
            }
        })).unwrap();
        assert_eq!(snapshot.five_hour.as_ref().unwrap().used, 25);
        assert_eq!(snapshot.weekly.as_ref().unwrap().used, 600);
        assert_eq!(snapshot.monthly_mcp.as_ref().unwrap().used, 10);
        assert_eq!(snapshot.plan_name.as_deref(), Some("Pro"));
    }

    #[test]
    fn business_error_inside_http_200_is_not_parsed_as_usage() {
        assert!(matches!(
            parse_quota(&serde_json::json!({"code":1001,"success":false,"msg":"private"})),
            Err(ZaiCodingPlanAuthError::Business(1001))
        ));
    }
}
