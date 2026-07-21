//! `image_gen` tool — generates images via the xAI Imagine API and saves
//! them to the local filesystem so the model can reference them in code
//! (e.g. `<img src="images/hero.jpg">`).
//!
//! Architecture follows the same pattern as `web_search`:
//!
//! - [`ImageGenConfig`] is built from session credentials by the host and
//!   injected into the tool registry.
//! - When `Enabled`, an [`ImageGenClient`] is constructed once and injected
//!   into `Resources`. The tool reads it at runtime via `resources.require()`.
//! - When `Disabled`, the tool is not registered so the model never sees it.
//!
//! The generated image is written to `<session_folder>/images/<n>.jpg`
//! where `<n>` is a session-scoped counter (1, 2, 3, ... — 1 token each).
//! The tool returns the absolute path so the model can copy or move the
//! image into the project working directory when it needs a persistent asset.

use base64::Engine as _;
use image::ImageReader;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};

use crate::attribution::{SharedAttributionCallback, ToolConsumer};
#[cfg(test)]
use crate::types::api_key_provider::validate_openai_codex_request_auth;
use crate::types::api_key_provider::{
    OPENAI_CODEX_PROVIDER_ID, require_openai_codex_auth_provider, resolve_openai_codex_request_auth,
};
use crate::types::{RequestAuth, RequestCredentialSnapshot, SharedApiKeyProvider};

use crate::types::output::{MediaGenOutput, ToolOutput};
use crate::types::requirements::{Expr, ToolRequirement};
use crate::types::resources::SessionFolder;
use crate::types::tool::{ToolKind, ToolNamespace};

/// Default Imagine model for `image_gen`. Used unless an explicit
/// `model_override` is supplied via `ImageGenConfig::Enabled`.
const XAI_IMAGINE_MODEL: &str = "grok-imagine-image-quality";
/// Current ChatGPT Codex image model, verified against openai/codex at
/// commit f737605606c14e3aa59a4c17be80d338f164dff5 (2026-07-16).
/// This backend is experimental and the model is deliberately isolated from
/// the coding-model catalog so an upstream change has one update point.
pub(crate) const OPENAI_CODEX_IMAGE_MODEL: &str = "gpt-image-2";
// Some Imagine models (e.g. `grok-imagine-image`, selectable via `model_override`)
// expand the prompt then generate, and the proxy buffers
// the whole image before sending any bytes — so the client may receive nothing
// for well over a minute. Keep these generous so a slow-but-progressing
// generation isn't cut off.
const IMAGE_GEN_TIMEOUT_SECS: u64 = 300;
const IMAGE_GEN_READ_TIMEOUT_SECS: u64 = 240;
const DEFAULT_IMAGE_DIR: &str = "images";
const OPENAI_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const MAX_IMAGE_RESPONSE_BODY_BYTES: usize = 48 * 1024 * 1024;
const MAX_GENERATED_IMAGE_BYTES: usize = 32 * 1024 * 1024;
const MAX_GENERATED_IMAGE_PIXELS: u64 = 40_000_000;
const MAX_GENERATED_IMAGE_DIMENSION: u32 = 8192;

pub use xai_grok_tools_api::slash_commands::{
    IMAGE_GEN_TOOL_NAME, IMAGINE_COMMAND_NAME, imagine_instruction, imagine_usage_message,
};

/// Prose returned to the model (as a normal, successful tool result) when a
/// free / X Basic user calls `image_gen` or `image_edit`. The model relays it
/// to the user. The deliberate `/imagine` slash command shows the richer
/// SuperGrok upsell modal instead; this covers the natural-language path.
pub(crate) const TIER_RESTRICTED_UPSELL: &str = "Image generation is a SuperGrok feature and isn't available on the free or X Basic tier. Let the user know they can unlock image and video generation by upgrading to SuperGrok: https://grok.com/supergrok?referrer=grok-build. Do not retry this tool.";

/// HTTP client for xAI Imagine API. Cloned per-request; shares `Arc` state.
#[derive(Clone)]
pub struct ImageGenClient {
    /// Existing eager xAI transport. Codex resolves routes asynchronously from
    /// each final request URL without blocking client construction.
    http: Option<reqwest::Client>,
    codex_http: Option<xai_grok_provider_http::OpenAiCodexClientPool>,
    base_url: String,
    /// Imagine model slug used by `generate()`. Selected at construction
    /// from `ImageGenConfig::model_override` (falling back to
    /// [`XAI_IMAGINE_MODEL`]). `image_edit` uses its own model and is
    /// unaffected.
    model: String,
    backend: ImageGenBackend,
    /// Static fallback exists only for the legacy xAI backend. Codex auth is
    /// always resolved dynamically so an access token is never treated as a
    /// permanent image API key.
    fallback_api_key: Option<String>,
    writer: super::storage::SessionFileWriter,
    api_key_provider: Option<SharedApiKeyProvider>,
    /// Optional 401-attribution hook. Hosts wire this so a 401 from the
    /// Imagine API emits an `auth_401_attribution` event with
    /// `consumer == "ImageGen"` for unified auth-failure telemetry.
    attribution_callback: Option<SharedAttributionCallback>,
    /// When `true`, the user is on a tier the Imagine server zero-limits
    /// (free / X Basic). `image_gen` / `image_edit` short-circuit before any
    /// HTTP call and return the SuperGrok upsell prose instead. See
    /// [`ImageGenClient::is_tier_restricted`].
    tier_restricted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageGenBackend {
    XaiImagine,
    OpenAiCodex,
}

impl ImageGenClient {
    pub fn new(
        config: &ImageGenConfig,
        api_key_provider: Option<SharedApiKeyProvider>,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        let (base_url, model, backend, fallback_api_key, extra_headers, tier_restricted) =
            match config {
                ImageGenConfig::Enabled {
                    api_key,
                    base_url,
                    extra_headers,
                    model_override,
                    tier_restricted,
                    ..
                } => (
                    validate_xai_image_base_url(base_url)?,
                    model_override
                        .clone()
                        .filter(|m| !m.trim().is_empty())
                        .unwrap_or_else(|| XAI_IMAGINE_MODEL.to_owned()),
                    ImageGenBackend::XaiImagine,
                    Some(api_key.clone()),
                    extra_headers.clone(),
                    *tier_restricted,
                ),
                ImageGenConfig::OpenAiCodex { base_url, .. } => (
                    validate_codex_base_url(base_url)?,
                    OPENAI_CODEX_IMAGE_MODEL.to_owned(),
                    ImageGenBackend::OpenAiCodex,
                    None,
                    indexmap::IndexMap::new(),
                    false,
                ),
                ImageGenConfig::Disabled | ImageGenConfig::Unavailable { .. } => {
                    return Err(xai_tool_runtime::ToolError::invalid_arguments(
                        "Cannot create ImageGenClient without provider credentials",
                    ));
                }
            };

        if backend == ImageGenBackend::OpenAiCodex {
            let provider = api_key_provider.as_ref().ok_or_else(|| {
                xai_tool_runtime::ToolError::invalid_arguments(
                    "Cannot create a Codex image client without dynamic provider authentication",
                )
            })?;
            require_openai_codex_auth_provider(provider).map_err(codex_image_auth_error)?;
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if backend == ImageGenBackend::OpenAiCodex {
            headers.insert(
                reqwest::header::USER_AGENT,
                HeaderValue::from_str(&format!("grok-build-codex/{}", xai_grok_version::VERSION))
                    .map_err(|_| {
                    xai_tool_runtime::ToolError::invalid_arguments(
                        "Invalid Codex image client version",
                    )
                })?,
            );
            headers.insert(
                reqwest::header::HeaderName::from_static("originator"),
                HeaderValue::from_static("grok_build_codex"),
            );
            headers.insert(
                reqwest::header::HeaderName::from_static("version"),
                HeaderValue::from_str(xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION)
                    .map_err(|_| {
                        xai_tool_runtime::ToolError::invalid_arguments(
                            "Invalid Codex image client version",
                        )
                    })?,
            );
        }
        if let Some(api_key) = fallback_api_key.as_deref() {
            // Legacy xAI fallback. Managed Codex auth is never installed as a
            // default client header and is resolved anew for every attempt.
            let mut authorization =
                HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|_| {
                    xai_tool_runtime::ToolError::invalid_arguments(
                        "Image provider authentication is invalid",
                    )
                })?;
            authorization.set_sensitive(true);
            headers.insert(AUTHORIZATION, authorization);
        }

        extra_headers.into_iter().try_for_each(|(key, value)| {
            let header_name =
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|_| {
                    xai_tool_runtime::ToolError::invalid_arguments(
                        "Image provider headers are invalid",
                    )
                })?;
            let mut header_value = HeaderValue::from_str(&value).map_err(|_| {
                xai_tool_runtime::ToolError::invalid_arguments("Image provider headers are invalid")
            })?;
            header_value.set_sensitive(true);
            headers.insert(header_name, header_value);
            Ok::<(), xai_tool_runtime::ToolError>(())
        })?;

