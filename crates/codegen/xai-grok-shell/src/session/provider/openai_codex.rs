//! Fail-closed OpenAI Codex session binding.
//!
//! The binder in this module is the only session-layer constructor that may
//! turn a Codex sampler configuration into a runnable sampler/tool pair.  It
//! attests the persisted credential record before either runtime is exposed,
//! and both runtimes share the same [`CodexAuthManager`].

use std::sync::Arc;

use xai_grok_sampler::{AuthScheme, SamplerConfig};
use xai_grok_sampling_types::{
    ApiBackend, CredentialBinding, CredentialSourceId, KIMI_CODE_BASE_URL, OPENAI_CODEX_BASE_URL,
    OPENAI_CODEX_RESPONSES_LITE_HEADER, ProviderId, ZAI_CODING_PLAN_BASE_URL,
};
use xai_grok_tools::types::SharedApiKeyProvider;

use crate::auth::codex::{
    CodexAuthError, CodexAuthManager, shared_api_key_provider, shared_sampler_request_auth,
};
use crate::auth::kimi_code::{
    KimiCodeAuthError, KimiCodeCredentialStore, shared_sampler_request_auth as kimi_sampler_auth,
    shared_tool_auth_provider as kimi_tool_auth,
};
use crate::auth::zai_coding_plan::{
    ZaiCodingPlanAuthError, ZaiCodingPlanCredentialStore,
    shared_sampler_request_auth as zai_sampler_auth, shared_tool_auth_provider as zai_tool_auth,
};

/// Provider-qualified model identity carried alongside every bound auxiliary
/// sampler.  Keeping the pair together prevents a caller from accidentally
/// sending an auxiliary model slug to the active session's other provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProviderModelRoute {
    pub(crate) provider: ProviderId,
    pub(crate) model: String,
}

/// A complete session runtime produced from one credential attestation.
///
/// For Codex, `sampler_config.request_auth` and `api_key_provider` are backed by
/// the same manager and credential record. The supplied generic tool provider
/// is retained only for xAI; custom providers receive no xAI-owned tool auth.
#[derive(Clone)]
pub(crate) struct BoundProviderRuntime {
    pub(crate) sampler_config: SamplerConfig,
    pub(crate) api_key_provider: Option<SharedApiKeyProvider>,
    pub(crate) route: ProviderModelRoute,
}

impl std::fmt::Debug for BoundProviderRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundProviderRuntime")
            .finish_non_exhaustive()
    }
}

