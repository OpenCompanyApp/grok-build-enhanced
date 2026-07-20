//! Model auth providers (`[auth_provider.<name>]`).
//!
//! A model opts in with `auth_provider = "<name>"`; the named table declares a
//! command that prints a fresh bearer token, which this module mints, caches,
//! and rotates for that model's requests.
//!
//! The minted token stays in memory only ([`AUTH_PROVIDER_SLOTS`] and chat
//! state, never `auth.json`); the command is a credential helper that owns its
//! own durable storage and OAuth2 refresh. See "Where model auth providers fit
//! (and don't)"
//! in `docs/internal/AUTH.md`.
//!
//! This is distinct from the `AuthCredentialProvider` HTTP consumers in
//! [`crate::auth::credential_provider`].

use super::token_output::{expiry_after_seconds, parse_provider_token_output};

/// One named `[auth_provider.<name>]` table, honored only from the trusted
/// config layers (`parse_auth_providers`). A new field here needs a
/// `parse_auth_providers` warning decision.
#[derive(Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct AuthProviderConfig {
    /// Command that prints a bearer token on stdout, bare or as JSON
    /// `{access_token, expires_in}`. Without `args` it runs via `sh -c`.
    pub command: String,
    /// Arguments for `command`. When present (even empty), the command runs
    /// directly with no shell; `command` is a program name on `PATH`, or a path.
    pub args: Option<Vec<String>>,
    /// Fallback token lifetime in seconds, used when the command's output
    /// carries no `expires_in`. Takes precedence over a JWT `exp` claim.
    pub token_ttl_secs: Option<u64>,
    /// Maximum seconds to wait for the command (default 30, clamped to 1..=600).
    /// A turn waits up to this long on a mint, so keep helpers fast and
    /// non-interactive.
    pub timeout_secs: Option<u64>,
}

impl std::fmt::Debug for AuthProviderConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthProviderConfig")
            .field("command_configured", &self.is_usable())
            .field("direct_exec", &self.args.is_some())
            .field("argument_count", &self.args.as_ref().map(Vec::len))
            .field("token_ttl_secs", &self.token_ttl_secs)
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

impl AuthProviderConfig {
    pub(crate) fn is_usable(&self) -> bool {
        !self.command.trim().is_empty()
    }
}

/// A model's reference to a named auth provider, built by `resolve_model_list`.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "AuthProviderRefData", into = "AuthProviderRefData")]
pub struct AuthProviderRef {
    pub(crate) name: String,
    pub(crate) config: AuthProviderConfig,
    slot: ProviderSlot,
    /// Exact custom-model endpoint this provider may authenticate. The route is
    /// attached only while resolving trusted config and is never serialized.
    route_base_url: Option<String>,
    /// `true` once the trusted table is attached. A ref revived from bytes is
    /// `false` and never mints or reads until [`AuthProviderRef::attach_trusted_config`]
    /// joins the shared slot for its name.
    resolved: bool,
}

/// Serialized form: the name only, so persisted bytes never carry a command.
#[derive(serde::Serialize, serde::Deserialize)]
struct AuthProviderRefData {
    name: String,
}

impl From<AuthProviderRefData> for AuthProviderRef {
    fn from(data: AuthProviderRefData) -> Self {
        AuthProviderRef::unresolved(data.name)
    }
}

impl From<AuthProviderRef> for AuthProviderRefData {
    fn from(provider: AuthProviderRef) -> Self {
        Self {
            name: provider.name,
        }
    }
}

impl AuthProviderRef {
    #[cfg(test)]
    const TEST_ROUTE: &'static str = "https://auth-provider.test/v1";