        let http = if backend == ImageGenBackend::OpenAiCodex {
            None
        } else {
            Some(
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(IMAGE_GEN_TIMEOUT_SECS))
                    .read_timeout(std::time::Duration::from_secs(IMAGE_GEN_READ_TIMEOUT_SECS))
                    .redirect(reqwest::redirect::Policy::none())
                    .default_headers(headers.clone())
                    .build()
                    .map_err(|_| {
                        xai_tool_runtime::ToolError::invalid_arguments(
                            "Image provider HTTP client could not be built",
                        )
                    })?,
            )
        };
        let codex_http = (backend == ImageGenBackend::OpenAiCodex).then(|| {
            let headers = headers.clone();
            xai_grok_provider_http::OpenAiCodexClientPool::new(
                xai_grok_provider_http::ClientRouteClass::Api,
                move || {
                    reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(IMAGE_GEN_TIMEOUT_SECS))
                        .read_timeout(std::time::Duration::from_secs(IMAGE_GEN_READ_TIMEOUT_SECS))
                        .redirect(reqwest::redirect::Policy::none())
                        .default_headers(headers.clone())
                },
            )
        });

        Ok(Self {
            http,
            codex_http,
            base_url,
            model,
            backend,
            fallback_api_key,
            writer: super::storage::SessionFileWriter::new(DEFAULT_IMAGE_DIR, "jpg"),
            api_key_provider,
            attribution_callback: None,
            tier_restricted,
        })
    }

    async fn http(
        &self,
        request_url: &str,
    ) -> Result<reqwest::Client, xai_tool_runtime::ToolError> {
        if self.backend != ImageGenBackend::OpenAiCodex {
            return self.http.as_ref().cloned().ok_or_else(|| {
                xai_tool_runtime::ToolError::new(
                    xai_tool_runtime::ToolErrorKind::ServiceUnavailable,
                    "Image provider is temporarily unavailable.",
                )
            });
        }
        self.codex_http
            .as_ref()
            .ok_or_else(|| {
                xai_tool_runtime::ToolError::new(
                    xai_tool_runtime::ToolErrorKind::ServiceUnavailable,
                    "Image provider is temporarily unavailable.",
                )
            })?
            .client_for_url(request_url)
            .await
            .map_err(|_| {
                xai_tool_runtime::ToolError::new(
                    xai_tool_runtime::ToolErrorKind::ServiceUnavailable,
                    "Image provider is temporarily unavailable.",
                )
            })
    }

    /// Whether the current user's tier (free / X Basic) is zero-limited on
    /// Imagine server-side. `image_gen` / `image_edit` use this to short-circuit
    /// with the SuperGrok upsell instead of issuing a doomed request.
    pub(crate) fn is_tier_restricted(&self) -> bool {
        self.tier_restricted
    }

    /// Wire a 401-attribution callback into this client. Idempotent;
    /// safe to call before or after the first request. Builder-style
    /// so `new()` callers that don't care can ignore it.
    pub fn with_attribution_callback(
        mut self,
        callback: Option<SharedAttributionCallback>,
    ) -> Self {
        self.attribution_callback = callback;
        self
    }

    pub(crate) fn backend(&self) -> ImageGenBackend {
        self.backend
    }

    async fn current_request_auth(&self) -> Result<RequestAuth, xai_tool_runtime::ToolError> {
        debug_assert_eq!(self.backend, ImageGenBackend::XaiImagine);
        let auth = crate::types::api_key_provider::resolve_request_auth(
            self.api_key_provider.as_ref(),
        )
        .await
        .or_else(|| self.fallback_api_key.clone().map(RequestAuth::bearer))
        .ok_or_else(|| {
            xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Custom,
                "Image provider authentication is required. Sign in to the selected provider and retry.",
            )
            .with_details(serde_json::json!({"code": "auth_required"}))
        })?;

        if auth.provider().is_some() {
            return Err(xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Custom,
                "Refusing provider-scoped credentials for the xAI image service.",
            )
            .with_details(serde_json::json!({"code": "auth_provider_mismatch"})));
        }
        Ok(auth)
    }

    fn apply_request_auth(
        &self,
        request: reqwest::RequestBuilder,
        auth: &RequestAuth,
    ) -> Result<reqwest::RequestBuilder, xai_tool_runtime::ToolError> {
        debug_assert_eq!(self.backend, ImageGenBackend::XaiImagine);
        if auth.provider().is_some() {
            return Err(xai_image_auth_error("auth_provider_mismatch"));
        }

        let mut headers = auth.headers();
        let Some((name, value)) = headers.next() else {
            return Err(xai_image_auth_error("auth_required"));
        };
        if headers.next().is_some()
            || !name.eq_ignore_ascii_case("authorization")
            || value
                .strip_prefix("Bearer ")
                .is_none_or(|token| token.trim().is_empty())
        {
            return Err(xai_image_auth_error("auth_provider_mismatch"));
        }
        let mut value = HeaderValue::from_str(value)
            .map_err(|_| xai_image_auth_error("auth_provider_mismatch"))?;
        value.set_sensitive(true);
        Ok(request.header(AUTHORIZATION, value))
    }

    async fn send_json_once(
        &self,
        url: &str,
        payload: &serde_json::Value,
    ) -> Result<(reqwest::Response, Option<RequestCredentialSnapshot>), xai_tool_runtime::ToolError>
    {
        let request = self.http(url).await?.post(url).json(payload);
        let (request, credential_snapshot) = match self.backend {
            ImageGenBackend::OpenAiCodex => {
                let provider = self.api_key_provider.as_ref().ok_or_else(|| {
                    codex_image_auth_error(
                        crate::types::api_key_provider::OpenAiCodexAuthError::Unavailable,
                    )
                })?;
                let auth = resolve_openai_codex_request_auth(provider)
                    .await
                    .map_err(codex_image_auth_error)?;
                let credential_snapshot = Some(auth.credential_snapshot().clone());
                (auth.apply(request), credential_snapshot)
            }
            ImageGenBackend::XaiImagine => {
                let auth = self.current_request_auth().await?;
                let credential_snapshot = auth.credential_snapshot().cloned();
                (
                    self.apply_request_auth(request, &auth)?,
                    credential_snapshot,
                )
            }
        };
        let response = request.send().await.map_err(|_| {
            xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Custom,
                "Image provider request could not be sent.",
            )
            .with_details(serde_json::json!({"code": "transport_failure"}))
        })?;
        Ok((response, credential_snapshot))
    }

    pub(crate) async fn post_json(
        &self,
        path: &str,
        payload: &serde_json::Value,
    ) -> Result<reqwest::Response, xai_tool_runtime::ToolError> {
        if self.backend == ImageGenBackend::OpenAiCodex
            && !matches!(
                path.trim_matches('/'),
                "images/generations" | "images/edits"
            )
        {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "Refusing an unsupported Codex image endpoint.",
            ));
        }
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{base}/{}", path.trim_start_matches('/'));
        let (mut response, rejected_credential) = self.send_json_once(&url, payload).await?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            // Attribution intentionally receives no credential material.
            if self.backend == ImageGenBackend::XaiImagine {
                self.record_401_attribution(ToolConsumer::ImageGen, None);
            }
            let recovered = match self.api_key_provider.as_ref() {
                Some(provider) => {
                    provider
                        .recover_unauthorized_async(rejected_credential)
                        .await
                }
                None => false,
            };
            if recovered {
                response = self.send_json_once(&url, payload).await?.0;
            }
        }
        Ok(response)
    }

    pub(crate) fn record_401_attribution(&self, consumer: ToolConsumer, sent_bearer: Option<&str>) {
        crate::attribution::emit_401(self.attribution_callback.as_ref(), consumer, sent_bearer);
    }

    pub(crate) fn writer(&self) -> &super::storage::SessionFileWriter {
        &self.writer
    }

    /// Structured details for a terminal provider response. A Codex 401 has
    /// already consumed the provider-scoped reload/refresh/replay path in
    /// [`Self::post_json`], so mark it as owned and exhausted. The shell uses
    /// this signal to prevent a second retry through xAI's `AuthManager`.
    pub(crate) fn http_failure_details(&self, status: reqwest::StatusCode) -> serde_json::Value {
        let mut details = serde_json::json!({
            "code": "http_failure",
            "status": status.as_u16(),
        });
        if self.backend == ImageGenBackend::OpenAiCodex
            && status == reqwest::StatusCode::UNAUTHORIZED
        {
            details[crate::types::AUTH_RECOVERY_PROVIDER_DETAILS_KEY] =
                serde_json::Value::String(OPENAI_CODEX_PROVIDER_ID.to_owned());
            details[crate::types::AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY] =
                serde_json::Value::Bool(true);
        }
        details
    }

    pub async fn generate(
        &self,
        prompt: &str,
        aspect_ratio: &str,
    ) -> Result<Vec<u8>, xai_tool_runtime::ToolError> {
        let payload = match self.backend {
            ImageGenBackend::XaiImagine => serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "n": 1,
                "aspect_ratio": aspect_ratio,
                "resolution": "1k",
                "response_format": "b64_json",
            }),
            ImageGenBackend::OpenAiCodex => serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "background": "auto",
                "quality": "auto",
                "size": codex_image_size(aspect_ratio),
            }),
        };

        let response = self.post_json("images/generations", &payload).await?;

        let status = response.status();
        if !status.is_success() {
            tracing::warn!(http_status = %status, "image provider generation request failed");
            return Err(xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Custom,
                format!("Image generation failed with HTTP {status}"),
            )
            .with_details(self.http_failure_details(status)));
        }

        let resp_json = read_image_response(response).await?;

        let b64_data = resp_json.b64_data().unwrap_or("");

        if b64_data.is_empty() {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "Image generation returned no image data.",
            ));
        }

        if b64_data.len() > MAX_GENERATED_IMAGE_BYTES.saturating_mul(4) / 3 + 8 {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "Image generation response exceeded the configured size limit.",
            ));
        }
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(b64_data)
            .map_err(|e| {
                xai_tool_runtime::ToolError::invalid_arguments(format!(
                    "Failed to decode base64 image data: {e}"
                ))
            })?;
        validate_generated_image(&decoded)?;
        Ok(decoded)
    }
}

