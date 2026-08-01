/// Structured errors for the `web_fetch` tool.
use std::net::IpAddr;

/// Reduce an HTTP(S) URL to its origin before it crosses an error, UI, hook,
/// or logging boundary. URL userinfo, paths, queries, and fragments commonly
/// contain credentials or private identifiers and are never diagnostic data.
pub fn safe_url_origin(raw_url: &str) -> String {
    url::Url::parse(raw_url)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
        .map(|url| url.origin().ascii_serialization())
        .unwrap_or_else(|| "<redacted-url>".to_owned())
}

#[derive(Debug, thiserror::Error)]
pub enum WebFetchError {
    #[error("URL exceeds maximum length of {max} characters")]
    UrlTooLong { max: usize },

    #[error("unsupported URL scheme: {scheme} (only http/https allowed)")]
    UnsupportedScheme { scheme: String },

    #[error("URLs with embedded credentials are not allowed")]
    CredentialsInUrl,

    #[error("hostname must have at least two dot-separated parts, got: {host}")]
    SingleLabelHost { host: String },

    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("SSRF blocked: {host} resolves to private/internal IP {ip}{}", ssrf_recovery_hint(.host))]
    SsrfBlocked { host: String, ip: IpAddr },

    #[error("DNS resolution failed for {host}: {source}")]
    DnsResolution {
        host: String,
        source: std::io::Error,
    },

    #[error("DNS resolution returned no addresses for {0}")]
    DnsEmpty(String),

    #[error("failed to build HTTP client")]
    ClientBuildError,

    #[error("Kimi Code hosted fetch authentication was rejected")]
    HostedAuthentication,

    #[error("Kimi Code hosted fetch is not included in this membership")]
    HostedMembership,

    #[error("Kimi Code hosted fetch rejected the validated request (HTTP {status})")]
    HostedRequestRejected { status: u16 },

    #[error("Kimi Code hosted fetch quota or concurrency limit was reached")]
    HostedQuota,

    #[error("HTTP request to {origin} failed ({kind})")]
    HttpRequest { origin: String, kind: &'static str },

    #[error("invalid redirect URL")]
    InvalidRedirect,

    #[error("too many redirects (max {max})")]
    TooManyRedirects { max: usize },

    #[error("response body exceeds maximum size of {max} bytes")]
    ResponseTooLarge { max: usize },

    #[error("invalid proxy configuration")]
    ProxyConfigError,

    #[error("failed to save downloaded file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("unsupported content type {content_type} from {url}")]
    UnsupportedContentType { content_type: String, url: String },

    #[error("content body does not match claimed content type {content_type} from {url}")]
    ContentTypeMismatch { content_type: String, url: String },
}

impl WebFetchError {
    pub fn failure_code(&self) -> crate::types::output::WebToolErrorCode {
        use crate::types::output::WebToolErrorCode;
        match self {
            Self::SsrfBlocked { .. } => WebToolErrorCode::SsrfBlocked,
            Self::ResponseTooLarge { .. } => WebToolErrorCode::ResponseTooLarge,
            Self::UnsupportedContentType { .. } | Self::ContentTypeMismatch { .. } => {
                WebToolErrorCode::UnsupportedContent
            }
            Self::HttpRequest {
                kind: "timeout", ..
            } => WebToolErrorCode::Timeout,
            Self::InvalidRedirect | Self::TooManyRedirects { .. } => {
                WebToolErrorCode::CrossHostRedirect
            }
            Self::UrlTooLong { .. }
            | Self::UnsupportedScheme { .. }
            | Self::CredentialsInUrl
            | Self::SingleLabelHost { .. }
            | Self::InvalidUrl(_) => WebToolErrorCode::DomainRejected,
            Self::DnsResolution { .. } | Self::DnsEmpty(_) => WebToolErrorCode::DnsResolutionFailed,
            Self::ClientBuildError
            | Self::HostedAuthentication
            | Self::HostedMembership
            | Self::HostedRequestRejected { .. }
            | Self::HostedQuota
            | Self::HttpRequest { .. }
            | Self::ProxyConfigError
            | Self::IoError(_) => WebToolErrorCode::ProviderUnavailable,
        }
    }