    /// Production uses `unresolved` + `attach_trusted_config`.
    #[cfg(test)]
    pub(crate) fn new(name: String, config: AuthProviderConfig) -> Self {
        Self::new_for_test_route(name, config, Self::TEST_ROUTE)
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_route(
        name: String,
        config: AuthProviderConfig,
        route_base_url: &str,
    ) -> Self {
        let slot = provider_slot(&name, route_base_url);
        Self {
            name,
            config,
            slot,
            route_base_url: Some(route_base_url.to_owned()),
            resolved: true,
        }
    }

    /// The in-memory form of a ref revived from bytes;
    /// [`AuthProviderRef::attach_trusted_config`] resolves it.
    pub(crate) fn unresolved(name: String) -> Self {
        Self {
            name,
            config: AuthProviderConfig::default(),
            slot: ProviderSlot::default(),
            route_base_url: None,
            resolved: false,
        }
    }

    /// Re-attach this name to a trusted provider definition and one exact
    /// custom-model endpoint. Either missing input leaves the ref unusable, so
    /// serialized session state and non-custom models can never execute a helper.
    pub(crate) fn attach_trusted_config(
        &mut self,
        config: Option<&AuthProviderConfig>,
        route_base_url: Option<&str>,
    ) {
        self.config = config.cloned().unwrap_or_default();
        self.route_base_url = route_base_url.map(str::to_owned);
        self.slot = match (config, route_base_url) {
            (Some(_), Some(route)) => provider_slot(&self.name, route),
            _ => ProviderSlot::default(),
        };
        self.resolved = true;
    }

    /// Whether this trusted ref is authorized for the active custom route.
    pub(crate) fn is_bound_to_custom_route(&self, route_base_url: &str) -> bool {
        self.resolved && self.route_base_url.as_deref() == Some(route_base_url)
    }
}

/// Ignores the slot; a deserialized ref compares unequal until resolution
/// re-attaches its config.
impl PartialEq for AuthProviderRef {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.config == other.config
    }
}

impl Eq for AuthProviderRef {}

impl std::fmt::Debug for AuthProviderRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthProviderRef")
            .field("name", &self.name)
            .field("config", &self.config)
            .field("resolved", &self.resolved)
            .finish_non_exhaustive()
    }
}

struct MintedProviderToken {
    token: String,
    /// Handed back to the command on the next run; never sent on the wire.
    refresh_token: Option<String>,
    /// Drives the 401 fresh-mint guard.
    minted_at: std::time::Instant,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// The table version that minted the token; a different version reads as
    /// stale (see [`token_identity`]), so edits re-mint.
    minted_with: AuthProviderConfig,
}

/// The async lock is held across the command run, single-flighting mints for
/// one provider name on one exact custom-model route (shared across sessions).
/// This dedupes concurrent successes without allowing a same-named provider on
/// another endpoint to reuse the credential. A persistently failing helper is
/// retried per waiter, each bounded by the timeout clamp.
type ProviderSlot = std::sync::Arc<tokio::sync::Mutex<Option<MintedProviderToken>>>;

#[derive(Clone, PartialEq, Eq, Hash)]
struct ProviderSlotKey {
    name: String,
    route_base_url: String,
}

/// Shared token slots, one per resolved `(provider name, exact route)` pair.
/// Bounded by trusted model/provider configuration (only
/// `attach_trusted_config` and test constructors insert), so no eviction.
static AUTH_PROVIDER_SLOTS: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<ProviderSlotKey, ProviderSlot>>,
> = std::sync::OnceLock::new();

fn provider_slot(name: &str, route_base_url: &str) -> ProviderSlot {
    let map = AUTH_PROVIDER_SLOTS.get_or_init(Default::default);
    let mut map = map.lock().unwrap_or_else(|e| e.into_inner());
    map.entry(ProviderSlotKey {
        name: name.to_owned(),
        route_base_url: route_base_url.to_owned(),
    })
    .or_default()
    .clone()
}

/// Pre-refresh margin: re-mint when the token expires within this window.
pub(crate) const PROVIDER_TOKEN_EXPIRY_SKEW_SECS: u64 = 60;
const PROVIDER_TOKEN_EXPIRY_SKEW: chrono::Duration =
    chrono::Duration::seconds(PROVIDER_TOKEN_EXPIRY_SKEW_SECS as i64);
/// 401 fresh-mint guard: a token minted this recently is never re-minted on
/// rejection. Same idea as the guard in `unauthorized_recovery`, with a shorter
/// window because a provider mint is local and cheap.
const PROVIDER_TOKEN_FRESH_MINT_GUARD: std::time::Duration = std::time::Duration::from_secs(30);
const DEFAULT_PROVIDER_TIMEOUT_SECS: u64 = 30;
/// The effective mint timeout is clamped to `[1, this]`. A configured value
/// outside the range is honored up to the bound and draws a parse warning,
/// since a turn waits on the mint.
pub(crate) const PROVIDER_TIMEOUT_CEILING_SECS: u64 = 600;
/// Caps on the helper's captured output so a runaway command can't exhaust
/// memory before the timeout fires. A bearer (even a large JWT) is far under
/// the stdout cap; stderr is drained but never surfaced or logged.
const PROVIDER_STDOUT_CAP_BYTES: u64 = 1 << 20; // 1 MiB
const PROVIDER_STDERR_CAP_BYTES: u64 = 64 << 10; // 64 KiB

