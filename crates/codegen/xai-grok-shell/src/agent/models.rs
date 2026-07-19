//! Model fetching, resolution, and management.

mod codex_catalog;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use parking_lot::RwLock;

use agent_client_protocol as acp;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use indexmap::IndexMap;

use crate::agent::config::{self, ModelEntry, resolve_credentials, sampling_config_for_model};
use crate::auth::{AuthManager, AuthMode, GrokAuth, GrokComConfig};
use crate::remote::{FetchModelsResult, fetch_models_blocking};
use crate::sampling::SamplerConfig as SamplingConfig;
#[cfg(test)]
use codex_catalog::CodexCatalogIdentity;
use codex_catalog::{CodexCatalogController, current_identity};
use globset::{Glob, GlobSet, GlobSetBuilder};
use xai_grok_sampling_types::{ProviderId, ReasoningEffort, ReasoningEffortOption};

// ── Auth method for model fetching ──────────────────────────────────────────

/// Credential for `/v1/models` fetching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelFetchAuth {
    Session,
    ApiKey,
    Deployment,
    CustomEndpoint,
}

impl ModelFetchAuth {
    /// custom_endpoint > session > deployment > API key.
    ///
    /// A `deployment_key` outranks an ambient `XAI_API_KEY` so a stray env key
    /// can't redirect model fetching from the deployment's entitlement-gated
    /// proxy to a raw `/v1/models` endpoint that lists the full model registry.
    pub(crate) fn resolve(endpoints: &config::EndpointsConfig, has_cached_session: bool) -> Self {
        if endpoints.has_custom_endpoint() {
            Self::CustomEndpoint
        } else if has_cached_session {
            Self::Session
        } else if endpoints.deployment_key.is_some() {
            Self::Deployment
        } else if crate::agent::auth_method::has_xai_api_key_env() {
            Self::ApiKey
        } else {
            Self::Session
        }
    }

