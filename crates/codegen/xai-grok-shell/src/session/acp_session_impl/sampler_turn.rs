//! Sampler-turn pipeline for `SessionActor`: tool definitions, model auth
//! facts/gates and retry, sampler config reconstruction, sampling-failure
//! recovery, and per-response usage recording.
use super::*;
/// Auth-failure detector for tool errors. Matches strictly on HTTP 401
/// when the error carries a structured status code, mirroring
/// `SamplingError::is_auth_error` in xai-grok-sampling-types: 403 is
/// deliberately excluded because it means "authenticated but forbidden"
/// (content-safety blocks, ZDR-gated requests, remote settings gates), where
/// a token refresh would be a no-op and would surface to the client as
/// a spurious auth_required teardown.
///
/// String fallbacks remain for tools that surface auth failures without
/// going through the structured `HttpFailure` path (e.g. JSON-only
/// `invalid_token` payloads, BYOK key-validation messages).
pub(super) fn is_auth_tool_error(err: &xai_tool_runtime::ToolError) -> bool {
    if let Some(details) = &err.details
        && let Some(status) = details
            .get(HTTP_STATUS_DETAILS_KEY)
            .and_then(|s| s.as_u64())
    {
        return status == 401;
    }
    let lower = err.to_string().to_ascii_lowercase();
    lower.contains("unauthorized")
        || lower.contains("invalid api key")
        || lower.contains("invalid_token")
}