/// The table fields that shape the minted token; a cached token minted under a
/// different set reads as stale, so a config edit re-mints. Destructured so a
/// new `AuthProviderConfig` field is a compile error until it is classified as
/// token-shaping (add it here) or an execution knob like `timeout_secs`
/// (editing it never invalidates).
fn token_identity(config: &AuthProviderConfig) -> (&str, Option<&[String]>, Option<u64>) {
    let AuthProviderConfig {
        command,
        args,
        token_ttl_secs,
        timeout_secs: _,
    } = config;
    (command, args.as_deref(), *token_ttl_secs)
}

fn minted_token_is_stale(minted: &MintedProviderToken, config: &AuthProviderConfig) -> bool {
    token_identity(&minted.minted_with) != token_identity(config)
        || minted
            .expires_at
            .is_some_and(|at| chrono::Utc::now() + PROVIDER_TOKEN_EXPIRY_SKEW >= at)
}

/// Log the missing-command warning once per provider, then at debug, so a
/// misconfigured model doesn't warn on every turn.
fn warn_empty_command(name: &str) {
    static WARNED: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> =
        std::sync::OnceLock::new();
    let first = WARNED
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(name.to_owned());
    const MSG: &str = "auth provider has no usable command: the [auth_provider.*] table is \
                       missing from the trusted config layers, or its `command` is empty";
    if first {
        tracing::warn!(provider = %name, "{MSG}");
    } else {
        tracing::debug!(provider = %name, "{MSG}");
    }
}

/// Read up to `keep` bytes into `buf`, then drain and discard any remainder so
/// the child never blocks on a full pipe. Memory stays bounded by `keep`.
async fn read_capped<R>(reader: R, keep: u64, buf: &mut Vec<u8>) -> std::io::Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
{
    use tokio::io::AsyncReadExt;
    let mut limited = reader.take(keep);
    limited.read_to_end(buf).await?;
    tokio::io::copy(&mut limited.into_inner(), &mut tokio::io::sink()).await?;
    Ok(())
}

/// Remove every first-party credential from the helper's environment. BYOK
/// isolates these keys on the wire, so the helper (the agent puts them in its
/// own env at startup) must not inherit them.
fn scrub_first_party_credentials(cmd: &mut tokio::process::Command) {
    for var in crate::agent::config::FIRST_PARTY_CREDENTIAL_ENV_VARS {
        cmd.env_remove(var);
    }
}

const PROVIDER_HANDBACK_ENV_VARS: &[&str] = &[
    "GROK_AUTH_EXPIRED",
    "GROK_AUTH_PROVIDER_ACCESS_TOKEN",
    "GROK_AUTH_PROVIDER_REFRESH_TOKEN",
    "GROK_AUTH_PROVIDER_EXPIRES_AT",
];

/// Start from an empty helper-handback namespace on every invocation, then add
/// only values belonging to this exact provider slot. This prevents stale
/// ambient variables or a previous command setup from crossing into a first
/// mint or a different `(provider, route)` slot.
fn configure_provider_handback_env(
    cmd: &mut tokio::process::Command,
    mark_expired: bool,
    previous: Option<&MintedProviderToken>,
) {
    for variable in PROVIDER_HANDBACK_ENV_VARS {
        cmd.env_remove(variable);
    }
    if mark_expired {
        cmd.env("GROK_AUTH_EXPIRED", "1");
    }
    if let Some(previous) = previous {
        cmd.env("GROK_AUTH_PROVIDER_ACCESS_TOKEN", &previous.token);
        if let Some(refresh) = &previous.refresh_token {
            cmd.env("GROK_AUTH_PROVIDER_REFRESH_TOKEN", refresh);
        }
        if let Some(expires_at) = previous.expires_at {
            cmd.env("GROK_AUTH_PROVIDER_EXPIRES_AT", expires_at.to_rfc3339());
        }
    }
}