/// Safe, provider-scoped binding failures.  No variant contains credential or
/// account material.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ProviderBindingError {
    #[error(transparent)]
    CodexAuth(#[from] CodexAuthError),
    #[error(transparent)]
    KimiCodeAuth(#[from] KimiCodeAuthError),
    #[error(transparent)]
    ZaiCodingPlanAuth(#[from] ZaiCodingPlanAuthError),
    #[error("restored OpenAI Codex credential binding is invalid")]
    InvalidRestoredBinding,
    #[error("restored Kimi Code credential binding is invalid")]
    InvalidKimiRestoredBinding,
    #[error("restored Z.AI Coding Plan credential binding is invalid")]
    InvalidZaiRestoredBinding,
    #[error("the restored Kimi Code API-key record changed; rebuild the provider session")]
    KimiCredentialChanged,
    #[error("the restored Z.AI Coding Plan API-key record changed; rebuild the provider session")]
    ZaiCredentialChanged,
    #[error("the restored OpenAI Codex account changed; rebuild the provider session")]
    AccountChanged,
    #[error("the restored OpenAI Codex credential generation moved backwards")]
    GenerationRegressed,
}

/// Bind a sampler configuration and its tool authentication as one runtime.
///
/// This is deliberately fallible even though ordinary xAI/custom sampler
/// configs pass through: a Codex logout, account switch, malformed restored
/// binding, or generation rollback is rejected before a sampling client/actor
/// or provider-authenticated tool client can be constructed.
pub(crate) async fn bind_provider_runtime(
    mut sampler_config: SamplerConfig,
    generic_api_key_provider: Option<SharedApiKeyProvider>,
) -> Result<BoundProviderRuntime, ProviderBindingError> {
    // A named rotating helper belongs exclusively to a Custom route. Restored
    // or hand-built first-party sampler state cannot reinterpret that bearer.
    if sampler_config.provider != ProviderId::Custom
        && sampler_config.credential_source == CredentialSourceId::RotatingAuthProvider
    {
        sampler_config.api_key = None;
        sampler_config.credential_source = CredentialSourceId::Unspecified;
        sampler_config.credential_binding = None;
    }
    if sampler_config.provider.is_kimi_code() {
        return bind_kimi_code(sampler_config);
    }
    if sampler_config.provider.is_zai_coding_plan() {
        return bind_zai_coding_plan(sampler_config);
    }
    if !sampler_config.provider.is_openai_codex() {
        let route = ProviderModelRoute {
            provider: sampler_config.provider,
            model: sampler_config.model.clone(),
        };
        // The generic provider is backed by the global xAI auth manager. A
        // custom endpoint owns arbitrary credential semantics and must never
        // inherit that provider merely because a caller supplied one.
        let api_key_provider = (sampler_config.provider == ProviderId::Xai)
            .then_some(generic_api_key_provider)
            .flatten();
        if sampler_config.provider == ProviderId::Custom {
            let owns_custom_credential = matches!(
                sampler_config.credential_source,
                CredentialSourceId::StaticApiKey
                    | CredentialSourceId::External
                    | CredentialSourceId::RotatingAuthProvider
            );
            if !owns_custom_credential {
                sampler_config.api_key = None;
                sampler_config.credential_source = CredentialSourceId::Unspecified;
            }
            sampler_config.credential_binding = None;
            sampler_config.bearer_resolver = None;
            sampler_config.request_auth = None;
            sampler_config.attribution_callback = None;
            sampler_config.deployment_id = None;
            sampler_config.user_id = None;
            sampler_config.compactions_remaining = None;
            sampler_config.compaction_at_tokens = None;
            sampler_config.extra_headers.retain(|name, _| {
                !name.eq_ignore_ascii_case("x-xai-token-auth")
                    && !name.eq_ignore_ascii_case("x-authenticateresponse")
            });
        }
        return Ok(BoundProviderRuntime {
            sampler_config,
            api_key_provider,
            route,
        });
    }

    let manager = Arc::new(CodexAuthManager::new(&crate::util::grok_home::grok_home())?);
    bind_openai_codex_with_manager(sampler_config, manager).await
}

fn bind_kimi_code(
    mut sampler_config: SamplerConfig,
) -> Result<BoundProviderRuntime, ProviderBindingError> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = KimiCodeCredentialStore::new(&grok_home);
    let (credentials, current) =
        crate::auth::kimi_code::current_credentials_and_binding(&grok_home)?;
    if let Some(expected) = restored_kimi_binding(sampler_config.credential_binding.as_ref())?
        && (!expected.same_record(&current) || current.generation < expected.generation)
    {
        return Err(ProviderBindingError::KimiCredentialChanged);
    }

    let backend = sampler_config.api_backend.clone();
    if !matches!(backend, ApiBackend::ChatCompletions | ApiBackend::Messages) {
        return Err(ProviderBindingError::InvalidKimiRestoredBinding);
    }
    sampler_config.provider = ProviderId::KimiCode;
    sampler_config.credential_source = CredentialSourceId::KimiCodeApiKey;
    sampler_config.credential_binding = Some(current.clone());
    sampler_config.api_key = None;
    sampler_config.base_url = KIMI_CODE_BASE_URL.to_owned();
    sampler_config.auth_scheme = if backend == ApiBackend::Messages {
        AuthScheme::XApiKey
    } else {
        AuthScheme::Bearer
    };
    sampler_config.extra_headers.clear();
    sampler_config.attribution_callback = None;
    sampler_config.bearer_resolver = None;
    sampler_config.request_auth = Some(kimi_sampler_auth(
        store.clone(),
        credentials.clone(),
        backend,
        current.clone(),
    ));
    sampler_config.deployment_id = None;
    sampler_config.user_id = None;
    sampler_config.compactions_remaining = None;
    sampler_config.compaction_at_tokens = None;

    let route = ProviderModelRoute {
        provider: ProviderId::KimiCode,
        model: sampler_config.model.clone(),
    };
    Ok(BoundProviderRuntime {
        sampler_config,
        api_key_provider: Some(kimi_tool_auth(store, credentials, current)),
        route,
    })
}

fn bind_zai_coding_plan(
    mut sampler_config: SamplerConfig,
) -> Result<BoundProviderRuntime, ProviderBindingError> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = ZaiCodingPlanCredentialStore::new(&grok_home);
    let (credentials, current) =
        crate::auth::zai_coding_plan::current_credentials_and_binding(&grok_home)?;
    if let Some(expected) = restored_zai_binding(sampler_config.credential_binding.as_ref())?
        && (!expected.same_record(&current) || current.generation < expected.generation)
    {
        return Err(ProviderBindingError::ZaiCredentialChanged);
    }
    if sampler_config.api_backend != ApiBackend::ChatCompletions {
        return Err(ProviderBindingError::InvalidZaiRestoredBinding);
    }

    sampler_config.provider = ProviderId::ZaiCodingPlan;
    sampler_config.credential_source = CredentialSourceId::ZaiCodingPlanApiKey;
    sampler_config.credential_binding = Some(current.clone());
    sampler_config.api_key = None;
    sampler_config.base_url = ZAI_CODING_PLAN_BASE_URL.to_owned();
    sampler_config.api_backend = ApiBackend::ChatCompletions;
    sampler_config.auth_scheme = AuthScheme::Bearer;
    sampler_config.extra_headers.clear();
    sampler_config.attribution_callback = None;
    sampler_config.bearer_resolver = None;
    sampler_config.request_auth = Some(zai_sampler_auth(
        store.clone(),
        credentials.clone(),
        current.clone(),
    ));
    sampler_config.deployment_id = None;
    sampler_config.user_id = None;
    sampler_config.compactions_remaining = None;
    sampler_config.compaction_at_tokens = None;
    sampler_config.doom_loop_recovery = None;

    let route = ProviderModelRoute {
        provider: ProviderId::ZaiCodingPlan,
        model: sampler_config.model.clone(),
    };
    Ok(BoundProviderRuntime {
        sampler_config,
        api_key_provider: Some(zai_tool_auth(store, credentials, current)),
        route,
    })
}

