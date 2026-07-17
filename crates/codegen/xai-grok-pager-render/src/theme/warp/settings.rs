use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use toml::Value;

use crate::theme::system_appearance::SystemAppearance;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarpChannel {
    Stable,
    Preview,
    Oss,
    Dev,
}

impl WarpChannel {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Preview => "preview",
            Self::Oss => "oss",
            Self::Dev => "dev",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Preview => "Preview",
            Self::Oss => "OSS",
            Self::Dev => "Dev",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarpInstallation {
    pub channel: WarpChannel,
    pub settings_path: PathBuf,
    pub themes_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarpThemeRef {
    pub name: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct WarpThemeSettings {
    pub system_theme: bool,
    pub theme: Option<WarpThemeRef>,
    pub dark: Option<WarpThemeRef>,
    pub light: Option<WarpThemeRef>,
}

impl WarpThemeSettings {
    pub fn selected(&self, appearance: Option<SystemAppearance>) -> Option<&WarpThemeRef> {
        if self.system_theme {
            match appearance {
                Some(SystemAppearance::Light) => self.light.as_ref().or(self.theme.as_ref()),
                Some(SystemAppearance::Dark) | None => self.dark.as_ref().or(self.theme.as_ref()),
            }
        } else {
            self.theme.as_ref()
        }
    }
}

pub fn is_local_warp() -> bool {
    let brand_is_warp =
        env::var("TERM_PROGRAM").is_ok_and(|value| value.eq_ignore_ascii_case("WarpTerminal"));
    let warp_client_marker = env::var_os("WARP_CLIENT_VERSION").is_some();
    let local_marker = env::var("WARP_IS_LOCAL_SHELL_SESSION").ok();
    let remote_shell = ["SSH_CONNECTION", "SSH_TTY", "MOSH_IP"]
        .into_iter()
        .any(|key| env::var_os(key).is_some());
    is_local_warp_from_signals(
        brand_is_warp,
        warp_client_marker,
        local_marker.as_deref(),
        remote_shell,
    )
}

fn is_local_warp_from_signals(
    brand_is_warp: bool,
    warp_client_marker: bool,
    local_marker: Option<&str>,
    remote_shell: bool,
) -> bool {
    let recognized = brand_is_warp || warp_client_marker || local_marker.is_some();
    let local = match local_marker {
        Some(value) => value != "0",
        None => !remote_shell,
    };
    recognized && local
}

pub fn detected_channel() -> WarpChannel {
    let version = env::var("WARP_CLIENT_VERSION")
        .or_else(|_| env::var("TERM_PROGRAM_VERSION"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if version.contains("preview") {
        WarpChannel::Preview
    } else if version.contains("oss") {
        WarpChannel::Oss
    } else if version.contains("dev") || version.contains("local") {
        WarpChannel::Dev
    } else {
        WarpChannel::Stable
    }
}

pub fn installations() -> Vec<WarpInstallation> {
    installations_for(dirs::home_dir(), detected_channel())
}

fn installations_for(home: Option<PathBuf>, preferred: WarpChannel) -> Vec<WarpInstallation> {
    let mut installs = platform_installations(home);
    installs.sort_by_key(|install| u8::from(install.channel != preferred));
    installs
}

#[cfg(target_os = "macos")]
fn platform_installations(home: Option<PathBuf>) -> Vec<WarpInstallation> {
    let Some(home) = home else { return Vec::new() };
    [
        (WarpChannel::Stable, ".warp"),
        (WarpChannel::Preview, ".warp-preview"),
        (WarpChannel::Oss, ".warp-oss"),
        (WarpChannel::Dev, ".warp-dev"),
    ]
    .into_iter()
    .map(|(channel, dir)| {
        let root = home.join(dir);
        WarpInstallation {
            channel,
            settings_path: root.join("settings.toml"),
            themes_dir: root.join("themes"),
        }
    })
    .collect()
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn platform_installations(home: Option<PathBuf>) -> Vec<WarpInstallation> {
    let Some(home) = home else { return Vec::new() };
    let config = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let data = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"));
    [
        (WarpChannel::Stable, "warp-terminal"),
        (WarpChannel::Preview, "warp-terminal-preview"),
        (WarpChannel::Oss, "warp-oss"),
        (WarpChannel::Dev, "warp-terminal-dev"),
    ]
    .into_iter()
    .map(|(channel, app)| WarpInstallation {
        channel,
        settings_path: config.join(app).join("settings.toml"),
        themes_dir: data.join(app).join("themes"),
    })
    .collect()
}

#[cfg(target_os = "windows")]
fn platform_installations(_home: Option<PathBuf>) -> Vec<WarpInstallation> {
    let local = env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let roaming = env::var_os("APPDATA").map(PathBuf::from);
    [
        (WarpChannel::Stable, "Warp"),
        (WarpChannel::Preview, "Warp-Preview"),
        (WarpChannel::Oss, "WarpOss"),
        (WarpChannel::Dev, "Warp-Dev"),
    ]
    .into_iter()
    .filter_map(|(channel, app)| {
        let settings_root = local.as_ref()?.join("warp").join(app);
        let themes_root = roaming.as_ref()?.join("warp").join(app);
        Some(WarpInstallation {
            channel,
            settings_path: settings_root.join("settings.toml"),
            themes_dir: themes_root.join("themes"),
        })
    })
    .collect()
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "windows"
)))]
fn platform_installations(_home: Option<PathBuf>) -> Vec<WarpInstallation> {
    Vec::new()
}

pub fn active_installation() -> Option<WarpInstallation> {
    installations()
        .into_iter()
        .find(|install| install.settings_path.is_file() || install.themes_dir.is_dir())
        .or_else(|| installations().into_iter().next())
}

pub fn read_settings(path: &Path) -> Result<WarpThemeSettings> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("failed to inspect Warp settings {}", path.display()))?;
    if metadata.len() > 1024 * 1024 {
        return Err(anyhow!("Warp settings exceed 1 MiB limit"));
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read Warp settings {}", path.display()))?;
    if bytes.len() > 1024 * 1024 {
        return Err(anyhow!("Warp settings exceed 1 MiB limit"));
    }
    let text = std::str::from_utf8(&bytes).context("Warp settings are not valid UTF-8")?;
    let root: Value = toml::from_str(text).context("invalid Warp settings TOML")?;
    parse_settings(&root)
}

pub fn parse_settings(root: &Value) -> Result<WarpThemeSettings> {
    let themes = root
        .get("appearance")
        .and_then(|value| value.get("themes"))
        .ok_or_else(|| anyhow!("missing [appearance.themes]"))?;
    let system_theme = themes
        .get("system_theme")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let theme = themes.get("theme").and_then(parse_theme_ref);
    let selected = themes.get("selected_system_themes");
    let dark = selected
        .and_then(|value| value.get("dark"))
        .and_then(parse_theme_ref);
    let light = selected
        .and_then(|value| value.get("light"))
        .and_then(parse_theme_ref);
    Ok(WarpThemeSettings {
        system_theme,
        theme,
        dark,
        light,
    })
}

fn parse_theme_ref(value: &Value) -> Option<WarpThemeRef> {
    if let Some(name) = value.as_str() {
        return Some(WarpThemeRef {
            name: name.to_owned(),
            path: None,
        });
    }
    let table = value.as_table()?;
    if let (Some(name), path) = (
        table.get("name").and_then(Value::as_str),
        table.get("path").and_then(Value::as_str),
    ) {
        return Some(WarpThemeRef {
            name: name.to_owned(),
            path: path.map(PathBuf::from),
        });
    }
    for key in ["custom", "custom_base16", "Custom", "CustomBase16"] {
        if let Some(inner) = table.get(key)
            && let Some(theme) = parse_theme_ref(inner)
        {
            return Some(theme);
        }
    }
    if table.len() == 1 {
        table.values().next().and_then(parse_theme_ref)
    } else {
        None
    }
}

pub fn display_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(relative) = path.strip_prefix(home)
    {
        return format!("~/{}", relative.display());
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_settings_shape() {
        let value: Value = toml::from_str(
            r#"
            [appearance.themes]
            system_theme = false
            theme = "dracula"
            selected_system_themes = { dark = "gruvbox_dark", light = "gruvbox_light" }
            "#,
        )
        .unwrap();
        let parsed = parse_settings(&value).unwrap();
        assert_eq!(parsed.theme.unwrap().name, "dracula");
        assert_eq!(parsed.dark.unwrap().name, "gruvbox_dark");
        assert_eq!(parsed.light.unwrap().name, "gruvbox_light");
    }

    #[test]
    fn parses_custom_inline_shape() {
        let value: Value = toml::from_str(
            r#"
            [appearance.themes]
            theme = { custom = { name = "Mine", path = "mine.yaml" } }
            "#,
        )
        .unwrap();
        let parsed = parse_settings(&value).unwrap();
        let theme = parsed.theme.unwrap();
        assert_eq!(theme.name, "Mine");
        assert_eq!(theme.path.as_deref(), Some(Path::new("mine.yaml")));
    }

    #[test]
    fn system_theme_selects_the_active_bucket() {
        let settings = WarpThemeSettings {
            system_theme: true,
            theme: None,
            dark: Some(WarpThemeRef {
                name: "dark".to_owned(),
                path: None,
            }),
            light: Some(WarpThemeRef {
                name: "light".to_owned(),
                path: None,
            }),
        };
        assert_eq!(
            settings
                .selected(Some(SystemAppearance::Dark))
                .unwrap()
                .name,
            "dark"
        );
        assert_eq!(
            settings
                .selected(Some(SystemAppearance::Light))
                .unwrap()
                .name,
            "light"
        );
    }

    #[test]
    fn remote_shell_is_not_treated_as_local_without_explicit_marker() {
        assert!(!is_local_warp_from_signals(true, true, None, true));
        assert!(is_local_warp_from_signals(true, true, Some("1"), true));
        assert!(!is_local_warp_from_signals(true, true, Some("0"), false));
        assert!(is_local_warp_from_signals(true, false, None, false));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_installation_paths_cover_stable_and_preview() {
        let home = PathBuf::from("/tmp/warp-home");
        let installs = installations_for(Some(home.clone()), WarpChannel::Preview);
        assert_eq!(installs[0].channel, WarpChannel::Preview);
        assert_eq!(
            installs[0].settings_path,
            home.join(".warp-preview/settings.toml")
        );
        assert!(installs.iter().any(|install| {
            install.channel == WarpChannel::Stable
                && install.themes_dir == home.join(".warp/themes")
        }));
    }
}