    pub fn into_tool_error(self) -> xai_tool_runtime::ToolError {
        use crate::types::output::{WebToolErrorCode, WebToolFailure};
        match self {
            Self::HostedAuthentication => xai_tool_runtime::ToolError::unauthorized(
                "Kimi Code hosted fetch API key was rejected".to_owned(),
            )
            .with_details(serde_json::json!({
                "tool_id": "web_fetch",
                "status": 401,
                "auth_recovery_provider": crate::types::KIMI_CODE_PROVIDER_ID,
                "auth_recovery_exhausted": true,
            })),
            Self::HostedMembership => xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Execution,
                "Kimi Code hosted fetch is not included in this membership".to_owned(),
            )
            .with_details(serde_json::json!({
                "tool_id": "web_fetch",
                "code": "membership_unavailable",
            })),
            Self::HostedRequestRejected { status } => xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Execution,
                "Kimi Code hosted fetch rejected the validated request".to_owned(),
            )
            .with_details(serde_json::json!({
                "tool_id": "web_fetch",
                "code": "hosted_request_rejected",
                "status": status,
            })),
            Self::HostedQuota => xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Execution,
                "Kimi Code hosted fetch quota or concurrency limit was reached".to_owned(),
            )
            .with_details(serde_json::json!({
                "tool_id": "web_fetch",
                "code": "quota_exhausted",
            })),
            error => {
                let code = error.failure_code();
                let failure = WebToolFailure::for_code(code);
                let kind = match code {
                    WebToolErrorCode::Timeout => xai_tool_runtime::ToolErrorKind::Timeout,
                    WebToolErrorCode::ProviderUnavailable => {
                        xai_tool_runtime::ToolErrorKind::NetworkError
                    }
                    WebToolErrorCode::DomainRejected
                    | WebToolErrorCode::DnsResolutionFailed
                    | WebToolErrorCode::SsrfBlocked
                    | WebToolErrorCode::CrossHostRedirect
                    | WebToolErrorCode::UnsupportedContent
                    | WebToolErrorCode::ResponseTooLarge => {
                        xai_tool_runtime::ToolErrorKind::Execution
                    }
                    _ => xai_tool_runtime::ToolErrorKind::Execution,
                };
                xai_tool_runtime::ToolError::new(kind, failure.prompt_text()).with_details(
                    serde_json::to_value(failure)
                        .expect("web failure envelope is JSON-serializable"),
                )
            }
        }
    }

    /// Convert a reqwest failure into a bounded, URL-safe diagnostic. The
    /// original reqwest error is deliberately not retained because both its
    /// Display and Debug implementations may include the full request URL.
    pub(crate) fn http_request(error: reqwest::Error, fallback_url: &str) -> Self {
        let origin = error
            .url()
            .map(url::Url::as_str)
            .map(safe_url_origin)
            .unwrap_or_else(|| safe_url_origin(fallback_url));
        let kind = if error.is_timeout() {
            "timeout"
        } else if error.is_connect() {
            "connection"
        } else if error.status().is_some() {
            "status"
        } else {
            "transport"
        };
        Self::HttpRequest { origin, kind }
    }
}

/// Extra recovery guidance appended to an [`WebFetchError::SsrfBlocked`] message.
///
/// `web_fetch` can't reach internal/private hosts, but GitHub / GitHub
/// Enterprise hosts (including internal GHE hostnames) are reachable via the
/// authenticated `gh` CLI. When the blocked host looks like GitHub **and `gh`
/// is actually installed**, point the agent at `gh` instead of letting it
/// conclude the resource is inaccessible and give up. If `gh` is not on `PATH`
/// (or the host isn't GitHub), fall back to the bare SSRF message by returning
/// an empty string.
fn ssrf_recovery_hint(host: &str) -> &'static str {
    if is_github_host(host) && gh_available() {
        ". Use the `gh` CLI instead (e.g. `gh pr view` or `gh api`)."
    } else {
        ""
    }
}

/// Whether `host` is a GitHub / GitHub Enterprise host (one `gh` can reach).
fn is_github_host(host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    h == "github.com" || h.ends_with(".github.com") || h.contains("github")
}