fn restored_zai_binding(
    binding: Option<&CredentialBinding>,
) -> Result<Option<&CredentialBinding>, ProviderBindingError> {
    let Some(binding) = binding else {
        return Ok(None);
    };
    let correct_owner = binding.provider == ProviderId::ZaiCodingPlan
        && binding.source == CredentialSourceId::ZaiCodingPlanApiKey;
    if correct_owner && binding.record_id.is_none() && binding.generation == 0 {
        return Ok(None);
    }
    let complete = correct_owner
        && binding.generation > 0
        && binding
            .record_id
            .as_deref()
            .is_some_and(|record_id| !record_id.trim().is_empty());
    complete
        .then_some(Some(binding))
        .ok_or(ProviderBindingError::InvalidZaiRestoredBinding)
}

fn restored_kimi_binding(
    binding: Option<&CredentialBinding>,
) -> Result<Option<&CredentialBinding>, ProviderBindingError> {
    let Some(binding) = binding else {
        return Ok(None);
    };
    let correct_owner = binding.provider == ProviderId::KimiCode
        && binding.source == CredentialSourceId::KimiCodeApiKey;
    if correct_owner && binding.record_id.is_none() && binding.generation == 0 {
        return Ok(None);
    }
    let complete = correct_owner
        && binding.generation > 0
        && binding
            .record_id
            .as_deref()
            .is_some_and(|record_id| !record_id.trim().is_empty());
    complete
        .then_some(Some(binding))
        .ok_or(ProviderBindingError::InvalidKimiRestoredBinding)
}

