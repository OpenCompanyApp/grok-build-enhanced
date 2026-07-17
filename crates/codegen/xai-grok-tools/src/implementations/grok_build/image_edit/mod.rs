//! `image_edit` tool — edits or transforms images through the configured
//! image provider's `/images/edits` endpoint using one or more references.
//!
//! Use cases include likeness preservation, style transfer, subject lock,
//! remixing, and general image-to-image editing. The model chooses this
//! tool (instead of `image_gen`) when the user provides reference photos.
//!
//! Reference images are specified as filesystem paths or
//! `data:image/...;base64,...` URLs. The tool reads through the shared
//! filesystem abstraction, applies provider-specific preparation, and POSTs
//! to the edit endpoint. xAI references are compressed to Imagine limits;
//! Codex-native PNG/JPEG/WebP bytes are preserved within explicit safety caps.
//!
//! Shares the same [`ImageGenClient`] and session credentials as
//! `image_gen` — no additional configuration is needed.

use std::io::Cursor;
use std::path::Path;

use crate::computer::types::AsyncFileSystem;
use crate::implementations::grok_build::image_gen::{
    ImageGenBackend, ImageGenClient, OPENAI_CODEX_IMAGE_MODEL, read_image_response,
    validate_generated_image,
};
use crate::types::output::{MediaGenOutput, ToolOutput};
use crate::types::requirements::{Expr, ToolRequirement};
use crate::types::resources::{FileSystem, SessionFolder};
use crate::types::tool::{ToolKind, ToolNamespace};
use crate::util::image_compress::{FilterType, ReEncodeParams, re_encode_under_limit};
use crate::util::image_validate::{
    format_structurally_complete, transcode_to_endpoint_png, validate_image_bytes_unrestricted,
};
use base64::Engine as _;
use image::{ImageFormat, ImageReader};

const XAI_IMAGINE_MODEL: &str = "grok-imagine-image-quality";

/// Size/dimension limits for reference images sent to the Imagine API.
/// Tighter than the vision path; the backend returns 400 when exceeded.
const MAX_REF_RAW_BYTES: usize = 400 * 1024;
const MAX_REF_DIMENSION: u32 = 768;
const MIN_REF_DIMENSION: u32 = 256;
const REF_QUALITY_STEPS: &[u8] = &[80, 65, 50, 35];
const MAX_REF_DECODE_PIXELS: u64 = 12_000_000;
const MAX_REF_INPUT_BYTES: usize = 24 * 1024 * 1024;
/// Combined raw-byte budget across all references in one Codex edit. This
/// bounds the base64 strings, cloned JSON values, and serialized request that
/// coexist while reqwest builds the body, while still allowing two maximum-
/// sized source images or five typical references.
const MAX_CODEX_REF_TOTAL_BYTES: usize = 48 * 1024 * 1024;
/// Codex reference images remain byte-for-byte original up to this decoded
/// pixel ceiling. Native PNG/JPEG/WebP inputs are only header/structure
/// checked locally, so the cap prevents pathological headers without forcing
/// the lossy 768 px Imagine preparation path onto Codex edits.
const MAX_CODEX_REF_PIXELS: u64 = 178_956_970;
const MAX_REFERENCE_IMAGES: usize = 5;

pub const IMAGE_EDIT_TOOL_NAME: &str = "image_edit";

// ---------------------------------------------------------------------------
// Compression
// ---------------------------------------------------------------------------

/// Compress a reference image to fit within Imagine API limits.
///
/// Returns `(bytes, mime)`. Small JPEG/PNG inputs pass through unchanged.
fn compress_reference(
    raw_bytes: Vec<u8>,
) -> Result<(Vec<u8>, &'static str), xai_tool_runtime::ToolError> {
    // Fast path: small JPEG/PNG passes through unchanged. Other formats
    // (WebP, GIF, etc.) always re-encode to guarantee API-compatible output.
    if raw_bytes.len() <= MAX_REF_RAW_BYTES
        && let Some(kind) = infer::get(&raw_bytes)
    {
        match kind.mime_type() {
            "image/jpeg" => return Ok((raw_bytes, "image/jpeg")),
            "image/png" => return Ok((raw_bytes, "image/png")),
            _ => {}
        }
    }

    // Refuse to decode absurdly large images.
    let reader = ImageReader::new(Cursor::new(&raw_bytes))
        .with_guessed_format()
        .map_err(|_| {
            xai_tool_runtime::ToolError::invalid_arguments(
                "could not detect image format for reference",
            )
        })?;

    if let Ok((w, h)) = reader.into_dimensions()
        && (w as u64) * (h as u64) > MAX_REF_DECODE_PIXELS
    {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(format!(
            "image reference is too large to process ({w}\u{00d7}{h} pixels)",
        )));
    }

    // `into_dimensions` consumed the reader; re-open to decode.
    let img = ImageReader::new(Cursor::new(&raw_bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.decode().ok())
        .ok_or_else(|| {
            xai_tool_runtime::ToolError::invalid_arguments("failed to decode image reference")
        })?;

    let params = ReEncodeParams {
        max_bytes: MAX_REF_RAW_BYTES,
        max_side_px: MAX_REF_DIMENSION,
        // Imagine backend limits are side-based; no pixel-area cap applies.
        max_pixels: u64::MAX,
        min_side_px: MIN_REF_DIMENSION,
        quality_steps: REF_QUALITY_STEPS,
        filter: FilterType::Lanczos3,
    };

    let (buf, _w, _h, mime) = re_encode_under_limit(&img, &params).map_err(|e| {
        xai_tool_runtime::ToolError::invalid_arguments(format!(
            "could not compress image reference small enough for Imagine API: {e}"
        ))
    })?;

    Ok((buf, mime))
}

