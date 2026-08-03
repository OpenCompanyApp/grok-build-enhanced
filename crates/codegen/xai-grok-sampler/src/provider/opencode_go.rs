use reqwest::header::{HeaderMap, HeaderName};
use xai_grok_sampling_types::{
    ApiBackend, ContentPart, ConversationItem, ConversationRequest, CredentialBinding,
    CredentialSourceId, OPENCODE_GO_BASE_URL, ProviderId, Result, SamplingError,
};

const MAX_MEDIA_BYTES: u64 = 100 * 1024 * 1024;

pub(crate) fn validate_config(provider: ProviderId, base_url: &str) -> Result<()> {
    if !provider.is_open_code_go() {
        return Ok(());
    }
    let normalized = base_url.trim_end_matches('/');
    #[cfg(any(test, feature = "test-support"))]
    let valid = normalized == OPENCODE_GO_BASE_URL
        || reqwest::Url::parse(normalized).is_ok_and(|url| {
            url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost"))
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
        });
    #[cfg(not(any(test, feature = "test-support")))]
    let valid = normalized == OPENCODE_GO_BASE_URL;
    if !valid {
        return Err(SamplingError::InvalidConfiguration(
            "OpenCode Go credentials may only be sent to the canonical Go endpoint",
        ));
    }
    Ok(())
}

pub(crate) fn http_client(force_http1: bool) -> Result<reqwest::Client> {
    let mut builder = xai_grok_provider_http::with_extra_root_certificates(
        reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()),
    );
    if force_http1 {
        builder = builder
            .pool_max_idle_per_host(0)
            .pool_idle_timeout(std::time::Duration::ZERO)
            .http1_only();
    }
    builder.build().map_err(SamplingError::Http)
}

fn allowed_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization"
            | "x-api-key"
            | "anthropic-version"
            | "content-type"
            | "accept"
            | "user-agent"
            | "traceparent"
            | "tracestate"
    )
}

pub(crate) fn is_protected_header(name: &HeaderName) -> bool {
    !allowed_header(name)
        || matches!(
            name.as_str(),
            "authorization" | "x-api-key" | "anthropic-version"
        )
}

pub(crate) fn seal_headers(
    headers: &HeaderMap,
    backend: ApiBackend,
    credential_binding: Option<&CredentialBinding>,
) -> Result<()> {
    let binding = credential_binding.ok_or(SamplingError::InvalidConfiguration(
        "OpenCode Go request authentication did not provide a credential binding",
    ))?;
    if binding.provider != ProviderId::OpenCodeGo
        || binding.source != CredentialSourceId::OpenCodeGoApiKey
        || binding.generation == 0
        || binding
            .record_id
            .as_deref()
            .is_none_or(|record_id| record_id.trim().is_empty())
    {
        return Err(SamplingError::InvalidConfiguration(
            "OpenCode Go request authentication returned a foreign credential binding",
        ));
    }
    if headers.keys().any(|name| !allowed_header(name)) {
        return Err(SamplingError::InvalidConfiguration(
            "unapproved headers are forbidden on OpenCode Go requests",
        ));
    }
    let bearer = headers
        .get("authorization")
        .is_some_and(reqwest::header::HeaderValue::is_sensitive);
    let x_api_key = headers
        .get("x-api-key")
        .is_some_and(reqwest::header::HeaderValue::is_sensitive);
    match backend {
        ApiBackend::Messages if !x_api_key || bearer => Err(SamplingError::InvalidConfiguration(
            "OpenCode Go Messages requires one sensitive x-api-key header",
        )),
        ApiBackend::ChatCompletions | ApiBackend::Responses if !bearer || x_api_key => {
            Err(SamplingError::InvalidConfiguration(
                "OpenCode Go OpenAI-compatible routes require one sensitive bearer header",
            ))
        }
        _ => Ok(()),
    }
}

