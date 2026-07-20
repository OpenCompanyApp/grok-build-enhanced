use super::*;
use crate::remote::DEFAULT_CONTEXT_WINDOW;
use xai_chat_state::conversation_util::replace_or_insert_system_head;

fn xai_image_gen_config_with_rotated_key(
    configured: &xai_grok_tools::implementations::grok_build::image_gen::ImageGenConfig,
    current_api_key: Option<&str>,
) -> Option<xai_grok_tools::implementations::grok_build::image_gen::ImageGenConfig> {
    use xai_grok_tools::implementations::grok_build::image_gen::ImageGenConfig;

    let ImageGenConfig::Enabled {
        api_key,
        base_url,
        extra_headers,
        image_gen_enabled,
        image_edit_enabled,
        model_override,
        tier_restricted,
    } = configured
    else {
        return None;
    };

    Some(ImageGenConfig::Enabled {
        api_key: current_api_key.unwrap_or(api_key).to_owned(),
        base_url: base_url.clone(),
        extra_headers: extra_headers.clone(),
        image_gen_enabled: *image_gen_enabled,
        image_edit_enabled: *image_edit_enabled,
        model_override: model_override.clone(),
        tier_restricted: *tier_restricted,
    })
}

fn remove_image_tool_definitions(bridge: &crate::tools::bridge::ToolBridge) {
    use xai_grok_tools::implementations::grok_build::{IMAGE_EDIT_TOOL_NAME, IMAGE_GEN_TOOL_NAME};

    bridge.unregister_tool_by_name(IMAGE_GEN_TOOL_NAME);
    bridge.unregister_tool_by_name(IMAGE_EDIT_TOOL_NAME);
}

async fn sync_image_tool_definitions(
    bridge: &crate::tools::bridge::ToolBridge,
    image_gen_enabled: bool,
    image_edit_enabled: bool,
) -> Result<(), xai_tool_runtime::ToolError> {
    use xai_grok_tools::implementations::grok_build::{
        IMAGE_EDIT_TOOL_NAME, IMAGE_GEN_TOOL_NAME, ImageEditTool, ImageGenTool,
    };

    if image_gen_enabled {
        if bridge.tool_kind(IMAGE_GEN_TOOL_NAME).is_none() {
            bridge
                .register_mcp_tools(IMAGE_GEN_TOOL_NAME.to_owned(), ImageGenTool, None)
                .await?;
        }
    } else {
        bridge.unregister_tool_by_name(IMAGE_GEN_TOOL_NAME);
    }

    if image_edit_enabled {
        if bridge.tool_kind(IMAGE_EDIT_TOOL_NAME).is_none() {
            bridge
                .register_mcp_tools(IMAGE_EDIT_TOOL_NAME.to_owned(), ImageEditTool, None)
                .await?;
        }
    } else {
        bridge.unregister_tool_by_name(IMAGE_EDIT_TOOL_NAME);
    }

    Ok(())
}

async fn refresh_image_gen_resource(
    bridge: &crate::tools::bridge::ToolBridge,
    config: Option<xai_grok_tools::implementations::grok_build::image_gen::ImageGenConfig>,
    api_key_provider: Option<xai_grok_tools::types::SharedApiKeyProvider>,
    attribution_callback: Option<xai_grok_tools::SharedAttributionCallback>,
) {
    use xai_grok_tools::implementations::grok_build::image_gen::ImageGenClient;

    let Some(config) = config else {
        remove_image_tool_definitions(bridge);
        bridge.remove_resource::<ImageGenClient>().await;
        return;
    };
    let image_gen_enabled = config.image_gen_enabled();
    let image_edit_enabled = config.image_edit_enabled();
    if !image_gen_enabled && !image_edit_enabled {
        remove_image_tool_definitions(bridge);
        bridge.remove_resource::<ImageGenClient>().await;
        return;
    }

    match ImageGenClient::new(&config, api_key_provider) {
        Ok(client) => {
            bridge
                .update_resource(client.with_attribution_callback(attribution_callback))
                .await;
            if let Err(error) =
                sync_image_tool_definitions(bridge, image_gen_enabled, image_edit_enabled).await
            {
                tracing::warn!(%error, "failed to restore image tool definitions after provider switch");
                remove_image_tool_definitions(bridge);
                bridge.remove_resource::<ImageGenClient>().await;
            }
        }
        Err(error) => {
            tracing::warn!(%error, "failed to restore image generation after provider switch");
            remove_image_tool_definitions(bridge);
            bridge.remove_resource::<ImageGenClient>().await;
        }
    }
}

fn video_gen_config_for_provider(
    provider: xai_grok_sampling_types::ProviderId,
    configured: &xai_grok_tools::implementations::grok_build::video_gen::VideoGenConfig,
    current_api_key: Option<&str>,
    current_xai_base_url: &str,
) -> xai_grok_tools::implementations::grok_build::video_gen::VideoGenConfig {
    use xai_grok_tools::implementations::grok_build::video_gen::VideoGenConfig;

    if provider != xai_grok_sampling_types::ProviderId::Xai {
        // Video generation is an xAI-owned resource. Codex and custom model
        // credentials must never be sent to that service.
        return VideoGenConfig::Disabled;
    }

    let Some(configured) = configured.xai_fallback() else {
        return VideoGenConfig::Disabled;
    };
    let VideoGenConfig::Enabled {
        api_key,
        base_url,
        extra_headers,
        zdr_video_output_s3,
        tier_restricted,
    } = configured
    else {
        return VideoGenConfig::Disabled;
    };

    VideoGenConfig::Enabled {
        api_key: current_api_key.unwrap_or(api_key).to_owned(),
        base_url: if current_xai_base_url.is_empty() {
            base_url.clone()
        } else {
            current_xai_base_url.to_owned()
        },
        extra_headers: extra_headers.clone(),
        zdr_video_output_s3: zdr_video_output_s3.clone(),
        tier_restricted: *tier_restricted,
    }
}

async fn video_tool_names(bridge: &crate::tools::bridge::ToolBridge) -> (String, String) {
    use xai_grok_tools::implementations::grok_build::video_gen::{
        IMAGE_TO_VIDEO_TOOL_NAME, REFERENCE_TO_VIDEO_TOOL_NAME,
    };
    use xai_grok_tools::types::tool::ToolKind;
    let image = bridge
        .tool_for_kind(ToolKind::ImageToVideo)
        .await
        .unwrap_or_else(|| IMAGE_TO_VIDEO_TOOL_NAME.to_owned());
    let reference = bridge
        .tool_for_kind(ToolKind::ReferenceToVideo)
        .await
        .unwrap_or_else(|| REFERENCE_TO_VIDEO_TOOL_NAME.to_owned());
    (image, reference)
}

async fn remove_video_tool_definitions(bridge: &crate::tools::bridge::ToolBridge) {
    let (image, reference) = video_tool_names(bridge).await;
    bridge.unregister_tool_by_name(&image);
    bridge.unregister_tool_by_name(&reference);
}

async fn ensure_video_tool_definitions(
    bridge: &crate::tools::bridge::ToolBridge,
) -> Result<(), xai_tool_runtime::ToolError> {
    use xai_grok_tools::implementations::grok_build::video_gen::{
        ImageToVideoTool, ReferenceToVideoTool,
    };

    let (image_name, reference_name) = video_tool_names(bridge).await;
    let added_image = if bridge.tool_kind(&image_name).is_none() {
        bridge
            .register_mcp_tools(image_name.clone(), ImageToVideoTool, None)
            .await?;
        true
    } else {
        false
    };
    if bridge.tool_kind(&reference_name).is_none()
        && let Err(error) = bridge
            .register_mcp_tools(reference_name, ReferenceToVideoTool, None)
            .await
    {
        if added_image {
            bridge.unregister_tool_by_name(&image_name);
        }
        return Err(error);
    }
    Ok(())
}

async fn refresh_video_gen_resource(
    bridge: &crate::tools::bridge::ToolBridge,
    provider: xai_grok_sampling_types::ProviderId,
    configured: &xai_grok_tools::implementations::grok_build::video_gen::VideoGenConfig,
    current_api_key: Option<&str>,
    current_xai_base_url: &str,
    api_key_provider: Option<xai_grok_tools::types::SharedApiKeyProvider>,
    attribution_callback: Option<xai_grok_tools::SharedAttributionCallback>,
) {
    use xai_grok_tools::implementations::grok_build::video_gen::{VideoGenClient, VideoGenConfig};

    let config =
        video_gen_config_for_provider(provider, configured, current_api_key, current_xai_base_url);
    if !matches!(config, VideoGenConfig::Enabled { .. }) {
        remove_video_tool_definitions(bridge).await;
        bridge.remove_resource::<VideoGenClient>().await;
        return;
    }

    match VideoGenClient::new(&config, api_key_provider) {
        Ok(client) => {
            bridge
                .update_resource(client.with_attribution_callback(attribution_callback))
                .await;
            if let Err(error) = ensure_video_tool_definitions(bridge).await {
                tracing::warn!(%error, "failed to restore video tool definitions after provider switch");
                remove_video_tool_definitions(bridge).await;
                bridge.remove_resource::<VideoGenClient>().await;
            }
        }
        Err(error) => {
            tracing::warn!(%error, "failed to restore video generation after provider switch");
            remove_video_tool_definitions(bridge).await;
            bridge.remove_resource::<VideoGenClient>().await;
        }
    }
}

