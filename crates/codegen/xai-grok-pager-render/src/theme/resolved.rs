use std::path::PathBuf;

use ratatui::style::Color;

use super::system_appearance::SystemAppearance;
use super::warp::{catalog, discovery, settings, translate};
use super::{Theme, ThemeKind, ThemeSelection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemePolarity {
    Dark,
    Light,
}

impl ThemePolarity {
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeRenderMode {
    Opaque,
    TerminalNative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorPolicy {
    ThemeColor(Color),
    TerminalDefault,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeFingerprint(pub [u8; 16]);

impl ThemeFingerprint {
    pub fn as_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

#[derive(Debug, Clone)]
pub enum ThemeStatus {
    BuiltIn,
    Terminal,
    Warp {
        channel: Option<settings::WarpChannel>,
        selected_name: Option<String>,
        settings_path: Option<PathBuf>,
        selected_theme_path: Option<PathBuf>,
        system_theme: bool,
        fallback_reason: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub selection: ThemeSelection,
    pub kind: ThemeKind,
    pub display_name: String,
    pub theme: Theme,
    pub polarity: ThemePolarity,
    pub render_mode: ThemeRenderMode,
    pub cursor_policy: CursorPolicy,
    pub syntax_policy: xai_grok_markdown::SyntaxColorPolicy,
    pub status: ThemeStatus,
    pub fingerprint: ThemeFingerprint,
    pub source_hash: Option<[u8; 32]>,
}

impl ResolvedTheme {
    pub fn built_in(kind: ThemeKind, selection: ThemeSelection) -> Self {
        let theme = theme_for_kind(kind);
        let polarity = if theme.is_dark() {
            ThemePolarity::Dark
        } else {
            ThemePolarity::Light
        };
        Self::finish(
            selection,
            kind,
            super::display_name_for_canonical(kind.display_name()).to_owned(),
            theme,
            polarity,
            ThemeRenderMode::Opaque,
            CursorPolicy::ThemeColor(theme.accent_user),
            xai_grok_markdown::SyntaxColorPolicy::Passthrough,
            ThemeStatus::BuiltIn,
            None,
        )
    }

    pub fn terminal(
        selection: ThemeSelection,
        display_name: impl Into<String>,
        polarity: ThemePolarity,
    ) -> Self {
        let theme = Theme::terminal_default();
        Self::finish(
            selection,
            ThemeKind::TerminalNative,
            display_name.into(),
            theme,
            polarity,
            ThemeRenderMode::TerminalNative,
            CursorPolicy::TerminalDefault,
            xai_grok_markdown::SyntaxColorPolicy::NamedAnsi,
            ThemeStatus::Terminal,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn finish(
        selection: ThemeSelection,
        kind: ThemeKind,
        display_name: String,
        theme: Theme,
        polarity: ThemePolarity,
        render_mode: ThemeRenderMode,
        cursor_policy: CursorPolicy,
        syntax_policy: xai_grok_markdown::SyntaxColorPolicy,
        status: ThemeStatus,
        source_hash: Option<[u8; 32]>,
    ) -> Self {
        let fingerprint = fingerprint(
            &selection,
            &display_name,
            &theme,
            polarity,
            render_mode,
            syntax_policy,
            source_hash,
        );
        Self {
            selection,
            kind,
            display_name,
            theme,
            polarity,
            render_mode,
            cursor_policy,
            syntax_policy,
            status,
            fingerprint,
            source_hash,
        }
    }

    pub(crate) fn with_selection(mut self, selection: ThemeSelection) -> Self {
        self.selection = selection;
        self.fingerprint = fingerprint(
            &self.selection,
            &self.display_name,
            &self.theme,
            self.polarity,
            self.render_mode,
            self.syntax_policy,
            self.source_hash,
        );
        self
    }

    pub fn requires_system_appearance_watcher(&self) -> bool {
        matches!(
            self.status,
            ThemeStatus::Warp {
                system_theme: true,
                ..
            }
        )
    }

    pub fn warp_watch_paths(&self) -> Vec<PathBuf> {
        let ThemeStatus::Warp {
            settings_path,
            selected_theme_path,
            ..
        } = &self.status
        else {
            return Vec::new();
        };
        settings_path
            .iter()
            .chain(selected_theme_path.iter())
            .cloned()
            .collect()
    }

    pub fn warp_fallback_reason(&self) -> Option<&str> {
        match &self.status {
            ThemeStatus::Warp {
                fallback_reason, ..
            } => fallback_reason.as_deref(),
            ThemeStatus::BuiltIn | ThemeStatus::Terminal => None,
        }
    }
}

pub fn resolve_selection(
    selection: ThemeSelection,
    appearance: Option<SystemAppearance>,
) -> ResolvedTheme {
    match selection.clone() {
        ThemeSelection::BuiltIn(kind) => ResolvedTheme::built_in(kind, selection),
        ThemeSelection::TerminalNative => ResolvedTheme::terminal(
            selection,
            "Terminal Native",
            polarity_from_appearance(appearance),
        ),
        ThemeSelection::WarpSync => resolve_warp_sync(selection, appearance),
        ThemeSelection::WarpCatalog(id) => {
            let Some(entry) = catalog::find(&id) else {
                return warp_fallback(
                    selection,
                    None,
                    None,
                    Some(id),
                    false,
                    appearance,
                    "official Warp theme is missing or invalid",
                );
            };
            resolve_pinned(
                selection,
                entry.display_name.clone(),
                &entry.data,
                entry.content_hash,
                ThemeStatus::Warp {
                    channel: None,
                    selected_name: Some(entry.display_name.clone()),
                    settings_path: None,
                    selected_theme_path: None,
                    system_theme: false,
                    fallback_reason: None,
                },
            )
        }
        ThemeSelection::WarpFile(canonical) => {
            let Some(entry) = discovery::find_by_canonical(&canonical) else {
                return warp_fallback(
                    selection,
                    None,
                    None,
                    None,
                    false,
                    appearance,
                    "installed Warp theme is missing or invalid",
                );
            };
            resolve_pinned(
                selection,
                entry.display_name.clone(),
                &entry.data,
                entry.content_hash,
                ThemeStatus::Warp {
                    channel: Some(entry.installation.channel),
                    selected_name: Some(entry.display_name),
                    settings_path: Some(entry.installation.settings_path),
                    selected_theme_path: Some(entry.path),
                    system_theme: false,
                    fallback_reason: None,
                },
            )
        }
        ThemeSelection::Auto => {
            let kind = match appearance {
                Some(SystemAppearance::Light) => ThemeKind::GrokDay,
                Some(SystemAppearance::Dark) | None => ThemeKind::GrokNight,
            };
            ResolvedTheme::built_in(kind, selection)
        }
    }
}

fn resolve_pinned(
    selection: ThemeSelection,
    display_name: String,
    data: &super::warp::model::WarpThemeData,
    source_hash: [u8; 32],
    status: ThemeStatus,
) -> ResolvedTheme {
    if !super::color_support::detect_capability().has_truecolor() {
        let mut fallback = ResolvedTheme::terminal(
            selection,
            format!("{display_name} (Terminal Native fallback)"),
            if data.is_dark() {
                ThemePolarity::Dark
            } else {
                ThemePolarity::Light
            },
        );
        fallback.kind = ThemeKind::WarpCustom;
        fallback.status = match status {
            ThemeStatus::Warp {
                channel,
                selected_name,
                settings_path,
                selected_theme_path,
                system_theme,
                ..
            } => ThemeStatus::Warp {
                channel,
                selected_name,
                settings_path,
                selected_theme_path,
                system_theme,
                fallback_reason: Some("pinned Warp themes require truecolor".to_owned()),
            },
            other => other,
        };
        return fallback;
    }
    let translated = translate::translate_pinned(data);
    let cursor = data
        .cursor
        .map(|fill| fill.midpoint().to_color())
        .unwrap_or(translated.theme.accent_user);
    ResolvedTheme::finish(
        selection,
        ThemeKind::WarpCustom,
        display_name,
        translated.theme,
        if translated.dark {
            ThemePolarity::Dark
        } else {
            ThemePolarity::Light
        },
        ThemeRenderMode::Opaque,
        CursorPolicy::ThemeColor(cursor),
        syntax_policy_for_warp(data),
        status,
        Some(source_hash),
    )
}

fn resolve_warp_sync(
    selection: ThemeSelection,
    appearance: Option<SystemAppearance>,
) -> ResolvedTheme {
    let Some(installation) = settings::active_installation() else {
        return warp_fallback(
            selection,
            None,
            None,
            None,
            false,
            appearance,
            "Warp installation was not found",
        );
    };
    let parsed = match settings::read_settings(&installation.settings_path) {
        Ok(parsed) => parsed,
        Err(error) => {
            return warp_fallback(
                selection,
                Some(installation.channel),
                Some(installation.settings_path),
                None,
                false,
                appearance,
                &error.to_string(),
            );
        }
    };
    let Some(selected) = parsed.selected(appearance).cloned() else {
        return warp_fallback(
            selection,
            Some(installation.channel),
            Some(installation.settings_path),
            None,
            parsed.system_theme,
            appearance,
            "Warp settings do not contain a selected theme",
        );
    };

    let mut selected_path = None;
    let mut selected_data = None;
    let mut source_hash = None;
    let mut display_name = selected.name.clone();

    if let Some(path) = selected.path.as_deref()
        && let Some(path) = discovery::resolve_settings_path(&installation, path)
        && let Ok(theme) = discovery::load_discovered(&installation, &path)
    {
        display_name = theme.display_name;
        source_hash = Some(theme.content_hash);
        selected_data = Some(theme.data);
        selected_path = Some(path);
    }
    if selected_data.is_none()
        && let Some(theme) = catalog::find_by_warp_name(&selected.name)
    {
        display_name = theme.display_name.clone();
        source_hash = Some(theme.content_hash);
        selected_data = Some(theme.data.clone());
    }

    let translated = translate::translate_sync(selected_data.as_ref());
    let display = format!("Warp Sync — {display_name}");
    let polarity = if selected_data.is_some() {
        if translated.dark {
            ThemePolarity::Dark
        } else {
            ThemePolarity::Light
        }
    } else {
        polarity_from_appearance(appearance)
    };
    ResolvedTheme::finish(
        selection,
        ThemeKind::WarpSync,
        display,
        translated.theme,
        polarity,
        ThemeRenderMode::TerminalNative,
        CursorPolicy::TerminalDefault,
        xai_grok_markdown::SyntaxColorPolicy::NamedAnsi,
        ThemeStatus::Warp {
            channel: Some(installation.channel),
            selected_name: Some(display_name),
            settings_path: Some(installation.settings_path),
            selected_theme_path: selected_path,
            system_theme: parsed.system_theme,
            fallback_reason: selected_data
                .is_none()
                .then(|| "theme metadata unavailable; using terminal palette".to_owned()),
        },
        source_hash,
    )
}

fn warp_fallback(
    selection: ThemeSelection,
    channel: Option<settings::WarpChannel>,
    settings_path: Option<PathBuf>,
    selected_name: Option<String>,
    system_theme: bool,
    appearance: Option<SystemAppearance>,
    reason: &str,
) -> ResolvedTheme {
    let translated = translate::translate_sync(None);
    let (kind, display_name) = match selection {
        ThemeSelection::WarpCatalog(_) | ThemeSelection::WarpFile(_) => (
            ThemeKind::WarpCustom,
            "Unavailable Warp theme — Terminal Native".to_owned(),
        ),
        _ => (
            ThemeKind::WarpSync,
            "Warp Sync — Terminal Native".to_owned(),
        ),
    };
    ResolvedTheme::finish(
        selection,
        kind,
        display_name,
        translated.theme,
        polarity_from_appearance(appearance),
        ThemeRenderMode::TerminalNative,
        CursorPolicy::TerminalDefault,
        xai_grok_markdown::SyntaxColorPolicy::NamedAnsi,
        ThemeStatus::Warp {
            channel,
            selected_name,
            settings_path,
            selected_theme_path: None,
            system_theme,
            fallback_reason: Some(reason.to_owned()),
        },
        None,
    )
}

fn polarity_from_appearance(appearance: Option<SystemAppearance>) -> ThemePolarity {
    if matches!(appearance, Some(SystemAppearance::Light)) {
        ThemePolarity::Light
    } else {
        ThemePolarity::Dark
    }
}

fn syntax_policy_for_warp(
    data: &super::warp::model::WarpThemeData,
) -> xai_grok_markdown::SyntaxColorPolicy {
    let mut palette = [[0u8; 3]; 16];
    for (index, color) in data
        .terminal
        .normal
        .iter()
        .chain(data.terminal.bright.iter())
        .enumerate()
    {
        palette[index] = [color.r, color.g, color.b];
    }
    xai_grok_markdown::SyntaxColorPolicy::WarpPalette(palette)
}

fn theme_for_kind(kind: ThemeKind) -> Theme {
    match kind {
        ThemeKind::GrokNight => Theme::groknight(),
        ThemeKind::GrokDay => Theme::grokday(),
        ThemeKind::TokyoNight => Theme::tokyonight(),
        ThemeKind::RosePineMoon => Theme::rosepine_moon(),
        ThemeKind::OscuraMidnight => Theme::oscura_midnight(),
        ThemeKind::TerminalNative | ThemeKind::WarpSync | ThemeKind::WarpCustom => {
            Theme::terminal_default()
        }
        ThemeKind::Auto => Theme::groknight(),
    }
}

fn fingerprint(
    selection: &ThemeSelection,
    display_name: &str,
    theme: &Theme,
    polarity: ThemePolarity,
    render_mode: ThemeRenderMode,
    syntax_policy: xai_grok_markdown::SyntaxColorPolicy,
    source_hash: Option<[u8; 32]>,
) -> ThemeFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(format!("warp-theme-v2|{selection:?}|{display_name}|{theme:?}|{polarity:?}|{render_mode:?}|{syntax_policy:?}").as_bytes());
    if let Some(source_hash) = source_hash {
        hasher.update(&source_hash);
    }
    let hash = hasher.finalize();
    let mut fingerprint = [0; 16];
    fingerprint.copy_from_slice(&hash.as_bytes()[..16]);
    ThemeFingerprint(fingerprint)
}