async fn bind_openai_codex_with_manager(
    mut sampler_config: SamplerConfig,
    manager: Arc<CodexAuthManager>,
) -> Result<BoundProviderRuntime, ProviderBindingError> {
    let expected = restored_binding(sampler_config.credential_binding.as_ref())?;

    // `ensure_fresh` starts with a disk verification, so logout/account switch
    // is observed before either runtime below is constructed.  A refresh may
    // advance the same record generation and is intentionally allowed.
    let credentials = manager.ensure_fresh().await?;
    let current = credentials.credential_binding();
    if let Some(expected) = expected {
        if !expected.same_record(&current) {
            return Err(ProviderBindingError::AccountChanged);
        }
        if current.generation < expected.generation {
            return Err(ProviderBindingError::GenerationRegressed);
        }
    }

    // Rebuild the provider-owned wire contract rather than trusting restored,
    // catalog, or generic config fields.  Keep only the non-credential Codex
    // compatibility marker; tracing remains available through header_injector.
    sampler_config.provider = ProviderId::OpenAiCodex;
    sampler_config.credential_source = CredentialSourceId::OpenAiCodexSubscription;
    sampler_config.credential_binding = Some(current);
    sampler_config.api_key = None;
    sampler_config.base_url = OPENAI_CODEX_BASE_URL.to_owned();
    sampler_config.api_backend = ApiBackend::Responses;
    sampler_config.auth_scheme = AuthScheme::Bearer;
    sampler_config.extra_headers.retain(|name, value| {
        name.eq_ignore_ascii_case(OPENAI_CODEX_RESPONSES_LITE_HEADER) && value == "true"
    });
    sampler_config.attribution_callback = None;
    sampler_config.bearer_resolver = None;
    sampler_config.request_auth = Some(shared_sampler_request_auth(manager.clone()));
    sampler_config.deployment_id = None;
    sampler_config.user_id = None;
    sampler_config.compactions_remaining = None;
    sampler_config.compaction_at_tokens = None;
    sampler_config.doom_loop_recovery = None;

    let route = ProviderModelRoute {
        provider: ProviderId::OpenAiCodex,
        model: sampler_config.model.clone(),
    };
    Ok(BoundProviderRuntime {
        sampler_config,
        api_key_provider: Some(shared_api_key_provider(manager)),
        route,
    })
}

/// Pin a subscription-provider model/auxiliary candidate to the active session
/// record. A cross-provider candidate must start unbound so it cannot retain a
/// process-current record from a different provider selection.
pub(crate) fn pin_provider_candidate_to_active_record(
    candidate: &mut SamplerConfig,
    active_provider: ProviderId,
    active_binding: Option<&CredentialBinding>,
) {
    let same_pinned_provider = (candidate.provider.is_openai_codex()
        && active_provider.is_openai_codex())
        || (candidate.provider.is_kimi_code() && active_provider.is_kimi_code())
        || (candidate.provider.is_zai_coding_plan() && active_provider.is_zai_coding_plan());
    candidate.credential_binding = same_pinned_provider
        .then(|| active_binding.cloned())
        .flatten();
}

/// Interpret the optional binding stored in session/config state.
///
/// `None` and the explicit unbound Codex baseline are accepted for a brand-new
/// or legacy session.  Any partially populated value is treated as corrupt
/// restored state rather than silently repinned.
fn restored_binding(
    binding: Option<&CredentialBinding>,
) -> Result<Option<&CredentialBinding>, ProviderBindingError> {
    let Some(binding) = binding else {
        return Ok(None);
    };
    let correct_owner = binding.provider == ProviderId::OpenAiCodex
        && binding.source == CredentialSourceId::OpenAiCodexSubscription;
    if correct_owner && binding.record_id.is_none() && binding.generation == 0 {
        return Ok(None);
    }
    let complete = correct_owner
        && binding.generation > 0
        && binding
            .record_id
            .as_deref()
            .is_some_and(|record_id| !record_id.trim().is_empty());
    complete
        .then_some(Some(binding))
        .ok_or(ProviderBindingError::InvalidRestoredBinding)
}

/// Result of the behavior-neutral `/fast` policy extraction.  The caller owns
/// chat-state mutation and user-visible output; provider/catalog policy stays
/// in this module.
pub(in crate::session) struct FastModeResolution {
    pub(in crate::session) updated_sampling_config: Option<xai_grok_sampling_types::SamplingConfig>,
    pub(in crate::session) output: String,
}