async fn refresh_provider_memory_resource(
    session: &SessionActor,
    bridge: &crate::tools::bridge::ToolBridge,
    sampling_config: &xai_grok_sampler::SamplerConfig,
) {
    let Some(mut params) = session.memory.backend_params.borrow().clone() else {
        return;
    };

    params.embed_config = session.rebuild_spec.memory_embedding_config.clone();
    params.embed_base_url = sampling_config.base_url.clone();
    params.embed_api_key = sampling_config.api_key.clone();
    match sampling_config.provider {
        xai_grok_sampling_types::ProviderId::OpenAiCodex
        | xai_grok_sampling_types::ProviderId::KimiCode
        | xai_grok_sampling_types::ProviderId::ZaiCodingPlan => {
            // Provider-owned subscription auth is not a general xAI
            // credential, and these scoped backends expose no supported
            // embeddings route.
            params.embed_config = None;
            params.embed_base_url.clear();
            params.embed_api_key = None;
            params.api_key_provider = None;
            params.auth_credentials = None;
        }
        xai_grok_sampling_types::ProviderId::Xai => {
            params.api_key_provider = session.auth_manager.as_ref().map(|manager| {
                std::sync::Arc::new(crate::auth::manager::SharedAuthKeyProvider(manager.clone()))
                    as xai_grok_tools::types::SharedApiKeyProvider
            });
            params.auth_credentials = session.auth_manager.as_ref().map(|manager| {
                std::sync::Arc::new(
                    crate::auth::credential_provider::ShellAuthCredentialProvider::new(
                        manager.clone(),
                        None,
                        None,
                    ),
                ) as std::sync::Arc<dyn xai_grok_auth::AuthCredentialProvider>
            });
        }
        xai_grok_sampling_types::ProviderId::Custom => {
            // A custom endpoint may use its explicit key, but the xAI
            // AuthManager must never cross that provider boundary.
            params.api_key_provider = None;
            params.auth_credentials = None;
        }
    }

    *session.memory.backend_params.borrow_mut() = Some(params.clone());
    let Some(storage) = session.memory.storage() else {
        return;
    };
    let backend = crate::session::memory::MemoryBackendImpl::from_session_params(storage, &params);
    *session.memory.search_counter.borrow_mut() = Some(backend.search_counter.clone());
    let backend: std::sync::Arc<dyn xai_grok_tools::types::memory_backend::MemoryBackend> =
        std::sync::Arc::new(backend);
    bridge.update_resource(backend).await;
}

fn web_search_config_for_provider(
    configured: &xai_grok_tools::implementations::web_search::WebSearchConfig,
    codex_settings: &xai_grok_tools::implementations::web_search::CodexWebSearchSettings,
    explicitly_disabled: bool,
    configured_provider: Option<xai_grok_sampling_types::ProviderId>,
    sampling_config: &xai_grok_sampler::SamplerConfig,
    session_id: &str,
    alpha_test_key: Option<&str>,
) -> xai_grok_tools::implementations::web_search::WebSearchConfig {
    use xai_grok_tools::implementations::web_search::WebSearchConfig;

    if explicitly_disabled {
        return WebSearchConfig::Disabled;
    }
    if sampling_config.provider.is_openai_codex() {
        if !codex_settings.mode.is_enabled() {
            return WebSearchConfig::Disabled;
        }
        return WebSearchConfig::CodexSubscription {
            base_url: sampling_config.base_url.clone(),
            model: sampling_config.model.clone(),
            session_id: session_id.to_owned(),
            settings: codex_settings.clone(),
        };
    }
    if sampling_config.provider.is_kimi_code() {
        return WebSearchConfig::KimiCode {
            base_url: xai_grok_sampling_types::KIMI_CODE_BASE_URL.to_owned(),
        };
    }
    if sampling_config.provider.is_zai_coding_plan() {
        return WebSearchConfig::ZaiCodingPlan {
            endpoint: xai_grok_sampling_types::ZAI_CODING_PLAN_SEARCH_MCP_URL.to_owned(),
        };
    }

    // Preserve a dedicated provider-owned web-search model when returning to
    // the provider that originally supplied it. Crossing to a different
    // provider must instead use the new provider's explicit sampling route;
    // never infer ownership from a URL or carry endpoint headers across.
    if configured_provider == Some(sampling_config.provider)
        && let WebSearchConfig::Enabled {
            api_key,
            base_url,
            model,
            extra_headers,
            alpha_test_key: configured_alpha_test_key,
        } = configured
    {
        return WebSearchConfig::Enabled {
            api_key: sampling_config
                .api_key
                .clone()
                .unwrap_or_else(|| api_key.clone()),
            base_url: base_url.clone(),
            model: model.clone(),
            extra_headers: extra_headers.clone(),
            alpha_test_key: configured_alpha_test_key
                .clone()
                .or_else(|| alpha_test_key.map(str::to_owned)),
        };
    }

    let Some(api_key) = sampling_config.api_key.clone() else {
        return WebSearchConfig::Disabled;
    };
    WebSearchConfig::Enabled {
        api_key,
        base_url: sampling_config.base_url.clone(),
        model: sampling_config.model.clone(),
        extra_headers: sampling_config.extra_headers.clone(),
        alpha_test_key: alpha_test_key.map(str::to_owned),
    }
}

fn web_fetch_params_for_provider(
    configured: &xai_grok_tools::implementations::grok_build::web_fetch::WebFetchConfig,
    codex_settings: &xai_grok_tools::implementations::web_search::CodexWebSearchSettings,
    provider: xai_grok_sampling_types::ProviderId,
) -> Option<xai_grok_tools::implementations::grok_build::web_fetch::WebFetchParams> {
    use xai_grok_tools::implementations::grok_build::web_fetch::WebFetchConfig;

    if provider.is_openai_codex()
        && !codex_settings.mode.is_enabled()
        && matches!(configured, WebFetchConfig::CodexDefault { .. })
    {
        return None;
    }
    configured
        .params_for_codex_subscription(
            provider.is_openai_codex() || provider.is_kimi_code() || provider.is_zai_coding_plan(),
        )
        .cloned()
}

async fn web_search_tool_name(bridge: &crate::tools::bridge::ToolBridge) -> String {
    use xai_grok_tools::types::tool::ToolKind;
    bridge
        .tool_for_kind(ToolKind::WebSearch)
        .await
        .unwrap_or_else(|| "web_search".to_owned())
}

async fn remove_web_search_tool_definition(bridge: &crate::tools::bridge::ToolBridge) {
    let name = web_search_tool_name(bridge).await;
    bridge.unregister_tool_by_name(&name);
}

async fn install_web_search_tool_definition(
    bridge: &crate::tools::bridge::ToolBridge,
    codex_subscription: bool,
) -> Result<(), xai_tool_runtime::ToolError> {
    use xai_grok_tools::implementations::grok_build::web_search::{
        CodexWebSearchTool, WebSearchTool, provider_input_schema,
    };

    let name = web_search_tool_name(bridge).await;
    bridge.unregister_tool_by_name(&name);
    if codex_subscription {
        bridge
            .register_mcp_tools(name, CodexWebSearchTool, None)
            .await
    } else {
        bridge
            .register_mcp_tools(name, WebSearchTool, Some(provider_input_schema(false)))
            .await
    }
}

async fn web_fetch_tool_name(bridge: &crate::tools::bridge::ToolBridge) -> String {
    use xai_grok_tools::types::tool::ToolKind;
    bridge
        .tool_for_kind(ToolKind::WebFetch)
        .await
        .unwrap_or_else(|| "web_fetch".to_owned())
}

async fn remove_web_fetch_tool_definition(bridge: &crate::tools::bridge::ToolBridge) {
    let name = web_fetch_tool_name(bridge).await;
    bridge.unregister_tool_by_name(&name);
}

async fn ensure_web_fetch_tool_definition(
    bridge: &crate::tools::bridge::ToolBridge,
) -> Result<(), xai_tool_runtime::ToolError> {
    use xai_grok_tools::implementations::grok_build::web_fetch::WebFetchTool;

    let name = web_fetch_tool_name(bridge).await;
    if bridge.tool_kind(&name).is_none() {
        bridge.register_mcp_tools(name, WebFetchTool, None).await?;
    }
    Ok(())
}

fn remove_zread_tool_definitions(bridge: &crate::tools::bridge::ToolBridge) {
    use xai_grok_tools::implementations::grok_build::{
        ZREAD_GET_REPO_STRUCTURE_TOOL_NAME, ZREAD_READ_FILE_TOOL_NAME, ZREAD_SEARCH_DOC_TOOL_NAME,
    };

    for name in [
        ZREAD_SEARCH_DOC_TOOL_NAME,
        ZREAD_GET_REPO_STRUCTURE_TOOL_NAME,
        ZREAD_READ_FILE_TOOL_NAME,
    ] {
        bridge.unregister_tool_by_name(name);
    }
}

