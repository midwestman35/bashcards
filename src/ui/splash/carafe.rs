use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{animation::Cue, theme::ThemeToken};

use super::super::{TuiApp, widgets};

pub const CARAFE_GLYPHS: [[&str; 6]; 2] = [
    ["⢀⣀⡀", "⣀⡀", "⡀⢀", "⡀⢀", "⣀⡀", "⢀⣀⡀"],
    ["⠛⠁", "⣸⠟", "⠉⠉", "⢾⠆", "⣸⠟", "⠉⠉⠆"],
];

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let accent = widgets::color(ThemeToken::Cyan, app.settings.high_contrast);
    let dim = widgets::color(ThemeToken::DimProse, app.settings.high_contrast);
    let area = frame.area();
    let [_, center, _] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(6),
            Constraint::Percentage(35),
        ])
        .areas(area);

    let progress = app
        .animator
        .sample(&Cue::ImprintMaterialize { progress: 0.0 })
        .map(|sample| sample.progress)
        .unwrap_or(1.0);

    let lines = CARAFE_GLYPHS
        .iter()
        .map(|row| {
            let visible = ((row.len() as f32) * progress).ceil().max(1.0) as usize;
            Line::from(
                row.iter()
                    .take(visible.min(row.len()))
                    .map(|letter| {
                        Span::styled(
                            format!("{letter} "),
                            Style::default().fg(accent).add_modifier(Modifier::BOLD),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .chain([
            Line::from(""),
            Line::from(Span::styled("arcade imprint", Style::default().fg(dim))),
        ])
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().bg(widgets::color(
                ThemeToken::DeepField,
                app.settings.high_contrast,
            ))),
        center,
    );
}
