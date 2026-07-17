use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use super::model::WarpThemeData;
use super::parser::{MAX_THEME_BYTES, parse_theme};
use super::settings::{WarpInstallation, installations};

const MAX_DISCOVERY_DEPTH: usize = 8;
const MAX_DISCOVERED_THEMES: usize = 4096;

#[derive(Debug, Clone)]
pub struct DiscoveredTheme {
    pub canonical: String,
    pub display_name: String,
    pub installation: WarpInstallation,
    pub relative_path: PathBuf,
    pub path: PathBuf,
    pub data: WarpThemeData,
    pub content_hash: [u8; 32],
}

pub fn discover_all() -> Vec<DiscoveredTheme> {
    let mut themes = Vec::new();
    for installation in installations() {
        discover_installation(&installation, &mut themes);
        if themes.len() >= MAX_DISCOVERED_THEMES {
            break;
        }
    }
    themes.sort_by(|a, b| {
        a.display_name
            .to_ascii_lowercase()
            .cmp(&b.display_name.to_ascii_lowercase())
            .then_with(|| a.canonical.cmp(&b.canonical))
    });
    themes
}

fn discover_installation(installation: &WarpInstallation, out: &mut Vec<DiscoveredTheme>) {
    let root = &installation.themes_dir;
    let Ok(metadata) = fs::symlink_metadata(root) else {
        return;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return;
    }
    let mut stack = vec![(root.clone(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if out.len() >= MAX_DISCOVERED_THEMES || depth > MAX_DISCOVERY_DEPTH {
            return;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_dir() {
                if depth < MAX_DISCOVERY_DEPTH {
                    stack.push((path, depth + 1));
                }
                continue;
            }
            if !file_type.is_file() || !is_yaml(&path) {
                continue;
            }
            match load_discovered(installation, &path) {
                Ok(theme) => out.push(theme),
                Err(_error) => tracing::warn!(
                    channel = installation.channel.id(),
                    "invalid or unreadable installed Warp theme"
                ),
            }
            if out.len() >= MAX_DISCOVERED_THEMES {
                return;
            }
        }
    }
}

pub fn load_discovered(installation: &WarpInstallation, path: &Path) -> Result<DiscoveredTheme> {
    let relative = path
        .strip_prefix(&installation.themes_dir)
        .context("theme path is outside the Warp themes directory")?;
    validate_relative(relative)?;
    reject_symlink_components(&installation.themes_dir, relative)?;
    let metadata =
        fs::metadata(path).with_context(|| format!("failed to inspect {}", path.display()))?;
    if !metadata.is_file() {
        bail!("theme path is not a regular file");
    }
    if metadata.len() > MAX_THEME_BYTES as u64 {
        bail!("theme exceeds {MAX_THEME_BYTES} byte limit");
    }
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    // Recheck after the read to cover a file growing between metadata and I/O.
    if bytes.len() > MAX_THEME_BYTES {
        bail!("theme exceeds {MAX_THEME_BYTES} byte limit");
    }
    let yaml = std::str::from_utf8(&bytes).context("theme YAML is not UTF-8")?;
    let data = parse_theme(yaml)?;
    let display_name = data.name.clone().unwrap_or_else(|| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(humanize)
            .unwrap_or_else(|| "Warp Theme".to_owned())
    });
    let canonical = format!(
        "warp-file:{}/{}",
        installation.channel.id(),
        encode_relative(relative)
    );
    Ok(DiscoveredTheme {
        canonical,
        display_name,
        installation: installation.clone(),
        relative_path: relative.to_owned(),
        path: path.to_owned(),
        data,
        content_hash: *blake3::hash(&bytes).as_bytes(),
    })
}

pub fn find_by_canonical(canonical: &str) -> Option<DiscoveredTheme> {
    let rest = canonical.strip_prefix("warp-file:")?;
    let (channel, encoded) = rest.split_once('/')?;
    let relative = decode_relative(encoded).ok()?;
    validate_relative(&relative).ok()?;
    installations()
        .into_iter()
        .find(|installation| installation.channel.id() == channel)
        .and_then(|installation| {
            let path = installation.themes_dir.join(&relative);
            load_discovered(&installation, &path).ok()
        })
}

pub fn resolve_settings_path(installation: &WarpInstallation, path: &Path) -> Option<PathBuf> {
    let candidate = if path.is_absolute() {
        path.to_owned()
    } else {
        installation.themes_dir.join(path)
    };
    let relative = candidate.strip_prefix(&installation.themes_dir).ok()?;
    validate_relative(relative).ok()?;
    Some(candidate)
}

fn reject_symlink_components(root: &Path, relative: &Path) -> Result<()> {
    let root_metadata = fs::symlink_metadata(root)
        .with_context(|| format!("failed to inspect {}", root.display()))?;
    if root_metadata.file_type().is_symlink() {
        bail!("Warp themes directory is a symbolic link");
    }

    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            bail!("theme path contains traversal or platform prefix");
        };
        current.push(component);
        let metadata = fs::symlink_metadata(&current)
            .with_context(|| format!("failed to inspect {}", current.display()))?;
        if metadata.file_type().is_symlink() {
            bail!("theme path contains a symbolic link");
        }
    }
    Ok(())
}