async fn ensure_zread_tool_definitions(
    bridge: &crate::tools::bridge::ToolBridge,
) -> Result<(), xai_tool_runtime::ToolError> {
    use xai_grok_tools::implementations::grok_build::{
        ZREAD_GET_REPO_STRUCTURE_TOOL_NAME, ZREAD_READ_FILE_TOOL_NAME, ZREAD_SEARCH_DOC_TOOL_NAME,
        ZreadGetRepoStructureTool, ZreadReadFileTool, ZreadSearchDocTool,
    };

    let added_search = if bridge.tool_kind(ZREAD_SEARCH_DOC_TOOL_NAME).is_none() {
        bridge
            .register_mcp_tools(
                ZREAD_SEARCH_DOC_TOOL_NAME.to_owned(),
                ZreadSearchDocTool,
                None,
            )
            .await?;
        true
    } else {
        false
    };
    let added_structure = if bridge
        .tool_kind(ZREAD_GET_REPO_STRUCTURE_TOOL_NAME)
        .is_none()
    {
        if let Err(error) = bridge
            .register_mcp_tools(
                ZREAD_GET_REPO_STRUCTURE_TOOL_NAME.to_owned(),
                ZreadGetRepoStructureTool,
                None,
            )
            .await
        {
            if added_search {
                bridge.unregister_tool_by_name(ZREAD_SEARCH_DOC_TOOL_NAME);
            }
            return Err(error);
        }
        true
    } else {
        false
    };
    if bridge.tool_kind(ZREAD_READ_FILE_TOOL_NAME).is_none()
        && let Err(error) = bridge
            .register_mcp_tools(
                ZREAD_READ_FILE_TOOL_NAME.to_owned(),
                ZreadReadFileTool,
                None,
            )
            .await
    {
        if added_search {
            bridge.unregister_tool_by_name(ZREAD_SEARCH_DOC_TOOL_NAME);
        }
        if added_structure {
            bridge.unregister_tool_by_name(ZREAD_GET_REPO_STRUCTURE_TOOL_NAME);
        }
        return Err(error);
    }
    Ok(())
}

fn remove_zai_vision_tool_definitions(bridge: &crate::tools::bridge::ToolBridge) {
    use xai_grok_tools::implementations::grok_build::{
        ZAI_VISION_ANALYZE_DATA_TOOL_NAME, ZAI_VISION_ANALYZE_IMAGE_TOOL_NAME,
        ZAI_VISION_ANALYZE_VIDEO_TOOL_NAME, ZAI_VISION_DIAGNOSE_ERROR_TOOL_NAME,
        ZAI_VISION_DOCTOR_TOOL_NAME, ZAI_VISION_EXTRACT_TEXT_TOOL_NAME,
        ZAI_VISION_UI_DIFF_TOOL_NAME, ZAI_VISION_UI_TO_ARTIFACT_TOOL_NAME,
        ZAI_VISION_UNDERSTAND_DIAGRAM_TOOL_NAME,
    };

    for name in [
        ZAI_VISION_DOCTOR_TOOL_NAME,
        ZAI_VISION_UI_TO_ARTIFACT_TOOL_NAME,
        ZAI_VISION_EXTRACT_TEXT_TOOL_NAME,
        ZAI_VISION_DIAGNOSE_ERROR_TOOL_NAME,
        ZAI_VISION_UNDERSTAND_DIAGRAM_TOOL_NAME,
        ZAI_VISION_ANALYZE_DATA_TOOL_NAME,
        ZAI_VISION_UI_DIFF_TOOL_NAME,
        ZAI_VISION_ANALYZE_IMAGE_TOOL_NAME,
        ZAI_VISION_ANALYZE_VIDEO_TOOL_NAME,
    ] {
        bridge.unregister_tool_by_name(name);
    }
}

async fn ensure_zai_vision_tool_definitions(
    bridge: &crate::tools::bridge::ToolBridge,
) -> Result<(), xai_tool_runtime::ToolError> {
    use xai_grok_tools::implementations::grok_build::{
        ZAI_VISION_ANALYZE_DATA_TOOL_NAME, ZAI_VISION_ANALYZE_IMAGE_TOOL_NAME,
        ZAI_VISION_ANALYZE_VIDEO_TOOL_NAME, ZAI_VISION_DIAGNOSE_ERROR_TOOL_NAME,
        ZAI_VISION_DOCTOR_TOOL_NAME, ZAI_VISION_EXTRACT_TEXT_TOOL_NAME,
        ZAI_VISION_UI_DIFF_TOOL_NAME, ZAI_VISION_UI_TO_ARTIFACT_TOOL_NAME,
        ZAI_VISION_UNDERSTAND_DIAGRAM_TOOL_NAME, ZaiVisionAnalyzeDataTool,
        ZaiVisionAnalyzeImageTool, ZaiVisionAnalyzeVideoTool, ZaiVisionDiagnoseErrorTool,
        ZaiVisionDoctorTool, ZaiVisionExtractTextTool, ZaiVisionUiDiffTool,
        ZaiVisionUiToArtifactTool, ZaiVisionUnderstandDiagramTool,
    };

    macro_rules! ensure {
        ($name:expr, $tool:expr) => {
            if bridge.tool_kind($name).is_none() {
                bridge
                    .register_mcp_tools($name.to_owned(), $tool, None)
                    .await?;
            }
        };
    }

    ensure!(ZAI_VISION_DOCTOR_TOOL_NAME, ZaiVisionDoctorTool);
    ensure!(
        ZAI_VISION_UI_TO_ARTIFACT_TOOL_NAME,
        ZaiVisionUiToArtifactTool
    );
    ensure!(ZAI_VISION_EXTRACT_TEXT_TOOL_NAME, ZaiVisionExtractTextTool);
    ensure!(
        ZAI_VISION_DIAGNOSE_ERROR_TOOL_NAME,
        ZaiVisionDiagnoseErrorTool
    );
    ensure!(
        ZAI_VISION_UNDERSTAND_DIAGRAM_TOOL_NAME,
        ZaiVisionUnderstandDiagramTool
    );
    ensure!(ZAI_VISION_ANALYZE_DATA_TOOL_NAME, ZaiVisionAnalyzeDataTool);
    ensure!(ZAI_VISION_UI_DIFF_TOOL_NAME, ZaiVisionUiDiffTool);
    ensure!(
        ZAI_VISION_ANALYZE_IMAGE_TOOL_NAME,
        ZaiVisionAnalyzeImageTool
    );
    ensure!(
        ZAI_VISION_ANALYZE_VIDEO_TOOL_NAME,
        ZaiVisionAnalyzeVideoTool
    );
    Ok(())
}

