//! Per-hop HTTP client builder for `web_fetch`.
//!
//! Every direct request uses a fresh client whose resolver is pinned to the
//! addresses that passed SSRF validation for that exact hop. A fresh pool also
//! prevents a connection validated for one hop from being reused after a
//! redirect or later DNS change.

use super::config::WebFetchParams;
use super::error::WebFetchError;
use super::ssrf::ValidatedTarget;

/// Validated parameters used to build a fresh, DNS-pinned client per hop.
#[derive(Clone, Debug)]
pub(crate) struct HttpClient {
    params: WebFetchParams,
}

impl HttpClient {
    pub(crate) fn new(params: &WebFetchParams) -> Result<Self, WebFetchError> {
        // Validate TLS/proxy/client configuration at tool construction time.
        let _ = Self::build(params)?;
        Ok(Self {
            params: params.clone(),
        })
    }

    /// Build a one-hop client whose DNS map is pinned to the exact address set
    /// that passed SSRF validation. The original hostname remains in the URL,
    /// preserving HTTP Host and TLS SNI/certificate verification.
    pub(crate) fn build_for_target(
        &self,
        target: &ValidatedTarget,
    ) -> Result<reqwest::Client, WebFetchError> {
        Self::build_with_target(&self.params, Some(target))
    }

    fn build(params: &WebFetchParams) -> Result<reqwest::Client, WebFetchError> {
        Self::build_with_target(params, None)
    }

    fn build_with_target(
        params: &WebFetchParams,
        target: Option<&ValidatedTarget>,
    ) -> Result<reqwest::Client, WebFetchError> {
        let mut builder = reqwest::Client::builder()
            .timeout(params.timeout_secs())
            .connect_timeout(std::time::Duration::from_secs(10))
            // We manage redirects for SSRF.
            .redirect(reqwest::redirect::Policy::none())
            .pool_max_idle_per_host(2)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .tcp_nodelay(true)
            // Reduce size of incoming payloads.
            .gzip(true)
            .brotli(true)
            .deflate(true);

        if let Some(target) = target
            && target.host.parse::<std::net::IpAddr>().is_err()
        {
            builder = builder.resolve_to_addrs(&target.host, &target.addrs);
        }

        // Route all traffic through the egress proxy when configured. A
        // configured proxy is an explicit trusted egress boundary; direct
        // connections use the pinned resolver above.
        if let Some(ref endpoint) = params.proxy_endpoint {
            let proxy =
                reqwest::Proxy::all(endpoint).map_err(|_| WebFetchError::ProxyConfigError)?;
            builder = builder.proxy(proxy);
        } else {
            // DNS pinning protects direct egress only. Do not silently adopt an
            // ambient process proxy that would perform a second, unvalidated
            // target lookup; proxies must be configured explicitly above.
            builder = builder.no_proxy();
        }

        builder.build().map_err(|_| WebFetchError::ClientBuildError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validated_hostname_connects_only_to_the_pinned_address() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .await
                .unwrap();
        });
        let target = ValidatedTarget {
            host: "validated-target.invalid".to_owned(),
            addrs: vec![address],
        };
        let managed = HttpClient::new(&WebFetchParams::default()).unwrap();
        let client = managed.build_for_target(&target).unwrap();

        let response = client
            .get(format!(
                "http://validated-target.invalid:{}/",
                address.port()
            ))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::OK);
        server.await.unwrap();
    }

    #[test]
    fn build_with_proxy_endpoint() {
        let params = WebFetchParams {
            proxy_endpoint: Some("https://proxy.corp.example.com".into()),
            ..Default::default()
        };
        // Should succeed — reqwest accepts the proxy URL.
        let client = HttpClient::new(&params);
        assert!(client.is_ok());
    }

    #[test]
    fn build_without_proxy_is_default() {
        let params = WebFetchParams::default();
        assert!(params.proxy_endpoint.is_none());
        let client = HttpClient::new(&params);
        assert!(client.is_ok());
    }

    #[test]
    fn build_with_invalid_proxy_endpoint() {
        let params = WebFetchParams {
            proxy_endpoint: Some("not a valid url".into()),
            ..Default::default()
        };
        let result = HttpClient::new(&params);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("proxy"),
            "Expected proxy-related error, got: {err}"
        );
    }
}