/// Spawn `cmd`, capture stdout/stderr with a byte cap (reading both
/// concurrently so a full pipe on one can't deadlock the other; a runaway helper
/// is drained to a sink past the cap so it can't wedge the wait), and bound the
/// whole run by `timeout`. Exceeding the stdout cap is an error.
///
/// On timeout the child's entire process group is killed. The helper is a group
/// leader (`detach_command`'s `setsid`), so a compound `sh -c` helper's
/// grandchildren -- and the `GROK_AUTH_PROVIDER_*` credentials in their env --
/// do not outlive the reported timeout; `kill_on_drop` alone would reap only the
/// direct child.
async fn run_capped(
    cmd: &mut tokio::process::Command,
    timeout: std::time::Duration,
) -> anyhow::Result<std::process::Output> {
    let mut child = cmd
        .spawn()
        .map_err(|e| anyhow::anyhow!("command failed to start: {e}"))?;
    // Enroll the child's process group so the timeout path can tear down the
    // whole tree. Best-effort: if enrollment fails, `kill_on_drop` still reaps
    // the direct child.
    let mut group = xai_grok_tools::util::ProcessGroup::new()
        .map_err(|e| anyhow::anyhow!("process group setup failed: {e}"))?;
    if let Err(e) = group.attach(&child) {
        tracing::debug!(error = %e, "auth provider: could not enroll helper process group");
    }
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let mut out_buf = Vec::new();
    let mut err_buf = Vec::new();

    // One extra stdout byte so an over-cap write is detectable, not truncated.
    // Stderr is drained only to keep the child from blocking and is never
    // surfaced; only stdout governs the mint.
    let capture = async {
        let (out_res, err_res) = tokio::join!(
            read_capped(stdout, PROVIDER_STDOUT_CAP_BYTES + 1, &mut out_buf),
            read_capped(stderr, PROVIDER_STDERR_CAP_BYTES, &mut err_buf),
        );
        if let Err(e) = err_res {
            tracing::debug!(error = %e, "auth provider: stderr capture failed (advisory)");
        }
        out_res.map_err(|e| anyhow::anyhow!("reading command stdout: {e}"))?;
        child
            .wait()
            .await
            .map_err(|e| anyhow::anyhow!("waiting on command: {e}"))
    };

    let status = match tokio::time::timeout(timeout, capture).await {
        Ok(res) => res?,
        Err(_elapsed) => {
            let _ = group.kill();
            anyhow::bail!("command timed out after {}s", timeout.as_secs());
        }
    };
    if out_buf.len() as u64 > PROVIDER_STDOUT_CAP_BYTES {
        anyhow::bail!("command wrote more than {PROVIDER_STDOUT_CAP_BYTES} bytes to stdout");
    }
    Ok(std::process::Output {
        status,
        stdout: out_buf,
        stderr: err_buf,
    })
}