impl SessionActor {
    pub(super) async fn handle_set_session_model(
        &self,
        mut sampling_config: xai_grok_sampler::SamplerConfig,
        use_concise: bool,
        apply_prompt_override: bool,
        skip_prompt_rewrite: bool,
        auto_compact_threshold_percent: u8,
    ) -> Result<acp::ModelId, acp::Error> {
        let previous_sampling_config = self.chat_state_handle.get_sampling_config().await;
        if sampling_config.provider == xai_grok_sampling_types::ProviderId::OpenAiCodex
            && previous_sampling_config.as_ref().is_some_and(|previous| {
                previous.provider == xai_grok_sampling_types::ProviderId::OpenAiCodex
            })
        {
            if let Some(previous_tier) = previous_sampling_config
                .as_ref()
                .and_then(|previous| previous.service_tier.clone())
            {
                // Preserve the user's session selection across Codex model
                // switches. Validation is intentionally deferred to the per-turn
                // wire boundary so switching through a model without Fast support
                // does not erase the preference before switching back.
                sampling_config.service_tier = Some(previous_tier);
            }
        }
        // A same-provider Codex/Kimi switch belongs to the existing session
        // record. Model resolution must not silently adopt a process-current
        // account or Kimi API-key record.
        if let Some(previous) = previous_sampling_config.as_ref()
            && previous.provider == sampling_config.provider
            && (sampling_config.provider.is_openai_codex()
                || sampling_config.provider.is_kimi_code()
                || sampling_config.provider.is_zai_coding_plan())
        {
            crate::session::provider::pin_provider_candidate_to_active_record(
                &mut sampling_config,
                previous.provider,
                previous.credential_binding.as_ref(),
            );
        }
        let generic_api_key_provider = (sampling_config.provider
            == xai_grok_sampling_types::ProviderId::Xai)
            .then(|| {
                self.auth_manager.as_ref().map(|manager| {
                    std::sync::Arc::new(crate::auth::manager::SharedAuthKeyProvider(
                        manager.clone(),
                    )) as xai_grok_tools::types::SharedApiKeyProvider
                })
            })
            .flatten();
        let bound_runtime = crate::session::provider::bind_provider_runtime(
            sampling_config,
            generic_api_key_provider,
        )
        .await
        .map_err(|error| acp::Error::auth_required().data(error.to_string()))?;
        let sampling_config = bound_runtime.sampler_config;
        let provider_api_key_provider = bound_runtime.api_key_provider;

        let model_id = match sampling_config.provider {
            xai_grok_sampling_types::ProviderId::OpenAiCodex => {
                acp::ModelId::new(format!("openai-codex/{}", sampling_config.model))
            }
            xai_grok_sampling_types::ProviderId::KimiCode => {
                acp::ModelId::new(format!("kimi-code/{}", sampling_config.model))
            }
            xai_grok_sampling_types::ProviderId::ZaiCodingPlan => {
                acp::ModelId::new(format!("zai-coding-plan/{}", sampling_config.model))
            }
            xai_grok_sampling_types::ProviderId::Xai
            | xai_grok_sampling_types::ProviderId::Custom => {
                acp::ModelId::new(sampling_config.model.clone())
            }
        };
        let new_context_window = self.compaction.context_window_override.unwrap_or_else(|| {
            std::num::NonZeroU64::new(sampling_config.context_window).unwrap_or_else(|| {
                std::num::NonZeroU64::new(DEFAULT_CONTEXT_WINDOW)
                    .expect("DEFAULT_CONTEXT_WINDOW is non-zero")
            })
        });
        let prev_threshold = self.compaction.threshold_percent.get();
        if prev_threshold != auto_compact_threshold_percent {
            tracing::info!(
                session_id = % self.session_info.id.0, new_model = % sampling_config
                .model, old_threshold = prev_threshold, new_threshold =
                auto_compact_threshold_percent,
                "auto_compact_threshold_percent updated for model switch"
            );
        }
        self.compaction
            .threshold_percent
            .set(auto_compact_threshold_percent);
        self.supports_backend_search
            .set(sampling_config.supports_backend_search);
        self.compactions_remaining
            .set(sampling_config.compactions_remaining);
        self.compaction_at_tokens
            .set(sampling_config.compaction_at_tokens);
        xai_grok_telemetry::unified_log::info(
            "backend_search: model switch",
            Some(self.session_info.id.0.as_ref()),
            Some(serde_json::json!(
                { "new_model" : & sampling_config.model, "api_backend" :
                format!("{:?}", sampling_config.api_backend),
                "supports_backend_search" : sampling_config.supports_backend_search,
                }
            )),
        );
        let previous_provider = previous_sampling_config.map(|config| config.provider);
        if previous_provider.is_some_and(|provider| provider != sampling_config.provider) {
            let conversation = self.chat_state_handle.get_conversation().await;
            let (portable_conversation, removed) =
                xai_grok_sampling_types::conversation::strip_provider_bound_response_items(
                    conversation,
                );
            if removed > 0 {
                self.chat_state_handle
                    .replace_conversation(portable_conversation.clone());
                // Keep durable replay aligned with the in-memory prefix; a
                // restart after the switch must not resurrect opaque state
                // belonging to the old provider.
                persist_chat_history_jsonl_sync(&self.session_info, &portable_conversation);
                let _ = self
                    .notifications
                    .persistence_tx
                    .send(PersistenceMsg::ReplaceChatHistory(portable_conversation));
                tracing::info!(
                    session_id = % self.session_info.id.0,
                    removed_provider_bound_items = removed,
                    "removed provider-bound response state before provider switch"
                );
            }
        }
        self.chat_state_handle
            .update_sampling_config(xai_grok_sampling_types::SamplingConfig {
                provider: sampling_config.provider,
                credential_binding: sampling_config.credential_binding.clone(),
                base_url: sampling_config.base_url.clone(),
                model: sampling_config.model.clone(),
                max_completion_tokens: sampling_config.max_completion_tokens,
                temperature: sampling_config.temperature,
                top_p: sampling_config.top_p,
                api_backend: sampling_config.api_backend.clone(),
                extra_headers: sampling_config.extra_headers.clone(),
                comp_hash: sampling_config.comp_hash.clone(),
                context_window: new_context_window,
                reasoning_effort: sampling_config.reasoning_effort,
                supports_reasoning_summary_parameter: sampling_config
                    .supports_reasoning_summary_parameter,
                default_reasoning_summary: sampling_config.default_reasoning_summary.clone(),
                service_tier: sampling_config.service_tier.clone(),
                stream_tool_calls: Some(sampling_config.stream_tool_calls),
            });
        let existing = self.chat_state_handle.get_credentials().await;
        let alpha_test_key = existing.alpha_test_key.clone();
        let session_key = self
            .auth_manager
            .as_ref()
            .and_then(|am| am.current_or_expired().map(|a| a.key));
        self.chat_state_handle
            .update_credentials(xai_chat_state::Credentials {
                provider: Some(sampling_config.provider),
                api_key: sampling_config.api_key.clone(),
                auth_type: crate::agent::config::resolve_chat_state_auth_type(
                    sampling_config.model.as_str(),
                    session_key.as_deref(),
                    existing.auth_type,
                ),
                alpha_test_key: existing.alpha_test_key,
                client_version: sampling_config.client_version.clone(),
            });
        self.refresh_provider_media_resources(&sampling_config, provider_api_key_provider.clone())
            .await;
        self.refresh_provider_web_resources(
            &sampling_config,
            alpha_test_key.as_deref(),
            provider_api_key_provider,
        )
        .await;
        let bridge = self.agent.borrow().tool_bridge().clone();
        refresh_provider_memory_resource(self, &bridge, &sampling_config).await;
        self.invalidate_model_auth_memo();
        self.signals_handle()
            .record_model_usage(&sampling_config.model);
        if apply_prompt_override && !skip_prompt_rewrite {
            let mut conversation = self.chat_state_handle.get_conversation().await;
            for item in conversation.iter_mut() {
                if let ConversationItem::System(sys) = item {
                    if use_concise {
                        sys.content = std::sync::Arc::<str>::from(
                            xai_grok_agent::prompt::template::COMPACT_SYSTEM_PROMPT,
                        );
                    } else {
                        sys.content =
                            std::sync::Arc::<str>::from(self.agent.borrow().system_prompt());
                    }
                    break;
                }
            }
            self.chat_state_handle.replace_conversation(conversation);
        } else if !apply_prompt_override {
            tracing::info!(
                session_id = % self.session_info.id.0, model_id = % model_id.0,
                "handle_set_session_model: skipping prompt override (apply_prompt_override=false)"
            );
        } else {
            tracing::info!(
                session_id = % self.session_info.id.0, model_id = % model_id.0,
                "handle_set_session_model: skipping prompt rewrite (just rebuilt harness)"
            );
        }
        let agent_name = self.agent.borrow().definition().name.clone();
        let persisted_binding = (sampling_config.provider.is_openai_codex()
            || sampling_config.provider.is_kimi_code()
            || sampling_config.provider.is_zai_coding_plan())
        .then(|| sampling_config.credential_binding.clone())
        .flatten();
        let _ = self
            .notifications
            .persistence_tx
            .send(PersistenceMsg::CurrentModel {
                model_id: model_id.clone(),
                agent_name: Some(agent_name),
                reasoning_effort: Some(sampling_config.reasoning_effort),
                comp_hash: Some(sampling_config.comp_hash.clone()),
                credential_binding: persisted_binding,
            });
        Ok(model_id)
    }

