//! Provider-local lifecycle controller for the authenticated ChatGPT Codex
//! model catalog.
//!
//! This controller owns Codex cache/ETag state, operation generations,
//! refresh/retry orchestration, catalog mapping, and auth-identity
//! reconciliation. The parent [`ModelsManager`](super::ModelsManager) retains
//! the combined picker/sampler catalog and delegates its existing public APIs.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use indexmap::IndexMap;
use parking_lot::RwLock;
use xai_grok_sampling_types::{ProviderId, ReasoningEffort, ReasoningEffortOption};

use super::{ModelsManager, allowlist_matches_nothing};
use crate::agent::config::{self, ModelEntry};

/// Codex provider state is intentionally independent from xAI's cache,
/// identity, ETag, generation, and retry lifecycle.
pub(super) struct CodexCatalogController {
    models: RwLock<IndexMap<String, ModelEntry>>,
    has_catalog: RwLock<bool>,
    identity: RwLock<Option<CodexCatalogIdentity>>,
    needs_revalidation: AtomicBool,
    etag: RwLock<Option<String>>,
    fetch_generation: AtomicU64,
    remote_fetch_enabled: AtomicBool,
    refresh_gate: tokio::sync::Mutex<()>,
    retry_in_flight: AtomicBool,
}

/// In-memory scope for an entitled Codex catalog. This is deliberately never
/// logged or persisted; the disk cache uses its own one-way account binding.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct CodexCatalogIdentity {
    credential_id: String,
    account_id: String,
    user_id: Option<String>,
    is_fedramp: bool,
}

