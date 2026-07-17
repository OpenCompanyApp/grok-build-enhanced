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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeChoice {
    pub canonical: String,
    pub display: String,
    pub description: String,
}

/// Discover runtime inputs, then delegate choice construction to the pure
/// deterministic helper below.
pub fn theme_choices(include_meta: bool) -> Vec<ThemeChoice> {
    let warp_sync_display = if include_meta {
        let resolved = super::cache::current_resolved();
        matches!(resolved.selection, ThemeSelection::WarpSync)
            .then(|| resolved.display_name.clone())
    } else {
        None
    };
    let available = ThemeKind::available();
    let installed = super::warp::discovery::discover_all();
    let current_selection = super::cache::current_selection();
    let official = super::warp::catalog::all();
    construct_theme_choices(
        include_meta,
        available,
        &installed,
        official,
        &current_selection,
        warp_sync_display.as_deref(),
    )
}

/// Construct the picker catalog from already-resolved inputs. This helper does
/// no filesystem discovery and reads no process-global theme/cache state.
fn construct_theme_choices(
    include_meta: bool,
    available: &[ThemeKind],
    installed: &[super::warp::discovery::DiscoveredTheme],
    official: &[super::warp::catalog::CatalogTheme],
    current_selection: &ThemeSelection,
    warp_sync_display: Option<&str>,
) -> Vec<ThemeChoice> {
    let mut choices = Vec::new();
    if include_meta {
        choices.push(ThemeChoice {
            canonical: "auto".to_owned(),
            display: "Auto".to_owned(),
            description: "Follow system dark/light appearance.".to_owned(),
        });
        let warp_display = warp_sync_display.unwrap_or("Warp Sync").to_owned();
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
    choices.extend(available.iter().map(|kind| ThemeChoice {
        canonical: kind.display_name().to_owned(),
        display: super::display_name_for_canonical(kind.display_name()).to_owned(),
        description: if kind.requires_truecolor() {
            "Built in · truecolor".to_owned()
        } else {
            "Built in".to_owned()
        },
    }));

    if let ThemeSelection::WarpFile(canonical) = current_selection
        && !installed
            .iter()
            .any(|theme| theme.canonical == canonical.as_str())
    {
        choices.push(ThemeChoice {
            canonical: canonical.clone(),
            display: "Unavailable Warp theme".to_owned(),
            description: "The installed Warp YAML file is missing or invalid; the saved selection is preserved.".to_owned(),
        });
    }
    choices.extend(installed.iter().map(|theme| {
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
            canonical: theme.canonical.clone(),
            display: theme.display_name.clone(),
            description: attributes.join(" · "),
        }
    }));

    let mut official = official
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

    fn deterministic_choices(include_meta: bool) -> Vec<ThemeChoice> {
        construct_theme_choices(
            include_meta,
            ThemeKind::ALL,
            &[],
            catalog::all(),
            &ThemeSelection::BuiltIn(ThemeKind::GrokNight),
            None,
        )
    }

    #[test]
    fn missing_selected_warp_file_is_preserved_as_unavailable_choice() {
        let canonical = "warp-file:stable/themes/missing.yaml";
        let choices = construct_theme_choices(
            true,
            ThemeKind::ALL,
            &[],
            catalog::all(),
            &ThemeSelection::WarpFile(canonical.to_owned()),
            None,
        );
        let unavailable = choices
            .iter()
            .filter(|choice| choice.canonical == canonical)
            .collect::<Vec<_>>();

        assert_eq!(
            unavailable.len(),
            1,
            "saved choice must appear exactly once"
        );
        assert_eq!(unavailable[0].display, "Unavailable Warp theme");
        assert_eq!(
            unavailable[0].description,
            "The installed Warp YAML file is missing or invalid; the saved selection is preserved."
        );
    }

    #[test]
    fn meta_theme_choice_metadata_is_stable() {
        let choices = deterministic_choices(true);
        let choice = |canonical: &str| {
            choices
                .iter()
                .find(|choice| choice.canonical == canonical)
                .unwrap_or_else(|| panic!("missing `{canonical}` theme choice"))
        };

        assert_eq!(
            choice("auto"),
            &ThemeChoice {
                canonical: "auto".to_owned(),
                display: "Auto".to_owned(),
                description: "Follow system dark/light appearance.".to_owned(),
            }
        );
        assert_eq!(
            choice("terminal"),
            &ThemeChoice {
                canonical: "terminal".to_owned(),
                display: "Terminal Native".to_owned(),
                description: "Use the terminal's foreground, background, and ANSI palette."
                    .to_owned(),
            }
        );
        assert_eq!(
            choice("warp-sync"),
            &ThemeChoice {
                canonical: "warp-sync".to_owned(),
                display: "Warp Sync".to_owned(),
                description: "Follow Warp's active theme and preserve its canvas.".to_owned(),
            }
        );
    }

    #[test]
    fn deterministic_catalog_has_exactly_340_unique_official_warp_choices() {
        use std::collections::HashSet;

        let first = deterministic_choices(true);
        let second = deterministic_choices(true);
        assert_eq!(
            first, second,
            "identical inputs must preserve order and metadata"
        );

        let official = first
            .iter()
            .filter(|choice| choice.canonical.starts_with("warp:"))
            .collect::<Vec<_>>();
        let unique = official
            .iter()
            .map(|choice| choice.canonical.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(official.len(), 340);
        assert_eq!(unique.len(), 340, "official Warp canonicals must be unique");
    }

    #[test]
    fn concrete_choices_exclude_meta_themes() {
        let choices = deterministic_choices(false);
        assert!(!choices.iter().any(|choice| choice.canonical == "auto"));
        assert!(!choices.iter().any(|choice| choice.canonical == "warp-sync"));
        assert!(choices.iter().any(|choice| choice.canonical == "terminal"));
    }
}
