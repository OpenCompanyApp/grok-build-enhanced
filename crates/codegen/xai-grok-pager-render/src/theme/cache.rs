//! In-memory theme cache + resolution.
//!
//! The pager reads the active `ThemeKind` on every render frame, so the
//! lookup must be cheaper than re-loading from `~/.grok/config.toml`.
//! [`current_kind`] returns the in-memory value, lazily seeding from the
//! shell's layered effective config on first call.
//!
//! Disk writes are NOT performed here — they live in
//! `xai_grok_shell::util::config::set_theme()` (and friends), invoked
//! via `Effect::PersistSetting` from the dispatcher. This module is a
//! pager-side in-memory cache + resolution layer only.

use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use arc_swap::ArcSwap;

use super::system_appearance;
use super::{ResolvedTheme, ThemeKind, ThemeRenderMode, ThemeSelection};

/// In-memory theme kind, encoded as a `u8` matching the
/// `ThemeKind` discriminants. Loaded from disk once at startup via
/// `load_from_disk()`, then kept in sync by `set()`.
static CURRENT: AtomicU8 = AtomicU8::new(ThemeKind::GrokNight as u8);
static LOADED: AtomicBool = AtomicBool::new(false);
static ACTIVE: LazyLock<ArcSwap<ResolvedTheme>> = LazyLock::new(|| {
    ArcSwap::from_pointee(ResolvedTheme::built_in(
        ThemeKind::GrokNight,
        ThemeSelection::BuiltIn(ThemeKind::GrokNight),
    ))
});
static ACTIVE_SELECTION: LazyLock<Mutex<ThemeSelection>> =
    LazyLock::new(|| Mutex::new(ThemeSelection::BuiltIn(ThemeKind::GrokNight)));
static REVISION: AtomicU64 = AtomicU64::new(1);
#[cfg(any(test, feature = "test-support"))]
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Whether auto-switching mode is active. Set when the config file
/// contains `theme = "auto"`. Checked by the event loop to decide
/// whether the `SystemAppearanceWatcher` should run.
///
/// Uses `AtomicBool` for thread-safe access from the watcher task.
static AUTO_MODE: AtomicBool = AtomicBool::new(false);

/// Whether the theme is locked to `Theme::terminal_default` for the whole
/// session (minimal mode — no theming).
static TERMINAL_NATIVE_LOCK: AtomicBool = AtomicBool::new(false);

/// Decode the u8 stored in `CURRENT` back to a `ThemeKind`. Falls
/// back to `GrokNight` if the byte is somehow out of range (which
/// can't happen via `set` — the discriminant is always a valid
/// variant — but defends against a future variant addition that
/// forgot to extend this match).
fn theme_kind_from_u8(byte: u8) -> ThemeKind {
    match byte {
        x if x == ThemeKind::GrokNight as u8 => ThemeKind::GrokNight,
        x if x == ThemeKind::GrokDay as u8 => ThemeKind::GrokDay,
        x if x == ThemeKind::TokyoNight as u8 => ThemeKind::TokyoNight,
        x if x == ThemeKind::RosePineMoon as u8 => ThemeKind::RosePineMoon,
        x if x == ThemeKind::OscuraMidnight as u8 => ThemeKind::OscuraMidnight,
        x if x == ThemeKind::TerminalNative as u8 => ThemeKind::TerminalNative,
        x if x == ThemeKind::WarpSync as u8 => ThemeKind::WarpSync,
        x if x == ThemeKind::WarpCustom as u8 => ThemeKind::WarpCustom,
        x if x == ThemeKind::Auto as u8 => ThemeKind::Auto,
        _ => ThemeKind::GrokNight,
    }
}

/// Cached auto-theme configuration (which themes map to dark/light).
///
/// Uses `Mutex<Option<_>>` rather than `OnceLock` so the cache can be
/// invalidated when the user changes mappings via the settings modal
/// or the `/theme auto` slash command.
static AUTO_THEME_CONFIG: Mutex<Option<AutoThemeConfig>> = Mutex::new(None);

/// Auto-theme config: which themes map to dark/light system appearance.
///
/// `dark_theme` and `light_theme` are the user-configured overrides read
/// from `[ui].auto_dark_theme` and `[ui].auto_light_theme` in `config.toml`.
/// When `None`, `to_theme_kind()` defaults to `GrokNight` / `GrokDay`.
#[derive(Debug, Clone, Default)]
pub struct AutoThemeConfig {
    pub dark_theme: Option<ThemeSelection>,
    pub light_theme: Option<ThemeSelection>,
}

/// Get the current theme kind.
///
/// On the first call, reads from `~/.grok/config.toml` (via the shell's
/// `load_effective_config`). After that, returns the in-memory value
/// (updated by [`set`]).
pub fn current_kind() -> ThemeKind {
    // Locked: return a constant nominal kind without seeding from disk.
    if terminal_native_locked() {
        return ThemeKind::GrokNight;
    }
    if !LOADED.load(Ordering::Acquire) {
        // Preserve the old lazy built-in seed path for callers that reach the
        // renderer before app startup has installed a fully resolved theme.
        if let Some(kind) = load_from_disk() {
            CURRENT.store(kind as u8, Ordering::Relaxed);
            if !kind.is_auto() {
                install_resolved(ResolvedTheme::built_in(kind, ThemeSelection::BuiltIn(kind)));
            }
        }
        LOADED.store(true, Ordering::Release);
    }
    theme_kind_from_u8(CURRENT.load(Ordering::Relaxed))
}