    async fn refresh_provider_web_resources(
        &self,
        sampling_config: &xai_grok_sampler::SamplerConfig,
        alpha_test_key: Option<&str>,
        api_key_provider: Option<xai_grok_tools::types::SharedApiKeyProvider>,
    ) {
        use xai_grok_tools::implementations::grok_build::web_fetch::{
            WebFetchClient, WebFetchParams,
        };
        use xai_grok_tools::implementations::web_search::{
            WebSearchConfig, client::WebSearchClient,
        };

        let bridge = self.agent.borrow().tool_bridge().clone();
        let mut web_search_config = web_search_config_for_provider(
            &self.rebuild_spec.web_search_config,
            &self.rebuild_spec.codex_web_search_settings,
            self.rebuild_spec.web_search_disabled,
            self.rebuild_spec.web_search_provider,
            sampling_config,
            self.session_info.id.0.as_ref(),
            alpha_test_key,
        );
        if (sampling_config.provider.is_openai_codex()
            || sampling_config.provider.is_kimi_code()
            || sampling_config.provider.is_zai_coding_plan())
            && api_key_provider.is_none()
        {
            tracing::warn!(
                provider = %sampling_config.provider,
                "web_search disabled after provider switch: scoped request authentication is unavailable"
            );
            web_search_config = WebSearchConfig::Disabled;
        }

        let codex_subscription_search = web_search_config.is_codex_subscription();
        let hosted_web_search_enabled = web_search_config.allows_hosted_responses_tool();
        let backend_search_enabled = self.rebuild_spec.backend_search
            && !sampling_config.provider.is_openai_codex()
            && !sampling_config.provider.is_kimi_code()
            && !sampling_config.provider.is_zai_coding_plan();
        self.agent.borrow_mut().refresh_backend_search_config(
            backend_search_enabled,
            hosted_web_search_enabled,
            codex_subscription_search,
        );

        if web_search_config.is_enabled() {
            let attribution_callback =
                if sampling_config.provider == xai_grok_sampling_types::ProviderId::Xai {
                    self.rebuild_spec.attribution_callback.clone()
                } else {
                    None
                };
            match WebSearchClient::new(&web_search_config, api_key_provider.clone()) {
                Ok(client) => {
                    bridge
                        .update_resource(client.with_attribution_callback(attribution_callback))
                        .await;
                    if let Err(error) =
                        install_web_search_tool_definition(&bridge, codex_subscription_search).await
                    {
                        tracing::warn!(
                            %error,
                            "failed to install web search tool after provider switch"
                        );
                        remove_web_search_tool_definition(&bridge).await;
                        bridge.remove_resource::<WebSearchClient>().await;
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        %error,
                        "failed to rebuild web search client after provider switch"
                    );
                    remove_web_search_tool_definition(&bridge).await;
                    bridge.remove_resource::<WebSearchClient>().await;
                }
            }
        } else {
            remove_web_search_tool_definition(&bridge).await;
            bridge.remove_resource::<WebSearchClient>().await;
        }

        use xai_grok_tools::implementations::grok_build::ZaiZreadClient;
        if sampling_config.provider.is_zai_coding_plan() {
            match api_key_provider
                .clone()
                .ok_or_else(|| {
                    xai_tool_runtime::ToolError::custom(
                        "zai_zread_authentication",
                        "Z.AI Coding Plan authentication is required for Zread",
                    )
                })
                .and_then(ZaiZreadClient::new)
            {
                Ok(client) => {
                    bridge.update_resource(client).await;
                    if let Err(error) = ensure_zread_tool_definitions(&bridge).await {
                        tracing::warn!(
                            %error,
                            "failed to install Z.AI Coding Plan Zread tools after provider switch"
                        );
                        remove_zread_tool_definitions(&bridge);
                        bridge.remove_resource::<ZaiZreadClient>().await;
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        %error,
                        "failed to bind Z.AI Coding Plan Zread after provider switch"
                    );
                    remove_zread_tool_definitions(&bridge);
                    bridge.remove_resource::<ZaiZreadClient>().await;
                }
            }
        } else {
            remove_zread_tool_definitions(&bridge);
            bridge.remove_resource::<ZaiZreadClient>().await;
        }

        use xai_grok_tools::implementations::grok_build::{
            ZaiVisionClient, zai_vision_mcp_enabled,
        };
        if sampling_config.provider.is_zai_coding_plan() && zai_vision_mcp_enabled() {
            match api_key_provider
                .clone()
                .ok_or_else(|| {
                    xai_tool_runtime::ToolError::custom(
                        "zai_vision_authentication",
                        "Z.AI Coding Plan authentication is required for Vision MCP",
                    )
                })
                .and_then(ZaiVisionClient::new)
            {
                Ok(client) => {
                    bridge.update_resource(client).await;
                    if let Err(error) = ensure_zai_vision_tool_definitions(&bridge).await {
                        tracing::warn!(
                            %error,
                            "failed to install Z.AI Vision MCP tools after provider switch"
                        );
                        remove_zai_vision_tool_definitions(&bridge);
                        bridge.remove_resource::<ZaiVisionClient>().await;
                    }
                }
                Err(error) => {
                    tracing::warn!(%error, "failed to bind Z.AI Vision MCP after provider switch");
                    remove_zai_vision_tool_definitions(&bridge);
                    bridge.remove_resource::<ZaiVisionClient>().await;
                }
            }
        } else {
            remove_zai_vision_tool_definitions(&bridge);
            bridge.remove_resource::<ZaiVisionClient>().await;
        }

        let fetch_params: Option<WebFetchParams> = web_fetch_params_for_provider(
            &self.rebuild_spec.web_fetch_config,
            &self.rebuild_spec.codex_web_search_settings,
            sampling_config.provider,
        );
        self.permissions.set_web_fetch_allowed_domains(
            fetch_params
                .as_ref()
                .map(WebFetchParams::allowed_domains)
                .unwrap_or_default(),
        );
        if let Some(params) = fetch_params {
            match WebFetchClient::new(&params) {
                Ok(client) => {
                    let client = if sampling_config.provider.is_kimi_code() {
                        api_key_provider
                            .clone()
                            .ok_or(xai_grok_tools::implementations::grok_build::web_fetch::WebFetchError::HostedAuthentication)
                            .and_then(|provider| client.with_kimi_hosted_fetch(provider))
                    } else if sampling_config.provider.is_zai_coding_plan() {
                        api_key_provider
                            .clone()
                            .ok_or(xai_grok_tools::implementations::grok_build::web_fetch::WebFetchError::ZaiReaderAuthentication)
                            .and_then(|provider| client.with_zai_coding_plan_reader(provider))
                    } else {
                        Ok(client)
                    };
                    let Ok(client) = client else {
                        tracing::warn!(
                            provider = %sampling_config.provider,
                            "failed to bind provider web-fetch authentication after model switch"
                        );
                        remove_web_fetch_tool_definition(&bridge).await;
                        bridge.remove_resource::<WebFetchClient>().await;
                        return;
                    };
                    bridge.update_resource(client).await;
                    if let Err(error) = ensure_web_fetch_tool_definition(&bridge).await {
                        tracing::warn!(
                            %error,
                            "failed to install web fetch tool after provider switch"
                        );
                        remove_web_fetch_tool_definition(&bridge).await;
                        bridge.remove_resource::<WebFetchClient>().await;
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        %error,
                        "failed to rebuild web fetch client after provider switch"
                    );
                    remove_web_fetch_tool_definition(&bridge).await;
                    bridge.remove_resource::<WebFetchClient>().await;
                }
            }
        } else {
            remove_web_fetch_tool_definition(&bridge).await;
            bridge.remove_resource::<WebFetchClient>().await;
        }
    }

