use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Clear, List, ListItem},
};

use crate::{arcade::Cabinet, settings::Settings, theme::ThemeToken};

use super::{SettingsReturn, TuiApp, widgets};

pub fn render(frame: &mut Frame<'_>, app: &TuiApp, from: SettingsReturn) {
    let area = centered_rect(74, 62, frame.area());
    frame.render_widget(Clear, area);
    let accent_token = match from {
        SettingsReturn::Lobby => ThemeToken::Pink,
        SettingsReturn::InCabinet => widgets::cabinet_token(app.selected_cabinet()),
    };
    let accent = widgets::color(accent_token, app.settings.high_contrast);
    let override_label = app
        .settings
        .theme_overrides
        .get(app.selected_cabinet().id())
        .map(|override_| format!("{:?}", override_.accent_primary))
        .unwrap_or_else(|| "manifest accent".to_string());

    let items = vec![
        ListItem::new(format!(
            "Difficulty: {} [b/u/o/c in lobby]",
            app.difficulty.label()
        )),
        ListItem::new(format!("Pressure: {} [p]", app.settings.pressure.label())),
        ListItem::new(format!(
            "Reduced motion: {} [r]",
            yes_no(app.settings.reduced_motion)
        )),
        ListItem::new(format!(
            "High contrast: {} [h]",
            yes_no(app.settings.high_contrast)
        )),
        ListItem::new(format!(
            "Animation speed: {} [a]",
            app.settings.animation_speed.label()
        )),
        ListItem::new(format!(
            "Hint style: {} [n]",
            app.settings.hint_style.label()
        )),
        ListItem::new(format!(
            "Explain after success: {} [e]",
            app.settings.explain_after_success.label()
        )),
        ListItem::new(format!(
            "Session length target: {} cards [+/-]",
            app.settings.session_length_target
        )),
        ListItem::new(format!(
            "{} theme override: {} [t]",
            app.selected_cabinet().display_name(),
            override_label
        )),
        ListItem::new("Esc or s returns"),
    ];
    frame.render_widget(
        List::new(items).block(widgets::arcade_block("settings", accent)),
        area,
    );
}

pub fn cycle_theme_override(settings: &mut Settings, cabinet: Cabinet) {
    let entry = settings
        .theme_overrides
        .entry(cabinet.id().to_string())
        .or_insert(crate::settings::ThemeOverride {
            accent_primary: widgets::cabinet_token(cabinet),
        });
    entry.accent_primary = entry.accent_primary.next_vapor();
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn yes_no(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
