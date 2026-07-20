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

/// Map a syntax token to a dual-polarity-safe terminal-native foreground.
///
/// Near-gray body/comment colors delegate to the host terminal foreground. Chromatic tokens use
/// only the six base ANSI accents, never black, white, gray, or bright variants that can vanish
/// against the opposite terminal background polarity.
fn polarity_safe_syntax_fg(r: u8, g: u8, b: u8) -> ratatui::style::Color {
    use ratatui::style::Color;

    let max = r.max(g).max(b) as i32;
    let min = r.min(g).min(b) as i32;
    let chroma = max - min;
    if chroma < 40 {
        return Color::Reset;
    }

    let (r, g, b) = (r as i32, g as i32, b as i32);
    let hue = if max == r {
        let mut hue = (g - b) * 60 / chroma;
        if hue < 0 {
            hue += 360;
        }
        hue
    } else if max == g {
        (b - r) * 60 / chroma + 120
    } else {
        (r - g) * 60 / chroma + 240
    };

    // Start magenta at 255° so Tokyo Night purple (~261°) stays magenta while pure blues stay blue.
    match hue {
        0..30 | 330..=360 => Color::Red,
        30..90 => Color::Yellow,
        90..150 => Color::Green,
        150..210 => Color::Cyan,
        210..255 => Color::Blue,
        _ => Color::Magenta,
    }
}

/// Convert syntect style to ratatui foreground-only style, quantized for terminal color support.
pub fn syntect_to_ratatui_fg(style: syntect::highlighting::Style) -> ratatui::style::Style {
    let fg = if crate::theme::cache::active_terminal_native() {
        polarity_safe_syntax_fg(style.foreground.r, style.foreground.g, style.foreground.b)
    } else {
        let mapped = xai_grok_markdown::map_syntect_style(style);
        match mapped.get_fg_color() {
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
        }
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
    use super::*;
    use ratatui::style::Color;

    const CANONICAL_MARKERS: [&str; 16] = [
        "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
        "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#ffffff",
    ];

    fn with_terminal_native<R>(run: impl FnOnce() -> R) -> R {
        let _guard = crate::theme::cache::test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        crate::theme::cache::reset_for_test();
        crate::theme::cache::set_terminal_native_lock(true);
        let result = run();
        crate::theme::cache::reset_for_test();
        result
    }

    #[test]
    fn terminal_native_grays_delegate_to_the_host_foreground() {
        for rgb in [
            (0, 0, 0),
            (0x66, 0x66, 0x66),
            (0xe5, 0xe5, 0xe5),
            (255, 255, 255),
        ] {
            assert_eq!(polarity_safe_syntax_fg(rgb.0, rgb.1, rgb.2), Color::Reset);
        }
    }

    #[test]
    fn terminal_native_chromatic_tokens_use_only_base_ansi_accents() {
        let samples = [
            (0xcd, 0x31, 0x31),
            (0x0d, 0xbc, 0x79),
            (0xe5, 0xe5, 0x10),
            (0x24, 0x72, 0xc8),
            (0xbc, 0x3f, 0xbc),
            (0x11, 0xa8, 0xcd),
            (0xf1, 0x4c, 0x4c),
            (0x3b, 0x8e, 0xea),
        ];
        for (r, g, b) in samples {
            assert!(matches!(
                polarity_safe_syntax_fg(r, g, b),
                Color::Red
                    | Color::Green
                    | Color::Yellow
                    | Color::Blue
                    | Color::Magenta
                    | Color::Cyan
            ));
        }
        assert_eq!(polarity_safe_syntax_fg(0xf1, 0x4c, 0x4c), Color::Red);
        assert_eq!(polarity_safe_syntax_fg(0xbb, 0x9a, 0xf7), Color::Magenta);
    }

    #[test]
    fn syntect_mapping_applies_polarity_safety_when_terminal_native_is_active() {
        with_terminal_native(|| {
            let gray = syntect::highlighting::Style {
                foreground: syntect::highlighting::Color {
                    r: 0xe5,
                    g: 0xe5,
                    b: 0xe5,
                    a: 0xff,
                },
                background: syntect::highlighting::Color::BLACK,
                font_style: syntect::highlighting::FontStyle::empty(),
            };
            let accent = syntect::highlighting::Style {
                foreground: syntect::highlighting::Color {
                    r: 0xf1,
                    g: 0x4c,
                    b: 0x4c,
                    a: 0xff,
                },
                ..gray
            };
            assert_eq!(syntect_to_ratatui_fg(gray).fg, Some(Color::Reset));
            assert!(matches!(
                syntect_to_ratatui_fg(accent).fg,
                Some(Color::Red | Color::Reset)
            ));
        });
    }

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