pub(in crate::session) async fn resolve_fast_mode(
    models: &crate::agent::models::ModelsManager,
    sampling: Option<xai_grok_sampling_types::SamplingConfig>,
    mode: crate::session::slash_commands::FastModeAction,
) -> FastModeResolution {
    use crate::session::slash_commands::FastModeAction;

    fn output_only(output: impl Into<String>) -> FastModeResolution {
        FastModeResolution {
            updated_sampling_config: None,
            output: output.into(),
        }
    }
    if !models.fast_mode_enabled() {
        return output_only("Fast mode is disabled by [features] fast_mode = false.");
    }
    if mode == FastModeAction::Invalid {
        return output_only("Usage: /fast on|off|status");
    }
    let Some(mut sampling) = sampling else {
        return output_only("Fast mode is unavailable until this session has selected a model.");
    };
    if !sampling.provider.is_openai_codex() {
        return output_only(
            "Fast mode is available only for supported models authenticated with a ChatGPT Codex subscription.",
        );
    }
    let Some(fast_tier) = models.fast_service_tier_for_provider(sampling.provider, &sampling.model)
    else {
        return output_only(format!(
            "Fast mode is not advertised for {} by the authenticated Codex model catalog.",
            sampling.model
        ));
    };

    let active = sampling
        .service_tier
        .as_deref()
        .and_then(|tier| {
            models.resolve_service_tier_for_provider(sampling.provider, &sampling.model, tier)
        })
        .as_deref()
        == Some(fast_tier.id.as_str());
    let credit_rate =
        if sampling.model.starts_with("gpt-5.6") || sampling.model.starts_with("gpt-5.5") {
            "2.5x Standard subscription credits"
        } else if sampling.model.starts_with("gpt-5.4") {
            "2x Standard subscription credits"
        } else {
            "increased subscription credits"
        };
    let (enabled, persisted_value) = match mode {
        FastModeAction::Toggle if active => (false, Some("default")),
        FastModeAction::Toggle => (true, Some("fast")),
        FastModeAction::On => (true, Some("fast")),
        FastModeAction::Off => (false, Some("default")),
        FastModeAction::Status => (active, None),
        FastModeAction::Invalid => unreachable!(),
    };

    let updated_sampling_config = persisted_value.map(|_| {
        sampling.service_tier = Some(if enabled {
            fast_tier.id.clone()
        } else {
            xai_grok_sampling_types::OPENAI_CODEX_STANDARD_SERVICE_TIER.to_owned()
        });
        sampling.clone()
    });
    let persistence_error = if let Some(value) = persisted_value {
        crate::util::config::set_codex_service_tier(value)
            .await
            .err()
    } else {
        None
    };
    let mut output = if enabled {
        format!(
            "Fast mode: on\nModel: {}\nSpeed: about 1.5x Standard\nUsage: {credit_rate}",
            sampling.model
        )
    } else {
        format!(
            "Fast mode: off\nModel: {}\nSpeed and usage: Standard",
            sampling.model
        )
    };
    if let Some(error) = persistence_error {
        output.push_str(&format!(
            "\n\nThe current session was updated, but the default could not be saved: {error}"
        ));
    }
    FastModeResolution {
        updated_sampling_config,
        output,
    }
}