/// Map Grok's aspect-ratio vocabulary to exact `gpt-image-2` canvases.
///
/// The current image endpoint accepts flexible dimensions when both edges are
/// multiples of 16, neither edge exceeds 3840px, the ratio is at most 3:1, and
/// the total pixel count is within the model's documented bounds. These sizes
/// stay close to one megapixel while preserving every advertised ratio.
pub(crate) fn codex_image_size(aspect_ratio: &str) -> &'static str {
    match aspect_ratio.trim() {
        "1:1" => "1024x1024",
        "16:9" => "1280x720",
        "9:16" => "720x1280",
        "4:3" => "1024x768",
        "3:4" => "768x1024",
        "3:2" => "1152x768",
        "2:3" => "768x1152",
        "2:1" => "1152x576",
        "1:2" => "576x1152",
        "19.5:9" => "1248x576",
        "9:19.5" => "576x1248",
        "20:9" => "1280x576",
        "9:20" => "576x1280",
        _ => "auto",
    }
}

fn xai_image_auth_error(code: &'static str) -> xai_tool_runtime::ToolError {
    xai_tool_runtime::ToolError::new(
        xai_tool_runtime::ToolErrorKind::Custom,
        "Image provider authentication is unavailable.",
    )
    .with_details(serde_json::json!({"code": code}))
}

fn codex_image_auth_error(
    _error: crate::types::api_key_provider::OpenAiCodexAuthError,
) -> xai_tool_runtime::ToolError {
    xai_tool_runtime::ToolError::new(
        xai_tool_runtime::ToolErrorKind::Custom,
        "Codex image authentication is unavailable for this session.",
    )
    .with_details(serde_json::json!({"code": "auth_provider_mismatch"}))
}

fn validate_xai_image_base_url(base_url: &str) -> Result<String, xai_tool_runtime::ToolError> {
    let url = reqwest::Url::parse(base_url).map_err(|_| {
        xai_tool_runtime::ToolError::invalid_arguments("Image provider endpoint is invalid")
    })?;
    let safe_shape = url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none();
    if !safe_shape {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "Image provider endpoint is invalid",
        ));
    }

    #[cfg(any(test, feature = "test-support"))]
    let test_loopback = url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"));
    #[cfg(not(any(test, feature = "test-support")))]
    let test_loopback = false;

    if url.scheme() != "https" && !test_loopback {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "Image provider endpoint must use HTTPS",
        ));
    }
    if !test_loopback && url.host_str().is_some_and(is_local_or_private_host) {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "Image provider endpoint is not permitted",
        ));
    }
    Ok(url.to_string().trim_end_matches('/').to_owned())
}