    async fn refresh_provider_media_resources(
        &self,
        sampling_config: &xai_grok_sampler::SamplerConfig,
        api_key_provider: Option<xai_grok_tools::types::SharedApiKeyProvider>,
    ) {
        use xai_grok_tools::implementations::grok_build::image_gen::ImageGenConfig;

        let bridge = self.agent.borrow().tool_bridge().clone();
        let endpoints = self.models_manager.endpoints();
        if sampling_config.provider == xai_grok_sampling_types::ProviderId::OpenAiCodex {
            let (image_gen_enabled, image_edit_enabled) = self.models_manager.image_tool_gates();
            // Image generation/editing uses the standalone Codex image model;
            // selected-model image-input support remains an input/read concern.
            // Reuse the binder-owned provider so sampler and tools cannot
            // attest different credential records.
            let config =
                (image_gen_enabled || image_edit_enabled).then(|| ImageGenConfig::OpenAiCodex {
                    base_url: xai_grok_sampling_types::OPENAI_CODEX_BASE_URL.to_owned(),
                    image_gen_enabled,
                    image_edit_enabled,
                });
            refresh_image_gen_resource(&bridge, config, api_key_provider, None).await;
            refresh_video_gen_resource(
                &bridge,
                sampling_config.provider,
                &self.rebuild_spec.video_gen_config,
                sampling_config.api_key.as_deref(),
                &endpoints.xai_api_base_url,
                None,
                None,
            )
            .await;
            return;
        }
        if sampling_config.provider == xai_grok_sampling_types::ProviderId::Custom {
            // Custom inference providers do not implicitly own xAI media
            // resources. Remove both clients and their advertised tools
            // instead of reusing a custom key for xAI-owned services.
            refresh_image_gen_resource(&bridge, None, None, None).await;
            refresh_video_gen_resource(
                &bridge,
                sampling_config.provider,
                &self.rebuild_spec.video_gen_config,
                sampling_config.api_key.as_deref(),
                &endpoints.xai_api_base_url,
                None,
                None,
            )
            .await;
            return;
        }

        let provider = api_key_provider;
        let (image_gen_enabled, image_edit_enabled) = self.models_manager.image_tool_gates();
        let config = if let Some(api_key) = sampling_config.api_key.clone()
            && (image_gen_enabled || image_edit_enabled)
        {
            Some(
                xai_image_gen_config_with_rotated_key(
                    &self.rebuild_spec.image_gen_config,
                    Some(&api_key),
                )
                .unwrap_or_else(|| {
                    // A session that began on Codex has no xAI image recipe in
                    // its rebuild spec. Construct the safe first xAI config from
                    // the provider endpoint and current feature gates. xAI-origin
                    // sessions take the lossless branch above.
                    let mut extra_headers = indexmap::IndexMap::new();
                    extra_headers.insert(
                        "user-agent".to_owned(),
                        format!("xai-grok-build/{}", xai_grok_version::VERSION),
                    );
                    ImageGenConfig::Enabled {
                        api_key,
                        base_url: endpoints.xai_api_base_url.clone(),
                        extra_headers,
                        image_gen_enabled,
                        image_edit_enabled,
                        model_override: self
                            .rebuild_spec
                            .image_gen_config
                            .model_override()
                            .map(ToOwned::to_owned),
                        tier_restricted: false,
                    }
                }),
            )
        } else {
            None
        };
        refresh_image_gen_resource(
            &bridge,
            config,
            provider.clone(),
            self.rebuild_spec.attribution_callback.clone(),
        )
        .await;

        // The Codex leg removes the xAI-only video client. Rebuild it from the
        // session's original xAI media configuration when switching back,
        // preserving ZDR and tier restrictions while using the current key.
        refresh_video_gen_resource(
            &bridge,
            sampling_config.provider,
            &self.rebuild_spec.video_gen_config,
            sampling_config.api_key.as_deref(),
            &endpoints.xai_api_base_url,
            provider,
            self.rebuild_spec.attribution_callback.clone(),
        )
        .await;
    }
    /// Handle [`SessionCommand::RebuildAgentForDefinition`].
    ///
    /// Builds a fresh [`xai_grok_agent::Agent`] from the cached
    /// [`crate::session::agent_rebuild::AgentRebuildSpec`] + the supplied
    /// [`xai_grok_agent::AgentDefinition`], replaces `self.agent`,
    /// rewrites the system message in the conversation, persists the
    /// new prompt artifacts, and updates `active_agent_type`.
    ///
    /// Triggered from `MvpAgent::set_session_model` only when the new
    /// model's `agent_type` differs from the session's current
    /// `active_agent_type` AND `turn_count == 0` (no user message has
    /// been sent yet). Defense-in-depth: rejects if a turn is in flight.
    pub(super) async fn handle_rebuild_agent_for_definition(
        &self,
        definition: xai_grok_agent::AgentDefinition,
    ) -> Result<(), acp::Error> {
        {
            let state = self.state.lock().await;
            if state.running_task.is_some() {
                tracing::warn!(
                    session_id = % self.session_info.id.0, new_agent_type = % definition
                    .name,
                    "handle_rebuild_agent_for_definition: turn in flight, rejecting rebuild"
                );
                return Err(acp::Error::internal_error()
                    .data("rebuild_agent: turn in flight, refusing to rebuild harness"));
            }
        }
        let new_agent_name = definition.name.clone();
        tracing::info!(
            session_id = % self.session_info.id.0, new_agent_type = % new_agent_name,
            "handle_rebuild_agent_for_definition: rebuilding harness"
        );
        let new_agent = self
            .rebuild_spec
            .build_agent(definition)
            .await
            .map_err(|e| {
                tracing::error!(
                    session_id = % self.session_info.id.0, new_agent_type = %
                    new_agent_name, error = % e,
                    "handle_rebuild_agent_for_definition: AgentBuilder::build failed"
                );
                acp::Error::internal_error().data(format!(
                    "rebuild_agent: build failed for agent_type={new_agent_name}: {e}"
                ))
            })?;
        let new_system_prompt = new_agent.system_prompt().to_string();
        let mut new_prompt_context = new_agent.prompt_context().clone();
        new_prompt_context.normalize_for_persistence();
        if let Some(handle) = self.compaction.prefire.take_handle() {
            handle.abort();
            let _ = handle.await;
            self.compaction.prefire.finish();
        }
        self.compaction.prefire.clear();
        *self.agent.borrow_mut() = new_agent;
        *self.active_agent_type.lock() = Some(new_agent_name.clone());
        self.queue_exit_reminder_on_approved_exit.store(
            self.is_cursor_harness(),
            std::sync::atomic::Ordering::Relaxed,
        );
        if let Err(e) = self.workspace_ops.bind_local_session(
            &self.session_id_string(),
            self.tool_context.cwd.as_path().to_path_buf(),
            self.tool_context.hunk_tracker_handle.clone(),
            self.agent.borrow().tool_bridge().toolset(),
            None,
        ) {
            tracing::warn!(
                error = % e, "failed to rebind local session toolset after agent rebuild"
            );
        }
        {
            let bridge = self.agent.borrow().tool_bridge().clone();
            let snapshot = self.tool_metadata_snapshot.clone();
            let tool_index = crate::session::tool_index::Bm25ToolSearchIndex::new(snapshot);
            bridge
                .update_resource(xai_grok_tools::types::tool_index::ToolIndex(
                    std::sync::Arc::new(tool_index),
                ))
                .await;
            if let Some(client) = self.rebuild_spec.managed_gateway_tool_client.clone() {
                bridge.update_resource(client).await;
            }
            let plan_path = self.plan_mode.lock().plan_file_path().to_path_buf();
            bridge
                .update_resource(xai_grok_tools::types::resources::PlanFilePath(plan_path))
                .await;
            if let Some(display_cwd) = self.display_cwd.get() {
                bridge
                    .set_display_cwd(std::path::PathBuf::from(display_cwd))
                    .await;
            }
            bridge
                .update_resource(
                    xai_grok_tools::implementations::grok_build::update_goal::GoalUpdateHandle(
                        self.goal_update_tx.clone(),
                    ),
                )
                .await;
            self.inject_deny_read_globs().await;
        }
        {
            let notified = self.mcp_handshakes_done.notified();
            tokio::pin!(notified);
            let needs_wait = {
                let s = self.mcp_state.lock().await;
                !s.configs.is_empty() && !s.is_initialized()
            };
            if needs_wait {
                const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
                tokio::select! {
                    () = & mut notified => {} () = tokio::time::sleep(TIMEOUT) => {
                    tracing::warn!(session_id = % self.session_info.id.0,
                    "handle_rebuild_agent_for_definition: timed out waiting for MCP handshakes");
                    }
                }
            }
        }
        self.re_register_mcp_tools_on_rebuilt_bridge().await;
        if let Some(old_handle) = self.deferred_prefix.take() {
            old_handle.abort();
        }
        let new_user_prefix = self.build_user_message_prefix().await;
        {
            let mut conversation = self.chat_state_handle.get_conversation().await;
            let _ = replace_or_insert_system_head(&mut conversation, &new_system_prompt);
            let drop_startup_skill_reminder = false;
            Self::rewrite_zero_turn_prefix(
                &mut conversation,
                new_user_prefix,
                drop_startup_skill_reminder,
            );
            if !conversation_has_project_instructions(&conversation)
                && let Some(agents_md_reminder) = self.agent.borrow().agents_md_user_reminder()
            {
                let agents_md_at = conversation.len().min(2);
                conversation.insert(
                    agents_md_at,
                    ConversationItem::project_instructions(agents_md_reminder),
                );
            }
            self.inject_baseline_skill_reminder(&mut conversation).await;
            self.chat_state_handle.replace_conversation(conversation);
        }
        save_prompt_context(&self.session_info, &new_prompt_context);
        save_system_prompt(&self.session_info, &new_system_prompt);
        let snapshot = self.chat_state_handle.get_conversation().await;
        persist_chat_history_jsonl_sync(&self.session_info, &snapshot);
        self.mcp_reminder_dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.send_available_commands_update().await;
        tracing::info!(
            session_id = % self.session_info.id.0, new_agent_type = % new_agent_name,
            "handle_rebuild_agent_for_definition: harness rebuild complete"
        );
        Ok(())
    }
    /// Apply a client-supplied `systemPromptOverride` on session attach without
    /// wiping user/assistant history: swap only the leading `System` message,
    /// atomically inside the `ChatStateActor` (see
    /// `ChatStateCommand::ReplaceSystemHead` for the serialization guarantees).
    /// `system_prompt.txt` (not owned by the persistence actor) is saved
    /// directly, even on a head no-op, so a previously-diverged secondary
    /// artifact self-heals. Skipped entirely on a verbatim mirror-fork
    /// (`preserve_inherited_system`).
    pub(super) async fn handle_replace_system_prompt(&self, system_prompt: String) {
        if self.startup_hints.preserve_inherited_system {
            tracing::debug!(
                session_id = % self.session_info.id.0,
                "handle_replace_system_prompt: skipped (preserve_inherited_system)"
            );
            return;
        }
        let Some(changed) = self
            .chat_state_handle
            .replace_system_head(&system_prompt)
            .await
        else {
            tracing::error!(
                session_id = % self.session_info.id.0,
                "handle_replace_system_prompt: chat-state actor unavailable; override not applied"
            );
            return;
        };
        save_system_prompt(&self.session_info, &system_prompt);
        if changed {
            tracing::info!(
                session_id = % self.session_info.id.0, prompt_len = system_prompt.len(),
                "handle_replace_system_prompt: client override applied"
            );
        } else {
            tracing::debug!(
                session_id = % self.session_info.id.0,
                "handle_replace_system_prompt: head already matches, no-op"
            );
        }
    }
}

#[cfg(test)]
mod provider_media_switch_tests {
    use super::*;
    use xai_grok_tools::implementations::grok_build::image_gen::{ImageGenClient, ImageGenConfig};
    use xai_grok_tools::implementations::grok_build::video_gen::{VideoGenClient, VideoGenConfig};
    use xai_grok_tools::implementations::grok_build::web_fetch::{WebFetchConfig, WebFetchParams};
    use xai_grok_tools::implementations::web_search::{
        CodexWebSearchMode, CodexWebSearchSettings, WebSearchConfig,
    };

    fn enabled_xai_web_search_config() -> WebSearchConfig {
        WebSearchConfig::Enabled {
            api_key: "old-xai-search-key".to_owned(),
            base_url: "https://search.x.ai/v1".to_owned(),
            model: "xai-dedicated-search".to_owned(),
            extra_headers: indexmap::indexmap! {
                "x-xai-route".to_owned() => "search".to_owned(),
            },
            alpha_test_key: Some("old-alpha".to_owned()),
        }
    }

    #[test]
    fn codex_web_search_route_uses_current_raw_model_and_session_identity() {
        let sampling = xai_grok_sampler::SamplerConfig::openai_codex("gpt-5.6-luna");
        let config = web_search_config_for_provider(
            &enabled_xai_web_search_config(),
            &Default::default(),
            false,
            Some(xai_grok_sampling_types::ProviderId::Xai),
            &sampling,
            "session-public-id",
            Some("must-not-cross"),
        );
        assert!(matches!(
            config,
            WebSearchConfig::CodexSubscription {
                ref base_url,
                ref model,
                ref session_id,
                ..
            } if base_url == xai_grok_sampling_types::OPENAI_CODEX_BASE_URL
                && model == "gpt-5.6-luna"
                && session_id == "session-public-id"
        ));
    }

    #[test]
    fn kimi_hosted_search_is_enabled_independently_of_codex_search_mode() {
        let sampling = xai_grok_sampler::SamplerConfig::kimi_code(
            "k3",
            xai_grok_sampling_types::ApiBackend::ChatCompletions,
        );
        let disabled_codex = CodexWebSearchSettings {
            mode: CodexWebSearchMode::Disabled,
            ..Default::default()
        };

        let config = web_search_config_for_provider(
            &WebSearchConfig::Disabled,
            &disabled_codex,
            false,
            None,
            &sampling,
            "session-public-id",
            None,
        );

        assert!(matches!(
            config,
            WebSearchConfig::KimiCode { ref base_url }
                if base_url == xai_grok_sampling_types::KIMI_CODE_BASE_URL
        ));
    }

