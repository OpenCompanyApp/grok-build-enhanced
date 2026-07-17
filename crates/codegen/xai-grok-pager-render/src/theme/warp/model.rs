use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn to_color(self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }

    pub fn midpoint(self, other: Self) -> Self {
        Self::new(
            ((u16::from(self.r) + u16::from(other.r)) / 2) as u8,
            ((u16::from(self.g) + u16::from(other.g)) / 2) as u8,
            ((u16::from(self.b) + u16::from(other.b)) / 2) as u8,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fill {
    Solid(Rgb),
    Vertical { top: Rgb, bottom: Rgb },
    Horizontal { left: Rgb, right: Rgb },
}

impl Fill {
    pub fn midpoint(self) -> Rgb {
        match self {
            Self::Solid(color) => color,
            Self::Vertical { top, bottom } => top.midpoint(bottom),
            Self::Horizontal { left, right } => left.midpoint(right),
        }
    }

    pub fn is_gradient(self) -> bool {
        !matches!(self, Self::Solid(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnsiPalette {
    pub normal: [Rgb; 8],
    pub bright: [Rgb; 8],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WarpThemeData {
    pub name: Option<String>,
    pub background: Fill,
    pub foreground: Rgb,
    pub accent: Fill,
    pub cursor: Option<Fill>,
    pub terminal: AnsiPalette,
    pub details: Option<String>,
    pub has_background_image: bool,
}

impl WarpThemeData {
    pub fn is_gradient(&self) -> bool {
        self.background.is_gradient() || self.accent.is_gradient()
    }

    pub fn is_dark(&self) -> bool {
        let bg = self.background.midpoint();
        super::translate::relative_luminance(bg) < 0.5
    }
}
