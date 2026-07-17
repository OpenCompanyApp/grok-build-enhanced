use xai_grok_sampling_types::{Result, SamplingError};

/// Accept only the current ChatGPT Codex backend in production. Unit and
/// feature-gated test builds may use an exact loopback HTTP origin.
pub(crate) fn is_valid_base_url(base_url: &str) -> bool {
    let production =
        base_url.trim_end_matches('/') == xai_grok_sampling_types::OPENAI_CODEX_BASE_URL;
    #[cfg(any(test, feature = "test-support"))]
    let loopback = reqwest::Url::parse(base_url).ok().is_some_and(|url| {
        url.scheme() == "http"
            && url.query().is_none()
            && url.fragment().is_none()
            && url
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"))
    });
    #[cfg(not(any(test, feature = "test-support")))]
    let loopback = false;
    production || loopback
}

/// Build the dedicated Codex transport. Redirects are disabled so dynamic
/// subscription credentials can never be forwarded to another origin.
pub(crate) fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(SamplingError::Http)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_allowlist_accepts_production_and_exact_loopback_only() {
        assert!(is_valid_base_url(
            xai_grok_sampling_types::OPENAI_CODEX_BASE_URL
        ));
        assert!(is_valid_base_url(&format!(
            "{}/",
            xai_grok_sampling_types::OPENAI_CODEX_BASE_URL
        )));
        assert!(is_valid_base_url("http://127.0.0.1:8123"));
        assert!(is_valid_base_url("http://localhost:8123"));
        assert!(!is_valid_base_url("https://api.openai.com/v1"));
        assert!(!is_valid_base_url(
            "https://chatgpt.com/backend-api/codex.evil.test"
        ));
        assert!(!is_valid_base_url("http://127.0.0.1:8123?redirect=1"));
        assert!(!is_valid_base_url("http://127.0.0.1:8123/#fragment"));
    }

    #[tokio::test]
    async fn dedicated_client_never_reaches_redirect_sink() {
        use axum::Router;
        use axum::extract::State;
        use axum::http::{StatusCode, header};
        use axum::routing::{get, post};
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        async fn sink(State(hits): State<Arc<AtomicUsize>>) -> StatusCode {
            hits.fetch_add(1, Ordering::SeqCst);
            StatusCode::OK
        }

        let sink_hits = Arc::new(AtomicUsize::new(0));
        let app = Router::new()
            .route(
                "/start",
                post(|| async { (StatusCode::FOUND, [(header::LOCATION, "/credential-sink")]) }),
            )
            .route("/credential-sink", get(sink).post(sink))
            .with_state(Arc::clone(&sink_hits));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let mut credential = reqwest::header::HeaderValue::from_static("Bearer opaque-credential");
        credential.set_sensitive(true);
        let response = http_client()
            .unwrap()
            .post(format!("http://{address}/start"))
            .header(reqwest::header::AUTHORIZATION, credential)
            .send()
            .await
            .unwrap();
        server.abort();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(response.url().path(), "/start");
        assert_eq!(sink_hits.load(Ordering::SeqCst), 0);
    }
}
