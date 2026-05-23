use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
};

use crate::{arcade::Cabinet, theme::ThemeToken};

use super::{TuiApp, widgets};

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let [header, cabinets, detail, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(7),
            Constraint::Length(7),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    let accent = widgets::color(ThemeToken::Pink, app.settings.high_contrast);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Bash Arcade",
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
            Line::from("Choose a cabinet. Difficulty and pressure stay independent."),
        ])
        .block(widgets::arcade_block("lobby", accent)),
        header,
    );

    if frame.area().width < 80 {
        render_vertical_cabinet_list(frame, app, cabinets);
    } else {
        render_cabinet_row(frame, app, cabinets);
    }

    let selected = app.selected_cabinet();
    let progress = app.profile.cabinet_progress(selected);
    let completed = progress
        .map(|progress| progress.completed_objectives.len())
        .unwrap_or(0);
    let score = progress.map(|progress| progress.score).unwrap_or(0);
    let streak = progress.map(|progress| progress.streak).unwrap_or(0);
    let selected_accent =
        widgets::color(widgets::cabinet_token(selected), app.settings.high_contrast);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("{} {}", selected.glyph(), selected.display_name()),
                    Style::default()
                        .fg(selected_accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" - {}", selected.tagline())),
            ]),
            Line::from(format!(
                "difficulty {} | pressure {} | completed {} | score {} | streak {}",
                app.difficulty.label(),
                app.settings.pressure.label(),
                completed,
                score,
                streak
            )),
            Line::from("Next objective appears when you launch. Progress is saved per cabinet."),
        ])
        .wrap(Wrap { trim: true })
        .block(widgets::arcade_block("selected session", selected_accent)),
        detail,
    );

    frame.render_widget(
        Paragraph::new(
            "h/l or arrows focus | 1/2/3 jump | Enter launch | s settings | ? help | q quit",
        )
        .block(widgets::arcade_block("controls", accent)),
        footer,
    );
}

fn render_cabinet_row(frame: &mut Frame<'_>, app: &TuiApp, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);
    for (index, cabinet) in Cabinet::all().iter().copied().enumerate() {
        render_cabinet_card(frame, app, chunks[index], index, cabinet);
    }
}

fn render_vertical_cabinet_list(frame: &mut Frame<'_>, app: &TuiApp, area: ratatui::layout::Rect) {
    let items = Cabinet::all()
        .iter()
        .copied()
        .enumerate()
        .map(|(index, cabinet)| {
            let progress = app.profile.cabinet_progress(cabinet);
            let completed = progress
                .map(|progress| progress.completed_objectives.len())
                .unwrap_or(0);
            let line = format!(
                "{} {} - {} [{} complete]",
                cabinet.glyph(),
                cabinet.display_name(),
                cabinet.tagline(),
                completed
            );
            let accent =
                widgets::color(widgets::cabinet_token(cabinet), app.settings.high_contrast);
            if index == app.selected {
                ListItem::new(line).style(widgets::selected_style(accent))
            } else {
                ListItem::new(line)
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(widgets::arcade_block(
            "cabinets",
            widgets::color(ThemeToken::Cyan, app.settings.high_contrast),
        )),
        area,
    );
}

fn render_cabinet_card(
    frame: &mut Frame<'_>,
    app: &TuiApp,
    area: ratatui::layout::Rect,
    index: usize,
    cabinet: Cabinet,
) {
    let accent = widgets::color(widgets::cabinet_token(cabinet), app.settings.high_contrast);
    let progress = app.profile.cabinet_progress(cabinet);
    let score = progress.map(|progress| progress.score).unwrap_or(0);
    let completed = progress
        .map(|progress| progress.completed_objectives.len())
        .unwrap_or(0);
    let mut block = widgets::arcade_block(cabinet.display_name(), accent);
    if index == app.selected {
        block = block.border_style(widgets::selected_style(accent));
    }
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                cabinet.glyph().to_string(),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
            Line::from(cabinet.tagline()),
            Line::from(format!("completed {completed} | score {score}")),
        ])
        .style(if index == app.selected {
            Style::default().fg(accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .block(block)
        .wrap(Wrap { trim: true }),
        area,
    );
}