    fn cache_auth_method(&self) -> CacheAuthMethod {
        match self {
            Self::CustomEndpoint | Self::ApiKey => CacheAuthMethod::ApiKey,
            Self::Session => CacheAuthMethod::Session,
            Self::Deployment => CacheAuthMethod::Deployment,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
enum CacheAuthMethod {
    Session,
    ApiKey,
    Deployment,
}

fn hash_scope_part(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

/// One-way credential/account binding for the legacy xAI model cache. Raw API
/// keys, session tokens, user IDs, and deployment keys are never persisted or
/// logged. First-party session bindings prefer stable principal identity so
/// ordinary token rotation does not discard a valid catalog. External and
/// stored API-key credentials are bound to the opaque bearer itself because
/// their profile fields can be carried across an arbitrary provider refresh.
fn xai_catalog_credential_scope(
    endpoints: &config::EndpointsConfig,
    auth: Option<&GrokAuth>,
    fetch_auth: ModelFetchAuth,
) -> Option<String> {
    let mut hasher = blake3::Hasher::new_derive_key("grok-build xai model cache scope v2");
    hash_scope_part(
        &mut hasher,
        crate::remote::models_list_url(endpoints, fetch_auth).as_bytes(),
    );
    hash_scope_part(&mut hasher, format!("{fetch_auth:?}").as_bytes());
    if fetch_auth == ModelFetchAuth::Deployment {
        hash_scope_part(
            &mut hasher,
            endpoints
                .deployment_key
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        );
    }

    if endpoints.has_custom_endpoint() || fetch_auth == ModelFetchAuth::ApiKey {
        let api_key = crate::agent::auth_method::read_xai_api_key_env()
            .ok()
            .filter(|key| !key.is_empty())
            .or_else(|| {
                auth.map(|value| value.key.clone())
                    .filter(|key| !key.is_empty())
            });
        hash_scope_part(&mut hasher, b"api_key");
        if let Some(api_key) = api_key {
            hash_scope_part(&mut hasher, api_key.as_bytes());
        } else {
            hash_scope_part(&mut hasher, b"missing");
        }
    } else {
        hash_scope_part(&mut hasher, b"session");
        let Some(auth) = auth else {
            hash_scope_part(&mut hasher, b"missing");
            return Some(hasher.finalize().to_hex().to_string());
        };
        hash_scope_part(&mut hasher, format!("{:?}", auth.auth_mode).as_bytes());
        if matches!(auth.auth_mode, AuthMode::External | AuthMode::ApiKey) {
            hash_scope_part(&mut hasher, b"opaque_credential");
            hash_scope_part(&mut hasher, auth.key.as_bytes());
        } else {
            hash_scope_part(
                &mut hasher,
                auth.oidc_issuer.as_deref().unwrap_or_default().as_bytes(),
            );
            hash_scope_part(
                &mut hasher,
                auth.oidc_client_id
                    .as_deref()
                    .unwrap_or_default()
                    .as_bytes(),
            );
            // `x-userid` is part of every session catalog request, including
            // team-principal requests, so it is always part of the cache
            // identity rather than only being a fallback.
            hash_scope_part(&mut hasher, auth.user_id.as_bytes());
            for value in [
                auth.principal_type.as_deref(),
                auth.principal_id.as_deref(),
                auth.team_id.as_deref(),
                auth.organization_id.as_deref(),
            ] {
                hash_scope_part(&mut hasher, value.unwrap_or_default().as_bytes());
            }
            if auth.user_id.is_empty()
                && auth.principal_id.is_none()
                && auth.team_id.is_none()
                && auth.organization_id.is_none()
            {
                hash_scope_part(&mut hasher, auth.key.as_bytes());
            }
        }
    }
    Some(hasher.finalize().to_hex().to_string())
}

pub(crate) fn task_model_error_for_catalog(
    requested: &str,
    available: &IndexMap<String, ModelEntry>,
    is_session_auth: bool,
) -> Option<String> {
    let is_available = |entry: &ModelEntry| {
        !entry.info.hidden
            && entry.info.user_selectable
            && model_addressable_for_auth(&entry.info, is_session_auth)
    };
    if config::find_model_by_id(available, requested).is_some_and(&is_available) {
        return None;
    }

    let mut slugs = available
        .iter()
        .filter(|(_, entry)| is_available(entry))
        .map(|(slug, _)| slug.as_str())
        .collect::<Vec<_>>();
    slugs.sort_unstable();
    let guidance = if slugs.is_empty() {
        "No valid model slugs are currently available. Omit `model` to inherit the parent model."
            .to_string()
    } else {
        format!(
            "Valid model slugs: {}. Omit `model` to inherit the parent model.",
            slugs.join(", ")
        )
    };
    Some(format!("Unknown Task.model slug '{requested}'. {guidance}"))
}

/// The ChatGPT Codex catalog is already scoped to an authenticated ChatGPT
/// account. Its `supported_in_api` flag describes OpenAI API-key availability,
/// not ChatGPT subscription visibility, and must not be gated by xAI's auth
/// mode. xAI/custom entries retain their existing API-key/session semantics.
fn model_visible_for_auth(info: &config::ModelInfo, is_xai_session_auth: bool) -> bool {
    !info.hidden && model_addressable_for_auth(info, is_xai_session_auth)
}

fn model_addressable_for_auth(info: &config::ModelInfo, is_xai_session_auth: bool) -> bool {
    info.provider == ProviderId::OpenAiCodex || is_xai_session_auth || info.supported_in_api
}

/// Thread-safe model manager.
///
/// Owns the auth manager, config, and gateway needed to refresh models.
/// Uses `parking_lot::RwLock` for short clone-and-release access.
#[derive(Clone)]
pub struct ModelsManager {
    inner: Arc<Inner>,
}

struct Inner {
    prefetched: RwLock<Option<IndexMap<String, ModelEntry>>>,
    /// Provider-local authenticated ChatGPT Codex catalog lifecycle. Its
    /// cache, identity, ETag, generation, and retry state are independent from
    /// xAI and are merged into `models` only after xAI/custom resolution.
    codex_catalog: CodexCatalogController,
    models: RwLock<IndexMap<String, ModelEntry>>,
    current_model_id: RwLock<acp::ModelId>,
    current_reasoning_effort: RwLock<Option<ReasoningEffort>>,
    etag: RwLock<Option<String>>,
    /// Set once a real catalog has been fetched; gates whether
    /// `apply_refresh_result` calls `reselect_default_model` (first
    /// time) or `reselect_current_model_if_missing` (subsequent).
    /// Reset in `clear()` for identity changes.
    has_fetched_real_catalog: RwLock<bool>,
    /// Independent xAI catalog lifecycle; Codex discovery must not suppress
    /// first-xAI reselection or xAI retry recovery.
    has_xai_catalog: RwLock<bool>,
    /// One-way credential/account scope that produced the installed xAI
    /// catalog. Stable across same-account token rotation, different across
    /// account/API/deployment changes.
    xai_catalog_credential_scope: RwLock<Option<String>>,
    /// Newest disk-cache publication accepted for the installed xAI scope.
    /// Prevents two same-generation cache readers from applying E2 and then
    /// regressing in memory to an older E1 that completed later.
    xai_catalog_cache_fetched_at: RwLock<Option<DateTime<Utc>>>,
    // ── Owned context for self-contained refresh ────────────────
    auth_manager: Arc<AuthManager>,
    cfg: RwLock<config::Config>,
    fetch_auth: RwLock<ModelFetchAuth>,
    gateway: RwLock<Option<xai_acp_lib::AcpAgentGatewaySender>>,
    cache: ModelsCacheManager,
    /// Serializes every provider-state + merged-catalog publication across
    /// config-watcher, session, ETag, and auth threads. No async work is done
    /// while held.
    catalog_write_gate: parking_lot::Mutex<()>,
    /// Monotonic xAI catalog operation/config generation. Only the newest
    /// request/cache snapshot may publish, preventing an older endpoint/auth
    /// response from winning after a config reload or newer fetch.
    xai_fetch_generation: AtomicU64,
    /// Guard to prevent overlapping retry loops.
    retry_in_flight: AtomicBool,
    /// `allowed_models` matched nothing in the fetched catalog; the prompt path
    /// blocks rather than run on the bundled default. Set in `apply_refresh_result`.
    allowlist_excludes_all: AtomicBool,
    /// Layer-3 LazinessDetector model-switch signal. Carries a
    /// monotonically-increasing generation counter (`u64`) that is
    /// bumped whenever the current model id actually changes via
    /// [`Self::set_current_model_id`].
    ///
    /// Two consumer patterns:
    /// 1. `subscribe_model_switch().changed().await` — used by the
    ///    `SessionActor` main loop to react to a switch (e.g. zero
    ///    the per-session nudge counter). Critically, `watch::Receiver`
    ///    only resolves `.changed()` on changes that happen **after**
    ///    subscription — there is no stored-permit hazard akin to
    ///    `tokio::sync::Notify::notify_one()`.
    /// 2. `model_switch_generation()` — cheap snapshot read used by
    ///    `maybe_fire_laziness_check`'s polling loop to detect a
    ///    switch that occurred during the idle wait or sampler call.
    ///
    /// `watch::Sender` natively fans out to every subscriber, so this
    /// replaces the previous `RwLock<Vec<Arc<Notify>>>` listener
    /// registry — no manual fan-out, no listener-leak risk, no
    /// `unregister` API to maintain.
    model_switch_watch: tokio::sync::watch::Sender<u64>,
}

#[derive(Clone)]
struct XaiCatalogSnapshot {
    generation: u64,
    config: config::Config,
    fetch_auth: ModelFetchAuth,
    origin: String,
    credential_scope: Option<String>,
}

struct XaiNetworkCatalog {
    models: IndexMap<String, ModelEntry>,
    etag: Option<String>,
}

impl Default for ModelsManager {
    fn default() -> Self {
        let grok_home = crate::util::grok_home::grok_home();
        let auth_manager = Arc::new(AuthManager::new(&grok_home, GrokComConfig::default()));
        Self::new(
            None,
            IndexMap::new(),
            acp::ModelId::new("default"),
            auth_manager,
            config::Config::default(),
        )
    }
}

impl ModelsManager {
    pub(crate) fn new(
        prefetched: Option<IndexMap<String, ModelEntry>>,
        models: IndexMap<String, ModelEntry>,
        current_model_id: acp::ModelId,
        auth_manager: Arc<AuthManager>,
        cfg: config::Config,
    ) -> Self {
        let has_session = auth_manager.current_or_expired().is_some();
        let fetch_auth = ModelFetchAuth::resolve(&cfg.endpoints, has_session);
        let current_reasoning_effort = cfg.models.default_reasoning_effort;
        let codex_models = models
            .iter()
            .filter(|(_, entry)| entry.info.provider == ProviderId::OpenAiCodex)
            .map(|(id, entry)| (id.clone(), entry.clone()))
            .collect();
        let codex_catalog = CodexCatalogController::new(
            codex_models,
            crate::util::config::resolve_remote_fetch_enabled(),
        );
        Self {
            inner: Arc::new(Inner {
                prefetched: RwLock::new(prefetched),
                codex_catalog,
                models: RwLock::new(models),
                current_model_id: RwLock::new(current_model_id),
                current_reasoning_effort: RwLock::new(current_reasoning_effort),
                etag: RwLock::new(None),
                has_fetched_real_catalog: RwLock::new(false),
                has_xai_catalog: RwLock::new(false),
                xai_catalog_credential_scope: RwLock::new(None),
                xai_catalog_cache_fetched_at: RwLock::new(None),
                auth_manager,
                cfg: RwLock::new(cfg),
                fetch_auth: RwLock::new(fetch_auth),
                gateway: RwLock::new(None),
                cache: ModelsCacheManager::new(),
                catalog_write_gate: parking_lot::Mutex::new(()),
                xai_fetch_generation: AtomicU64::new(1),
                retry_in_flight: AtomicBool::new(false),
                allowlist_excludes_all: AtomicBool::new(false),
                model_switch_watch: tokio::sync::watch::channel(0u64).0,
            }),
        }
    }

    /// Subscribe to model-switch events. Returns a `watch::Receiver`
    /// carrying the monotonic generation counter. `.changed()` only
    /// resolves on switches that occur **after** subscription, so
    /// there is no stored-permit hazard (the bug that motivated
    /// replacing the previous `Arc<Notify>` design).
    pub fn subscribe_model_switch(&self) -> tokio::sync::watch::Receiver<u64> {
        self.inner.model_switch_watch.subscribe()
    }

    /// Cheap snapshot of the current model-switch generation. Used by
    /// `maybe_fire_laziness_check`'s polling loop to detect a switch
    /// that occurred during the idle wait or sampler call without
    /// having to allocate a fresh `Receiver` per fire.
    pub fn model_switch_generation(&self) -> u64 {
        *self.inner.model_switch_watch.borrow()
    }

    /// Build from a resolved config. Falls back to bundled default if no models available.
    ///
    /// When `prefetched_models` is `None`, the disk cache is consulted so that
    /// server-side models are available for default-model resolution even when
    /// the caller didn't do an explicit prefetch.
    pub fn from_config(
        cfg: &config::Config,
        prefetched_models: Option<IndexMap<String, ModelEntry>>,
        auth_manager: Arc<AuthManager>,
    ) -> Result<Self, String> {
        let cached_session = auth_manager.current_or_expired();
        let has_session = cached_session.is_some();
        let is_session_auth = auth_manager
            .auth_mode()
            .is_some_and(|m| m.is_session_auth());
        let fetch_auth = ModelFetchAuth::resolve(&cfg.endpoints, has_session);
        let credential_scope =
            xai_catalog_credential_scope(&cfg.endpoints, cached_session.as_ref(), fetch_auth);
        let cache = ModelsCacheManager::new();
        let scoped_cache = cache.load_fresh(
            &fetch_auth.cache_auth_method(),
            &crate::remote::models_list_url(&cfg.endpoints, fetch_auth),
            credential_scope.as_deref(),
        );
        let must_attest_prefetch = has_session || fetch_auth != ModelFetchAuth::Session;
        let (prefetched_models, xai_cache_fetched_at, xai_cache_etag) =
            match (prefetched_models, scoped_cache) {
                // Every production early-prefetch path persists before
                // returning. Re-adopt through the *current* credential scope
                // so account A cannot finish prefetching and then be installed
                // after a switch to B.
                (Some(_), Some(cached)) if must_attest_prefetch => {
                    (Some(cached.models), Some(cached.fetched_at), cached.etag)
                }
                (Some(_), None) if must_attest_prefetch => {
                    tracing::warn!(
                        "discarding unattested xAI startup prefetch after auth/config transition"
                    );
                    (None, None, None)
                }
                (Some(prefetched), _) => (Some(prefetched), None, None),
                (None, Some(cached)) => (Some(cached.models), Some(cached.fetched_at), cached.etag),
                (None, None) => (None, None, None),
            };
        let has_prefetched = prefetched_models.is_some();
        let codex_prefetch = CodexCatalogController::prefetch_blocking()
            .filter(|_| crate::util::config::resolve_remote_fetch_enabled());
        let has_codex_catalog = codex_prefetch.is_some();
        let codex_etag = codex_prefetch
            .as_ref()
            .and_then(|prefetch| prefetch.etag.clone());
        let codex_catalog_identity = codex_prefetch
            .as_ref()
            .map(|prefetch| prefetch.identity.clone());
        let codex_catalog_needs_revalidation = codex_prefetch
            .as_ref()
            .is_some_and(|prefetch| !prefetch.authoritative);
        let codex_models = codex_prefetch
            .as_ref()
            .map(|prefetch| prefetch.models.clone())
            .unwrap_or_default();
        let catalog = resolve_combined_model_catalog(cfg, prefetched_models.clone(), codex_models);

        // Validate only against a complete-enough real catalog. If one
        // configured provider is still pending recovery, the other provider's
        // successful prefetch must not make its allowlist patterns fatal.
        if has_prefetched || has_codex_catalog {
            let remote_fetch_enabled = crate::util::config::resolve_remote_fetch_enabled();
            let xai_expected = match fetch_auth {
                ModelFetchAuth::Session => has_session,
                ModelFetchAuth::ApiKey
                | ModelFetchAuth::Deployment
                | ModelFetchAuth::CustomEndpoint => true,
            };
            let codex_expected = current_identity().is_some();
            let provider_pending = remote_fetch_enabled
                && ((xai_expected && !has_prefetched)
                    || (codex_expected
                        && (!has_codex_catalog || codex_catalog_needs_revalidation)));
            if let Err(error) = validate_selectable(cfg, &catalog)
                && !(provider_pending && allowlist_matches_nothing(cfg, &catalog))
            {
                return Err(error);
            }
        }

        let (current_model_key, current_model, model_source) =
            resolve_default_model(cfg, &catalog, is_session_auth);

        tracing::info!(
            model_id = %current_model.model,
            source = %model_source,
            "default model resolved"
        );

        let current_model_id = acp::ModelId::new(Arc::from(current_model_key));

        let mgr = Self::new(
            prefetched_models,
            catalog,
            current_model_id,
            auth_manager,
            cfg.clone(),
        );
        if has_prefetched || has_codex_catalog {
            *mgr.inner.has_fetched_real_catalog.write() = true;
        }
        *mgr.inner.has_xai_catalog.write() = has_prefetched;
        *mgr.inner.xai_catalog_credential_scope.write() =
            has_prefetched.then_some(credential_scope).flatten();
        *mgr.inner.xai_catalog_cache_fetched_at.write() =
            has_prefetched.then_some(xai_cache_fetched_at).flatten();
        *mgr.inner.etag.write() = has_prefetched.then_some(xai_cache_etag).flatten();
        mgr.inner.codex_catalog.initialize_prefetch(
            codex_etag,
            codex_catalog_identity,
            codex_catalog_needs_revalidation,
            has_codex_catalog,
        );
        mgr.inner.allowlist_excludes_all.store(
            allowlist_matches_nothing(cfg, &mgr.inner.models.read()),
            Ordering::Release,
        );
        // Keep the cross-process identity lease alive until every piece of the
        // prefetched account-bound catalog has been installed in the manager.
        drop(codex_prefetch);
        Ok(mgr)
    }

    pub(crate) fn set_gateway(&self, gateway: xai_acp_lib::AcpAgentGatewaySender) {
        *self.inner.gateway.write() = Some(gateway);
    }

    /// Swap config, rebuild catalog, and reselect the model.
    ///
    /// Calls `reselect_default_model` when the preferred model changed
    /// (and is `Some`); otherwise `reselect_current_model_if_missing`.
    pub fn apply_config(&self, new_config: config::Config) {
        let catalog_write = self.inner.catalog_write_gate.lock();
        let current_auth = self.inner.auth_manager.current_or_expired();
        let live_config = self.inner.cfg.read().clone();
        let has_session = current_auth.is_some();
        // Resolve from the live auth snapshot rather than trusting the stored
        // enum: an auth switch can arrive before its watcher callback.
        let live_fetch_auth = ModelFetchAuth::resolve(&live_config.endpoints, has_session);
        let live_credential_scope = xai_catalog_credential_scope(
            &live_config.endpoints,
            current_auth.as_ref(),
            live_fetch_auth,
        );
        let had_xai_catalog = *self.inner.has_xai_catalog.read();
        let live_scope_changed = had_xai_catalog
            && self.inner.xai_catalog_credential_scope.read().as_ref()
                != live_credential_scope.as_ref();

        // Reject an invalid reload instead of mutating live state: bad globs or
        // (once a real catalog exists) an allowlist that excludes everything.
        // Credential reconciliation still happens before every early return.
        if let Err(e) = new_config.validate_model_filters() {
            tracing::error!(error = %e, "ignoring config reload: invalid model filters");
            if live_scope_changed {
                *self.inner.fetch_auth.write() = live_fetch_auth;
                self.inner.cache.invalidate();
                self.clear_xai_catalog_unlocked();
                drop(catalog_write);
                self.notify_models_updated();
                self.start_xai_catalog_recovery();
            }
            return;
        }
        let new_fetch_auth = ModelFetchAuth::resolve(&new_config.endpoints, has_session);
        let new_credential_scope = xai_catalog_credential_scope(
            &new_config.endpoints,
            current_auth.as_ref(),
            new_fetch_auth,
        );
        let xai_scope_changed = had_xai_catalog
            && self.inner.xai_catalog_credential_scope.read().as_ref()
                != new_credential_scope.as_ref();
        let prefetched = if xai_scope_changed {
            None
        } else {
            self.inner.prefetched.read().clone()
        };
        let new_catalog =
            resolve_combined_model_catalog(&new_config, prefetched, self.codex_models_snapshot());
        let has_real_catalog = *self.inner.has_fetched_real_catalog.read();
        let xai_expected = new_fetch_auth != ModelFetchAuth::Session || has_session;
        let start_xai_recovery = xai_expected && (!had_xai_catalog || xai_scope_changed);
        let codex_expected = current_identity().is_some();
        let remote_fetch_enabled = crate::util::config::resolve_remote_fetch_enabled();
        let start_codex_recovery =
            remote_fetch_enabled && codex_expected && self.inner.codex_catalog.recovery_needed();
        let provider_pending = remote_fetch_enabled
            && ((xai_expected && (!had_xai_catalog || xai_scope_changed))
                || (codex_expected && self.inner.codex_catalog.recovery_needed()));
        if has_real_catalog
            && let Err(e) = validate_selectable(&new_config, &new_catalog)
            && !(provider_pending && allowlist_matches_nothing(&new_config, &new_catalog))
        {
            tracing::error!(error = %e, "ignoring config reload: allowed_models excludes all models");
            if live_scope_changed {
                // Credential identity safety wins over retaining a rejected
                // config reload. Never leave the prior producer's entitlements
                // installed after a known account/key/endpoint transition.
                *self.inner.fetch_auth.write() = live_fetch_auth;
                self.inner.cache.invalidate();
                self.clear_xai_catalog_unlocked();
                drop(catalog_write);
                self.notify_models_updated();
                self.start_xai_catalog_recovery();
            }
            return;
        }

        let (old_preferred, old_default_is_campaign) = {
            let cfg = self.inner.cfg.read();
            (
                cfg.models.default.clone(),
                cfg.models.default_is_campaign_driven,
            )
        };
        let new_preferred = new_config.models.default.clone();
        self.inner
            .xai_fetch_generation
            .fetch_add(1, Ordering::AcqRel);
        self.inner
            .codex_catalog
            .advance_config_generation(remote_fetch_enabled);
        if xai_scope_changed {
            *self.inner.prefetched.write() = None;
            *self.inner.has_xai_catalog.write() = false;
            *self.inner.xai_catalog_credential_scope.write() = None;
            *self.inner.etag.write() = None;
            *self.inner.has_fetched_real_catalog.write() = self.inner.codex_catalog.has_catalog();
            self.inner.cache.invalidate();
        }
        *self.inner.fetch_auth.write() = new_fetch_auth;
        *self.inner.cfg.write() = new_config.clone();
        // Always fail closed at the prompt seam while an allowlisted provider
        // is still pending discovery; successful publication recomputes this.
        let excludes_all = allowlist_matches_nothing(&new_config, &new_catalog);
        self.inner
            .allowlist_excludes_all
            .store(excludes_all, Ordering::Relaxed);
        *self.inner.models.write() = new_catalog;

        // A preferred-model flip caused only by a campaign overlay appearing or
        // disappearing must not yank an in-flight session whose current model is
        // still usable — the campaign applies to /new sessions only.
        let preferred_changed = new_preferred != old_preferred && new_preferred.is_some();
        // Recognize an appearing OR withdrawing campaign from the
        // `default_is_campaign_driven` flag on each config (no disk I/O); correct
        // even when the user has no base default (where a value compare would miss).
        let mut campaign_defaults = std::collections::HashSet::new();
        if new_config.models.default_is_campaign_driven
            && let Some(d) = &new_preferred
        {
            campaign_defaults.insert(d.clone());
        }
        if old_default_is_campaign && let Some(d) = &old_preferred {
            campaign_defaults.insert(d.clone());
        }
        let campaign_only_flip =
            is_campaign_only_flip(&old_preferred, &new_preferred, &campaign_defaults);
        let current_still_ok = {
            let models = self.inner.models.read();
            let cur = self.inner.current_model_id.read();
            models
                .get(cur.0.as_ref())
                .is_some_and(|e| e.info.user_selectable)
        };
        if preferred_changed && !(campaign_only_flip && current_still_ok) {
            self.reselect_default_model(&new_config);
        } else {
            self.reselect_current_model_if_missing(&new_config);
        }
        drop(catalog_write);

        // Push the new catalog to connected clients (`x.ai/models/update`).
        // Without this, a long-running agent (leader mode) correctly swaps
        // its in-memory catalog on a config.toml `[model.*]`/`[models]` edit,
        // but already-connected clients keep rendering the stale model list
        // until they reconnect. No-op when no gateway is attached (tests,
        // pre-init).
        self.notify_models_updated();
        if start_xai_recovery {
            self.start_xai_catalog_recovery();
        }
        if start_codex_recovery {
            self.start_openai_codex_catalog_recovery();
        }
    }

    // ── Accessors ───────────────────────────────────────────────────

    pub fn models(&self) -> IndexMap<String, ModelEntry> {
        self.inner.models.read().clone()
    }

    pub fn endpoints(&self) -> config::EndpointsConfig {
        self.inner.cfg.read().endpoints.clone()
    }

    /// Whether authenticated Codex service-tier controls are enabled.
    pub fn fast_mode_enabled(&self) -> bool {
        self.inner.cfg.read().features.fast_mode.unwrap_or(true)
    }

    /// Does the current credential grant access to OAuth-only models?
    pub(crate) fn is_session_auth(&self) -> bool {
        self.inner
            .auth_manager
            .auth_mode()
            .is_some_and(|m| m.is_session_auth())
    }

    /// ACP-visible (non-hidden) projection of the catalog.
    /// The catalog coming from `resolve_model_catalog` already has
    /// allowed_models + disabled_models + hidden_models applied.
    pub fn available(&self) -> IndexMap<acp::ModelId, acp::ModelInfo> {
        let snapshot = {
            let models = self.inner.models.read();
            models.clone()
        };

        let selectable: IndexMap<_, _> = snapshot
            .into_iter()
            .filter(|(_, e)| e.info.user_selectable)
            .collect();

        available_models(&selectable, self.is_session_auth())
    }

    pub(crate) fn task_model_error(&self, requested: &str) -> Option<String> {
        let is_session_auth = self.is_session_auth();
        let models = self.inner.models.read();
        task_model_error_for_catalog(requested, &models, is_session_auth)
    }

    pub fn current_model_id(&self) -> acp::ModelId {
        self.inner.current_model_id.read().clone()
    }

    pub fn set_current_model_id(&self, id: acp::ModelId) {
        // Only bump the model-switch generation on a real change.
        // The pager's `/model` handler can call this with the
        // already-active id during re-resolution; bumping the counter
        // in that case would needlessly cancel a healthy in-flight
        // classifier call and zero the per-session nudge counter.
        let changed = {
            let mut cur = self.inner.current_model_id.write();
            let changed = *cur != id;
            *cur = id;
            changed
        };
        if changed {
            self.inner
                .model_switch_watch
                .send_modify(|generation| *generation += 1);
        }
    }

    /// Look up the per-model Layer-3 LazinessDetector config for the
    /// model identified by `model_id`. Returns the default (disabled)
    /// config when the id isn't in the catalog — same fallback
    /// semantics as the `auto_compact_threshold_percent` lookup.
    pub fn laziness_detector_for(&self, model_id: &str) -> config::LazinessDetectorPerModelConfig {
        self.inner
            .models
            .read()
            .get(model_id)
            .map(|e| e.info().laziness_detector.clone())
            .unwrap_or_default()
    }

    /// Test-only catalog poke: inserts a `ModelEntry` keyed by `id`,
    /// allowing integration tests to enable Layer-3 features per
    /// model without spinning up the full config-merge pipeline.
    #[cfg(test)]
    pub(crate) fn insert_test_entry(&self, id: impl Into<String>, entry: ModelEntry) {
        self.inner.models.write().insert(id.into(), entry);
    }

    pub fn current_reasoning_effort(&self) -> Option<ReasoningEffort> {
        *self.inner.current_reasoning_effort.read()
    }

    pub fn set_current_reasoning_effort(&self, effort: Option<ReasoningEffort>) {
        *self.inner.current_reasoning_effort.write() = effort;
    }

    /// Whether the given model supports reasoning effort according to the catalog.
    pub fn model_supports_reasoning_effort(&self, model_id: &str) -> bool {
        let models = self.inner.models.read();
        model_entry_by_id_or_slug(&models, model_id)
            .map(|e| e.info().supports_reasoning_effort)
            .unwrap_or(false)
    }

    /// Whether `effort` is explicitly accepted by the model identified by a
    /// catalog id or routing slug.
    ///
    /// All effort assignment seams use this method so an ACP client, restored
    /// session, subagent override, or global default cannot bypass a dynamic
    /// catalog's per-model menu (for example, GPT-5.6 Luna does not offer
    /// `ultra`, while Sol and Terra do).
    pub fn model_offers_reasoning_effort(&self, model_id: &str, effort: ReasoningEffort) -> bool {
        self.resolve_reasoning_effort_for_model(model_id, effort)
            .is_some()
    }

    /// Resolve a user/config effort token to the effective value for this
    /// catalog model. Before Codex introduced a distinct `max` tier, Grok
    /// Build accepted `max` as the legacy spelling for `xhigh`; retain that
    /// behavior for every non-Codex provider while preserving real Codex Max.
    pub fn resolve_reasoning_effort_for_model(
        &self,
        model_id: &str,
        effort: ReasoningEffort,
    ) -> Option<ReasoningEffort> {
        let models = self.inner.models.read();
        let info = model_entry_by_id_or_slug(&models, model_id)?.info();
        resolve_model_reasoning_effort(info, effort)
    }

    /// Provider-qualified variant for persisted sampling configs, which store
    /// the routing slug rather than the namespaced catalog key. Without the
    /// provider discriminator a custom/xAI entry sharing a Codex slug could
    /// supply the wrong effort menu.
    pub fn model_offers_reasoning_effort_for_provider(
        &self,
        provider: ProviderId,
        model_id: &str,
        effort: ReasoningEffort,
    ) -> bool {
        self.resolve_reasoning_effort_for_provider(provider, model_id, effort)
            .is_some()
    }

    /// Provider-qualified effort resolver for persisted sampling configs,
    /// which carry routing slugs rather than namespaced catalog ids.
    pub fn resolve_reasoning_effort_for_provider(
        &self,
        provider: ProviderId,
        model_id: &str,
        effort: ReasoningEffort,
    ) -> Option<ReasoningEffort> {
        let models = self.inner.models.read();
        let info = model_entry_by_id_or_slug_for_provider(&models, model_id, provider)?.info();
        resolve_model_reasoning_effort(info, effort)
    }

    /// The catalog default reasoning effort for `model_id`, if the catalog
    /// pins one. Used as the final fallback when neither the session handle
    /// nor the global config sets an explicit effort, so surfaced config stays
    /// consistent with the effort sampling actually uses.
    pub fn model_default_reasoning_effort(&self, model_id: &str) -> Option<ReasoningEffort> {
        let models = self.inner.models.read();
        let info = model_entry_by_id_or_slug(&models, model_id)?.info();
        info.reasoning_effort
            .and_then(|effort| resolve_model_reasoning_effort(info, effort))
            .or_else(|| advertised_default_reasoning_effort(info))
    }

    pub fn model_default_reasoning_effort_for_provider(
        &self,
        provider: ProviderId,
        model_id: &str,
    ) -> Option<ReasoningEffort> {
        let models = self.inner.models.read();
        let info = model_entry_by_id_or_slug_for_provider(&models, model_id, provider)?.info();
        info.reasoning_effort
            .and_then(|effort| resolve_model_reasoning_effort(info, effort))
            .or_else(|| advertised_default_reasoning_effort(info))
    }

    /// Return the provider-advertised Fast tier for a model. Fast is an exact
    /// provider contract: its catalog and request id is `priority`. Do not
    /// infer request behavior from the display name.
    pub fn fast_service_tier_for_provider(
        &self,
        provider: ProviderId,
        model_id: &str,
    ) -> Option<config::ModelServiceTier> {
        if provider != ProviderId::OpenAiCodex || !self.fast_mode_enabled() {
            return None;
        }
        let models = self.inner.models.read();
        let info = model_entry_by_id_or_slug_for_provider(&models, model_id, provider)?.info();
        info.service_tiers
            .iter()
            .find(|tier| tier.id == xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER)
            .cloned()
    }

    /// Validate a persisted service-tier selection against the live,
    /// provider-qualified model catalog. Explicit Standard is always valid for
    /// a known Codex model; every other value must be advertised by that model.
    pub fn resolve_service_tier_for_provider(
        &self,
        provider: ProviderId,
        model_id: &str,
        service_tier: &str,
    ) -> Option<String> {
        if provider != ProviderId::OpenAiCodex {
            return None;
        }
        let models = self.inner.models.read();
        let info = model_entry_by_id_or_slug_for_provider(&models, model_id, provider)?.info();
        let normalized = normalize_service_tier_alias(service_tier.trim());
        if normalized == xai_grok_sampling_types::OPENAI_CODEX_STANDARD_SERVICE_TIER {
            return Some(normalized.to_owned());
        }
        if !self.fast_mode_enabled() {
            return None;
        }
        info.service_tiers
            .iter()
            .any(|tier| tier.id == normalized)
            .then(|| normalized.to_owned())
    }

    /// Turn-local Codex v2 delegation policy. Ultra retains Grok Build's
    /// native agent loop and activates its existing task/subagent tool
    /// proactively; every other level explicitly returns to request-only
    /// delegation. No tools, permissions, nesting limits, or session state are
    /// changed by this instruction.
    pub fn codex_multi_agent_mode_instruction(
        &self,
        provider: ProviderId,
        model_id: &str,
        effort: Option<ReasoningEffort>,
    ) -> Option<String> {
        if provider != ProviderId::OpenAiCodex {
            return None;
        }
        let models = self.inner.models.read();
        let entry = model_entry_by_id_or_slug_for_provider(&models, model_id, provider)?;
        if entry.info().multi_agent_version.as_deref() != Some("v2") {
            return None;
        }
        let proactive = effort == Some(ReasoningEffort::Ultra)
            && model_info_offers_reasoning_effort(entry.info(), ReasoningEffort::Ultra);
        let body = if proactive {
            "Proactive multi-agent delegation is enabled for this turn. Use Grok Build's existing subagents when independent parallel work would materially improve speed or quality. This changes delegation policy only; all existing tool, permission, nesting, and session rules still apply."
        } else {
            "Proactive multi-agent delegation is disabled for this turn. Spawn a Grok Build subagent only when the user, project instructions, or an applicable skill explicitly requests delegation. All existing tool, permission, nesting, and session rules remain in force."
        };
        Some(format!(
            "<multi_agent_mode provider=\"openai_codex\">\n{body}\n</multi_agent_mode>"
        ))
    }

    /// The raw catalog `reasoning_efforts` list for `model_id` with no fallback,
    /// empty when the catalog pins none (caller falls back to the built-in
    /// session modes). Distinct from the pager's gate-first, fallback-applied
    /// `ModelState::reasoning_effort_options`.
    pub fn model_reasoning_efforts(&self, model_id: &str) -> Vec<ReasoningEffortOption> {
        let models = self.inner.models.read();
        model_entry_by_id_or_slug(&models, model_id)
            .map(|e| e.info().reasoning_efforts.clone())
            .unwrap_or_default()
    }

    /// Resolve the current image tool gates from the live configuration rather
    /// than inheriting an `ImageGenConfig` constructed for a previous provider.
    pub(crate) fn image_tool_gates(&self) -> (bool, bool) {
        let cfg = self.inner.cfg.read();
        (
            cfg.resolve_image_gen().value,
            cfg.resolve_image_edit().value,
        )
    }

    pub fn model_supports_backend_search(&self, model_id: &str) -> bool {
        self.inner
            .models
            .read()
            .get(model_id)
            .map(|e| e.info().supports_backend_search)
            .unwrap_or(false)
    }

    pub fn model_compactions_remaining(
        &self,
        model_id: &str,
    ) -> Option<xai_grok_sampling_types::CompactionsRemaining> {
        self.inner
            .models
            .read()
            .get(model_id)
            .and_then(|e| e.info().compactions_remaining)
    }

    pub fn model_compaction_at_tokens(
        &self,
        model_id: &str,
    ) -> Option<xai_grok_sampling_types::CompactionAtTokens> {
        self.inner
            .models
            .read()
            .get(model_id)
            .and_then(|e| e.info().compaction_at_tokens)
    }

    /// Catalog opt-in to display the served-checkpoint fingerprint for this model.
    ///
    /// `model_id` may be a routing slug (`config.model`, e.g. `grok-4.5`)
    /// OR a catalog key; the catalog map is keyed by the config key, which can
    /// differ from the slug for custom/enterprise ids (e.g. key `enterprise-grok-build`
    /// → slug `grok-4.5`). Resolve to the catalog key first so a slug
    /// caller still finds the opted-in entry.
    pub fn model_show_model_fingerprint(&self, model_id: &str) -> bool {
        let models = self.inner.models.read();
        resolve_catalog_key(&models, &acp::ModelId::new(model_id))
            .and_then(|key| models.get(key.0.as_ref()))
            .map(|e| e.info().show_model_fingerprint)
            .unwrap_or(false)
    }

    /// Resolved next-prompt-suggestion model pin from the live config
    /// (`env > [models] prompt_suggestion > remote settings`); tracks config
    /// hot-reloads via [`Self::apply_config`]. Consumed catalog-guarded by
    /// `handle_suggest_prompt`.
    pub fn prompt_suggest_model_pin(&self) -> crate::config::PromptSuggestModelPin {
        self.inner.cfg.read().prompt_suggest_model_pin.clone()
    }

    /// Whether `model_id` resolves in the current catalog — as a config key
    /// or a routing slug (see [`resolve_catalog_key`]). Deliberately checks
    /// the full catalog rather than the user-selectable projection: auxiliary
    /// background calls need a *sampleable* model, and hidden or
    /// non-selectable entries are still sampleable.
    pub fn model_in_catalog(&self, model_id: &str) -> bool {
        let models = self.inner.models.read();
        resolve_catalog_key(&models, &acp::ModelId::new(model_id)).is_some()
    }

    pub fn model_in_catalog_for_provider(&self, provider: ProviderId, model_id: &str) -> bool {
        let models = self.inner.models.read();
        model_entry_by_id_or_slug_for_provider(&models, model_id, provider).is_some()
    }

    pub fn model_supports_image_input_for_provider(
        &self,
        provider: ProviderId,
        model_id: &str,
    ) -> bool {
        let models = self.inner.models.read();
        model_entry_by_id_or_slug_for_provider(&models, model_id, provider)
            .is_some_and(|entry| entry.info.supports_image_input)
    }

    /// Resolve request-local image input policy from the authenticated Codex
    /// catalog. Other providers retain their established attachment behavior;
    /// they do not inherit Codex catalog enforcement or its fail-closed lookup.
    pub fn codex_image_input_capability_for_request(
        &self,
        provider: ProviderId,
        model_id: &str,
    ) -> xai_grok_sampling_types::ImageInputCapability {
        use xai_grok_sampling_types::ImageInputCapability;

        if !provider.is_openai_codex() {
            return ImageInputCapability::Unspecified;
        }
        if self.model_supports_image_input_for_provider(provider, model_id) {
            ImageInputCapability::Supported
        } else {
            // Missing/stale cross-provider entries cannot opt a Codex request
            // into image input. An authoritative refreshed catalog can do so on
            // the next request without rewriting persisted conversation state.
            ImageInputCapability::Unsupported
        }
    }

    #[cfg(test)]
    fn prefetched(&self) -> Option<IndexMap<String, ModelEntry>> {
        self.inner.prefetched.read().clone()
    }

    #[cfg(test)]
    fn has_fetched_real_catalog(&self) -> bool {
        *self.inner.has_fetched_real_catalog.read()
    }

    // ── Mutations ───────────────────────────────────────────────────

    fn rebuild(&self, cfg: &config::Config, prefetched: Option<IndexMap<String, ModelEntry>>) {
        let _write = self.inner.catalog_write_gate.lock();
        self.rebuild_unlocked(cfg, prefetched);
    }

    /// Rebuild while the caller holds `catalog_write_gate` across any related
    /// provider-state mutation/snapshot.
    fn rebuild_unlocked(
        &self,
        cfg: &config::Config,
        prefetched: Option<IndexMap<String, ModelEntry>>,
    ) {
        let models = resolve_combined_model_catalog(cfg, prefetched, self.codex_models_snapshot());
        *self.inner.models.write() = models;
    }

    fn codex_models_snapshot(&self) -> IndexMap<String, ModelEntry> {
        self.inner.codex_catalog.models_snapshot()
    }

    /// Refresh the authenticated ChatGPT account's entitled Codex catalog.
    /// Failures retain the last in-memory catalog; only an authoritative
    /// network/cache result replaces it. This path never consults xAI auth.
    pub async fn refresh_openai_codex_models(&self) -> bool {
        self.inner.codex_catalog.refresh_models(self).await
    }

    #[cfg(test)]
    fn apply_openai_codex_catalog(
        &self,
        catalog: crate::remote::CodexModelCatalog,
        etag: Option<String>,
        identity: CodexCatalogIdentity,
        authoritative: bool,
    ) {
        self.inner.codex_catalog.apply_catalog_for_test(
            self,
            catalog,
            etag,
            identity,
            authoritative,
        );
    }

    /// Clear only Codex provider state when its current credential/account no
    /// longer matches the catalog producer. xAI models and credentials remain
    /// untouched. Returns whether visible state changed.
    #[cfg(test)]
    fn reconcile_openai_codex_identity(&self, current: Option<&CodexCatalogIdentity>) -> bool {
        self.inner.codex_catalog.reconcile_identity(self, current)
    }

    /// Route response-header model ETags to the provider that produced them.
    /// Codex and xAI ETags/caches are deliberately independent.
    pub async fn refresh_if_new_etag_for_provider(&self, provider: ProviderId, etag: String) {
        self.refresh_if_new_etag_for_provider_binding(provider, etag, None)
            .await;
    }

    /// Route response-header ETags with the immutable credential generation
    /// that produced the response. A missing or stale Codex binding forces a
    /// normal authenticated `/models` fetch rather than renewing cached bytes.
    pub async fn refresh_if_new_etag_for_provider_binding(
        &self,
        provider: ProviderId,
        etag: String,
        credential_binding: Option<xai_grok_sampling_types::CredentialBinding>,
    ) {
        if provider == ProviderId::OpenAiCodex {
            self.inner
                .codex_catalog
                .refresh_if_new_etag(self, etag, credential_binding)
                .await;
        } else {
            self.refresh_if_new_etag(etag).await;
        }
    }

    /// Refresh models when the etag changes.
    ///
    /// Commits a new ETag only with a successful generation-checked fetch.
    /// Concurrent duplicate notifications may launch duplicate work, but a
    /// failed request can never permanently suppress later recovery.
    pub async fn refresh_if_new_etag(&self, etag: String) {
        let same_etag = {
            let current = self.inner.etag.read();
            current.as_deref() == Some(etag.as_str())
        };
        if same_etag {
            let snapshot = self.xai_catalog_snapshot();
            let catalog_write = self.inner.catalog_write_gate.lock();
            let installed_scope_matches = *self.inner.has_xai_catalog.read()
                && self.inner.xai_catalog_credential_scope.read().as_ref()
                    == snapshot.credential_scope.as_ref();
            if installed_scope_matches && self.xai_catalog_snapshot_is_current(&snapshot) {
                self.inner.cache.renew_ttl(
                    &snapshot.fetch_auth.cache_auth_method(),
                    &snapshot.origin,
                    snapshot.credential_scope.as_deref(),
                    &etag,
                );
                return;
            }
            let clear_stale_catalog = *self.inner.has_xai_catalog.read()
                && self.inner.xai_catalog_credential_scope.read().as_ref()
                    != snapshot.credential_scope.as_ref();
            if clear_stale_catalog {
                self.inner.cache.invalidate();
                self.clear_xai_catalog_unlocked();
            }
            drop(catalog_write);
            if clear_stale_catalog {
                self.notify_models_updated();
            }
            tracing::info!(etag = %etag, "models etag matched stale or missing identity; refreshing");
            self.do_refresh(Some(etag), RefreshStrategy::Online);
            return;
        }
        tracing::info!(etag = %etag, "models etag changed, refreshing");
        self.do_refresh(Some(etag), RefreshStrategy::Online);
    }

    /// Auth identity changed: invalidate disk cache and refresh the catalog.
    ///
    /// Safe on OIDC token recovery after idle: we never drop a successfully-fetched
    /// xAI catalog on transient failure. Only fall back to the bundled default when
    /// we have never had a real xAI catalog, or via
    /// the genuine no-auth path (`clear()`).
    ///
    /// Respects the auth snapshot / hot-swap discipline.
    pub async fn on_auth_changed(&self) {
        let config = {
            let _catalog_write = self.inner.catalog_write_gate.lock();
            let config = self.inner.cfg.read().clone();
            let current_auth = self.inner.auth_manager.current_or_expired();
            let has_session = current_auth.is_some();
            let fetch_auth = ModelFetchAuth::resolve(&config.endpoints, has_session);
            self.inner
                .xai_fetch_generation
                .fetch_add(1, Ordering::AcqRel);
            *self.inner.fetch_auth.write() = fetch_auth;
            // Invalidate under the same gate as the generation transition:
            // readers from the old generation are rejected, and readers from
            // the new generation cannot observe the old account's cache.
            self.inner.cache.invalidate();
            let current_scope =
                xai_catalog_credential_scope(&config.endpoints, current_auth.as_ref(), fetch_auth);
            let account_changed = *self.inner.has_xai_catalog.read()
                && self.inner.xai_catalog_credential_scope.read().as_ref()
                    != current_scope.as_ref();
            if account_changed {
                // Preserve on same-account token refresh, but never expose A's
                // entitlements while B's first fetch is pending or failing.
                self.clear_xai_catalog_unlocked();
            }
            config
        };
        crate::agent::init::update_telemetry_config(&config, &self.inner.auth_manager);
        if self.clear_if_unauthenticated_session() {
            return;
        }

        // Never eagerly drop prefetched on auth recovery. Only fall back to
        // bundled defaults when we have never had a real catalog. Resolved once
        // so the fetch and the failure-vs-disabled classification below agree.
        let remote_fetch_enabled = crate::util::config::resolve_remote_fetch_enabled();
        self.fetch_and_apply_inner(remote_fetch_enabled).await;

        let fell_back_to_bundled = {
            let _catalog_write = self.inner.catalog_write_gate.lock();
            if !*self.inner.has_xai_catalog.read() && self.inner.prefetched.read().is_none() {
                let live_config = self.inner.cfg.read().clone();
                self.rebuild_unlocked(&live_config, None);
                self.reselect_current_model_if_missing(&live_config);
                true
            } else {
                false
            }
        };

        if fell_back_to_bundled {
            if remote_fetch_enabled {
                xai_grok_telemetry::unified_log::warn(
                    "model catalog: falling back to bundled defaults only",
                    None,
                    Some(serde_json::json!({
                        "trigger": "on_auth_changed",
                        "had_real_catalog": false,
                    })),
                );
            } else {
                // Deliberate no-fetch state, not a failure: no warn-class log.
                tracing::debug!("model catalog: bundled defaults in use (remote_fetch disabled)");
            }
            // Schedule background retries so we recover once the network is
            // back (e.g. after sleep/resume when the first fetch races DNS).
            // With remote_fetch disabled a retry can never succeed, so none is
            // scheduled.
            if remote_fetch_enabled {
                self.spawn_catalog_retry();
            }
        }

        self.notify_models_updated();
    }

    /// Notify clients about the current model catalog.
    fn notify_models_updated(&self) {
        let available = self.available();
        let current = self.current_model_id();
        let count = available.len();
        xai_grok_telemetry::unified_log::info(
            "model catalog: notifying clients",
            None,
            Some(serde_json::json!({
                "model_count": count,
                "current_model_id": current.0.as_ref(),
            })),
        );
        if let Some(ref gw) = *self.inner.gateway.read() {
            let model_state =
                acp::SessionModelState::new(current, available.values().cloned().collect());
            if let Ok(params) = serde_json::value::to_raw_value(&model_state) {
                gw.forward_fire_and_forget(acp::ExtNotification::new(
                    "x.ai/models/update",
                    params.into(),
                ));
            }
        }
    }

    /// Hot-reload the catalog from `~/.grok/models_cache.json` after an
    /// external write (detected by the config file watcher).
    ///
    /// A long-running leader otherwise only refreshes its catalog from its
    /// *own* fetch paths (startup prefetch, auth change, response-header etag).
    /// When another grok process sharing `~/.grok` (a `--no-leader` run, a
    /// newer client, grok-desktop) fetches a fresher `/v1/models` catalog and
    /// persists it, this picks it up without a network round-trip.
    ///
    /// Guards, in order:
    /// 1. `load_fresh` — rejects stale (TTL), version-mismatched,
    ///    auth-method-mismatched, or origin-mismatched cache files (another
    ///    process running with different credentials or pointed at a
    ///    different backend must not poison this catalog).
    /// 2. Content dedup — the leader itself rewrites the cache file
    ///    (`persist` after fetch, `renew_ttl` on same-etag responses), and the
    ///    watcher has no self-write suppression. If the cached models match
    ///    the in-memory prefetched catalog this is a no-op (the etag is still
    ///    adopted so `refresh_if_new_etag` doesn't refetch needlessly).
    ///
    /// On a real change: swaps the prefetched catalog, rebuilds, re-resolves
    /// the configured default when this is the first real catalog (otherwise
    /// reselects the current model if it disappeared), and notifies clients.
    pub fn reload_from_disk_cache(&self) {
        self.reload_from_cache_manager(&self.inner.cache);
    }

    /// Core of [`Self::reload_from_disk_cache`], parameterized over the cache
    /// manager so tests can point it at a temp file (the production
    /// `ModelsCacheManager` path is fixed to `grok_home()`, a process-wide
    /// `OnceLock`).
    fn reload_from_cache_manager(&self, cache: &ModelsCacheManager) {
        let snapshot = self.xai_catalog_snapshot();
        let Some(cached) = cache.load_fresh(
            &snapshot.fetch_auth.cache_auth_method(),
            &snapshot.origin,
            snapshot.credential_scope.as_deref(),
        ) else {
            tracing::debug!("models cache changed on disk but is not loadable; ignoring");
            return;
        };
        let catalog_write = self.inner.catalog_write_gate.lock();
        if !self.xai_catalog_snapshot_is_current(&snapshot) {
            tracing::debug!("models cache was loaded for stale config; ignoring");
            return;
        }
        if self.inner.xai_catalog_credential_scope.read().as_ref()
            == snapshot.credential_scope.as_ref()
            && self
                .inner
                .xai_catalog_cache_fetched_at
                .read()
                .is_some_and(|installed| cached.fetched_at < installed)
        {
            tracing::debug!("older models cache publication completed late; ignoring");
            return;
        }
        // Self-write / no-change dedup by content. `ModelEntry` doesn't impl
        // `PartialEq` (nested config types), so compare the serialized form —
        // catalogs are small (tens of entries) and writes are debounced.
        let same_content = {
            let prefetched = self.inner.prefetched.read();
            prefetched.as_ref().is_some_and(|current| {
                serde_json::to_string(current).ok() == serde_json::to_string(&cached.models).ok()
            })
        };
        if same_content {
            // Adopt the (possibly newer) etag without a rebuild so the next
            // response-header comparison in `refresh_if_new_etag` is accurate.
            if cached.etag.is_some() {
                *self.inner.etag.write() = cached.etag;
            }
            *self.inner.has_xai_catalog.write() = true;
            *self.inner.has_fetched_real_catalog.write() = true;
            *self.inner.xai_catalog_credential_scope.write() = snapshot.credential_scope.clone();
            *self.inner.xai_catalog_cache_fetched_at.write() = Some(cached.fetched_at);
            tracing::debug!("models cache changed on disk but catalog is identical; skipping");
            return;
        }

        let cfg = self.inner.cfg.read().clone();
        let count = cached.models.len();
        // Capture whether this is the first real catalog (mirrors
        // `apply_refresh_result`): if the leader bootstrapped on bundled
        // defaults, the configured default must be re-resolved against the
        // real catalog rather than left on a placeholder.
        let first_xai_catalog = {
            let mut flag = self.inner.has_xai_catalog.write();
            let was_first = !*flag;
            *flag = true;
            was_first
        };
        *self.inner.xai_catalog_credential_scope.write() = snapshot.credential_scope.clone();
        *self.inner.xai_catalog_cache_fetched_at.write() = Some(cached.fetched_at);
        *self.inner.has_fetched_real_catalog.write() = true;
        *self.inner.prefetched.write() = Some(cached.models.clone());
        self.rebuild_unlocked(&cfg, Some(cached.models));
        *self.inner.etag.write() = cached.etag;
        if first_xai_catalog {
            self.reselect_default_model(&cfg);
        } else {
            self.reselect_current_model_if_missing(&cfg);
        }

        // Recompute the prompt-block flag (mirrors `apply_refresh_result`) so
        // a corrective external cache write unlatches a previously latched
        // "allowlist excludes everything" state instead of keeping prompts
        // blocked against a stale catalog.
        let excludes_all = allowlist_matches_nothing(&cfg, &self.inner.models.read());
        self.inner
            .allowlist_excludes_all
            .store(excludes_all, Ordering::Relaxed);
        if excludes_all {
            tracing::error!("allowed_models excludes all fetched models; prompts will be blocked");
        }
        drop(catalog_write);

        tracing::info!(count, "model catalog hot-reloaded from disk cache");
        xai_grok_telemetry::unified_log::info(
            "model catalog: reloaded from external disk-cache write",
            None,
            Some(serde_json::json!({ "model_count": count })),
        );
        self.notify_models_updated();
    }

    /// Reconcile a Codex-only auth file change, refresh the new account's
    /// entitlement catalog, and retry transient discovery failures without
    /// disturbing xAI authentication.
    pub async fn on_openai_codex_auth_changed(&self) -> bool {
        self.inner.codex_catalog.on_auth_changed(self).await
    }

    /// Start provider-local startup recovery when credentials exist but the
    /// initial Codex prefetch did not produce an account-bound catalog.
    pub fn start_openai_codex_catalog_recovery(&self) {
        self.inner.codex_catalog.start_recovery(self);
    }

    /// Retry model catalog fetch in the background with exponential backoff.
    ///
    /// Spawned when `on_auth_changed` falls back to bundled defaults. Uses the
    /// crate-standard `execute_with_backoff` (5 attempts, 5s base, 60s cap) and
    /// notifies clients on success so the UI recovers after sleep/resume without
    /// requiring a manual restart.
    fn spawn_catalog_retry(&self) {
        // Deliberate no-fetch state: a retry loop can never succeed, so don't
        // start one (defensive re-check; the spawn site already gates).
        if !crate::util::config::resolve_remote_fetch_enabled() {
            return;
        }
        // Prevent overlapping retry loops.
        if self
            .inner
            .retry_in_flight
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            tracing::debug!("model catalog retry already in flight, skipping");
            return;
        }

        let mgr = self.clone();
        tokio::task::spawn(async move {
            let backoff = crate::tools::retry::BackoffConfig::new(5, 5_000, 60_000);

            let result = crate::tools::retry::execute_with_backoff(
                &backoff,
                || {
                    let mgr = mgr.clone();
                    async move {
                        // Bail out early if another code path already loaded a real catalog.
                        if *mgr.inner.has_xai_catalog.read() {
                            return Ok(());
                        }

                        mgr.fetch_and_apply().await;

                        if *mgr.inner.has_xai_catalog.read() {
                            Ok(())
                        } else {
                            Err("model catalog fetch returned no models")
                        }
                    }
                },
                |attempt, max_retries, delay| async move {
                    xai_grok_telemetry::unified_log::warn(
                        "model catalog: retry scheduled",
                        None,
                        Some(serde_json::json!({
                            "attempt": attempt,
                            "max_retries": max_retries,
                            "delay_ms": delay.as_millis() as u64,
                        })),
                    );
                },
            )
            .await;

            match result {
                Ok(()) => {
                    let count = mgr.available().len();
                    xai_grok_telemetry::unified_log::info(
                        "model catalog: retry succeeded",
                        None,
                        Some(serde_json::json!({ "model_count": count })),
                    );
                    mgr.notify_models_updated();
                }
                Err(e) => {
                    xai_grok_telemetry::unified_log::warn(
                        "model catalog: all retries exhausted",
                        None,
                        Some(serde_json::json!({ "error": e })),
                    );
                }
            }

            mgr.inner.retry_in_flight.store(false, Ordering::Release);
        });
    }

    /// Recover a missing credential-scoped xAI startup catalog without
    /// waiting for a later file/auth notification. This is required when an
    /// early prefetch is correctly discarded because its producing account
    /// changed before manager installation.
    pub fn start_xai_catalog_recovery(&self) {
        if !crate::util::config::resolve_remote_fetch_enabled()
            || *self.inner.has_xai_catalog.read()
        {
            return;
        }
        let fetch_auth = *self.inner.fetch_auth.read();
        let expected = match fetch_auth {
            ModelFetchAuth::Session => self.inner.auth_manager.current_or_expired().is_some(),
            ModelFetchAuth::ApiKey
            | ModelFetchAuth::Deployment
            | ModelFetchAuth::CustomEndpoint => true,
        };
        if expected {
            self.spawn_catalog_retry();
        }
    }

    /// Refresh the model catalog on every auth token refresh.
    ///
    /// Listens for [`AuthManager::refresh_notifier`] signals directly,
    /// bypassing the FSEvents file watcher which can silently stop
    /// delivering events on macOS after resume from sleep. On each
    /// notification the catalog is re-fetched from the server; if the
    /// fetch succeeds and the catalog changed, clients are notified
    /// via `x.ai/models/update`.
    pub fn start_auth_refresh_watcher(&self, notify: Arc<tokio::sync::Notify>) {
        let mgr = self.clone();
        let had_catalog_at_start = *self.inner.has_xai_catalog.read();
        xai_grok_telemetry::unified_log::info(
            "model catalog: auth refresh watcher started",
            None,
            Some(serde_json::json!({
                "had_real_catalog": had_catalog_at_start,
                "model_count": self.available().len(),
            })),
        );
        tokio::spawn(async move {
            loop {
                notify.notified().await;
                // Deliberate no-fetch state: skip the refresh entirely so the
                // failure-classifying logs below keep meaning "actually failed".
                if !crate::util::config::resolve_remote_fetch_enabled() {
                    tracing::debug!(
                        "model catalog: auth refresh watcher skipped (remote_fetch disabled)"
                    );
                    continue;
                }
                let had_catalog = *mgr.inner.has_xai_catalog.read();
                let old_count = mgr.available().len();
                xai_grok_telemetry::unified_log::info(
                    "model catalog: auth refresh watcher triggered",
                    None,
                    Some(serde_json::json!({
                        "had_real_catalog": had_catalog,
                        "model_count_before": old_count,
                    })),
                );
                // The notifier can represent an account switch, not merely a
                // same-account token rotation. Reconcile the installed scope
                // synchronously before any network await.
                mgr.on_auth_changed().await;
                let has_catalog = *mgr.inner.has_xai_catalog.read();
                let new_count = mgr.available().len();
                if has_catalog {
                    if !had_catalog || new_count != old_count {
                        xai_grok_telemetry::unified_log::info(
                            "model catalog: auth refresh watcher updated catalog",
                            None,
                            Some(serde_json::json!({
                                "model_count_before": old_count,
                                "model_count_after": new_count,
                                "was_recovery": !had_catalog,
                            })),
                        );
                    }
                    mgr.notify_models_updated();
                } else {
                    xai_grok_telemetry::unified_log::warn(
                        "model catalog: auth refresh watcher fetch failed",
                        None,
                        Some(serde_json::json!({
                            "model_count": old_count,
                        })),
                    );
                }
            }
        });
    }

    /// Wipe in-memory state so a previous identity's catalog doesn't leak.
    fn clear(&self) {
        let catalog_write = self.inner.catalog_write_gate.lock();
        self.clear_xai_catalog_unlocked();
        drop(catalog_write);
        self.notify_models_updated();
    }

    /// Clear only if the *current* config/auth context still represents an
    /// unauthenticated session. The check and mutation share the write gate so
    /// a concurrent API-key/custom-endpoint config reload cannot be erased by
    /// a stale auth-change callback.
    fn clear_if_unauthenticated_session(&self) -> bool {
        let catalog_write = self.inner.catalog_write_gate.lock();
        if *self.inner.fetch_auth.read() != ModelFetchAuth::Session
            || self.inner.auth_manager.current_or_expired().is_some()
        {
            return false;
        }
        self.clear_xai_catalog_unlocked();
        drop(catalog_write);
        self.notify_models_updated();
        true
    }

    /// Caller holds `catalog_write_gate`.
    fn clear_xai_catalog_unlocked(&self) {
        self.inner
            .xai_fetch_generation
            .fetch_add(1, Ordering::AcqRel);
        *self.inner.prefetched.write() = None;
        *self.inner.has_xai_catalog.write() = false;
        *self.inner.xai_catalog_credential_scope.write() = None;
        *self.inner.xai_catalog_cache_fetched_at.write() = None;
        *self.inner.etag.write() = None;
        let has_codex_catalog = self.inner.codex_catalog.has_catalog();
        *self.inner.has_fetched_real_catalog.write() = has_codex_catalog;
        let config = self.inner.cfg.read().clone();
        // xAI identity changes must not erase the independent ChatGPT Codex
        // account catalog or replace its credentials.
        self.rebuild_unlocked(&config, None);
        let excludes_all = allowlist_matches_nothing(&config, &self.inner.models.read());
        self.inner
            .allowlist_excludes_all
            .store(excludes_all, Ordering::Relaxed);
        self.reselect_current_model_if_missing(&config);
    }

    /// Build a `SamplingConfig` from the current model + auth state.
    pub fn sampling_config(&self) -> SamplingConfig {
        let config = self.inner.cfg.read().clone();
        let auth_manager = self.inner.auth_manager.as_ref();
        let current_model_id = self.current_model_id();
        let all_models = self.models();
        let fallback;
        let current_model = match all_models
            .get(current_model_id.0.as_ref())
            .or_else(|| all_models.values().next())
        {
            Some(m) => m,
            None => {
                tracing::warn!("no models available in catalog; defaulting to bundled model");
                let default_id = crate::models::default_model().to_string();
                fallback = ModelEntry::fallback(&default_id, &config.endpoints);
                &fallback
            }
        };

        let session_auth = auth_manager.current_or_expired();
        let credentials =
            resolve_credentials(current_model, session_auth.as_ref().map(|a| a.key.as_str()));

        sampling_config_for_model(
            current_model,
            credentials,
            config.endpoints.alpha_test_key.clone(),
            config.client_version.clone(),
            crate::managed_config::resolve_deployment_id(
                config.endpoints.deployment_key.as_deref(),
            ),
            None,
        )
    }

    /// Disk-cache origin key for this manager's current endpoints/auth shape
    /// (see [`ModelsCache::origin`]).
    fn cache_origin(&self) -> String {
        let endpoints = self.inner.cfg.read().endpoints.clone();
        let fetch_auth = *self.inner.fetch_auth.read();
        crate::remote::models_list_url(&endpoints, fetch_auth)
    }

    /// Capture one internally-consistent xAI fetch/cache context. Network
    /// launches supersede every older in-flight operation immediately.
    fn begin_xai_catalog_operation(&self) -> XaiCatalogSnapshot {
        let _catalog_write = self.inner.catalog_write_gate.lock();
        let config = self.inner.cfg.read().clone();
        let fetch_auth = *self.inner.fetch_auth.read();
        let origin = crate::remote::models_list_url(&config.endpoints, fetch_auth);
        let auth = self.inner.auth_manager.current_or_expired();
        let credential_scope =
            xai_catalog_credential_scope(&config.endpoints, auth.as_ref(), fetch_auth);
        let generation = self
            .inner
            .xai_fetch_generation
            .fetch_add(1, Ordering::AcqRel)
            .saturating_add(1);
        XaiCatalogSnapshot {
            generation,
            config,
            fetch_auth,
            origin,
            credential_scope,
        }
    }

    /// Re-attest the credential returned by `AuthManager::auth()` before a
    /// network request starts. `auth()` may itself refresh an external bearer
    /// or adopt a sibling-process credential, so the identity can differ from
    /// the launch snapshot. A different producer scope clears the installed
    /// catalog while holding the publication gate; a failed request for B can
    /// therefore never leave A's entitlements visible.
    fn prepare_xai_network_operation(
        &self,
        launch: &XaiCatalogSnapshot,
        auth: Option<&GrokAuth>,
    ) -> Option<(XaiCatalogSnapshot, bool)> {
        let _catalog_write = self.inner.catalog_write_gate.lock();
        if self.inner.xai_fetch_generation.load(Ordering::Acquire) != launch.generation {
            tracing::debug!(
                generation = launch.generation,
                "discarding superseded xAI model catalog launch"
            );
            return None;
        }

        let config = self.inner.cfg.read().clone();
        let fetch_auth = ModelFetchAuth::resolve(&config.endpoints, auth.is_some());
        let origin = crate::remote::models_list_url(&config.endpoints, fetch_auth);
        let credential_scope = xai_catalog_credential_scope(&config.endpoints, auth, fetch_auth);
        let fetch_shape_changed =
            *self.inner.fetch_auth.read() != fetch_auth || launch.origin != origin;
        let installed_scope_changed = *self.inner.has_xai_catalog.read()
            && self.inner.xai_catalog_credential_scope.read().as_ref() != credential_scope.as_ref();

        *self.inner.fetch_auth.write() = fetch_auth;
        let cleared = if installed_scope_changed {
            self.inner.cache.invalidate();
            self.clear_xai_catalog_unlocked();
            true
        } else {
            if fetch_shape_changed {
                self.inner
                    .xai_fetch_generation
                    .fetch_add(1, Ordering::AcqRel);
                self.inner.cache.invalidate();
            }
            false
        };

        Some((
            XaiCatalogSnapshot {
                generation: self.inner.xai_fetch_generation.load(Ordering::Acquire),
                config,
                fetch_auth,
                origin,
                credential_scope,
            },
            cleared,
        ))
    }

    /// Snapshot without superseding an in-flight fetch. Cache readers validate
    /// this again under the write gate; an already-started authoritative
    /// network response is deliberately allowed to win afterward.
    fn xai_catalog_snapshot(&self) -> XaiCatalogSnapshot {
        let _catalog_write = self.inner.catalog_write_gate.lock();
        let config = self.inner.cfg.read().clone();
        let fetch_auth = *self.inner.fetch_auth.read();
        let auth = self.inner.auth_manager.current_or_expired();
        let credential_scope =
            xai_catalog_credential_scope(&config.endpoints, auth.as_ref(), fetch_auth);
        XaiCatalogSnapshot {
            generation: self.inner.xai_fetch_generation.load(Ordering::Acquire),
            origin: crate::remote::models_list_url(&config.endpoints, fetch_auth),
            config,
            fetch_auth,
            credential_scope,
        }
    }

    /// Caller holds `catalog_write_gate`.
    fn xai_catalog_snapshot_is_current(&self, snapshot: &XaiCatalogSnapshot) -> bool {
        let config = self.inner.cfg.read();
        let fetch_auth = *self.inner.fetch_auth.read();
        let auth = self.inner.auth_manager.current_or_expired();
        let credential_scope =
            xai_catalog_credential_scope(&config.endpoints, auth.as_ref(), fetch_auth);
        self.inner.xai_fetch_generation.load(Ordering::Acquire) == snapshot.generation
            && fetch_auth == snapshot.fetch_auth
            && crate::remote::models_list_url(&config.endpoints, fetch_auth) == snapshot.origin
            && credential_scope == snapshot.credential_scope
    }

    fn try_load_cache(&self) -> bool {
        let snapshot = self.xai_catalog_snapshot();
        let Some(cached) = self.inner.cache.load_fresh(
            &snapshot.fetch_auth.cache_auth_method(),
            &snapshot.origin,
            snapshot.credential_scope.as_deref(),
        ) else {
            return false;
        };
        let _catalog_write = self.inner.catalog_write_gate.lock();
        if !self.xai_catalog_snapshot_is_current(&snapshot) {
            tracing::debug!("discarding xAI model cache loaded for stale config");
            return false;
        }
        if self.inner.xai_catalog_credential_scope.read().as_ref()
            == snapshot.credential_scope.as_ref()
            && self
                .inner
                .xai_catalog_cache_fetched_at
                .read()
                .is_some_and(|installed| cached.fetched_at < installed)
        {
            tracing::debug!("older xAI model cache load completed late; ignoring");
            return false;
        }
        let cfg = self.inner.cfg.read().clone();
        *self.inner.has_fetched_real_catalog.write() = true;
        *self.inner.has_xai_catalog.write() = true;
        *self.inner.xai_catalog_credential_scope.write() = snapshot.credential_scope.clone();
        *self.inner.xai_catalog_cache_fetched_at.write() = Some(cached.fetched_at);
        *self.inner.prefetched.write() = Some(cached.models.clone());
        self.rebuild_unlocked(&cfg, Some(cached.models));
        *self.inner.etag.write() = cached.etag;
        true
    }

    fn spawn_fetch(&self, new_etag: Option<String>) {
        // Degrade to Offline: keep serving the current (cache/static) catalog.
        if !crate::util::config::resolve_remote_fetch_enabled() {
            tracing::info!("model catalog refresh skipped: remote_fetch disabled");
            return;
        }
        let launch = self.begin_xai_catalog_operation();
        let auth_manager = self.inner.auth_manager.clone();
        let mgr = self.clone();

        tokio::task::spawn(async move {
            let auth = match auth_manager.auth().await {
                Ok(auth) => Some(auth),
                Err(_)
                    if launch.fetch_auth == ModelFetchAuth::Session
                        && auth_manager.current_or_expired().is_some() =>
                {
                    // A transient refresh failure is not an account change.
                    // Preserve the installed scope and do not attempt an
                    // unauthenticated request.
                    return;
                }
                Err(_) => None,
            };
            let Some((snapshot, cleared)) =
                mgr.prepare_xai_network_operation(&launch, auth.as_ref())
            else {
                return;
            };
            if cleared {
                mgr.notify_models_updated();
            }
            if snapshot.fetch_auth == ModelFetchAuth::Session && auth.is_none() {
                return;
            }
            let fetched = fetch_models_network_async(
                snapshot.config.endpoints.clone(),
                auth,
                snapshot.fetch_auth,
            )
            .await;
            let (new_prefetched, committed_etag) = match fetched {
                Some(fetched) => (Some(fetched.models), fetched.etag.or(new_etag)),
                None => (None, None),
            };
            if !mgr.apply_xai_refresh_result(&snapshot, new_prefetched, committed_etag) {
                return;
            }
            tracing::info!("models manager refreshed");
            mgr.notify_models_updated();
        });
    }

    /// Fetch models, rebuild state, and notify clients.
    fn do_refresh(&self, new_etag: Option<String>, strategy: RefreshStrategy) {
        match strategy {
            RefreshStrategy::Offline => {
                if self.try_load_cache() {
                    tracing::info!("models manager refreshed from cache (offline)");
                }
            }
            RefreshStrategy::OnlineIfUncached => {
                if self.try_load_cache() {
                    tracing::info!("models manager refreshed from cache (online_if_uncached)");
                    return;
                }
                self.spawn_fetch(new_etag);
            }
            RefreshStrategy::Online => {
                self.spawn_fetch(new_etag);
            }
        }
    }

    /// Resolve the model list: tries cache first, then fetches from the network.
    pub async fn list_models(&self, strategy: RefreshStrategy) {
        match strategy {
            RefreshStrategy::Offline => {
                self.try_load_cache();
            }
            RefreshStrategy::OnlineIfUncached => {
                if self.try_load_cache() {
                    return;
                }
                self.fetch_and_apply().await;
            }
            RefreshStrategy::Online => {
                self.fetch_and_apply().await;
            }
        }
    }

    async fn fetch_and_apply(&self) {
        self.fetch_and_apply_inner(crate::util::config::resolve_remote_fetch_enabled())
            .await
    }

    /// `remote_fetch_enabled` is a parameter so tests can drive the gate
    /// without touching on-disk config layers.
    async fn fetch_and_apply_inner(&self, remote_fetch_enabled: bool) {
        // Degrade to Offline: keep serving the current (cache/static) catalog.
        if !remote_fetch_enabled {
            tracing::info!("model catalog refresh skipped: remote_fetch disabled");
            return;
        }
        let launch = self.begin_xai_catalog_operation();
        let auth = match self.inner.auth_manager.auth().await {
            Ok(auth) => Some(auth),
            Err(_)
                if launch.fetch_auth == ModelFetchAuth::Session
                    && self.inner.auth_manager.current_or_expired().is_some() =>
            {
                return;
            }
            Err(_) => None,
        };
        let Some((snapshot, cleared)) = self.prepare_xai_network_operation(&launch, auth.as_ref())
        else {
            return;
        };
        if cleared {
            self.notify_models_updated();
        }
        if snapshot.fetch_auth == ModelFetchAuth::Session && auth.is_none() {
            return;
        }
        let has_auth = auth.is_some();
        let fetch_auth = snapshot.fetch_auth;
        xai_grok_telemetry::unified_log::info(
            "model catalog: fetching",
            None,
            Some(serde_json::json!({
                "has_auth": has_auth,
                "fetch_auth": format!("{fetch_auth:?}"),
            })),
        );
        let fetched =
            fetch_models_network_async(snapshot.config.endpoints.clone(), auth, fetch_auth).await;
        let (new_prefetched, response_etag) = match fetched {
            Some(fetched) => (Some(fetched.models), fetched.etag),
            None => (None, None),
        };
        let success = self.apply_xai_refresh_result(&snapshot, new_prefetched, response_etag);
        if success {
            xai_grok_telemetry::unified_log::info(
                "model catalog: fetch succeeded",
                None,
                Some(serde_json::json!({
                    "model_count": self.available().len(),
                })),
            );
        }
    }

    fn apply_refresh_result(
        &self,
        config: &config::Config,
        new_prefetched: Option<IndexMap<String, ModelEntry>>,
        new_etag: Option<String>,
    ) -> bool {
        self.apply_refresh_result_inner(Some(config), None, new_prefetched, new_etag)
    }

    fn apply_xai_refresh_result(
        &self,
        snapshot: &XaiCatalogSnapshot,
        new_prefetched: Option<IndexMap<String, ModelEntry>>,
        new_etag: Option<String>,
    ) -> bool {
        self.apply_refresh_result_inner(None, Some(snapshot), new_prefetched, new_etag)
    }

    fn apply_refresh_result_inner(
        &self,
        test_config: Option<&config::Config>,
        snapshot: Option<&XaiCatalogSnapshot>,
        new_prefetched: Option<IndexMap<String, ModelEntry>>,
        new_etag: Option<String>,
    ) -> bool {
        let _catalog_write = self.inner.catalog_write_gate.lock();
        if let Some(snapshot) = snapshot
            && !self.xai_catalog_snapshot_is_current(snapshot)
        {
            tracing::debug!(
                generation = snapshot.generation,
                "discarding stale xAI model catalog response"
            );
            return false;
        }

        let Some(new_prefetched) = new_prefetched else {
            tracing::warn!("model refresh failed, leaving existing models unchanged");
            xai_grok_telemetry::unified_log::warn(
                "model catalog refresh failed",
                None,
                Some(serde_json::json!({
                    "had_real_catalog": *self.inner.has_fetched_real_catalog.read(),
                })),
            );
            return false;
        };

        // Production fetches always rebuild against the live config protected
        // by the validated generation. Unit-only direct callers retain the
        // historical explicit-config seam.
        let live_config;
        let config = if snapshot.is_some() {
            live_config = self.inner.cfg.read().clone();
            &live_config
        } else {
            test_config.expect("direct catalog apply requires config")
        };
        let first_xai_catalog = {
            let mut flag = self.inner.has_xai_catalog.write();
            let was_first = !*flag;
            *flag = true;
            was_first
        };
        if let Some(snapshot) = snapshot {
            *self.inner.xai_catalog_credential_scope.write() = snapshot.credential_scope.clone();
        } else {
            // The direct catalog-publication seam used by synchronous tests
            // must establish the same credential binding as the production
            // snapshot path. Otherwise the next config reload mistakes the
            // freshly installed catalog for a cross-account transition and
            // clears it.
            let current_auth = self.inner.auth_manager.current_or_expired();
            let fetch_auth = ModelFetchAuth::resolve(&config.endpoints, current_auth.is_some());
            *self.inner.xai_catalog_credential_scope.write() =
                xai_catalog_credential_scope(&config.endpoints, current_auth.as_ref(), fetch_auth);
        }
        *self.inner.has_fetched_real_catalog.write() = true;
        if let Some(snapshot) = snapshot {
            // Persist only after generation/auth-origin validation, under the
            // same publication gate. A rejected pre-switch request can never
            // poison the shared cache and re-enter through the file watcher.
            let fetched_at = self.inner.cache.persist(
                &new_prefetched,
                new_etag.as_deref(),
                snapshot.fetch_auth.cache_auth_method(),
                &snapshot.origin,
                snapshot.credential_scope.as_deref(),
            );
            *self.inner.xai_catalog_cache_fetched_at.write() = Some(fetched_at);
        }
        *self.inner.prefetched.write() = Some(new_prefetched.clone());
        self.rebuild_unlocked(config, Some(new_prefetched));
        *self.inner.etag.write() = new_etag;

        // Can't exit a running app; flag it so the prompt path blocks instead.
        let excludes_all = allowlist_matches_nothing(config, &self.inner.models.read());
        self.inner
            .allowlist_excludes_all
            .store(excludes_all, Ordering::Relaxed);
        if excludes_all {
            tracing::error!("allowed_models excludes all fetched models; prompts will be blocked");
        }

        if first_xai_catalog {
            self.reselect_default_model(config);
        } else {
            self.reselect_current_model_if_missing(config);
        }
        true
    }

    pub fn allowlist_excludes_all(&self) -> bool {
        self.inner.allowlist_excludes_all.load(Ordering::Relaxed)
    }

    /// Re-pick the default if `current_model_id` is gone from the catalog *or*
    /// is no longer `user_selectable` (e.g. a config reload narrowed
    /// `allowed_models`), so UI and sampling don't disagree on the active model.
    fn reselect_current_model_if_missing(&self, config: &config::Config) {
        let current = self.inner.current_model_id.read().clone();
        let needs_reselection = {
            let models = self.inner.models.read();
            match models.get(current.0.as_ref()) {
                None => true,
                Some(entry) => !entry.info.user_selectable,
            }
        };
        if !needs_reselection {
            return;
        }
        let (key, _, source) = {
            let models = self.inner.models.read();
            resolve_default_model(config, &models, self.is_session_auth())
        };
        let new_id = acp::ModelId::new(Arc::from(key));
        tracing::info!(
            old = %current.0, new = %new_id.0, source = %source,
            "current model not in new catalog, reselecting default"
        );
        self.set_current_model_id(new_id);
    }

    /// Re-resolve the default model against the current catalog.
    ///
    /// Called on first catalog fetch and when `apply_config` detects a
    /// preferred-model change.
    fn reselect_default_model(&self, config: &config::Config) {
        let (key, _, source) = {
            let models = self.inner.models.read();
            resolve_default_model(config, &models, self.is_session_auth())
        };
        let new_id = acp::ModelId::new(Arc::from(key));
        let current = self.inner.current_model_id.read().clone();
        if current.0.as_ref() != new_id.0.as_ref() {
            tracing::info!(
                old = %current.0, new = %new_id.0, source = %source,
                "re-resolved default model after catalog populated"
            );
            self.set_current_model_id(new_id);
        }
    }
}

// ── Refresh strategy ────────────────────────────────────────────────────────

/// How to resolve the model list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshStrategy {
    /// Always fetch from network, ignore cache.
    Online,
    /// Only use cached data, never fetch.
    Offline,
    /// Use cache if fresh, otherwise fetch.
    OnlineIfUncached,
}

// ── Disk cache ──────────────────────────────────────────────────────────────

const MODELS_CACHE_FILE: &str = "models_cache.json";
const CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(300);

#[derive(serde::Serialize, serde::Deserialize)]
struct ModelsCache {
    fetched_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grok_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auth_method: Option<CacheAuthMethod>,
    /// One-way binding to the effective xAI account/API/deployment credential.
    /// Raw credential and account material is never stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    credential_scope: Option<String>,
    /// Models-list URL this catalog was fetched from
    /// ([`crate::remote::models_list_url`]). Compared on load so a cache
    /// written against one backend is a miss for another: entries embed
    /// absolute `base_url`s, so adopting a foreign-origin cache silently
    /// re-points inference (the windows lifecycle e2e failed exactly this
    /// way — test 1's mock-server catalog, cached in the shared profile,
    /// sent test 2's prompts to a dead port). `None` (legacy files) never
    /// matches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    origin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    etag: Option<String>,
    models: IndexMap<String, ModelEntry>,
}

impl ModelsCache {
    fn is_fresh(&self, ttl: std::time::Duration) -> bool {
        let Ok(ttl) = ChronoDuration::from_std(ttl) else {
            return false;
        };
        let age = Utc::now().signed_duration_since(self.fetched_at);
        age >= ChronoDuration::zero() && age < ttl
    }
}

struct CacheResult {
    fetched_at: DateTime<Utc>,
    models: IndexMap<String, ModelEntry>,
    etag: Option<String>,
}

struct ModelsCacheManager {
    path: std::path::PathBuf,
    ttl: std::time::Duration,
}

impl ModelsCacheManager {
    fn new() -> Self {
        Self {
            path: crate::util::grok_home::grok_home().join(MODELS_CACHE_FILE),
            ttl: CACHE_TTL,
        }
    }

    /// Sync; used by `prefetch_models_blocking`. Will be removed once startup
    /// prefetch is async.
    fn load_fresh(
        &self,
        expected_auth: &CacheAuthMethod,
        expected_origin: &str,
        expected_credential_scope: Option<&str>,
    ) -> Option<CacheResult> {
        let data = std::fs::read(&self.path).ok()?;
        let cache: ModelsCache = serde_json::from_slice(&data).ok()?;
        if cache.grok_version.as_deref() != Some(xai_grok_version::VERSION) {
            tracing::debug!("models cache version mismatch");
            return None;
        }
        if cache.auth_method.as_ref() != Some(expected_auth) {
            tracing::debug!("models cache auth method mismatch");
            return None;
        }
        if cache.origin.as_deref() != Some(expected_origin) {
            tracing::debug!(
                cached = ?cache.origin,
                expected = expected_origin,
                "models cache origin mismatch"
            );
            return None;
        }
        if cache.credential_scope.as_deref() != expected_credential_scope {
            tracing::debug!("models cache credential scope mismatch");
            return None;
        }
        if !cache.is_fresh(self.ttl) {
            tracing::debug!("models cache is stale");
            return None;
        }
        tracing::debug!(count = cache.models.len(), "loaded models from disk cache");
        Some(CacheResult {
            fetched_at: cache.fetched_at,
            models: cache.models,
            etag: cache.etag,
        })
    }

    /// Sync; see `load_fresh` note.
    fn persist(
        &self,
        models: &IndexMap<String, ModelEntry>,
        etag: Option<&str>,
        auth_method: CacheAuthMethod,
        origin: &str,
        credential_scope: Option<&str>,
    ) -> DateTime<Utc> {
        let fetched_at = Utc::now();
        let cache = ModelsCache {
            fetched_at,
            grok_version: Some(xai_grok_version::VERSION.to_string()),
            auth_method: Some(auth_method),
            credential_scope: credential_scope.map(ToOwned::to_owned),
            origin: Some(origin.to_string()),
            etag: etag.map(|s| s.to_string()),
            models: models.clone(),
        };
        self.atomic_write(&cache);
        fetched_at
    }

    fn renew_ttl(
        &self,
        expected_auth: &CacheAuthMethod,
        expected_origin: &str,
        expected_credential_scope: Option<&str>,
        expected_etag: &str,
    ) {
        let data = match std::fs::read(&self.path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
            Err(e) => {
                tracing::warn!(error = %e, "models cache TTL renewal: read failed");
                return;
            }
        };
        let Ok(mut cache) = serde_json::from_slice::<ModelsCache>(&data) else {
            return;
        };
        if cache.auth_method.as_ref() != Some(expected_auth) {
            tracing::debug!("models cache TTL renewal skipped: auth method mismatch");
            return;
        }
        if cache.origin.as_deref() != Some(expected_origin) {
            tracing::debug!("models cache TTL renewal skipped: origin mismatch");
            return;
        }
        if cache.credential_scope.as_deref() != expected_credential_scope {
            tracing::debug!("models cache TTL renewal skipped: credential scope mismatch");
            return;
        }
        if cache.etag.as_deref() != Some(expected_etag) {
            tracing::debug!("models cache TTL renewal skipped: ETag mismatch");
            return;
        }
        cache.fetched_at = Utc::now();
        self.atomic_write(&cache);
        tracing::debug!("models cache TTL renewed");
    }

    /// Sync; see `load_fresh` note.
    fn invalidate(&self) {
        match std::fs::remove_file(&self.path) {
            Ok(()) => tracing::info!("models disk cache invalidated"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => tracing::warn!(error = %e, "failed to invalidate models disk cache"),
        }
    }

    /// Sync; see `load_fresh` note.
    fn atomic_write(&self, cache: &ModelsCache) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = self.path.with_extension("json.tmp");
        if let Ok(json) = serde_json::to_vec_pretty(cache)
            && std::fs::write(&tmp, &json).is_ok()
        {
            let _ = std::fs::rename(&tmp, &self.path);
        }
    }
}

// ── Fetch ───────────────────────────────────────────────────────────────────

/// Build the prefetched model map from a flat list of entries.
///
/// Each entry is keyed by its `id` field (falling back to the `model` slug
/// when `id` is absent). This lets A/B experiments that share the same
/// routing slug (e.g. "Auto" and "Grok Build" both route to `grok-build`)
/// coexist in the catalog without collision.
fn build_prefetched_map(
    models: Vec<config::ModelEntryConfig>,
    api_base_url_override: Option<String>,
) -> IndexMap<String, ModelEntry> {
    let mut map: IndexMap<String, ModelEntry> = IndexMap::with_capacity(models.len());
    for m in models {
        let key = m.id.clone().unwrap_or_else(|| m.model.clone());
        let info = config::ModelInfo::from_config(&m);
        let entry = ModelEntry {
            info,
            api_key: None,
            env_key: None,
            api_base_url: m.api_base_url.clone().or(api_base_url_override.clone()),
        };
        map.insert(key, entry);
    }
    map
}

/// Fetch remote models. Checks disk cache first; persists after fetch.
pub(crate) fn prefetch_models_blocking(
    endpoints: &config::EndpointsConfig,
    auth: Option<&GrokAuth>,
    fetch_auth: ModelFetchAuth,
) -> Option<IndexMap<String, ModelEntry>> {
    prefetch_models_blocking_gated(
        endpoints,
        auth,
        fetch_auth,
        crate::util::config::resolve_remote_fetch_enabled(),
    )
}

/// Blocking models + `/v1/settings` prefetch pair, shared by the early
/// prefetch thread and the leader's startup phase so the settings gate lives
/// once. The remote_fetch knob is resolved a single time so the two fetch
/// decisions cannot disagree mid-startup.
pub(crate) fn prefetch_models_and_settings_blocking(
    endpoints: &config::EndpointsConfig,
    auth: Option<&GrokAuth>,
    fetch_auth: ModelFetchAuth,
) -> (
    Option<IndexMap<String, ModelEntry>>,
    Option<crate::util::config::RemoteSettings>,
) {
    let remote_fetch_enabled = crate::util::config::resolve_remote_fetch_enabled();
    let models = prefetch_models_blocking_gated(endpoints, auth, fetch_auth, remote_fetch_enabled);
    // Settings need a grok.com session; skip for BYOK.
    let settings = match auth {
        Some(auth) if remote_fetch_enabled => {
            let _timer = crate::instrumentation_timer!("startup.early_settings_fetch");
            crate::remote::fetch_settings_blocking(
                &endpoints.proxy_url(),
                auth,
                endpoints.alpha_test_key.as_deref(),
            )
        }
        _ => None,
    };
    (models, settings)
}

#[cfg(test)]
fn map_openai_codex_catalog(
    catalog: crate::remote::CodexModelCatalog,
) -> IndexMap<String, ModelEntry> {
    CodexCatalogController::map_catalog(catalog)
}

/// `remote_fetch_enabled` is a parameter so the pair helper above resolves the
/// knob once for both halves.
fn prefetch_models_blocking_gated(
    endpoints: &config::EndpointsConfig,
    auth: Option<&GrokAuth>,
    fetch_auth: ModelFetchAuth,
    remote_fetch_enabled: bool,
) -> Option<IndexMap<String, ModelEntry>> {
    let cache_auth = fetch_auth.cache_auth_method();
    // Same URL the fetch below will hit — the cache is only valid for it.
    let cache_origin = crate::remote::models_list_url(endpoints, fetch_auth);
    let credential_scope = xai_catalog_credential_scope(endpoints, auth, fetch_auth);
    let cache = ModelsCacheManager::new();
    if let Some(cached) = cache.load_fresh(&cache_auth, &cache_origin, credential_scope.as_deref())
    {
        return Some(cached.models);
    }

    // Every catalog fetch in the product funnels through here, so this single
    // gate also covers callers that don't go through the prefetch-env check
    // (leader, headless, stdio, server). Cache above is local and stays usable.
    if !remote_fetch_enabled {
        tracing::info!("models fetch skipped: remote_fetch disabled");
        return None;
    }

    let _timer = crate::instrumentation_timer!("startup.fetch_models_blocking");
    match fetch_models_blocking(endpoints, auth, fetch_auth) {
        Ok(FetchModelsResult { models, etag }) if !models.is_empty() => {
            let api_base_url_override = match fetch_auth {
                ModelFetchAuth::ApiKey => Some(endpoints.xai_api_base_url.clone()),
                _ => None,
            };
            let map = build_prefetched_map(models, api_base_url_override);

            // NOTE: inheriting context_window / agent_type / api_backend
            // from hardcoded defaults is handled centrally in
            // `resolve_model_list` (config.rs), not here. Don't re-add it.

            tracing::info!(count = map.len(), etag = ?etag, "Prefetched models");
            cache.persist(
                &map,
                etag.as_deref(),
                cache_auth,
                &cache_origin,
                credential_scope.as_deref(),
            );
            Some(map)
        }
        Ok(FetchModelsResult { .. }) => {
            tracing::warn!("Models endpoint returned empty list");
            None
        }
        Err(e) => {
            tracing::warn!("Failed to fetch models: {:?}", e);
            None
        }
    }
}

/// Startup prefetch result: models + remote settings.
pub struct EarlyPrefetchResult {
    pub models: Option<IndexMap<String, ModelEntry>>,
    pub settings: Option<crate::util::config::RemoteSettings>,
}

/// Handle for a startup prefetch thread.
pub type EarlyPrefetchHandle = std::thread::JoinHandle<EarlyPrefetchResult>;

struct PrefetchEnv {
    auth: Option<GrokAuth>,
    endpoints: config::EndpointsConfig,
    model_fetch_auth: ModelFetchAuth,
}

fn resolve_prefetch_env_with_auth(auth: Option<GrokAuth>) -> Option<PrefetchEnv> {
    let _timer = crate::instrumentation_timer!("startup.early_prefetch_launch");
    // Config-aware (not env-only) so the prefetch can't leak the bearer to api.x.ai.
    let mut endpoints = config::EndpointsConfig::from_effective_config();

    if endpoints.deployment_key.is_none() {
        endpoints.deployment_key = crate::managed_config::resolve_deployment_key();
    }

    resolve_prefetch_env_from_parts(
        auth,
        endpoints,
        crate::util::config::resolve_remote_fetch_enabled(),
    )
}

/// Decision core of [`resolve_prefetch_env_with_auth`], split from the config
/// loading so the gate is unit-testable.
///
/// `remote_fetch_enabled = false` wins over every credential shape AND over
/// `has_custom_endpoint()` (which otherwise forces the prefetch to run): the
/// explicit off switch must hold even when a stray login, `XAI_API_KEY`, or
/// `deployment_key` would re-arm the prefetch — and with it the `/v1/settings`
/// fetch and the deployment-config sync on the prefetch thread.
fn resolve_prefetch_env_from_parts(
    auth: Option<GrokAuth>,
    endpoints: config::EndpointsConfig,
    remote_fetch_enabled: bool,
) -> Option<PrefetchEnv> {
    if !remote_fetch_enabled {
        tracing::info!("startup model/settings prefetch skipped: remote_fetch disabled");
        return None;
    }

    let model_fetch_auth = ModelFetchAuth::resolve(&endpoints, auth.is_some());

    if auth.is_none()
        && !endpoints.has_custom_endpoint()
        && model_fetch_auth == ModelFetchAuth::Session
    {
        return None;
    }

    Some(PrefetchEnv {
        auth,
        endpoints,
        model_fetch_auth,
    })
}

fn resolve_prefetch_env(grok_com_config: Option<GrokComConfig>) -> Option<PrefetchEnv> {
    let grok_home = crate::util::grok_home::grok_home();
    let auth_manager = AuthManager::new(&grok_home, grok_com_config.unwrap_or_default());
    let auth = auth_manager.current();
    resolve_prefetch_env_with_auth(auth)
}

/// Start model + settings prefetch on a background thread using pre-resolved auth.
///
/// When the caller has already obtained valid credentials (e.g. via
/// `try_ensure_fresh_auth`), pass them here to avoid re-reading stale cached
/// credentials from disk.
pub fn start_early_prefetch_with_auth(auth: Option<GrokAuth>) -> Option<EarlyPrefetchHandle> {
    let env = resolve_prefetch_env_with_auth(auth)?;
    Some(spawn_prefetch_thread(env))
}

/// Start model + settings prefetch on a background thread.
///
/// Convenience wrapper that reads cached auth from disk. Prefer
/// `start_early_prefetch_with_auth` when you have pre-resolved credentials.
pub fn start_early_prefetch(grok_com_config: Option<GrokComConfig>) -> Option<EarlyPrefetchHandle> {
    let env = resolve_prefetch_env(grok_com_config)?;
    Some(spawn_prefetch_thread(env))
}

fn spawn_prefetch_thread(env: PrefetchEnv) -> EarlyPrefetchHandle {
    std::thread::spawn(move || {
        let mut timer = crate::instrumentation_timer!("startup.early_prefetch");
        let proxy_endpoint = env.endpoints.proxy_url();
        timer.with_field("endpoint", proxy_endpoint.as_str());
        let (models, settings) = prefetch_models_and_settings_blocking(
            &env.endpoints,
            env.auth.as_ref(),
            env.model_fetch_auth,
        );
        if (env.endpoints.deployment_key.is_some() || crate::managed_config::has_active_team_auth())
            && crate::config::is_managed_config_stale_for(
                &crate::managed_config::current_serving_identity(),
            )
            && crate::managed_config::is_fetch_enabled()
            && let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
        {
            crate::managed_config::clear_orphan();
            let _ = rt.block_on(crate::managed_config::sync());
        }

        EarlyPrefetchResult { models, settings }
    })
}

/// Map a model id (catalog key or routing slug) to its catalog key.
///
/// Sessions persist the routing slug (`[model.X].model`, e.g. `grok-4.5`);
/// the catalog and `/model` picker use config keys (e.g. `enterprise-grok-build`).
/// Last slug match wins so user overrides beat defaults (matches `MvpAgent::resolve_model_id`).
pub(crate) fn resolve_catalog_key(
    models: &IndexMap<String, ModelEntry>,
    id: &acp::ModelId,
) -> Option<acp::ModelId> {
    let id_str = id.0.as_ref();
    if models.contains_key(id_str) {
        return Some(id.clone());
    }
    models
        .iter()
        .rev()
        .find(|(_, entry)| entry.info.model == id_str)
        .map(|(key, _)| acp::ModelId::new(key.clone()))
}

/// Resolve either the catalog key or the provider routing slug without
/// allocating an ACP id. Last slug match wins, matching `resolve_catalog_key`
/// and `MvpAgent::resolve_model_id` so user overrides take precedence.
fn model_entry_by_id_or_slug<'a>(
    models: &'a IndexMap<String, ModelEntry>,
    id: &str,
) -> Option<&'a ModelEntry> {
    models.get(id).or_else(|| {
        models
            .iter()
            .rev()
            .find(|(_, entry)| entry.info.model == id)
            .map(|(_, entry)| entry)
    })
}

