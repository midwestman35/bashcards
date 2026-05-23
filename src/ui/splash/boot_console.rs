use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::{arcade::Cabinet, theme::ThemeToken};

use super::super::{TuiApp, widgets};

pub fn boot_lines() -> Vec<String> {
    let mut lines = vec!["initializing vapor console".to_string()];
    lines.extend(
        Cabinet::all()
            .iter()
            .map(|cabinet| format!("mounting cabinet {}", cabinet.id())),
    );
    lines.extend([
        "loading profile".to_string(),
        "checking reset protocol".to_string(),
    ]);
    lines
}

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let accent = widgets::color(ThemeToken::Pink, app.settings.high_contrast);
    let cyan = widgets::color(ThemeToken::DeepCyan, app.settings.high_contrast);
    let mint = widgets::color(ThemeToken::Mint, app.settings.high_contrast);
    let [title, body, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "BASH ARCADE",
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "PASTEL TERMINAL TRAINING SYSTEM // SOFT URGENCY",
                Style::default().fg(cyan),
            )),
        ])
        .alignment(Alignment::Center)
        .block(widgets::arcade_block("bash arcade boot console", accent)),
        title,
    );

    let boot_lines = boot_lines()
        .into_iter()
        .map(|line| {
            Line::from(vec![
                Span::styled("[OK] ", Style::default().fg(mint)),
                Span::raw(line),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(boot_lines)
            .block(widgets::arcade_block("mount log", cyan))
            .wrap(Wrap { trim: true }),
        body,
    );

    frame.render_widget(
        Paragraph::new("press Enter to insert coin | Esc skips to lobby | q quits")
            .alignment(Alignment::Center)
            .block(widgets::arcade_block("ready", mint)),
        footer,
    );
}