fn is_local_or_private_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return true;
    }
    let Ok(address) = host.parse::<std::net::IpAddr>() else {
        return false;
    };
    match address {
        std::net::IpAddr::V4(address) => {
            let octets = address.octets();
            address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_unspecified()
                || address.is_broadcast()
                || address.is_multicast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || octets[0] >= 240
        }
        std::net::IpAddr::V6(address) => {
            let first = address.segments()[0];
            address.is_loopback()
                || address.is_unspecified()
                || address.is_multicast()
                || first & 0xfe00 == 0xfc00
                || first & 0xffc0 == 0xfe80
        }
    }
}

fn validate_codex_base_url(base_url: &str) -> Result<String, xai_tool_runtime::ToolError> {
    let production = base_url == OPENAI_CODEX_BASE_URL
        || base_url.strip_suffix('/') == Some(OPENAI_CODEX_BASE_URL);

    if production {
        // Retain no caller-controlled URL spelling once production validation
        // succeeds. This also rejects userinfo, explicit ports, alternate host
        // casing, and path/query/fragment variants before credentials exist.
        return Ok(OPENAI_CODEX_BASE_URL.to_owned());
    }

    // Unit tests need a narrow loopback seam for generation/edit mock servers.
    // Shipping builds cannot direct bearer/account headers to any caller-
    // selected destination.
    #[cfg(test)]
    let test_origin = reqwest::Url::parse(base_url).ok().is_some_and(|url| {
        url.scheme() == "http"
            && matches!(url.path(), "" | "/")
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"))
    });
    #[cfg(not(test))]
    let test_origin = false;
    if test_origin {
        return Ok(base_url.trim_end_matches('/').to_owned());
    }

    Err(xai_tool_runtime::ToolError::invalid_arguments(
        "Codex image credentials may only be sent to the ChatGPT Codex endpoint.",
    ))
}

/// `Enabled` means credentials are present; each tool has its own gate.
#[derive(Clone, Default)]
pub enum ImageGenConfig {
    #[default]
    Disabled,
    /// Fail-closed facade used while the feature gates are known but provider
    /// credentials are not yet available. Initial construction may advertise
    /// the gated definitions without a client; provider switching later removes
    /// or restores both the definitions and client as one lifecycle.
    Unavailable {
        image_gen_enabled: bool,
        image_edit_enabled: bool,
    },
    Enabled {
        api_key: String,
        base_url: String,
        extra_headers: indexmap::IndexMap<String, String>,
        image_gen_enabled: bool,
        image_edit_enabled: bool,
        /// Optional Imagine model override for `image_gen`. When `Some(non-empty)`,
        /// `image_gen` calls that model instead of the default quality model
        /// ([`XAI_IMAGINE_MODEL`]). Driven by the remote
        /// `image_gen_model_override` config flag. `image_edit` is unaffected.
        model_override: Option<String>,
        /// `true` when the user is on a tier the Imagine server zero-limits
        /// (free / X Basic). The tools stay advertised to the model, but
        /// `image_gen` / `image_edit` short-circuit at call time with the
        /// SuperGrok upsell prose instead of a doomed request. Set by the
        /// host from the subscription tier; always `false` for team /
        /// API-key / workspace callers.
        tier_restricted: bool,
    },
    /// ChatGPT Codex subscription image backend. Authentication is supplied
    /// dynamically through `SharedApiKeyProvider`; no access token is stored
    /// in this configuration.
    OpenAiCodex {
        base_url: String,
        image_gen_enabled: bool,
        image_edit_enabled: bool,
    },
}

impl std::fmt::Debug for ImageGenConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Configuration may carry static credentials, credential header names,
        // endpoint userinfo, model routing, and capability/tier state.
        f.debug_struct("ImageGenConfig").finish_non_exhaustive()
    }
}

impl ImageGenConfig {
    /// Credentials present — required to construct any of the clients.
    pub fn has_credentials(&self) -> bool {
        matches!(self, Self::Enabled { .. } | Self::OpenAiCodex { .. })
    }

    pub fn image_gen_enabled(&self) -> bool {
        matches!(
            self,
            Self::Enabled {
                image_gen_enabled: true,
                ..
            } | Self::Unavailable {
                image_gen_enabled: true,
                ..
            } | Self::OpenAiCodex {
                image_gen_enabled: true,
                ..
            }
        )
    }

    pub fn image_edit_enabled(&self) -> bool {
        matches!(
            self,
            Self::Enabled {
                image_edit_enabled: true,
                ..
            } | Self::Unavailable {
                image_edit_enabled: true,
                ..
            } | Self::OpenAiCodex {
                image_edit_enabled: true,
                ..
            }
        )
    }

    /// The configured `image_gen` model override, if any. `None` means the
    /// default quality model ([`XAI_IMAGINE_MODEL`]) is used.
    pub fn model_override(&self) -> Option<&str> {
        match self {
            Self::Enabled { model_override, .. } => {
                model_override.as_deref().filter(|m| !m.trim().is_empty())
            }
            Self::Disabled | Self::Unavailable { .. } | Self::OpenAiCodex { .. } => None,
        }
    }
}

pub(crate) async fn read_image_response(
    response: reqwest::Response,
) -> Result<ImageGenResponse, xai_tool_runtime::ToolError> {
    read_image_response_with_limit(response, MAX_IMAGE_RESPONSE_BODY_BYTES).await
}

async fn read_image_response_with_limit(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<ImageGenResponse, xai_tool_runtime::ToolError> {
    if response
        .content_length()
        .is_some_and(|len| len > max_bytes as u64)
    {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "Image provider response exceeded the configured size limit.",
        ));
    }
    let body = collect_limited_image_response(response.bytes_stream(), max_bytes).await?;
    serde_json::from_slice(&body).map_err(|_| {
        xai_tool_runtime::ToolError::invalid_arguments(
            "Image provider returned an invalid response.",
        )
    })
}

async fn collect_limited_image_response<S, B, E>(
    stream: S,
    max_bytes: usize,
) -> Result<Vec<u8>, xai_tool_runtime::ToolError>
where
    S: futures_util::Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
    E: std::fmt::Display,
{
    use futures_util::StreamExt as _;

    futures_util::pin_mut!(stream);
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| {
            xai_tool_runtime::ToolError::invalid_arguments(
                "Image provider response could not be read.",
            )
        })?;
        let chunk = chunk.as_ref();
        if body.len().saturating_add(chunk.len()) > max_bytes {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "Image provider response exceeded the configured size limit.",
            ));
        }
        body.extend_from_slice(chunk);
    }
    Ok(body)
}