async fn mint_provider_token(
    provider: &AuthProviderRef,
    mark_expired: bool,
    previous: Option<&MintedProviderToken>,
) -> anyhow::Result<MintedProviderToken> {
    use std::process::Stdio;

    let name = &provider.name;
    let config = &provider.config;
    // Clamp to [1, ceiling]: the slot lock is held across the run, so an
    // unbounded timeout would let one hung helper stall every turn sharing this
    // provider name. The ceiling is a hard bound, not just a parse warning.
    let timeout_secs = config
        .timeout_secs
        .unwrap_or(DEFAULT_PROVIDER_TIMEOUT_SECS)
        .clamp(1, PROVIDER_TIMEOUT_CEILING_SECS);
    tracing::info!(
        provider = %name,
        mark_expired,
        timeout_secs,
        "auth provider: running helper command"
    );

    let mut cmd = match config.args {
        Some(ref args) => {
            // Direct exec: the program name is a PATH lookup, so trim stray
            // whitespace that would otherwise fail to resolve.
            let mut cmd = tokio::process::Command::new(config.command.trim());
            cmd.args(args);
            cmd
        }
        None => {
            let mut cmd = tokio::process::Command::new("sh");
            cmd.args(["-c", &config.command]);
            cmd
        }
    };
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        // Drain stderr privately; inheriting corrupts the TUI and surfacing it
        // could expose helper diagnostics or credentials.
        .stderr(Stdio::piped())
        // Reaps the direct child if the future is dropped; `run_capped`
        // additionally kills the whole process group on timeout.
        .kill_on_drop(true);
    xai_grok_tools::util::detach_command(&mut cmd);
    cmd.envs(xai_grok_tools::util::pager_env());
    // Apply handback after the inherited pager environment so stale ambient
    // values cannot be reintroduced. Then scrub first-party credentials last.
    configure_provider_handback_env(&mut cmd, mark_expired, previous);
    scrub_first_party_credentials(&mut cmd);

    let output = run_capped(&mut cmd, std::time::Duration::from_secs(timeout_secs)).await?;

    // Never surface helper stdout or stderr: either stream may contain a
    // credential or private helper diagnostic. The shared parser's errors
    // describe only the rejected shape/status.
    let parsed = parse_provider_token_output(&output)
        .map_err(|error| anyhow::anyhow!("provider helper output rejected: {error}"))?;
    let expires_at = parsed
        .expires_at
        .or_else(|| config.token_ttl_secs.and_then(expiry_after_seconds))
        .or_else(|| crate::auth::parse_jwt_expiration(&parsed.access_token));
    tracing::info!(
        provider = %name,
        mark_expired,
        expires_at = ?expires_at,
        "auth provider minted token"
    );
    Ok(MintedProviderToken {
        token: parsed.access_token,
        refresh_token: parsed.refresh_token,
        minted_at: std::time::Instant::now(),
        expires_at,
        minted_with: config.clone(),
    })
}

#[derive(Debug, PartialEq, Eq)]
#[must_use = "a rotated token must be written to chat-state, or the wire keeps the stale key"]
pub(crate) enum ProviderRefreshOutcome {
    /// `current_key` is already the fresh cached token; nothing to write.
    Unchanged,
    /// A token that should replace `current_key` on the wire.
    Rotated(String),
    /// The provider is unusable (unresolved or removed); already warned.
    Unusable,
    /// The mint ran and failed (logged).
    MintFailed,
}

impl ProviderRefreshOutcome {
    pub(crate) fn rotated(self) -> Option<String> {
        match self {
            Self::Rotated(token) => Some(token),
            Self::Unchanged | Self::Unusable | Self::MintFailed => None,
        }
    }
}

impl AuthProviderRef {
    /// The slot, locked for a mutating operation. A removed provider drops
    /// its cached token and yields `None`, failing closed. An unresolved ref
    /// (revived from bytes) fails closed without touching the shared slot.
    async fn locked_slot(
        &self,
    ) -> Option<tokio::sync::OwnedMutexGuard<Option<MintedProviderToken>>> {
        if !self.resolved {
            return None;
        }
        let mut slot = self.slot.clone().lock_owned().await;
        if !self.config.is_usable() {
            if slot.take().is_some() {
                tracing::warn!(
                    provider = %self.name,
                    "auth provider removed from config: dropping its cached token"
                );
            }
            warn_empty_command(&self.name);
            return None;
        }
        Some(slot)
    }

    /// Cache-only read for sync resolution: never runs the command, blocks, or
    /// mutates. `None` for an unresolved ref, a cold or stale cache, or a mint
    /// in progress; minting happens pre-turn via [`AuthProviderRef::ensure_fresh_token`].
    pub(crate) fn cached_token(&self) -> Option<String> {
        if !self.resolved {
            return None;
        }
        if !self.config.is_usable() {
            warn_empty_command(&self.name);
            return None;
        }
        // A mint in progress holds the lock; treat it as a miss rather than
        // block the sync path.
        let Ok(guard) = self.slot.try_lock() else {
            tracing::debug!(provider = %self.name, "cache read skipped: mint in progress");
            return None;
        };
        guard
            .as_ref()
            .filter(|m| !minted_token_is_stale(m, &self.config))
            .map(|m| m.token.clone())
    }

