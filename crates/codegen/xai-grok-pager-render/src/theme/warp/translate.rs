use ratatui::style::Color;

use crate::theme::Theme;

use super::model::{Rgb, WarpThemeData};

#[derive(Debug, Clone, Copy)]
pub struct TranslatedWarpTheme {
    pub theme: Theme,
    pub dark: bool,
}

pub fn translate_sync(data: Option<&WarpThemeData>) -> TranslatedWarpTheme {
    let mut theme = Theme::terminal_default();
    let dark = data.is_none_or(WarpThemeData::is_dark);
    if let Some(data) = data {
        let background = data.background.midpoint();
        let accent = data.accent.midpoint();
        if contrast_ratio(accent, background) >= 3.0 {
            let accent = accent.to_color();
            theme.selection_border = accent;
            theme.prompt_border_active = accent;
            theme.fuzzy_accent = accent;
        }
    }
    TranslatedWarpTheme { theme, dark }
}

pub fn translate_pinned(data: &WarpThemeData) -> TranslatedWarpTheme {
    let bg = data.background.midpoint();
    let fg = ensure_contrast(data.foreground, bg, 4.5, data.foreground);
    let accent = ensure_contrast(data.accent.midpoint(), bg, 3.0, fg);
    let dark = data.is_dark();

    let mut theme = Theme::terminal_default();
    theme.bg_base = bg.to_color();
    theme.bg_light = blend(bg, fg, 0.07).to_color();
    theme.bg_dark = blend(bg, if dark { Rgb::new(0, 0, 0) } else { fg }, 0.06).to_color();
    theme.bg_highlight = blend(bg, accent, 0.16).to_color();
    theme.bg_hover = blend(bg, accent, 0.11).to_color();
    theme.bg_visual = blend(bg, accent, 0.28).to_color();
    theme.bg_terminal = theme.bg_dark;

    theme.text_primary = fg.to_color();
    theme.text_secondary = ensure_contrast(blend(bg, fg, 0.78), bg, 4.5, fg).to_color();
    theme.gray_dim = ensure_contrast(blend(bg, fg, 0.58), bg, 3.0, fg).to_color();
    theme.gray = ensure_contrast(blend(bg, fg, 0.70), bg, 3.0, fg).to_color();
    theme.gray_bright = ensure_contrast(blend(bg, fg, 0.86), bg, 4.5, fg).to_color();

    let red = semantic(data, 1, bg, fg);
    let green = semantic(data, 2, bg, fg);
    let yellow = semantic(data, 3, bg, fg);
    let blue = semantic(data, 4, bg, fg);
    let magenta = semantic(data, 5, bg, fg);
    let cyan = semantic(data, 6, bg, fg);

    theme.accent_user = accent.to_color();
    theme.accent_assistant = magenta;
    theme.accent_thinking = theme.gray;
    theme.accent_tool = cyan;
    theme.accent_system = blue;
    theme.accent_error = red;
    theme.accent_success = green;
    theme.accent_running = magenta;
    theme.accent_skill = blue;
    theme.command = yellow;
    theme.path = cyan;
    theme.running = cyan;
    theme.warning = yellow;
    theme.fuzzy_accent = accent.to_color();
    theme.accent_plan = yellow;
    theme.accent_verify = magenta;
    theme.accent_feedback = cyan;
    theme.accent_remember = green;
    theme.selection_border = accent.to_color();
    theme.hover_border = ensure_contrast(blend(bg, accent, 0.72), bg, 3.0, fg).to_color();
    theme.prompt_border = theme.gray;
    theme.prompt_border_active = accent.to_color();
    theme.accent_model = cyan;

    theme.scrollbar_bg = blend(bg, fg, 0.06).to_color();
    theme.scrollbar_fg = ensure_contrast(blend(bg, fg, 0.55), bg, 3.0, fg).to_color();

    theme.diff_delete_bg = blend(bg, color_rgb(red).unwrap_or(fg), 0.14).to_color();
    theme.diff_delete_fg = red;
    theme.diff_insert_bg = blend(bg, color_rgb(green).unwrap_or(fg), 0.14).to_color();
    theme.diff_insert_fg = green;
    theme.diff_equal_fg = theme.gray;
    theme.diff_gutter_fg = theme.gray_dim;

    theme.paste_bg = blend(bg, accent, 0.10).to_color();
    theme.paste_fg = fg.to_color();
    theme.paste_dim = theme.gray;

    theme.md_heading_h1 = accent.to_color();
    theme.md_heading_h2 = magenta;
    theme.md_heading_h3 = blue;
    theme.md_heading_h4 = cyan;
    theme.md_heading_h5 = green;
    theme.md_heading_h6 = theme.gray_bright;
    theme.md_code = cyan;
    theme.md_task_checked = green;
    theme.md_task_unchecked = theme.gray;
    theme.md_muted = theme.gray;
    theme.md_code_bg = theme.bg_dark;
    theme.md_text = fg.to_color();
    theme.link_fg = blue;

    TranslatedWarpTheme { theme, dark }
}