pub(crate) fn validate_generated_image(
    image_bytes: &[u8],
) -> Result<(), xai_tool_runtime::ToolError> {
    if image_bytes.is_empty() || image_bytes.len() > MAX_GENERATED_IMAGE_BYTES {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "Generated image exceeded the configured size limit.",
        ));
    }
    let kind = infer::get(image_bytes).ok_or_else(|| {
        xai_tool_runtime::ToolError::invalid_arguments(
            "Image provider returned data that is not a recognized image.",
        )
    })?;
    if !matches!(
        kind.mime_type(),
        "image/png" | "image/jpeg" | "image/webp" | "image/gif"
    ) {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "Image provider returned an unsupported image format.",
        ));
    }
    let reader = ImageReader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()
        .map_err(|_| {
            xai_tool_runtime::ToolError::invalid_arguments(
                "Image provider returned an invalid image.",
            )
        })?;
    let (width, height) = reader.into_dimensions().map_err(|_| {
        xai_tool_runtime::ToolError::invalid_arguments("Image provider returned an invalid image.")
    })?;
    if width > MAX_GENERATED_IMAGE_DIMENSION
        || height > MAX_GENERATED_IMAGE_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_GENERATED_IMAGE_PIXELS
    {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "Generated image dimensions exceeded the configured safety limit.",
        ));
    }
    Ok(())
}

/// Returns the extension corresponding to the already-validated image bytes.
///
/// Image providers may return PNG, JPEG, WebP, or GIF regardless of the
/// writer's historical JPEG default. Keeping the extension in sync with the
/// actual media type makes generated files usable by downstream readers.
pub(crate) fn generated_image_extension(image_bytes: &[u8]) -> Option<&'static str> {
    match infer::get(image_bytes)?.mime_type() {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        _ => None,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ImageGenInput {
    #[schemars(description = "Text description of the image to generate.")]
    pub prompt: String,

    #[serde(default = "default_aspect_ratio")]
    #[schemars(
        description = "Aspect ratio of the generated image, decide it based on the user's request. Defaults to 'auto'. 1:1 for square (icons, profiles), 16:9 for wide (landscapes, cinematic), 9:16 for tall (phone wallpapers, stories), 3:2 for horizontal photos, 2:3 for vertical (portraits, posters)."
    )]
    pub aspect_ratio: String,
}

fn default_aspect_ratio() -> String {
    "auto".to_owned()
}

#[derive(Debug, serde::Deserialize)]
pub struct ImageGenResponse {
    #[serde(default)]
    data: Vec<ImageGenData>,
}

impl ImageGenResponse {
    pub fn b64_data(&self) -> Option<&str> {
        self.data.first().and_then(|d| d.b64_json.as_deref())
    }
}

#[derive(Debug, serde::Deserialize)]
struct ImageGenData {
    b64_json: Option<String>,
}

#[derive(Debug, Default)]
pub struct ImageGenTool;

impl crate::types::tool_metadata::ToolMetadata for ImageGenTool {
    fn kind(&self) -> ToolKind {
        ToolKind::ImageGen
    }

    fn tool_namespace(&self) -> ToolNamespace {
        ToolNamespace::GrokBuild
    }

    fn description_template(&self) -> &str {
        "Generate a new image from a text description using the selected provider; returns the saved image's absolute path. When telling the user where it was saved, refer to it by its short session-relative path (e.g. `images/1.jpg`) rather than the absolute path, so it renders as a clickable link that opens the image. To produce multiple images, emit multiple tool calls with distinct prompts."
    }

    fn requires_expr(&self) -> Expr<ToolRequirement> {
        Expr::True
    }
}