pub(crate) async fn prepare_media_inputs(
    provider: ProviderId,
    backend: ApiBackend,
    request: &mut ConversationRequest,
) -> Result<()> {
    let model = request.model.clone().unwrap_or_default();
    let has_video = request.items.iter().any(|item| {
        matches!(item, ConversationItem::User(user) if user.content.iter().any(|part| {
            matches!(part, ContentPart::Video { .. })
        }))
    });
    let has_audio_or_document = request.items.iter().any(|item| {
        matches!(item, ConversationItem::User(user) if user.content.iter().any(|part| {
            matches!(part, ContentPart::Audio { .. } | ContentPart::Document { .. })
        }))
    });
    if !provider.is_open_code_go() {
        if has_audio_or_document || (has_video && !provider.is_kimi_code()) {
            return Err(SamplingError::InvalidConfiguration(
                "audio, video, and document inputs require the OpenCode Go provider",
            ));
        }
        return Ok(());
    }

    for item in &mut request.items {
        let ConversationItem::User(user) = item else {
            continue;
        };
        for part in &mut user.content {
            match part {
                ContentPart::Image { .. } if !supports_media(&model, MediaKind::Image) => {
                    return Err(SamplingError::InvalidConfiguration(
                        "the selected OpenCode Go model does not advertise image input",
                    ));
                }
                ContentPart::Video { path, mime_type } => {
                    if !supports_media(&model, MediaKind::Video) {
                        return Err(SamplingError::InvalidConfiguration(
                            "the selected OpenCode Go model does not advertise video input",
                        ));
                    }
                    if backend == ApiBackend::Responses {
                        return Err(SamplingError::InvalidConfiguration(
                            "OpenCode Go video input is unavailable on the Responses route",
                        ));
                    }
                    validate_mime(
                        mime_type,
                        &[
                            "video/mp4",
                            "video/mpeg",
                            "video/quicktime",
                            "video/webm",
                            "video/x-matroska",
                            "video/x-msvideo",
                            "video/x-flv",
                            "video/3gpp",
                        ],
                    )?;
                    *path = read_data_url(path, mime_type).await?.into();
                }
                ContentPart::Audio { path, mime_type } => {
                    if !supports_media(&model, MediaKind::Audio) {
                        return Err(SamplingError::InvalidConfiguration(
                            "the selected OpenCode Go model does not advertise audio input",
                        ));
                    }
                    if backend != ApiBackend::ChatCompletions {
                        return Err(SamplingError::InvalidConfiguration(
                            "OpenCode Go audio input requires the Chat Completions route",
                        ));
                    }
                    validate_mime(mime_type, &["audio/mpeg", "audio/wav"])?;
                    *path = read_data_url(path, mime_type).await?.into();
                }
                ContentPart::Document { path, mime_type } => {
                    if !supports_media(&model, MediaKind::Document) {
                        return Err(SamplingError::InvalidConfiguration(
                            "the selected OpenCode Go model does not advertise PDF input",
                        ));
                    }
                    validate_mime(mime_type, &["application/pdf"])?;
                    *path = read_data_url(path, mime_type).await?.into();
                }
                ContentPart::Text { .. } | ContentPart::Image { .. } => {}
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum MediaKind {
    Image,
    Audio,
    Video,
    Document,
}

fn supports_media(model: &str, kind: MediaKind) -> bool {
    let model = model.strip_prefix("opencode-go/").unwrap_or(model);
    let supported = match kind {
        MediaKind::Image => &[
            "grok-4.5",
            "gpt-5.6-luna",
            "kimi-k3",
            "kimi-k2.7-code",
            "kimi-k2.6",
            "kimi-k2.5",
            "mimo-v2.5",
            "mimo-v2-omni",
            "minimax-m3",
            "qwen3.7-plus",
            "qwen3.6-plus",
            "qwen3.5-plus",
        ][..],
        MediaKind::Audio => &["mimo-v2.5", "mimo-v2-omni"][..],
        MediaKind::Video => &[
            "kimi-k3",
            "kimi-k2.7-code",
            "kimi-k2.6",
            "kimi-k2.5",
            "mimo-v2.5",
            "minimax-m3",
            "qwen3.7-plus",
            "qwen3.6-plus",
            "qwen3.5-plus",
        ][..],
        MediaKind::Document => &["gpt-5.6-luna", "mimo-v2-omni"][..],
    };
    supported.contains(&model)
}

fn validate_mime(mime_type: &str, allowed: &[&str]) -> Result<()> {
    if !allowed.contains(&mime_type) {
        return Err(SamplingError::InvalidConfiguration(
            "OpenCode Go media input has an unsupported MIME type",
        ));
    }
    Ok(())
}

async fn read_data_url(path: &str, mime_type: &str) -> Result<String> {
    if path.starts_with("data:") {
        let expected = format!("data:{mime_type};base64,");
        if path.starts_with(&expected) && path.len() <= MAX_MEDIA_BYTES as usize * 2 {
            return Ok(path.to_owned());
        }
        return Err(SamplingError::InvalidConfiguration(
            "OpenCode Go media data URL is invalid",
        ));
    }
    let metadata = tokio::fs::metadata(path).await.map_err(|_| {
        SamplingError::InvalidConfiguration("OpenCode Go media file is missing or unreadable")
    })?;
    if !metadata.is_file() || metadata.len() > MAX_MEDIA_BYTES {
        return Err(SamplingError::InvalidConfiguration(
            "OpenCode Go media file exceeds the 100 MiB limit or is not a regular file",
        ));
    }
    let bytes = tokio::fs::read(path).await.map_err(|_| {
        SamplingError::InvalidConfiguration("OpenCode Go media file is missing or unreadable")
    })?;
    if bytes.len() as u64 > MAX_MEDIA_BYTES {
        return Err(SamplingError::InvalidConfiguration(
            "OpenCode Go media file exceeds the 100 MiB limit",
        ));
    }
    use base64::Engine as _;
    Ok(format!(
        "data:{mime_type};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use xai_grok_sampling_types::{ConversationItem, UserItem};

    fn request(model: &str, part: ContentPart) -> ConversationRequest {
        ConversationRequest {
            model: Some(model.to_owned()),
            items: vec![ConversationItem::User(UserItem {
                content: vec![part],
                ..Default::default()
            })],
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn foreign_provider_rejects_audio_before_filesystem_or_network() {
        let mut request = request(
            "mimo-v2.5",
            ContentPart::Audio {
                path: Arc::from("/missing/sentinel.mp3"),
                mime_type: Arc::from("audio/mpeg"),
            },
        );
        let error =
            prepare_media_inputs(ProviderId::Xai, ApiBackend::ChatCompletions, &mut request)
                .await
                .unwrap_err();
        assert!(error.to_string().contains("OpenCode Go provider"));
    }

    #[tokio::test]
    async fn supported_audio_is_bounded_and_encoded_only_at_request_time() {
        let path = std::env::temp_dir().join(format!("grok-go-media-{}.mp3", uuid::Uuid::new_v4()));
        tokio::fs::write(&path, b"synthetic-audio").await.unwrap();
        let mut request = request(
            "mimo-v2.5",
            ContentPart::Audio {
                path: Arc::from(path.to_string_lossy().into_owned()),
                mime_type: Arc::from("audio/mpeg"),
            },
        );
        prepare_media_inputs(
            ProviderId::OpenCodeGo,
            ApiBackend::ChatCompletions,
            &mut request,
        )
        .await
        .unwrap();
        let ConversationItem::User(user) = &request.items[0] else {
            unreachable!()
        };
        assert!(matches!(
            &user.content[0],
            ContentPart::Audio { path, .. } if path.starts_with("data:audio/mpeg;base64,")
        ));
        tokio::fs::remove_file(path).await.unwrap();
    }

    #[tokio::test]
    async fn model_capability_is_checked_before_opening_media() {
        let mut request = request(
            "glm-5.2",
            ContentPart::Video {
                path: Arc::from("/missing/sentinel.mp4"),
                mime_type: Arc::from("video/mp4"),
            },
        );
        let error = prepare_media_inputs(
            ProviderId::OpenCodeGo,
            ApiBackend::ChatCompletions,
            &mut request,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("does not advertise video"));
    }
}
