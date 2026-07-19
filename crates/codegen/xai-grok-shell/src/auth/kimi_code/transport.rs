use futures_util::StreamExt as _;

use super::KimiCodeAuthError;

pub(super) fn http_client(
    timeout: std::time::Duration,
) -> Result<reqwest::Client, KimiCodeAuthError> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(timeout)
        .build()
        .map_err(|_| KimiCodeAuthError::InvalidResponse)
}

pub(super) async fn read_limited_body(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, KimiCodeAuthError> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(KimiCodeAuthError::InvalidResponse);
    }

    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| KimiCodeAuthError::InvalidResponse)?;
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(KimiCodeAuthError::InvalidResponse);
        }
        body.try_reserve_exact(chunk.len())
            .map_err(|_| KimiCodeAuthError::InvalidResponse)?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use axum::Router;
    use axum::body::{Body, Bytes};
    use axum::routing::get;

    use super::*;

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

        assert!(matches!(error, KimiCodeAuthError::InvalidResponse));
    }
}