fn model_entry_by_id_or_slug_for_provider<'a>(
    models: &'a IndexMap<String, ModelEntry>,
    id: &str,
    provider: ProviderId,
) -> Option<&'a ModelEntry> {
    models
        .get(id)
        .filter(|entry| entry.info.provider == provider)
        .or_else(|| {
            models
                .iter()
                .rev()
                .find(|(_, entry)| entry.info.provider == provider && entry.info.model == id)
                .map(|(_, entry)| entry)
        })
}

/// Catalog key for a persisted session model id, restricted to **selectable**
/// entries. A selectable exact-key match wins (as in [`resolve_catalog_key`]);
/// otherwise the last selectable entry whose routing slug matches `id`, so a
/// non-selectable exact-key entry never shadows a selectable slug match.
pub(crate) fn selectable_catalog_key_for_persisted(
    models: &IndexMap<String, ModelEntry>,
    available: &IndexMap<acp::ModelId, acp::ModelInfo>,
    id: &acp::ModelId,
    is_session_auth: bool,
) -> Option<acp::ModelId> {
    let restorable = |key: &str, entry: &ModelEntry| {
        available.contains_key(&acp::ModelId::new(key.to_owned()))
            || (entry.info.hidden
                && entry.info.user_selectable
                && model_addressable_for_auth(&entry.info, is_session_auth))
    };
    if let Some(entry) = models.get(id.0.as_ref())
        && restorable(id.0.as_ref(), entry)
    {
        return Some(id.clone());
    }
    let id_str = id.0.as_ref();
    if let Some((key, _)) = models
        .iter()
        .rev()
        .find(|(key, entry)| restorable(key, entry) && entry.info.model == id_str)
    {
        return Some(acp::ModelId::new(key.clone()));
    }
    None
}