/// Prepare an edit reference for the ChatGPT Codex image endpoint.
///
/// Current Codex behavior uses prompt-image `Original` mode: PNG, JPEG, and
/// WebP source bytes are preserved, while formats that the endpoint does not
/// consume natively are converted to PNG. Keep a fork-local byte ceiling so a
/// model cannot turn an arbitrary filesystem read into an unbounded request.
fn prepare_codex_reference(
    raw_bytes: Vec<u8>,
) -> Result<(Vec<u8>, &'static str), xai_tool_runtime::ToolError> {
    if raw_bytes.is_empty() {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "image reference contained no data",
        ));
    }
    if raw_bytes.len() > MAX_REF_INPUT_BYTES {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "image reference exceeds the configured size limit",
        ));
    }

    let (width, height, format) =
        validate_image_bytes_unrestricted(&raw_bytes, false).map_err(|e| {
            xai_tool_runtime::ToolError::invalid_arguments(format!(
                "could not validate image reference: {e}"
            ))
        })?;
    let pixels = (width as u64).saturating_mul(height as u64);
    if pixels > MAX_CODEX_REF_PIXELS {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(format!(
            "image reference is too large to process ({width}\u{00d7}{height} pixels)",
        )));
    }

    if matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        if !format_structurally_complete(format, &raw_bytes) {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "image reference is truncated or structurally incomplete",
            ));
        }
        let mime = match format {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::WebP => "image/webp",
            _ => unreachable!("native Codex image formats were matched above"),
        };
        return Ok((raw_bytes, mime));
    }

    let encoded = transcode_to_endpoint_png(&raw_bytes)
        .ok_or_else(|| {
            xai_tool_runtime::ToolError::invalid_arguments(
                "unsupported image format for Codex image editing",
            )
        })?
        .map_err(|e| {
            xai_tool_runtime::ToolError::invalid_arguments(format!(
                "could not convert image reference for Codex: {e}"
            ))
        })?;
    if encoded.len() > MAX_REF_INPUT_BYTES {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "converted image reference exceeds the configured size limit",
        ));
    }
    Ok((encoded, "image/png"))
}

fn prepare_reference(
    backend: ImageGenBackend,
    raw_bytes: Vec<u8>,
) -> Result<(Vec<u8>, &'static str), xai_tool_runtime::ToolError> {
    match backend {
        ImageGenBackend::XaiImagine => compress_reference(raw_bytes),
        ImageGenBackend::OpenAiCodex => prepare_codex_reference(raw_bytes),
    }
}

// ---------------------------------------------------------------------------
// Reference resolution
// ---------------------------------------------------------------------------

struct ResolvedReference {
    data_url: String,
    prepared_bytes: usize,
}

fn checked_codex_reference_total(
    current: usize,
    next: usize,
) -> Result<usize, xai_tool_runtime::ToolError> {
    let total = current.checked_add(next).ok_or_else(|| {
        xai_tool_runtime::ToolError::invalid_arguments(
            "combined image references exceed the configured size limit",
        )
    })?;
    if total > MAX_CODEX_REF_TOTAL_BYTES {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "combined image references exceed the configured size limit",
        ));
    }
    Ok(total)
}

/// Resolve a reference (filesystem path or `data:image/...;base64,...` URL)
/// into a provider-ready data URL.
async fn resolve_to_data_url(
    value: &str,
    backend: ImageGenBackend,
    file_system: &dyn AsyncFileSystem,
) -> Result<ResolvedReference, xai_tool_runtime::ToolError> {
    let value = value.trim();
    // Accept `file://` URIs (e.g. an attachment's durable URI) by reading
    // the underlying path. Data URLs and bare paths are untouched.
    let value = value.strip_prefix("file://").unwrap_or(value);

    let raw_bytes = if value.starts_with("data:image/") {
        let comma = value.find(',').ok_or_else(|| {
            xai_tool_runtime::ToolError::invalid_arguments("malformed data URL in image reference")
        })?;
        if !value[..comma].contains(";base64") {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "image references only support base64 data URLs",
            ));
        }
        if value.len().saturating_sub(comma + 1) > MAX_REF_INPUT_BYTES * 4 / 3 + 8 {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "image reference exceeds the configured size limit",
            ));
        }
        base64::engine::general_purpose::STANDARD
            .decode(&value[comma + 1..])
            .map_err(|e| {
                xai_tool_runtime::ToolError::invalid_arguments(format!(
                    "invalid base64 in image reference: {e}"
                ))
            })?
    } else {
        file_system
            .read_file_limited(Path::new(value), MAX_REF_INPUT_BYTES)
            .await
            .map_err(|e| {
                xai_tool_runtime::ToolError::invalid_arguments(format!(
                    "image reference not readable: {value} ({e})"
                ))
            })?
    };

    if raw_bytes.is_empty() {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "image reference contained no data",
        ));
    }
    if raw_bytes.len() > MAX_REF_INPUT_BYTES {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(
            "image reference exceeds the configured size limit",
        ));
    }

    let (prepared, mime) = prepare_reference(backend, raw_bytes)?;
    let prepared_bytes = prepared.len();
    let b64 = base64::engine::general_purpose::STANDARD.encode(&prepared);
    Ok(ResolvedReference {
        data_url: format!("data:{mime};base64,{b64}"),
        prepared_bytes,
    })
}

