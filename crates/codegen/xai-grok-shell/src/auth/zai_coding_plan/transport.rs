use futures_util::StreamExt as _;

use super::ZaiCodingPlanAuthError;

pub(super) fn http_client(
    timeout: std::time::Duration,
) -> Result<reqwest::Client, ZaiCodingPlanAuthError> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(timeout)
        .build()
        .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)
}

pub(super) async fn read_limited_body(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, ZaiCodingPlanAuthError> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(ZaiCodingPlanAuthError::InvalidResponse);
    }

    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(ZaiCodingPlanAuthError::InvalidResponse);
        }
        body.try_reserve_exact(chunk.len())
            .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Return an unsuccessful Z.AI business code, including failures carried by
/// an HTTP 200 response. Successful `0` and `200` envelopes are ignored.
pub(super) fn business_code(value: &serde_json::Value) -> Option<i64> {
    let object = value.as_object()?;
    let success = object.get("success").and_then(serde_json::Value::as_bool);
    let code = object.get("code").and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
            .or_else(|| value.as_str().and_then(|value| value.parse::<i64>().ok()))
    });
    match (success, code) {
        (Some(false), Some(code)) => Some(code),
        (Some(false), None) => Some(-1),
        (_, Some(code)) if !matches!(code, 0 | 200) => Some(code),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use axum::Router;
    use axum::body::{Body, Bytes};
    use axum::routing::get;

    use super::*;

    #[test]
    fn business_failures_are_detected_inside_successful_http_envelopes() {
        assert_eq!(
            business_code(&serde_json::json!({"code": 1001, "success": false})),
            Some(1001)
        );
        assert_eq!(
            business_code(&serde_json::json!({"code": 200, "success": true})),
            None
        );
        assert_eq!(business_code(&serde_json::json!({"data": []})), None);
    }

    #[tokio::test]
    async fn chunked_response_is_rejected_when_it_crosses_the_body_limit() {
        let app = Router::new().route(
            "/chunked",
            get(|| async {
                Body::from_stream(futures_util::stream::iter([
                    Ok::<_, Infallible>(Bytes::from_static(b"1234")),
                    Ok::<_, Infallible>(Bytes::from_static(b"5678")),
                ]))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let response = http_client(std::time::Duration::from_secs(5))
            .unwrap()
            .get(format!("http://{address}/chunked"))
            .send()
            .await
            .unwrap();

        let error = read_limited_body(response, 7).await.unwrap_err();
        server.abort();
        assert!(matches!(error, ZaiCodingPlanAuthError::InvalidResponse));
    }
}