/// Provider-scoped variant of [`selectable_catalog_key_for_persisted`].
///
/// This is used when a persisted id itself carries a provider identity (for
/// example `openai-codex/...`). A routing-slug collision must never let a
/// session cross that provider boundary while it is being restored.
pub(crate) fn selectable_catalog_key_for_persisted_provider(
    models: &IndexMap<String, ModelEntry>,
    available: &IndexMap<acp::ModelId, acp::ModelInfo>,
    id: &acp::ModelId,
    is_session_auth: bool,
    provider: ProviderId,
) -> Option<acp::ModelId> {
    let restorable = |key: &str, entry: &ModelEntry| {
        entry.info.provider == provider
            && (available.contains_key(&acp::ModelId::new(key.to_owned()))
                || (entry.info.hidden
                    && entry.info.user_selectable
                    && model_addressable_for_auth(&entry.info, is_session_auth)))
    };
    if let Some(entry) = models.get(id.0.as_ref())
        && restorable(id.0.as_ref(), entry)
    {
        return Some(id.clone());
    }
    let id_str = id.0.as_ref();
    models
        .iter()
        .rev()
        .find(|(key, entry)| restorable(key, entry) && entry.info.model == id_str)
        .map(|(key, _)| acp::ModelId::new(key.clone()))
}