// ---------------------------------------------------------------------------
// Attachment reference resolution
// ---------------------------------------------------------------------------

/// Parse an attached-image reference token into its 1-based display number.
///
/// Accepts the forms the model naturally produces for an image the user
/// attached to the conversation: `[Image #1]`, `Image #1`, `image #1`, or
/// a bare `#1`. Returns `None` for anything else — filesystem paths and
/// `data:` / `file://` URLs fall through to direct resolution.
fn parse_attachment_token(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    // Strip optional surrounding brackets: `[…]`.
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(trimmed)
        .trim();
    // Strip an optional leading `image` label (case-insensitive). The
    // 5-byte prefix is ASCII, so slicing at byte 5 stays on a boundary.
    let rest = match inner.get(..5).map(str::to_ascii_lowercase).as_deref() {
        Some("image") => inner[5..].trim_start(),
        _ => inner,
    };
    // Require the `#` sigil followed by a bare positive integer.
    let digits = rest.strip_prefix('#')?.trim();
    match digits.parse::<usize>() {
        Ok(n) if n >= 1 => Some(n),
        _ => None,
    }
}

/// Resolve a single `image` argument to a reference `resolve_to_data_url`
/// can read.
///
/// Attachment tokens (`[Image #N]`) are mapped to the durable reference
/// the shell recorded for the current turn; everything else (filesystem
/// paths, `data:` / `file://` URLs) passes through unchanged.
fn resolve_attachment_reference(
    reference: &str,
    attached: Option<&crate::types::resources::AttachedImages>,
) -> Result<String, xai_tool_runtime::ToolError> {
    let Some(n) = parse_attachment_token(reference) else {
        return Ok(reference.to_owned());
    };
    let registry = attached.filter(|a| !a.0.is_empty()).ok_or_else(|| {
        // Tokens only resolve against the current message's attachments. An
        // empty registry usually means the image was attached in an earlier
        // message (cross-turn editing isn't supported yet), so steer the
        // model to ask for a re-attach rather than retry the dead token.
        xai_tool_runtime::ToolError::invalid_arguments(format!(
            "image reference {reference:?} matches no image attached to this message. If it was \
             attached earlier in the conversation, ask the user to re-attach it here; otherwise \
             pass an absolute filesystem path or a data: URL."
        ))
    })?;
    registry.reference_for(n).map(str::to_owned).ok_or_else(|| {
        let available: Vec<String> = registry
            .0
            .iter()
            .map(|(num, _)| format!("[Image #{num}]"))
            .collect();
        xai_tool_runtime::ToolError::invalid_arguments(format!(
            "image reference {reference:?} does not match any attached image. Available: {}.",
            available.join(", ")
        ))
    })
}

// ---------------------------------------------------------------------------
// Tool input / schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ImageEditInput {
    #[schemars(
        description = "A text description of the desired edit or transformation. Describe what the output image should look like, referencing the input image(s)."
    )]
    pub prompt: String,

    #[schemars(
        description = "Reference image(s) to condition the edit on. Each is one reference, in priority order: (1) a user attachment — its placeholder token, e.g. \"[Image #1]\" (attachments have no path you can see, so never invent one); (2) an absolute filesystem path the user gave you; (3) a `data:image/...;base64,...` URL."
    )]
    pub image: Vec<String>,

    #[serde(default = "default_aspect_ratio")]
    #[schemars(
        description = "The aspect ratio of the output image. For single-image edits this is ignored — the output matches the input image's aspect ratio. For multi-image edits, defaults to 'auto'. Supported values: 1:1, 16:9, 9:16, 4:3, 3:4, 3:2, 2:3, 2:1, 1:2, 19.5:9, 9:19.5, 20:9, 9:20, auto."
    )]
    pub aspect_ratio: String,
}

fn default_aspect_ratio() -> String {
    "auto".to_owned()
}

// ---------------------------------------------------------------------------
// Tool implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct ImageEditTool;

impl crate::types::tool_metadata::ToolMetadata for ImageEditTool {
    fn kind(&self) -> ToolKind {
        ToolKind::ImageGen
    }

    fn tool_namespace(&self) -> ToolNamespace {
        ToolNamespace::GrokBuild
    }

    fn description_template(&self) -> &str {
        r##"Edit or transform existing image(s) with the selected image provider; use instead of image_gen for image-to-image work (preserve likeness, transfer style, remix). Returns the saved image's absolute path. When telling the user where it was saved, refer to it by its short session-relative path (e.g. `images/1.jpg`) rather than the absolute path, so it renders as a clickable link that opens the image. Each required `image` is one reference — a user-attachment token (e.g. "[Image #1]"), an absolute filesystem path, or a `data:image/...;base64,...` URL (see the `image` parameter for the resolution order and details)."##
    }

    fn requires_expr(&self) -> Expr<ToolRequirement> {
        Expr::True
    }
}

