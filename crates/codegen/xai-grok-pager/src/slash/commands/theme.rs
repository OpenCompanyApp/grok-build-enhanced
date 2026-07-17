//! `/theme` (alias `/t`) -- switch the color theme.
//!
//! Toggles between available themes or switches to a named theme.
//! Selecting `auto` enables system-appearance-driven theme switching.
//! Selecting an explicit theme disengages auto mode.
//!
//! `run` dispatches `Action::SetTheme(<canonical>)` — the dispatcher
//! handles mutation + persistence + toast. `preview_arg` /
//! `cancel_preview` call `Theme::apply_kind` directly for non-persisting
//! visual previews (no toast/disk writes per keystroke).

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};
#[cfg(test)]
use crate::theme::Theme;
use crate::theme::{ThemeKind, ThemeSelection, cache as theme_cache};

/// Switch the pager color theme.
pub struct ThemeCommand;

impl SlashCommand for ThemeCommand {
    fn name(&self) -> &str {
        "theme"
    }

    fn aliases(&self) -> &[&str] {
        &["t"]
    }

    fn description(&self) -> &str {
        "Switch the color theme"
    }

    /// Minimal has no theming, so there is nothing for `/theme` to switch.
    fn available_in_minimal(&self) -> bool {
        false
    }

    fn usage(&self) -> &str {
        "/theme <name>"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn args_required(&self) -> bool {
        false
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("<theme>")
    }

    fn supports_preview(&self) -> bool {
        true
    }

    fn preview_state(&self) -> Option<String> {
        Some(theme_cache::current_selection().canonical().into_owned())
    }

    fn preview_arg(&self, arg: &str) {
        if let Some(selection) = ThemeSelection::from_name(arg) {
            let resolved = theme_cache::resolve_selection_for_display(
                selection,
                crate::theme::system_appearance::detect(),
            );
            theme_cache::install_resolved(resolved);
            crate::theme::apply_cursor_color();
        }
    }

    fn cancel_preview(&self, previous: &str) {
        self.preview_arg(previous);
    }

    fn suggest_args(&self, _ctx: &AppCtx, _args_query: &str) -> Option<Vec<ArgItem>> {
        let current = theme_cache::current_selection().canonical().into_owned();
        Some(
            crate::theme::theme_choices(true)
                .into_iter()
                .map(|choice| {
                    let active = if choice.canonical == current {
                        " (active)"
                    } else {
                        ""
                    };
                    ArgItem {
                        display: choice.display.clone(),
                        match_text: format!("{} {}", choice.display, choice.canonical),
                        insert_text: choice.canonical,
                        description: format!("{}{active}", choice.description),
                    }
                })
                .collect(),
        )
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();

        // No args: cycle only the curated set, never the 340-theme catalog.
        if trimmed.is_empty() {
            let mut cycle = vec![
                ThemeSelection::TerminalNative,
                ThemeSelection::BuiltIn(ThemeKind::GrokNight),
                ThemeSelection::BuiltIn(ThemeKind::GrokDay),
            ];
            cycle.extend(
                ThemeKind::available()
                    .iter()
                    .copied()
                    .filter(|kind| !matches!(kind, ThemeKind::GrokNight | ThemeKind::GrokDay))
                    .map(ThemeSelection::BuiltIn),
            );
            if crate::theme::warp::settings::is_local_warp() {
                cycle.insert(0, ThemeSelection::WarpSync);
            }
            let current = theme_cache::current_selection();
            let current_idx = cycle
                .iter()
                .position(|selection| *selection == current)
                .unwrap_or(0);
            let next = &cycle[(current_idx + 1) % cycle.len()];
            return CommandResult::Action(Action::SetTheme(next.canonical().into_owned()));
        }

        match ThemeSelection::from_name(trimmed) {
            Some(selection) => {
                CommandResult::Action(Action::SetTheme(selection.canonical().into_owned()))
            }
            None => CommandResult::Error(format!(
                "Unknown theme: {trimmed}. Run /theme to search built-in, installed, and official Warp themes."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{cache as theme_cache, system_appearance};

    /// Run a test with a clean in-memory state. Prevents disk reads by
    /// pre-loading the theme state.
    fn with_test_env(f: impl FnOnce()) {
        let _guard = theme_cache::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        theme_cache::reset_for_test();
        theme_cache::seed_auto_theme_defaults_for_test();
        system_appearance::clear_mock();
        // Set LOADED=true so current_kind() doesn't try to read from disk.
        theme_cache::set(ThemeKind::GrokNight);
        f();
        system_appearance::clear_mock();
        theme_cache::reset_for_test();
    }

    #[test]
    fn theme_unavailable_in_minimal() {
        assert!(!ThemeCommand.available_in_minimal());
    }

    // -- suggest_args ---------------------------------------------------------

    #[test]
    fn suggest_args_prepends_auto_option() {
        with_test_env(|| {
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let ctx = AppCtx {
                models: &models,
                cwd: std::path::Path::new("."),
                has_session_announcements: false,
                screen_mode: crate::app::ScreenMode::Fullscreen,
            };
            let items = cmd.suggest_args(&ctx, "").expect("should return items");
            assert_eq!(items[0].insert_text, "auto");
            assert!(items[0].description.contains("system dark/light"));
            assert_eq!(items.len(), crate::theme::theme_choices(true).len());
        });
    }

    #[test]
    fn suggest_args_auto_active_when_auto_mode() {
        with_test_env(|| {
            theme_cache::apply_selection(ThemeSelection::Auto, system_appearance::detect());
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let ctx = AppCtx {
                models: &models,
                cwd: std::path::Path::new("."),
                has_session_announcements: false,
                screen_mode: crate::app::ScreenMode::Fullscreen,
            };
            let items = cmd.suggest_args(&ctx, "").expect("should return items");
            assert!(
                items[0].description.contains("(active)"),
                "auto should show (active), got: {}",
                items[0].description
            );
        });
    }

    #[test]
    fn suggest_args_auto_not_active_when_explicit() {
        with_test_env(|| {
            theme_cache::set_auto_mode(false);
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let ctx = AppCtx {
                models: &models,
                cwd: std::path::Path::new("."),
                has_session_announcements: false,
                screen_mode: crate::app::ScreenMode::Fullscreen,
            };
            let items = cmd.suggest_args(&ctx, "").expect("should return items");
            assert!(
                !items[0].description.contains("(active)"),
                "auto should not show (active), got: {}",
                items[0].description
            );
        });
    }

    #[test]
    fn suggest_args_explicit_active_when_not_auto() {
        with_test_env(|| {
            theme_cache::set_auto_mode(false);
            theme_cache::set(ThemeKind::GrokNight);
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let ctx = AppCtx {
                models: &models,
                cwd: std::path::Path::new("."),
                has_session_announcements: false,
                screen_mode: crate::app::ScreenMode::Fullscreen,
            };
            let items = cmd.suggest_args(&ctx, "").expect("should return items");
            let groknight = items
                .iter()
                .find(|i| i.insert_text == "groknight")
                .expect("groknight should be in list");
            assert!(
                groknight.description.contains("(active)"),
                "explicit theme should show (active), got: {}",
                groknight.description
            );
        });
    }

    #[test]
    fn suggest_args_no_explicit_active_when_auto() {
        with_test_env(|| {
            theme_cache::apply_selection(ThemeSelection::Auto, system_appearance::detect());
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let ctx = AppCtx {
                models: &models,
                cwd: std::path::Path::new("."),
                has_session_announcements: false,
                screen_mode: crate::app::ScreenMode::Fullscreen,
            };
            let items = cmd.suggest_args(&ctx, "").expect("should return items");
            // No concrete theme should show "(active)" in auto mode.
            for item in items.iter().skip(1) {
                assert!(
                    !item.description.contains("(active)"),
                    "{} should not show (active) in auto mode",
                    item.insert_text
                );
            }
        });
    }

    // -- run (dispatches Action::SetTheme) ------------------------------------

    /// `/theme <name>` returns `Action::SetTheme(<canonical>)` —
    /// the dispatcher handles in-memory state + disk write + toast.
    #[test]
    fn run_explicit_dispatches_set_theme_action() {
        with_test_env(|| {
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let bundle = crate::app::bundle::BundleState::default();
            let mut ctx = CommandExecCtx {
                models: &models,
                session_id: None,
                bundle_state: &bundle,
                screen_mode: crate::app::ScreenMode::Inline,
                pager_state: crate::settings::PagerLocalSnapshot {
                    multiline_mode: false,
                    yolo_mode: false,
                    ..crate::settings::PagerLocalSnapshot::default()
                },
            };
            let result = cmd.run(&mut ctx, "groknight");
            match result {
                CommandResult::Action(Action::SetTheme(name)) => {
                    assert_eq!(name, "groknight");
                }
                other => panic!("expected Action::SetTheme(\"groknight\"), got {other:?}"),
            }
        });
    }

    /// `/theme` (no args) toggles by dispatching `Action::SetTheme(<next>)`.
    /// Precondition-assert that `ThemeKind::available()` has ≥2 entries;
    /// otherwise the previous `unwrap_or` masked a broken upstream
    /// invariant.
    #[test]
    fn run_toggle_dispatches_set_theme_action() {
        with_test_env(|| {
            theme_cache::set(ThemeKind::GrokNight);
            // Hard-fail with a clear message if the precondition
            // breaks — `(0 + 1) % 0` in `run` would otherwise panic
            // with `attempt to calculate the remainder with a
            // divisor of zero`, which is a worse error message.
            assert!(
                ThemeKind::available().len() >= 2,
                "toggle test requires ≥2 available themes, got {}",
                ThemeKind::available().len(),
            );
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let bundle = crate::app::bundle::BundleState::default();
            let mut ctx = CommandExecCtx {
                models: &models,
                session_id: None,
                bundle_state: &bundle,
                screen_mode: crate::app::ScreenMode::Inline,
                pager_state: crate::settings::PagerLocalSnapshot {
                    multiline_mode: false,
                    yolo_mode: false,
                    ..crate::settings::PagerLocalSnapshot::default()
                },
            };
            let result = cmd.run(&mut ctx, "");
            match result {
                CommandResult::Action(Action::SetTheme(name)) => {
                    // available[0] = GrokNight; next is available[1].
                    let expected = ThemeKind::available()[1].display_name();
                    assert_eq!(name, expected);
                }
                other => panic!("expected Action::SetTheme(...), got {other:?}"),
            }
        });
    }

    /// `/theme auto` dispatches `SetTheme("auto")`.
    #[test]
    fn run_auto_dispatches_set_theme_auto() {
        with_test_env(|| {
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let bundle = crate::app::bundle::BundleState::default();
            let mut ctx = CommandExecCtx {
                models: &models,
                session_id: None,
                bundle_state: &bundle,
                screen_mode: crate::app::ScreenMode::Inline,
                pager_state: crate::settings::PagerLocalSnapshot {
                    multiline_mode: false,
                    yolo_mode: false,
                    ..crate::settings::PagerLocalSnapshot::default()
                },
            };
            let result = cmd.run(&mut ctx, "auto");
            match result {
                CommandResult::Action(Action::SetTheme(name)) => {
                    assert_eq!(name, "auto");
                }
                other => panic!("expected Action::SetTheme(\"auto\"), got {other:?}"),
            }
        });
    }

    /// Aliases normalise to canonical `display_name` before dispatch.
    #[test]
    fn run_alias_normalises_to_canonical() {
        with_test_env(|| {
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let bundle = crate::app::bundle::BundleState::default();
            let mut ctx = CommandExecCtx {
                models: &models,
                session_id: None,
                bundle_state: &bundle,
                screen_mode: crate::app::ScreenMode::Inline,
                pager_state: crate::settings::PagerLocalSnapshot {
                    multiline_mode: false,
                    yolo_mode: false,
                    ..crate::settings::PagerLocalSnapshot::default()
                },
            };
            // "dark" is an alias for GrokNight.
            let result = cmd.run(&mut ctx, "dark");
            match result {
                CommandResult::Action(Action::SetTheme(name)) => {
                    assert_eq!(name, "groknight", "alias must normalise to canonical");
                }
                other => panic!("expected Action::SetTheme(\"groknight\"), got {other:?}"),
            }
        });
    }

    // -- preview_arg ----------------------------------------------------------

    #[test]
    fn preview_auto_applies_resolved_theme() {
        with_test_env(|| {
            system_appearance::set_mock(Some(system_appearance::SystemAppearance::Light));
            let cmd = ThemeCommand;
            cmd.preview_arg("auto");
            // Default auto config maps Light -> GrokDay.
            assert_eq!(Theme::current_kind(), ThemeKind::GrokDay);
        });
    }

    /// `preview_arg` applies the named theme directly.
    #[test]
    fn preview_explicit_theme_applies_directly() {
        with_test_env(|| {
            theme_cache::set(ThemeKind::GrokNight);
            let cmd = ThemeCommand;
            cmd.preview_arg("grokday");
            assert_eq!(Theme::current_kind(), ThemeKind::GrokDay);
        });
    }

    /// `preview_arg` with unknown theme is a no-op.
    #[test]
    fn preview_unknown_theme_is_no_op() {
        with_test_env(|| {
            theme_cache::set(ThemeKind::GrokNight);
            let cmd = ThemeCommand;
            cmd.preview_arg("nonexistent-theme");
            assert_eq!(
                Theme::current_kind(),
                ThemeKind::GrokNight,
                "unknown theme name must NOT change Theme::current_kind",
            );
        });
    }

    // -- cancel_preview -------------------------------------------------------

    /// `cancel_preview` restores the previously-applied theme.
    #[test]
    fn cancel_preview_restores_previous_kind() {
        with_test_env(|| {
            theme_cache::set(ThemeKind::GrokNight);
            let cmd = ThemeCommand;
            // Simulate user navigating into a different theme during preview.
            cmd.preview_arg("grokday");
            assert_eq!(Theme::current_kind(), ThemeKind::GrokDay);

            // Then Escape (or arg picker dismissal): restore.
            cmd.cancel_preview("groknight");
            assert_eq!(
                Theme::current_kind(),
                ThemeKind::GrokNight,
                "cancel_preview must restore the previous canonical",
            );
        });
    }

    /// `cancel_preview` with unknown theme is a no-op.
    #[test]
    fn cancel_preview_unknown_theme_is_no_op() {
        with_test_env(|| {
            theme_cache::set(ThemeKind::GrokDay);
            let cmd = ThemeCommand;
            cmd.cancel_preview("nonexistent-theme");
            assert_eq!(
                Theme::current_kind(),
                ThemeKind::GrokDay,
                "unknown previous must NOT change Theme::current_kind",
            );
        });
    }

    // -- error handling -------------------------------------------------------

    #[test]
    fn run_unknown_lists_auto_in_available() {
        with_test_env(|| {
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let bundle = crate::app::bundle::BundleState::default();
            let mut ctx = CommandExecCtx {
                models: &models,
                session_id: None,
                bundle_state: &bundle,
                screen_mode: crate::app::ScreenMode::Inline,
                pager_state: crate::settings::PagerLocalSnapshot {
                    multiline_mode: false,
                    yolo_mode: false,
                    ..crate::settings::PagerLocalSnapshot::default()
                },
            };
            let result = cmd.run(&mut ctx, "nonexistent");
            if let CommandResult::Error(msg) = result {
                assert!(
                    msg.contains("Run /theme"),
                    "error should point to /theme: {msg}"
                );
                assert!(
                    msg.contains("official Warp themes"),
                    "error should describe the searchable catalog: {msg}"
                );
            } else {
                panic!("expected Error, got: {result:?}");
            }
        });
    }

    /// Truecolor-only themes are accepted; clamping happens downstream.
    #[test]
    fn run_truecolor_theme_dispatches_set_theme_action() {
        with_test_env(|| {
            let cmd = ThemeCommand;
            let models = crate::acp::model_state::ModelState::default();
            let bundle = crate::app::bundle::BundleState::default();
            let mut ctx = CommandExecCtx {
                models: &models,
                session_id: None,
                bundle_state: &bundle,
                screen_mode: crate::app::ScreenMode::Inline,
                pager_state: crate::settings::PagerLocalSnapshot {
                    multiline_mode: false,
                    yolo_mode: false,
                    ..crate::settings::PagerLocalSnapshot::default()
                },
            };
            let result = cmd.run(&mut ctx, "tokyonight");
            match result {
                CommandResult::Action(Action::SetTheme(name)) => {
                    assert_eq!(
                        name, "tokyonight",
                        "truecolor themes must be accepted; clamping happens \
                         downstream in `Theme::apply_kind`",
                    );
                }
                other => panic!("expected Action::SetTheme(\"tokyonight\"), got {other:?}"),
            }
        });
    }
}