/// First picker-visible model belonging to `provider`.
///
/// Provider-qualified session recovery uses this only after the persisted
/// model is no longer entitled. Keeping the filter here prevents a merged
/// catalog's insertion order from turning a provider-local fallback into an
/// xAI/Codex cross-over.
pub(crate) fn first_available_catalog_key_for_provider(
    models: &IndexMap<String, ModelEntry>,
    available: &IndexMap<acp::ModelId, acp::ModelInfo>,
    provider: ProviderId,
) -> Option<acp::ModelId> {
    models.iter().find_map(|(key, entry)| {
        let id = acp::ModelId::new(key.clone());
        (entry.info.provider == provider && available.contains_key(&id)).then_some(id)
    })
}

/// A "campaign-only" preferred flip: the default changed and either side's value
/// is an active campaign default, i.e. the change is attributable to a campaign
/// overlay appearing/disappearing rather than a user/CLI/env edit.
fn is_campaign_only_flip(
    old_preferred: &Option<String>,
    new_preferred: &Option<String>,
    campaign_defaults: &std::collections::HashSet<String>,
) -> bool {
    if new_preferred == old_preferred || new_preferred.is_none() {
        return false;
    }
    new_preferred
        .as_ref()
        .is_some_and(|p| campaign_defaults.contains(p))
        || old_preferred
            .as_ref()
            .is_some_and(|p| campaign_defaults.contains(p))
}