impl xai_tool_runtime::Tool for ImageEditTool {
    type Args = ImageEditInput;
    type Output = ToolOutput;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new("image_edit").expect("valid tool id")
    }

    fn description(
        &self,
        _ctx: &::xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        xai_tool_types::ToolDescription::new(
            "image_edit",
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
        name = "tool.image_edit",
        skip_all,
        fields(prompt_len = input.prompt.len(), num_images = input.image.len(), aspect_ratio = %input.aspect_ratio)
    )]
    async fn run(
        &self,
        ctx: xai_tool_runtime::ToolCallContext,
        input: ImageEditInput,
    ) -> Result<ToolOutput, xai_tool_runtime::ToolError> {
        use crate::types::tool_metadata::shared_resources;
        let resources = shared_resources(&ctx)?;

        if input.image.is_empty() {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "image_edit requires at least one reference image. \
                 Use image_gen for text-only generation.",
            ));
        }
        if input.image.len() > MAX_REFERENCE_IMAGES {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(format!(
                "image_edit accepts at most {MAX_REFERENCE_IMAGES} reference images",
            )));
        }

        let client = {
            let res = resources.lock().await;
            res.require::<ImageGenClient>()?.clone()
        };

        // Free / X Basic users are zero-limited on Imagine server-side; return
        // the upsell prose instead of a doomed request (shares `image_gen`'s
        // message and short-circuits before resolving any attachments).
        if client.is_tier_restricted() {
            return Ok(ToolOutput::Text(
                super::image_gen::TIER_RESTRICTED_UPSELL.into(),
            ));
        }

        // Snapshot the per-turn attachment registry so `[Image #N]` tokens
        // resolve to the real attachment (see `resolve_attachment_reference`).
        let attached_images = {
            let res = resources.lock().await;
            res.get::<crate::types::resources::AttachedImages>()
                .cloned()
        };

        // Use the same filesystem abstraction as the other local tools. The
        // host wires this to its sandbox-aware implementation; direct
        // `tokio::fs` reads would bypass that boundary.
        let file_system = {
            let res = resources.lock().await;
            res.require::<FileSystem>()?.0.clone()
        };

        // Resolve all references to provider-ready data URLs.
        let mut data_urls = Vec::with_capacity(input.image.len());
        let mut codex_reference_bytes = 0;
        for r in &input.image {
            let resolved = resolve_attachment_reference(r, attached_images.as_ref())?;
            let reference =
                resolve_to_data_url(&resolved, client.backend(), file_system.as_ref()).await?;
            if client.backend() == ImageGenBackend::OpenAiCodex {
                codex_reference_bytes =
                    checked_codex_reference_total(codex_reference_bytes, reference.prepared_bytes)?;
            }
            data_urls.push(reference.data_url);
        }
        tracing::info!(count = data_urls.len(), "resolved image references");

        let payload = build_edit_payload(
            client.backend(),
            &input.prompt,
            &data_urls,
            &input.aspect_ratio,
        );
        // `serde_json::Value` owns its image URL strings; release the source
        // vector before reqwest serializes another copy of the request body.
        drop(data_urls);

        let response = client.post_json("images/edits", &payload).await?;

        let status = response.status();
        if !status.is_success() {
            tracing::warn!(http_status = %status, "image provider edit request failed");
            return Err(xai_tool_runtime::ToolError::new(
                xai_tool_runtime::ToolErrorKind::Custom,
                format!("Image edit failed with HTTP {status}"),
            )
            .with_details(client.http_failure_details(status)));
        }

        let resp_json = read_image_response(response).await?;

        let b64_data = resp_json.b64_data().unwrap_or("");
        if b64_data.is_empty() {
            return Err(xai_tool_runtime::ToolError::invalid_arguments(
                "Image edit returned no image data.",
            ));
        }

        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(b64_data)
            .map_err(|e| {
                xai_tool_runtime::ToolError::invalid_arguments(format!(
                    "Failed to decode base64 image data: {e}"
                ))
            })?;
        validate_generated_image(&image_bytes)?;

        let session_folder = {
            let res = resources.lock().await;
            res.require::<SessionFolder>()?.0.clone()
        };

        let extension = super::image_gen::generated_image_extension(&image_bytes);
        let absolute_path = client
            .writer()
            .save(&session_folder, &image_bytes, extension)
            .await
            .map_err(|e| xai_tool_runtime::ToolError::invalid_arguments(e.to_string()))?;

        tracing::info!(
            path = %absolute_path.display(),
            bytes = image_bytes.len(),
            "edited image saved to disk"
        );

        Ok(ToolOutput::ImageEdit(MediaGenOutput::new(absolute_path)))
    }
}

