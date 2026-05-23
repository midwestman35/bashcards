use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::{App, launch_session},
    effects::gradient_cells,
    game::{AttemptOutcome, Mode},
    settings::{Difficulty, Settings},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Launcher,
    Playing,
}

pub struct TuiApp {
    screen: Screen,
    mode: Mode,
    difficulty: Difficulty,
    settings: Settings,
    game: Option<App>,
    command: String,
    log: Vec<String>,
}

impl TuiApp {
    fn new() -> Self {
        Self {
            screen: Screen::Launcher,
            mode: Mode::EscapeRoom,
            difficulty: Difficulty::Beginner,
            settings: Settings::default(),
            game: None,
            command: String::new(),
            log: vec![
                "Welcome to the Shell Motel. The prompt is already judging the furniture."
                    .to_string(),
            ],
        }
    }

    fn start(&mut self) -> anyhow::Result<()> {
        self.game = Some(launch_session(
            self.mode,
            self.difficulty,
            self.settings.clone(),
        )?);
        self.screen = Screen::Playing;
        self.command.clear();
        self.log.clear();
        self.log.push(format!(
            "{} / {} started.",
            self.mode.label(),
            self.difficulty.label()
        ));
        Ok(())
    }

    fn submit_command(&mut self) {
        let command = self.command.trim().to_string();
        if command.is_empty() {
            return;
        }
        self.log.push(format!("$ {command}"));
        if let Some(game) = self.game.as_mut() {
            match game.submit_command(&command) {
                AttemptOutcome::Correct { feedback } => {
                    self.log.push(format!("OK: {feedback}"));
                    if game.session().completed() {
                        self.log.push(format!(
                            "Session complete. Score: {} in {} attempts.",
                            game.session().score(),
                            game.session().attempts()
                        ));
                    }
                }
                AttemptOutcome::Incorrect { feedback } => {
                    self.log.push(format!("Try again: {feedback}"))
                }
                AttemptOutcome::Blocked { feedback } => {
                    self.log.push(format!("Blocked: {feedback}"))
                }
            }
        }
        self.command.clear();
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub fn run_tui() -> anyhow::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = TuiApp::new();

    loop {
        terminal.draw(|frame| render(frame, &app))?;
        if let Event::Key(key) = event::read()? {
            match app.screen {
                Screen::Launcher => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('1') => app.mode = Mode::EscapeRoom,
                    KeyCode::Char('2') => app.mode = Mode::Dojo,
                    KeyCode::Char('3') => app.mode = Mode::OpsSim,
                    KeyCode::Char('b') => app.difficulty = Difficulty::Beginner,
                    KeyCode::Char('u') => app.difficulty = Difficulty::Builder,
                    KeyCode::Char('o') => app.difficulty = Difficulty::Operator,
                    KeyCode::Char('c') => app.difficulty = Difficulty::Chaos,
                    KeyCode::Char('p') => app.settings.pressure = app.settings.pressure.next(),
                    KeyCode::Char('r') => {
                        app.settings.reduced_motion = !app.settings.reduced_motion
                    }
                    KeyCode::Char('h') => app.settings.high_contrast = !app.settings.high_contrast,
                    KeyCode::Char('a') => {
                        app.settings.animation_speed = app.settings.animation_speed.next()
                    }
                    KeyCode::Char('n') => app.settings.hint_style = app.settings.hint_style.next(),
                    KeyCode::Char('e') => {
                        app.settings.explain_after_success =
                            app.settings.explain_after_success.next()
                    }
                    KeyCode::Char('+') => {
                        app.settings.session_length_target =
                            (app.settings.session_length_target + 1).min(20)
                    }
                    KeyCode::Char('-') => {
                        app.settings.session_length_target =
                            app.settings.session_length_target.saturating_sub(1).max(3)
                    }
                    KeyCode::Enter => app.start()?,
                    _ => {}
                },
                Screen::Playing => match key.code {
                    KeyCode::Esc => app.screen = Screen::Launcher,
                    KeyCode::Char('q') => break,
                    KeyCode::Enter => app.submit_command(),
                    KeyCode::Backspace => {
                        app.command.pop();
                    }
                    KeyCode::Char(ch) => app.command.push(ch),
                    _ => {}
                },
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    match app.screen {
        Screen::Launcher => render_launcher(frame, app),
        Screen::Playing => render_game(frame, app),
    }
}

fn render_launcher(frame: &mut Frame<'_>, app: &TuiApp) {
    let [header, body, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "bashcards",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - a terminal training console for learning bash by doing"),
            ]),
            Line::from("The door has no handle. It has a prompt."),
        ])
        .block(Block::default().borders(Borders::ALL).title("Launcher")),
        header,
    );

    let settings = vec![
        ListItem::new(format!(
            "Mode: {}   [1 Escape] [2 Dojo] [3 Ops]",
            app.mode.label()
        )),
        ListItem::new(format!(
            "Difficulty: {}   [b Beginner] [u Builder] [o Operator] [c Chaos]",
            app.difficulty.label()
        )),
        ListItem::new(format!(
            "Pressure: {}   [p cycle]",
            app.settings.pressure.label()
        )),
        ListItem::new(format!(
            "Reduced motion: {}   [r toggle]",
            yes_no(app.settings.reduced_motion)
        )),
        ListItem::new(format!(
            "High contrast: {}   [h toggle]",
            yes_no(app.settings.high_contrast)
        )),
        ListItem::new(format!(
            "Motion: {}   [a cycle]",
            app.settings.animation_speed.label()
        )),
        ListItem::new(format!(
            "Learning: hints {} [n], explain {} [e], {} cards [+/-]",
            app.settings.hint_style.label(),
            app.settings.explain_after_success.label(),
            app.settings.session_length_target
        )),
        ListItem::new("Press Enter to start. Press q to quit."),
    ];
    frame.render_widget(
        List::new(settings).block(Block::default().borders(Borders::ALL).title("Settings")),
        body,
    );

    frame.render_widget(
        Paragraph::new("All choices are available in the first build; higher difficulties reuse early content with stricter scoring hooks ready.")
            .block(Block::default().borders(Borders::ALL).title("Status")),
        footer,
    );
}