/// Snapshot the current fully-resolved theme.
pub fn current_resolved() -> Arc<ResolvedTheme> {
    ACTIVE.load_full()
}

/// Current persisted/user-facing selection (as opposed to its resolved family).
pub fn current_selection() -> ThemeSelection {
    ACTIVE_SELECTION
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// Process-local revision incremented whenever the resolved visual fingerprint changes.
pub fn current_revision() -> u64 {
    REVISION.load(Ordering::Relaxed)
}

/// Whether the active full-TUI theme delegates its canvas/palette to the terminal.
pub fn active_terminal_native() -> bool {
    terminal_native_locked() || current_resolved().render_mode == ThemeRenderMode::TerminalNative
}

/// Install a fully-resolved theme and synchronize all hot-path mirrors.
pub fn install_resolved(resolved: ResolvedTheme) -> bool {
    let previous = ACTIVE.load_full();
    let changed = previous.fingerprint != resolved.fingerprint;
    xai_grok_markdown::set_syntax_color_policy(resolved.syntax_policy);
    CURRENT.store(resolved.kind as u8, Ordering::Relaxed);
    *ACTIVE_SELECTION.lock().unwrap_or_else(|e| e.into_inner()) = resolved.selection.clone();
    ACTIVE.store(Arc::new(resolved));
    LOADED.store(true, Ordering::Release);
    if changed {
        REVISION.fetch_add(1, Ordering::Relaxed);
    }
    sync_color_caps();
    changed
}

/// Resolve a selection for preview/display without changing mode flags.
pub fn resolve_selection_for_display(
    selection: ThemeSelection,
    appearance: Option<system_appearance::SystemAppearance>,
) -> ResolvedTheme {
    match selection.clone() {
        ThemeSelection::Auto => resolve_auto_resolved(appearance),
        other => super::resolved::resolve_selection(other, appearance),
    }
}

/// Resolve and install a selection for the supplied appearance.
pub fn apply_selection(
    selection: ThemeSelection,
    appearance: Option<system_appearance::SystemAppearance>,
) -> bool {
    if terminal_native_locked() {
        *ACTIVE_SELECTION.lock().unwrap_or_else(|e| e.into_inner()) = selection;
        return false;
    }
    set_auto_mode(selection.is_auto());
    let resolved = resolve_selection_for_display(selection, appearance);
    let changed = install_resolved(resolved);
    super::apply_cursor_color();
    changed
}

/// Re-resolve the current selection after a watched external input changes.
pub fn refresh_current(appearance: Option<system_appearance::SystemAppearance>) -> bool {
    if terminal_native_locked() {
        return false;
    }
    let selection = current_selection();
    let candidate = resolve_selection_for_display(selection.clone(), appearance);
    install_refresh_candidate(&selection, candidate)
}

fn install_refresh_candidate(selection: &ThemeSelection, candidate: ResolvedTheme) -> bool {
    let previous = current_resolved();

    // A settings file is commonly replaced through rename, producing a short
    // missing/partial-read window. Keep the last good visual result while still
    // publishing the warning status; a later successful event replaces it.
    if let Some(retained) = retain_last_good(&previous, &candidate, selection) {
        ACTIVE.store(Arc::new(retained));
        return false;
    }

    let changed = install_resolved(candidate);
    super::apply_cursor_color();
    changed
}

fn retain_last_good(
    previous: &ResolvedTheme,
    candidate: &ResolvedTheme,
    selection: &ThemeSelection,
) -> Option<ResolvedTheme> {
    // Successfully loaded Warp visuals carry a source hash. A retained visual
    // keeps that hash when its status is replaced with a transient warning, so
    // repeated watcher events must continue retaining it. Initial fallbacks do
    // not have a source hash and therefore never become "last known good."
    if previous.selection != *selection
        || !matches!(previous.status, super::ThemeStatus::Warp { .. })
        || previous.source_hash.is_none()
        || candidate.warp_fallback_reason().is_none()
    {
        return None;
    }
    let mut retained = previous.clone();
    retained.status = candidate.status.clone();
    Some(retained)
}

fn sync_color_caps() {
    let native = active_terminal_native();
    xai_grok_markdown::set_color_level_cap(if native {
        xai_grok_markdown::ColorLevel::Basic
    } else {
        xai_grok_markdown::ColorLevel::TrueColor
    });
}

/// Set the in-memory theme kind without writing to disk.
///
/// Used by the dispatcher (after `Action::SetTheme` is processed) and
/// by the live-preview path during the picker. Disk-write happens via
/// `Effect::PersistSetting`, NOT here.
pub fn set(kind: ThemeKind) {
    match kind {
        ThemeKind::GrokNight
        | ThemeKind::GrokDay
        | ThemeKind::TokyoNight
        | ThemeKind::RosePineMoon
        | ThemeKind::OscuraMidnight => {
            install_resolved(ResolvedTheme::built_in(kind, ThemeSelection::BuiltIn(kind)));
        }
        ThemeKind::TerminalNative => {
            install_resolved(ResolvedTheme::terminal(
                ThemeSelection::TerminalNative,
                "Terminal Native",
                super::ThemePolarity::Dark,
            ));
        }
        ThemeKind::WarpSync | ThemeKind::WarpCustom => {
            CURRENT.store(kind as u8, Ordering::Relaxed);
            LOADED.store(true, Ordering::Release);
        }
        ThemeKind::Auto => {
            CURRENT.store(kind as u8, Ordering::Relaxed);
            LOADED.store(true, Ordering::Release);
        }
    }
}

// -- Terminal-native lock (minimal mode) --------------------------------------

/// Whether the theme is locked to the terminal-native palette.
#[must_use]
pub fn terminal_native_locked() -> bool {
    TERMINAL_NATIVE_LOCK.load(Ordering::Relaxed)
}

/// Engage or clear the terminal-native theme lock.
pub fn set_terminal_native_lock(locked: bool) {
    TERMINAL_NATIVE_LOCK.store(locked, Ordering::Relaxed);
    let syntax_policy = if locked {
        xai_grok_markdown::SyntaxColorPolicy::NamedAnsi
    } else {
        current_resolved().syntax_policy
    };
    xai_grok_markdown::set_syntax_color_policy(syntax_policy);
    sync_color_caps();
}

// -- Auto-mode ---------------------------------------------------------------

/// Whether auto-switching mode is active.
#[must_use]
pub fn is_auto_mode() -> bool {
    AUTO_MODE.load(Ordering::Relaxed)
}

/// Set or clear auto-switching mode.
pub fn set_auto_mode(enabled: bool) {
    AUTO_MODE.store(enabled, Ordering::Relaxed);
}

/// Get the cached auto-theme configuration, loading from config on first access.
///
/// The cache can be invalidated via [`invalidate_auto_theme_config`] so
/// subsequent lookups re-read from disk.
#[must_use]
pub fn auto_theme_config() -> AutoThemeConfig {
    let mut guard = AUTO_THEME_CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    guard.get_or_insert_with(load_auto_theme_config).clone()
}

/// Invalidate the cached auto-theme configuration.
///
/// Call after updating `auto_dark_theme` or `auto_light_theme` in config
/// so subsequent lookups see the new values. Used by the settings modal
/// and the `/theme auto` slash command.
pub fn invalidate_auto_theme_config() {
    *AUTO_THEME_CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// Replace the cached auto-theme mapping with values already present in the
/// pager's in-memory UI snapshot. This avoids re-reading stale on-disk values
/// between an optimistic settings commit and its asynchronous persistence.
pub fn set_auto_theme_config(config: AutoThemeConfig) {
    *AUTO_THEME_CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = Some(config);
}

/// Preview one concrete auto-theme bucket while preserving `Auto` as the
/// active user intent and watcher identity.
pub fn apply_auto_mapping_preview(
    selection: ThemeSelection,
    appearance: Option<system_appearance::SystemAppearance>,
) -> bool {
    if terminal_native_locked() {
        return false;
    }
    let resolved = super::resolved::resolve_selection(selection, appearance)
        .with_selection(ThemeSelection::Auto);
    let changed = install_resolved(resolved);
    super::apply_cursor_color();
    changed
}

// -- Theme resolution --------------------------------------------------------

/// Resolve the effective theme, respecting the full precedence chain.
///
/// Called once at startup. Returns the concrete `ThemeKind` (never `Auto`).
///
/// Precedence:
/// 1. Environment variable (`GROK_THEME`)
/// 2. Config file (`[ui].theme`)
/// 3. Default: `GrokNight`
#[must_use]
pub fn resolve_initial_theme() -> ThemeKind {
    resolve_initial_resolved().kind
}

/// Resolve the complete startup selection, including terminal-native and Warp themes.
#[must_use]
pub fn resolve_initial_resolved() -> ResolvedTheme {
    resolve_initial_resolved_inner(true)
}

fn resolve_initial_resolved_inner(osc11_fallback: bool) -> ResolvedTheme {
    let appearance = if osc11_fallback {
        system_appearance::detect_with_osc11_fallback()
    } else {
        system_appearance::detect()
    };
    match load_raw_theme_from_disk() {
        Some(raw) => match ThemeSelection::from_name(&raw) {
            Some(ThemeSelection::Auto) => {
                set_auto_mode(true);
                resolve_auto_resolved(appearance)
            }
            Some(selection) => {
                set_auto_mode(false);
                super::resolved::resolve_selection(selection, appearance)
            }
            None => {
                tracing::warn!(theme = %raw, "invalid configured theme; using Grok Night");
                set_auto_mode(false);
                ResolvedTheme::built_in(
                    ThemeKind::GrokNight,
                    ThemeSelection::BuiltIn(ThemeKind::GrokNight),
                )
            }
        },
        None if super::warp::settings::is_local_warp() => {
            set_auto_mode(false);
            super::resolved::resolve_selection(ThemeSelection::WarpSync, appearance)
        }
        None => {
            set_auto_mode(false);
            ResolvedTheme::built_in(
                ThemeKind::GrokNight,
                ThemeSelection::BuiltIn(ThemeKind::GrokNight),
            )
        }
    }
}

/// Inner resolution logic, factored out for testability.
#[cfg(test)]
fn resolve_from_config(config_theme: Option<ThemeKind>, osc11_fallback: bool) -> ThemeKind {
    if let Some(kind) = config_theme {
        if kind.is_auto() {
            set_auto_mode(true);
            let appearance = if osc11_fallback {
                system_appearance::detect_with_osc11_fallback()
            } else {
                system_appearance::detect()
            };
            return resolve_from_appearance(appearance);
        }
        return kind;
    }

    // Default: GrokNight
    ThemeKind::GrokNight
}

/// Map an optional appearance detection result to a concrete `ThemeKind`.
fn auto_selection_from_appearance(
    appearance: Option<system_appearance::SystemAppearance>,
) -> ThemeSelection {
    let config = auto_theme_config();
    match appearance {
        Some(system_appearance::SystemAppearance::Light) => config
            .light_theme
            .unwrap_or(ThemeSelection::BuiltIn(ThemeKind::GrokDay)),
        Some(system_appearance::SystemAppearance::Dark) | None => config
            .dark_theme
            .unwrap_or(ThemeSelection::BuiltIn(ThemeKind::GrokNight)),
    }
}

fn resolve_from_appearance(appearance: Option<system_appearance::SystemAppearance>) -> ThemeKind {
    let selection = auto_selection_from_appearance(appearance);
    super::resolved::resolve_selection(selection, appearance).kind
}

#[must_use]
pub fn resolve_auto_resolved(
    appearance: Option<system_appearance::SystemAppearance>,
) -> ResolvedTheme {
    let mapped = auto_selection_from_appearance(appearance);
    super::resolved::resolve_selection(mapped, appearance).with_selection(ThemeSelection::Auto)
}

/// Resolve "auto" by detecting system appearance and mapping via config.
///
/// Returns the concrete `ThemeKind` based on the current system appearance
/// and the user's dark/light theme mapping. Falls back to `GrokNight`
/// when detection fails.
///
/// Uses desktop APIs only (no OSC 11) — safe to call at runtime while
/// crossterm's `EventStream` is active. Called from the settings modal
/// and the `/theme auto` slash command.
#[must_use]
pub fn resolve_auto() -> ThemeKind {
    resolve_from_appearance(system_appearance::detect())
}

/// Variant of [`resolve_initial_theme`] without the OSC 11 startup
/// fallback, for resolution after the terminal is initialized.
#[must_use]
pub fn resolve_initial_theme_no_osc11() -> ThemeKind {
    resolve_initial_resolved_no_osc11().kind
}

#[must_use]
pub fn resolve_initial_resolved_no_osc11() -> ResolvedTheme {
    resolve_initial_resolved_inner(false)
}

// -- Disk reads --------------------------------------------------------------
//
// All writes go through `xai_grok_shell::util::config::set_theme()` (and
// friends) via `Effect::PersistSetting`. This module only READS from the
// shell's layered effective config.

/// Read the theme from the effective config (managed_config.toml merged
/// under config.toml — user wins).
///
/// Checks `[ui].theme` first (the canonical location), then falls back
/// to a top-level `theme` key for backwards compatibility.
fn load_raw_theme_from_disk() -> Option<String> {
    let root = xai_grok_config::load_effective_config_disk_only().ok()?;
    let table = root.as_table()?;
    table
        .get("ui")
        .and_then(|ui| ui.get("theme"))
        .and_then(|v| v.as_str())
        .or_else(|| table.get("theme").and_then(|v| v.as_str()))
        .map(str::to_owned)
}

fn load_from_disk() -> Option<ThemeKind> {
    load_raw_theme_from_disk().and_then(|value| ThemeKind::from_name(&value))
}

/// Load auto-theme configuration from the effective config.
///
/// Reads `[ui].auto_dark_theme` and `[ui].auto_light_theme`, parsing them
/// as theme names. Filters out `Auto` to prevent circular reference.
fn load_auto_theme_config() -> AutoThemeConfig {
    let Ok(root) = xai_grok_config::load_effective_config_disk_only() else {
        return AutoThemeConfig::default();
    };
    let Some(table) = root.as_table() else {
        return AutoThemeConfig::default();
    };
    let ui = table.get("ui");
    AutoThemeConfig {
        dark_theme: ui
            .and_then(|u| u.get("auto_dark_theme"))
            .and_then(|v| v.as_str())
            .and_then(ThemeSelection::from_name)
            .filter(ThemeSelection::is_concrete_for_auto),
        light_theme: ui
            .and_then(|u| u.get("auto_light_theme"))
            .and_then(|v| v.as_str())
            .and_then(ThemeSelection::from_name)
            .filter(ThemeSelection::is_concrete_for_auto),
    }
}

// -- Test support ------------------------------------------------------------

#[cfg(any(test, feature = "test-support"))]
pub fn reset_for_test() {
    // Tests are serialized via TEST_LOCK so the AtomicU8/AtomicBool
    // pair is safe to reset without any cross-thread coordination.
    CURRENT.store(ThemeKind::GrokNight as u8, Ordering::Relaxed);
    LOADED.store(false, Ordering::Release);
    AUTO_MODE.store(false, Ordering::Relaxed);
    ACTIVE.store(Arc::new(ResolvedTheme::built_in(
        ThemeKind::GrokNight,
        ThemeSelection::BuiltIn(ThemeKind::GrokNight),
    )));
    *ACTIVE_SELECTION.lock().unwrap_or_else(|e| e.into_inner()) =
        ThemeSelection::BuiltIn(ThemeKind::GrokNight);
    REVISION.store(1, Ordering::Relaxed);
    set_terminal_native_lock(false);
    *AUTO_THEME_CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// Seed `AUTO_THEME_CONFIG` with explicit defaults so `auto_theme_config()`
/// never falls through to `load_auto_theme_config()` (which reads the
/// user's real `config.toml`). Call from test setup after `reset_for_test()`.
#[cfg(any(test, feature = "test-support"))]
pub fn seed_auto_theme_defaults_for_test() {
    *AUTO_THEME_CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = Some(AutoThemeConfig::default());
}

#[cfg(any(test, feature = "test-support"))]
pub fn test_lock() -> &'static Mutex<()> {
    &TEST_LOCK
}

/// Pin a deterministic theme + color level for a test's duration so exact
/// height / screen-position assertions are hermetic. Rendered heights are
/// computed under the process-global `Theme::current()` (which concurrent
/// `set_theme` tests mutate) and `Theme::current()` reads the global color
/// level; holding the shared test lock blocks a mid-test theme change. Hold the
/// returned guard for the whole test.
#[cfg(any(test, feature = "test-support"))]
pub fn pin_theme() -> std::sync::MutexGuard<'static, ()> {
    let guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    set(ThemeKind::GrokNight);
    // Color level is a write-once `OnceLock`; tests run without a TTY so it
    // resolves to `TrueColor` anyway. Pin it explicitly (best-effort: ignore the
    // already-initialized `Err`) so the measure path that reads it stays fixed.
    let _ = super::color_support::set(super::color_support::ColorLevel::TrueColor);
    guard
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: run a test body while holding the global test lock and
    /// with a clean initial state.
    fn with_test_env(f: impl FnOnce()) {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset_for_test();
        seed_auto_theme_defaults_for_test();
        // Set LOADED=true so current_kind() doesn't read from disk.
        set(ThemeKind::GrokNight);
        system_appearance::clear_mock();
        f();
        system_appearance::clear_mock();
        reset_for_test();
    }

    /// Pre-populate the auto-theme config cache for testing.
    fn set_test_auto_config(config: AutoThemeConfig) {
        *AUTO_THEME_CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = Some(config);
    }

    // -- Terminal-native lock (minimal mode) ----------------------------------

    #[test]
    fn terminal_native_lock_pins_kind_and_blocks_apply_kind() {
        with_test_env(|| {
            set(ThemeKind::GrokDay);
            set_terminal_native_lock(true);
            assert!(terminal_native_locked());
            assert_eq!(current_kind(), ThemeKind::GrokNight, "nominal kind");

            let applied = super::super::Theme::apply_kind(ThemeKind::GrokDay);
            assert_eq!(applied, ThemeKind::GrokNight, "apply_kind must no-op");
            assert_eq!(current_kind(), ThemeKind::GrokNight);

            set_terminal_native_lock(false);
            assert_eq!(
                current_kind(),
                ThemeKind::GrokDay,
                "unlocking restores the cached kind"
            );
        });
    }

    #[test]
    fn terminal_native_lock_serves_terminal_default_palette() {
        with_test_env(|| {
            set(ThemeKind::GrokDay);
            set_terminal_native_lock(true);
            let theme = super::super::Theme::current();
            let native = super::super::Theme::terminal_default();
            assert_eq!(theme.bg_base, native.bg_base);
            assert_eq!(theme.text_primary, native.text_primary);
            assert_eq!(theme.accent_user, native.accent_user);
            assert_ne!(
                theme.text_primary,
                super::super::Theme::grokday().text_primary,
                "must not serve the cached (GrokDay) theme"
            );
        });
    }

    #[test]
    fn reset_for_test_clears_terminal_native_lock() {
        with_test_env(|| {
            set_terminal_native_lock(true);
            reset_for_test();
            assert!(!terminal_native_locked());
        });
    }

    #[test]
    fn terminal_native_lock_switches_and_restores_syntax_policy() {
        with_test_env(|| {
            assert_eq!(
                xai_grok_markdown::syntax_color_policy(),
                xai_grok_markdown::SyntaxColorPolicy::Passthrough
            );
            set_terminal_native_lock(true);
            assert_eq!(
                xai_grok_markdown::syntax_color_policy(),
                xai_grok_markdown::SyntaxColorPolicy::NamedAnsi
            );
            set_terminal_native_lock(false);
            assert_eq!(
                xai_grok_markdown::syntax_color_policy(),
                xai_grok_markdown::SyntaxColorPolicy::Passthrough
            );
        });
    }

    #[test]
    fn terminal_native_row_styles_keep_canvas_transparent_and_focus_visible() {
        use ratatui::style::{Color, Modifier};

        with_test_env(|| {
            install_resolved(ResolvedTheme::terminal(
                ThemeSelection::TerminalNative,
                "Terminal Native",
                super::super::ThemePolarity::Dark,
            ));
            let theme = super::super::Theme::current();
            assert_eq!(theme.bg_base, Color::Reset);
            let selected = theme.selected_row_style();
            assert_eq!(selected.bg, None);
            assert!(selected.add_modifier.contains(Modifier::REVERSED));
            assert!(selected.add_modifier.contains(Modifier::BOLD));
            assert!(
                theme
                    .hovered_row_style()
                    .add_modifier
                    .contains(Modifier::UNDERLINED)
            );

            set(ThemeKind::GrokNight);
            let opaque = super::super::Theme::current();
            assert_eq!(opaque.selected_row_style().bg, Some(opaque.bg_visual));
            assert_eq!(opaque.hovered_row_style().bg, Some(opaque.bg_hover));
        });
    }

    #[test]
    fn terminal_native_lock_caps_quantize_at_ansi16() {
        use ratatui::style::Color;

        use crate::theme::color_support;
        with_test_env(|| {
            set_terminal_native_lock(true);
            assert!(color_support::detect() <= color_support::ColorLevel::Basic);
            for input in [
                Color::Rgb(0x26, 0x26, 0x26), // grokday text_primary
                Color::Rgb(122, 162, 247),
                Color::Indexed(141),
            ] {
                let q = color_support::quantize(input);
                assert!(
                    !matches!(q, Color::Rgb(..) | Color::Indexed(_)),
                    "quantize({input:?}) must collapse to Reset/named ANSI under \
                     the lock, got {q:?}"
                );
            }
        });
    }

    #[test]
    fn resolve_no_osc11_explicit_auto_and_default() {
        with_test_env(|| {
            assert_eq!(
                resolve_from_config(Some(ThemeKind::GrokDay), false),
                ThemeKind::GrokDay
            );
            assert!(!is_auto_mode());

            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Light));
            assert_eq!(
                resolve_from_config(Some(ThemeKind::Auto), false),
                ThemeKind::GrokDay
            );
            assert!(is_auto_mode(), "auto must arm the appearance watcher");

            assert_eq!(resolve_from_config(None, false), ThemeKind::GrokNight);
        });
    }

    // -- AUTO_MODE -----------------------------------------------------------

    #[test]
    fn auto_mode_default_is_false() {
        with_test_env(|| {
            assert!(!is_auto_mode());
        });
    }

    #[test]
    fn set_auto_mode_toggles() {
        with_test_env(|| {
            set_auto_mode(true);
            assert!(is_auto_mode());
            set_auto_mode(false);
            assert!(!is_auto_mode());
        });
    }

    // -- AutoThemeConfig -----------------------------------------------------

    #[test]
    fn auto_theme_config_defaults_to_none() {
        let config = AutoThemeConfig::default();
        assert!(config.dark_theme.is_none());
        assert!(config.light_theme.is_none());
    }

    // -- resolve_auto --------------------------------------------------------

    #[test]
    fn resolve_auto_dark_system_returns_groknight() {
        with_test_env(|| {
            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Dark));
            let result = resolve_auto();
            assert_eq!(result, ThemeKind::GrokNight);
        });
    }

    #[test]
    fn resolve_auto_light_system_returns_grokday() {
        with_test_env(|| {
            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Light));
            let result = resolve_auto();
            assert_eq!(result, ThemeKind::GrokDay);
        });
    }

    #[test]
    fn resolve_auto_detection_failure_returns_groknight() {
        with_test_env(|| {
            system_appearance::set_mock(None);
            let result = resolve_auto();
            assert_eq!(result, ThemeKind::GrokNight);
        });
    }

    // -- invalidate_auto_theme_config ----------------------------------------

    #[test]
    fn invalidate_clears_cached_config() {
        with_test_env(|| {
            // Pre-populate the cache with a known config.
            set_test_auto_config(AutoThemeConfig {
                dark_theme: Some(ThemeSelection::BuiltIn(ThemeKind::TokyoNight)),
                light_theme: None,
            });
            let config1 = auto_theme_config();
            assert_eq!(
                config1.dark_theme,
                Some(ThemeSelection::BuiltIn(ThemeKind::TokyoNight))
            );

            // Invalidate — next read re-loads (defaults in test env).
            invalidate_auto_theme_config();
            // Pre-populate again with defaults to avoid disk dependency.
            set_test_auto_config(AutoThemeConfig::default());
            let config2 = auto_theme_config();
            assert!(config2.dark_theme.is_none());
        });
    }

    // -- resolve_from_config (resolve_initial_theme inner logic) ---------------

    #[test]
    fn resolve_from_config_no_config_returns_groknight() {
        with_test_env(|| {
            let result = resolve_from_config(None, true);
            assert_eq!(result, ThemeKind::GrokNight);
            assert!(!is_auto_mode());
        });
    }

    #[test]
    fn resolve_from_config_explicit_theme_returns_it() {
        with_test_env(|| {
            let result = resolve_from_config(Some(ThemeKind::GrokDay), true);
            assert_eq!(result, ThemeKind::GrokDay);
            assert!(
                !is_auto_mode(),
                "explicit theme should not enable auto mode"
            );
        });
    }

    #[test]
    fn resolve_from_config_auto_sets_auto_mode_dark() {
        with_test_env(|| {
            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Dark));
            let result = resolve_from_config(Some(ThemeKind::Auto), true);
            assert_eq!(result, ThemeKind::GrokNight);
            assert!(is_auto_mode(), "auto config must enable auto mode");
        });
    }

    #[test]
    fn resolve_from_config_auto_with_light_system() {
        with_test_env(|| {
            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Light));
            let result = resolve_from_config(Some(ThemeKind::Auto), true);
            assert_eq!(result, ThemeKind::GrokDay);
            assert!(is_auto_mode());
        });
    }

    #[test]
    fn resolve_from_config_auto_detection_failure() {
        with_test_env(|| {
            system_appearance::set_mock(None);
            let result = resolve_from_config(Some(ThemeKind::Auto), true);
            assert_eq!(result, ThemeKind::GrokNight);
            assert!(is_auto_mode(), "auto mode is set before detection");
        });
    }

    // -- resolve_auto with custom config -------------------------------------

    #[test]
    fn resolve_auto_with_custom_dark_config() {
        with_test_env(|| {
            set_test_auto_config(AutoThemeConfig {
                dark_theme: Some(ThemeSelection::BuiltIn(ThemeKind::TokyoNight)),
                light_theme: None,
            });
            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Dark));
            assert_eq!(resolve_auto(), ThemeKind::TokyoNight);
        });
    }

    #[test]
    fn resolve_auto_with_custom_light_config() {
        with_test_env(|| {
            set_test_auto_config(AutoThemeConfig {
                dark_theme: None,
                light_theme: Some(ThemeSelection::BuiltIn(ThemeKind::RosePineMoon)),
            });
            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Light));
            assert_eq!(resolve_auto(), ThemeKind::RosePineMoon);
        });
    }

    #[test]
    fn auto_resolution_keeps_auto_in_the_stable_fingerprint() {
        with_test_env(|| {
            let auto = resolve_auto_resolved(Some(system_appearance::SystemAppearance::Dark));
            let explicit = ResolvedTheme::built_in(
                ThemeKind::GrokNight,
                ThemeSelection::BuiltIn(ThemeKind::GrokNight),
            );
            assert_eq!(auto.selection, ThemeSelection::Auto);
            assert_ne!(auto.fingerprint, explicit.fingerprint);
        });
    }

    #[test]
    fn auto_bucket_preview_preserves_auto_as_active_intent() {
        with_test_env(|| {
            apply_selection(
                ThemeSelection::Auto,
                Some(system_appearance::SystemAppearance::Dark),
            );
            apply_auto_mapping_preview(
                ThemeSelection::BuiltIn(ThemeKind::TokyoNight),
                Some(system_appearance::SystemAppearance::Dark),
            );
            assert_eq!(current_selection(), ThemeSelection::Auto);
            assert_eq!(current_kind(), ThemeKind::TokyoNight);
            assert!(is_auto_mode());
        });
    }

    #[test]
    fn install_revision_changes_only_with_visual_fingerprint() {
        with_test_env(|| {
            let before = current_revision();
            let day = ResolvedTheme::built_in(
                ThemeKind::GrokDay,
                ThemeSelection::BuiltIn(ThemeKind::GrokDay),
            );
            assert!(install_resolved(day.clone()));
            assert_eq!(current_revision(), before + 1);
            assert!(!install_resolved(day));
            assert_eq!(current_revision(), before + 1);
        });
    }

    #[test]
    fn repeated_warp_failures_retain_palette_until_installed_recovery() {
        with_test_env(|| {
            let selection = ThemeSelection::WarpFile("warp-file:stable/fixture.yaml".to_owned());
            let fallback = |reason: &str| {
                let mut candidate = ResolvedTheme::terminal(
                    selection.clone(),
                    "Unavailable Warp theme — Terminal Native",
                    super::super::ThemePolarity::Dark,
                );
                candidate.kind = ThemeKind::WarpCustom;
                candidate.status = super::super::ThemeStatus::Warp {
                    channel: None,
                    selected_name: Some("Fixture".to_owned()),
                    settings_path: None,
                    selected_theme_path: None,
                    system_theme: false,
                    fallback_reason: Some(reason.to_owned()),
                };
                assert!(candidate.source_hash.is_none());
                candidate
            };

            let initial_palette = [[0x11; 3]; 16];
            let initial_policy =
                xai_grok_markdown::SyntaxColorPolicy::rgb_ansi_palette(initial_palette);
            let initial_source_hash = [0x42; 32];
            let previous = super::super::resolved::pinned_fixture_for_test(
                selection.clone(),
                initial_palette,
                initial_source_hash,
            );
            assert_eq!(previous.source_hash, Some(initial_source_hash));
            assert!(install_resolved(previous));
            let revision = current_revision();
            assert_eq!(xai_grok_markdown::syntax_color_policy(), initial_policy);

            assert!(!install_refresh_candidate(
                &selection,
                fallback("partial settings write"),
            ));
            let retained = current_resolved();
            assert_eq!(
                retained.warp_fallback_reason(),
                Some("partial settings write")
            );
            assert_eq!(retained.syntax_policy, initial_policy);
            assert_eq!(retained.source_hash, Some(initial_source_hash));
            assert_eq!(xai_grok_markdown::syntax_color_policy(), initial_policy);
            assert_eq!(current_revision(), revision);

            assert!(!install_refresh_candidate(
                &selection,
                fallback("settings file still incomplete"),
            ));
            let retained_again = current_resolved();
            assert_eq!(
                retained_again.warp_fallback_reason(),
                Some("settings file still incomplete")
            );
            assert_eq!(retained_again.syntax_policy, initial_policy);
            assert_eq!(retained_again.source_hash, Some(initial_source_hash));
            assert_eq!(xai_grok_markdown::syntax_color_policy(), initial_policy);
            assert_eq!(current_revision(), revision);

            let mut recovered_palette = initial_palette;
            recovered_palette[15] = [0xaa, 0xbb, 0xcc];
            let recovered_policy =
                xai_grok_markdown::SyntaxColorPolicy::rgb_ansi_palette(recovered_palette);
            let recovered_source_hash = [0x84; 32];
            let recovered = super::super::resolved::pinned_fixture_for_test(
                selection.clone(),
                recovered_palette,
                recovered_source_hash,
            );
            assert!(install_refresh_candidate(&selection, recovered));

            let installed = current_resolved();
            assert!(installed.warp_fallback_reason().is_none());
            assert_eq!(installed.syntax_policy, recovered_policy);
            assert_eq!(installed.source_hash, Some(recovered_source_hash));
            assert_eq!(xai_grok_markdown::syntax_color_policy(), recovered_policy);
            assert_eq!(current_revision(), revision + 1);
        });
    }

    // -- auto_theme_config filter --------------------------------------------

    #[test]
    fn auto_theme_config_filter_rejects_auto_value() {
        // Simulates the .filter(|k| !k.is_auto()) guard in load_auto_theme_config().
        // When config contains auto_dark_theme = "auto", from_name returns Some(Auto),
        // but the filter discards it to prevent circular reference.
        let parsed = ThemeKind::from_name("auto").filter(|k| !k.is_auto());
        assert!(parsed.is_none(), "Auto must be filtered out");
    }

    #[test]
    fn auto_theme_config_filter_accepts_concrete_theme() {
        let parsed = ThemeKind::from_name("tokyonight").filter(|k| !k.is_auto());
        assert_eq!(parsed, Some(ThemeKind::TokyoNight));
    }

    // -- set / current_kind --------------------------------------------------

    /// `set` followed by `current_kind` returns the set value, and the
    /// `LOADED` flag flips so subsequent reads don't re-seed from disk.
    /// The optimistic-update invariant the dispatcher relies on.
    ///
    /// Explicitly observe the `LOADED` flag
    /// side-effect by calling `reset_for_test()` between sets — if
    /// `set` didn't flip `LOADED = true`, the second `current_kind`
    /// read would re-seed from disk and the assertion would fail.
    #[test]
    fn set_then_current_kind_round_trips() {
        with_test_env(|| {
            set(ThemeKind::TokyoNight);
            assert_eq!(current_kind(), ThemeKind::TokyoNight);
            set(ThemeKind::GrokDay);
            assert_eq!(current_kind(), ThemeKind::GrokDay);
        });
    }

    /// `set` flips `LOADED` so a subsequent `current_kind` read does
    /// NOT re-seed from disk. Mirror of the
    /// `set_then_current_kind_round_trips` test that the docstring
    /// claims to enforce — exercises the `LOADED` flag invariant
    /// directly via the atomic statics.
    #[test]
    fn set_flips_loaded_flag_so_current_kind_skips_disk_reseed() {
        with_test_env(|| {
            // with_test_env seeds LOADED=true to prevent disk reads;
            // this test specifically needs LOADED=false to verify that
            // set() flips it.
            LOADED.store(false, Ordering::Release);
            assert!(
                !LOADED.load(Ordering::Acquire),
                "LOADED must be false for this test"
            );
            set(ThemeKind::GrokDay);
            assert!(
                LOADED.load(Ordering::Acquire),
                "set must flip LOADED to true"
            );
            // Subsequent current_kind read returns the set value (no
            // disk re-seed).
            assert_eq!(current_kind(), ThemeKind::GrokDay);
            assert!(
                LOADED.load(Ordering::Acquire),
                "current_kind must NOT flip LOADED back to false"
            );
        });
    }
}