/// Whether the `gh` CLI is available on `PATH`, via the same `which` lookup the
/// rest of the codebase uses for binary discovery (e.g. `xai-grok-mcp`).
fn gh_available() -> bool {
    which::which("gh").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::output::{WebToolErrorCode, WebToolFailure};

    #[test]
    fn dns_failures_are_non_retryable_destination_errors() {
        let failures = [
            WebFetchError::DnsResolution {
                host: "missing.invalid".to_owned(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
            },
            WebFetchError::DnsEmpty("missing.invalid".to_owned()),
        ];

        for error in failures {
            let tool_error = error.into_tool_error();
            let failure: WebToolFailure = serde_json::from_value(
                tool_error
                    .details
                    .expect("DNS failure must include a structured failure envelope"),
            )
            .unwrap();

            assert_eq!(tool_error.kind, xai_tool_runtime::ToolErrorKind::Execution);
            assert_eq!(failure.code, WebToolErrorCode::DnsResolutionFailed);
            assert!(!failure.retryable);
        }
    }

    #[test]
    fn github_host_detection() {
        assert!(is_github_host("github.com"));
        assert!(is_github_host("api.github.com"));
        assert!(is_github_host("github.ghe.example.com")); // synthetic GHE-style
        assert!(!is_github_host("ghe.example.com"));
        assert!(!is_github_host("internal-wiki.corp.example.com"));
        assert!(!is_github_host("gitlab.example.com"));
    }

    #[test]
    fn safe_url_origin_strips_userinfo_path_query_and_fragment() {
        let safe = safe_url_origin(
            "https://user:password@example.com:8443/private/object?token=super-secret#fragment",
        );
        assert_eq!(safe, "https://example.com:8443");
        for secret in [
            "user",
            "password",
            "private",
            "object",
            "token",
            "super-secret",
            "fragment",
        ] {
            assert!(!safe.contains(secret), "origin leaked {secret}: {safe}");
        }
        assert_eq!(safe_url_origin("not a URL"), "<redacted-url>");
    }

    #[test]
    fn content_errors_expose_only_pre_sanitized_origin() {
        let origin = safe_url_origin(
            "https://user:password@example.com/private?token=super-secret#fragment",
        );
        for message in [
            WebFetchError::UnsupportedContentType {
                content_type: "application/octet-stream".to_owned(),
                url: origin.clone(),
            }
            .to_string(),
            WebFetchError::ContentTypeMismatch {
                content_type: "image/png".to_owned(),
                url: origin.clone(),
            }
            .to_string(),
        ] {
            assert!(message.contains("https://example.com"));
            for secret in ["user", "password", "private", "token", "super-secret"] {
                assert!(
                    !message.contains(secret),
                    "error leaked {secret}: {message}"
                );
            }
        }
    }

    #[test]
    fn which_detects_gh_in_dir() {
        // Exercises the same `which` lookup `gh_available` uses, with a
        // controlled search dir so it doesn't depend on the test host's PATH.
        let dir = tempfile::tempdir().unwrap();
        // No gh in this dir yet.
        assert!(which::which_in("gh", Some(dir.path()), dir.path()).is_err());
        // Create an executable `gh`.
        let gh = dir.path().join("gh");
        std::fs::write(&gh, b"#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(which::which_in("gh", Some(dir.path()), dir.path()).is_ok());
    }

    #[test]
    fn ssrf_non_github_host_never_hints() {
        let err = WebFetchError::SsrfBlocked {
            host: "internal-wiki.corp.example.com".to_string(),
            ip: "10.0.0.5".parse().unwrap(),
        };
        let msg = err.to_string();
        assert!(msg.contains("resolves to private/internal IP 10.0.0.5"));
        assert!(
            !msg.contains("gh"),
            "non-github host should not mention gh: {msg}"
        );
    }

    #[test]
    fn ssrf_github_host_hint_follows_gh_availability() {
        let err = WebFetchError::SsrfBlocked {
            // Synthetic host must contain "github" for is_github_host; IP is RFC1918 example.
            host: "github.ghe.example.com".to_string(),
            ip: "10.0.0.1".parse().unwrap(),
        };
        let msg = err.to_string();
        assert!(msg.contains("resolves to private/internal IP 10.0.0.1"));
        if gh_available() {
            assert!(msg.contains("`gh` CLI"), "gh present -> should hint: {msg}");
            assert!(msg.contains("gh pr view") && msg.contains("gh api"));
        } else {
            // Host names like "github…" contain the substring "gh"; assert on the
            // hint markers only, not a bare "gh" contains check.
            assert!(
                !msg.contains("`gh` CLI") && !msg.contains("gh pr view") && !msg.contains("gh api"),
                "gh absent -> previous behavior, no hint: {msg}"
            );
        }
    }
}