impl xai_tool_runtime::Tool for ImageGenTool {
    type Args = ImageGenInput;
    type Output = ToolOutput;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new("image_gen").expect("valid tool id")
    }

    fn description(
        &self,
        _ctx: &::xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        xai_tool_types::ToolDescription::new(
            "image_gen",
            crate::types::tool_metadata::ToolMetadata::description_template(self),
        )
    }

    fn capabilities(&self) -> xai_tool_protocol::ToolCapabilities {
        xai_tool_protocol::ToolCapabilities {
            is_read_only: false,
            tool_scope: Some(xai_tool_protocol::ToolScope::Write),
            ..Default::default()
        }
    }

    #[tracing::instrument(
        name = "tool.image_gen",
        skip_all,
        fields(prompt_len = input.prompt.len(), aspect_ratio = %input.aspect_ratio)
    )]
    async fn run(
        &self,
        ctx: xai_tool_runtime::ToolCallContext,
        input: ImageGenInput,
    ) -> Result<ToolOutput, xai_tool_runtime::ToolError> {
        use crate::types::tool_metadata::shared_resources;
        let resources = shared_resources(&ctx)?;

        let client = {
            let res = resources.lock().await;
            res.require::<ImageGenClient>()?.clone()
        };

        // Free / X Basic users are zero-limited on Imagine server-side; return
        // the upsell prose instead of a doomed request (the tool stays
        // advertised so the model can surface the nudge in-conversation).
        if client.is_tier_restricted() {
            return Ok(ToolOutput::Text(TIER_RESTRICTED_UPSELL.into()));
        }

        let image_bytes = client.generate(&input.prompt, &input.aspect_ratio).await?;

        let session_folder = {
            let res = resources.lock().await;
            res.require::<SessionFolder>()?.0.clone()
        };

        let extension = generated_image_extension(&image_bytes);
        let absolute_path = client
            .writer
            .save(&session_folder, &image_bytes, extension)
            .await
            .map_err(|e| xai_tool_runtime::ToolError::invalid_arguments(e.to_string()))?;

        tracing::info!(
            path = %absolute_path.display(),
            bytes = image_bytes.len(),
            "image saved to disk"
        );

        Ok(ToolOutput::ImageGen(MediaGenOutput::new(absolute_path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::tool_metadata::test_ctx_with_call_id;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    struct CodexTestAuth;

    impl crate::types::ApiKeyProvider for CodexTestAuth {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(OPENAI_CODEX_PROVIDER_ID)
        }

        fn current_request_auth_async(
            &self,
        ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
            Box::pin(std::future::ready(Some(
                RequestAuth::for_provider_snapshot(
                    OPENAI_CODEX_PROVIDER_ID,
                    RequestCredentialSnapshot::new("test-credential", 1),
                    [
                        ("authorization".to_owned(), "Bearer test-access".to_owned()),
                        ("chatgpt-account-id".to_owned(), "test-account".to_owned()),
                        ("x-openai-fedramp".to_owned(), "true".to_owned()),
                    ],
                ),
            )))
        }
    }

    struct ZeroGenerationCodexTestAuth;

    impl crate::types::ApiKeyProvider for ZeroGenerationCodexTestAuth {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(OPENAI_CODEX_PROVIDER_ID)
        }

        fn current_request_auth_async(
            &self,
        ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
            Box::pin(std::future::ready(Some(
                RequestAuth::for_provider_snapshot(
                    OPENAI_CODEX_PROVIDER_ID,
                    RequestCredentialSnapshot::new("test-credential", 0),
                    [
                        ("authorization".to_owned(), "Bearer test-access".to_owned()),
                        ("chatgpt-account-id".to_owned(), "test-account".to_owned()),
                    ],
                ),
            )))
        }
    }

    #[tokio::test]
    async fn response_stream_is_bounded_before_appending_next_chunk() {
        let stream = futures_util::stream::iter([
            Ok::<_, std::io::Error>(vec![1_u8, 2, 3]),
            Ok(vec![4_u8, 5]),
        ]);
        assert_eq!(
            collect_limited_image_response(stream, 5).await.unwrap(),
            vec![1, 2, 3, 4, 5]
        );

        let oversized = futures_util::stream::iter([
            Ok::<_, std::io::Error>(vec![1_u8, 2, 3]),
            Ok(vec![4_u8, 5, 6]),
        ]);
        let err = collect_limited_image_response(oversized, 5)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("size limit"), "got: {err}");
    }

    #[tokio::test]
    async fn chunked_http_response_without_content_length_is_stream_bounded() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n3\r\ndef\r\n0\r\n\r\n",
                )
                .await
                .unwrap();
        });

        let response = reqwest::get(format!("http://{address}/chunked"))
            .await
            .unwrap();
        assert_eq!(response.content_length(), None);
        let err = read_image_response_with_limit(response, 5)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("size limit"), "got: {err}");
        server.await.unwrap();
    }

    #[test]
    fn tool_name_and_description() {
        let tool = ImageGenTool;
        assert_eq!(xai_tool_runtime::Tool::id(&tool).as_str(), "image_gen");
        assert!(
            crate::types::tool_metadata::ToolMetadata::description_template(&tool)
                .contains("Generate a new image from a text description")
        );
    }

    #[test]
    fn default_aspect_ratio_is_auto() {
        let input: ImageGenInput = serde_json::from_str(r#"{"prompt": "test"}"#).unwrap();
        assert_eq!(input.aspect_ratio, "auto");
    }

    #[test]
    fn codex_aspect_ratios_map_to_supported_canvases() {
        assert_eq!(codex_image_size("1:1"), "1024x1024");
        assert_eq!(codex_image_size("16:9"), "1280x720");
        assert_eq!(codex_image_size("9:16"), "720x1280");
        assert_eq!(codex_image_size("2:3"), "768x1152");
        assert_eq!(codex_image_size("19.5:9"), "1248x576");
        assert_eq!(codex_image_size("9:20"), "576x1280");
        assert_eq!(codex_image_size("auto"), "auto");
        assert_eq!(codex_image_size("unsupported"), "auto");

        for (ratio, numerator, denominator) in [
            ("1:1", 1_u64, 1_u64),
            ("16:9", 16, 9),
            ("9:16", 9, 16),
            ("4:3", 4, 3),
            ("3:4", 3, 4),
            ("3:2", 3, 2),
            ("2:3", 2, 3),
            ("2:1", 2, 1),
            ("1:2", 1, 2),
            ("19.5:9", 13, 6),
            ("9:19.5", 6, 13),
            ("20:9", 20, 9),
            ("9:20", 9, 20),
        ] {
            let (width, height) = codex_image_size(ratio)
                .split_once('x')
                .map(|(width, height)| {
                    (
                        width.parse::<u64>().unwrap(),
                        height.parse::<u64>().unwrap(),
                    )
                })
                .expect("known ratio must map to concrete dimensions");
            assert_eq!(width % 16, 0, "{ratio} width");
            assert_eq!(height % 16, 0, "{ratio} height");
            assert!(width <= 3840 && height <= 3840, "{ratio} edge bound");
            assert!(
                (655_360..=8_294_400).contains(&(width * height)),
                "{ratio} pixel bound"
            );
            assert_eq!(width * denominator, height * numerator, "{ratio}");
        }
    }

    #[test]
    fn image_config_debug_redacts_credentials_and_header_values() {
        let mut extra_headers = indexmap::IndexMap::new();
        extra_headers.insert(
            "x-provider-credential".to_owned(),
            "sentinel-header-value".to_owned(),
        );
        let config = ImageGenConfig::Enabled {
            api_key: "sentinel-api-key".to_owned(),
            base_url: "https://sentinel-user:sentinel-pass@example.com".to_owned(),
            extra_headers,
            image_gen_enabled: true,
            image_edit_enabled: true,
            model_override: None,
            tier_restricted: false,
        };
        assert_eq!(format!("{config:?}"), "ImageGenConfig { .. }");
    }

    #[test]
    fn xai_request_auth_accepts_only_one_provider_neutral_bearer() {
        let config = ImageGenConfig::Enabled {
            api_key: "sentinel-static-key".to_owned(),
            base_url: "https://api.x.ai/v1".to_owned(),
            extra_headers: indexmap::IndexMap::new(),
            image_gen_enabled: true,
            image_edit_enabled: true,
            model_override: None,
            tier_restricted: false,
        };
        let client = ImageGenClient::new(&config, None).unwrap();
        let valid = RequestAuth::bearer("sentinel-dynamic-key");
        let request = client
            .apply_request_auth(
                client
                    .http
                    .as_ref()
                    .expect("xAI transport is eager")
                    .post("https://api.x.ai/v1/images/generations"),
                &valid,
            )
            .unwrap()
            .build()
            .unwrap();
        assert!(request.headers()[AUTHORIZATION].is_sensitive());

        let invalid = [
            RequestAuth::for_provider(
                "sentinel-provider",
                [("authorization".to_owned(), "Bearer sentinel-key".to_owned())],
            ),
            RequestAuth::from_headers([
                ("authorization".to_owned(), "Bearer sentinel-key".to_owned()),
                (
                    "chatgpt-account-id".to_owned(),
                    "sentinel-account".to_owned(),
                ),
            ]),
            RequestAuth::from_headers([(
                "authorization".to_owned(),
                "Basic sentinel-key".to_owned(),
            )]),
            RequestAuth::from_headers([(
                "chatgpt-account-id".to_owned(),
                "sentinel-account".to_owned(),
            )]),
        ];
        for auth in invalid {
            let error = client
                .apply_request_auth(
                    client
                        .http
                        .as_ref()
                        .expect("xAI transport is eager")
                        .post("https://api.x.ai/v1/images/generations"),
                    &auth,
                )
                .err()
                .expect("structured or non-bearer auth must fail")
                .to_string();
            for forbidden in [
                "sentinel-provider",
                "sentinel-key",
                "sentinel-account",
                "chatgpt-account-id",
            ] {
                assert!(!error.contains(forbidden));
            }
        }
    }

    #[test]
    fn per_tool_gates_are_independent() {
        let cfg = ImageGenConfig::Enabled {
            api_key: "k".into(),
            base_url: "https://api.x.ai/v1".into(),
            extra_headers: indexmap::IndexMap::new(),
            image_gen_enabled: false,
            image_edit_enabled: true,
            model_override: Some("grok-imagine-image".into()),
            tier_restricted: false,
        };
        assert!(cfg.has_credentials());
        assert!(!cfg.image_gen_enabled());
        assert!(cfg.image_edit_enabled());
        assert_eq!(cfg.model_override(), Some("grok-imagine-image"));

        assert!(!ImageGenConfig::Disabled.has_credentials());
        let unavailable = ImageGenConfig::Unavailable {
            image_gen_enabled: true,
            image_edit_enabled: false,
        };
        assert!(!unavailable.has_credentials());
        assert!(unavailable.image_gen_enabled());
        assert!(!unavailable.image_edit_enabled());
        assert!(ImageGenClient::new(&unavailable, None).is_err());
    }

    #[test]
    fn client_selects_model_from_override() {
        let mk = |model_override: Option<&str>| ImageGenConfig::Enabled {
            api_key: "k".into(),
            base_url: "https://api.x.ai/v1".into(),
            extra_headers: indexmap::IndexMap::new(),
            image_gen_enabled: true,
            image_edit_enabled: true,
            model_override: model_override.map(String::from),
            tier_restricted: false,
        };
        // No override → default quality model.
        assert_eq!(
            ImageGenClient::new(&mk(None), None).unwrap().model,
            XAI_IMAGINE_MODEL
        );
        // Empty override → treated as no override.
        assert_eq!(
            ImageGenClient::new(&mk(Some("")), None).unwrap().model,
            XAI_IMAGINE_MODEL
        );
        // Override → that exact model slug.
        assert_eq!(
            ImageGenClient::new(&mk(Some("grok-imagine-image")), None)
                .unwrap()
                .model,
            "grok-imagine-image"
        );
    }

    #[test]
    fn xai_image_constructor_rejects_unsafe_endpoints_without_reflection() {
        for rejected in [
            "http://example.com/v1",
            "https://sentinel-user:sentinel-pass@example.com/v1",
            "https://example.com/v1?token=sentinel-query",
            "https://example.com/v1#sentinel-fragment",
            "https://localhost/v1",
            "https://127.0.0.1/v1",
            "file:///tmp/provider",
        ] {
            let config = ImageGenConfig::Enabled {
                api_key: "sentinel-static-key".to_owned(),
                base_url: rejected.to_owned(),
                extra_headers: indexmap::IndexMap::new(),
                image_gen_enabled: true,
                image_edit_enabled: true,
                model_override: None,
                tier_restricted: false,
            };
            let error = ImageGenClient::new(&config, None)
                .err()
                .unwrap_or_else(|| panic!("unsafe image endpoint was accepted: {rejected}"))
                .to_string();
            for forbidden in [
                rejected,
                "sentinel-user",
                "sentinel-pass",
                "sentinel-query",
                "sentinel-fragment",
                "sentinel-static-key",
            ] {
                assert!(!error.contains(forbidden), "unexpected error: {error}");
            }
        }
    }

    #[test]
    fn xai_image_constructor_does_not_reflect_custom_header_names() {
        let config = ImageGenConfig::Enabled {
            api_key: "sentinel-static-key".to_owned(),
            base_url: "https://api.x.ai/v1".to_owned(),
            extra_headers: indexmap::IndexMap::from([(
                "sentinel-invalid-header\n".to_owned(),
                "sentinel-header-value".to_owned(),
            )]),
            image_gen_enabled: true,
            image_edit_enabled: true,
            model_override: None,
            tier_restricted: false,
        };
        let error = ImageGenClient::new(&config, None)
            .err()
            .expect("invalid custom header must fail")
            .to_string();
        for forbidden in [
            "sentinel-invalid-header",
            "sentinel-header-value",
            "sentinel-static-key",
        ] {
            assert!(!error.contains(forbidden));
        }
    }

    #[test]
    fn codex_client_requires_dynamic_provider_authentication() {
        let config = ImageGenConfig::OpenAiCodex {
            base_url: "https://chatgpt.com/backend-api/codex".to_owned(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let error = ImageGenClient::new(&config, None)
            .err()
            .expect("Codex image clients must not exist without provider auth");
        assert!(
            error
                .to_string()
                .contains("dynamic provider authentication")
        );
    }

    #[test]
    fn codex_dynamic_image_credentials_are_sensitive_headers() {
        let config = ImageGenConfig::OpenAiCodex {
            base_url: "https://chatgpt.com/backend-api/codex".to_owned(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: SharedApiKeyProvider = Arc::new(CodexTestAuth);
        ImageGenClient::new(&config, Some(provider)).unwrap();
        let auth = RequestAuth::for_provider_snapshot(
            OPENAI_CODEX_PROVIDER_ID,
            RequestCredentialSnapshot::new("test-credential", 1),
            [
                ("authorization".to_owned(), "Bearer test-access".to_owned()),
                ("chatgpt-account-id".to_owned(), "test-account".to_owned()),
                ("x-openai-fedramp".to_owned(), "true".to_owned()),
            ],
        );
        let request = validate_openai_codex_request_auth(&auth)
            .unwrap()
            .apply(
                reqwest::Client::new()
                    .get("https://chatgpt.com/backend-api/codex/images/generations"),
            )
            .build()
            .unwrap();

        assert!(request.headers()["authorization"].is_sensitive());
        assert!(request.headers()["chatgpt-account-id"].is_sensitive());
        assert!(request.headers()["x-openai-fedramp"].is_sensitive());
    }

    #[tokio::test]
    async fn codex_generation_uses_exact_provider_contract_without_xai_headers() {
        use wiremock::matchers::{body_json, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(1, 1)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let encoded = base64::engine::general_purpose::STANDARD.encode(png.into_inner());

        Mock::given(method("POST"))
            .and(path("/images/generations"))
            .and(header("authorization", "Bearer test-access"))
            .and(header("chatgpt-account-id", "test-account"))
            .and(header("x-openai-fedramp", "true"))
            .and(header("originator", "grok_build_codex"))
            .and(header(
                "version",
                xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION,
            ))
            .and(header(
                "user-agent",
                format!("grok-build-codex/{}", xai_grok_version::VERSION),
            ))
            .and(body_json(serde_json::json!({
                "model": OPENAI_CODEX_IMAGE_MODEL,
                "prompt": "a lighthouse",
                "background": "auto",
                "quality": "auto",
                "size": "1280x720",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"b64_json": encoded}]
            })))
            .mount(&server)
            .await;

        let config = ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: SharedApiKeyProvider = Arc::new(CodexTestAuth);
        let client = ImageGenClient::new(&config, Some(provider)).unwrap();
        let result = client.generate("a lighthouse", "16:9").await.unwrap();
        assert!(!result.is_empty());

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let headers = &requests[0].headers;
        assert!(
            headers
                .keys()
                .all(|name| !name.as_str().starts_with("x-grok-"))
        );
        assert!(!headers.contains_key("x-xai-token-auth"));
    }

    #[tokio::test]
    async fn codex_generation_rejects_zero_credential_generation_before_request() {
        let server = wiremock::MockServer::start().await;
        let config = ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: SharedApiKeyProvider = Arc::new(ZeroGenerationCodexTestAuth);
        let error = ImageGenClient::new(&config, Some(provider))
            .unwrap()
            .generate("a lighthouse", "auto")
            .await
            .expect_err("generation-zero credentials must fail before dispatch");

        assert!(error.to_string().contains("authentication is unavailable"));
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    struct StaggeredCodexTestAuth {
        auth_calls: AtomicUsize,
        recoveries: AtomicUsize,
        rejected: Mutex<Option<RequestCredentialSnapshot>>,
    }

    impl crate::types::ApiKeyProvider for StaggeredCodexTestAuth {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(OPENAI_CODEX_PROVIDER_ID)
        }

        fn current_request_auth_async(
            &self,
        ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
            let call = self.auth_calls.fetch_add(1, Ordering::SeqCst);
            let (token, generation) = if call == 0 {
                // Simulate another request rotating the manager immediately
                // after this request was signed but before its 401 arrives.
                ("Bearer generation-seven", 7)
            } else {
                ("Bearer generation-eight", 8)
            };
            Box::pin(std::future::ready(Some(
                RequestAuth::for_provider_snapshot(
                    OPENAI_CODEX_PROVIDER_ID,
                    RequestCredentialSnapshot::new("stable-credential", generation),
                    [
                        ("authorization".to_owned(), token.to_owned()),
                        ("chatgpt-account-id".to_owned(), "test-account".to_owned()),
                    ],
                ),
            )))
        }

        fn recover_unauthorized_async(
            &self,
            rejected: Option<RequestCredentialSnapshot>,
        ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
            self.recoveries.fetch_add(1, Ordering::SeqCst);
            *self.rejected.lock().unwrap() = rejected;
            Box::pin(std::future::ready(true))
        }
    }

    #[tokio::test]
    async fn codex_401_recovery_receives_generation_that_signed_request() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/images/generations"))
            .and(header("authorization", "Bearer generation-seven"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(1, 1)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let encoded = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
        Mock::given(method("POST"))
            .and(path("/images/generations"))
            .and(header("authorization", "Bearer generation-eight"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"b64_json": encoded}]
            })))
            .mount(&server)
            .await;

        let config = ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider = Arc::new(StaggeredCodexTestAuth {
            auth_calls: AtomicUsize::new(0),
            recoveries: AtomicUsize::new(0),
            rejected: Mutex::new(None),
        });
        let client =
            ImageGenClient::new(&config, Some(Arc::clone(&provider) as SharedApiKeyProvider))
                .unwrap();
        assert!(
            !client
                .generate("a lighthouse", "auto")
                .await
                .unwrap()
                .is_empty()
        );

        assert_eq!(provider.auth_calls.load(Ordering::SeqCst), 2);
        assert_eq!(provider.recoveries.load(Ordering::SeqCst), 1);
        let rejected = provider.rejected.lock().unwrap().clone().unwrap();
        assert!(rejected.opaque_id() == "stable-credential");
        assert_eq!(rejected.generation(), 7);
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn terminal_codex_401_marks_provider_recovery_as_exhausted() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/images/generations"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let config = ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: SharedApiKeyProvider = Arc::new(CodexTestAuth);
        let error = ImageGenClient::new(&config, Some(provider))
            .unwrap()
            .generate("a lighthouse", "auto")
            .await
            .expect_err("401 must surface");
        let details = error.details.expect("structured failure details");
        assert_eq!(details.get("status"), Some(&serde_json::json!(401)));
        assert_eq!(
            details.get(crate::types::AUTH_RECOVERY_PROVIDER_DETAILS_KEY),
            Some(&serde_json::json!(OPENAI_CODEX_PROVIDER_ID))
        );
        assert_eq!(
            details.get(crate::types::AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            server.received_requests().await.unwrap().len(),
            1,
            "a provider that cannot recover must not replay"
        );
    }

    #[test]
    fn codex_production_base_url_is_canonicalized_and_sealed() {
        let provider: SharedApiKeyProvider = Arc::new(CodexTestAuth);
        let trailing_slash = format!("{OPENAI_CODEX_BASE_URL}/");
        for accepted in [OPENAI_CODEX_BASE_URL, trailing_slash.as_str()] {
            let config = ImageGenConfig::OpenAiCodex {
                base_url: accepted.to_owned(),
                image_gen_enabled: true,
                image_edit_enabled: true,
            };
            let client = ImageGenClient::new(&config, Some(Arc::clone(&provider))).unwrap();
            assert_eq!(client.base_url, OPENAI_CODEX_BASE_URL);
        }

        for rejected in [
            "http://chatgpt.com/backend-api/codex",
            "https://chatgpt.com:443/backend-api/codex",
            "https://chatgpt.com:444/backend-api/codex",
            "https://user@chatgpt.com/backend-api/codex",
            "https://user:pass@chatgpt.com/backend-api/codex",
            "https://CHATGPT.com/backend-api/codex",
            "https://chatgpt.com/backend-api/codex//",
            "https://chatgpt.com/backend-api/codex/images",
            "https://chatgpt.com/backend-api/codex?redirect=foreign",
            "https://chatgpt.com/backend-api/codex#fragment",
            "https://example.com/backend-api/codex",
        ] {
            let config = ImageGenConfig::OpenAiCodex {
                base_url: rejected.to_owned(),
                image_gen_enabled: true,
                image_edit_enabled: true,
            };
            let error = ImageGenClient::new(&config, Some(Arc::clone(&provider)))
                .err()
                .unwrap_or_else(|| panic!("noncanonical Codex image URL was accepted: {rejected}"));
            assert!(
                error.to_string().contains("ChatGPT Codex endpoint"),
                "unexpected error for {rejected}: {error}"
            );
        }
    }

    #[tokio::test]
    async fn xai_image_redirects_are_terminal_and_do_not_forward_credentials() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let target = MockServer::start().await;
        let source = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/images/generations"))
            .respond_with(
                ResponseTemplate::new(307)
                    .insert_header("location", format!("{}/credential-sink", target.uri())),
            )
            .mount(&source)
            .await;

        let config = ImageGenConfig::Enabled {
            api_key: "sentinel-static-key".to_owned(),
            base_url: source.uri(),
            extra_headers: indexmap::IndexMap::new(),
            image_gen_enabled: true,
            image_edit_enabled: true,
            model_override: None,
            tier_restricted: false,
        };
        let response = ImageGenClient::new(&config, None)
            .unwrap()
            .post_json("images/generations", &serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(source.received_requests().await.unwrap().len(), 1);
        assert!(target.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn errors_when_client_missing() {
        let tool = ImageGenTool;
        let resources = crate::types::resources::Resources::new();
        let result = xai_tool_runtime::Tool::run(
            &tool,
            test_ctx_with_call_id(resources.into_shared(), "test-call"),
            ImageGenInput {
                prompt: "a test image".into(),
                aspect_ratio: "auto".into(),
            },
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("missing required resource"),
            "Expected MissingResource error, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn tier_restricted_short_circuits_with_upsell() {
        // A free / X Basic user's image_gen call returns the SuperGrok upsell
        // prose as a normal result (no HTTP, no error card) so the model can
        // relay it. Only the client is inserted — the short-circuit returns
        // before any other resource (e.g. SessionFolder) is required.
        let cfg = ImageGenConfig::Enabled {
            api_key: "k".into(),
            base_url: "https://api.x.ai/v1".into(),
            extra_headers: indexmap::IndexMap::new(),
            image_gen_enabled: true,
            image_edit_enabled: true,
            model_override: None,
            tier_restricted: true,
        };
        let mut resources = crate::types::resources::Resources::new();
        resources.insert(ImageGenClient::new(&cfg, None).unwrap());

        let result = xai_tool_runtime::Tool::run(
            &ImageGenTool,
            test_ctx_with_call_id(resources.into_shared(), "test-call"),
            ImageGenInput {
                prompt: "a cat".into(),
                aspect_ratio: "auto".into(),
            },
        )
        .await
        .expect("tier-restricted call must succeed with upsell prose");

        match result {
            ToolOutput::Text(t) => {
                assert!(t.text.contains("SuperGrok"), "got: {}", t.text);
                assert!(t.text.contains("supergrok?referrer=grok-build"));
            }
            other => panic!("expected Text upsell, got {other:?}"),
        }
    }
}