fn render_game(frame: &mut Frame<'_>, app: &TuiApp) {
    let [header, scene, lesson, command, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    let (prompt, score, attempts, complete, sandbox_root) = app
        .game
        .as_ref()
        .map(|game| {
            (
                game.session()
                    .current_objective_prompt()
                    .unwrap_or("Session complete."),
                game.session().score(),
                game.session().attempts(),
                game.session().completed(),
                game.session()
                    .sandbox_root()
                    .map(|path| path.display().to_string()),
            )
        })
        .unwrap_or(("No session.", 0, 0, false, None));

    frame.render_widget(
        Paragraph::new(format!(
            "{} | {} | {} | score {score} | attempts {attempts}",
            app.mode.label(),
            app.difficulty.label(),
            app.settings.pressure.label()
        ))
        .block(Block::default().borders(Borders::ALL).title("Header")),
        header,
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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Room / Challenge"),
            )
            .wrap(Wrap { trim: true }),
        scene,
    );

    let lesson_line = if complete {
        gradient_line("Objective complete. Press Esc for launcher or q to quit.")
    } else if let Some(root) = sandbox_root {
        Line::from(format!(
            "Current objective: {prompt} | Sandbox: {root} | prefix real shell commands with !"
        ))
    } else {
        Line::from(format!("Current objective: {prompt}"))
    };
    frame.render_widget(
        Paragraph::new(lesson_line)
            .block(Block::default().borders(Borders::ALL).title("Lesson Card"))
            .wrap(Wrap { trim: true }),
        lesson,
    );

    frame.render_widget(
        Paragraph::new(format!("$ {}", app.command))
            .style(Style::default().fg(if app.settings.high_contrast {
                Color::White
            } else {
                Color::Green
            }))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Panel"),
            ),
        command,
    );

    frame.render_widget(
        Paragraph::new("Enter submit | Backspace edit | Esc launcher | q quit")
            .block(Block::default().borders(Borders::ALL).title("Footer")),
        footer,
    );
}

fn yes_no(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

fn gradient_line(text: &str) -> Line<'static> {
    Line::from(
        gradient_cells(text)
            .into_iter()
            .map(|cell| {
                Span::styled(
                    cell.ch.to_string(),
                    Style::default().fg(Color::Rgb(cell.rgb.0, cell.rgb.1, cell.rgb.2)),
                )
            })
            .collect::<Vec<_>>(),
    )
}