/// Pick the default model: CLI > env > config > remote-settings hint, falling
/// back to the bundled default when the catalog is empty or the preferred
/// model isn't present.
pub(crate) fn resolve_default_model(
    cfg: &config::Config,
    catalog: &IndexMap<String, ModelEntry>,
    is_session_auth: bool,
) -> (String, ModelEntry, config::ConfigSource) {
    let addressable: IndexMap<String, ModelEntry> = catalog
        .iter()
        .filter(|(_, e)| {
            model_addressable_for_auth(&e.info, is_session_auth) && e.info.user_selectable
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let visible: IndexMap<String, ModelEntry> = addressable
        .iter()
        .filter(|(_, entry)| !entry.info.hidden)
        .map(|(key, entry)| (key.clone(), entry.clone()))
        .collect();

    let model_pref = config::resolve_string_flag(
        cfg.default_model_override.as_deref(),
        "GROK_DEFAULT_MODEL",
        cfg.models.default.as_deref(),
        cfg.remote_settings
            .as_ref()
            .and_then(|rs| rs.default_model.as_deref()),
    );

    let first_or_fallback = || -> (String, ModelEntry) {
        if let Some((key, first)) = visible.first() {
            return (key.clone(), first.clone());
        }
        if let Some((key, entry)) = addressable.first() {
            tracing::warn!("no picker-visible model; using first explicitly addressable entry");
            return (key.clone(), entry.clone());
        }
        // Pre-catalog/degenerate only: nothing selectable. Set the bundled
        // default's flag from `allowed_models` so no reader treats it as allowed.
        tracing::warn!("no selectable models; falling back to bundled default (pre-catalog)");
        let default_id = crate::models::default_model().to_string();
        let mut entry = ModelEntry::fallback(&default_id, &cfg.endpoints);
        entry.info.user_selectable = match ModelGlobSet::compile(cfg.models.allowed_models.as_ref())
        {
            Ok(None) => true,
            Ok(Some(set)) => set.matches(&default_id, &default_id),
            Err(_) => false,
        };
        (default_id, entry)
    };

    match &model_pref {
        None => {
            let (key, first) = first_or_fallback();
            (key, first, config::ConfigSource::Default)
        }
        Some(pref) => {
            let is_explicit = matches!(
                pref.source,
                config::ConfigSource::Cli | config::ConfigSource::Env
            ) || (matches!(pref.source, config::ConfigSource::Config)
                && !cfg.models.default_is_campaign_driven);
            // Backend `hide` means "omit from picker", not "unusable". Honor
            // it for an explicit CLI/env/config selection while keeping
            // remote and campaign defaults picker-visible only.
            let candidates = if is_explicit { &addressable } else { &visible };
            let found = candidates
                .get_key_value(&pref.value)
                .or_else(|| candidates.iter().find(|(_, m)| m.model == pref.value));

            if let Some((key, entry)) = found {
                (key.clone(), entry.clone(), pref.source)
            } else {
                if is_explicit {
                    tracing::warn!(
                        model_id = %pref.value, source = %pref.source,
                        "preferred model not in available models, falling back"
                    );
                } else {
                    tracing::debug!(
                        model_id = %pref.value, source = %pref.source,
                        "remote default_model not in available models, skipping"
                    );
                }
                // A campaign default missing from the catalog falls back to the
                // pre-campaign default before the first-visible fallback. Gated
                // on the missing pref actually being the campaign-driven config
                // value — a CLI/env pref that misses the catalog is not a
                // campaign problem and must not detour through campaign state.
                let campaign_pref_missing = cfg.models.default_is_campaign_driven
                    && matches!(pref.source, config::ConfigSource::Config);
                if campaign_pref_missing
                    && let Some(prev) = cfg
                        .models
                        .pre_campaign_default
                        .as_deref()
                        .filter(|s| !s.is_empty())
                    && let Some((key, entry)) = visible
                        .get_key_value(prev)
                        .or_else(|| visible.iter().find(|(_, m)| m.model == prev))
                {
                    tracing::info!(
                        unavailable = %pref.value, fallback = %prev,
                        "campaign-driven default unavailable in catalog; recovering the pre-campaign default"
                    );
                    return (key.clone(), entry.clone(), config::ConfigSource::Config);
                }
                let (key, first) = first_or_fallback();
                (key, first, config::ConfigSource::Default)
            }
        }
    }
}

/// Filter hidden and auth-gated entries out of `catalog` and convert to ACP wire format.
pub fn available_models(
    catalog: &IndexMap<String, ModelEntry>,
    is_session_auth: bool,
) -> IndexMap<acp::ModelId, acp::ModelInfo> {
    let visible: IndexMap<String, ModelEntry> = catalog
        .iter()
        .filter(|(_, e)| model_visible_for_auth(&e.info, is_session_auth))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    config::to_acp_model_info(&visible)
}

/// Compiled glob matcher shared by `allowed_models`, `disabled_models`, and
/// `hidden_models`. Patterns (globset syntax: `*`, `?`, `[...]`) are matched
/// against either the catalog key or the model id.
pub(crate) struct ModelGlobSet(GlobSet);

impl ModelGlobSet {
    /// Compile a filter list (`Ok(None)` for `None`/empty). Fails **closed**: an
    /// invalid pattern returns `Err` listing every bad one for config to reject.
    pub(crate) fn compile(patterns: Option<&Vec<String>>) -> Result<Option<Self>, Vec<String>> {
        let patterns = match patterns {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };
        let mut builder = GlobSetBuilder::new();
        let mut invalid = Vec::new();
        for pat in patterns {
            match Glob::new(pat) {
                Ok(glob) => {
                    builder.add(glob);
                }
                Err(_) => invalid.push(pat.clone()),
            }
        }
        if !invalid.is_empty() {
            return Err(invalid);
        }
        builder
            .build()
            .map(|set| Some(Self(set)))
            .map_err(|e| vec![e.to_string()])
    }

    fn matches(&self, key: &str, model: &str) -> bool {
        self.0.is_match(key) || self.0.is_match(model)
    }
}

/// Single source of truth for the catalog. Applies, in order: `disabled_models`
/// (remove), `allowed_models` (mark `user_selectable`), `hidden_models` (mark
/// `hidden`). Special/internal models (web_search, subagents, …) resolve via
/// `find_model_by_id`/`models()` and ignore `user_selectable`, so they need no
/// exemption. Globs are validated at load (`Config::validate_model_filters`);
/// the arms here fail closed if one slips through.
pub fn resolve_model_catalog(
    cfg: &config::Config,
    prefetched: Option<IndexMap<String, ModelEntry>>,
) -> IndexMap<String, ModelEntry> {
    resolve_combined_model_catalog(cfg, prefetched, IndexMap::new())
}

/// Resolve xAI/custom models first, then merge the independent authenticated
/// Codex catalog and apply one common visibility/allowlist/effort policy pass.
/// Codex entries are namespaced and provider-authoritative, so a custom model
/// cannot silently turn a ChatGPT entitlement into static API-key traffic.
fn resolve_combined_model_catalog(
    cfg: &config::Config,
    prefetched: Option<IndexMap<String, ModelEntry>>,
    codex_models: IndexMap<String, ModelEntry>,
) -> IndexMap<String, ModelEntry> {
    let mut catalog: IndexMap<String, ModelEntry> = config::resolve_model_list(cfg, prefetched);
    // Subscription models are provider-authoritative. Never let a static
    // custom-model entry impersonate an entitlement or survive logout.
    catalog.retain(|_, entry| entry.info.provider != ProviderId::OpenAiCodex);
    catalog.extend(codex_models);

    let fast_mode_enabled = cfg.features.fast_mode.unwrap_or(true);
    for entry in catalog
        .values_mut()
        .filter(|entry| entry.info.provider == ProviderId::OpenAiCodex)
    {
        entry.info.service_tier =
            resolve_model_service_tier(&entry.info, cfg.service_tier.as_deref(), fast_mode_enabled);
    }

    if let Ok(Some(disabled)) = ModelGlobSet::compile(cfg.models.disabled_models.as_ref()) {
        let before = catalog.len();
        catalog.retain(|key, entry| !disabled.matches(key, &entry.model));
        let removed = before - catalog.len();
        if removed > 0 {
            tracing::info!(count = removed, "disabled_models: removed from catalog");
        }
    }

    // None/empty allowlist = allow all.
    match ModelGlobSet::compile(cfg.models.allowed_models.as_ref()) {
        // Preserve provider-level addressability. In particular Codex
        // visibility `none` must not be promoted by an absent allowlist.
        Ok(None) => {}
        Ok(Some(allowed)) => {
            for (key, entry) in catalog.iter_mut() {
                entry.info.user_selectable &= allowed.matches(key, &entry.model);
            }
        }
        Err(bad) => {
            tracing::error!(patterns = ?bad, "allowed_models: invalid glob(s); marking nothing selectable");
            for entry in catalog.values_mut() {
                entry.info.user_selectable = false;
            }
        }
    }

    if let Ok(Some(hidden)) = ModelGlobSet::compile(cfg.models.hidden_models.as_ref()) {
        for (key, entry) in catalog.iter_mut() {
            if hidden.matches(key, &entry.model) {
                entry.info.hidden = true;
            }
        }
    }

    // A direct per-model override may independently replace the scalar default
    // and the advertised menu. Reconcile that pair before any global override
    // is applied so a stale value (for example Luna + ultra) can never reach a
    // sampler config. Prefer the model-advertised default, then its first menu
    // entry; legacy models without a menu fall back to omission.
    for entry in catalog.values_mut() {
        let Some(configured) = entry.info.reasoning_effort else {
            continue;
        };
        match resolve_model_reasoning_effort(&entry.info, configured) {
            Some(effective) => entry.info.reasoning_effort = Some(effective),
            None => {
                entry.info.reasoning_effort = advertised_default_reasoning_effort(&entry.info);
                tracing::warn!(
                    model = %entry.info.model,
                    rejected_effort = ?configured,
                    fallback_effort = ?entry.info.reasoning_effort,
                    "configured reasoning effort is not advertised by model; using catalog default"
                );
            }
        }
    }

    // Persisted default first; CLI override below wins when set. A support
    // boolean is not sufficient because dynamic catalogs advertise a distinct
    // menu for each model.
    if let Some(effort) = cfg.models.default_reasoning_effort
        && let Some(default_id) = cfg.models.default.as_deref()
        && let Some(entry) = catalog.get_mut(default_id)
        && let Some(effective) = resolve_model_reasoning_effort(&entry.info, effort)
    {
        entry.info.reasoning_effort = Some(effective);
    }

    // Skip non-reasoning models so we don't send the field to providers that reject it.
    // Also skip models whose effort menu does not include the override (e.g. `--effort none`
    // must not stamp `none` onto grok-4.5, which only offers low/medium/high).
    if let Some(effort) = cfg.reasoning_effort_override {
        for entry in catalog.values_mut() {
            if let Some(effective) = resolve_model_reasoning_effort(&entry.info, effort) {
                entry.info.reasoning_effort = Some(effective);
            }
        }
    }

    catalog
}

/// Whether `effort` is a value this model will accept on the wire.
///
/// Uses the server `reasoning_efforts` menu when present; otherwise the
/// built-in low/medium/high/xhigh set (same as the pager legacy menu — no
/// `none`/`minimal`).
fn model_info_offers_reasoning_effort(info: &config::ModelInfo, effort: ReasoningEffort) -> bool {
    resolve_model_reasoning_effort(info, effort).is_some()
}

/// Return the effective effort accepted by `info`. Codex and Kimi Code own
/// distinct extended effort contracts. For xAI/custom providers, the
/// historical Grok Build `max` alias remains equivalent to Xhigh.
fn resolve_model_reasoning_effort(
    info: &config::ModelInfo,
    effort: ReasoningEffort,
) -> Option<ReasoningEffort> {
    if !info.supports_reasoning_effort {
        return None;
    }
    let effective = if matches!(info.provider, ProviderId::Xai | ProviderId::Custom)
        && effort == ReasoningEffort::Max
    {
        ReasoningEffort::Xhigh
    } else {
        effort
    };
    if info.reasoning_efforts.is_empty() {
        matches!(
            effective,
            ReasoningEffort::Low
                | ReasoningEffort::Medium
                | ReasoningEffort::High
                | ReasoningEffort::Xhigh
        )
        .then_some(effective)
    } else {
        info.reasoning_efforts
            .iter()
            .any(|opt| opt.value == effective)
            .then_some(effective)
    }
}

fn advertised_default_reasoning_effort(info: &config::ModelInfo) -> Option<ReasoningEffort> {
    info.reasoning_efforts
        .iter()
        .find(|option| option.default)
        .or_else(|| info.reasoning_efforts.first())
        .map(|option| option.value)
        .and_then(|effort| resolve_model_reasoning_effort(info, effort))
}

fn normalize_service_tier_alias(value: &str) -> &str {
    if value.eq_ignore_ascii_case("fast") {
        xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER
    } else {
        value
    }
}

fn resolve_model_service_tier(
    info: &config::ModelInfo,
    configured: Option<&str>,
    fast_mode_enabled: bool,
) -> Option<String> {
    if info.provider != ProviderId::OpenAiCodex || !fast_mode_enabled {
        return None;
    }

    if let Some(configured) = configured {
        let configured = normalize_service_tier_alias(configured.trim());
        if configured == xai_grok_sampling_types::OPENAI_CODEX_STANDARD_SERVICE_TIER {
            return Some(configured.to_owned());
        }
        return info
            .service_tiers
            .iter()
            .any(|tier| tier.id == configured)
            .then(|| configured.to_owned());
    }

    // Keep Fast mode opt-in. The catalog default is capability metadata and must
    // not silently turn on the higher-usage priority tier for a user who did not
    // select a service tier.
    None
}

/// True when an active `allowed_models` allowlist leaves no selectable model.
/// (An excluded *default* does not count — that is recoverable by reselection.)
pub(crate) fn allowlist_matches_nothing(
    cfg: &config::Config,
    catalog: &IndexMap<String, ModelEntry>,
) -> bool {
    cfg.models
        .allowed_models
        .as_ref()
        .is_some_and(|a| !a.is_empty())
        && !catalog.values().any(|e| e.info.user_selectable)
}

/// Reject an `allowed_models` allowlist that leaves no selectable model, or that
/// excludes an explicitly configured default (`default`/`-m`). Run only against a
/// real catalog (cache/prefetch/fetched), not the bundled bootstrap set.
pub(crate) fn validate_selectable(
    cfg: &config::Config,
    catalog: &IndexMap<String, ModelEntry>,
) -> Result<(), String> {
    let Some(allowed) = cfg.models.allowed_models.as_ref().filter(|a| !a.is_empty()) else {
        return Ok(());
    };
    let patterns = allowed.join(", ");
    if !catalog.values().any(|e| e.info.user_selectable) {
        return Err(format!(
            "None of your available models match allowed_models ({patterns}). \
             Broaden the patterns or remove allowed_models, then try again."
        ));
    }
    for (src, id) in [
        ("default", cfg.models.default.as_deref()),
        ("-m flag", cfg.default_model_override.as_deref()),
    ] {
        if let Some(id) = id
            && let Some(entry) = catalog
                .get(id)
                .or_else(|| catalog.values().find(|e| e.model == id))
            && !entry.info.user_selectable
        {
            return Err(format!(
                "\"{id}\" (your {src}) isn't allowed by allowed_models ({patterns}). \
                 Add it to allowed_models, or set a different model."
            ));
        }
    }
    Ok(())
}

/// Network-only xAI catalog fetch. Runtime `Online` refreshes must not consult
/// or persist the cache before their generation/auth-origin snapshot is
/// validated by `ModelsManager`; startup retains the cache-first helper above.
async fn fetch_models_network_async(
    endpoints: config::EndpointsConfig,
    auth: Option<GrokAuth>,
    fetch_auth: ModelFetchAuth,
) -> Option<XaiNetworkCatalog> {
    tokio::task::spawn_blocking(move || {
        let FetchModelsResult { models, etag } =
            fetch_models_blocking(&endpoints, auth.as_ref(), fetch_auth).ok()?;
        if models.is_empty() {
            tracing::warn!("Models endpoint returned empty list");
            return None;
        }
        let api_base_url_override = match fetch_auth {
            ModelFetchAuth::ApiKey => Some(endpoints.xai_api_base_url.clone()),
            _ => None,
        };
        let models = build_prefetched_map(models, api_base_url_override);
        tracing::info!(count = models.len(), etag = ?etag, "Fetched models for runtime refresh");
        Some(XaiNetworkCatalog { models, etag })
    })
    .await
    .unwrap_or(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manager() -> ModelsManager {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
        // Use a temp dir so AuthManager finds no credentials — ensures
        // refresh_async bails at the auth check without needing a tokio runtime.
        let tmp = std::env::temp_dir().join("grok-test-models-manager");
        let auth_manager = Arc::new(AuthManager::new(&tmp, GrokComConfig::default()));
        ModelsManager::new(
            None,
            IndexMap::new(),
            acp::ModelId::new("default"),
            auth_manager,
            config::Config::default(),
        )
    }

    fn config_from_toml(toml: &str) -> config::Config {
        config::Config::new_from_toml_cfg(&toml::from_str(toml).unwrap()).unwrap()
    }

    fn gpt_56_codex_catalog() -> IndexMap<String, ModelEntry> {
        use crate::remote::{
            CodexCatalogModel, CodexModelCatalog, CodexModelServiceTier, CodexReasoningEffortPreset,
        };

        let model = |slug: &str, default: &str, efforts: &[&str]| CodexCatalogModel {
            id: format!("openai-codex/{slug}"),
            slug: slug.to_owned(),
            display_name: slug.to_owned(),
            default_reasoning_level: Some(default.to_owned()),
            supported_reasoning_levels: efforts
                .iter()
                .map(|effort| CodexReasoningEffortPreset {
                    effort: (*effort).to_owned(),
                    description: format!("{effort} reasoning"),
                })
                .collect(),
            visibility: Some("list".to_owned()),
            supported_in_api: true,
            context_window: Some(272_000),
            max_context_window: Some(272_000),
            input_modalities: vec!["text".to_owned(), "image".to_owned()],
            use_responses_lite: true,
            multi_agent_version: Some(serde_json::json!("v2")),
            service_tiers: vec![CodexModelServiceTier {
                id: xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER.to_owned(),
                name: "Fast".to_owned(),
                description: "1.5x speed, increased usage".to_owned(),
            }],
            ..Default::default()
        };
        let through_ultra = ["low", "medium", "high", "xhigh", "max", "ultra"];
        let through_max = ["low", "medium", "high", "xhigh", "max"];
        map_openai_codex_catalog(CodexModelCatalog {
            models: vec![
                model("gpt-5.6-sol", "low", &through_ultra),
                model("gpt-5.6-terra", "medium", &through_ultra),
                model("gpt-5.6-luna", "medium", &through_max),
            ],
        })
    }

    #[test]
    fn codex_fast_alias_is_applied_only_when_catalog_and_feature_allow_it() {
        let mut catalog = gpt_56_codex_catalog();
        let unsupported_id = "openai-codex/gpt-5.6-no-fast";
        let mut unsupported = catalog["openai-codex/gpt-5.6-luna"].clone();
        unsupported.info.id = Some(unsupported_id.to_owned());
        unsupported.info.model = "gpt-5.6-no-fast".to_owned();
        unsupported.info.service_tiers.clear();
        unsupported.info.default_service_tier = None;
        catalog.insert(unsupported_id.to_owned(), unsupported);

        let mut cfg = config_from_toml(
            r#"
            service_tier = "fast"

            [features]
            fast_mode = true
            "#,
        );
        let resolved = resolve_combined_model_catalog(&cfg, None, catalog.clone());
        assert_eq!(
            resolved["openai-codex/gpt-5.6-luna"]
                .info
                .service_tier
                .as_deref(),
            Some(xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER),
        );
        assert_eq!(resolved[unsupported_id].info.service_tier, None);

        cfg.features.fast_mode = Some(false);
        let disabled = resolve_combined_model_catalog(&cfg, None, catalog);
        assert!(
            disabled
                .values()
                .filter(|entry| entry.info.provider == ProviderId::OpenAiCodex)
                .all(|entry| entry.info.service_tier.is_none())
        );
    }

    #[test]
    fn codex_standard_sentinel_overrides_catalog_default_without_auto_enabling_fast() {
        let mut catalog = gpt_56_codex_catalog();
        catalog
            .get_mut("openai-codex/gpt-5.6-luna")
            .unwrap()
            .info
            .default_service_tier =
            Some(xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER.to_owned());

        let defaulted =
            resolve_combined_model_catalog(&config::Config::default(), None, catalog.clone());
        assert_eq!(
            defaulted["openai-codex/gpt-5.6-luna"].info.service_tier, None,
            "an advertised catalog default must not silently opt the user into Fast",
        );

        let cfg = config::Config {
            service_tier: Some("default".to_owned()),
            ..Default::default()
        };
        let standard = resolve_combined_model_catalog(&cfg, None, catalog);
        assert_eq!(
            standard["openai-codex/gpt-5.6-luna"]
                .info
                .service_tier
                .as_deref(),
            Some(xai_grok_sampling_types::OPENAI_CODEX_STANDARD_SERVICE_TIER),
            "explicit Standard must override a catalog Fast default",
        );
    }

    #[test]
    fn codex_service_tier_manager_resolution_is_provider_qualified() {
        let manager = manager_with_catalog(gpt_56_codex_catalog());
        let fast = manager
            .fast_service_tier_for_provider(ProviderId::OpenAiCodex, "gpt-5.6-luna")
            .expect("Luna advertises Fast in this catalog fixture");
        assert_eq!(
            fast.id,
            xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER
        );
        assert_eq!(
            manager.resolve_service_tier_for_provider(
                ProviderId::OpenAiCodex,
                "gpt-5.6-luna",
                "fast",
            ),
            Some(xai_grok_sampling_types::OPENAI_CODEX_FAST_SERVICE_TIER.to_owned()),
        );
        assert_eq!(
            manager.resolve_service_tier_for_provider(
                ProviderId::OpenAiCodex,
                "gpt-5.6-luna",
                "default",
            ),
            Some(xai_grok_sampling_types::OPENAI_CODEX_STANDARD_SERVICE_TIER.to_owned()),
        );
        assert_eq!(
            manager.resolve_service_tier_for_provider(
                ProviderId::OpenAiCodex,
                "gpt-5.6-luna",
                "unadvertised",
            ),
            None,
        );
        assert!(
            manager
                .fast_service_tier_for_provider(ProviderId::Xai, "gpt-5.6-luna")
                .is_none()
        );

        let mut disabled_config = config::Config::default();
        disabled_config.features.fast_mode = Some(false);
        let disabled = manager_with_catalog_and_config(gpt_56_codex_catalog(), disabled_config);
        assert!(!disabled.fast_mode_enabled());
        assert!(
            disabled
                .fast_service_tier_for_provider(ProviderId::OpenAiCodex, "gpt-5.6-luna")
                .is_none()
        );
        assert_eq!(
            disabled.resolve_service_tier_for_provider(
                ProviderId::OpenAiCodex,
                "gpt-5.6-luna",
                "fast",
            ),
            None,
            "the feature kill switch must reject a restored Fast selection",
        );
    }

    fn manager_with_catalog(catalog: IndexMap<String, ModelEntry>) -> ModelsManager {
        manager_with_catalog_and_config(catalog, config::Config::default())
    }

    fn manager_with_catalog_and_config(
        catalog: IndexMap<String, ModelEntry>,
        cfg: config::Config,
    ) -> ModelsManager {
        let tmp = std::env::temp_dir().join("grok-test-models-manager-gpt-56");
        ModelsManager::new(
            None,
            catalog,
            acp::ModelId::new("openai-codex/gpt-5.6-luna"),
            Arc::new(AuthManager::new(&tmp, GrokComConfig::default())),
            cfg,
        )
    }

    #[test]
    fn codex_image_input_capability_tracks_catalog_modalities_on_every_route_selection() {
        use crate::remote::{CodexCatalogModel, CodexModelCatalog};
        use xai_grok_sampling_types::ImageInputCapability;

        let catalog = map_openai_codex_catalog(CodexModelCatalog {
            models: vec![
                CodexCatalogModel {
                    id: "openai-codex/vision-coding".to_owned(),
                    slug: "vision-coding".to_owned(),
                    display_name: "Vision Coding".to_owned(),
                    visibility: Some("list".to_owned()),
                    input_modalities: vec!["text".to_owned(), "image".to_owned()],
                    ..Default::default()
                },
                CodexCatalogModel {
                    id: "openai-codex/text-coding".to_owned(),
                    slug: "text-coding".to_owned(),
                    display_name: "Text Coding".to_owned(),
                    visibility: Some("list".to_owned()),
                    input_modalities: vec!["text".to_owned()],
                    ..Default::default()
                },
            ],
        });
        assert!(
            catalog["openai-codex/vision-coding"]
                .info
                .supports_image_input
        );
        assert!(
            !catalog["openai-codex/text-coding"]
                .info
                .supports_image_input
        );

        let manager = manager_with_catalog(catalog);
        let selections = [
            (
                ProviderId::OpenAiCodex,
                "vision-coding",
                ImageInputCapability::Supported,
            ),
            (
                ProviderId::OpenAiCodex,
                "text-coding",
                ImageInputCapability::Unsupported,
            ),
            // Switching away from Codex must preserve the other provider's
            // established attachment behavior, even for the same model slug.
            (
                ProviderId::Xai,
                "text-coding",
                ImageInputCapability::Unspecified,
            ),
            // Switching back re-reads the provider-qualified catalog entry.
            (
                ProviderId::OpenAiCodex,
                "vision-coding",
                ImageInputCapability::Supported,
            ),
            // Unknown Codex models fail closed until authenticated discovery
            // installs an authoritative entry.
            (
                ProviderId::OpenAiCodex,
                "not-entitled",
                ImageInputCapability::Unsupported,
            ),
        ];

        for (provider, model, expected) in selections {
            assert_eq!(
                manager.codex_image_input_capability_for_request(provider, model),
                expected,
                "unexpected request-time image capability for {provider}/{model}"
            );
        }
    }

    #[test]
    fn gpt_56_effort_policy_matches_each_dynamic_catalog_menu() {
        let catalog = gpt_56_codex_catalog();
        for (id, expected_default) in [
            ("openai-codex/gpt-5.6-sol", ReasoningEffort::Low),
            ("openai-codex/gpt-5.6-terra", ReasoningEffort::Medium),
            ("openai-codex/gpt-5.6-luna", ReasoningEffort::Medium),
        ] {
            let info = &catalog[id].info;
            assert_eq!(info.context_window.get(), 272_000);
            assert_eq!(info.reasoning_effort, Some(expected_default));
            assert_eq!(info.provider, ProviderId::OpenAiCodex);
            assert!(info.supports_image_input);
            assert_eq!(
                info.extra_headers
                    .get(xai_grok_sampling_types::OPENAI_CODEX_RESPONSES_LITE_HEADER)
                    .map(String::as_str),
                Some("true")
            );
        }
        let manager = manager_with_catalog(catalog);
        let through_ultra = [
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::Xhigh,
            ReasoningEffort::Max,
            ReasoningEffort::Ultra,
        ];
        for model in ["gpt-5.6-sol", "gpt-5.6-terra"] {
            for effort in through_ultra {
                assert!(
                    manager.model_offers_reasoning_effort(model, effort),
                    "{model} should offer {effort} by routing slug"
                );
                assert!(
                    manager
                        .model_offers_reasoning_effort(&format!("openai-codex/{model}"), effort,),
                    "{model} should offer {effort} by catalog id"
                );
            }
        }

        for effort in through_ultra
            .into_iter()
            .filter(|effort| *effort != ReasoningEffort::Ultra)
        {
            assert!(manager.model_offers_reasoning_effort("gpt-5.6-luna", effort));
        }
        assert!(!manager.model_offers_reasoning_effort("gpt-5.6-luna", ReasoningEffort::Ultra,));
        assert_eq!(
            manager.model_default_reasoning_effort("gpt-5.6-luna"),
            Some(ReasoningEffort::Medium),
        );
    }

    #[test]
    fn luna_ultra_config_reconciles_to_advertised_medium_default() {
        let mut prefetched = gpt_56_codex_catalog();
        prefetched
            .get_mut("openai-codex/gpt-5.6-luna")
            .unwrap()
            .info
            .reasoning_effort = Some(ReasoningEffort::Ultra);
        let mut cfg = config::Config::default();
        cfg.models.default = Some("openai-codex/gpt-5.6-luna".to_owned());
        cfg.models.default_reasoning_effort = Some(ReasoningEffort::Ultra);

        let resolved = resolve_combined_model_catalog(&cfg, None, prefetched);
        assert_eq!(
            resolved["openai-codex/gpt-5.6-luna"].info.reasoning_effort,
            Some(ReasoningEffort::Medium),
            "neither a direct scalar override nor persisted default may force Luna to ultra",
        );
    }

    #[test]
    fn codex_v2_ultra_activates_grok_native_proactive_delegation_only_when_offered() {
        let manager = manager_with_catalog(gpt_56_codex_catalog());

        let ultra = manager
            .codex_multi_agent_mode_instruction(
                ProviderId::OpenAiCodex,
                "gpt-5.6-sol",
                Some(ReasoningEffort::Ultra),
            )
            .unwrap();
        assert!(ultra.contains("enabled for this turn"));
        assert!(ultra.contains("existing tool, permission, nesting, and session rules"));

        let max = manager
            .codex_multi_agent_mode_instruction(
                ProviderId::OpenAiCodex,
                "gpt-5.6-sol",
                Some(ReasoningEffort::Max),
            )
            .unwrap();
        assert!(max.contains("disabled for this turn"));

        let luna_stale_ultra = manager
            .codex_multi_agent_mode_instruction(
                ProviderId::OpenAiCodex,
                "gpt-5.6-luna",
                Some(ReasoningEffort::Ultra),
            )
            .unwrap();
        assert!(luna_stale_ultra.contains("disabled for this turn"));
        assert!(
            manager
                .codex_multi_agent_mode_instruction(
                    ProviderId::Xai,
                    "gpt-5.6-sol",
                    Some(ReasoningEffort::Ultra),
                )
                .is_none()
        );
    }

    #[test]
    fn codex_non_v2_catalog_does_not_inject_multi_agent_policy() {
        let mut catalog = gpt_56_codex_catalog();
        catalog
            .get_mut("openai-codex/gpt-5.6-sol")
            .unwrap()
            .info
            .multi_agent_version = Some("v1".to_owned());
        let manager = manager_with_catalog(catalog);
        assert!(
            manager
                .codex_multi_agent_mode_instruction(
                    ProviderId::OpenAiCodex,
                    "gpt-5.6-sol",
                    Some(ReasoningEffort::Ultra),
                )
                .is_none()
        );
    }

    #[test]
    fn provider_qualified_effort_lookup_ignores_same_slug_xai_entry() {
        let mut catalog = gpt_56_codex_catalog();
        let mut xai = catalog["openai-codex/gpt-5.6-sol"].clone();
        xai.info.id = Some("xai-shadow".to_owned());
        xai.info.provider = ProviderId::Xai;
        xai.info
            .reasoning_efforts
            .retain(|option| option.value == ReasoningEffort::Low);
        catalog.insert("xai-shadow".to_owned(), xai);
        let manager = manager_with_catalog(catalog);

        assert!(manager.model_offers_reasoning_effort_for_provider(
            ProviderId::OpenAiCodex,
            "gpt-5.6-sol",
            ReasoningEffort::Ultra,
        ));
        assert!(!manager.model_offers_reasoning_effort_for_provider(
            ProviderId::Xai,
            "gpt-5.6-sol",
            ReasoningEffort::Ultra,
        ));
    }

    #[test]
    fn codex_mapping_preserves_api_flag_lite_marker_and_context_semantics() {
        use crate::remote::{CodexCatalogModel, CodexModelCatalog};

        let mapped = map_openai_codex_catalog(CodexModelCatalog {
            models: vec![CodexCatalogModel {
                id: "openai-codex/provider-only".to_owned(),
                slug: "provider-only".to_owned(),
                display_name: "Provider only".to_owned(),
                visibility: Some("list".to_owned()),
                supported_in_api: false,
                context_window: Some(123_000),
                max_context_window: Some(456_000),
                auto_compact_token_limit: Some(100_000),
                comp_hash: Some("opaque-gpt-5.2-hash".to_owned()),
                supports_reasoning_summary_parameter: true,
                default_reasoning_summary: Some("auto".to_owned()),
                use_responses_lite: true,
                ..Default::default()
            }],
        });
        let info = &mapped["openai-codex/provider-only"].info;
        assert_eq!(info.context_window.get(), 123_000);
        assert_eq!(info.comp_hash.as_deref(), Some("opaque-gpt-5.2-hash"));
        assert_eq!(info.auto_compact_threshold_percent, Some(81));
        assert!(info.supports_reasoning_summary_parameter);
        assert_eq!(info.default_reasoning_summary.as_deref(), Some("auto"));
        assert!(!info.supported_in_api);
        assert!(model_visible_for_auth(info, false));
        assert_eq!(
            info.extra_headers
                .get(xai_grok_sampling_types::OPENAI_CODEX_RESPONSES_LITE_HEADER)
                .map(String::as_str),
            Some("true")
        );

        let fallback = map_openai_codex_catalog(CodexModelCatalog {
            models: vec![CodexCatalogModel {
                id: "openai-codex/no-window".to_owned(),
                slug: "no-window".to_owned(),
                visibility: Some("list".to_owned()),
                ..Default::default()
            }],
        });
        assert_eq!(
            fallback["openai-codex/no-window"].info.context_window.get(),
            crate::remote::DEFAULT_CONTEXT_WINDOW
        );
    }

    #[test]
    fn codex_catalog_identity_debug_is_fixed_shape_and_routing_opaque() {
        let identity = CodexCatalogIdentity::for_test(
            "private-credential",
            "private-account",
            Some("private-user"),
            false,
        );
        let alternate_routing = CodexCatalogIdentity::for_test(
            "private-credential",
            "private-account",
            Some("private-user"),
            true,
        );
        let rendered = format!("{identity:?}");
        let alternate = format!("{alternate_routing:?}");

        assert_eq!(rendered, "CodexCatalogIdentity { .. }");
        assert_eq!(rendered, alternate);
        for private in [
            "fedramp",
            "private-credential",
            "private-account",
            "private-user",
        ] {
            assert!(!rendered.to_ascii_lowercase().contains(private));
        }
    }

    #[test]
    fn codex_catalog_and_etag_survive_xai_identity_clear() {
        use crate::remote::{CodexCatalogModel, CodexModelCatalog};

        let manager = test_manager();
        *manager.inner.etag.write() = Some("xai-etag".to_owned());
        manager.apply_openai_codex_catalog(
            CodexModelCatalog {
                models: vec![CodexCatalogModel {
                    id: "openai-codex/account-model".to_owned(),
                    slug: "account-model".to_owned(),
                    display_name: "Account model".to_owned(),
                    visibility: Some("list".to_owned()),
                    supported_in_api: true,
                    context_window: Some(372_000),
                    ..Default::default()
                }],
            },
            Some("codex-etag".to_owned()),
            CodexCatalogIdentity::for_test("credential-a", "account-a", Some("user-a"), false),
            true,
        );

        assert_eq!(manager.inner.etag.read().as_deref(), Some("xai-etag"));
        assert_eq!(
            manager.inner.codex_catalog.etag().as_deref(),
            Some("codex-etag")
        );
        assert!(manager.models().contains_key("openai-codex/account-model"));

        manager.clear();
        assert_eq!(manager.inner.etag.read().as_deref(), None);
        assert_eq!(
            manager.inner.codex_catalog.etag().as_deref(),
            Some("codex-etag")
        );
        assert!(manager.models().contains_key("openai-codex/account-model"));
        assert!(manager.has_fetched_real_catalog());
    }

    #[test]
    fn codex_catalog_is_cleared_on_account_switch_or_logout_but_xai_survives() {
        use crate::remote::{CodexCatalogModel, CodexModelCatalog};

        let manager = test_manager();
        let mut xai_models = IndexMap::new();
        xai_models.insert(
            "xai-model".to_owned(),
            ModelEntry::fallback("xai-model", &config::EndpointsConfig::default()),
        );
        *manager.inner.prefetched.write() = Some(xai_models.clone());
        manager.rebuild(&config::Config::default(), Some(xai_models));

        let account_a =
            CodexCatalogIdentity::for_test("credential-a", "account-a", Some("user-a"), false);
        let account_b =
            CodexCatalogIdentity::for_test("credential-b", "account-b", Some("user-b"), false);
        let catalog = || CodexModelCatalog {
            models: vec![CodexCatalogModel {
                id: "openai-codex/account-model".to_owned(),
                slug: "account-model".to_owned(),
                visibility: Some("list".to_owned()),
                ..Default::default()
            }],
        };

        manager.apply_openai_codex_catalog(
            catalog(),
            Some("codex-etag".to_owned()),
            account_a.clone(),
            true,
        );
        assert!(manager.models().contains_key("openai-codex/account-model"));
        assert!(
            !manager.reconcile_openai_codex_identity(Some(&account_a)),
            "a token revision for the same credential/account must retain the catalog"
        );

        assert!(manager.reconcile_openai_codex_identity(Some(&account_b)));
        assert!(!manager.models().contains_key("openai-codex/account-model"));
        assert!(manager.models().contains_key("xai-model"));
        assert!(manager.inner.codex_catalog.etag().is_none());
        assert!(manager.inner.codex_catalog.identity_is_none());

        manager.apply_openai_codex_catalog(catalog(), None, account_b, true);
        assert!(manager.reconcile_openai_codex_identity(None));
        assert!(!manager.models().contains_key("openai-codex/account-model"));
        assert!(manager.models().contains_key("xai-model"));
    }

    #[test]
    fn codex_visibility_hide_is_explicitly_addressable_but_none_is_not() {
        use crate::remote::{CodexCatalogModel, CodexModelCatalog};

        let mapped = map_openai_codex_catalog(CodexModelCatalog {
            models: vec![
                CodexCatalogModel {
                    id: "openai-codex/visible".to_owned(),
                    slug: "visible".to_owned(),
                    visibility: Some("list".to_owned()),
                    ..Default::default()
                },
                CodexCatalogModel {
                    id: "openai-codex/hidden".to_owned(),
                    slug: "hidden".to_owned(),
                    visibility: Some("hide".to_owned()),
                    ..Default::default()
                },
                CodexCatalogModel {
                    id: "openai-codex/none".to_owned(),
                    slug: "none".to_owned(),
                    visibility: Some("none".to_owned()),
                    ..Default::default()
                },
            ],
        });
        assert!(mapped["openai-codex/hidden"].info.hidden);
        assert!(mapped["openai-codex/hidden"].info.user_selectable);
        assert!(mapped["openai-codex/none"].info.hidden);
        assert!(!mapped["openai-codex/none"].info.user_selectable);

        let mut cfg = config::Config::default();
        cfg.models.default = Some("openai-codex/hidden".to_owned());
        let resolved = resolve_combined_model_catalog(&cfg, None, mapped);
        let (key, _, source) = resolve_default_model(&cfg, &resolved, false);
        assert_eq!(key, "openai-codex/hidden");
        assert_eq!(source, config::ConfigSource::Config);
        let picker = available_models(&resolved, false);
        assert!(picker.contains_key(&acp::ModelId::new("openai-codex/visible")));
        assert!(!picker.contains_key(&acp::ModelId::new("openai-codex/hidden")));
        assert!(!picker.contains_key(&acp::ModelId::new("openai-codex/none")));
    }

    #[test]
    fn static_custom_entry_cannot_impersonate_codex_entitlement() {
        let mut static_models = IndexMap::new();
        let mut entry = ModelEntry::fallback("gpt-static", &config::EndpointsConfig::default());
        entry.info.provider = ProviderId::OpenAiCodex;
        static_models.insert("openai-codex/gpt-static".to_owned(), entry);

        let resolved = resolve_model_catalog(&config::Config::default(), Some(static_models));
        assert!(!resolved.contains_key("openai-codex/gpt-static"));
    }

    #[test]
    fn model_show_model_fingerprint_reads_catalog_flag() {
        let mgr = test_manager();

        // Entry with the catalog flag set → accessor returns true.
        let mut flagged = ModelEntry {
            info: config::ModelInfo::fallback("fp-model"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        flagged.info.show_model_fingerprint = true;
        mgr.insert_test_entry("fp-model", flagged);

        // Entry without the flag → defaults false.
        mgr.insert_test_entry(
            "plain-model",
            ModelEntry {
                info: config::ModelInfo::fallback("plain-model"),
                api_key: None,
                env_key: None,
                api_base_url: None,
            },
        );

        // Catalog KEY differs from the routing SLUG (custom/enterprise id): the
        // map is keyed "enterprise-key" but the model slug is "enterprise-slug".
        let mut custom = ModelEntry {
            info: config::ModelInfo::fallback("enterprise-slug"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        custom.info.show_model_fingerprint = true;
        mgr.insert_test_entry("enterprise-key", custom);

        assert!(mgr.model_show_model_fingerprint("fp-model"));
        assert!(!mgr.model_show_model_fingerprint("plain-model"));
        // Unknown model id → false (no catalog entry).
        assert!(!mgr.model_show_model_fingerprint("missing-model"));
        // Lookup by the routing SLUG must resolve to the differing catalog KEY —
        // a direct `.get(slug)` would miss this entry and wrongly return false.
        assert!(
            mgr.model_show_model_fingerprint("enterprise-slug"),
            "slug lookup must resolve to the catalog key and read the flag",
        );
        // Lookup by the catalog KEY itself still works (exact-match path).
        assert!(mgr.model_show_model_fingerprint("enterprise-key"));
    }

    /// The active model must be selectable, not the first entry of the
    /// un-allowlisted catalog.
    #[test]
    fn default_model_honors_allowlist_when_no_default_set() {
        let cfg = config_from_toml(
            r#"
            [models]
            allowed_models = ["keep-*"]
            [model.zzz-first]
            model = "zzz-first"
            base_url = "https://api.x.ai/v1"
            context_window = 256000
            [model.keep-one]
            model = "keep-one"
            base_url = "https://api.x.ai/v1"
            context_window = 256000
            "#,
        );
        let catalog = resolve_model_catalog(&cfg, None);
        let (_key, entry, _src) = resolve_default_model(&cfg, &catalog, true);
        assert!(
            entry.info.user_selectable,
            "picked non-selectable {}",
            entry.model
        );
    }

    #[test]
    fn validate_selectable_rejects_bad_allowlists() {
        // Excluded explicit default → error names the default.
        let excluded = config_from_toml(
            r#"
            [models]
            default = "grok-3"
            allowed_models = ["grok-4*"]
            [model.grok-3]
            model = "grok-3"
            base_url = "https://api.x.ai/v1"
            context_window = 256000
            [model.grok-4]
            model = "grok-4"
            base_url = "https://api.x.ai/v1"
            context_window = 256000
            "#,
        );
        let catalog = resolve_model_catalog(&excluded, None);
        assert!(
            validate_selectable(&excluded, &catalog)
                .unwrap_err()
                .contains("grok-3")
        );

        // Matches nothing → error.
        let zero = config_from_toml(
            r#"
            [models]
            allowed_models = ["nomatch-*"]
            [model.grok-4]
            model = "grok-4"
            base_url = "https://api.x.ai/v1"
            context_window = 256000
            "#,
        );
        let catalog = resolve_model_catalog(&zero, None);
        assert!(validate_selectable(&zero, &catalog).is_err());
    }

    #[tokio::test]
    async fn refresh_if_new_etag_skips_when_same() {
        let mgr = test_manager();
        // Set initial etag
        *mgr.inner.etag.write() = Some("\"abc123\"".to_string());

        // Same etag — should be a no-op (etag stays the same)
        mgr.refresh_if_new_etag("\"abc123\"".to_string()).await;
        assert_eq!(
            mgr.inner.etag.read().as_deref(),
            Some("\"abc123\""),
            "etag should remain unchanged when same"
        );
    }

    #[tokio::test]
    async fn set_current_model_id_change_fires_watch_to_all_subscribers() {
        // Two subscribers (simulating two SessionActors sharing one
        // ModelsManager catalog) both observe the change. Fast-path
        // "same id" must NOT bump the generation.
        let mgr = test_manager();
        let mut rx_a = mgr.subscribe_model_switch();
        let mut rx_b = mgr.subscribe_model_switch();
        let initial_a = *rx_a.borrow_and_update();
        let initial_b = *rx_b.borrow_and_update();
        assert_eq!(initial_a, initial_b);

        // Same id is the fast path — no bump.
        mgr.set_current_model_id(acp::ModelId::new("default"));
        // Force-yield so any spurious wakeup would have a chance to
        // surface. `try_recv` on a watch channel: use a timeout-zero
        // race; if `.changed()` resolves within 25ms we have a bug.
        let same_id_ticked =
            tokio::time::timeout(std::time::Duration::from_millis(25), rx_a.changed())
                .await
                .is_ok();
        assert!(
            !same_id_ticked,
            "set_current_model_id(same id) must NOT bump the watch generation",
        );

        // Real switch: both subscribers see the change.
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));
        tokio::time::timeout(std::time::Duration::from_millis(100), rx_a.changed())
            .await
            .expect("rx_a saw the switch")
            .expect("watch channel still open");
        tokio::time::timeout(std::time::Duration::from_millis(100), rx_b.changed())
            .await
            .expect("rx_b saw the switch")
            .expect("watch channel still open");
        assert_ne!(*rx_a.borrow(), initial_a);
        assert_eq!(*rx_a.borrow(), *rx_b.borrow());
        assert!(mgr.model_switch_generation() > initial_a);
    }

    #[tokio::test]
    async fn model_switch_generation_snapshot_reflects_current_state() {
        let mgr = test_manager();
        let start = mgr.model_switch_generation();
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));
        assert_eq!(mgr.model_switch_generation(), start + 1);
        // Idempotent: same id → no bump.
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));
        assert_eq!(mgr.model_switch_generation(), start + 1);
        // Another real change: another bump.
        mgr.set_current_model_id(acp::ModelId::new("grok-3"));
        assert_eq!(mgr.model_switch_generation(), start + 2);
    }

    #[test]
    fn rebuild_updates_models_and_available() {
        let mgr = test_manager();
        assert!(mgr.models().is_empty());
        assert!(mgr.available().is_empty());

        let cfg = config::Config::default();
        let mut prefetched = IndexMap::new();
        prefetched.insert(
            "test-model".to_string(),
            ModelEntry {
                info: config::ModelInfo::fallback("test-model"),
                api_key: None,
                env_key: None,
                api_base_url: None,
            },
        );

        mgr.rebuild(&cfg, Some(prefetched));

        assert!(
            !mgr.models().is_empty(),
            "models should be populated after rebuild"
        );
    }

    #[test]
    fn current_reasoning_effort_round_trip() {
        let mgr = test_manager();
        assert_eq!(mgr.current_reasoning_effort(), None);

        mgr.set_current_reasoning_effort(Some(ReasoningEffort::High));
        assert_eq!(mgr.current_reasoning_effort(), Some(ReasoningEffort::High));

        mgr.set_current_reasoning_effort(None);
        assert_eq!(mgr.current_reasoning_effort(), None);
    }

    #[test]
    fn current_reasoning_effort_seeded_from_config() {
        let tmp = std::env::temp_dir().join("grok-test-models-manager-seed");
        let auth_manager = Arc::new(AuthManager::new(&tmp, GrokComConfig::default()));
        let mut cfg = config::Config::default();
        cfg.models.default_reasoning_effort = Some(ReasoningEffort::Xhigh);
        let mgr = ModelsManager::new(
            None,
            IndexMap::new(),
            acp::ModelId::new("default"),
            auth_manager,
            cfg,
        );
        assert_eq!(mgr.current_reasoning_effort(), Some(ReasoningEffort::Xhigh),);
    }

    #[test]
    fn default_reasoning_effort_only_stamps_supporting_model() {
        use indexmap::IndexMap;

        // Model that supports reasoning effort — effort should be applied.
        let mut cfg = config::Config::default();
        cfg.models.default = Some("reasoning-model".to_string());
        cfg.models.default_reasoning_effort = Some(ReasoningEffort::High);

        let mut prefetched = IndexMap::new();
        let mut reasoning_entry = ModelEntry {
            info: config::ModelInfo::fallback("reasoning-model"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        reasoning_entry.info.supports_reasoning_effort = true;
        prefetched.insert("reasoning-model".to_string(), reasoning_entry);

        let catalog = resolve_model_catalog(&cfg, Some(prefetched));
        assert_eq!(
            catalog["reasoning-model"].info.reasoning_effort,
            Some(ReasoningEffort::High),
            "reasoning-supporting default model should be stamped",
        );

        // Model that does NOT support reasoning effort — effort must NOT be applied.
        let mut cfg = config::Config::default();
        cfg.models.default = Some("plain-model".to_string());
        cfg.models.default_reasoning_effort = Some(ReasoningEffort::High);

        let mut prefetched = IndexMap::new();
        let plain_entry = ModelEntry {
            info: config::ModelInfo::fallback("plain-model"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        prefetched.insert("plain-model".to_string(), plain_entry);

        let catalog = resolve_model_catalog(&cfg, Some(prefetched));
        assert_eq!(
            catalog["plain-model"].info.reasoning_effort, None,
            "non-reasoning default model must NOT be stamped with persisted effort",
        );
    }

    #[test]
    fn legacy_max_alias_resolves_to_xhigh_without_collapsing_codex_max() {
        let mut cfg = config::Config::default();
        cfg.models.default = Some("reasoning-model".to_owned());
        cfg.models.default_reasoning_effort = Some(ReasoningEffort::Max);

        let mut prefetched = IndexMap::new();
        let mut legacy = ModelEntry {
            info: config::ModelInfo::fallback("reasoning-model"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        legacy.info.provider = ProviderId::Xai;
        legacy.info.supports_reasoning_effort = true;
        prefetched.insert("reasoning-model".to_owned(), legacy);

        let catalog = resolve_model_catalog(&cfg, Some(prefetched));
        assert_eq!(
            catalog["reasoning-model"].info.reasoning_effort,
            Some(ReasoningEffort::Xhigh),
            "the pre-Codex max spelling must retain its xhigh meaning",
        );

        let auth_dir = std::env::temp_dir().join("grok-test-models-manager-legacy-max");
        let manager = ModelsManager::new(
            None,
            catalog,
            acp::ModelId::new("reasoning-model"),
            Arc::new(AuthManager::new(&auth_dir, GrokComConfig::default())),
            cfg,
        );
        assert_eq!(
            manager.resolve_reasoning_effort_for_model("reasoning-model", ReasoningEffort::Max,),
            Some(ReasoningEffort::Xhigh),
        );

        let codex = manager_with_catalog(gpt_56_codex_catalog());
        assert_eq!(
            codex.resolve_reasoning_effort_for_model(
                "openai-codex/gpt-5.6-sol",
                ReasoningEffort::Max,
            ),
            Some(ReasoningEffort::Max),
            "an explicitly advertised Codex Max tier must remain distinct",
        );
    }

    #[test]
    fn reasoning_effort_override_skips_models_that_do_not_offer_level() {
        use indexmap::IndexMap;
        use xai_grok_sampling_types::ReasoningEffortOption;

        let cfg = config::Config {
            reasoning_effort_override: Some(ReasoningEffort::None),
            ..Default::default()
        };

        let mut prefetched = IndexMap::new();
        // 4.5-style: supports effort, menu is high only (no none).
        let mut no_none = ModelEntry {
            info: config::ModelInfo::fallback("grok-4.5"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        no_none.info.supports_reasoning_effort = true;
        no_none.info.reasoning_efforts = vec![ReasoningEffortOption {
            id: "high".into(),
            value: ReasoningEffort::High,
            label: "High".into(),
            description: None,
            default: true,
        }];
        no_none.info.reasoning_effort = Some(ReasoningEffort::High);
        prefetched.insert("grok-4.5".to_string(), no_none);

        // Model that explicitly offers none.
        let mut with_none = ModelEntry {
            info: config::ModelInfo::fallback("legacy-none"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        with_none.info.supports_reasoning_effort = true;
        with_none.info.reasoning_efforts = vec![ReasoningEffortOption {
            id: "none".into(),
            value: ReasoningEffort::None,
            label: "None".into(),
            description: None,
            default: true,
        }];
        prefetched.insert("legacy-none".to_string(), with_none);

        let catalog = resolve_model_catalog(&cfg, Some(prefetched));
        assert_eq!(
            catalog["grok-4.5"].info.reasoning_effort,
            Some(ReasoningEffort::High),
            "--effort none must not stamp onto models that do not offer none"
        );
        assert_eq!(
            catalog["legacy-none"].info.reasoning_effort,
            Some(ReasoningEffort::None),
            "models that list none should still accept the override"
        );
    }

    #[test]
    fn config_menu_only_model_derives_support_and_default() {
        // The config-TOML path: a model configured with ONLY `reasoning_efforts`
        // (no `supports_reasoning_effort`, no scalar `reasoning_effort`) must read
        // as supported with the marked-default option's value on the internal
        // gates that BugBot flagged (support gate + wire default).
        let mut cfg = config::Config::default();
        cfg.config_models.insert(
            "menu-only".to_string(),
            config::ConfigModelOverride {
                reasoning_efforts: vec![
                    ReasoningEffortOption {
                        id: "balanced".to_string(),
                        value: ReasoningEffort::Medium,
                        label: "Balanced".to_string(),
                        description: None,
                        default: false,
                    },
                    ReasoningEffortOption {
                        id: "deep".to_string(),
                        value: ReasoningEffort::Xhigh,
                        label: "Deep".to_string(),
                        description: None,
                        default: true,
                    },
                ],
                ..Default::default()
            },
        );
        // A sibling with no menu must stay underived (empty-list path unchanged).
        cfg.config_models
            .insert("plain".to_string(), config::ConfigModelOverride::default());

        let catalog = resolve_model_catalog(&cfg, None);
        let info = &catalog["menu-only"].info;
        assert!(
            info.supports_reasoning_effort,
            "menu-only model must derive support"
        );
        assert_eq!(
            info.reasoning_effort,
            Some(ReasoningEffort::Xhigh),
            "derived default = marked-default option value"
        );
        assert!(!catalog["plain"].info.supports_reasoning_effort);
        assert_eq!(catalog["plain"].info.reasoning_effort, None);

        // The internal getters read those derived fields.
        let tmp = std::env::temp_dir().join("grok-test-models-manager-menu-only");
        let auth_manager = Arc::new(AuthManager::new(&tmp, GrokComConfig::default()));
        let mgr = ModelsManager::new(
            None,
            catalog,
            acp::ModelId::new("menu-only"),
            auth_manager,
            cfg,
        );
        assert!(mgr.model_supports_reasoning_effort("menu-only"));
        assert_eq!(
            mgr.model_default_reasoning_effort("menu-only"),
            Some(ReasoningEffort::Xhigh)
        );
        assert_eq!(mgr.model_reasoning_efforts("menu-only").len(), 2);
        assert!(!mgr.model_supports_reasoning_effort("plain"));
        assert_eq!(mgr.model_default_reasoning_effort("plain"), None);
    }

    #[test]
    fn cli_reasoning_effort_override_only_stamps_supporting_models() {
        use indexmap::IndexMap;

        let cfg = config::Config {
            reasoning_effort_override: Some(ReasoningEffort::High),
            ..config::Config::default()
        };

        let mut prefetched = IndexMap::new();
        let mut reasoning_entry = ModelEntry {
            info: config::ModelInfo::fallback("reasoning-model"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        reasoning_entry.info.supports_reasoning_effort = true;
        prefetched.insert("reasoning-model".to_string(), reasoning_entry);

        let plain_entry = ModelEntry {
            info: config::ModelInfo::fallback("plain-model"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        prefetched.insert("plain-model".to_string(), plain_entry);

        let catalog = resolve_model_catalog(&cfg, Some(prefetched));
        assert_eq!(
            catalog["reasoning-model"].info.reasoning_effort,
            Some(ReasoningEffort::High),
            "reasoning-supporting model should be stamped",
        );
        assert_eq!(
            catalog["plain-model"].info.reasoning_effort, None,
            "non-reasoning model must NOT be stamped",
        );
    }

    #[test]
    fn apply_refresh_result_only_updates_etag_on_success() {
        let mgr = test_manager();
        let cfg = config::Config::default();
        *mgr.inner.etag.write() = Some("\"old\"".to_string());

        assert!(
            !mgr.apply_refresh_result(&cfg, None, Some("\"new\"".to_string())),
            "failed refresh should report no update"
        );
        assert_eq!(
            mgr.inner.etag.read().as_deref(),
            Some("\"old\""),
            "etag should remain unchanged when refresh fails"
        );
        assert!(
            mgr.prefetched().is_none(),
            "prefetched models should stay unchanged"
        );
    }

    fn make_model_entry(model_id: &str) -> ModelEntry {
        ModelEntry {
            info: config::ModelInfo::fallback(model_id),
            api_key: None,
            env_key: None,
            api_base_url: None,
        }
    }

    fn make_prefetched(ids: &[&str]) -> IndexMap<String, ModelEntry> {
        ids.iter()
            .map(|id| (id.to_string(), make_model_entry(id)))
            .collect()
    }

    // ── auth-change refresh: has_fetched_real_catalog flag ─────────────

    #[test]
    fn first_apply_refresh_reselects_default_model() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        assert!(!mgr.has_fetched_real_catalog());

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);

        assert!(mgr.has_fetched_real_catalog());
        assert_eq!(mgr.current_model_id().0.as_ref(), "grok-3");
    }

    #[test]
    fn subsequent_apply_refresh_preserves_user_model() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        // Simulate on_auth_changed clearing prefetched + etag.
        *mgr.inner.prefetched.write() = None;
        *mgr.inner.etag.write() = None;

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);

        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "grok-4",
            "user's model selection must survive auth-change refresh"
        );
    }

    #[test]
    fn subsequent_refresh_reselects_when_model_removed() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        // Second refresh with grok-4 removed.
        let prefetched = make_prefetched(&["grok-3", "grok-4.5"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);

        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "grok-3",
            "should fall back to config default when current is removed"
        );
    }

    #[test]
    fn failed_refresh_does_not_set_has_fetched_real_catalog() {
        let mgr = test_manager();
        let cfg = config::Config::default();

        mgr.apply_refresh_result(&cfg, None, None);

        assert!(
            !mgr.has_fetched_real_catalog(),
            "failed refresh must not flip has_fetched_real_catalog"
        );
    }

    // ── apply_config: honor changed preferred model from config ────────

    #[test]
    fn apply_config_honors_new_preferred_model() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        // Simulate stale inner cfg (no default) from a racing auth refresh.
        let mut stale_cfg = config::Config::default();
        stale_cfg.models.default = None;
        *mgr.inner.cfg.write() = stale_cfg;

        let mut new_cfg = config::Config::default();
        new_cfg.models.default = Some("grok-3".to_string());
        mgr.apply_config(new_cfg);

        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "grok-3",
            "apply_config must honor updated preferred model from config"
        );
    }

    #[test]
    fn apply_config_preserves_current_when_preferred_unchanged() {
        let mgr = test_manager();
        let cfg = config::Config::default();

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);

        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        // Unrelated config change — preferred model unchanged.
        let new_cfg = config::Config::default();
        mgr.apply_config(new_cfg);

        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "grok-4",
            "apply_config must not reset model when preferred hasn't changed"
        );
    }

    #[test]
    fn apply_config_falls_back_when_preferred_not_in_catalog() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);

        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        // Preferred model not in catalog — falls back to first entry.
        let mut new_cfg = config::Config::default();
        new_cfg.models.default = Some("grok-nonexistent".to_string());
        mgr.apply_config(new_cfg);

        let current = mgr.current_model_id();
        let first_available = mgr.available().keys().next().unwrap().clone();
        assert_eq!(
            current.0.as_ref(),
            first_available.0.as_ref(),
            "should fall back to first visible model when preferred not in catalog"
        );
    }

    #[test]
    fn apply_config_both_none_preferred_preserves_current() {
        let mgr = test_manager();
        let cfg = config::Config::default();
        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));
        let new_cfg = config::Config::default();
        mgr.apply_config(new_cfg);

        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "grok-4",
            "both-None preferred must preserve user's runtime model"
        );
    }

    #[test]
    fn apply_config_old_some_new_none_preserves_current() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        assert_eq!(mgr.current_model_id().0.as_ref(), "grok-3");

        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        // [models] default removed — is_some() guard prevents reset.
        let new_cfg = config::Config::default();
        mgr.apply_config(new_cfg);

        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "grok-4",
            "old=Some new=None must not reset model (is_some guard)"
        );
    }

    // ── end-to-end: auth refresh + config reload compose correctly ───

    #[test]
    fn auth_refresh_then_config_reload_preserves_user_model() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        // Initial fetch.
        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);

        // User runs /model grok-4.
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        // Auth refresh races — clears prefetched/etag.
        *mgr.inner.prefetched.write() = None;
        *mgr.inner.etag.write() = None;

        // Second fetch must preserve user's model.
        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        assert_eq!(mgr.current_model_id().0.as_ref(), "grok-4");

        // Config reload with persisted preference.
        let mut new_cfg = config::Config::default();
        new_cfg.models.default = Some("grok-4".to_string());
        mgr.apply_config(new_cfg);
        assert_eq!(mgr.current_model_id().0.as_ref(), "grok-4");
    }

    // ── disk-cache hot-reload (external models_cache.json writes) ────

    fn test_cache_manager(dir: &std::path::Path) -> ModelsCacheManager {
        ModelsCacheManager {
            path: dir.join(MODELS_CACHE_FILE),
            ttl: CACHE_TTL,
        }
    }

    fn test_cache_scope(manager: &ModelsManager) -> Option<String> {
        manager.xai_catalog_snapshot().credential_scope
    }

    /// An external process persisting a fresh catalog must be picked up:
    /// catalog swapped, etag adopted, real-catalog flag set.
    #[test]
    fn reload_from_disk_cache_applies_external_catalog() {
        let mgr = test_manager();
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());

        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        cache.persist(
            &make_prefetched(&["grok-4.5", "grok-4.3"]),
            Some("etag-ext"),
            auth_method,
            &mgr.cache_origin(),
            test_cache_scope(&mgr).as_deref(),
        );

        mgr.reload_from_cache_manager(&cache);

        assert!(mgr.has_fetched_real_catalog());
        assert!(mgr.models().contains_key("grok-4.5"));
        assert!(mgr.models().contains_key("grok-4.3"));
        assert_eq!(mgr.inner.etag.read().as_deref(), Some("etag-ext"));
    }

    /// A latched "allowlist excludes everything" prompt block must clear when
    /// an external cache write delivers a catalog the allowlist matches —
    /// `reload_from_cache_manager` recomputes `allowlist_excludes_all` after
    /// the rebuild, like `apply_refresh_result` does.
    #[test]
    fn reload_from_disk_cache_recomputes_allowlist_excludes_all() {
        let mgr = test_manager();
        let cfg = config_from_toml("[models]\nallowed_models = [\"keep-*\"]");

        // Latch the flag: neither the fetched model nor the bundled defaults
        // merged by `resolve_model_catalog` match `keep-*`.
        mgr.apply_refresh_result(&cfg, Some(make_prefetched(&["other-1"])), None);
        assert!(
            mgr.allowlist_excludes_all(),
            "setup: allowlist should exclude the entire catalog"
        );
        // `apply_refresh_result` borrows the config without storing it, while
        // `reload_from_cache_manager` reads `inner.cfg` — install it there.
        *mgr.inner.cfg.write() = cfg.clone();

        // External process persists a catalog containing an allowed model.
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        cache.persist(
            &make_prefetched(&["keep-1"]),
            Some("etag-keep"),
            auth_method,
            &mgr.cache_origin(),
            test_cache_scope(&mgr).as_deref(),
        );

        mgr.reload_from_cache_manager(&cache);

        assert!(mgr.models().contains_key("keep-1"));
        assert!(
            !mgr.allowlist_excludes_all(),
            "corrective external cache write must unlatch the prompt block"
        );
    }

    /// When the *first* real catalog arrives via an external cache write (the
    /// leader never completed its own fetch), the configured `[models]`
    /// default must be resolved — mirroring `apply_refresh_result`'s
    /// first-catalog branch — instead of staying on the bundled placeholder.
    #[test]
    fn reload_from_disk_cache_resolves_default_on_first_catalog() {
        let mgr = test_manager();
        assert!(!mgr.has_fetched_real_catalog());
        let cfg = config_from_toml("[models]\ndefault = \"keep-1\"");
        // `reload_from_cache_manager` reads the manager's stored config.
        *mgr.inner.cfg.write() = cfg.clone();

        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        cache.persist(
            &make_prefetched(&["keep-1", "other-1"]),
            Some("etag-first"),
            auth_method,
            &mgr.cache_origin(),
            test_cache_scope(&mgr).as_deref(),
        );

        mgr.reload_from_cache_manager(&cache);

        assert!(mgr.has_fetched_real_catalog());
        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "keep-1",
            "first real catalog must resolve the configured default"
        );
    }

    /// A cache write whose catalog matches the in-memory prefetched map (the
    /// leader's own `persist`/`renew_ttl` self-writes, or a same-content fetch
    /// by another process) must be a no-op apart from adopting the etag — no
    /// rebuild, no model reselection.
    #[test]
    fn reload_from_disk_cache_skips_identical_catalog_and_adopts_etag() {
        let mgr = test_manager();
        let cfg = config::Config::default();
        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched.clone()), Some("etag-a".into()));
        mgr.set_current_model_id(acp::ModelId::new("grok-4"));

        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        cache.persist(
            &prefetched,
            Some("etag-b"),
            auth_method,
            &mgr.cache_origin(),
            test_cache_scope(&mgr).as_deref(),
        );

        mgr.reload_from_cache_manager(&cache);

        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "grok-4",
            "identical catalog must not disturb the user's model"
        );
        assert_eq!(
            mgr.inner.etag.read().as_deref(),
            Some("etag-b"),
            "etag should be adopted so refresh_if_new_etag stays accurate"
        );
    }

    /// A cache file older than the TTL is rejected by `load_fresh` — the
    /// watcher event arrives within the debounce window of the write, so a
    /// stale file means the write was not a fresh fetch.
    #[test]
    fn reload_from_disk_cache_ignores_stale_cache() {
        let mgr = test_manager();
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        let stale = ModelsCache {
            fetched_at: Utc::now() - ChronoDuration::seconds(3600),
            grok_version: Some(xai_grok_version::VERSION.to_string()),
            auth_method: Some(auth_method),
            credential_scope: test_cache_scope(&mgr),
            origin: Some(mgr.cache_origin()),
            etag: Some("etag-stale".into()),
            models: make_prefetched(&["grok-stale"]),
        };
        cache.atomic_write(&stale);

        mgr.reload_from_cache_manager(&cache);

        assert!(!mgr.models().contains_key("grok-stale"));
        assert!(mgr.inner.etag.read().is_none());
    }

    /// A cache persisted by a process running with different credentials
    /// (e.g. an API-key `--no-leader` run next to a session-auth leader)
    /// must not poison this manager's catalog.
    #[test]
    fn reload_from_disk_cache_ignores_auth_method_mismatch() {
        let mgr = test_manager();
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let current = mgr.inner.fetch_auth.read().cache_auth_method();
        let other = if current == CacheAuthMethod::Session {
            CacheAuthMethod::ApiKey
        } else {
            CacheAuthMethod::Session
        };
        cache.persist(
            &make_prefetched(&["grok-other-auth"]),
            Some("etag-x"),
            other,
            &mgr.cache_origin(),
            test_cache_scope(&mgr).as_deref(),
        );

        mgr.reload_from_cache_manager(&cache);

        assert!(!mgr.models().contains_key("grok-other-auth"));
    }

    #[test]
    fn reload_from_disk_cache_ignores_credential_scope_mismatch() {
        let mgr = test_manager();
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        cache.persist(
            &make_prefetched(&["grok-other-account"]),
            Some("etag-other-account"),
            auth_method,
            &mgr.cache_origin(),
            Some("one-way-scope-for-another-account"),
        );

        mgr.reload_from_cache_manager(&cache);

        assert!(!mgr.models().contains_key("grok-other-account"));
        assert!(mgr.inner.etag.read().is_none());
    }

    /// A cache persisted by a process pointed at a *different backend* (env
    /// override, another deployment, a test's mock server) must not poison
    /// this manager's catalog: cached entries embed absolute `base_url`s from
    /// their origin, so adopting them silently re-points inference. This is
    /// the windows-x86_64 lifecycle e2e failure mode — the shared-profile
    /// cache from test 1's mock sent test 2's prompts to a dead port.
    #[test]
    fn reload_from_disk_cache_ignores_origin_mismatch() {
        let mgr = test_manager();
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        cache.persist(
            &make_prefetched(&["grok-other-origin"]),
            Some("etag-y"),
            auth_method,
            "http://127.0.0.1:49953/v1/models",
            test_cache_scope(&mgr).as_deref(),
        );

        mgr.reload_from_cache_manager(&cache);

        assert!(!mgr.models().contains_key("grok-other-origin"));
        assert!(mgr.inner.etag.read().is_none());
    }

    /// A legacy cache file written before the `origin` field existed must be
    /// treated as a miss (`None` origin never matches) — its entries could
    /// have come from anywhere.
    #[test]
    fn reload_from_disk_cache_ignores_legacy_cache_without_origin() {
        let mgr = test_manager();
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = test_cache_manager(tmp.path());
        let auth_method = mgr.inner.fetch_auth.read().cache_auth_method();
        let legacy = ModelsCache {
            fetched_at: Utc::now(),
            grok_version: Some(xai_grok_version::VERSION.to_string()),
            auth_method: Some(auth_method),
            credential_scope: None,
            origin: None,
            etag: Some("etag-legacy".into()),
            models: make_prefetched(&["grok-legacy"]),
        };
        cache.atomic_write(&legacy);

        mgr.reload_from_cache_manager(&cache);

        assert!(!mgr.models().contains_key("grok-legacy"));
    }

    // ── clear() resets has_fetched_real_catalog ──────────────────────

    #[test]
    fn clear_resets_has_fetched_real_catalog() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-3".to_string());

        let prefetched = make_prefetched(&["grok-3", "grok-4"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        assert!(mgr.has_fetched_real_catalog());

        mgr.clear();
        assert!(!mgr.has_fetched_real_catalog());

        // New identity fetch — resolves default via reselect_default_model.
        let prefetched = make_prefetched(&["grok-4.5", "grok-4.3"]);
        mgr.apply_refresh_result(&cfg, Some(prefetched), None);
        let first_available = mgr.available().keys().next().unwrap().clone();
        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            first_available.0.as_ref()
        );
    }

    /// A flip is "campaign-only" iff the preferred changed and either side is an
    /// active campaign default.
    #[test]
    fn is_campaign_only_flip_detects_campaign_driven_changes() {
        let camp: std::collections::HashSet<String> = ["beta".into()].into_iter().collect();
        // New side is the campaign default (campaign appearing) → campaign-only.
        assert!(is_campaign_only_flip(
            &Some("alpha".into()),
            &Some("beta".into()),
            &camp
        ));
        // Old side was the campaign default (campaign withdrawing) → campaign-only.
        assert!(is_campaign_only_flip(
            &Some("beta".into()),
            &Some("alpha".into()),
            &camp
        ));
        // Neither side a campaign default → ordinary user/CLI/env flip.
        assert!(!is_campaign_only_flip(
            &Some("alpha".into()),
            &Some("gamma".into()),
            &camp
        ));
        // No change, cleared default, or empty campaign set → never campaign-only.
        assert!(!is_campaign_only_flip(
            &Some("beta".into()),
            &Some("beta".into()),
            &camp
        ));
        assert!(!is_campaign_only_flip(&Some("beta".into()), &None, &camp));
        assert!(!is_campaign_only_flip(
            &Some("alpha".into()),
            &Some("beta".into()),
            &std::collections::HashSet::new()
        ));
    }

    /// A campaign-only flip must NOT reselect a live session whose current model
    /// is still selectable; a non-campaign flip must. "Campaign-driven" is marked
    /// by `default_is_campaign_driven` on the incoming config.
    #[test]
    fn campaign_only_flip_does_not_reselect_live_session() {
        let mgr = test_manager();
        let mut cfg = config::Config::default();
        cfg.models.default = Some("alpha".to_string());
        mgr.apply_refresh_result(&cfg, Some(make_prefetched(&["alpha", "beta"])), None);
        *mgr.inner.cfg.write() = cfg.clone(); // old_preferred = "alpha"
        assert_eq!(mgr.current_model_id().0.as_ref(), "alpha");

        let mut new_cfg = config::Config::default();
        new_cfg.models.default = Some("beta".to_string());
        new_cfg.models.default_is_campaign_driven = true; // campaign overriding
        mgr.apply_config(new_cfg);
        assert_eq!(
            mgr.current_model_id().0.as_ref(),
            "alpha",
            "campaign-only flip must not yank a still-selectable live session"
        );

        // Control: same flip with no campaign (no pre_campaign_default) → reselect.
        let mgr2 = test_manager();
        let mut cfg2 = config::Config::default();
        cfg2.models.default = Some("alpha".to_string());
        mgr2.apply_refresh_result(&cfg2, Some(make_prefetched(&["alpha", "beta"])), None);
        *mgr2.inner.cfg.write() = cfg2.clone();
        let mut new_cfg2 = config::Config::default();
        new_cfg2.models.default = Some("beta".to_string());
        mgr2.apply_config(new_cfg2);
        assert_eq!(
            mgr2.current_model_id().0.as_ref(),
            "beta",
            "a non-campaign preferred change must reselect"
        );
    }

    /// A campaign default missing from the catalog falls back to
    /// `pre_campaign_default`, then to the first visible model — and only when
    /// the missing pref is actually the campaign-driven config value.
    #[test]
    fn unavailable_campaign_default_falls_back_to_config_default() {
        let catalog = make_prefetched(&["real-model", "other-model"]);

        let mut cfg = config::Config::default();
        cfg.models.default = Some("missing-model".to_string());
        cfg.models.default_is_campaign_driven = true;
        cfg.models.pre_campaign_default = Some("real-model".to_string());
        let (key, _, _) = resolve_default_model(&cfg, &catalog, true);
        assert_eq!(
            key, "real-model",
            "must fall back to the pre-campaign default"
        );

        // Control: pre-campaign default also absent → first visible model.
        let mut cfg2 = config::Config::default();
        cfg2.models.default = Some("missing-model".to_string());
        cfg2.models.default_is_campaign_driven = true;
        cfg2.models.pre_campaign_default = Some("also-missing".to_string());
        let (key2, _, _) = resolve_default_model(&cfg2, &catalog, true);
        assert_eq!(&key2, catalog.keys().next().unwrap());

        // Control: not campaign-driven (e.g. stale recovery value alongside a
        // user-set default) → the campaign detour must NOT fire; a missing
        // config pref falls to the first visible model.
        let mut cfg3 = config::Config::default();
        cfg3.models.default = Some("missing-model".to_string());
        cfg3.models.pre_campaign_default = Some("real-model".to_string());
        let (key3, _, _) = resolve_default_model(&cfg3, &catalog, true);
        assert_eq!(
            &key3,
            catalog.keys().next().unwrap(),
            "non-campaign catalog miss must not recover via campaign state"
        );

        // Control: CLI override misses the catalog while campaign state is set
        // → CLI is not a campaign problem; no campaign detour.
        let mut cfg4 = config::Config {
            default_model_override: Some("missing-cli-model".to_string()),
            ..Default::default()
        };
        cfg4.models.default = Some("campaign-model".to_string());
        cfg4.models.default_is_campaign_driven = true;
        cfg4.models.pre_campaign_default = Some("real-model".to_string());
        let (key4, _, _) = resolve_default_model(&cfg4, &catalog, true);
        assert_eq!(
            &key4,
            catalog.keys().next().unwrap(),
            "a CLI pref miss must not detour through pre_campaign_default"
        );
    }

    // ── ModelFetchAuth::resolve priority tests ──────────────────────

    use serial_test::serial;
    use xai_grok_test_support::EnvGuard;

    #[test]
    #[serial]
    fn resolve_custom_endpoint_always_wins() {
        let _key = EnvGuard::set("XAI_API_KEY", "test-key");
        let endpoints = config::EndpointsConfig {
            models_base_url: Some("https://custom.example.com".to_owned()),
            ..config::EndpointsConfig::default()
        };
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, true),
            ModelFetchAuth::CustomEndpoint,
        );
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, false),
            ModelFetchAuth::CustomEndpoint,
        );
    }

    #[test]
    #[serial]
    fn resolve_cached_session_wins_over_api_key() {
        let _key = EnvGuard::set("XAI_API_KEY", "test-key");
        let endpoints = config::EndpointsConfig::default();
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, true),
            ModelFetchAuth::Session,
            "cached session should take priority over API key",
        );
    }

    #[test]
    #[serial]
    fn resolve_api_key_used_when_no_session() {
        let _key = EnvGuard::set("XAI_API_KEY", "test-key");
        let endpoints = config::EndpointsConfig::default();
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, false),
            ModelFetchAuth::ApiKey,
            "API key should be used when no cached session exists",
        );
    }

    #[test]
    #[serial]
    fn resolve_falls_back_to_session_when_nothing_set() {
        let _unset = EnvGuard::unset("XAI_API_KEY");
        let _unset_legacy = EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let endpoints = config::EndpointsConfig::default();
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, false),
            ModelFetchAuth::Session,
            "should fall back to Session when nothing else is configured",
        );
    }

    #[test]
    #[serial]
    fn resolve_deployment_key_when_no_session_or_api_key() {
        let _unset = EnvGuard::unset("XAI_API_KEY");
        let _unset_legacy = EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let endpoints = config::EndpointsConfig {
            deployment_key: Some("deploy-key".to_owned()),
            ..config::EndpointsConfig::default()
        };
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, false),
            ModelFetchAuth::Deployment,
        );
    }

    #[test]
    fn deployment_cache_scope_changes_with_deployment_key_without_session() {
        let first = config::EndpointsConfig {
            deployment_key: Some("deploy-a".to_owned()),
            ..config::EndpointsConfig::default()
        };
        let second = config::EndpointsConfig {
            deployment_key: Some("deploy-b".to_owned()),
            ..config::EndpointsConfig::default()
        };
        let first_scope =
            xai_catalog_credential_scope(&first, None, ModelFetchAuth::Deployment).unwrap();
        let second_scope =
            xai_catalog_credential_scope(&second, None, ModelFetchAuth::Deployment).unwrap();
        assert_ne!(first_scope, second_scope);
        assert!(!first_scope.contains("deploy-a"));
        assert!(!second_scope.contains("deploy-b"));
    }

    #[test]
    fn external_cache_scope_changes_with_bearer_even_when_profile_is_carried() {
        let endpoints = config::EndpointsConfig::default();
        let mut first = GrokAuth {
            auth_mode: AuthMode::External,
            key: "external-bearer-a".to_owned(),
            user_id: "carried-user".to_owned(),
            principal_id: Some("carried-principal".to_owned()),
            team_id: Some("carried-team".to_owned()),
            ..GrokAuth::default()
        };
        let first_scope =
            xai_catalog_credential_scope(&endpoints, Some(&first), ModelFetchAuth::Session)
                .unwrap();
        first.key = "external-bearer-b".to_owned();
        let second_scope =
            xai_catalog_credential_scope(&endpoints, Some(&first), ModelFetchAuth::Session)
                .unwrap();

        assert_ne!(first_scope, second_scope);
        assert!(!first_scope.contains("external-bearer-a"));
        assert!(!second_scope.contains("external-bearer-b"));
    }

    #[test]
    fn first_party_team_cache_scope_includes_request_user_id() {
        let endpoints = config::EndpointsConfig::default();
        let mut first = GrokAuth {
            auth_mode: AuthMode::Oidc,
            key: "rotating-access-token".to_owned(),
            user_id: "team-user-a".to_owned(),
            principal_type: Some("Team".to_owned()),
            principal_id: Some("shared-principal".to_owned()),
            team_id: Some("shared-team".to_owned()),
            ..GrokAuth::default()
        };
        let first_scope =
            xai_catalog_credential_scope(&endpoints, Some(&first), ModelFetchAuth::Session)
                .unwrap();
        first.user_id = "team-user-b".to_owned();
        let second_scope =
            xai_catalog_credential_scope(&endpoints, Some(&first), ModelFetchAuth::Session)
                .unwrap();

        assert_ne!(first_scope, second_scope);
    }

    #[test]
    fn first_party_cache_scope_survives_access_token_rotation() {
        let endpoints = config::EndpointsConfig::default();
        let mut auth = GrokAuth {
            auth_mode: AuthMode::Oidc,
            key: "access-token-a".to_owned(),
            user_id: "stable-user".to_owned(),
            principal_id: Some("stable-principal".to_owned()),
            ..GrokAuth::default()
        };
        let first_scope =
            xai_catalog_credential_scope(&endpoints, Some(&auth), ModelFetchAuth::Session).unwrap();
        auth.key = "access-token-b".to_owned();
        let second_scope =
            xai_catalog_credential_scope(&endpoints, Some(&auth), ModelFetchAuth::Session).unwrap();

        assert_eq!(first_scope, second_scope);
    }

    #[test]
    fn network_prepare_clears_external_account_a_before_account_b_fetch() {
        let home = tempfile::tempdir().unwrap();
        let auth_manager = Arc::new(AuthManager::new(home.path(), GrokComConfig::default()));
        let auth_a = GrokAuth {
            auth_mode: AuthMode::External,
            key: "external-a".to_owned(),
            user_id: "carried-user".to_owned(),
            principal_id: Some("carried-principal".to_owned()),
            ..GrokAuth::default()
        };
        auth_manager.hot_swap(auth_a.clone());
        let manager = ModelsManager::new(
            Some(IndexMap::new()),
            IndexMap::new(),
            acp::ModelId::new("default"),
            Arc::clone(&auth_manager),
            config::Config::default(),
        );
        let scope_a = xai_catalog_credential_scope(
            &config::EndpointsConfig::default(),
            Some(&auth_a),
            ModelFetchAuth::Session,
        );
        *manager.inner.has_xai_catalog.write() = true;
        *manager.inner.xai_catalog_credential_scope.write() = scope_a.clone();

        let launch = manager.begin_xai_catalog_operation();
        let auth_b = GrokAuth {
            key: "external-b".to_owned(),
            ..auth_a
        };
        let (prepared, cleared) = manager
            .prepare_xai_network_operation(&launch, Some(&auth_b))
            .expect("latest operation should remain current");

        assert!(cleared);
        assert!(!*manager.inner.has_xai_catalog.read());
        assert_eq!(*manager.inner.xai_catalog_credential_scope.read(), None);
        assert_ne!(prepared.credential_scope, scope_a);
        assert_eq!(
            prepared.generation,
            manager.inner.xai_fetch_generation.load(Ordering::Acquire)
        );
    }

    /// `deployment_key` outranks a stray `XAI_API_KEY`, but session wins over both.
    #[test]
    #[serial]
    fn resolve_deployment_key_outranks_ambient_api_key() {
        let _key = EnvGuard::set("XAI_API_KEY", "stray-env-key");
        let endpoints = config::EndpointsConfig {
            deployment_key: Some("deploy-key".to_owned()),
            ..config::EndpointsConfig::default()
        };
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, false),
            ModelFetchAuth::Deployment,
            "managed deployment_key should outrank an ambient XAI_API_KEY",
        );
        assert_eq!(
            ModelFetchAuth::resolve(&endpoints, true),
            ModelFetchAuth::Session,
            "an active session should still win over a managed deployment",
        );
    }

    // ── remote_fetch gate: resolve_prefetch_env_from_parts ───────────

    /// remote_fetch=false must return `None` against every re-arming shape at
    /// once — session auth, ambient `XAI_API_KEY`, `deployment_key`, AND a
    /// custom models endpoint (which normally forces the prefetch to run).
    #[test]
    #[serial]
    fn prefetch_env_none_when_remote_fetch_disabled_despite_credentials() {
        let _key = EnvGuard::set("XAI_API_KEY", "stray-env-key");
        let endpoints = config::EndpointsConfig {
            deployment_key: Some("deploy-key".to_owned()),
            models_base_url: Some("https://custom.example.com".to_owned()),
            ..config::EndpointsConfig::default()
        };
        assert!(
            resolve_prefetch_env_from_parts(
                Some(GrokAuth::test_default()),
                endpoints.clone(),
                false,
            )
            .is_none(),
            "session auth must not re-arm the prefetch when remote_fetch is off",
        );
        assert!(
            resolve_prefetch_env_from_parts(None, endpoints, false).is_none(),
            "API key / deployment key / custom endpoint must not re-arm it either",
        );
    }

    /// Inverse sanity: with remote_fetch enabled the same credential shapes DO
    /// arm the prefetch, and the credential-less default still doesn't.
    #[test]
    #[serial]
    fn prefetch_env_resolves_when_remote_fetch_enabled() {
        let _unset = EnvGuard::unset("XAI_API_KEY");
        let _unset_legacy = EnvGuard::unset("GROK_CODE_XAI_API_KEY");
        let endpoints = config::EndpointsConfig {
            deployment_key: Some("deploy-key".to_owned()),
            ..config::EndpointsConfig::default()
        };
        assert!(resolve_prefetch_env_from_parts(None, endpoints, true).is_some());
        assert!(
            resolve_prefetch_env_from_parts(None, config::EndpointsConfig::default(), true)
                .is_none(),
            "no credentials and no custom endpoint must stay a no-prefetch launch",
        );
    }

    /// remote_fetch=false: an online catalog refresh is a no-op — nothing is
    /// fetched, no real-catalog flag is set, and the static catalog keeps
    /// resolving. Covers `list_models`/`do_refresh` online strategies too,
    /// which funnel into `fetch_and_apply`/`spawn_fetch`.
    #[tokio::test]
    async fn fetch_and_apply_degrades_offline_when_remote_fetch_disabled() {
        let mgr = test_manager();
        mgr.insert_test_entry(
            "static-one",
            ModelEntry {
                info: config::ModelInfo::fallback("static-one"),
                api_key: None,
                env_key: None,
                api_base_url: None,
            },
        );

        mgr.fetch_and_apply_inner(false).await;

        assert!(
            !mgr.has_fetched_real_catalog(),
            "no catalog fetch may be recorded when remote_fetch is disabled",
        );
        assert!(
            mgr.models().contains_key("static-one"),
            "the static catalog must keep resolving",
        );
    }

    // ── supported_in_api tests ──────────────────────────────────────

    #[test]
    fn default_model_skips_oauth_only_for_api_key_users() {
        let cfg = config::Config::default();
        let mut catalog = IndexMap::new();

        let mut oauth_only = ModelEntry {
            info: config::ModelInfo::fallback("oauth-only"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        oauth_only.info.supported_in_api = false;
        catalog.insert("oauth-only".to_string(), oauth_only);

        let public = ModelEntry {
            info: config::ModelInfo::fallback("public-model"),
            api_key: None,
            env_key: None,
            api_base_url: None,
        };
        catalog.insert("public-model".to_string(), public);

        // API-key user: default should NOT be the oauth-only model
        let (key, _, _) = resolve_default_model(&cfg, &catalog, false);
        assert_ne!(
            key, "oauth-only",
            "API-key default must not be an OAuth-only model"
        );
        assert_eq!(key, "public-model");

        // OAuth user: oauth-only is valid as default (it's first in the map)
        let (key, _, _) = resolve_default_model(&cfg, &catalog, true);
        assert!(
            key == "oauth-only" || key == "public-model",
            "OAuth user should be able to use either model as default"
        );
    }

    #[test]
    fn visible_for_auth_logic() {
        let mut info = config::ModelInfo::fallback("test");

        // Default: visible to everyone
        assert!(info.visible_for_auth(true));
        assert!(info.visible_for_auth(false));

        // hidden = true: invisible to everyone
        info.hidden = true;
        assert!(!info.visible_for_auth(true));
        assert!(!info.visible_for_auth(false));

        // hidden = false, supported_in_api = false: visible to session only
        info.hidden = false;
        info.supported_in_api = false;
        assert!(info.visible_for_auth(true));
        assert!(!info.visible_for_auth(false));
    }

    // ── duplicate model slug re-keying (A/B experiment "auto" alias) ──

    fn make_entry_config(model: &str, name: Option<&str>) -> config::ModelEntryConfig {
        make_entry_config_with_id(None, model, name)
    }

    fn make_entry_config_with_id(
        id: Option<&str>,
        model: &str,
        name: Option<&str>,
    ) -> config::ModelEntryConfig {
        config::ModelEntryConfig {
            id: id.map(|s| s.to_owned()),
            provider: xai_grok_sampling_types::ProviderId::Xai,
            model: model.to_owned(),
            base_url: "https://test.api/v1".to_owned(),
            name: name.map(|n| n.to_owned()),
            description: None,
            max_completion_tokens: None,
            temperature: None,
            top_p: None,
            api_key: None,
            env_key: None,
            api_backend: Default::default(),
            context_window: std::num::NonZeroU64::new(200_000).unwrap(),
            auto_compact_threshold_percent: None,
            system_prompt_label: None,
            extra_headers: IndexMap::new(),
            api_base_url: None,
            use_concise: false,
            agent_type: config::default_agent_type(),
            inference_idle_timeout_secs: None,
            max_retries: None,
            hidden: false,
            supported_in_api: true,
            auth_scheme: None,
            reasoning_effort: None,
            supports_reasoning_effort: false,
            reasoning_efforts: Vec::new(),
            supports_backend_search: false,
            compactions_remaining: None,
            compaction_at_tokens: None,
            show_model_fingerprint: false,
            stream_tool_calls: None,
            laziness_detector: config::LazinessDetectorPerModelConfig::default(),
        }
    }

    /// Experiment: two entries share the same routing slug but have distinct ids.
    /// Both survive, keyed by their respective ids.
    #[test]
    fn build_prefetched_map_distinct_ids_same_slug() {
        let entries = vec![
            make_entry_config_with_id(Some("auto"), "grok-build", Some("Auto")),
            make_entry_config_with_id(Some("grok-build"), "grok-build", Some("Grok Build")),
            make_entry_config_with_id(
                Some("grok-composer-2.5-fast"),
                "grok-composer-2.5-fast",
                Some("Grok Fast"),
            ),
        ];
        let map = build_prefetched_map(entries, None);

        assert_eq!(map.len(), 3, "all three entries should survive");
        assert!(map.contains_key("auto"));
        assert!(map.contains_key("grok-build"));
        assert!(map.contains_key("grok-composer-2.5-fast"));
        assert_eq!(
            map["auto"].info.model, "grok-build",
            "auto entry should still route to grok-build"
        );
        assert_eq!(map["grok-build"].info.model, "grok-build");
    }

    /// No id field — falls back to model slug as key.
    #[test]
    fn build_prefetched_map_no_id_falls_back_to_slug() {
        let entries = vec![
            make_entry_config("model-a", Some("Model A")),
            make_entry_config("model-b", Some("Model B")),
        ];
        let map = build_prefetched_map(entries, None);

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("model-a"));
        assert!(map.contains_key("model-b"));
    }

    /// Duplicate ids — second overwrites first (same as duplicate slugs before).
    #[test]
    fn build_prefetched_map_duplicate_id_overwrites() {
        let entries = vec![
            make_entry_config_with_id(Some("grok-build"), "grok-build", Some("First")),
            make_entry_config_with_id(Some("grok-build"), "grok-build", Some("Second")),
        ];
        let map = build_prefetched_map(entries, None);

        assert_eq!(map.len(), 1, "duplicate id: second overwrites first");
        assert_eq!(map["grok-build"].info.name.as_deref(), Some("Second"));
    }

    /// Regression: resolve_default_model must match by id before scanning
    /// by model slug, otherwise entries sharing a slug resolve to whichever
    /// appears first in the catalog.
    #[test]
    fn resolve_default_model_prefers_id_over_model_slug() {
        let mut catalog: IndexMap<String, ModelEntry> = IndexMap::new();
        catalog.insert(
            "auto-grok-build".to_string(),
            make_model_entry("grok-build"),
        );
        catalog.insert("grok-build".to_string(), make_model_entry("grok-build"));

        let mut cfg = config::Config::default();
        cfg.models.default = Some("grok-build".to_string());

        let (key, _, _) = resolve_default_model(&cfg, &catalog, true);
        assert_eq!(key, "grok-build", "must match id, not first slug hit");
    }

    /// No id field — falls back to slug as key.
    #[test]
    fn build_prefetched_map_none_id_falls_back_to_slug() {
        let entries = vec![make_entry_config_with_id(
            None,
            "grok-build",
            Some("Grok Build"),
        )];
        let map = build_prefetched_map(entries, None);

        assert_eq!(map.len(), 1);
        assert!(map.contains_key("grok-build"));
    }

    // ── persisted model id → catalog key (session resume) ─────────────

    #[test]
    fn resolve_catalog_key_maps_routing_slug_to_config_key() {
        let mut models = IndexMap::new();
        models.insert(
            "enterprise-grok-build".to_string(),
            make_model_entry("grok-4.5"),
        );
        models.insert("grok-4.3".to_string(), make_model_entry("grok-4.3"));

        let persisted = acp::ModelId::new("grok-4.5");
        let key = resolve_catalog_key(&models, &persisted).expect("slug must resolve");
        assert_eq!(key.0.as_ref(), "enterprise-grok-build");
    }

    #[test]
    fn resolve_catalog_key_prefers_exact_key_match() {
        let mut models = IndexMap::new();
        models.insert("grok-4.5".to_string(), make_model_entry("grok-4.5"));

        let persisted = acp::ModelId::new("grok-4.5");
        let key = resolve_catalog_key(&models, &persisted).expect("exact key must resolve");
        assert_eq!(key.0.as_ref(), "grok-4.5");
    }

    #[test]
    fn resolve_catalog_key_last_slug_match_wins() {
        let mut models = IndexMap::new();
        models.insert(
            "default-grok-build".to_string(),
            make_model_entry("grok-4.5"),
        );
        models.insert("user-grok-build".to_string(), make_model_entry("grok-4.5"));

        let persisted = acp::ModelId::new("grok-4.5");
        let key = resolve_catalog_key(&models, &persisted).expect("slug must resolve");
        assert_eq!(key.0.as_ref(), "user-grok-build");
    }

    #[test]
    fn selectable_catalog_key_for_persisted_none_when_resolved_not_available() {
        let mut models = IndexMap::new();
        models.insert(
            "enterprise-grok-build".to_string(),
            make_model_entry("grok-4.5"),
        );

        let available: IndexMap<_, _> = IndexMap::new();
        let persisted = acp::ModelId::new("grok-4.5");
        assert!(
            selectable_catalog_key_for_persisted(&models, &available, &persisted, true).is_none()
        );
    }

    #[test]
    fn selectable_prefers_available_identity_over_non_selectable_exact_key() {
        let mut models = IndexMap::new();
        models.insert("grok-build".to_string(), make_model_entry("grok-build"));
        models.insert(
            "enterprise-grok-build".to_string(),
            make_model_entry("grok-build"),
        );
        models.insert("grok-4.3".to_string(), make_model_entry("grok-4.3"));

        let available = test_available_keys(&["enterprise-grok-build", "grok-4.3"]);

        let persisted = acp::ModelId::new("grok-build");
        assert_eq!(
            resolve_catalog_key(&models, &persisted)
                .expect("exact key exists")
                .0
                .as_ref(),
            "grok-build"
        );
        let key = selectable_catalog_key_for_persisted(&models, &available, &persisted, true)
            .expect("must resolve to selectable section");
        assert_eq!(key.0.as_ref(), "enterprise-grok-build");
    }

    #[test]
    fn selectable_matches_routing_slug_when_no_exact_key() {
        let mut models = IndexMap::new();
        models.insert(
            "enterprise-grok-build".to_string(),
            make_model_entry("grok-build"),
        );
        models.insert("grok-4.3".to_string(), make_model_entry("grok-4.3"));

        let available = test_available_keys(&["enterprise-grok-build", "grok-4.3"]);

        let persisted = acp::ModelId::new("grok-build");
        let key = selectable_catalog_key_for_persisted(&models, &available, &persisted, true)
            .expect("slug must resolve to selectable key");
        assert_eq!(key.0.as_ref(), "enterprise-grok-build");
    }

    /// A persisted *selectable* catalog key binds to itself even when a later
    /// selectable section's routing slug equals that key (exact key wins).
    #[test]
    fn selectable_prefers_exact_key_over_later_slug_match() {
        let mut models = IndexMap::new();
        models.insert("grok-build".to_string(), make_model_entry("grok-4.5"));
        models.insert("other".to_string(), make_model_entry("grok-build"));

        let available = test_available_keys(&["grok-build", "other"]);

        let persisted = acp::ModelId::new("grok-build");
        let key = selectable_catalog_key_for_persisted(&models, &available, &persisted, true)
            .expect("exact selectable key must win");
        assert_eq!(key.0.as_ref(), "grok-build");
    }

    #[test]
    fn provider_scoped_persisted_resolution_ignores_same_slug_xai_entry() {
        let mut models = IndexMap::new();
        models.insert("xai-shadow".to_string(), make_model_entry("gpt-5.6-sol"));
        let mut codex = make_model_entry("gpt-5.6-sol");
        codex.info.provider = ProviderId::OpenAiCodex;
        models.insert("openai-codex/gpt-5.6-sol".to_string(), codex);
        let available = test_available_keys(&["xai-shadow", "openai-codex/gpt-5.6-sol"]);

        let key = selectable_catalog_key_for_persisted_provider(
            &models,
            &available,
            &acp::ModelId::new("gpt-5.6-sol"),
            true,
            ProviderId::OpenAiCodex,
        )
        .expect("Codex routing slug must remain provider-scoped");
        assert_eq!(key.0.as_ref(), "openai-codex/gpt-5.6-sol");
    }

    #[test]
    fn provider_scoped_fallback_never_crosses_from_codex_to_xai() {
        let mut models = IndexMap::new();
        models.insert("grok-4".to_string(), make_model_entry("grok-4"));
        let mut codex = make_model_entry("gpt-5.6-terra");
        codex.info.provider = ProviderId::OpenAiCodex;
        models.insert("openai-codex/gpt-5.6-terra".to_string(), codex);

        let only_xai = test_available_keys(&["grok-4"]);
        assert!(
            first_available_catalog_key_for_provider(&models, &only_xai, ProviderId::OpenAiCodex,)
                .is_none(),
            "an xAI model must never satisfy a Codex fallback"
        );

        let both = test_available_keys(&["grok-4", "openai-codex/gpt-5.6-terra"]);
        let fallback =
            first_available_catalog_key_for_provider(&models, &both, ProviderId::OpenAiCodex)
                .expect("entitled Codex fallback must be selected");
        assert_eq!(fallback.0.as_ref(), "openai-codex/gpt-5.6-terra");
    }

    fn test_available_keys(keys: &[&str]) -> IndexMap<acp::ModelId, acp::ModelInfo> {
        keys.iter()
            .map(|k| {
                let id = acp::ModelId::new(*k);
                (id.clone(), acp::ModelInfo::new(id, (*k).to_string()))
            })
            .collect()
    }
}
