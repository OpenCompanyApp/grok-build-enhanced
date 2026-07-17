use std::borrow::Cow;

use super::ThemeKind;
use super::warp::catalog;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThemeSelection {
    BuiltIn(ThemeKind),
    Auto,
    TerminalNative,
    WarpSync,
    WarpCatalog(String),
    WarpFile(String),
}

impl ThemeSelection {
    pub fn from_name(name: &str) -> Option<Self> {
        let trimmed = name.trim();
        let lower = trimmed.to_ascii_lowercase();
        match lower.as_str() {
            "auto" | "system" => return Some(Self::Auto),
            "terminal" | "terminal-native" | "native" => return Some(Self::TerminalNative),
            "warp" | "warp-sync" => return Some(Self::WarpSync),
            _ => {}
        }
        if let Some(id) = lower.strip_prefix("warp:") {
            return catalog::find(id).map(|_| Self::WarpCatalog(id.to_owned()));
        }
        if lower.starts_with("warp-file:")
            && lower.len() > "warp-file:".len()
            && !lower.contains("..")
            && !lower.contains('\\')
        {
            let rest = &trimmed["warp-file:".len()..];
            let (channel, path) = rest.split_once('/')?;
            if channel.is_empty() || path.is_empty() || path.starts_with('/') {
                return None;
            }
            return Some(Self::WarpFile(format!(
                "warp-file:{}/{}",
                channel.to_ascii_lowercase(),
                path
            )));
        }
        let kind = ThemeKind::from_name(&lower)?;
        if kind.is_auto() {
            Some(Self::Auto)
        } else {
            Some(Self::BuiltIn(kind))
        }
    }

    pub fn canonical(&self) -> Cow<'_, str> {
        match self {
            Self::BuiltIn(kind) => Cow::Borrowed(kind.display_name()),
            Self::Auto => Cow::Borrowed("auto"),
            Self::TerminalNative => Cow::Borrowed("terminal"),
            Self::WarpSync => Cow::Borrowed("warp-sync"),
            Self::WarpCatalog(id) => Cow::Owned(format!("warp:{id}")),
            Self::WarpFile(value) => Cow::Borrowed(value),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::BuiltIn(kind) => {
                super::display_name_for_canonical(kind.display_name()).to_owned()
            }
            Self::Auto => "Auto".to_owned(),
            Self::TerminalNative => "Terminal Native".to_owned(),
            Self::WarpSync => "Warp Sync".to_owned(),
            Self::WarpCatalog(id) => catalog::find(id)
                .map(|theme| theme.display_name.clone())
                .unwrap_or_else(|| id.clone()),
            Self::WarpFile(value) => super::warp::discovery::find_by_canonical(value)
                .map(|theme| theme.display_name)
                .unwrap_or_else(|| "Unavailable Warp theme".to_owned()),
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    pub fn is_warp_sync(&self) -> bool {
        matches!(self, Self::WarpSync)
    }

    pub fn is_terminal_native(&self) -> bool {
        matches!(self, Self::TerminalNative | Self::WarpSync)
    }

    pub fn is_concrete_for_auto(&self) -> bool {
        !matches!(self, Self::Auto | Self::WarpSync)
    }

    pub fn requires_truecolor(&self) -> bool {
        match self {
            Self::BuiltIn(kind) => kind.requires_truecolor(),
            Self::WarpCatalog(_) | Self::WarpFile(_) => true,
            Self::Auto | Self::TerminalNative | Self::WarpSync => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeChoice {
    pub canonical: String,
    pub display: String,
    pub description: String,
}

pub fn theme_choices(include_meta: bool) -> Vec<ThemeChoice> {
    let mut choices = Vec::new();
    if include_meta {
        choices.push(ThemeChoice {
            canonical: "auto".to_owned(),
            display: "Auto".to_owned(),
            description: "Follow system dark/light appearance.".to_owned(),
        });
        let resolved = super::cache::current_resolved();
        let warp_display = if matches!(resolved.selection, ThemeSelection::WarpSync) {
            resolved.display_name.clone()
        } else {
            "Warp Sync".to_owned()
        };
        choices.push(ThemeChoice {
            canonical: "warp-sync".to_owned(),
            display: warp_display,
            description: "Follow Warp's active theme and preserve its canvas.".to_owned(),
        });
    }
    choices.push(ThemeChoice {
        canonical: "terminal".to_owned(),
        display: "Terminal Native".to_owned(),
        description: "Use the terminal's foreground, background, and ANSI palette.".to_owned(),
    });
    choices.extend(ThemeKind::available().iter().map(|kind| ThemeChoice {
        canonical: kind.display_name().to_owned(),
        display: super::display_name_for_canonical(kind.display_name()).to_owned(),
        description: if kind.requires_truecolor() {
            "Built in · truecolor".to_owned()
        } else {
            "Built in".to_owned()
        },
    }));

    let installed = super::warp::discovery::discover_all();
    if let ThemeSelection::WarpFile(canonical) = super::cache::current_selection()
        && !installed.iter().any(|theme| theme.canonical == canonical)
    {
        choices.push(ThemeChoice {
            canonical,
            display: "Unavailable Warp theme".to_owned(),
            description: "The installed Warp YAML file is missing or invalid; the saved selection is preserved.".to_owned(),
        });
    }
    choices.extend(installed.into_iter().map(|theme| {
        let mut attributes = vec![format!(
            "Warp {} · installed",
            theme.installation.channel.display_name()
        )];
        if theme.data.is_gradient() {
            attributes.push("gradient flattens when pinned".to_owned());
        }
        if theme.data.has_background_image {
            attributes.push("image omitted when pinned".to_owned());
        }
        ThemeChoice {
            canonical: theme.canonical,
            display: theme.display_name,
            description: attributes.join(" · "),
        }
    }));

    let mut official = super::warp::catalog::all()
        .iter()
        .map(|theme| {
            let mut attributes = vec![format!("Warp official · {}", theme.category)];
            attributes.push(
                if theme.data.is_dark() {
                    "dark"
                } else {
                    "light"
                }
                .to_owned(),
            );
            if theme.data.is_gradient() {
                attributes.push("gradient flattens when pinned".to_owned());
            }
            if theme.data.has_background_image {
                attributes.push("image omitted when pinned".to_owned());
            }
            ThemeChoice {
                canonical: format!("warp:{}", theme.id),
                display: theme.display_name.clone(),
                description: attributes.join(" · "),
            }
        })
        .collect::<Vec<_>>();
    official.sort_by(|a, b| {
        a.display
            .to_ascii_lowercase()
            .cmp(&b.display.to_ascii_lowercase())
            .then_with(|| a.canonical.cmp(&b.canonical))
    });
    choices.extend(official);
    choices
}

impl std::str::FromStr for ThemeSelection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_name(s).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_new_meta_themes_and_aliases() {
        assert_eq!(
            ThemeSelection::from_name("native"),
            Some(ThemeSelection::TerminalNative)
        );
        assert_eq!(
            ThemeSelection::from_name("warp"),
            Some(ThemeSelection::WarpSync)
        );
        assert_eq!(
            ThemeSelection::from_name("SYSTEM"),
            Some(ThemeSelection::Auto)
        );
    }

    #[test]
    fn catalog_round_trip() {
        let selection = ThemeSelection::from_name("warp:standard/dracula").unwrap();
        assert_eq!(selection.canonical(), "warp:standard/dracula");
    }
}
