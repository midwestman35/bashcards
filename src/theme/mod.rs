use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub const THEME_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeToken {
    Pink,
    Cyan,
    Violet,
    Mint,
    Yellow,
    DimProse,
    DeepField,
    MidField,
    LowField,
    NeonMagenta,
    DeepCyan,
    NavyInset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl ThemeToken {
    pub const fn rgb(self) -> Rgb {
        match self {
            Self::Pink => Rgb(0xff, 0x9a, 0xd2),
            Self::Cyan => Rgb(0x8a, 0xf0, 0xff),
            Self::Violet => Rgb(0xc8, 0xa8, 0xff),
            Self::Mint => Rgb(0xc9, 0xff, 0xb3),
            Self::Yellow => Rgb(0xff, 0xe8, 0xa3),
            Self::DimProse => Rgb(0x9f, 0x87, 0xc8),
            Self::DeepField => Rgb(0x1b, 0x0a, 0x36),
            Self::MidField => Rgb(0x2a, 0x10, 0x52),
            Self::LowField => Rgb(0x1f, 0x0a, 0x44),
            Self::NeonMagenta => Rgb(0xff, 0x3e, 0x9d),
            Self::DeepCyan => Rgb(0x00, 0xd4, 0xff),
            Self::NavyInset => Rgb(0x2e, 0x3a, 0xa0),
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        VAPOR_AND_PUNCTUATION_TOKENS
            .iter()
            .copied()
            .find(|token| token.rgb().hex().eq_ignore_ascii_case(hex))
    }

    pub fn next_vapor(self) -> Self {
        match self {
            Self::Pink => Self::Cyan,
            Self::Cyan => Self::Violet,
            Self::Violet => Self::Mint,
            Self::Mint => Self::Yellow,
            Self::Yellow => Self::Pink,
            _ => Self::Pink,
        }
    }
}

impl Rgb {
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    pub fn to_ratatui(self) -> Color {
        Color::Rgb(self.0, self.1, self.2)
    }
}

pub const VAPOR_TOKENS: [ThemeToken; 5] = [
    ThemeToken::Pink,
    ThemeToken::Cyan,
    ThemeToken::Violet,
    ThemeToken::Mint,
    ThemeToken::Yellow,
];

const VAPOR_AND_PUNCTUATION_TOKENS: [ThemeToken; 12] = [
    ThemeToken::Pink,
    ThemeToken::Cyan,
    ThemeToken::Violet,
    ThemeToken::Mint,
    ThemeToken::Yellow,
    ThemeToken::DimProse,
    ThemeToken::DeepField,
    ThemeToken::MidField,
    ThemeToken::LowField,
    ThemeToken::NeonMagenta,
    ThemeToken::DeepCyan,
    ThemeToken::NavyInset,
];

pub mod adapt {
    use super::Rgb;

    pub fn to_ansi(rgb: Rgb) -> u8 {
        let mut best = 0;
        let mut best_distance = u32::MAX;
        for index in 16u8..=255 {
            let candidate = color_256(index);
            let distance = squared_distance(rgb, candidate);
            if distance < best_distance {
                best = index;
                best_distance = distance;
            }
        }
        best
    }

    fn squared_distance(a: Rgb, b: Rgb) -> u32 {
        let dr = a.0 as i32 - b.0 as i32;
        let dg = a.1 as i32 - b.1 as i32;
        let db = a.2 as i32 - b.2 as i32;
        (dr * dr + dg * dg + db * db) as u32
    }

    fn color_256(index: u8) -> Rgb {
        if index >= 232 {
            let value = 8 + (index - 232) * 10;
            return Rgb(value, value, value);
        }

        let offset = index - 16;
        let r = offset / 36;
        let g = (offset % 36) / 6;
        let b = offset % 6;
        Rgb(level(r), level(g), level(b))
    }

    fn level(value: u8) -> u8 {
        if value == 0 { 0 } else { 55 + value * 40 }
    }
}
