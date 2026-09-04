//! Runtime detection for the Ghostty/Herdr terminal-native theme sync.
//!
//! Ghostty owns the effective foreground, background, and ANSI palette after
//! merging its selected theme, included config files, and explicit overrides.
//! Herdr's Ghostty-backed pane runtime exposes that same palette to child
//! processes.  Following the runtime palette is therefore both more accurate
//! and safer than reparsing either application's configuration files.

use std::collections::HashMap;

/// Runtime that owns the palette followed by `ghostty-sync`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostThemeSource {
    Herdr,
    Ghostty,
}

impl HostThemeSource {
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Herdr => "Herdr / Ghostty Sync",
            Self::Ghostty => "Ghostty Sync",
        }
    }
}

/// Detect a Ghostty terminal or a Herdr-managed pane from the process
/// environment. Herdr wins because an outer Ghostty marker may be inherited
/// while Herdr is the immediate palette-owning terminal engine.
pub fn detect() -> Option<HostThemeSource> {
    detect_from_env(&crate::host::collect_unicode_env())
}

pub fn is_active() -> bool {
    detect().is_some()
}

fn detect_from_env(env: &HashMap<String, String>) -> Option<HostThemeSource> {
    if nonempty(env, "HERDR_ENV").is_some() {
        return Some(HostThemeSource::Herdr);
    }

    let term_program_is_ghostty =
        nonempty(env, "TERM_PROGRAM").is_some_and(|value| value.eq_ignore_ascii_case("ghostty"));
    let term_is_ghostty =
        nonempty(env, "TERM").is_some_and(|value| value.eq_ignore_ascii_case("xterm-ghostty"));
    let has_resources_marker = nonempty(env, "GHOSTTY_RESOURCES_DIR").is_some();
    (term_program_is_ghostty || term_is_ghostty || has_resources_marker)
        .then_some(HostThemeSource::Ghostty)
}

fn nonempty<'a>(env: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    env.get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn herdr_marker_wins_over_inherited_ghostty_identity() {
        let env = env(&[
            ("HERDR_ENV", "1"),
            ("TERM_PROGRAM", "ghostty"),
            ("TERM", "xterm-ghostty"),
        ]);
        assert_eq!(detect_from_env(&env), Some(HostThemeSource::Herdr));
    }

    #[test]
    fn recognizes_ghostty_stable_runtime_markers() {
        for env in [
            env(&[("TERM_PROGRAM", "ghostty")]),
            env(&[("TERM_PROGRAM", "Ghostty")]),
            env(&[("TERM", "xterm-ghostty")]),
            env(&[("GHOSTTY_RESOURCES_DIR", "/opt/ghostty")]),
        ] {
            assert_eq!(detect_from_env(&env), Some(HostThemeSource::Ghostty));
        }
    }

    #[test]
    fn empty_or_unrelated_markers_do_not_enable_sync() {
        assert_eq!(detect_from_env(&HashMap::new()), None);
        assert_eq!(detect_from_env(&env(&[("HERDR_ENV", "")])), None);
        assert_eq!(
            detect_from_env(&env(&[("TERM_PROGRAM", "WarpTerminal")])),
            None
        );
        assert_eq!(detect_from_env(&env(&[("TERM", "xterm-256color")])), None);
    }
}
