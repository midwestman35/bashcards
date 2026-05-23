use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::{effects::gradient_cells, theme::ThemeToken};

use super::{Screen, TuiApp, widgets};

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let cabinet = app
        .game
        .as_ref()
        .map(|game| game.session().cabinet())
        .unwrap_or_else(|| app.selected_cabinet());
    let accent = widgets::color(widgets::cabinet_token(cabinet), app.settings.high_contrast);
    let [top, body, hint, input, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    let transition = match app.screen {
        Screen::CabinetEnter => "entering",
        Screen::CabinetExit => "returning to lobby",
        _ => "playing",
    };
    let (prompt, score, attempts, streak, complete, sandbox_root) = app
        .game
        .as_ref()
        .map(|game| {
            (
                game.session()
                    .current_objective_prompt()
                    .unwrap_or("Session complete."),
                game.session().score(),
                game.session().attempts(),
                game.session().streak(),
                game.session().completed(),
                game.session()
                    .sandbox_root()
                    .map(|path| path.display().to_string()),
            )
        })
        .unwrap_or(("No session.", 0, 0, 0, false, None));

    frame.render_widget(
        Paragraph::new(format!(
            "{} {} | {} | score {score} | attempts {attempts} | streak {streak}",
            cabinet.glyph(),
            cabinet.display_name(),
            transition
        ))
        .block(widgets::arcade_block("cabinet", accent)),
        top,
    );

    let log = app
        .log
        .iter()
        .rev()
        .take(8)
        .rev()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    frame.render_widget(
        Paragraph::new(log)
            .block(widgets::arcade_block("room / challenge", accent))
            .wrap(Wrap { trim: true }),
        body,
    );

    let objective = if complete {
        Line::from(
            gradient_cells("Objective complete. Esc returns to the arcade.")
                .into_iter()
                .map(|cell| {
                    Span::styled(
                        cell.ch.to_string(),
                        Style::default().fg(ratatui::style::Color::Rgb(
                            cell.rgb.0, cell.rgb.1, cell.rgb.2,
                        )),
                    )
                })
                .collect::<Vec<_>>(),
        )
    } else if let Some(root) = sandbox_root {
        Line::from(format!(
            "Objective: {prompt} | Sandbox: {root} | prefix real shell commands with !"
        ))
    } else {
        Line::from(format!("Objective: {prompt}"))
    };
    frame.render_widget(
        Paragraph::new(objective)
            .block(widgets::arcade_block(
                "objective",
                widgets::color(ThemeToken::Violet, app.settings.high_contrast),
            ))
            .wrap(Wrap { trim: true }),
        hint,
    );

    frame.render_widget(
        Paragraph::new(format!("$ {}", app.command))
            .style(Style::default().fg(accent))
            .block(widgets::arcade_block("command panel", accent)),
        input,
    );

    frame.render_widget(
        Paragraph::new("Enter submit | ? hint | s settings | Esc lobby | q quit")
            .block(widgets::arcade_block("controls", accent)),
        footer,
    );
}
