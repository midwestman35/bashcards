use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Frame, Terminal, backend::CrosstermBackend};

use crate::{
    animation::{Animator, Cue, MotionBudget},
    app::{App, launch_session},
    arcade::Cabinet,
    game::AttemptOutcome,
    profile::Profile,
    settings::{Difficulty, Settings},
};

mod cabinet;
mod lobby;
mod settings;
mod splash;
mod widgets;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    CarafeImprint,
    Splash,
    Lobby,
    CabinetEnter,
    InCabinet,
    CabinetExit,
    Settings { from: SettingsReturn },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsReturn {
    Lobby,
    InCabinet,
}

pub struct TuiApp {
    screen: Screen,
    selected: usize,
    difficulty: Difficulty,
    settings: Settings,
    profile: Profile,
    game: Option<App>,
    command: String,
    log: Vec<String>,
    animator: Animator,
    state_elapsed: Duration,
    should_quit: bool,
}

impl TuiApp {
    fn new() -> Self {
        let settings = Settings::default();
        let profile = Profile::default_path()
            .filter(|path| path.exists())
            .and_then(|path| Profile::load_from(&path).ok())
            .unwrap_or_default();
        let selected_cabinet = profile.default_focus();
        let selected = Cabinet::all()
            .iter()
            .position(|cabinet| *cabinet == selected_cabinet)
            .unwrap_or(0);
        let mut animator = Animator::new(MotionBudget::from_settings(&settings));
        animator.fire(Cue::ImprintMaterialize { progress: 0.0 });

        Self {
            screen: Screen::CarafeImprint,
            selected,
            difficulty: Difficulty::Beginner,
            settings,
            profile,
            game: None,
            command: String::new(),
            log: Vec::new(),
            animator,
            state_elapsed: Duration::ZERO,
            should_quit: false,
        }
    }

    fn selected_cabinet(&self) -> Cabinet {
        Cabinet::all()[self.selected]
    }

    fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.state_elapsed = Duration::ZERO;
    }

    fn start_selected_cabinet(&mut self) -> anyhow::Result<()> {
        let cabinet = self.selected_cabinet();
        self.game = Some(launch_session(
            cabinet,
            self.difficulty,
            self.settings.clone(),
        )?);
        self.log.clear();
        self.log.push(format!(
            "{} started at {}.",
            cabinet.display_name(),
            self.difficulty.label()
        ));
        self.command.clear();
        self.animator.fire(Cue::CabinetEnter { cabinet });
        self.set_screen(Screen::CabinetEnter);
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
                    self.animator.fire(Cue::CorrectGlow);
                    if game.session().streak() >= 3 {
                        self.animator.fire(Cue::StreakSurge {
                            streak: game.session().streak() as u32,
                        });
                    }
                    self.log.push(format!("OK: {feedback}"));
                    if game.session().completed() {
                        self.log.push(format!(
                            "Session complete. Score: {} in {} attempts.",
                            game.session().score(),
                            game.session().attempts()
                        ));
                    } else {
                        self.animator.fire(Cue::ObjectiveGradient);
                    }
                }
                AttemptOutcome::Incorrect { feedback } => {
                    self.animator.fire(Cue::IncorrectFlicker);
                    self.log.push(format!("Try again: {feedback}"));
                }
                AttemptOutcome::Blocked { feedback } => {
                    self.log.push(format!("Blocked: {feedback}"));
                }
            }
        }
        self.command.clear();
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }

        match self.screen {
            Screen::CarafeImprint => match key.code {
                KeyCode::Enter | KeyCode::Esc => self.enter_splash(),
                _ => {}
            },
            Screen::Splash => match key.code {
                KeyCode::Esc | KeyCode::Enter => self.set_screen(Screen::Lobby),
                KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            },
            Screen::Lobby => self.handle_lobby_key(key)?,
            Screen::InCabinet => self.handle_cabinet_key(key),
            Screen::Settings { from } => self.handle_settings_key(key, from),
            Screen::CabinetEnter | Screen::CabinetExit => {}
        }
        Ok(())
    }

    fn handle_lobby_key(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') | KeyCode::Left => {
                self.selected = if self.selected == 0 {
                    Cabinet::all().len() - 1
                } else {
                    self.selected - 1
                };
                self.animator.fire(Cue::LobbyHover {
                    cabinet: self.selected_cabinet(),
                });
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.selected = (self.selected + 1) % Cabinet::all().len();
                self.animator.fire(Cue::LobbyHover {
                    cabinet: self.selected_cabinet(),
                });
            }
            KeyCode::Char('1') => self.selected = 0,
            KeyCode::Char('2') => self.selected = 1,
            KeyCode::Char('3') => self.selected = 2,
            KeyCode::Char('b') => self.difficulty = Difficulty::Beginner,
            KeyCode::Char('u') => self.difficulty = Difficulty::Builder,
            KeyCode::Char('o') => self.difficulty = Difficulty::Operator,
            KeyCode::Char('c') => self.difficulty = Difficulty::Chaos,
            KeyCode::Char('p') => self.settings.pressure = self.settings.pressure.next(),
            KeyCode::Char('s') => self.set_screen(Screen::Settings {
                from: SettingsReturn::Lobby,
            }),
            KeyCode::Char('?') => {
                self.log.push(
                    "Move with h/l or arrows. Enter launches the focused cabinet.".to_string(),
                );
            }
            KeyCode::Enter => self.start_selected_cabinet()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_cabinet_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.animator.fire(Cue::CabinetExit);
                self.set_screen(Screen::CabinetExit);
            }
            KeyCode::Char('q') if self.command.is_empty() => self.should_quit = true,
            KeyCode::Char('s') if self.command.is_empty() => self.set_screen(Screen::Settings {
                from: SettingsReturn::InCabinet,
            }),
            KeyCode::Char('?') if self.command.is_empty() => {
                self.animator.fire(Cue::HintFlash);
                self.log
                    .push("Hint: read the objective literally, then type the command.".to_string());
            }
            KeyCode::Enter => self.submit_command(),
            KeyCode::Backspace => {
                self.command.pop();
            }
            KeyCode::Char(ch) => self.command.push(ch),
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: crossterm::event::KeyEvent, from: SettingsReturn) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('s') => {
                self.animator
                    .set_motion_budget(MotionBudget::from_settings(&self.settings));
                self.set_screen(match from {
                    SettingsReturn::Lobby => Screen::Lobby,
                    SettingsReturn::InCabinet => Screen::InCabinet,
                });
            }
            KeyCode::Char('p') => self.settings.pressure = self.settings.pressure.next(),
            KeyCode::Char('r') => self.settings.reduced_motion = !self.settings.reduced_motion,
            KeyCode::Char('h') => self.settings.high_contrast = !self.settings.high_contrast,
            KeyCode::Char('a') => {
                self.settings.animation_speed = self.settings.animation_speed.next()
            }
            KeyCode::Char('n') => self.settings.hint_style = self.settings.hint_style.next(),
            KeyCode::Char('e') => {
                self.settings.explain_after_success = self.settings.explain_after_success.next()
            }
            KeyCode::Char('+') => {
                self.settings.session_length_target =
                    (self.settings.session_length_target + 1).min(20)
            }
            KeyCode::Char('-') => {
                self.settings.session_length_target =
                    self.settings.session_length_target.saturating_sub(1).max(3)
            }
            KeyCode::Char('t') => {
                let cabinet = self.selected_cabinet();
                settings::cycle_theme_override(&mut self.settings, cabinet);
            }
            _ => {}
        }
    }

    fn enter_splash(&mut self) {
        self.animator.fire(Cue::TitleSweep);
        for (idx, _) in splash::boot_console::boot_lines().iter().enumerate() {
            self.animator.fire(Cue::BootLineReveal { line_idx: idx });
        }
        self.set_screen(Screen::Splash);
    }

    fn advance(&mut self, elapsed: Duration) {
        self.state_elapsed = self.state_elapsed.saturating_add(elapsed);
        self.animator.tick(elapsed);
        match self.screen {
            Screen::CarafeImprint => {
                let duration = if self.settings.reduced_motion {
                    Duration::from_secs(1)
                } else {
                    Duration::from_millis(3_500)
                };
                if self.state_elapsed >= duration {
                    self.enter_splash();
                }
            }
            Screen::CabinetEnter if self.state_elapsed >= Duration::from_millis(400) => {
                self.animator.fire(Cue::ObjectiveGradient);
                self.set_screen(Screen::InCabinet);
            }
            Screen::Splash
                if self.settings.reduced_motion
                    && self.state_elapsed >= Duration::from_millis(500) =>
            {
                self.set_screen(Screen::Lobby);
            }
            Screen::CabinetExit if self.state_elapsed >= Duration::from_millis(300) => {
                if let Some(game) = self.game.as_ref() {
                    self.profile = game.profile().clone();
                }
                self.game = None;
                self.command.clear();
                self.set_screen(Screen::Lobby);
            }
            _ => {}
        }
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
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| render(frame, &app))?;
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(last_tick);
        last_tick = now;
        app.advance(elapsed);

        let frame_budget = if app.settings.reduced_motion {
            Duration::ZERO
        } else {
            Duration::from_millis(16)
        };
        if event::poll(frame_budget)?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key)?;
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    match app.screen {
        Screen::CarafeImprint => splash::carafe::render(frame, app),
        Screen::Splash => splash::boot_console::render(frame, app),
        Screen::Lobby => lobby::render(frame, app),
        Screen::CabinetEnter | Screen::InCabinet | Screen::CabinetExit => {
            cabinet::render(frame, app)
        }
        Screen::Settings { from } => match from {
            SettingsReturn::Lobby => {
                lobby::render(frame, app);
                settings::render(frame, app, SettingsReturn::Lobby);
            }
            SettingsReturn::InCabinet => {
                cabinet::render(frame, app);
                settings::render(frame, app, SettingsReturn::InCabinet);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::{app::App, content::ContentLibrary};

    fn test_app(screen: Screen) -> TuiApp {
        let settings = Settings::default();
        TuiApp {
            screen,
            selected: 0,
            difficulty: Difficulty::Beginner,
            settings: settings.clone(),
            profile: Profile::default(),
            game: None,
            command: String::new(),
            log: vec!["render smoke".to_string()],
            animator: Animator::new(MotionBudget::from_settings(&settings)),
            state_elapsed: Duration::ZERO,
            should_quit: false,
        }
    }

    #[test]
    fn renders_arcade_chrome_states_without_panicking() {
        for screen in [
            Screen::CarafeImprint,
            Screen::Splash,
            Screen::Lobby,
            Screen::Settings {
                from: SettingsReturn::Lobby,
            },
        ] {
            let backend = TestBackend::new(100, 32);
            let mut terminal = Terminal::new(backend).expect("terminal");
            let app = test_app(screen);
            terminal.draw(|frame| render(frame, &app)).expect("render");
        }
    }

    #[test]
    fn renders_compact_lobby_below_eighty_columns() {
        let backend = TestBackend::new(72, 22);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let app = test_app(Screen::Lobby);

        terminal.draw(|frame| render(frame, &app)).expect("render");
    }

    #[test]
    fn renders_in_cabinet_state_with_live_session() {
        let backend = TestBackend::new(100, 32);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut app = test_app(Screen::InCabinet);
        app.game = Some(
            App::launch_cabinet(
                ContentLibrary::bundled_for(Cabinet::ShellMotel).expect("content"),
                Cabinet::ShellMotel,
                Difficulty::Beginner,
                Settings::default(),
            )
            .expect("launch"),
        );

        terminal.draw(|frame| render(frame, &app)).expect("render");
    }

    #[test]
    fn ignores_key_release_events_to_prevent_duplicate_input() {
        let mut app = test_app(Screen::InCabinet);
        app.handle_key(crossterm::event::KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::NONE,
        ))
        .expect("press handled");
        app.handle_key(crossterm::event::KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        })
        .expect("release ignored");

        assert_eq!(app.command, "p");
    }
}