fn semantic(data: &WarpThemeData, index: usize, bg: Rgb, fallback: Rgb) -> Color {
    let normal = data.terminal.normal[index];
    let bright = data.terminal.bright[index];
    let candidate = if contrast_ratio(bright, bg) > contrast_ratio(normal, bg) {
        bright
    } else {
        normal
    };
    ensure_contrast(candidate, bg, 4.5, fallback).to_color()
}

fn ensure_contrast(candidate: Rgb, background: Rgb, minimum: f64, fallback: Rgb) -> Rgb {
    if contrast_ratio(candidate, background) >= minimum {
        candidate
    } else if contrast_ratio(fallback, background) >= minimum {
        fallback
    } else {
        let black = Rgb::new(0, 0, 0);
        let white = Rgb::new(255, 255, 255);
        if contrast_ratio(white, background) >= contrast_ratio(black, background) {
            white
        } else {
            black
        }
    }
}

pub fn blend(from: Rgb, to: Rgb, amount: f64) -> Rgb {
    let amount = amount.clamp(0.0, 1.0);
    let channel = |a: u8, b: u8| {
        (f64::from(a) + (f64::from(b) - f64::from(a)) * amount)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Rgb::new(
        channel(from.r, to.r),
        channel(from.g, to.g),
        channel(from.b, to.b),
    )
}

pub fn relative_luminance(color: Rgb) -> f64 {
    let linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (lighter, darker) = {
        let a = relative_luminance(a);
        let b = relative_luminance(b);
        if a >= b { (a, b) } else { (b, a) }
    };
    (lighter + 0.05) / (darker + 0.05)
}

fn color_rgb(color: Color) -> Option<Rgb> {
    match color {
        Color::Rgb(r, g, b) => Some(Rgb::new(r, g, b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::warp::catalog;

    #[test]
    fn pinned_body_text_meets_contrast() {
        for entry in catalog::all() {
            let translated = translate_pinned(&entry.data);
            let Color::Rgb(br, bg, bb) = translated.theme.bg_base else {
                panic!("{} background must be RGB", entry.id);
            };
            let Color::Rgb(fr, fg, fb) = translated.theme.text_primary else {
                panic!("{} foreground must be RGB", entry.id);
            };
            assert!(
                contrast_ratio(Rgb::new(br, bg, bb), Rgb::new(fr, fg, fb)) >= 4.5,
                "{} body text lacks contrast",
                entry.id
            );
        }
    }

    #[test]
    fn sync_keeps_canvas_transparent() {
        let entry = catalog::find("warp_bundled/fancy_dracula").unwrap();
        let theme = translate_sync(Some(&entry.data)).theme;
        assert_eq!(theme.bg_base, Color::Reset);
        assert_eq!(theme.bg_light, Color::Reset);
        assert_eq!(theme.bg_visual, Color::Reset);
        assert_eq!(theme.md_code_bg, Color::Reset);
    }
}