fn build_edit_payload(
    backend: ImageGenBackend,
    prompt: &str,
    data_urls: &[String],
    aspect_ratio: &str,
) -> serde_json::Value {
    match backend {
        ImageGenBackend::XaiImagine => {
            let mut payload = serde_json::json!({
                "model": XAI_IMAGINE_MODEL,
                "prompt": prompt,
                "n": 1,
                "resolution": "1k",
                "response_format": "b64_json",
            });
            // xAI API: single ref → `image`; multiple → `images`.
            let mut imgs: Vec<serde_json::Value> = data_urls
                .iter()
                .map(|u| serde_json::json!({ "url": u }))
                .collect();
            if imgs.len() == 1 {
                payload["image"] = imgs.pop().expect("one image");
            } else {
                payload["images"] = serde_json::Value::Array(imgs);
                payload["aspect_ratio"] = serde_json::json!(aspect_ratio);
            }
            payload
        }
        ImageGenBackend::OpenAiCodex => serde_json::json!({
            "model": OPENAI_CODEX_IMAGE_MODEL,
            "prompt": prompt,
            "background": "auto",
            "quality": "auto",
            "size": super::image_gen::codex_image_size(aspect_ratio),
            "images": data_urls
                .iter()
                .map(|image_url| serde_json::json!({"image_url": image_url}))
                .collect::<Vec<_>>(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::tool_metadata::test_ctx_with_call_id;

    struct CodexEditTestAuth;

    impl crate::types::ApiKeyProvider for CodexEditTestAuth {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn current_request_auth_async(
            &self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Option<crate::types::RequestAuth>> + Send + '_>,
        > {
            Box::pin(std::future::ready(Some(
                crate::types::RequestAuth::for_provider_snapshot(
                    "openai_codex",
                    crate::types::RequestCredentialSnapshot::new("edit-credential", 3),
                    [
                        ("authorization".to_owned(), "Bearer edit-access".to_owned()),
                        ("chatgpt-account-id".to_owned(), "edit-account".to_owned()),
                        ("x-openai-fedramp".to_owned(), "true".to_owned()),
                    ],
                ),
            )))
        }
    }

    struct ZeroGenerationCodexEditTestAuth;

    impl crate::types::ApiKeyProvider for ZeroGenerationCodexEditTestAuth {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn current_request_auth_async(
            &self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Option<crate::types::RequestAuth>> + Send + '_>,
        > {
            Box::pin(std::future::ready(Some(
                crate::types::RequestAuth::for_provider_snapshot(
                    "openai_codex",
                    crate::types::RequestCredentialSnapshot::new("edit-credential", 0),
                    [
                        ("authorization".to_owned(), "Bearer edit-access".to_owned()),
                        ("chatgpt-account-id".to_owned(), "edit-account".to_owned()),
                    ],
                ),
            )))
        }
    }

    #[test]
    fn tool_name_and_description() {
        let tool = ImageEditTool;
        assert_eq!(xai_tool_runtime::Tool::id(&tool).as_str(), "image_edit");
        let desc = crate::types::tool_metadata::ToolMetadata::description_template(&tool);
        assert!(desc.contains("Edit or transform"));
    }

    #[test]
    fn input_deserialization() {
        let input: ImageEditInput =
            serde_json::from_str(r#"{"prompt": "anime style", "image": ["/Users/me/photo.jpg"]}"#)
                .unwrap();
        assert_eq!(input.prompt, "anime style");
        assert_eq!(input.image, vec!["/Users/me/photo.jpg"]);
        assert_eq!(input.aspect_ratio, "auto");
    }

    #[test]
    fn codex_edit_payload_matches_current_contract() {
        let payload = build_edit_payload(
            ImageGenBackend::OpenAiCodex,
            "make it nocturnal",
            &["data:image/png;base64,AAAA".to_owned()],
            "16:9",
        );
        assert_eq!(
            payload,
            serde_json::json!({
                "model": OPENAI_CODEX_IMAGE_MODEL,
                "prompt": "make it nocturnal",
                "background": "auto",
                "quality": "auto",
                "size": "1280x720",
                "images": [{"image_url": "data:image/png;base64,AAAA"}],
            })
        );
        assert!(payload.get("aspect_ratio").is_none());
        assert!(payload.get("response_format").is_none());
    }

    #[tokio::test]
    async fn codex_edit_uses_exact_provider_contract_without_xai_headers() {
        use std::sync::Arc;
        use wiremock::matchers::{body_json, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let payload = build_edit_payload(
            ImageGenBackend::OpenAiCodex,
            "make it nocturnal",
            &["data:image/png;base64,AAAA".to_owned()],
            "16:9",
        );
        Mock::given(method("POST"))
            .and(path("/images/edits"))
            .and(header("authorization", "Bearer edit-access"))
            .and(header("chatgpt-account-id", "edit-account"))
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
            .and(body_json(payload.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"b64_json": "unused-by-this-contract-test"}]
            })))
            .mount(&server)
            .await;

        let config = super::super::image_gen::ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: crate::types::SharedApiKeyProvider = Arc::new(CodexEditTestAuth);
        let client = ImageGenClient::new(&config, Some(provider)).unwrap();
        let response = client.post_json("images/edits", &payload).await.unwrap();
        assert!(response.status().is_success());

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
    async fn codex_edit_rejects_zero_credential_generation_before_request() {
        use std::sync::Arc;

        let server = wiremock::MockServer::start().await;
        let config = super::super::image_gen::ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: crate::types::SharedApiKeyProvider =
            Arc::new(ZeroGenerationCodexEditTestAuth);
        let client = ImageGenClient::new(&config, Some(provider)).unwrap();
        let error = client
            .post_json("images/edits", &serde_json::json!({}))
            .await
            .expect_err("generation-zero credentials must fail before edit dispatch");

        assert!(error.to_string().contains("credential generation"));
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[test]
    fn codex_edit_rejects_noncanonical_provider_origin() {
        use std::sync::Arc;

        let config = super::super::image_gen::ImageGenConfig::OpenAiCodex {
            base_url: "https://chatgpt.com:444/backend-api/codex".to_owned(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: crate::types::SharedApiKeyProvider = Arc::new(CodexEditTestAuth);
        let error = ImageGenClient::new(&config, Some(provider))
            .err()
            .expect("edit credentials must not be bound to a noncanonical origin");

        assert!(error.to_string().contains("ChatGPT Codex endpoint"));
    }

    #[test]
    fn input_requires_image() {
        // image field is required by schema — empty array is a runtime check.
        let input: ImageEditInput =
            serde_json::from_str(r#"{"prompt": "test", "image": []}"#).unwrap();
        assert!(input.image.is_empty());
    }

    #[tokio::test]
    async fn rejects_empty_image_array() {
        let tool = ImageEditTool;
        let resources = crate::types::resources::Resources::new();
        let result = xai_tool_runtime::Tool::run(
            &tool,
            test_ctx_with_call_id(resources.into_shared(), "test-call"),
            ImageEditInput {
                prompt: "test".into(),
                image: vec![],
                aspect_ratio: "auto".into(),
            },
        )
        .await;
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least one reference image"), "got: {err}");
    }

    #[tokio::test]
    async fn errors_when_client_missing() {
        let tool = ImageEditTool;
        let resources = crate::types::resources::Resources::new();
        let result = xai_tool_runtime::Tool::run(
            &tool,
            test_ctx_with_call_id(resources.into_shared(), "test-call"),
            ImageEditInput {
                prompt: "test".into(),
                image: vec!["/some/path.jpg".into()],
                aspect_ratio: "auto".into(),
            },
        )
        .await;
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing required resource"), "got: {err}");
    }

    // ── compress_reference ───────────────────────────────────────────

    fn tiny_jpeg() -> Vec<u8> {
        use image::{DynamicImage, RgbImage};
        let img = DynamicImage::ImageRgb8(RgbImage::new(2, 2));
        let mut buf = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Jpeg,
        )
        .unwrap();
        buf
    }

    fn tiny_png() -> Vec<u8> {
        use image::{DynamicImage, RgbaImage};
        let img = DynamicImage::ImageRgba8(RgbaImage::new(2, 2));
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    fn noisy_png() -> Vec<u8> {
        use image::{DynamicImage, Rgb, RgbImage};

        let mut state = 0x1234_5678_u32;
        let image = RgbImage::from_fn(512, 512, |_x, _y| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            Rgb([(state >> 24) as u8, (state >> 16) as u8, (state >> 8) as u8])
        });
        let mut original = Vec::new();
        DynamicImage::ImageRgb8(image)
            .write_to(&mut std::io::Cursor::new(&mut original), ImageFormat::Png)
            .unwrap();
        original
    }

    #[test]
    fn compress_small_jpeg_passthrough() {
        let jpeg = tiny_jpeg();
        let (out, mime) = compress_reference(jpeg.clone()).unwrap();
        assert_eq!(out, jpeg);
        assert_eq!(mime, "image/jpeg");
    }

    #[test]
    fn compress_small_png_passthrough() {
        let png = tiny_png();
        let (out, mime) = compress_reference(png.clone()).unwrap();
        assert_eq!(out, png);
        assert_eq!(mime, "image/png");
    }

    #[test]
    fn compress_oversized_shrinks() {
        use image::{DynamicImage, RgbImage};
        let mut img = RgbImage::new(1600, 1600);
        for (i, px) in img.pixels_mut().enumerate() {
            let v = (i * 37 + 13) as u8;
            *px = image::Rgb([v, v.wrapping_add(80), v.wrapping_add(160)]);
        }
        let mut buf = Vec::new();
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 100);
        DynamicImage::ImageRgb8(img)
            .write_with_encoder(enc)
            .unwrap();
        assert!(buf.len() > MAX_REF_RAW_BYTES);

        let (out, mime) = compress_reference(buf).unwrap();
        assert!(out.len() <= MAX_REF_RAW_BYTES);
        assert!(mime == "image/jpeg" || mime == "image/png");
    }

    #[test]
    fn codex_preserves_native_reference_above_imagine_limit() {
        let original = noisy_png();
        assert!(original.len() > MAX_REF_RAW_BYTES);

        let (prepared, mime) = prepare_codex_reference(original.clone()).unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(prepared, original, "Codex edit input must remain original");
    }

    #[test]
    fn codex_converts_non_native_reference_to_png() {
        use image::{DynamicImage, RgbaImage};

        let image = DynamicImage::ImageRgba8(RgbaImage::new(32, 32));
        let mut gif = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut gif), ImageFormat::Gif)
            .unwrap();

        let (prepared, mime) = prepare_codex_reference(gif).unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(image::guess_format(&prepared).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn codex_reference_input_has_explicit_byte_cap() {
        let err = prepare_codex_reference(vec![0; MAX_REF_INPUT_BYTES + 1])
            .unwrap_err()
            .to_string();
        assert!(err.contains("size limit"), "got: {err}");
    }

    #[test]
    fn codex_reference_set_has_aggregate_byte_cap() {
        assert_eq!(
            checked_codex_reference_total(MAX_CODEX_REF_TOTAL_BYTES - 1, 1).unwrap(),
            MAX_CODEX_REF_TOTAL_BYTES
        );
        let err = checked_codex_reference_total(MAX_CODEX_REF_TOTAL_BYTES - 1, 2)
            .unwrap_err()
            .to_string();
        assert!(err.contains("combined image references"), "got: {err}");
    }

    #[tokio::test]
    async fn codex_edit_tool_reads_virtual_fs_and_sends_original_source() {
        use crate::computer::local::MockFs;
        use crate::types::resources::{FileSystem, SessionFolder};
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let output_png = tiny_png();
        Mock::given(method("POST"))
            .and(path("/images/edits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{
                    "b64_json": base64::engine::general_purpose::STANDARD.encode(&output_png)
                }]
            })))
            .mount(&server)
            .await;

        let original = noisy_png();
        assert!(original.len() > MAX_REF_RAW_BYTES);
        let fs = Arc::new(MockFs::new());
        fs.set_file("/sandbox/source.png", &original).await;
        let session = tempfile::tempdir().unwrap();
        let config = super::super::image_gen::ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: crate::types::SharedApiKeyProvider = Arc::new(CodexEditTestAuth);
        let mut resources = crate::types::resources::Resources::new();
        resources.insert(ImageGenClient::new(&config, Some(provider)).unwrap());
        resources.insert(FileSystem(fs));
        resources.insert(SessionFolder(session.path().to_path_buf()));

        let output = xai_tool_runtime::Tool::run(
            &ImageEditTool,
            test_ctx_with_call_id(resources.into_shared(), "codex-edit"),
            ImageEditInput {
                prompt: "preserve every source detail".to_owned(),
                image: vec!["/sandbox/source.png".to_owned()],
                aspect_ratio: "auto".to_owned(),
            },
        )
        .await
        .unwrap();
        let ToolOutput::ImageEdit(media) = output else {
            panic!("expected image edit output");
        };
        assert!(media.path.is_file());

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        let data_url = body["images"][0]["image_url"].as_str().unwrap();
        let encoded = data_url.strip_prefix("data:image/png;base64,").unwrap();
        let sent = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(sent, original, "Codex request must retain source bytes");
    }

    #[tokio::test]
    async fn generated_output_path_is_reusable_by_later_codex_edit() {
        use crate::computer::local::LocalFs;
        use crate::implementations::grok_build::image_gen::{ImageGenInput, ImageGenTool};
        use crate::types::resources::{FileSystem, SessionFolder};
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let generated = noisy_png();
        let edited = tiny_png();
        Mock::given(method("POST"))
            .and(path("/images/generations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{
                    "b64_json": base64::engine::general_purpose::STANDARD.encode(&generated)
                }]
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/images/edits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{
                    "b64_json": base64::engine::general_purpose::STANDARD.encode(&edited)
                }]
            })))
            .mount(&server)
            .await;

        let config = super::super::image_gen::ImageGenConfig::OpenAiCodex {
            base_url: server.uri(),
            image_gen_enabled: true,
            image_edit_enabled: true,
        };
        let provider: crate::types::SharedApiKeyProvider = Arc::new(CodexEditTestAuth);
        let session = tempfile::tempdir().unwrap();
        let mut resources = crate::types::resources::Resources::new();
        resources.insert(ImageGenClient::new(&config, Some(provider)).unwrap());
        resources.insert(FileSystem(Arc::new(LocalFs)));
        resources.insert(SessionFolder(session.path().to_path_buf()));
        let resources = resources.into_shared();

        let generated_output = xai_tool_runtime::Tool::run(
            &ImageGenTool,
            test_ctx_with_call_id(resources.clone(), "generate"),
            ImageGenInput {
                prompt: "a reusable source".to_owned(),
                aspect_ratio: "auto".to_owned(),
            },
        )
        .await
        .unwrap();
        let ToolOutput::ImageGen(generated_media) = generated_output else {
            panic!("expected generated image output");
        };

        let edited_output = xai_tool_runtime::Tool::run(
            &ImageEditTool,
            test_ctx_with_call_id(resources, "edit-later"),
            ImageEditInput {
                prompt: "edit the generated source".to_owned(),
                image: vec![generated_media.path.to_string_lossy().into_owned()],
                aspect_ratio: "auto".to_owned(),
            },
        )
        .await
        .unwrap();
        assert!(matches!(edited_output, ToolOutput::ImageEdit(_)));

        let requests = server.received_requests().await.unwrap();
        let edit_request = requests
            .iter()
            .find(|request| request.url.path() == "/images/edits")
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&edit_request.body).unwrap();
        let data_url = body["images"][0]["image_url"].as_str().unwrap();
        let encoded = data_url.strip_prefix("data:image/png;base64,").unwrap();
        let sent = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(sent, generated);
    }

    // ── resolve_to_data_url ──────────────────────────────────────────

    struct LimitedReadOnlyFs(Vec<u8>);

    #[async_trait::async_trait]
    impl crate::computer::types::AsyncFileSystem for LimitedReadOnlyFs {
        async fn read_file(
            &self,
            _path: &std::path::Path,
        ) -> Result<Vec<u8>, crate::computer::types::ComputerError> {
            panic!("image_edit must not use the unbounded read_file method")
        }

        async fn read_file_limited(
            &self,
            _path: &std::path::Path,
            max_bytes: usize,
        ) -> Result<Vec<u8>, crate::computer::types::ComputerError> {
            assert_eq!(max_bytes, MAX_REF_INPUT_BYTES);
            Ok(self.0.clone())
        }

        async fn write_file(
            &self,
            _path: &std::path::Path,
            _data: &[u8],
        ) -> Result<(), crate::computer::types::ComputerError> {
            unreachable!()
        }

        async fn delete_file(
            &self,
            _path: &std::path::Path,
        ) -> Result<(), crate::computer::types::ComputerError> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn resolve_filesystem_path_uses_bounded_fs_read() {
        let fs = LimitedReadOnlyFs(tiny_jpeg());
        let reference = resolve_to_data_url("/virtual/test.jpg", ImageGenBackend::XaiImagine, &fs)
            .await
            .unwrap();
        assert!(reference.data_url.starts_with("data:image/jpeg;base64,"));
    }

    #[tokio::test]
    async fn resolve_filesystem_path() {
        let fs = crate::computer::local::MockFs::new();
        fs.set_file("/virtual/test.jpg", &tiny_jpeg()).await;
        let url = resolve_to_data_url("/virtual/test.jpg", ImageGenBackend::XaiImagine, &fs)
            .await
            .unwrap();
        assert!(url.data_url.starts_with("data:image/jpeg;base64,"));
    }

    #[tokio::test]
    async fn resolve_data_url_roundtrip() {
        let jpeg = tiny_jpeg();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&jpeg);
        let input = format!("data:image/jpeg;base64,{b64}");
        let fs = crate::computer::local::MockFs::new();
        let url = resolve_to_data_url(&input, ImageGenBackend::XaiImagine, &fs)
            .await
            .unwrap();
        assert!(url.data_url.starts_with("data:image/jpeg;base64,"));
    }

    #[tokio::test]
    async fn resolve_missing_file_errors() {
        let fs = crate::computer::local::MockFs::new();
        assert!(
            resolve_to_data_url("/nonexistent/image.jpg", ImageGenBackend::XaiImagine, &fs)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn resolve_malformed_data_url_errors() {
        let fs = crate::computer::local::MockFs::new();
        assert!(
            resolve_to_data_url("data:image/jpeg", ImageGenBackend::XaiImagine, &fs)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn resolve_file_uri_reads_path() {
        let fs = crate::computer::local::MockFs::new();
        fs.set_file("/virtual/test.jpg", &tiny_jpeg()).await;
        let url = resolve_to_data_url("file:///virtual/test.jpg", ImageGenBackend::XaiImagine, &fs)
            .await
            .unwrap();
        assert!(url.data_url.starts_with("data:image/jpeg;base64,"));
    }

    // ── parse_attachment_token ───────────────────────────────────────

    #[test]
    fn parse_attachment_token_accepts_known_forms() {
        assert_eq!(parse_attachment_token("[Image #1]"), Some(1));
        assert_eq!(parse_attachment_token("Image #2"), Some(2));
        assert_eq!(parse_attachment_token("image #3"), Some(3));
        assert_eq!(parse_attachment_token("[image #4]"), Some(4));
        assert_eq!(parse_attachment_token("Image#5"), Some(5));
        assert_eq!(parse_attachment_token("#6"), Some(6));
        assert_eq!(parse_attachment_token("  [Image #7]  "), Some(7));
    }

    #[test]
    fn parse_attachment_token_rejects_non_tokens() {
        assert_eq!(parse_attachment_token("/Users/me/photo.jpg"), None);
        assert_eq!(parse_attachment_token("data:image/png;base64,AAAA"), None);
        assert_eq!(parse_attachment_token("file:///tmp/x.png"), None);
        assert_eq!(parse_attachment_token("[Image #0]"), None);
        assert_eq!(parse_attachment_token("[Image #]"), None);
        assert_eq!(parse_attachment_token("Image one"), None);
        assert_eq!(parse_attachment_token(""), None);
    }

    // ── resolve_attachment_reference ─────────────────────────────────

    #[test]
    fn resolve_reference_passes_through_non_tokens() {
        let resolved = resolve_attachment_reference("/Users/me/photo.jpg", None).unwrap();
        assert_eq!(resolved, "/Users/me/photo.jpg");
    }

    #[test]
    fn resolve_reference_maps_token_to_registry() {
        let attached = crate::types::resources::AttachedImages(vec![
            (1, "/tmp/a.png".to_owned()),
            (2, "/tmp/b.png".to_owned()),
        ]);
        assert_eq!(
            resolve_attachment_reference("[Image #1]", Some(&attached)).unwrap(),
            "/tmp/a.png"
        );
        assert_eq!(
            resolve_attachment_reference("Image #2", Some(&attached)).unwrap(),
            "/tmp/b.png"
        );
    }

    #[test]
    fn resolve_reference_maps_by_number_not_position() {
        // After a mid-compose chip removal the surviving numbers are
        // non-contiguous (`#1`, `#3`). Resolution must key on the number,
        // not the list position, or `[Image #3]` would resolve to the wrong
        // file (or wrongly error).
        let attached = crate::types::resources::AttachedImages(vec![
            (1, "/tmp/first.png".to_owned()),
            (3, "/tmp/third.png".to_owned()),
        ]);
        assert_eq!(
            resolve_attachment_reference("[Image #3]", Some(&attached)).unwrap(),
            "/tmp/third.png"
        );
        // `[Image #2]` was removed → no match.
        assert!(resolve_attachment_reference("[Image #2]", Some(&attached)).is_err());
    }

    #[test]
    fn resolve_reference_token_without_registry_errors() {
        let err = resolve_attachment_reference("[Image #1]", None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("re-attach"), "got: {err}");
    }

    #[test]
    fn resolve_reference_unmatched_number_errors() {
        let attached = crate::types::resources::AttachedImages(vec![(1, "/tmp/a.png".to_owned())]);
        let err = resolve_attachment_reference("[Image #2]", Some(&attached))
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"), "got: {err}");
        assert!(err.contains("[Image #1]"), "should list available: {err}");
    }
}