/// Whether a tool error has already exhausted an explicitly provider-owned
/// auth recovery path. This is deliberately fail-closed for every named
/// provider: the session-level retry wrapper only owns unscoped/xAI errors
/// and must never guess that another provider's 401 is safe to replay with
/// the global xAI `AuthManager`.
fn provider_auth_recovery_exhausted(err: &xai_tool_runtime::ToolError) -> bool {
    let Some(details) = err.details.as_ref() else {
        return false;
    };
    details
        .get(xai_grok_tools::types::AUTH_RECOVERY_PROVIDER_DETAILS_KEY)
        .and_then(serde_json::Value::as_str)
        .is_some()
        && details
            .get(xai_grok_tools::types::AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
}
/// Gate inputs bundled with the composed decision so the 401-recovery log can
/// report the components.
#[derive(Clone, Copy)]
struct SessionTokenAuthGate {
    is_session_based: bool,
    model_byok: crate::agent::auth_method::ModelByok,
    /// Whether the request targets a first-party host. Lets an `Unknown`
    /// BYOK status still refresh against cli-chat-proxy / `*.x.ai` without
    /// risking a session-token leak to a third-party BYOK endpoint.
    endpoint_is_first_party: bool,
}
impl SessionTokenAuthGate {
    /// Single place `is_session_based` / `endpoint_is_first_party` are derived,
    /// so all call sites assemble the gate identically.
    fn new(
        auth_method_id: Option<&acp::AuthMethodId>,
        model_byok: crate::agent::auth_method::ModelByok,
        base_url: &str,
    ) -> Self {
        Self {
            is_session_based: auth_method_id
                .is_some_and(crate::agent::auth_method::is_session_based_method),
            model_byok,
            endpoint_is_first_party: crate::util::is_first_party_xai_url(base_url),
        }
    }
    fn active(self) -> bool {
        crate::agent::auth_method::session_token_auth_gate(
            self.is_session_based,
            self.model_byok,
            self.endpoint_is_first_party,
        )
    }
}
/// Run a tool call; on an auth-shaped failure, attempt recovery via
/// `AuthManager` and one retry. When `shared_recovery` is `Some`, concurrent
/// 401s in the same batch deduplicate via `OnceCell::get_or_init`.
pub(super) async fn call_with_auth_retry<F, Fut>(
    auth_manager: Option<&std::sync::Arc<crate::auth::AuthManager>>,
    shared_recovery: Option<&tokio::sync::OnceCell<bool>>,
    tool_name: &str,
    mut call: F,
) -> Result<xai_grok_tools::types::output::ToolRunResult, xai_tool_runtime::ToolError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<
            Output = Result<
                xai_grok_tools::types::output::ToolRunResult,
                xai_tool_runtime::ToolError,
            >,
        >,
{
    let result = call().await;
    let Err(ref err) = result else { return result };
    if !is_auth_tool_error(err) {
        return result;
    }
    if provider_auth_recovery_exhausted(err) {
        tracing::warn!(
            tool = tool_name,
            "provider-owned tool auth recovery exhausted; refusing cross-provider retry"
        );
        return result;
    }
    let Some(am) = auth_manager else {
        return result;
    };
    let src = crate::auth::recovery::RecoverySource::Background;
    let recovered = match shared_recovery {
        Some(cell) => *cell.get_or_init(|| am.try_recover_unauthorized(src)).await,
        None => am.try_recover_unauthorized(src).await,
    };
    if recovered {
        tracing::info!(
            tool = tool_name,
            "auth recovery: tool 401, recovered, retrying"
        );
        call().await
    } else {
        tracing::warn!(tool = tool_name, "auth recovery: tool 401, refresh failed");
        xai_grok_telemetry::unified_log::warn(
            "auth recovery: tool 401, refresh failed",
            None,
            Some(serde_json::json!({ "tool" : tool_name })),
        );
        result
    }
}
impl SessionActor {
    pub(super) async fn prepare_tool_definitions_timed(&self) -> (Vec<ToolDefinition>, u64) {
        let mcp_wait_start = std::time::Instant::now();
        match self.mcp_strategy {
            McpInitStrategy::Blocking => {
                if !self.mcp_state.lock().await.is_initialized() {
                    tracing::info!(
                        "Blocking strategy: waiting for MCP initialization before first prompt..."
                    );
                    self.wait_for_mcp_initialized().await;
                }
            }
            McpInitStrategy::Progressive => {}
        }
        let mcp_wait_ms = mcp_wait_start.elapsed().as_millis() as u64;
        let defs = self.prepare_tool_definitions_inner().await;
        (defs, mcp_wait_ms)
    }
    pub(super) async fn prepare_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.prepare_tool_definitions_timed().await.0
    }
    /// The exact tool specs a turn sends, BEFORE the turn-specific
    /// structured-output append. Single source of truth shared by the turn
    /// (`acp_session_impl/turn.rs`) and the `SnapshotToolDefinitions` handler, so
    /// a verbatim-fork child's tool prefix can never silently drift from what the
    /// parent turn actually sends. `defs` is the already-resolved tool list
    /// (`prepare_tool_definitions_*`); this applies only the `web_search` drop
    /// under backend search and the `ToolSpec::from` mapping.
    pub(crate) fn turn_base_tool_specs(&self, defs: &[ToolDefinition]) -> Vec<ToolSpec> {
        let use_backend_search =
            self.agent.borrow().backend_search_enabled() && self.supports_backend_search.get();
        defs.iter()
            .filter(|td| !use_backend_search || td.function.name != "web_search")
            .cloned()
            .map(ToolSpec::from)
            .collect()
    }
    pub(super) async fn prepare_tool_definitions_inner(&self) -> Vec<ToolDefinition> {
        let bridge = self.agent.borrow().tool_bridge().clone();
        let defs = bridge.tool_definitions_builtins_only().await;
        let plan_active = self.plan_mode.lock().is_active();
        filter_cursor_tools_by_plan_mode(defs, plan_active)
    }
    /// Memoized per-model [`ModelAuthFacts`](crate::agent::config::ModelAuthFacts),
    /// keyed by `model_id`.
    ///
    /// A fresh `Unknown` (config currently unparseable) falls back to the last
    /// definite value for the same `model_id` rather than demoting a live session
    /// to non-refreshable api-key mode. Because a config edit can turn the
    /// currently-selected model into a per-model BYOK model without changing
    /// `model_id`, keying on `model_id` alone is insufficient — each
    /// model/credential chokepoint must clear this memo (`replace(None)`).
    pub(super) fn model_auth_facts(&self, model_id: &str) -> crate::agent::config::ModelAuthFacts {
        use crate::agent::auth_method::ModelByok;
        if let Some((cached_id, facts)) = self.model_auth_facts.borrow().as_ref()
            && cached_id == model_id
            && facts.byok != ModelByok::Unknown
        {
            return *facts;
        }
        let fresh = crate::agent::config::resolve_model_auth_facts(model_id);
        if fresh.byok == ModelByok::Unknown {
            if let Some((cached_id, facts)) = self.model_auth_facts.borrow().as_ref()
                && cached_id == model_id
            {
                return *facts;
            }
            return fresh;
        }
        *self.model_auth_facts.borrow_mut() = Some((model_id.to_string(), fresh));
        fresh
    }
    /// Gate inputs for `model_id` routed to `base_url`. See
    /// [`crate::agent::auth_method::session_token_auth_gate`] for the rationale
    /// (`base_url` keeps an `Unknown` BYOK status refreshable only
    /// against first-party xAI hosts).
    fn auth_gate(&self, model_id: &str, base_url: &str) -> SessionTokenAuthGate {
        let byok = self.model_auth_facts(model_id).byok;
        let auth_method = self.auth_method_id.load();
        SessionTokenAuthGate::new(auth_method.as_deref(), byok, base_url)
    }
    /// Emit a unified-log breadcrumb whenever the session-token refresh gate is
    /// evaluated with an **`Unknown`** per-model BYOK status on a session-based
    /// method — the condition that (pre-fix) silently demoted live sessions to
    /// stale-token 401s. The uploaded per-turn unified log then shows whether
    /// the first-party-endpoint fallback kept refresh active or withheld it, so
    /// we can confirm the fix works (or catch a residual demotion) per session
    /// even when server-side metrics only show the aggregate 401. No-op for a
    /// definite `Byok`/`NotByok`, so steady-state turns stay quiet — a burst of
    /// these is itself the signal that `Unknown` is being hit in the field.
    fn log_auth_gate_unknown(&self, site: &str, gate: SessionTokenAuthGate, base_url: &str) {
        use crate::agent::auth_method::ModelByok;
        if gate.model_byok != ModelByok::Unknown || !gate.is_session_based {
            return;
        }
        let refresh_active = gate.active();
        let ctx = serde_json::json!(
            { "site" : site, "model_byok" : gate.model_byok.as_str(), "is_session_based"
            : gate.is_session_based, "endpoint_is_first_party" : gate
            .endpoint_is_first_party, "refresh_active" : refresh_active, "base_url" :
            base_url, }
        );
        let sid = Some(self.session_info.id.0.as_ref());
        if refresh_active {
            xai_grok_telemetry::unified_log::info(
                "auth gate: Unknown BYOK on first-party endpoint — session-token refresh kept active",
                sid,
                Some(ctx),
            );
        } else {
            xai_grok_telemetry::unified_log::warn(
                "auth gate: Unknown BYOK on non-first-party endpoint — refresh withheld (may surface stale-token 401)",
                sid,
                Some(ctx),
            );
        }
    }
    /// Reconstruct a full `SamplerConfig` (with credentials) by combining
    /// the actor's `SamplingConfig` and `Credentials`. Folds in the
    /// URL-derived headers (cli-chat-proxy auth, the staging auth header)
    /// so the sampler crate stays URL-agnostic.
    pub(super) async fn reconstruct_full_config(&self) -> Result<SamplingConfig, acp::Error> {
        #[allow(clippy::items_after_statements)]
        #[derive(Debug)]
        struct TraceContextInjector;
        impl xai_grok_sampler::HeaderInjector for TraceContextInjector {
            fn inject(&self, headers: &mut reqwest::header::HeaderMap) {
                if let Some(tp) = xai_file_utils::trace_context::current_traceparent()
                    && let Ok(v) = reqwest::header::HeaderValue::from_str(&tp)
                {
                    headers.insert("traceparent", v);
                }
            }
        }
        #[allow(clippy::items_after_statements)]
        struct AuthManagerBearerResolver(std::sync::Arc<crate::auth::AuthManager>);
        impl std::fmt::Debug for AuthManagerBearerResolver {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("AuthManagerBearerResolver").finish()
            }
        }
        impl xai_grok_sampler::BearerResolver for AuthManagerBearerResolver {
            fn current_bearer(&self) -> Option<String> {
                self.0.current_or_expired().map(|a| a.key)
            }
        }
        let mut cfg = self
            .chat_state_handle
            .get_sampling_config()
            .await
            .unwrap_or_else(|| xai_grok_sampling_types::SamplingConfig {
                provider: xai_grok_sampling_types::ProviderId::Xai,
                credential_binding: None,
                base_url: String::new(),
                model: String::new(),
                max_completion_tokens: None,
                temperature: None,
                top_p: None,
                api_backend: Default::default(),
                extra_headers: Default::default(),
                comp_hash: None,
                context_window: std::num::NonZeroU64::new(256_000).unwrap(),
                reasoning_effort: None,
                supports_reasoning_summary_parameter: false,
                default_reasoning_summary: None,
                service_tier: None,
                stream_tool_calls: None,
            });
        let creds = self.chat_state_handle.get_credentials().await;
        let model_facts = self.model_auth_facts(cfg.model.as_str());
        let is_xai = cfg.provider == xai_grok_sampling_types::ProviderId::Xai;
        let stored_credentials_match_provider = match cfg.provider {
            xai_grok_sampling_types::ProviderId::OpenAiCodex
            | xai_grok_sampling_types::ProviderId::KimiCode
            | xai_grok_sampling_types::ProviderId::ZaiCodingPlan => true,
            xai_grok_sampling_types::ProviderId::Custom => {
                creds.provider == Some(xai_grok_sampling_types::ProviderId::Custom)
            }
            xai_grok_sampling_types::ProviderId::Xai => matches!(
                creds.provider,
                None | Some(xai_grok_sampling_types::ProviderId::Xai)
            ),
        };
        if !stored_credentials_match_provider {
            tracing::error!(
                request_provider = %cfg.provider,
                credential_provider = ?creds.provider,
                "stored credentials do not belong to the request provider; dropping them"
            );
        }
        let auth_method = self.auth_method_id.load();
        let gate =
            SessionTokenAuthGate::new(auth_method.as_deref(), model_facts.byok, &cfg.base_url);
        let use_bearer_resolver = is_xai && gate.active();
        self.log_auth_gate_unknown("reconstruct_full_config", gate, &cfg.base_url);
        let auth_scheme = model_facts.auth_scheme;
        let mut extra_headers = cfg.extra_headers.clone();
        if is_xai {
            crate::agent::config::inject_url_derived_headers(
                &mut extra_headers,
                creds.alpha_test_key.as_deref(),
                &cfg.base_url,
            );
        }
        let compaction_at_tokens = self.compaction_at_tokens.get();
        let compactions_remaining = self.compactions_remaining.get();
        if is_xai && (compactions_remaining.is_some() || compaction_at_tokens.is_some()) {
            let has_compaction_summary = self
                .chat_state_handle
                .get_last_compaction_prompt_index()
                .await
                .is_some();
            if let Some(value) =
                compactions_remaining.and_then(|c| c.resolve(has_compaction_summary))
            {
                extra_headers.insert("x-compactions-remaining".to_string(), value.to_string());
            }
            if !has_compaction_summary
                && let Some(value) = compaction_at_tokens.and_then(|c| {
                    c.resolve(
                        cfg.context_window.get(),
                        self.compaction.threshold_percent.get(),
                    )
                })
            {
                extra_headers.insert("x-compaction-at".to_string(), value.to_string());
            }
        }
        // Provider binding is performed once the complete generic/session
        // fields have been assembled below. Until then, preserve only the
        // expected non-secret record metadata; never attach a partial Codex
        // auth runtime here.
        let credential_binding = cfg.credential_binding.clone();
        let request_auth = None;
        // Last wire-boundary guard for legacy/restored chat state. Ingress
        // paths validate effort overrides, but an older persisted session may
        // still contain a value its current dynamic catalog no longer offers
        // (notably GPT-5.6 Luna + ultra). Unknown models retain their existing
        // value for backwards-compatible custom routing; known models fall
        // back to their advertised default.
        let reasoning_effort = match cfg.reasoning_effort {
            Some(effort)
                if self
                    .models_manager
                    .model_in_catalog_for_provider(cfg.provider, &cfg.model) =>
            {
                match self.models_manager.resolve_reasoning_effort_for_provider(
                    cfg.provider,
                    &cfg.model,
                    effort,
                ) {
                    Some(effective) => Some(effective),
                    None => {
                        let fallback = self
                            .models_manager
                            .model_default_reasoning_effort_for_provider(cfg.provider, &cfg.model);
                        tracing::warn!(
                            model = %cfg.model,
                            rejected_effort = %effort,
                            fallback_effort = ?fallback,
                            "stored reasoning effort is not advertised by model; using catalog default"
                        );
                        fallback
                    }
                }
            }
            effort => effort,
        };
        if cfg.reasoning_effort != reasoning_effort {
            cfg.reasoning_effort = reasoning_effort;
            self.chat_state_handle.update_sampling_config(cfg.clone());
        }
        let is_codex = cfg.provider == xai_grok_sampling_types::ProviderId::OpenAiCodex;
        let service_tier = if is_codex
            && self
                .models_manager
                .model_in_catalog_for_provider(cfg.provider, &cfg.model)
        {
            cfg.service_tier.as_deref().and_then(|tier| {
                self.models_manager.resolve_service_tier_for_provider(
                    cfg.provider,
                    &cfg.model,
                    tier,
                )
            })
        } else {
            None
        };
        let custom_owns_key = cfg.provider == xai_grok_sampling_types::ProviderId::Custom
            && stored_credentials_match_provider
            && model_facts.byok == crate::agent::auth_method::ModelByok::Byok
            && creds.auth_type == xai_chat_state::AuthType::ApiKey;
        let credential_source = match cfg.provider {
            xai_grok_sampling_types::ProviderId::OpenAiCodex => {
                xai_grok_sampling_types::CredentialSourceId::OpenAiCodexSubscription
            }
            xai_grok_sampling_types::ProviderId::KimiCode => {
                xai_grok_sampling_types::CredentialSourceId::KimiCodeApiKey
            }
            xai_grok_sampling_types::ProviderId::ZaiCodingPlan => {
                xai_grok_sampling_types::CredentialSourceId::ZaiCodingPlanApiKey
            }
            xai_grok_sampling_types::ProviderId::Custom if custom_owns_key => {
                xai_grok_sampling_types::CredentialSourceId::StaticApiKey
            }
            xai_grok_sampling_types::ProviderId::Custom => {
                xai_grok_sampling_types::CredentialSourceId::Unspecified
            }
            xai_grok_sampling_types::ProviderId::Xai if use_bearer_resolver => {
                xai_grok_sampling_types::CredentialSourceId::XaiSession
            }
            xai_grok_sampling_types::ProviderId::Xai
                if stored_credentials_match_provider
                    && creds.auth_type == xai_chat_state::AuthType::SessionToken =>
            {
                xai_grok_sampling_types::CredentialSourceId::XaiSession
            }
            xai_grok_sampling_types::ProviderId::Xai if stored_credentials_match_provider => {
                xai_grok_sampling_types::CredentialSourceId::XaiApiKey
            }
            xai_grok_sampling_types::ProviderId::Xai => {
                xai_grok_sampling_types::CredentialSourceId::Unspecified
            }
        };
        let request_api_key = match cfg.provider {
            xai_grok_sampling_types::ProviderId::OpenAiCodex
            | xai_grok_sampling_types::ProviderId::KimiCode
            | xai_grok_sampling_types::ProviderId::ZaiCodingPlan => None,
            xai_grok_sampling_types::ProviderId::Custom if custom_owns_key => creds.api_key.clone(),
            xai_grok_sampling_types::ProviderId::Custom => None,
            xai_grok_sampling_types::ProviderId::Xai if stored_credentials_match_provider => {
                creds.api_key.clone()
            }
            xai_grok_sampling_types::ProviderId::Xai => None,
        };
        let mut state_cfg = cfg.clone();
        let full_config = SamplingConfig {
            provider: cfg.provider,
            credential_source,
            credential_binding,
            api_key: request_api_key,
            base_url: cfg.base_url,
            model: cfg.model,
            max_completion_tokens: cfg.max_completion_tokens,
            temperature: cfg.temperature,
            top_p: cfg.top_p,
            api_backend: cfg.api_backend,
            auth_scheme,
            extra_headers,
            comp_hash: cfg.comp_hash.clone(),
            context_window: cfg.context_window.get(),
            client_version: creds.client_version,
            reasoning_effort,
            supports_reasoning_summary_parameter: cfg.supports_reasoning_summary_parameter,
            default_reasoning_summary: cfg.default_reasoning_summary.clone(),
            service_tier,
            force_http1: false,
            max_retries: Some(self.max_retries),
            stream_tool_calls: cfg.stream_tool_calls.unwrap_or(false),
            idle_timeout_secs: None,
            client_identifier: self.client_identifier.clone(),
            deployment_id: is_xai
                .then(|| {
                    crate::managed_config::resolve_deployment_id(
                        crate::managed_config::resolve_deployment_key().as_deref(),
                    )
                })
                .flatten(),
            user_id: is_xai
                .then(|| {
                    self.auth_manager
                        .as_ref()
                        .and_then(|am| am.current_or_expired())
                        .filter(|a| a.is_xai_auth())
                        .map(|a| a.user_id)
                })
                .flatten(),
            origin_client: self.origin_client.clone(),
            attribution_callback: is_xai.then(|| self.attribution_callback.clone()).flatten(),
            bearer_resolver: if use_bearer_resolver {
                self.auth_manager
                    .as_ref()
                    .map(|am| -> xai_grok_sampler::SharedBearerResolver {
                        std::sync::Arc::new(AuthManagerBearerResolver(am.clone()))
                    })
            } else {
                None
            },
            request_auth,
            supports_backend_search: self.supports_backend_search.get(),
            compactions_remaining: is_xai.then(|| self.compactions_remaining.get()).flatten(),
            compaction_at_tokens: is_xai.then(|| self.compaction_at_tokens.get()).flatten(),
            doom_loop_recovery: (!is_codex).then_some(self.doom_loop_recovery).flatten(),
            header_injector: Some(std::sync::Arc::new(TraceContextInjector)),
        };
        let bound_runtime = crate::session::provider::bind_provider_runtime(full_config, None)
            .await
            .map_err(|error| acp::Error::auth_required().data(error.to_string()))?;
        if (is_codex || cfg.provider.is_kimi_code() || cfg.provider.is_zai_coding_plan())
            && state_cfg.credential_binding != bound_runtime.sampler_config.credential_binding
        {
            state_cfg.credential_binding = bound_runtime.sampler_config.credential_binding.clone();
            self.chat_state_handle.update_sampling_config(state_cfg);
            let _ = self
                .notifications
                .persistence_tx
                .send(PersistenceMsg::CredentialBinding(
                    bound_runtime.sampler_config.credential_binding.clone(),
                ));
        }
        Ok(bound_runtime.sampler_config)
    }
    /// Install auto-mode permission classifier with a live LLM side-query
    /// (laziness-classifier pattern: `prepare_chat_completion` +
    /// `conversation_collect` on a LocalSet task; channel bridges the
    /// `Send` permission actor). Heuristic runs only when the side-query
    /// errors or returns unparseable text.
    pub(crate) async fn wire_permission_auto_llm_classifier(self: &Arc<Self>) {
        if !self.permissions.is_auto_mode() {
            return;
        }
        if self.permissions.has_llm_side_query() {
            return;
        }
        let auto_cfg = crate::util::config::resolve_auto_mode_config_from_disk();
        let session_model = self
            .chat_state_handle
            .get_sampling_config()
            .await
            .map(|c| c.model)
            .unwrap_or_default();
        let aux_classifier_sampler = match auto_cfg.classifier_model.as_deref() {
            Some(slug) => self
                .resolve_routed_aux_sampling_client(slug)
                .await
                .map_err(|error| error.to_string()),
            None => Ok(None),
        };
        let models = self.models_manager.models();
        let effective_supports_re = crate::agent::config::effective_classifier_supports_re(
            aux_classifier_sampler
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|(_, route)| route.model.as_str()),
            &session_model,
            &models,
        );
        let (prompt_type, classifier_reasoning_effort) =
            crate::util::config::auto_mode_classifier_defaults(&auto_cfg, effective_supports_re);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(
            Vec<xai_grok_workspace::permission::ClassifierMessage>,
            tokio::sync::oneshot::Sender<Result<String, String>>,
        )>();
        let session = Arc::clone(self);
        tokio::task::spawn_local(async move {
            const TIMEOUT_MS: u64 = 15_000;
            while let Some((messages, respond_to)) = rx.recv().await {
                let result = async {
                    let (sampling_client, model) = match &aux_classifier_sampler {
                        Ok(Some((client, route))) => (client.clone(), route.model.clone()),
                        Ok(None) => {
                            let client = session
                                .prepare_chat_completion(false)
                                .await
                                .map_err(|e| e.to_string())?;
                            let model = session
                                .chat_state_handle
                                .get_sampling_config()
                                .await
                                .map(|c| c.model)
                                .unwrap_or_default();
                            (client, model)
                        }
                        Err(error) => return Err(error.clone()),
                    };
                    let session_id = session.session_info.id.to_string();
                    let items = messages
                        .into_iter()
                        .map(|m| match m.role {
                            xai_grok_workspace::permission::ClassifierMessageRole::System => {
                                ConversationItem::system(m.text)
                            }
                            xai_grok_workspace::permission::ClassifierMessageRole::User => {
                                ConversationItem::user(m.text)
                            }
                        })
                        .collect::<Vec<_>>();
                    let request = ConversationRequest {
                        items,
                        tools: vec![],
                        hosted_tools: vec![],
                        tool_choice: None,
                        model: Some(model),
                        temperature: None,
                        max_output_tokens: None,
                        json_schema: Some(
                            xai_grok_workspace::permission::classifier_output_json_schema(),
                        ),
                        reasoning_effort: classifier_reasoning_effort,
                        x_grok_conv_id: Some(session_id.clone()),
                        x_grok_req_id: Some(format!("xai-perm-auto-{}", uuid::Uuid::new_v4())),
                        x_grok_session_id: Some(session_id),
                        x_grok_agent_id: Some(xai_grok_telemetry::id::agent_id()),
                        ..ConversationRequest::default()
                    };
                    let fut = sampling_client.conversation_collect(request);
                    let response =
                        tokio::time::timeout(std::time::Duration::from_millis(TIMEOUT_MS), fut)
                            .await
                            .map_err(|_| "permission auto classifier timed out".to_string())?
                            .map_err(|e| e.to_string())?;
                    Ok(response.assistant_text())
                }
                .await;
                let _ = respond_to.send(result);
            }
        });
        let clf =
            xai_grok_workspace::permission::LlmPermissionClassifier::with_channel(tx, prompt_type);
        debug_assert!(
            clf.has_side_query(),
            "channel-wired classifier must report has_side_query"
        );
        self.permissions.set_classifier_with_side_query(clf, true);
        tracing::info!(
            session_id = % self.session_info.id,
            "Wired live LLM permission auto-mode classifier (session sampling channel)"
        );
    }
    /// Resolve a standalone aux-model `SamplerConfig` for `slug` via the shared
    /// catalog routing (Tier-1 catalog creds / Tier-2 xAI-proxy via session token
    /// / `XAI_API_KEY` / deployment key), gathering the session-local auth context
    /// once. Shared by image-describe and the classifier so the gather can't
    /// drift. `None` ⇒ caller falls back to the session model.
    pub(super) async fn resolve_aux_sampler_config(
        &self,
        slug: &str,
    ) -> Option<xai_grok_sampler::SamplerConfig> {
        let creds = self.chat_state_handle.get_credentials().await;
        let session_key = self
            .auth_manager
            .as_ref()
            .and_then(|am| am.current_or_expired().map(|a| a.key.clone()));
        let models = self.models_manager.models();
        let endpoints = self.models_manager.endpoints();
        let disable_api_key_auth = self
            .auth_manager
            .as_ref()
            .map(|am| am.grok_com_config().api_key_auth_disabled())
            .unwrap_or(false);
        crate::agent::config::resolve_aux_model_sampling_config(
            slug,
            &models,
            &endpoints,
            session_key.as_deref(),
            disable_api_key_auth,
            creds.alpha_test_key.clone(),
            creds.client_version.clone(),
        )
    }
    /// Resolve a dedicated sampler for an auxiliary model `slug`, stamping
    /// session-local auth/attribution while preserving the model's catalog
    /// provider, endpoint, and credential owner. `None` means the caller must
    /// fall back to the active session model rather than sending `slug` to the
    /// active provider's endpoint.
    pub(super) async fn resolve_routed_aux_sampling_client(
        &self,
        slug: &str,
    ) -> Result<
        Option<(
            xai_grok_sampler::SamplingClient,
            crate::session::provider::ProviderModelRoute,
        )>,
        acp::Error,
    > {
        let active_session_config = self.reconstruct_full_config().await?;
        let Some(mut cfg) = self.resolve_aux_sampler_config(slug).await else {
            return Ok(None);
        };
        // A subscription-provider auxiliary route remains pinned to the
        // restored session record. Catalog resolution must not silently adopt
        // a process-current account or Kimi API-key record.
        crate::session::provider::pin_provider_candidate_to_active_record(
            &mut cfg,
            active_session_config.provider,
            active_session_config.credential_binding.as_ref(),
        );
        crate::agent::config::stamp_session_local_sampler_fields(
            &mut cfg,
            &active_session_config,
            self.client_identifier.clone(),
            Some(self.max_retries),
        );
        let bound_runtime = crate::session::provider::bind_provider_runtime(cfg, None)
            .await
            .map_err(|error| acp::Error::auth_required().data(error.to_string()))?;
        self.mark_codex_auxiliary_usage_incomplete(bound_runtime.route.provider)
            .await;
        let route = bound_runtime.route;
        let client = xai_grok_sampler::SamplingClient::new(bound_runtime.sampler_config).map_err(
            |error| {
                tracing::warn!(
                    %error,
                    aux_model = %slug,
                    "provider-bound auxiliary sampler construction failed"
                );
                self.to_acp_error(error)
            },
        )?;
        Ok(Some((client, route)))
    }

    /// Prepare an auxiliary sampler with provider-safe fallback. An explicit
    /// model is used only when catalog routing can construct that model's own
    /// provider client; otherwise both client and model fall back together to
    /// the active session route.
    pub(super) async fn prepare_aux_sampling_client(
        &self,
        requested_model: Option<&str>,
        purpose: &'static str,
    ) -> Result<
        (
            xai_grok_sampler::SamplingClient,
            crate::session::provider::ProviderModelRoute,
        ),
        acp::Error,
    > {
        if let Some(slug) = requested_model {
            if let Some(routed) = self.resolve_routed_aux_sampling_client(slug).await? {
                return Ok(routed);
            }
            tracing::warn!(
                purpose,
                requested_model = %slug,
                "auxiliary model unavailable with provider-owned credentials; using session model"
            );
        }

        let (client, config) = self.prepare_bound_chat_completion(false).await?;
        let route = crate::session::provider::ProviderModelRoute {
            provider: config.provider,
            model: config.model,
        };
        Ok((client, route))
    }
    #[tracing::instrument(
        name = "session.prepare_chat_completion",
        skip_all,
        fields(force_http1)
    )]
    pub(super) async fn prepare_chat_completion(
        &self,
        force_http1: bool,
    ) -> Result<xai_grok_sampler::SamplingClient, acp::Error> {
        self.prepare_bound_chat_completion(force_http1)
            .await
            .map(|(client, _)| client)
    }

    /// Build the provider-bound compaction route, optionally using the prior
    /// Codex model as the primary route after a model downshift. The active
    /// route is retained as a same-provider fallback, matching Codex CLI's
    /// previous-model compaction contract without mutating live chat state.
    pub(super) async fn prepare_compaction_sampling_plan(
        &self,
        previous_model: Option<&crate::session::compaction_config::PreviousModelInfo>,
    ) -> Result<
        (
            xai_grok_sampler::SamplingClient,
            xai_grok_sampler::SamplerConfig,
            Option<(
                xai_grok_sampler::SamplingClient,
                xai_grok_sampler::SamplerConfig,
            )>,
        ),
        acp::Error,
    > {
        self.refresh_token_if_expired().await;
        let active_config = self.reconstruct_full_config().await?;
        self.mark_codex_auxiliary_usage_incomplete(active_config.provider)
            .await;
        let (primary_config, fallback_config) =
            self.compaction_sampling_configs(active_config, previous_model);
        let primary_client = xai_grok_sampler::SamplingClient::new(primary_config.clone())
            .map_err(|error| self.to_acp_error(error))?;
        let fallback = match fallback_config {
            Some(config) => {
                let client = xai_grok_sampler::SamplingClient::new(config.clone())
                    .map_err(|error| self.to_acp_error(error))?;
                Some((client, config))
            }
            None => None,
        };
        Ok((primary_client, primary_config, fallback))
    }

    /// Select compaction model configs from one provider-bound credential
    /// snapshot. Only a same-provider Codex switch may use the previous model;
    /// every other path keeps the active session model and has no fallback.
    pub(super) fn compaction_sampling_configs(
        &self,
        active_config: xai_grok_sampler::SamplerConfig,
        previous_model: Option<&crate::session::compaction_config::PreviousModelInfo>,
    ) -> (
        xai_grok_sampler::SamplerConfig,
        Option<xai_grok_sampler::SamplerConfig>,
    ) {
        let Some(previous_model) = previous_model.filter(|previous| {
            let comp_hash_changed = previous
                .comp_hash
                .as_deref()
                .filter(|hash| !hash.is_empty())
                .zip(
                    active_config
                        .comp_hash
                        .as_deref()
                        .filter(|hash| !hash.is_empty()),
                )
                .is_some_and(|(previous, current)| previous != current);
            active_config.provider.is_openai_codex()
                && previous.provider == active_config.provider
                && (previous.model_slug != active_config.model || comp_hash_changed)
        }) else {
            return (active_config, None);
        };

        let mut previous_config = active_config.clone();
        previous_config.model = previous_model.model_slug.clone();
        previous_config.context_window = previous_model.context_window;
        previous_config
            .comp_hash
            .clone_from(&previous_model.comp_hash);
        if self
            .models_manager
            .model_in_catalog_for_provider(previous_config.provider, &previous_config.model)
        {
            previous_config.reasoning_effort = previous_config
                .reasoning_effort
                .and_then(|effort| {
                    self.models_manager.resolve_reasoning_effort_for_provider(
                        previous_config.provider,
                        &previous_config.model,
                        effort,
                    )
                })
                .or_else(|| {
                    self.models_manager
                        .model_default_reasoning_effort_for_provider(
                            previous_config.provider,
                            &previous_config.model,
                        )
                });
            previous_config.service_tier =
                previous_config.service_tier.as_deref().and_then(|tier| {
                    self.models_manager.resolve_service_tier_for_provider(
                        previous_config.provider,
                        &previous_config.model,
                        tier,
                    )
                });
        }

        (previous_config, Some(active_config))
    }

    /// Construct an auxiliary/compaction client and return the exact bound
    /// config used for that client. This avoids a second credential snapshot
    /// between route decisions and sampler construction.
    pub(super) async fn prepare_bound_chat_completion(
        &self,
        force_http1: bool,
    ) -> Result<
        (
            xai_grok_sampler::SamplingClient,
            xai_grok_sampler::SamplerConfig,
        ),
        acp::Error,
    > {
        self.refresh_token_if_expired().await;
        let mut full_config = self.reconstruct_full_config().await?;
        self.mark_codex_auxiliary_usage_incomplete(full_config.provider)
            .await;
        full_config.force_http1 = force_http1;
        let sampling_client = xai_grok_sampler::SamplingClient::new(full_config.clone())
            .map_err(|e| self.to_acp_error(e))?;
        Ok((sampling_client, full_config))
    }

    /// Auxiliary and compaction samplers do not currently fold their response
    /// usage into the main ledger. Mark Codex totals incomplete as soon as such
    /// a client is prepared so the hypothetical API total fails closed instead
    /// of presenting a known lower bound as complete.
    pub(super) async fn mark_codex_auxiliary_usage_incomplete(
        &self,
        provider: xai_grok_sampling_types::ProviderId,
    ) {
        if provider == xai_grok_sampling_types::ProviderId::OpenAiCodex {
            let _ = self
                .chat_state_handle
                .mark_usage_incomplete(false, true)
                .await;
            self.persist_current_session_usage().await;
        }
    }
    /// Push a fresh `SamplerConfig` into the per-session sampler actor
    /// before each turn. Mirrors `prepare_chat_completion`'s
    /// auth-refresh + config rebuild, but routes the result to the
    /// `xai-grok-sampler` instead of constructing a new
    /// `OaiCompatClient`.
    ///
    /// Behaviour parity: we run the same `refresh_token_if_expired()`
    /// and `reconstruct_full_config()` so the sampler picks up any
    /// newly issued session token. The previous client cache inside
    /// the sampler actor is invalidated automatically by
    /// `update_config`.
    pub(crate) async fn prepare_sampler_for_turn(&self) -> Result<(), acp::Error> {
        self.refresh_token_if_expired().await;
        let mut sampler_config = self.reconstruct_full_config().await?;
        sampler_config.idle_timeout_secs = Some(self.inference_idle_timeout.as_secs());
        self.sampler_handle.update_config(sampler_config);
        Ok(())
    }
    fn log_terminal_failure(
        &self,
        request_provider: xai_grok_sampling_types::ProviderId,
        error_type: &str,
        status_code: Option<u16>,
        message: &str,
    ) {
        // The shell AuthManager contains xAI state only. Never label a Codex
        // or custom-provider failure with whatever xAI login happens to exist
        // alongside it.
        let auth = (request_provider == xai_grok_sampling_types::ProviderId::Xai)
            .then(|| {
                self.auth_manager
                    .as_ref()
                    .and_then(|am| am.current_or_expired())
            })
            .flatten();
        let auth_mode = if request_provider == xai_grok_sampling_types::ProviderId::Xai {
            auth.as_ref().map(|auth| format!("{:?}", auth.auth_mode))
        } else {
            Some(request_provider.as_str().to_owned())
        };
        let credential_present = (request_provider == xai_grok_sampling_types::ProviderId::Xai)
            .then(|| auth.as_ref().is_some_and(|auth| !auth.key.is_empty()));
        let reauthable = is_reauthable_failure(Some(error_type), message);
        xai_grok_telemetry::unified_log::warn(
            "turn.terminal_failure",
            Some(self.session_info.id.0.as_ref()),
            Some(serde_json::json!(
                { "error_type" : error_type, "status_code" : status_code,
                "reauthable" : reauthable, "provider" : request_provider.as_str(),
                "auth_mode" : auth_mode, "credential_present" : credential_present,
                "expires_at" : auth
                .as_ref().and_then(| a | a.expires_at.map(| e | e.to_rfc3339())),
                "message" : crate ::util::truncate(message, 300), }
            )),
        );
    }
    pub(crate) async fn handle_sampling_failure(
        self: &Arc<Self>,
        error: xai_grok_sampler::SamplingErrorInfo,
    ) -> Result<SamplerFailureRecovery, acp::Error> {
        self.handle_sampling_failure_with_codex_policy(error, true)
            .await
    }

    async fn handle_sampling_failure_with_codex_policy(
        self: &Arc<Self>,
        error: xai_grok_sampler::SamplingErrorInfo,
        _allow_codex_recovery: bool,
    ) -> Result<SamplerFailureRecovery, acp::Error> {
        use xai_grok_sampler::SamplingErrorKind;
        if self.should_compact_on_error(&error).await {
            let cw = error
                .model_metadata
                .as_ref()
                .and_then(|m| m.context_window)
                .expect("should_compact_on_error guarantees context_window");
            {
                let total_tokens = self.chat_state_handle.get_estimated_total_tokens().await;
                let percentage = xai_token_estimation::usage_percentage_u8(total_tokens, cw);
                if let Some(mut cfg) = self.chat_state_handle.get_sampling_config().await
                    && let Some(new_cw) = std::num::NonZeroU64::new(cw)
                    && self.compaction.context_window_override.is_none()
                {
                    cfg.context_window = new_cw;
                    self.chat_state_handle.update_sampling_config(cfg);
                }
                let trigger_info = compaction::AutoCompactTriggerInfo {
                    tokens_used: total_tokens,
                    context_window: cw,
                    percentage,
                };
                self.run_compact_only(trigger_info).await?;
                return Ok(SamplerFailureRecovery::CompactAndResubmit);
            }
        }
        let detailed_message = error.message.clone();
        let request_provider = self
            .chat_state_handle
            .get_sampling_config()
            .await
            .map(|config| config.provider)
            .unwrap_or_default();
        if matches!(error.kind, SamplingErrorKind::Api)
            && error.status_code == Some(400)
            && error.message.contains("encrypted_content")
        {
            self.signals_handle()
                .record_error_typed("encrypted_content_mismatch");
            let friendly = "This session's conversation history is incompatible \
                            with the current model. Please start a new session."
                .to_string();
            self.log_terminal_failure(
                request_provider,
                "encrypted_content_mismatch",
                error.status_code,
                &friendly,
            );
            self.send_xai_notification(XaiSessionUpdate::RetryState(
                crate::extensions::notification::RetryState::Failed {
                    error_type: "encrypted_content_mismatch".to_string(),
                    message: friendly.clone(),
                },
            ))
            .await;
            return Err(acp::Error::invalid_params().data(friendly));
        }
        if matches!(error.kind, SamplingErrorKind::RateLimited) {
            self.log_terminal_failure(
                request_provider,
                "rate_limited",
                error.status_code,
                &detailed_message,
            );
            self.send_xai_notification(XaiSessionUpdate::RetryState(
                crate::extensions::notification::RetryState::Exhausted {
                    attempts: 0,
                    reason: detailed_message.clone(),
                    is_rate_limited: true,
                },
            ))
            .await;
            let acp_err = acp::Error::new(
                crate::sampling::error::RATE_LIMITED_ERROR_CODE,
                "Rate limited".to_string(),
            )
            .data(detailed_message);
            return Err(acp_err);
        }
        let has_provider_owned_auth = request_provider.is_openai_codex()
            || request_provider.is_kimi_code()
            || request_provider.is_zai_coding_plan();
        if matches!(error.kind, SamplingErrorKind::Auth) && has_provider_owned_auth {
            tracing::warn!(
                session_id = %self.session_info.id.0,
                provider = %request_provider,
                "provider auth recovery exhausted; refusing xAI auth fallback"
            );
        }
        let auth_recovery_eligible =
            matches!(error.kind, SamplingErrorKind::Auth) && !has_provider_owned_auth && {
                let (model_id, base_url) = self
                    .chat_state_handle
                    .get_sampling_config()
                    .await
                    .map(|c| (c.model, c.base_url))
                    .unwrap_or_default();
                let gate = self.auth_gate(&model_id, &base_url);
                let eligible = gate.active();
                self.log_auth_gate_unknown("handle_sampling_failure", gate, &base_url);
                if !eligible {
                    tracing::warn!(
                        session_id = % self.session_info.id.0, is_session_based = gate
                        .is_session_based, model_byok = gate.model_byok.as_str(),
                        endpoint_is_first_party = gate.endpoint_is_first_party,
                        "auth recovery: sampler 401 not refreshable (api-key auth) — surfacing 401",
                    );
                    xai_grok_telemetry::unified_log::warn(
                        "auth recovery: sampler 401 not eligible (api-key auth)",
                        Some(self.session_info.id.0.as_ref()),
                        Some(serde_json::json!(
                            { "kind" : error.kind.as_str(), "status_code" : error
                            .status_code, "is_session_based" : gate.is_session_based,
                            "model_byok" : gate.model_byok.as_str(),
                            "endpoint_is_first_party" : gate.endpoint_is_first_party, }
                        )),
                    );
                }
                eligible
            };
        if !matches!(error.kind, SamplingErrorKind::Auth) && error.status_code == Some(401) {
            xai_grok_telemetry::unified_log::warn(
                "auth recovery: sampler 401 not eligible (non-auth error kind)",
                Some(self.session_info.id.0.as_ref()),
                Some(serde_json::json!(
                    { "kind" : error.kind.as_str(), "status_code" : error
                    .status_code, }
                )),
            );
        }
        if auth_recovery_eligible
            && crate::auth::devbox_login::is_devbox_environment()
            && let Some(ref am) = self.auth_manager
        {
            match am.try_devbox_recovery().await {
                Ok(auth) => {
                    tracing::info!(
                        session_id = % self.session_info.id.0,
                        has_user_id = !auth.user_id.is_empty(),
                        "auth recovery: sampler 401, devbox re-mint, retrying"
                    );
                    self.prepare_sampler_for_turn().await?;
                    return Ok(SamplerFailureRecovery::RefreshAuthAndResubmit);
                }
                Err(e) => {
                    tracing::warn!(
                        session_id = % self.session_info.id.0, error = % e,
                        "auth recovery: sampler 401, devbox re-mint failed"
                    );
                    xai_grok_telemetry::unified_log::warn(
                        "auth recovery: sampler 401, devbox re-mint failed",
                        Some(self.session_info.id.0.as_ref()),
                        Some(serde_json::json!({ "error" : format!("{e}") })),
                    );
                }
            }
        }
        if auth_recovery_eligible && let Some(ref am) = self.auth_manager {
            if am
                .try_recover_unauthorized(crate::auth::recovery::RecoverySource::Turn)
                .await
            {
                tracing::info!(
                    session_id = % self.session_info.id.0,
                    "auth recovery: sampler 401, recovered, retrying"
                );
                xai_grok_telemetry::unified_log::info(
                    "auth recovery: sampler 401, recovered, retrying",
                    Some(self.session_info.id.0.as_ref()),
                    None,
                );
                self.prepare_sampler_for_turn().await?;
                return Ok(SamplerFailureRecovery::RefreshAuthAndResubmit);
            }
            tracing::warn!(
                session_id = % self.session_info.id.0,
                "auth recovery: sampler 401, refresh failed"
            );
            xai_grok_telemetry::unified_log::warn(
                "auth recovery: sampler 401, refresh failed",
                Some(self.session_info.id.0.as_ref()),
                None,
            );
        }
        if matches!(error.kind, SamplingErrorKind::IdleTimeout) {
            self.signals_handle().record_idle_timeout();
        }
        if matches!(error.kind, SamplingErrorKind::EmptyResponse) {
            if let Some(ref ctx) = error.empty_response_context {
                tracing::warn!(
                    empty_response = true, empty_reason = ctx.reason.as_str(),
                    had_reasoning = ctx.had_reasoning, content_len = ctx.content_len,
                    tool_call_count = ctx.tool_call_count, completion_tokens = ctx
                    .completion_tokens.unwrap_or(0), reasoning_tokens = ctx
                    .reasoning_tokens.unwrap_or(0), finish_reason = ctx
                    .finish_reason_str(), first_choice_seen = ctx.first_choice_seen,
                    model = % ctx.model,
                    "empty response after retries exhausted: {reason}", reason = ctx
                    .reason,
                );
                {
                    let mut cap = self.streaming_turn_capture.lock();
                    cap.reasoning_tokens = ctx.reasoning_tokens;
                    cap.completion_tokens = ctx.completion_tokens;
                    cap.finish_reason = ctx.finish_reason.clone();
                    cap.empty_reason = Some(ctx.reason.as_str().to_owned());
                }
            }
            self.signals_handle().record_error_typed("empty_response");
        }
        let xai_auth_mode = self
            .auth_manager
            .as_ref()
            .and_then(|am| am.current())
            .map(|a| a.auth_mode)
            .unwrap_or(crate::auth::AuthMode::ApiKey);
        let auth_mode_str = if request_provider == xai_grok_sampling_types::ProviderId::Xai {
            format!("{xai_auth_mode:?}")
        } else {
            request_provider.as_str().to_owned()
        };
        let client_version = xai_grok_version::VERSION;
        if request_provider == xai_grok_sampling_types::ProviderId::Xai
            && xai_auth_mode == crate::auth::AuthMode::WebLogin
        {
            let msg = format!(
                "{detailed_message}\n\n\
                 You are using a deprecated authentication method (WebLogin).\n\
                 This auth method is no longer supported and will cause errors.\n\n\
                 To fix: run `grok logout` then `grok login` to re-authenticate with OAuth2.\n\n\
                 Version: {client_version}"
            );
            self.log_terminal_failure(request_provider, "legacy_auth", error.status_code, &msg);
            self.send_xai_notification(XaiSessionUpdate::RetryState(
                crate::extensions::notification::RetryState::Failed {
                    error_type: "legacy_auth".to_string(),
                    message: msg.clone(),
                },
            ))
            .await;
            return Err(acp::Error::internal_error().data(msg));
        }
        let is_model_404 =
            error.status_code == Some(404) && detailed_message.contains("does not exist");
        let is_auth_401 =
            error.status_code == Some(401) || matches!(error.kind, SamplingErrorKind::Auth);
        let detailed_message = if is_model_404 || is_auth_401 {
            let current_model = self
                .chat_state_handle
                .get_sampling_config()
                .await
                .map(|c| c.model)
                .unwrap_or_else(|| "unknown".to_string());
            let available: Vec<String> = self
                .models_manager
                .models()
                .values()
                .map(|m| m.model.clone())
                .collect();
            let mut msg = format!("{detailed_message}\n");
            msg.push_str(&format!("\n  Model:     {current_model}"));
            msg.push_str(&format!("\n  Auth:      {auth_mode_str}"));
            msg.push_str(&format!("\n  Version:   {client_version}"));
            if available.is_empty() {
                msg.push_str("\n  Available: (none)");
            } else {
                msg.push_str(&format!("\n  Available: {}", available.join(", ")));
            }
            if is_model_404 && !available.iter().any(|m| m == &current_model) {
                msg.push_str(&format!(
                    "\n\n  '{}' is not in your available models.",
                    current_model
                ));
                msg.push_str("\n  Switch models with /model or start a new session.");
            }
            if is_auth_401 {
                if request_provider.is_openai_codex() {
                    msg.push_str(
                        "\n\n  Re-authenticate: run `grok logout --provider openai-codex` then \
                         `grok login --provider openai-codex`.",
                    );
                } else if request_provider.is_kimi_code() {
                    msg.push_str("\n\n  Re-authenticate: run `grok login --provider kimi-code`.");
                } else if request_provider.is_zai_coding_plan() {
                    msg.push_str(
                        "\n\n  Re-authenticate: run `grok login --provider zai-coding-plan`.",
                    );
                }
            }
            msg
        } else {
            detailed_message
        };
        let error_type = if xai_grok_sampling_types::is_context_length_error(&error.message) {
            "context_length"
        } else {
            error.kind.as_str()
        };
        self.log_terminal_failure(
            request_provider,
            error_type,
            error.status_code,
            &detailed_message,
        );
        self.send_xai_notification(XaiSessionUpdate::RetryState(
            crate::extensions::notification::RetryState::Failed {
                error_type: error_type.to_string(),
                message: detailed_message.clone(),
            },
        ))
        .await;
        Err(
            acp::Error::internal_error().data(crate::sampling::error::terminal_error_data(
                detailed_message,
                error.status_code,
                error.kind,
            )),
        )
    }
    /// Drive a single turn through the sampler-based path.
    ///
    /// Calls `prepare_sampler_for_turn` first (auth refresh + config
    /// push), then submits via `SamplerHandle::submit_and_collect` and
    /// returns:
    /// * `Ok(SamplerTurnOutcome::Response(_))` - model responded.
    /// * `Ok(SamplerTurnOutcome::CompactAndResubmit)` - compaction
    ///    ran, the outer turn loop should `continue`.
    /// * `Ok(SamplerTurnOutcome::RefreshAuthAndResubmit)` - auth 401
    ///    recovery succeeded, credentials refreshed, retry once.
    /// * `Err(acp::Error)` - terminal failure already reported via
    ///    `send_xai_notification(RetryState::Failed)`.
    pub(crate) async fn run_turn_via_sampler(
        self: &Arc<Self>,
        mut request: ConversationRequest,
        allow_codex_auth_recovery: bool,
    ) -> Result<SamplerTurnOutcome, acp::Error> {
        self.prepare_sampler_for_turn().await?;
        // Resolve against the active provider/model and the live authenticated
        // catalog immediately before handing this request-copy to Responses.
        // Every loop iteration (including after a model/provider switch) gets a
        // fresh decision; the chat-state conversation is never rewritten.
        if let Some(sampling) = self.chat_state_handle.get_sampling_config().await {
            request.image_input_capability = self
                .models_manager
                .codex_image_input_capability_for_request(sampling.provider, &sampling.model);
        }
        let stream_drained_rx = {
            let (tx, rx) = tokio::sync::oneshot::channel();
            *self.turn_stream_drained.lock() = Some(tx);
            rx
        };
        let request_id = xai_grok_sampler::RequestId::random();
        let request_id_str = request_id.as_str().to_string();
        match self
            .sampler_handle
            .submit_and_collect(request_id, request)
            .await
        {
            Ok((response, metrics)) => {
                let span = tracing::Span::current();
                span.record("request_id", request_id_str.as_str());
                if let Some(ttft) = metrics.time_to_first_token_ms {
                    span.record("ttft_ms", ttft as i64);
                }
                if metrics.attempts > 0 {
                    span.record("attempt", i64::from(metrics.attempts));
                }
                if tokio::time::timeout(std::time::Duration::from_secs(5), stream_drained_rx)
                    .await
                    .is_err()
                {
                    self.turn_stream_drained.lock().take();
                    tracing::warn!(
                        "stream-drain barrier timed out; proceeding to emit tool \
                         calls (eventId ordering may be imperfect this turn)"
                    );
                }
                Ok(SamplerTurnOutcome::Response(
                    Box::new(response),
                    Box::new(metrics),
                ))
            }
            Err(rich_err) => {
                self.turn_stream_drained.lock().take();
                let info = xai_grok_sampler::SamplingErrorInfo::from(&rich_err);
                match self
                    .handle_sampling_failure_with_codex_policy(info, allow_codex_auth_recovery)
                    .await?
                {
                    SamplerFailureRecovery::CompactAndResubmit => {
                        Ok(SamplerTurnOutcome::CompactAndResubmit)
                    }
                    SamplerFailureRecovery::RefreshAuthAndResubmit => {
                        Ok(SamplerTurnOutcome::RefreshAuthAndResubmit)
                    }
                }
            }
        }
    }
    /// Proactively refresh the auth token if near expiry.
    pub(super) async fn refresh_token_if_expired(&self) {
        let (provider, current_model_id, base_url) = self
            .chat_state_handle
            .get_sampling_config()
            .await
            .map(|selected| (selected.provider, selected.model, selected.base_url))
            .unwrap_or_else(|| {
                (
                    xai_grok_sampling_types::ProviderId::Xai,
                    String::new(),
                    String::new(),
                )
            });
        // Subscription-provider refresh belongs exclusively to the provider
        // binder, which runs after reconstructing the request config. Do not
        // evaluate the global xAI or generic static-key refresh paths for a
        // Codex, Kimi, or Z.AI Coding Plan session.
        if provider.is_openai_codex() || provider.is_kimi_code() || provider.is_zai_coding_plan() {
            return;
        }
        if provider == xai_grok_sampling_types::ProviderId::Xai {
            if let Some(ref am) = self.auth_manager {
                let creds = self.chat_state_handle.get_credentials().await;
                if self.auth_gate(&current_model_id, &base_url).active()
                    && let Ok(key) = am.get_valid_token().await
                {
                    if creds.api_key.as_deref() != Some(&key)
                        || creds.provider != Some(xai_grok_sampling_types::ProviderId::Xai)
                    {
                        let mut creds = creds;
                        creds.provider = Some(xai_grok_sampling_types::ProviderId::Xai);
                        creds.api_key = Some(key);
                        self.chat_state_handle.update_credentials(creds);
                    }
                    return;
                }
            } else {
                xai_grok_telemetry::unified_log::debug(
                    "token refresh skipped: no auth manager",
                    Some(self.session_info.id.0.as_ref()),
                    None,
                );
            }
        }
        use crate::auth::{is_jwt_expired_or_near, parse_jwt_expiration};
        const REFRESH_THRESHOLD: chrono::Duration = chrono::Duration::minutes(5);
        let creds = self.chat_state_handle.get_credentials().await;
        let current_key = creds.api_key;
        let Some(ref key) = current_key else { return };
        if !is_jwt_expired_or_near(key, REFRESH_THRESHOLD) {
            if let Some(exp) = parse_jwt_expiration(key) {
                let remaining_secs = (exp - chrono::Utc::now()).num_seconds();
                tracing::debug!(
                    model = % current_model_id, remaining_secs,
                    "JWT token valid, no refresh needed"
                );
            } else {
                tracing::debug!(
                    model = % current_model_id, credential_present = !key.is_empty(),
                    "Token is not a JWT, expiry-based refresh not applicable"
                );
            }
            return;
        }
        let remaining_secs =
            parse_jwt_expiration(key).map_or(0, |exp| (exp - chrono::Utc::now()).num_seconds());
        tracing::info!(
            model = % current_model_id, remaining_secs,
            "JWT near expiry, refreshing from config.toml"
        );
        let Some(new_key) = self.reload_api_key_from_config(&current_model_id) else {
            return;
        };
        if key == &new_key {
            tracing::warn!(
                model = % current_model_id,
                "Config.toml returned same token (not yet rotated by external process?)"
            );
            return;
        }
        let new_remaining_secs = parse_jwt_expiration(&new_key)
            .map_or(0, |exp| (exp - chrono::Utc::now()).num_seconds());
        tracing::info!(
            model = % current_model_id, new_remaining_secs,
            credential_present = !new_key.is_empty(),
            "Refreshed API token from config.toml"
        );
        let mut creds = self.chat_state_handle.get_credentials().await;
        creds.provider = Some(provider);
        creds.api_key = Some(new_key);
        self.chat_state_handle.update_credentials(creds);
    }
    fn reload_api_key_from_config(&self, current_model_id: &str) -> Option<String> {
        let raw_config = crate::config::load_effective_config()
            .map_err(|e| tracing::warn!(error = % e, "Failed to reload config"))
            .ok()?;
        let config = crate::agent::config::Config::new_from_toml_cfg(&raw_config)
            .map_err(|e| tracing::warn!(error = % e, "Failed to parse reloaded config.toml"))
            .ok()?;
        let config_model = config
            .config_models
            .iter()
            .find(|(k, v)| v.model.as_deref().unwrap_or(k.as_str()) == current_model_id)
            .map(|(_, v)| v);
        let Some(model) = config_model else {
            tracing::warn!(
                model = % current_model_id, available = ? config.config_models.keys()
                .collect::< Vec < _ >> (), "Model not found in config.toml [model.*]"
            );
            return None;
        };
        let key = crate::agent::config::first_own_credential(
            model.api_key.as_deref(),
            model.env_key.as_ref(),
        );
        if key.is_none() {
            tracing::warn!(
                model = % current_model_id, env_key = ? model.env_key,
                "No api_key or env_key resolved for model"
            );
        }
        key
    }
    /// Propagate the model-reported token usage from a turn response into
    /// chat state, the per-prompt usage ledger, and per-turn signals.
    ///
    /// This is the only place per-turn `total_tokens` is refreshed in the
    /// post-sampler-refactor path; without it `state.total_tokens` would
    /// stay frozen at the `estimate_conversation_tokens` seed from
    /// `ChatState::new`, freezing `/context` and corrupting the resume
    /// restore that reads `meta.totalTokens` from `updates.jsonl`.
    /// Resetting `estimated_tokens_since_model = 0` here also keeps the
    /// preflight-overflow guard accurate against the next turn's
    /// tool-result deltas.
    pub(crate) async fn record_response_token_usage(
        &self,
        response: &ConversationResponse,
        api_duration_ms: Option<u64>,
    ) {
        if let Some(ref u) = response.usage {
            self.chat_state_handle
                .record_token_usage(u64::from(u.total_tokens));
            self.chat_state_handle.record_last_turn_usage(u.clone());
            self.chat_state_handle.record_model_call_usage(
                response.assistant().and_then(|a| a.model_id.clone()),
                u.clone(),
                api_duration_ms,
                response.cost_usd_ticks,
            );
            self.signals_handle()
                .record_token_usage(u.completion_tokens, u.reasoning_tokens);
            self.persist_current_session_usage().await;
        }
    }
    pub(super) async fn record_assistant_response(&self, assistant_item: ConversationItem) {
        self.signals_handle().record_assistant_message();
        if let ConversationItem::Assistant(ref a) = assistant_item {
            tracing::info!(
                model_id = ? a.model_id, "DEBUG record_assistant_response model_id"
            );
        }
        if let ConversationItem::Assistant(ref a) = assistant_item
            && let Some(first_call) = a.tool_calls.first()
        {
            tracing::info!("Assistant requested tool call: {}", first_call.id);
        }
        self.chat_state_handle
            .push_assistant_response(assistant_item);
    }
}