/// Append the ephemeral Codex multi-agent policy without changing the agent
/// loop or persisting provider policy into conversation history.
pub(in crate::session) fn inject_codex_multi_agent_policy(
    models: &crate::agent::models::ModelsManager,
    sampling: Option<&xai_grok_sampling_types::SamplingConfig>,
    request: &mut xai_grok_sampling_types::ConversationRequest,
) {
    let Some(sampling) = sampling else { return };
    let Some(instruction) = models.codex_multi_agent_mode_instruction(
        sampling.provider,
        &sampling.model,
        sampling.reasoning_effort,
    ) else {
        return;
    };
    request
        .items
        .push(xai_grok_sampling_types::ConversationItem::system(
            instruction,
        ));
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use tempfile::TempDir;

    use super::*;
    use crate::auth::codex::{CodexCredentialStore, credentials_for_test};

    async fn manager_with_credentials(
        dir: &TempDir,
        mut credentials: crate::auth::codex::CodexCredentials,
        generation: u64,
    ) -> (Arc<CodexAuthManager>, CredentialBinding) {
        credentials.revision = generation;
        let binding = credentials.credential_binding();
        let store = CodexCredentialStore::from_auth_path(dir.path().join("auth.json"));
        store.save(credentials).await.expect("save credentials");
        let manager = Arc::new(CodexAuthManager::from_store(store).expect("load manager"));
        (manager, binding)
    }

    fn codex_config(binding: Option<CredentialBinding>) -> SamplerConfig {
        SamplerConfig {
            credential_binding: binding,
            api_key: Some("must-be-cleared".to_owned()),
            base_url: "https://api.x.ai/v1".to_owned(),
            api_backend: ApiBackend::ChatCompletions,
            bearer_resolver: Some(Arc::new(TestBearerResolver)),
            deployment_id: Some("must-be-cleared".to_owned()),
            user_id: Some("must-be-cleared".to_owned()),
            ..SamplerConfig::openai_codex("gpt-5.4")
        }
    }

    #[derive(Debug)]
    struct TestBearerResolver;
    impl xai_grok_sampler::BearerResolver for TestBearerResolver {
        fn current_bearer(&self) -> Option<String> {
            Some("must-not-be-used".to_owned())
        }
    }

    #[test]
    fn imported_or_foreign_bindings_never_cross_provider_boundaries() {
        let mut foreign = CredentialBinding::kimi_code(Some("SENTINEL-FOREIGN-RECORD".to_owned()));
        foreign.generation = 7;
        assert!(matches!(
            restored_binding(Some(&foreign)),
            Err(ProviderBindingError::InvalidRestoredBinding)
        ));

        let mut codex = CredentialBinding::openai_codex(Some("SENTINEL-CODEX-RECORD".to_owned()));
        codex.generation = 4;
        assert!(matches!(
            restored_kimi_binding(Some(&codex)),
            Err(ProviderBindingError::InvalidKimiRestoredBinding)
        ));
        assert!(matches!(
            restored_zai_binding(Some(&codex)),
            Err(ProviderBindingError::InvalidZaiRestoredBinding)
        ));

        let mut candidate = SamplerConfig::openai_codex("gpt-5.4");
        pin_provider_candidate_to_active_record(
            &mut candidate,
            ProviderId::KimiCode,
            Some(&foreign),
        );
        assert!(
            candidate.credential_binding.is_none(),
            "a provider candidate must not fall back to another provider's active record"
        );
    }

    #[test]
    fn model_switch_and_aux_candidates_keep_provider_and_active_record_together() {
        let mut active = CredentialBinding::openai_codex(Some("record-a".to_owned()));
        active.generation = 7;
        let mut candidate = SamplerConfig::openai_codex("gpt-5.6-luna");
        candidate.credential_binding = Some(CredentialBinding::openai_codex(Some(
            "process-current-other-record".to_owned(),
        )));

        pin_provider_candidate_to_active_record(
            &mut candidate,
            ProviderId::OpenAiCodex,
            Some(&active),
        );

        assert_eq!(candidate.provider, ProviderId::OpenAiCodex);
        assert_eq!(candidate.model, "gpt-5.6-luna");
        assert_eq!(candidate.credential_binding, Some(active));
    }

    #[test]
    fn kimi_aux_candidates_keep_the_active_api_key_record() {
        let mut active = CredentialBinding::kimi_code(Some("record-a".to_owned()));
        active.generation = 4;
        let mut candidate = SamplerConfig::kimi_code("k3", ApiBackend::Messages);
        candidate.credential_binding = Some(CredentialBinding::kimi_code(Some(
            "process-current-other-record".to_owned(),
        )));

        pin_provider_candidate_to_active_record(
            &mut candidate,
            ProviderId::KimiCode,
            Some(&active),
        );

        assert_eq!(candidate.provider, ProviderId::KimiCode);
        assert_eq!(candidate.model, "k3");
        assert_eq!(candidate.credential_binding, Some(active));
    }

    #[tokio::test]
    async fn custom_runtime_drops_xai_credentials_and_generic_tool_auth() {
        let dir = TempDir::new().unwrap();
        let credentials =
            credentials_for_test("account-a", "refresh-a", Utc::now() + Duration::hours(2));
        let (manager, _) = manager_with_credentials(&dir, credentials, 1).await;
        let config = SamplerConfig {
            provider: ProviderId::Custom,
            credential_source: CredentialSourceId::XaiSession,
            api_key: Some("xai-session-sentinel".to_owned()),
            base_url: "https://custom.invalid/v1".to_owned(),
            model: "custom-model".to_owned(),
            bearer_resolver: Some(Arc::new(TestBearerResolver)),
            deployment_id: Some("xai-deployment-sentinel".to_owned()),
            user_id: Some("xai-user-sentinel".to_owned()),
            extra_headers: indexmap::indexmap! {
                "X-XAI-Token-Auth".to_owned() => "xai-grok-cli".to_owned(),
                "x-authenticateresponse".to_owned() => "authenticate-response".to_owned(),
                "x-custom-route".to_owned() => "blue".to_owned(),
            },
            ..SamplerConfig::default()
        };

        let runtime = bind_provider_runtime(config, Some(shared_api_key_provider(manager)))
            .await
            .expect("custom runtime binds without provider-owned auth");

        assert_eq!(runtime.route.provider, ProviderId::Custom);
        assert!(runtime.api_key_provider.is_none());
        assert!(runtime.sampler_config.api_key.is_none());
        assert_eq!(
            runtime.sampler_config.credential_source,
            CredentialSourceId::Unspecified
        );
        assert!(runtime.sampler_config.bearer_resolver.is_none());
        assert!(runtime.sampler_config.deployment_id.is_none());
        assert!(runtime.sampler_config.user_id.is_none());
        assert_eq!(
            runtime
                .sampler_config
                .extra_headers
                .get("x-custom-route")
                .map(String::as_str),
            Some("blue")
        );
        assert_eq!(runtime.sampler_config.extra_headers.len(), 1);
    }

    #[tokio::test]
    async fn custom_runtime_retains_only_explicit_custom_static_key() {
        let config = SamplerConfig {
            provider: ProviderId::Custom,
            credential_source: CredentialSourceId::StaticApiKey,
            api_key: Some("custom-static-key".to_owned()),
            base_url: "https://custom.invalid/v1".to_owned(),
            model: "custom-model".to_owned(),
            ..SamplerConfig::default()
        };

        let runtime = bind_provider_runtime(config, None)
            .await
            .expect("explicit custom credential binds");

        assert_eq!(
            runtime.sampler_config.api_key.as_deref(),
            Some("custom-static-key")
        );
        assert_eq!(
            runtime.sampler_config.credential_source,
            CredentialSourceId::StaticApiKey
        );
    }

    #[tokio::test]
    async fn custom_runtime_retains_a_named_rotating_provider_key() {
        let config = SamplerConfig {
            provider: ProviderId::Custom,
            credential_source: CredentialSourceId::RotatingAuthProvider,
            api_key: Some("custom-rotating-key".to_owned()),
            base_url: "https://custom.invalid/v1".to_owned(),
            model: "custom-model".to_owned(),
            ..SamplerConfig::default()
        };

        let runtime = bind_provider_runtime(config, None)
            .await
            .expect("named custom-provider credential binds");

        assert_eq!(
            runtime.sampler_config.api_key.as_deref(),
            Some("custom-rotating-key")
        );
        assert_eq!(
            runtime.sampler_config.credential_source,
            CredentialSourceId::RotatingAuthProvider
        );
    }

    #[tokio::test]
    async fn xai_runtime_drops_a_rotating_custom_provider_key() {
        let config = SamplerConfig {
            provider: ProviderId::Xai,
            credential_source: CredentialSourceId::RotatingAuthProvider,
            api_key: Some("custom-rotating-key".to_owned()),
            base_url: "https://api.x.ai/v1".to_owned(),
            model: "grok-test".to_owned(),
            ..SamplerConfig::default()
        };

        let runtime = bind_provider_runtime(config, None)
            .await
            .expect("xAI runtime binds after rejecting the foreign credential");

        assert!(runtime.sampler_config.api_key.is_none());
        assert_eq!(
            runtime.sampler_config.credential_source,
            CredentialSourceId::Unspecified
        );
    }

    #[tokio::test]
    async fn restored_binding_allows_only_same_record_generation_advancement() {
        let dir = TempDir::new().unwrap();
        let credentials =
            credentials_for_test("account-a", "refresh-a", Utc::now() + Duration::hours(2));
        let (manager, current) = manager_with_credentials(&dir, credentials, 3).await;
        let mut restored = current.clone();
        restored.generation = 2;

        let runtime = bind_openai_codex_with_manager(codex_config(Some(restored)), manager)
            .await
            .expect("same-record advancement binds");

        assert_eq!(format!("{runtime:?}"), "BoundProviderRuntime { .. }");
        assert_eq!(runtime.sampler_config.credential_binding, Some(current));
        assert_eq!(runtime.route.provider, ProviderId::OpenAiCodex);
        assert_eq!(runtime.route.model, "gpt-5.4");
        assert!(runtime.api_key_provider.is_some());
        assert!(runtime.sampler_config.request_auth.is_some());
        assert!(runtime.sampler_config.api_key.is_none());
        assert!(runtime.sampler_config.bearer_resolver.is_none());
        assert_eq!(runtime.sampler_config.base_url, OPENAI_CODEX_BASE_URL);
        assert_eq!(runtime.sampler_config.api_backend, ApiBackend::Responses);
    }

    #[tokio::test]
    async fn restored_binding_rejects_account_switch_before_client_construction() {
        let dir = TempDir::new().unwrap();
        let original =
            credentials_for_test("account-a", "refresh-a", Utc::now() + Duration::hours(2));
        let restored = original.credential_binding();
        let replacement =
            credentials_for_test("account-b", "refresh-b", Utc::now() + Duration::hours(2));
        let (manager, _) = manager_with_credentials(&dir, replacement, 1).await;

        let error = bind_openai_codex_with_manager(codex_config(Some(restored)), manager)
            .await
            .unwrap_err();
        assert!(matches!(error, ProviderBindingError::AccountChanged));
    }

    #[tokio::test]
    async fn logout_and_generation_regression_are_fallible() {
        let dir = TempDir::new().unwrap();
        let credentials =
            credentials_for_test("account-a", "refresh-a", Utc::now() + Duration::hours(2));
        let (manager, current) = manager_with_credentials(&dir, credentials, 2).await;
        std::fs::remove_file(dir.path().join("auth.json")).unwrap();
        let error = bind_openai_codex_with_manager(codex_config(Some(current.clone())), manager)
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            ProviderBindingError::CodexAuth(CodexAuthError::NotLoggedIn)
        ));

        let dir = TempDir::new().unwrap();
        let credentials =
            credentials_for_test("account-a", "refresh-a", Utc::now() + Duration::hours(2));
        let (manager, mut restored) = manager_with_credentials(&dir, credentials, 2).await;
        restored.generation = 3;
        let error = bind_openai_codex_with_manager(codex_config(Some(restored)), manager)
            .await
            .unwrap_err();
        assert!(matches!(error, ProviderBindingError::GenerationRegressed));
    }

    #[tokio::test]
    async fn binder_forces_codex_wire_contract_and_drops_generic_auth_headers() {
        let dir = TempDir::new().unwrap();
        let credentials =
            credentials_for_test("account-a", "refresh-a", Utc::now() + Duration::hours(2));
        let (manager, binding) = manager_with_credentials(&dir, credentials, 1).await;
        let mut config = codex_config(Some(binding));
        config
            .extra_headers
            .insert("authorization".to_owned(), "must-be-cleared".to_owned());
        config.extra_headers.insert(
            OPENAI_CODEX_RESPONSES_LITE_HEADER.to_owned(),
            "true".to_owned(),
        );

        let runtime = bind_openai_codex_with_manager(config, manager)
            .await
            .expect("bind runtime");
        assert_eq!(runtime.sampler_config.extra_headers.len(), 1);
        assert_eq!(
            runtime
                .sampler_config
                .extra_headers
                .get(OPENAI_CODEX_RESPONSES_LITE_HEADER)
                .map(String::as_str),
            Some("true")
        );
        xai_grok_sampler::SamplingClient::new(runtime.sampler_config)
            .expect("bound config constructs a provider-safe client");
    }
}