    /// The token that should replace `current_key` on the wire: serves the
    /// fresh cached token when chat-state lags behind a rotation, mints when
    /// the cache is cold or stale. Mints or rotates a bearer; unrelated to an
    /// OAuth refresh token.
    pub(crate) async fn ensure_fresh_token(
        &self,
        current_key: Option<&str>,
    ) -> ProviderRefreshOutcome {
        let Some(mut slot) = self.locked_slot().await else {
            return ProviderRefreshOutcome::Unusable;
        };
        if let Some(ref minted) = *slot
            && !minted_token_is_stale(minted, &self.config)
        {
            return if current_key == Some(minted.token.as_str()) {
                ProviderRefreshOutcome::Unchanged
            } else {
                ProviderRefreshOutcome::Rotated(minted.token.clone())
            };
        }
        let mark_expired = slot.is_some();
        let minted = match mint_provider_token(self, mark_expired, slot.as_ref()).await {
            Ok(minted) => minted,
            Err(e) => {
                tracing::warn!(
                    provider = %self.name,
                    error = %e,
                    "auth provider pre-turn mint failed"
                );
                return ProviderRefreshOutcome::MintFailed;
            }
        };
        let token = minted.token.clone();
        *slot = Some(minted);
        ProviderRefreshOutcome::Rotated(token)
    }

    /// The replacement for a server-rejected `rejected_key` (chat-state's
    /// current key): a fresher cached token is adopted without a re-run,
    /// otherwise the command runs once. `None` for a token minted moments ago
    /// under the current table (the fresh-mint guard, which an edited table
    /// bypasses).
    pub(crate) async fn recover_rejected_token(&self, rejected_key: &str) -> Option<String> {
        let mut slot = self.locked_slot().await?;
        if let Some(ref minted) = *slot {
            if minted.token != rejected_key && !minted_token_is_stale(minted, &self.config) {
                return Some(minted.token.clone());
            }
            if minted.token == rejected_key
                && token_identity(&minted.minted_with) == token_identity(&self.config)
                && minted.minted_at.elapsed() < PROVIDER_TOKEN_FRESH_MINT_GUARD
            {
                tracing::warn!(
                    provider = %self.name,
                    "auth provider token rejected moments after mint: not \
                     re-running (fresh-mint guard); surfacing the 401"
                );
                return None;
            }
        }
        tracing::info!(provider = %self.name, "auth provider token rejected: re-minting");
        let minted = match mint_provider_token(self, true, slot.as_ref()).await {
            Ok(minted) => minted,
            Err(e) => {
                tracing::warn!(
                    provider = %self.name,
                    error = %e,
                    "auth provider 401 re-mint failed"
                );
                // The server rejected the cached token and the re-mint failed;
                // mark it stale so it is not re-served next turn (fail closed).
                // The entry stays so its refresh token still feeds the next
                // handback attempt.
                if let Some(minted) = slot.as_mut() {
                    minted.expires_at = Some(chrono::Utc::now());
                }
                return None;
            }
        };
        let token = minted.token.clone();
        *slot = Some(minted);
        Some(token)
    }
}

/// Backdate a provider's mint time past the fresh-mint guard.
#[cfg(test)]
pub(crate) fn test_backdate_provider_mint(name: &str, age: std::time::Duration) {
    let slot = provider_slot(name, AuthProviderRef::TEST_ROUTE);
    let mut slot = slot
        .try_lock()
        .expect("no mint in flight during test mutation");
    if let Some(ref mut minted) = *slot {
        minted.minted_at = std::time::Instant::now()
            .checked_sub(age)
            .expect("backdate before the process epoch");
    }
}

/// A counting provider that prints "tok-1", "tok-2", ... on successive runs.
#[cfg(test)]
pub(crate) fn test_counting_provider(name: &str, dir: &std::path::Path) -> AuthProviderRef {
    let counter = dir.join("count");
    AuthProviderRef::new(
        name.to_owned(),
        AuthProviderConfig {
            command: format!(
                "echo run >> {c}; printf 'tok-%s' \"$(wc -l < {c} | tr -d ' ')\"",
                c = counter.display()
            ),
            args: None,
            token_ttl_secs: Some(3600),
            timeout_secs: None,
        },
    )
}

#[cfg(test)]
fn test_expire_provider_token(name: &str) {
    let slot = provider_slot(name, AuthProviderRef::TEST_ROUTE);
    let mut slot = slot
        .try_lock()
        .expect("no mint in flight during test mutation");
    if let Some(ref mut minted) = *slot {
        minted.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
    }
}

#[cfg(test)]
#[path = "auth_provider_tests.rs"]
mod tests;
