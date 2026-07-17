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

fn video_gen_config_for_provider(
    provider: xai_grok_sampling_types::ProviderId,
    configured: &xai_grok_tools::implementations::grok_build::video_gen::VideoGenConfig,
    current_api_key: Option<&str>,
    current_xai_base_url: &str,
) -> xai_grok_tools::implementations::grok_build::video_gen::VideoGenConfig {
    use xai_grok_tools::implementations::grok_build::video_gen::VideoGenConfig;

    if provider == xai_grok_sampling_types::ProviderId::OpenAiCodex {
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
        xai_grok_sampling_types::ProviderId::OpenAiCodex => {
            // ChatGPT Codex auth is not a general OpenAI API credential and
            // its backend does not expose a supported embeddings endpoint.
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
        return WebSearchConfig::CodexSubscription {
            base_url: sampling_config.base_url.clone(),
            model: sampling_config.model.clone(),
            session_id: session_id.to_owned(),
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
            && let Some(previous_tier) = previous_sampling_config
                .as_ref()
                .and_then(|previous| previous.service_tier.clone())
        {
            // Preserve the user's session selection across Codex model
            // switches. Validation is intentionally deferred to the per-turn
            // wire boundary so switching through a model without Fast support
            // does not erase the preference before switching back.
            sampling_config.service_tier = Some(previous_tier);
        }
        let model_id =
            if sampling_config.provider == xai_grok_sampling_types::ProviderId::OpenAiCodex {
                acp::ModelId::new(format!("openai-codex/{}", sampling_config.model))
            } else {
                acp::ModelId::new(sampling_config.model.clone())
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
                context_window: new_context_window,
                reasoning_effort: sampling_config.reasoning_effort,
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
                api_key: sampling_config.api_key.clone(),
                auth_type: crate::agent::config::resolve_chat_state_auth_type(
                    sampling_config.model.as_str(),
                    session_key.as_deref(),
                    existing.auth_type,
                ),
                alpha_test_key: existing.alpha_test_key,
                client_version: sampling_config.client_version.clone(),
            });
        self.refresh_provider_media_resources(&sampling_config)
            .await;
        self.refresh_provider_web_resources(&sampling_config, alpha_test_key.as_deref())
            .await;
        let bridge = self.agent.borrow().tool_bridge().clone();
        refresh_provider_memory_resource(self, &bridge, &sampling_config).await;
        self.model_auth_facts.replace(None);
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
        let _ = self
            .notifications
            .persistence_tx
            .send(PersistenceMsg::CurrentModel {
                model_id: model_id.clone(),
                agent_name: Some(agent_name),
                reasoning_effort: Some(sampling_config.reasoning_effort),
            });
        Ok(model_id)
    }

    async fn refresh_provider_web_resources(
        &self,
        sampling_config: &xai_grok_sampler::SamplerConfig,
        alpha_test_key: Option<&str>,
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
            self.rebuild_spec.web_search_disabled,
            self.rebuild_spec.web_search_provider,
            sampling_config,
            self.session_info.id.0.as_ref(),
            alpha_test_key,
        );
        let api_key_provider = match sampling_config.provider {
            xai_grok_sampling_types::ProviderId::OpenAiCodex => {
                crate::auth::codex::CodexAuthManager::new(&crate::util::grok_home::grok_home())
                    .ok()
                    .map(std::sync::Arc::new)
                    .map(crate::auth::codex::shared_api_key_provider)
            }
            xai_grok_sampling_types::ProviderId::Xai => self.auth_manager.as_ref().map(|manager| {
                std::sync::Arc::new(crate::auth::manager::SharedAuthKeyProvider(manager.clone()))
                    as xai_grok_tools::types::SharedApiKeyProvider
            }),
            xai_grok_sampling_types::ProviderId::Custom => None,
        };
        if sampling_config.provider.is_openai_codex() && api_key_provider.is_none() {
            tracing::warn!(
                provider = "openai_codex",
                "web_search disabled after provider switch: scoped request authentication is unavailable"
            );
            web_search_config = WebSearchConfig::Disabled;
        }

        let codex_subscription_search = web_search_config.is_codex_subscription();
        let hosted_web_search_enabled = web_search_config.allows_hosted_responses_tool();
        let backend_search_enabled =
            self.rebuild_spec.backend_search && !sampling_config.provider.is_openai_codex();
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
            match WebSearchClient::new(&web_search_config, api_key_provider) {
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

        let fetch_params: Option<WebFetchParams> = self
            .rebuild_spec
            .web_fetch_config
            .params_for_codex_subscription(sampling_config.provider.is_openai_codex())
            .cloned();
        self.permissions.set_web_fetch_allowed_domains(
            fetch_params
                .as_ref()
                .map(WebFetchParams::allowed_domains)
                .unwrap_or_default(),
        );
        if let Some(params) = fetch_params {
            match WebFetchClient::new(&params) {
                Ok(client) => {
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
    ) {
        use xai_grok_tools::implementations::grok_build::image_gen::{
            ImageGenClient, ImageGenConfig,
        };

        let bridge = self.agent.borrow().tool_bridge().clone();
        let endpoints = self.models_manager.endpoints();
        if sampling_config.provider == xai_grok_sampling_types::ProviderId::OpenAiCodex {
            let (image_gen_enabled, image_edit_enabled) = self.models_manager.image_tool_gates();
            if !image_gen_enabled && !image_edit_enabled {
                bridge.remove_resource::<ImageGenClient>().await;
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
            // Image generation/editing uses the standalone Codex image model;
            // selected-model image-input support remains an input/read concern.
            let manager =
                crate::auth::codex::CodexAuthManager::new(&crate::util::grok_home::grok_home())
                    .ok()
                    .map(std::sync::Arc::new);
            let provider = manager.map(crate::auth::codex::shared_api_key_provider);
            let config = ImageGenConfig::OpenAiCodex {
                base_url: xai_grok_sampling_types::OPENAI_CODEX_BASE_URL.to_owned(),
                image_gen_enabled,
                image_edit_enabled,
            };
            if let Ok(client) = ImageGenClient::new(&config, provider) {
                bridge
                    .update_resource(client.with_attribution_callback(None))
                    .await;
            } else {
                bridge.remove_resource::<ImageGenClient>().await;
            }
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

        let provider = self.auth_manager.as_ref().map(|manager| {
            std::sync::Arc::new(crate::auth::manager::SharedAuthKeyProvider(manager.clone()))
                as xai_grok_tools::types::SharedApiKeyProvider
        });
        let (image_gen_enabled, image_edit_enabled) = self.models_manager.image_tool_gates();
        if let Some(api_key) = sampling_config.api_key.clone()
            && (image_gen_enabled || image_edit_enabled)
        {
            let config = xai_image_gen_config_with_rotated_key(
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
            });
            match ImageGenClient::new(&config, provider.clone()) {
                Ok(client) => {
                    bridge
                        .update_resource(client.with_attribution_callback(
                            self.rebuild_spec.attribution_callback.clone(),
                        ))
                        .await;
                }
                Err(_) => {
                    bridge.remove_resource::<ImageGenClient>().await;
                }
            }
        } else {
            bridge.remove_resource::<ImageGenClient>().await;
        }

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
    use xai_grok_tools::implementations::grok_build::image_gen::ImageGenConfig;
    use xai_grok_tools::implementations::grok_build::video_gen::{VideoGenClient, VideoGenConfig};
    use xai_grok_tools::implementations::web_search::WebSearchConfig;

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
            } if base_url == xai_grok_sampling_types::OPENAI_CODEX_BASE_URL
                && model == "gpt-5.6-luna"
                && session_id == "session-public-id"
        ));
    }

    #[test]
    fn unavailable_xai_search_does_not_block_later_codex_subscription_search() {
        let sampling = xai_grok_sampler::SamplerConfig::openai_codex("gpt-5.6-luna");
        let config = web_search_config_for_provider(
            &WebSearchConfig::Disabled,
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
    fn provider_video_config_disables_codex_and_restores_xai_settings() {
        let configured = enabled_video_config();
        assert!(matches!(
            video_gen_config_for_provider(
                xai_grok_sampling_types::ProviderId::OpenAiCodex,
                &configured,
                None,
                "https://api.x.ai/v1",
            ),
            VideoGenConfig::Disabled
        ));

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
