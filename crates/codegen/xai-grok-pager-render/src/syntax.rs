//! Syntax highlighting initialization.
//!
//! Provides lazily-initialized `Syntect` instances for code highlighting.
//! Dark themes (GrokNight, TokyoNight) share `grok-night.tmTheme`;
//! GrokDay uses `grok-day.tmTheme` with deepened colors for light backgrounds.

use std::sync::OnceLock;

pub use xai_grok_markdown::Syntect;

use crate::theme::ThemeKind;

static SYNTECT_GROKNIGHT: OnceLock<Syntect> = OnceLock::new();
static SYNTECT_TOKYONIGHT: OnceLock<Syntect> = OnceLock::new();
static SYNTECT_GROKDAY: OnceLock<Syntect> = OnceLock::new();
static SYNTECT_TERMINAL_ANSI: OnceLock<Syntect> = OnceLock::new();

/// Convert syntect style to ratatui foreground-only style, quantized for terminal color support.
pub fn syntect_to_ratatui_fg(style: syntect::highlighting::Style) -> ratatui::style::Style {
    let mapped = xai_grok_markdown::map_syntect_style(style);
    let fg = match mapped.get_fg_color() {
        Some(anstyle::Color::Ansi(color)) => match color {
            anstyle::AnsiColor::Black => ratatui::style::Color::Black,
            anstyle::AnsiColor::Red => ratatui::style::Color::Red,
            anstyle::AnsiColor::Green => ratatui::style::Color::Green,
            anstyle::AnsiColor::Yellow => ratatui::style::Color::Yellow,
            anstyle::AnsiColor::Blue => ratatui::style::Color::Blue,
            anstyle::AnsiColor::Magenta => ratatui::style::Color::Magenta,
            anstyle::AnsiColor::Cyan => ratatui::style::Color::Cyan,
            anstyle::AnsiColor::White => ratatui::style::Color::Gray,
            anstyle::AnsiColor::BrightBlack => ratatui::style::Color::DarkGray,
            anstyle::AnsiColor::BrightRed => ratatui::style::Color::LightRed,
            anstyle::AnsiColor::BrightGreen => ratatui::style::Color::LightGreen,
            anstyle::AnsiColor::BrightYellow => ratatui::style::Color::LightYellow,
            anstyle::AnsiColor::BrightBlue => ratatui::style::Color::LightBlue,
            anstyle::AnsiColor::BrightMagenta => ratatui::style::Color::LightMagenta,
            anstyle::AnsiColor::BrightCyan => ratatui::style::Color::LightCyan,
            anstyle::AnsiColor::BrightWhite => ratatui::style::Color::White,
        },
        Some(anstyle::Color::Ansi256(index)) => ratatui::style::Color::Indexed(index.index()),
        Some(anstyle::Color::Rgb(rgb)) => ratatui::style::Color::Rgb(rgb.0, rgb.1, rgb.2),
        None => ratatui::style::Color::Reset,
    };
    let mut out = ratatui::style::Style::default().fg(crate::theme::quantize(fg));
    use syntect::highlighting::FontStyle;
    if style.font_style.contains(FontStyle::BOLD) {
        out = out.add_modifier(ratatui::style::Modifier::BOLD);
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        out = out.add_modifier(ratatui::style::Modifier::ITALIC);
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        out = out.add_modifier(ratatui::style::Modifier::UNDERLINED);
    }
    out
}

/// Highlight a single line of source, falling back to plain text style.
pub fn highlight_line(
    text: &str,
    highlighter: &mut Option<syntect::easy::HighlightLines<'_>>,
    syntect: &Syntect,
    fallback: ratatui::style::Style,
) -> Vec<ratatui::text::Span<'static>> {
    if let Some(hl) = highlighter.as_mut()
        && let Ok(ranges) = hl.highlight_line(&format!("{text}\n"), &syntect.syntax_set)
    {
        let mut spans = Vec::new();
        for (style, segment) in ranges {
            let mut s = segment.to_owned();
            while s.ends_with('\n') || s.ends_with('\r') {
                s.pop();
            }
            if s.is_empty() {
                continue;
            }
            spans.push(ratatui::text::Span::styled(s, syntect_to_ratatui_fg(style)));
        }
        if !spans.is_empty() {
            return spans;
        }
    }
    vec![ratatui::text::Span::styled(text.to_string(), fallback)]
}

/// Returns the syntect instance matching the active theme.
pub fn get_syntect() -> &'static Syntect {
    if crate::theme::cache::active_terminal_native() {
        return SYNTECT_TERMINAL_ANSI
            .get_or_init(|| Syntect::new(include_bytes!("../assets/terminal-ansi.tmTheme")));
    }
    match crate::theme::Theme::current_kind() {
        ThemeKind::GrokNight
        | ThemeKind::RosePineMoon
        | ThemeKind::OscuraMidnight
        | ThemeKind::Auto => SYNTECT_GROKNIGHT
            .get_or_init(|| Syntect::new(include_bytes!("../assets/grok-night.tmTheme"))),
        ThemeKind::TokyoNight => SYNTECT_TOKYONIGHT
            .get_or_init(|| Syntect::new(include_bytes!("../assets/tokyo-night.tmTheme"))),
        ThemeKind::GrokDay => SYNTECT_GROKDAY
            .get_or_init(|| Syntect::new(include_bytes!("../assets/grok-day.tmTheme"))),
        ThemeKind::TerminalNative | ThemeKind::WarpSync | ThemeKind::WarpCustom => {
            SYNTECT_TERMINAL_ANSI
                .get_or_init(|| Syntect::new(include_bytes!("../assets/terminal-ansi.tmTheme")))
        }
    }
}

#[cfg(test)]
mod tests {
    const CANONICAL_MARKERS: [&str; 16] = [
        "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
        "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#ffffff",
    ];

    #[test]
    fn terminal_ansi_theme_uses_only_canonical_marker_values() {
        let theme = include_str!("../assets/terminal-ansi.tmTheme");
        let color_entry =
            regex::Regex::new(r"(?s)<key>(foreground|background)</key>\s*<string>([^<]*)</string>")
                .unwrap();
        let expected_entry_count = ["foreground", "background"]
            .into_iter()
            .map(|key| {
                let marker = format!("<key>{key}</key>");
                theme.matches(marker.as_str()).count()
            })
            .sum::<usize>();
        let entries = color_entry.captures_iter(theme).collect::<Vec<_>>();

        assert!(
            expected_entry_count > 0,
            "terminal-ansi.tmTheme contains no foreground/background entries"
        );
        assert_eq!(
            entries.len(),
            expected_entry_count,
            "every foreground/background key must contain one plain string value"
        );
        for entry in entries {
            let key = &entry[1];
            let value = entry[2].trim();
            assert!(
                CANONICAL_MARKERS.contains(&value),
                "terminal-ansi.tmTheme {key} value {value:?} is not a canonical lowercase #rrggbb marker"
            );
        }
    }
}
