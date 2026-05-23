use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};

use crate::{
    arcade::Cabinet,
    theme::{ThemeToken, adapt},
};

pub fn color(token: ThemeToken, high_contrast: bool) -> Color {
    if high_contrast {
        match token {
            ThemeToken::DeepField | ThemeToken::MidField | ThemeToken::LowField => Color::Black,
            ThemeToken::DimProse => Color::Rgb(0xe0, 0xd4, 0xff),
            _ => token.rgb().to_ratatui(),
        }
    } else if truecolor_available() {
        token.rgb().to_ratatui()
    } else {
        Color::Indexed(adapt::to_ansi(token.rgb()))
    }
}

pub fn cabinet_token(cabinet: Cabinet) -> ThemeToken {
    match cabinet {
        Cabinet::ShellMotel => ThemeToken::Pink,
        Cabinet::MonasteryOfForms => ThemeToken::Cyan,
        Cabinet::MidnightCarnival => ThemeToken::Mint,
    }
}

pub fn arcade_block<'a>(title: &'a str, accent: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .title(title)
}

pub fn selected_style(accent: Color) -> Style {
    Style::default()
        .fg(accent)
        .bg(Color::Indexed(17))
        .add_modifier(Modifier::BOLD)
}

fn truecolor_available() -> bool {
    std::env::var("COLORTERM")
        .map(|value| value.eq_ignore_ascii_case("truecolor") || value.eq_ignore_ascii_case("24bit"))
        .unwrap_or(true)
}