fn validate_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!("theme path must be a non-empty relative path");
    }
    if !path
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        bail!("theme path contains traversal or platform prefix");
    }
    Ok(())
}

fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "yaml" | "yml"))
}

fn humanize(stem: &str) -> String {
    stem.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect())
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn encode_relative(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'/') {
            out.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

fn decode_relative(value: &str) -> Result<PathBuf> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(anyhow!("truncated percent escape"));
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3])?;
            decoded.push(u8::from_str_radix(hex, 16)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    let string = String::from_utf8(decoded)?;
    Ok(string.split('/').collect::<PathBuf>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installation(themes_dir: PathBuf) -> WarpInstallation {
        WarpInstallation {
            channel: super::super::settings::WarpChannel::Stable,
            settings_path: themes_dir.join("../settings.toml"),
            themes_dir,
        }
    }

    #[test]
    fn path_encoding_round_trips() {
        let path = Path::new("nested/My theme #1.yaml");
        let encoded = encode_relative(path);
        assert_eq!(decode_relative(&encoded).unwrap(), path);
    }

    #[test]
    fn traversal_is_rejected() {
        assert!(validate_relative(Path::new("../secret.yaml")).is_err());
        assert!(validate_relative(Path::new("/tmp/theme.yaml")).is_err());
        let encoded = decode_relative("%2E%2E/secret.yaml").unwrap();
        assert!(validate_relative(&encoded).is_err());
    }

    #[test]
    fn oversized_theme_is_rejected_before_parsing() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("themes");
        fs::create_dir(&root).unwrap();
        let path = root.join("huge.yaml");
        fs::write(&path, vec![b' '; MAX_THEME_BYTES + 1]).unwrap();
        let error = load_discovered(&installation(root), &path).unwrap_err();
        assert!(error.to_string().contains("byte limit"));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_theme_file_is_rejected() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("themes");
        fs::create_dir(&root).unwrap();
        let outside = temp.path().join("outside.yaml");
        fs::write(&outside, "not a theme").unwrap();
        let linked = root.join("linked.yaml");
        symlink(&outside, &linked).unwrap();

        let error = load_discovered(&installation(root), &linked).unwrap_err();
        assert!(error.to_string().contains("symbolic link"));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_theme_root_is_rejected() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let real_root = temp.path().join("real-themes");
        fs::create_dir(&real_root).unwrap();
        fs::write(real_root.join("theme.yaml"), "not a theme").unwrap();
        let linked_root = temp.path().join("themes");
        symlink(&real_root, &linked_root).unwrap();
        let path = linked_root.join("theme.yaml");

        let error = load_discovered(&installation(linked_root), &path).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("themes directory is a symbolic link")
        );
    }
}
