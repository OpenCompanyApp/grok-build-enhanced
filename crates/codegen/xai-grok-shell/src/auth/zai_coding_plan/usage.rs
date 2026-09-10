//! Numeric-only projection of the official Coding Plan usage-plugin contract.
//! No browser credentials, account metadata, or upstream error bodies escape.
use futures_util::StreamExt as _;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::{Deserialize, Serialize};

use super::{ZaiCodingPlanAuthError, ZaiCodingPlanCredentials};

const QUOTA_URL: &str = "https://api.z.ai/api/monitor/usage/quota/limit";
const MAX_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZaiCodingPlanUsageRow {
    pub label: String,
    pub percentage: f64,
    pub used: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZaiCodingPlanUsageSnapshot {
    pub limits: Vec<ZaiCodingPlanUsageRow>,
}

impl ZaiCodingPlanUsageSnapshot {
    pub fn highest_used_percent(&self) -> Option<f64> {
        self.limits
            .iter()
            .map(|row| row.percentage)
            .max_by(f64::total_cmp)
    }

    pub fn summary(&self) -> String {
        let mut text = String::from("Z.AI GLM Coding Plan quota");
        for row in &self.limits {
            text.push_str(&format!("\n{}: {:.1}% used", row.label, row.percentage));
            if let (Some(used), Some(limit)) = (row.used, row.limit) {
                text.push_str(&format!(" ({used}/{limit})"));
            }
        }
        text.push_str("\nReset times are not supplied by the audited quota contract.");
        text
    }
}

pub async fn fetch_usage(
    credentials: &ZaiCodingPlanCredentials,
) -> Result<ZaiCodingPlanUsageSnapshot, ZaiCodingPlanAuthError> {
    fetch_at(credentials, QUOTA_URL).await
}

async fn fetch_at(
    credentials: &ZaiCodingPlanCredentials,
    endpoint: &str,
) -> Result<ZaiCodingPlanUsageSnapshot, ZaiCodingPlanAuthError> {
    #[cfg(not(test))]
    if endpoint != QUOTA_URL {
        return Err(ZaiCodingPlanAuthError::InvalidResponse);
    }
    let http = xai_grok_provider_http::with_extra_root_certificates(reqwest::Client::builder())
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    // Unlike inference/MCP, the official usage plugin sends a bare API key.
    let mut authorization = HeaderValue::from_str(credentials.api_key())
        .map_err(|_| ZaiCodingPlanAuthError::InvalidCredential)?;
    authorization.set_sensitive(true);
    let response = http
        .get(endpoint)
        .header(AUTHORIZATION, authorization)
        .header("accept", "application/json")
        .header("accept-language", "en-US,en")
        .send()
        .await
        .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    if !response.status().is_success() {
        return Err(ZaiCodingPlanAuthError::Http(response.status()));
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_BYTES as u64)
    {
        return Err(ZaiCodingPlanAuthError::InvalidResponse);
    }
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
        if chunk.len() > MAX_BYTES.saturating_sub(bytes.len()) {
            return Err(ZaiCodingPlanAuthError::InvalidResponse);
        }
        bytes.extend_from_slice(&chunk);
    }
    parse_quota(
        &serde_json::from_slice(&bytes).map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?,
    )
}

fn parse_quota(
    value: &serde_json::Value,
) -> Result<ZaiCodingPlanUsageSnapshot, ZaiCodingPlanAuthError> {
    if value.get("success").and_then(serde_json::Value::as_bool) == Some(false)
        || value.get("code").is_some_and(|code| {
            !matches!(code.as_i64(), Some(0 | 200)) && !matches!(code.as_str(), Some("0" | "200"))
        })
    {
        return Err(ZaiCodingPlanAuthError::InvalidResponse);
    }
    let values = value
        .get("data")
        .unwrap_or(value)
        .get("limits")
        .and_then(serde_json::Value::as_array)
        .filter(|values| !values.is_empty() && values.len() <= 32)
        .ok_or(ZaiCodingPlanAuthError::InvalidResponse)?;
    let mut limits = Vec::new();
    for raw in values {
        let label = match raw.get("type").and_then(serde_json::Value::as_str) {
            Some("TOKENS_LIMIT") => "Model token quota",
            Some("CREDIT_LIMIT") => "Model credit quota",
            Some("TIME_LIMIT") => "MCP tool quota",
            _ => "Provider quota",
        };
        let used = raw.get("currentValue").and_then(serde_json::Value::as_u64);
        let limit = raw.get("usage").and_then(serde_json::Value::as_u64);
        let percentage = raw
            .get("percentage")
            .and_then(serde_json::Value::as_f64)
            .or_else(|| Some(used? as f64 / limit.filter(|limit| *limit > 0)? as f64 * 100.0))
            .filter(|value| value.is_finite() && (0.0..=100.0).contains(value))
            .ok_or(ZaiCodingPlanAuthError::InvalidResponse)?;
        limits.push(ZaiCodingPlanUsageRow {
            label: label.to_owned(),
            percentage,
            used,
            limit,
        });
    }
    Ok(ZaiCodingPlanUsageSnapshot { limits })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_preserves_reported_percentage_without_inventing_reset_or_account_data() {
        let quota = parse_quota(&serde_json::json!({"success": true, "data": {
            "accountId": "sentinel-private", "limits": [
                {"type": "TOKENS_LIMIT", "percentage": 42, "usage": 1000},
                {"type": "TIME_LIMIT", "percentage": 10, "currentValue": 10, "usage": 100},
                {"type": "CREDIT_LIMIT", "percentage": 55}
            ]
        }}))
        .unwrap();
        assert_eq!(quota.highest_used_percent(), Some(55.0));
        assert_eq!(quota.limits[0].used, None);
        assert!(!format!("{quota:?}").contains("sentinel-private"));
    }

    #[test]
    fn malformed_quota_is_not_reported_as_zero_usage() {
        for value in [
            serde_json::json!({"limits": []}),
            serde_json::json!({"limits": [{"type": "TOKENS_LIMIT"}]}),
            serde_json::json!({"limits": [{"percentage": -1}]}),
            serde_json::json!({"success": false, "message": "sentinel-secret"}),
        ] {
            assert!(parse_quota(&value).is_err());
        }
    }

    #[tokio::test]
    async fn quota_uses_official_bare_key_and_rejects_redirects() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let app = axum::Router::new().route(
            "/",
            axum::routing::get(|headers: axum::http::HeaderMap| async move {
                assert_eq!(headers["authorization"], "sentinel-zai");
                (
                    axum::http::StatusCode::FOUND,
                    [("location", "https://example.com")],
                )
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let key = ZaiCodingPlanCredentials::new("sentinel-zai").unwrap();
        assert!(matches!(
            fetch_at(&key, &endpoint).await,
            Err(ZaiCodingPlanAuthError::Http(_))
        ));
        server.abort();
    }
}