    #[test]
    fn zai_search_route_is_provider_owned_and_independent_of_codex_mode() {
        let sampling = xai_grok_sampler::SamplerConfig::zai_coding_plan("glm-5.2");
        let disabled_codex = CodexWebSearchSettings {
            mode: CodexWebSearchMode::Disabled,
            ..Default::default()
        };

        let config = web_search_config_for_provider(
            &WebSearchConfig::Disabled,
            &disabled_codex,
            false,
            None,
            &sampling,
            "session-public-id",
            None,
        );

        assert!(matches!(
            config,
            WebSearchConfig::ZaiCodingPlan { ref endpoint }
                if endpoint == xai_grok_sampling_types::ZAI_CODING_PLAN_SEARCH_MCP_URL
        ));
    }

    #[test]
    fn unavailable_xai_search_does_not_block_later_codex_subscription_search() {
        let sampling = xai_grok_sampler::SamplerConfig::openai_codex("gpt-5.6-luna");
        let config = web_search_config_for_provider(
            &WebSearchConfig::Disabled,
            &Default::default(),
            false,
            None,
            &sampling,
            "session-public-id",
            None,
        );
        assert!(matches!(
            config,
            WebSearchConfig::CodexSubscription { ref model, .. }
                if model == "gpt-5.6-luna"
        ));
    }

    #[test]
    fn explicit_web_search_kill_switch_survives_provider_switch() {
        let sampling = xai_grok_sampler::SamplerConfig::openai_codex("gpt-5.6-luna");
        assert!(matches!(
            web_search_config_for_provider(
                &enabled_xai_web_search_config(),
                &Default::default(),
                true,
                Some(xai_grok_sampling_types::ProviderId::Xai),
                &sampling,
                "session-public-id",
                None,
            ),
            WebSearchConfig::Disabled
        ));
    }

    #[test]
    fn codex_disabled_search_does_not_survive_switches_in_either_direction() {
        let disabled = CodexWebSearchSettings {
            mode: CodexWebSearchMode::Disabled,
            ..Default::default()
        };
        let codex = xai_grok_sampler::SamplerConfig::openai_codex("gpt-5.6-luna");
        assert!(matches!(
            web_search_config_for_provider(
                &enabled_xai_web_search_config(),
                &disabled,
                false,
                Some(xai_grok_sampling_types::ProviderId::Xai),
                &codex,
                "session-public-id",
                None,
            ),
            WebSearchConfig::Disabled
        ));

        let mut xai = xai_grok_sampler::SamplerConfig::default();
        xai.provider = xai_grok_sampling_types::ProviderId::Xai;
        xai.api_key = Some("rotated-xai-key".to_owned());
        assert!(matches!(
            web_search_config_for_provider(
                &enabled_xai_web_search_config(),
                &disabled,
                false,
                Some(xai_grok_sampling_types::ProviderId::Xai),
                &xai,
                "session-public-id",
                None,
            ),
            WebSearchConfig::Enabled { .. }
        ));
    }

    #[test]
    fn implicit_fetch_tracks_codex_mode_but_explicit_fetch_survives_switches() {
        let implicit = WebFetchConfig::CodexDefault {
            params: WebFetchParams::default(),
        };
        let explicit = WebFetchConfig::Enabled {
            params: WebFetchParams::default(),
        };
        let cached = CodexWebSearchSettings::default();
        let disabled = CodexWebSearchSettings {
            mode: CodexWebSearchMode::Disabled,
            ..Default::default()
        };

        assert!(
            web_fetch_params_for_provider(
                &implicit,
                &cached,
                xai_grok_sampling_types::ProviderId::OpenAiCodex,
            )
            .is_some()
        );
        assert!(
            web_fetch_params_for_provider(
                &implicit,
                &disabled,
                xai_grok_sampling_types::ProviderId::OpenAiCodex,
            )
            .is_none()
        );
        assert!(
            web_fetch_params_for_provider(
                &implicit,
                &cached,
                xai_grok_sampling_types::ProviderId::Xai,
            )
            .is_none()
        );
        for provider in [
            xai_grok_sampling_types::ProviderId::Xai,
            xai_grok_sampling_types::ProviderId::OpenAiCodex,
        ] {
            assert!(
                web_fetch_params_for_provider(&explicit, &disabled, provider).is_some(),
                "explicit provider-independent fetch must survive {provider:?}"
            );
        }
    }

    #[test]
    fn implicit_fetch_remains_enabled_for_kimi_and_zai_when_codex_search_is_disabled() {
        let implicit = WebFetchConfig::CodexDefault {
            params: WebFetchParams::default(),
        };
        let disabled_codex = CodexWebSearchSettings {
            mode: CodexWebSearchMode::Disabled,
            ..Default::default()
        };

        for provider in [
            xai_grok_sampling_types::ProviderId::KimiCode,
            xai_grok_sampling_types::ProviderId::ZaiCodingPlan,
        ] {
            assert!(
                web_fetch_params_for_provider(&implicit, &disabled_codex, provider).is_some(),
                "provider-scoped Reader must remain available for {provider:?}"
            );
        }
    }

    #[test]
    fn cross_provider_web_search_drops_old_endpoint_headers_and_credentials() {
        let mut sampling = xai_grok_sampler::SamplerConfig::default();
        sampling.provider = xai_grok_sampling_types::ProviderId::Custom;
        sampling.api_key = Some("custom-key".to_owned());
        sampling.base_url = "https://custom.example/v1".to_owned();
        sampling.model = "custom-search-model".to_owned();
        sampling.extra_headers = indexmap::indexmap! {
            "x-custom-route".to_owned() => "green".to_owned(),
        };

        let config = web_search_config_for_provider(
            &enabled_xai_web_search_config(),
            &Default::default(),
            false,
            Some(xai_grok_sampling_types::ProviderId::Xai),
            &sampling,
            "session-id",
            Some("custom-alpha"),
        );
        let WebSearchConfig::Enabled {
            api_key,
            base_url,
            model,
            extra_headers,
            alpha_test_key,
        } = config
        else {
            panic!("custom route with an explicit key must remain enabled");
        };
        assert_eq!(api_key, "custom-key");
        assert_eq!(base_url, "https://custom.example/v1");
        assert_eq!(model, "custom-search-model");
        assert_eq!(
            extra_headers.get("x-custom-route").map(String::as_str),
            Some("green")
        );
        assert!(!extra_headers.contains_key("x-xai-route"));
        assert_eq!(alpha_test_key.as_deref(), Some("custom-alpha"));
    }

    #[test]
    fn returning_to_original_web_search_provider_preserves_dedicated_route() {
        let mut sampling = xai_grok_sampler::SamplerConfig::default();
        sampling.provider = xai_grok_sampling_types::ProviderId::Xai;
        sampling.api_key = Some("rotated-xai-key".to_owned());
        sampling.base_url = "https://main.x.ai/v1".to_owned();
        sampling.model = "main-chat-model".to_owned();

        let config = web_search_config_for_provider(
            &enabled_xai_web_search_config(),
            &Default::default(),
            false,
            Some(xai_grok_sampling_types::ProviderId::Xai),
            &sampling,
            "session-id",
            None,
        );
        assert!(matches!(
            config,
            WebSearchConfig::Enabled {
                ref api_key,
                ref base_url,
                ref model,
                ref extra_headers,
                ..
            } if api_key == "rotated-xai-key"
                && base_url == "https://search.x.ai/v1"
                && model == "xai-dedicated-search"
                && extra_headers.contains_key("x-xai-route")
        ));
    }

    #[tokio::test]
    async fn dynamic_web_search_tool_contract_switches_both_schema_and_description() {
        let agent = super::super::support::test_agent_with_tools(vec![]).await;
        let bridge = agent.tool_bridge().clone();

        install_web_search_tool_definition(&bridge, true)
            .await
            .unwrap();
        let codex = bridge
            .tool_definitions()
            .await
            .into_iter()
            .find(|definition| definition.function.name == "web_search")
            .expect("Codex web search definition");
        assert!(
            codex.function.parameters["properties"]
                .get("search_query")
                .is_some()
        );
        assert!(
            codex
                .function
                .description
                .as_deref()
                .is_some_and(|description| {
                    description.contains("search_query")
                        && description.contains("open: open")
                        && description.contains("click: follow")
                        && description.contains("find: locate")
                        && description.contains("fetches arbitrary public URLs")
                })
        );

        install_web_search_tool_definition(&bridge, false)
            .await
            .unwrap();
        let xai = bridge
            .tool_definitions()
            .await
            .into_iter()
            .find(|definition| definition.function.name == "web_search")
            .expect("xAI web search definition");
        assert!(xai.function.parameters["properties"].get("query").is_some());
        assert!(
            xai.function.parameters["properties"]
                .get("search_query")
                .is_none()
        );
        assert!(
            xai.function
                .description
                .as_deref()
                .is_some_and(|description| description.contains("coding and software development"))
        );

        remove_web_search_tool_definition(&bridge).await;
        assert!(
            bridge
                .tool_definitions()
                .await
                .into_iter()
                .all(|definition| definition.function.name != "web_search")
        );
    }