impl CodexCatalogIdentity {
    fn from_credentials(credentials: &crate::auth::codex::CodexCredentials) -> Self {
        Self {
            credential_id: credentials.credential_id().to_owned(),
            account_id: credentials.account_id().to_owned(),
            user_id: credentials.user_id().map(ToOwned::to_owned),
            is_fedramp: credentials.is_fedramp,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(
        credential_id: &str,
        account_id: &str,
        user_id: Option<&str>,
        is_fedramp: bool,
    ) -> Self {
        Self {
            credential_id: credential_id.to_owned(),
            account_id: account_id.to_owned(),
            user_id: user_id.map(ToOwned::to_owned),
            is_fedramp,
        }
    }
}

impl std::fmt::Debug for CodexCatalogIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Account, local record, user, and FedRAMP state are not diagnostics.
        formatter
            .debug_struct("CodexCatalogIdentity")
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CodexCatalogRefreshOutcome {
    Updated,
    CachedFallback,
    TransientFailure,
    TerminalFailure,
    NotAuthenticated,
    Disabled,
}

impl CodexCatalogRefreshOutcome {
    fn updated(self) -> bool {
        matches!(self, Self::Updated | Self::CachedFallback)
    }
}

/// Startup result whose identity lease must remain live through publication in
/// the parent manager.
pub(super) struct CodexPrefetchResult {
    pub(super) models: IndexMap<String, ModelEntry>,
    pub(super) etag: Option<String>,
    pub(super) identity: CodexCatalogIdentity,
    pub(super) authoritative: bool,
    pub(super) _identity_lease: crate::auth::codex::CodexCatalogIdentityLease,
}

impl CodexCatalogController {
    pub(super) fn new(models: IndexMap<String, ModelEntry>, remote_fetch_enabled: bool) -> Self {
        Self {
            models: RwLock::new(models),
            has_catalog: RwLock::new(false),
            identity: RwLock::new(None),
            needs_revalidation: AtomicBool::new(false),
            etag: RwLock::new(None),
            fetch_generation: AtomicU64::new(1),
            remote_fetch_enabled: AtomicBool::new(remote_fetch_enabled),
            refresh_gate: tokio::sync::Mutex::new(()),
            retry_in_flight: AtomicBool::new(false),
        }
    }

    pub(super) fn models_snapshot(&self) -> IndexMap<String, ModelEntry> {
        self.models.read().clone()
    }

    pub(super) fn has_catalog(&self) -> bool {
        *self.has_catalog.read()
    }

    pub(super) fn needs_revalidation(&self) -> bool {
        self.needs_revalidation.load(Ordering::Acquire)
    }

    fn set_needs_revalidation(&self, value: bool) {
        self.needs_revalidation.store(value, Ordering::Release);
    }

    pub(super) fn initialize_prefetch(
        &self,
        etag: Option<String>,
        identity: Option<CodexCatalogIdentity>,
        needs_revalidation: bool,
        has_catalog: bool,
    ) {
        *self.has_catalog.write() = has_catalog;
        *self.etag.write() = etag;
        *self.identity.write() = identity;
        self.set_needs_revalidation(needs_revalidation);
    }

    /// Invalidate in-flight Codex work for an accepted config transition and
    /// atomically adopt the runtime network policy under the parent's catalog
    /// publication gate.
    pub(super) fn advance_config_generation(&self, remote_fetch_enabled: bool) {
        self.fetch_generation.fetch_add(1, Ordering::AcqRel);
        self.remote_fetch_enabled
            .store(remote_fetch_enabled, Ordering::Release);
    }

    pub(super) fn recovery_needed(&self) -> bool {
        !self.has_catalog() || self.needs_revalidation()
    }

    #[cfg(test)]
    pub(super) fn etag(&self) -> Option<String> {
        self.etag.read().clone()
    }

    #[cfg(test)]
    pub(super) fn identity_is_none(&self) -> bool {
        self.identity.read().is_none()
    }

    #[cfg(test)]
    pub(super) fn generation(&self) -> u64 {
        self.fetch_generation.load(Ordering::Acquire)
    }

    pub(super) async fn refresh_models(&self, owner: &ModelsManager) -> bool {
        self.refresh(owner, None).await.updated()
    }

    async fn lock_binding(
        &self,
        owner: &ModelsManager,
        manager: &crate::auth::codex::CodexAuthManager,
        expected: &xai_grok_sampling_types::CredentialBinding,
    ) -> Result<
        (
            crate::auth::codex::CodexCredentials,
            crate::auth::codex::CodexCatalogIdentityLease,
        ),
        CodexCatalogRefreshOutcome,
    > {
        match manager.lock_catalog_binding(expected).await {
            Ok(locked) => Ok(locked),
            Err(error) => {
                // Account changes and logout are expected races; storage/lock
                // failures fail closed because the prior account scope can no
                // longer be proven current.
                let current = manager.reload().ok().flatten();
                self.reconcile_identity(owner, None);
                tracing::warn!(error = %error, "OpenAI Codex catalog identity verification failed");
                Err(catalog_auth_failure_outcome(&error, current.is_some()))
            }
        }
    }

    async fn attest_identity_after_failure(
        &self,
        owner: &ModelsManager,
        manager: &crate::auth::codex::CodexAuthManager,
    ) -> Result<(), CodexCatalogRefreshOutcome> {
        let current = match manager.current_verified() {
            Ok(current) => current,
            Err(error) => {
                let credentials_still_exist = manager.reload().ok().flatten().is_some();
                self.reconcile_identity(owner, None);
                return Err(catalog_auth_failure_outcome(
                    &error,
                    credentials_still_exist,
                ));
            }
        };
        let (_credentials, _lease) = self
            .lock_binding(owner, manager, &current.credential_binding())
            .await?;
        Ok(())
    }

    fn config_rejection_outcome(&self) -> CodexCatalogRefreshOutcome {
        if self.remote_fetch_enabled.load(Ordering::Acquire) {
            CodexCatalogRefreshOutcome::TransientFailure
        } else {
            CodexCatalogRefreshOutcome::Disabled
        }
    }

    async fn refresh(
        &self,
        owner: &ModelsManager,
        observed: Option<(String, xai_grok_sampling_types::CredentialBinding)>,
    ) -> CodexCatalogRefreshOutcome {
        let _refresh = self.refresh_gate.lock().await;
        let operation_generation = self.fetch_generation.load(Ordering::Acquire);
        let grok_home = crate::util::grok_home::grok_home();
        let manager = match crate::auth::codex::CodexAuthManager::new(&grok_home) {
            Ok(manager) => manager,
            Err(error) => {
                self.reconcile_identity(owner, None);
                tracing::warn!(error = %error, "could not load OpenAI Codex credentials for model refresh");
                return CodexCatalogRefreshOutcome::TerminalFailure;
            }
        };
        let credentials = match manager.current() {
            Some(credentials) => credentials,
            None => {
                self.reconcile_identity(owner, None);
                tracing::debug!("OpenAI Codex model refresh skipped: not logged in");
                return CodexCatalogRefreshOutcome::NotAuthenticated;
            }
        };
        let identity = CodexCatalogIdentity::from_credentials(&credentials);

        // Invalidate account A before attempting account B's network request.
        // A failed B refresh must never leave A's entitlements visible.
        self.reconcile_identity(owner, Some(&identity));

        if !self.remote_fetch_enabled.load(Ordering::Acquire) {
            let _identity_lease = match self
                .lock_binding(owner, &manager, &credentials.credential_binding())
                .await
            {
                Ok((_credentials, lease)) => lease,
                Err(outcome) => return outcome,
            };
            tracing::debug!("OpenAI Codex model refresh skipped: remote_fetch disabled");
            return CodexCatalogRefreshOutcome::Disabled;
        }
        let config = match crate::remote::CodexCatalogClientConfig::new(grok_home.join("cache")) {
            Ok(config) => config,
            Err(error) => {
                let _ = self
                    .lock_binding(owner, &manager, &credentials.credential_binding())
                    .await;
                tracing::warn!(error = %error, "could not configure OpenAI Codex model refresh");
                return CodexCatalogRefreshOutcome::TerminalFailure;
            }
        };
        let client = match crate::remote::CodexCatalogClient::new(config) {
            Ok(client) => client,
            Err(error) => {
                let _ = self
                    .lock_binding(owner, &manager, &credentials.credential_binding())
                    .await;
                tracing::warn!(error = %error, "could not create OpenAI Codex model client");
                return CodexCatalogRefreshOutcome::TerminalFailure;
            }
        };

        if let Some((observed_etag, observed_binding)) = observed.as_ref()
            && observed_binding == &credentials.credential_binding()
            && self.has_catalog()
            && self.identity.read().as_ref() == Some(&identity)
            && self.etag.read().as_deref() == Some(observed_etag.as_str())
            && !self.needs_revalidation()
        {
            // The matching ETag came from a live authenticated Codex response,
            // so it is authoritative for the current account. Renew only that
            // account's disk cache, then hold the identity lease through the
            // in-memory validation marker update.
            let auth = crate::auth::codex::CodexAuthSnapshot::new(
                credentials.access_token(),
                credentials.account_id(),
                credentials.user_id(),
                credentials.credential_binding(),
                credentials.is_fedramp,
            );
            let renewed_binding = match auth {
                Ok(auth) => match client.renew_cached(&auth, observed_etag).await {
                    Ok(binding) => binding,
                    Err(error) => {
                        tracing::debug!(error = %error, "could not renew OpenAI Codex catalog cache TTL");
                        observed_binding.clone()
                    }
                },
                Err(_) => return CodexCatalogRefreshOutcome::TerminalFailure,
            };
            let (_locked_credentials, _identity_lease) =
                match self.lock_binding(owner, &manager, &renewed_binding).await {
                    Ok(locked) => locked,
                    Err(outcome) => return outcome,
                };
            let _catalog_write = owner.inner.catalog_write_gate.lock();
            if self.fetch_generation.load(Ordering::Acquire) != operation_generation
                || !self.remote_fetch_enabled.load(Ordering::Acquire)
            {
                return self.config_rejection_outcome();
            }
            self.set_needs_revalidation(false);
            return CodexCatalogRefreshOutcome::Updated;
        }

        let (catalog, etag, authoritative, request_binding) = match client.fetch(&manager).await {
            Ok(fetched) => (
                fetched.catalog,
                fetched.etag,
                true,
                fetched.credential_binding,
            ),
            Err(error) if catalog_error_is_transient(&error) => {
                match client.load_cached(&manager).await {
                    Ok(cached) => (
                        cached.catalog,
                        cached.etag,
                        false,
                        cached.credential_binding,
                    ),
                    Err(_) => {
                        if let Err(outcome) =
                            self.attest_identity_after_failure(owner, &manager).await
                        {
                            return outcome;
                        }
                        tracing::warn!(error = %error, "OpenAI Codex model refresh failed");
                        return CodexCatalogRefreshOutcome::TransientFailure;
                    }
                }
            }
            Err(error) => {
                if let Err(outcome) = self.attest_identity_after_failure(owner, &manager).await {
                    return outcome;
                }
                tracing::warn!(error = %error, "OpenAI Codex model refresh failed");
                return CodexCatalogRefreshOutcome::TerminalFailure;
            }
        };

        // Serialize the final identity check and publication with all shipped
        // login/logout/refresh writers. This closes the last account-switch
        // window between a post-request reload and in-memory installation.
        let (locked_credentials, _identity_lease) =
            match self.lock_binding(owner, &manager, &request_binding).await {
                Ok(locked) => locked,
                Err(outcome) => return outcome,
            };
        let identity = CodexCatalogIdentity::from_credentials(&locked_credentials);
        let etag = etag.or_else(|| {
            observed.and_then(|(etag, binding)| (binding == request_binding).then_some(etag))
        });
        if !self.try_apply_catalog(
            owner,
            catalog,
            etag,
            identity,
            authoritative,
            operation_generation,
        ) {
            return self.config_rejection_outcome();
        }
        if authoritative {
            CodexCatalogRefreshOutcome::Updated
        } else {
            CodexCatalogRefreshOutcome::CachedFallback
        }
    }

    fn try_apply_catalog(
        &self,
        owner: &ModelsManager,
        catalog: crate::remote::CodexModelCatalog,
        etag: Option<String>,
        identity: CodexCatalogIdentity,
        authoritative: bool,
        operation_generation: u64,
    ) -> bool {
        let catalog_write = owner.inner.catalog_write_gate.lock();
        if self.fetch_generation.load(Ordering::Acquire) != operation_generation
            || !self.remote_fetch_enabled.load(Ordering::Acquire)
        {
            return false;
        }
        let count = self.apply_catalog_unlocked(owner, catalog, etag, identity, authoritative);
        drop(catalog_write);
        tracing::info!(count, "OpenAI Codex model catalog refreshed");
        owner.notify_models_updated();
        true
    }

    fn apply_catalog_unlocked(
        &self,
        owner: &ModelsManager,
        catalog: crate::remote::CodexModelCatalog,
        etag: Option<String>,
        identity: CodexCatalogIdentity,
        authoritative: bool,
    ) -> usize {
        let mapped = Self::map_catalog(catalog);
        let count = mapped.len();
        let first_catalog = !self.has_catalog();
        *self.models.write() = mapped;
        *self.etag.write() = etag;
        *self.identity.write() = Some(identity);
        *self.has_catalog.write() = true;
        self.set_needs_revalidation(!authoritative);

        *owner.inner.has_fetched_real_catalog.write() = true;
        let config = owner.inner.cfg.read().clone();
        let prefetched = owner.inner.prefetched.read().clone();
        owner.rebuild_unlocked(&config, prefetched);

        let excludes_all = allowlist_matches_nothing(&config, &owner.inner.models.read());
        owner
            .inner
            .allowlist_excludes_all
            .store(excludes_all, Ordering::Relaxed);
        if first_catalog {
            owner.reselect_default_model(&config);
        } else {
            owner.reselect_current_model_if_missing(&config);
        }
        count
    }

    #[cfg(test)]
    pub(super) fn apply_catalog_for_test(
        &self,
        owner: &ModelsManager,
        catalog: crate::remote::CodexModelCatalog,
        etag: Option<String>,
        identity: CodexCatalogIdentity,
        authoritative: bool,
    ) {
        let catalog_write = owner.inner.catalog_write_gate.lock();
        let count = self.apply_catalog_unlocked(owner, catalog, etag, identity, authoritative);
        drop(catalog_write);
        tracing::info!(count, "OpenAI Codex model catalog refreshed");
        owner.notify_models_updated();
    }

    /// Clear only Codex provider state when its current credential/account no
    /// longer matches the catalog producer. xAI models and credentials remain
    /// untouched. Returns whether visible state changed.
    pub(super) fn reconcile_identity(
        &self,
        owner: &ModelsManager,
        current: Option<&CodexCatalogIdentity>,
    ) -> bool {
        let catalog_write = owner.inner.catalog_write_gate.lock();
        let has_state = self.has_catalog()
            || !self.models.read().is_empty()
            || self.etag.read().is_some()
            || self.identity.read().is_some();
        if !has_state || (self.identity.read().as_ref() == current && current.is_some()) {
            return false;
        }

        self.models.write().clear();
        *self.etag.write() = None;
        *self.identity.write() = None;
        *self.has_catalog.write() = false;
        self.set_needs_revalidation(false);

        let config = owner.inner.cfg.read().clone();
        let prefetched = owner.inner.prefetched.read().clone();
        let has_xai_catalog = *owner.inner.has_xai_catalog.read();
        *owner.inner.has_fetched_real_catalog.write() = has_xai_catalog;
        owner.rebuild_unlocked(&config, prefetched);
        let excludes_all = allowlist_matches_nothing(&config, &owner.inner.models.read());
        owner
            .inner
            .allowlist_excludes_all
            .store(excludes_all, Ordering::Relaxed);
        owner.reselect_current_model_if_missing(&config);
        drop(catalog_write);
        tracing::info!("OpenAI Codex model catalog cleared after auth identity change");
        owner.notify_models_updated();
        true
    }

    pub(super) async fn refresh_if_new_etag(
        &self,
        owner: &ModelsManager,
        etag: String,
        credential_binding: Option<xai_grok_sampling_types::CredentialBinding>,
    ) {
        tracing::info!(etag = %etag, "OpenAI Codex models etag changed, refreshing");
        let outcome = self
            .refresh(owner, credential_binding.map(|binding| (etag, binding)))
            .await;
        if matches!(
            outcome,
            CodexCatalogRefreshOutcome::TransientFailure
                | CodexCatalogRefreshOutcome::CachedFallback
        ) {
            self.set_needs_revalidation(true);
            self.start_recovery(owner);
        }
    }

    /// Reconcile a Codex-only auth file change, refresh the new account's
    /// entitlement catalog, and retry transient discovery failures without
    /// disturbing xAI authentication.
    pub(super) async fn on_auth_changed(&self, owner: &ModelsManager) -> bool {
        // A same-account record rotation may carry a new plan/workspace
        // entitlement even though its stable identity is unchanged.
        self.set_needs_revalidation(true);
        let outcome = self.refresh(owner, None).await;
        let needs_revalidation =
            outcome != CodexCatalogRefreshOutcome::Updated && self.has_catalog();
        self.set_needs_revalidation(needs_revalidation);
        if matches!(
            outcome,
            CodexCatalogRefreshOutcome::TransientFailure
                | CodexCatalogRefreshOutcome::CachedFallback
        ) {
            self.start_recovery(owner);
        }
        outcome.updated()
    }

    /// Start provider-local startup recovery when credentials exist but the
    /// initial Codex prefetch did not produce an account-bound catalog.
    pub(super) fn start_recovery(&self, owner: &ModelsManager) {
        let Some(identity) = current_identity() else {
            self.reconcile_identity(owner, None);
            return;
        };
        // Reconcile the installed account even when remote discovery is
        // disabled. Disabling network access must never leave another
        // account's entitled models visible.
        self.reconcile_identity(owner, Some(&identity));
        if !self.remote_fetch_enabled.load(Ordering::Acquire) {
            return;
        }
        if self.has_catalog()
            && self.identity.read().as_ref() == Some(&identity)
            && !self.needs_revalidation()
        {
            return;
        }
        self.spawn_retry(owner, identity);
    }

    fn spawn_retry(&self, owner: &ModelsManager, expected_identity: CodexCatalogIdentity) {
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            // Constructors and config reloads are also used by synchronous
            // embedders. A missing runtime must not panic the caller; the next
            // authenticated runtime hook or ETag notification will retry.
            tracing::debug!("OpenAI Codex model catalog retry deferred: no Tokio runtime");
            return;
        };
        if self
            .retry_in_flight
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            tracing::debug!("OpenAI Codex model catalog retry already in flight");
            return;
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum RetryCompletion {
            Updated,
            IdentityChanged,
            Stopped,
        }

        let manager = owner.clone();
        runtime.spawn(async move {
            let backoff = crate::tools::retry::BackoffConfig::new(5, 5_000, 60_000);
            let result = crate::tools::retry::execute_with_backoff(
                &backoff,
                || {
                    let manager = manager.clone();
                    let expected_identity = expected_identity.clone();
                    async move {
                        let controller = &manager.inner.codex_catalog;
                        if current_identity().as_ref() != Some(&expected_identity) {
                            return Ok(RetryCompletion::IdentityChanged);
                        }
                        if controller.has_catalog()
                            && controller.identity.read().as_ref() == Some(&expected_identity)
                            && !controller.needs_revalidation()
                        {
                            return Ok(RetryCompletion::Updated);
                        }

                        match controller.refresh(&manager, None).await {
                            CodexCatalogRefreshOutcome::Updated => Ok(RetryCompletion::Updated),
                            CodexCatalogRefreshOutcome::CachedFallback => {
                                Err("cached OpenAI Codex model catalog fallback")
                            }
                            CodexCatalogRefreshOutcome::TransientFailure => {
                                Err("transient OpenAI Codex model catalog failure")
                            }
                            CodexCatalogRefreshOutcome::NotAuthenticated => {
                                Ok(RetryCompletion::IdentityChanged)
                            }
                            CodexCatalogRefreshOutcome::TerminalFailure => {
                                if current_identity().as_ref() != Some(&expected_identity) {
                                    Ok(RetryCompletion::IdentityChanged)
                                } else {
                                    Ok(RetryCompletion::Stopped)
                                }
                            }
                            CodexCatalogRefreshOutcome::Disabled => Ok(RetryCompletion::Stopped),
                        }
                    }
                },
                |attempt, max_retries, delay| async move {
                    xai_grok_telemetry::unified_log::warn(
                        "OpenAI Codex model catalog: retry scheduled",
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

            let controller = &manager.inner.codex_catalog;
            controller.retry_in_flight.store(false, Ordering::Release);
            match result {
                Ok(RetryCompletion::Updated) => {
                    xai_grok_telemetry::unified_log::info(
                        "OpenAI Codex model catalog: retry succeeded",
                        None,
                        Some(serde_json::json!({
                            "model_count": controller.models.read().len(),
                        })),
                    );
                }
                Ok(RetryCompletion::IdentityChanged) => {
                    tracing::debug!("OpenAI Codex model catalog retry stopped after auth change");
                    // A new account may have arrived while its immediate
                    // refresh was blocked by this retry guard. Re-evaluate it
                    // only after releasing the guard.
                    controller.start_recovery(&manager);
                }
                Ok(RetryCompletion::Stopped) => {
                    tracing::debug!("OpenAI Codex model catalog retry stopped");
                }
                Err(error) => {
                    xai_grok_telemetry::unified_log::warn(
                        "OpenAI Codex model catalog: all retries exhausted",
                        None,
                        Some(serde_json::json!({ "error": error })),
                    );
                }
            }
        });
    }

    /// Fetch the current ChatGPT account's entitled Codex models without
    /// sharing any state with xAI model discovery. The async OAuth/catalog
    /// client runs on a short-lived thread so this startup seam remains safe
    /// when called from an existing Tokio runtime.
    pub(super) fn prefetch_blocking() -> Option<CodexPrefetchResult> {
        if !crate::util::config::resolve_remote_fetch_enabled() {
            return None;
        }
        let grok_home = crate::util::grok_home::grok_home();
        let worker = std::thread::Builder::new()
            .name("openai-codex-models".to_owned())
            .spawn(move || {
                let manager = crate::auth::codex::CodexAuthManager::new(&grok_home).ok()?;
                manager.current()?;
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .ok()?;
                runtime.block_on(async move {
                    let config =
                        crate::remote::CodexCatalogClientConfig::new(grok_home.join("cache"))
                            .ok()?;
                    let client = crate::remote::CodexCatalogClient::new(config).ok()?;
                    let (catalog, etag, authoritative, request_binding) = match client
                        .fetch(&manager)
                        .await
                    {
                        Ok(fetched) => (
                            fetched.catalog,
                            fetched.etag,
                            true,
                            fetched.credential_binding,
                        ),
                        Err(error) if catalog_error_is_transient(&error) => {
                            let cached = client.load_cached(&manager).await.ok()?;
                            (
                                cached.catalog,
                                cached.etag,
                                false,
                                cached.credential_binding,
                            )
                        }
                        Err(error) => {
                            tracing::warn!(error = %error, "OpenAI Codex model discovery failed");
                            return None;
                        }
                    };
                    // Serialize the final identity check with login/logout and
                    // keep the lease alive through installation in `from_config`.
                    let (locked_credentials, identity_lease) =
                        manager.lock_catalog_binding(&request_binding).await.ok()?;
                    if !crate::util::config::resolve_remote_fetch_enabled() {
                        return None;
                    }
                    let identity = CodexCatalogIdentity::from_credentials(&locked_credentials);
                    Some(CodexPrefetchResult {
                        models: Self::map_catalog(catalog),
                        etag,
                        identity,
                        authoritative,
                        _identity_lease: identity_lease,
                    })
                })
            })
            .ok()?;
        worker.join().ok().flatten()
    }

    /// Map provider-authoritative wire metadata into Grok Build's existing
    /// model-entry representation without changing the agent/session runtime.
    pub(super) fn map_catalog(
        catalog: crate::remote::CodexModelCatalog,
    ) -> IndexMap<String, ModelEntry> {
        catalog
            .models
            .into_iter()
            .filter_map(|model| {
                let context_window = model
                    .context_window
                    .or(model.max_context_window)
                    .and_then(|value| u64::try_from(value).ok())
                    .and_then(std::num::NonZeroU64::new)
                    .unwrap_or_else(|| {
                        std::num::NonZeroU64::new(crate::remote::DEFAULT_CONTEXT_WINDOW)
                            .expect("default context window is non-zero")
                    });
                let default_effort = model
                    .default_reasoning_level
                    .as_deref()
                    .and_then(|value| value.parse::<ReasoningEffort>().ok());
                // Grok's session gate is percentage-based while the Codex
                // catalog publishes an absolute auto-compact token limit.
                let effective_context_percent =
                    u8::try_from(model.effective_context_window_percent)
                        .ok()
                        .filter(|value| (1..=100).contains(value));
                let auto_compact_threshold_percent = model
                    .auto_compact_token_limit
                    .and_then(|value| u64::try_from(value).ok())
                    .filter(|value| *value > 0)
                    .map(|limit| {
                        let percent = limit
                            .saturating_mul(100)
                            .checked_div(context_window.get())
                            .unwrap_or(1)
                            .clamp(1, 100) as u8;
                        effective_context_percent.map_or(percent, |cap| percent.min(cap))
                    });
                let reasoning_efforts: Vec<ReasoningEffortOption> = model
                    .supported_reasoning_levels
                    .iter()
                    .filter_map(|preset| {
                        let value = preset.effort.parse::<ReasoningEffort>().ok()?;
                        Some(ReasoningEffortOption {
                            id: preset.effort.clone(),
                            value,
                            label: preset.effort.clone(),
                            description: (!preset.description.is_empty())
                                .then(|| preset.description.clone()),
                            default: Some(value) == default_effort,
                        })
                    })
                    .collect();
                // `hide` models remain explicitly addressable (for `-m` and
                // persisted sessions) but stay out of the picker. `none` and
                // unknown visibility values fail closed and are not addressable.
                let (hidden, addressable) = match model.visibility.as_deref() {
                    Some("list") => (false, true),
                    Some("hide") => (true, true),
                    _ => (true, false),
                };
                let id = model.id.clone();
                let display_name = if model.display_name.trim().is_empty() {
                    model.slug.clone()
                } else {
                    model.display_name.clone()
                };
                let mut extra_headers = IndexMap::new();
                if model.use_responses_lite {
                    extra_headers.insert(
                        xai_grok_sampling_types::OPENAI_CODEX_RESPONSES_LITE_HEADER.to_owned(),
                        "true".to_owned(),
                    );
                }
                let multi_agent_version = model
                    .multi_agent_version
                    .as_ref()
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_ascii_lowercase);
                let supports_image_input = model.accepts_images();
                Some((
                    id.clone(),
                    ModelEntry {
                        info: config::ModelInfo {
                            id: Some(id),
                            provider: ProviderId::OpenAiCodex,
                            model: model.slug,
                            base_url: xai_grok_sampling_types::OPENAI_CODEX_BASE_URL.to_owned(),
                            name: Some(display_name),
                            description: model.description,
                            max_completion_tokens: None,
                            temperature: None,
                            top_p: None,
                            api_backend: xai_grok_sampling_types::ApiBackend::Responses,
                            auth_scheme: xai_grok_sampler::AuthScheme::Bearer,
                            extra_headers,
                            context_window,
                            comp_hash: model.comp_hash,
                            auto_compact_threshold_percent,
                            system_prompt_label: None,
                            use_concise: false,
                            // Preserve Grok Build's agent loop and system
                            // prompt; Codex is a provider, not an agent profile.
                            agent_type: config::default_agent_type(),
                            inference_idle_timeout_secs: None,
                            max_retries: None,
                            hidden,
                            user_selectable: addressable,
                            supported_in_api: model.supported_in_api,
                            reasoning_effort: default_effort,
                            supports_reasoning_effort: !reasoning_efforts.is_empty(),
                            supports_reasoning_summary_parameter: model
                                .supports_reasoning_summary_parameter,
                            default_reasoning_summary: model.default_reasoning_summary,
                            reasoning_efforts,
                            service_tiers: model
                                .service_tiers
                                .into_iter()
                                .map(|tier| config::ModelServiceTier {
                                    id: tier.id,
                                    name: tier.name,
                                    description: tier.description,
                                })
                                .collect(),
                            default_service_tier: model.default_service_tier,
                            service_tier: None,
                            multi_agent_version,
                            supports_image_input,
                            supports_backend_search: false,
                            compactions_remaining: None,
                            compaction_at_tokens: None,
                            show_model_fingerprint: false,
                            stream_tool_calls: None,
                            laziness_detector: config::LazinessDetectorPerModelConfig::default(),
                        },
                        api_key: None,
                        env_key: None,
                        auth_provider: None,
                        api_base_url: None,
                    },
                ))
            })
            .collect()
    }
}

pub(super) fn current_identity() -> Option<CodexCatalogIdentity> {
    let manager =
        crate::auth::codex::CodexAuthManager::new(&crate::util::grok_home::grok_home()).ok()?;
    manager
        .current()
        .as_ref()
        .map(CodexCatalogIdentity::from_credentials)
}

fn catalog_error_is_transient(error: &crate::remote::CodexCatalogError) -> bool {
    matches!(
        error,
        crate::remote::CodexCatalogError::AuthenticationTransient
            | crate::remote::CodexCatalogError::Timeout
            | crate::remote::CodexCatalogError::Transport
            | crate::remote::CodexCatalogError::HttpStatus(408 | 429 | 500..=599)
    )
}

fn catalog_lock_error_is_transient(error: &crate::auth::codex::CodexAuthError) -> bool {
    matches!(
        error,
        crate::auth::codex::CodexAuthError::AccountChanged
            | crate::auth::codex::CodexAuthError::LockTimeout
    ) || matches!(
        error,
        crate::auth::codex::CodexAuthError::Storage(source)
            if matches!(
                source.kind(),
                std::io::ErrorKind::Interrupted
                    | std::io::ErrorKind::WouldBlock
                    | std::io::ErrorKind::TimedOut
            )
    )
}

fn catalog_auth_failure_outcome(
    error: &crate::auth::codex::CodexAuthError,
    credentials_still_exist: bool,
) -> CodexCatalogRefreshOutcome {
    if catalog_lock_error_is_transient(error) {
        CodexCatalogRefreshOutcome::TransientFailure
    } else if credentials_still_exist {
        CodexCatalogRefreshOutcome::TerminalFailure
    } else {
        CodexCatalogRefreshOutcome::NotAuthenticated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_debug_redacts_all_scope_and_fedramp_state() {
        let identity = CodexCatalogIdentity::for_test(
            "sentinel-record-id",
            "sentinel-account-id",
            Some("sentinel-user-id"),
            true,
        );
        let rendered = format!("{identity:?}");
        assert!(
            rendered == "CodexCatalogIdentity { .. }",
            "identity debug must be strictly redacted"
        );
        assert!(!rendered.contains("sentinel"));
        assert!(!rendered.contains("true"));
    }

    #[test]
    fn stale_generation_result_cannot_publish_after_config_transition() {
        let manager = ModelsManager::default();
        let controller = &manager.inner.codex_catalog;
        let stale_generation = controller.generation();
        controller.advance_config_generation(true);

        let applied = controller.try_apply_catalog(
            &manager,
            crate::remote::CodexModelCatalog {
                models: vec![crate::remote::CodexCatalogModel {
                    id: "openai-codex/stale".to_owned(),
                    slug: "stale".to_owned(),
                    visibility: Some("list".to_owned()),
                    ..Default::default()
                }],
            },
            Some("stale-etag".to_owned()),
            CodexCatalogIdentity::for_test(
                "sentinel-record-id",
                "sentinel-account-id",
                None,
                false,
            ),
            true,
            stale_generation,
        );

        assert!(!applied);
        assert!(!controller.has_catalog());
        assert!(controller.etag().is_none());
        assert!(controller.models_snapshot().is_empty());
        assert!(!manager.models().contains_key("openai-codex/stale"));
    }
}