    #[test]
    fn xai_image_key_rotation_preserves_complete_provider_configuration() {
        let configured = ImageGenConfig::Enabled {
            api_key: "old-key".to_owned(),
            base_url: "https://custom-image.example/v1".to_owned(),
            extra_headers: indexmap::indexmap! {
                "user-agent".to_owned() => "custom-agent".to_owned(),
                "x-custom-route".to_owned() => "blue".to_owned(),
            },
            image_gen_enabled: false,
            image_edit_enabled: true,
            model_override: Some("custom-imagine-model".to_owned()),
            tier_restricted: true,
        };

        let rotated =
            xai_image_gen_config_with_rotated_key(&configured, Some("current-key")).unwrap();
        let ImageGenConfig::Enabled {
            api_key,
            base_url,
            extra_headers,
            image_gen_enabled,
            image_edit_enabled,
            model_override,
            tier_restricted,
        } = rotated
        else {
            panic!("configured xAI image provider must remain enabled");
        };
        assert_eq!(api_key, "current-key");
        assert_eq!(base_url, "https://custom-image.example/v1");
        assert_eq!(
            extra_headers.get("user-agent").map(String::as_str),
            Some("custom-agent")
        );
        assert_eq!(
            extra_headers.get("x-custom-route").map(String::as_str),
            Some("blue")
        );
        assert!(!image_gen_enabled);
        assert!(image_edit_enabled);
        assert_eq!(model_override.as_deref(), Some("custom-imagine-model"));
        assert!(tier_restricted);

        let original = xai_image_gen_config_with_rotated_key(&configured, None).unwrap();
        assert!(matches!(
            original,
            ImageGenConfig::Enabled { ref api_key, .. } if api_key == "old-key"
        ));
        assert!(xai_image_gen_config_with_rotated_key(&ImageGenConfig::Disabled, None).is_none());
    }

    fn enabled_image_config() -> ImageGenConfig {
        ImageGenConfig::Enabled {
            api_key: "xai-image-key".to_owned(),
            base_url: "https://api.x.ai/v1".to_owned(),
            extra_headers: indexmap::IndexMap::new(),
            image_gen_enabled: true,
            image_edit_enabled: true,
            model_override: None,
            tier_restricted: false,
        }
    }

    #[derive(Debug)]
    struct CodexImageAuth;

    impl xai_grok_tools::types::ApiKeyProvider for CodexImageAuth {
        fn current_api_key(&self) -> Option<String> {
            Some("codex-image-key".to_owned())
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(xai_grok_tools::types::OPENAI_CODEX_PROVIDER_ID)
        }
    }

    #[tokio::test]
    async fn image_resource_and_definitions_follow_provider_switches() {
        use xai_grok_tools::implementations::grok_build::{
            IMAGE_EDIT_TOOL_NAME, IMAGE_GEN_TOOL_NAME,
        };

        async fn image_definition_names(bridge: &crate::tools::bridge::ToolBridge) -> Vec<String> {
            bridge
                .tool_definitions()
                .await
                .into_iter()
                .filter(|definition| {
                    matches!(
                        definition.function.name.as_str(),
                        IMAGE_GEN_TOOL_NAME | IMAGE_EDIT_TOOL_NAME
                    )
                })
                .map(|definition| definition.function.name)
                .collect()
        }

        let agent = super::super::support::test_agent_with_tools(vec![]).await;
        let bridge = agent.tool_bridge().clone();

        refresh_image_gen_resource(&bridge, Some(enabled_image_config()), None, None).await;
        assert!(bridge.read_resource::<ImageGenClient>().await.is_some());
        assert_eq!(image_definition_names(&bridge).await.len(), 2);

        // Custom providers own neither the xAI client nor its advertised tools.
        refresh_image_gen_resource(&bridge, None, None, None).await;
        assert!(bridge.read_resource::<ImageGenClient>().await.is_none());
        assert!(image_definition_names(&bridge).await.is_empty());

        // Codex keeps its provider-owned image path and advertises only live gates.
        let codex_config = ImageGenConfig::OpenAiCodex {
            base_url: xai_grok_sampling_types::OPENAI_CODEX_BASE_URL.to_owned(),
            image_gen_enabled: true,
            image_edit_enabled: false,
        };
        let codex_auth: xai_grok_tools::types::SharedApiKeyProvider =
            std::sync::Arc::new(CodexImageAuth);
        refresh_image_gen_resource(&bridge, Some(codex_config), Some(codex_auth), None).await;
        assert!(bridge.read_resource::<ImageGenClient>().await.is_some());
        assert_eq!(
            image_definition_names(&bridge).await,
            vec![IMAGE_GEN_TOOL_NAME.to_owned()]
        );

        refresh_image_gen_resource(&bridge, None, None, None).await;
        refresh_image_gen_resource(&bridge, Some(enabled_image_config()), None, None).await;
        assert!(bridge.read_resource::<ImageGenClient>().await.is_some());
        assert_eq!(image_definition_names(&bridge).await.len(), 2);
    }

    fn enabled_video_config() -> VideoGenConfig {
        VideoGenConfig::Enabled {
            api_key: "original-xai-key".to_owned(),
            base_url: "https://old-api.x.ai/v1".to_owned(),
            extra_headers: indexmap::indexmap! {
                "user-agent".to_owned() => "test-agent".to_owned(),
            },
            zdr_video_output_s3: None,
            tier_restricted: true,
        }
    }

    #[test]
    fn provider_video_config_disables_non_xai_and_restores_xai_settings() {
        let configured = enabled_video_config();
        for provider in [
            xai_grok_sampling_types::ProviderId::OpenAiCodex,
            xai_grok_sampling_types::ProviderId::Custom,
        ] {
            assert!(matches!(
                video_gen_config_for_provider(
                    provider,
                    &configured,
                    Some("must-not-cross-provider-boundary"),
                    "https://api.x.ai/v1",
                ),
                VideoGenConfig::Disabled
            ));
        }

        let restored = video_gen_config_for_provider(
            xai_grok_sampling_types::ProviderId::Xai,
            &configured,
            Some("current-xai-key"),
            "https://api.x.ai/v1",
        );
        let VideoGenConfig::Enabled {
            api_key,
            base_url,
            extra_headers,
            tier_restricted,
            ..
        } = restored
        else {
            panic!("xAI switch must restore video generation");
        };
        assert_eq!(api_key, "current-xai-key");
        assert_eq!(base_url, "https://api.x.ai/v1");
        assert_eq!(
            extra_headers.get("user-agent").map(String::as_str),
            Some("test-agent")
        );
        assert!(tier_restricted, "tier restriction must survive the switch");
    }

    #[tokio::test]
    async fn video_resource_lifecycle_is_xai_to_codex_to_xai() {
        use xai_grok_tools::implementations::grok_build::video_gen::{
            IMAGE_TO_VIDEO_TOOL_NAME, REFERENCE_TO_VIDEO_TOOL_NAME,
        };

        async fn video_definition_names(bridge: &crate::tools::bridge::ToolBridge) -> Vec<String> {
            bridge
                .tool_definitions()
                .await
                .into_iter()
                .filter(|definition| {
                    matches!(
                        definition.function.name.as_str(),
                        IMAGE_TO_VIDEO_TOOL_NAME | REFERENCE_TO_VIDEO_TOOL_NAME
                    )
                })
                .map(|definition| definition.function.name)
                .collect()
        }

        let agent = super::super::support::test_agent_with_tools(vec![]).await;
        let bridge = agent.tool_bridge().clone();
        let configured = enabled_video_config();

        refresh_video_gen_resource(
            &bridge,
            xai_grok_sampling_types::ProviderId::Xai,
            &configured,
            Some("first-xai-key"),
            "https://api.x.ai/v1",
            None,
            None,
        )
        .await;
        assert!(bridge.read_resource::<VideoGenClient>().await.is_some());
        assert_eq!(video_definition_names(&bridge).await.len(), 2);

        refresh_video_gen_resource(
            &bridge,
            xai_grok_sampling_types::ProviderId::OpenAiCodex,
            &configured,
            None,
            "https://api.x.ai/v1",
            None,
            None,
        )
        .await;
        assert!(bridge.read_resource::<VideoGenClient>().await.is_none());
        assert!(video_definition_names(&bridge).await.is_empty());

        refresh_video_gen_resource(
            &bridge,
            xai_grok_sampling_types::ProviderId::Xai,
            &configured,
            Some("second-xai-key"),
            "https://api.x.ai/v1",
            None,
            None,
        )
        .await;
        assert!(bridge.read_resource::<VideoGenClient>().await.is_some());
        assert_eq!(video_definition_names(&bridge).await.len(), 2);
    }

    #[test]
    fn unavailable_video_recipe_restores_xai_without_exposing_codex_tools() {
        let configured = VideoGenConfig::Unavailable {
            xai_fallback: Some(Box::new(enabled_video_config())),
        };
        assert!(!configured.is_enabled());

        let restored = video_gen_config_for_provider(
            xai_grok_sampling_types::ProviderId::Xai,
            &configured,
            Some("new-key"),
            "https://api.x.ai/v1",
        );
        assert!(matches!(
            restored,
            VideoGenConfig::Enabled { ref api_key, .. } if api_key == "new-key"
        ));
    }
}
