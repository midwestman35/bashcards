use std::path::PathBuf;

use crate::content::ContentLibrary;
use crate::game::{AttemptOutcome, GameSession, Mode};
use crate::modes::{dojo::DojoMode, escape_room::EscapeRoomMode, ops_sim::OpsSimMode};
use crate::profile::Profile;
use crate::settings::{Difficulty, Settings};

pub struct App {
    session: GameSession,
    profile: Profile,
    profile_path: Option<PathBuf>,
}

impl App {
    pub fn launch_mode(
        content: ContentLibrary,
        mode: Mode,
        difficulty: Difficulty,
        settings: Settings,
    ) -> anyhow::Result<Self> {
        let objectives = match mode {
            Mode::EscapeRoom => EscapeRoomMode::objectives(&content),
            Mode::Dojo => DojoMode::objectives(&content, settings.session_length_target),
            Mode::OpsSim => OpsSimMode::objectives(&content),
        };

        let mut session = GameSession::new(mode, difficulty, settings, objectives);
        if mode == Mode::OpsSim && OpsSimMode::requires_real_sandbox(difficulty) {
            session.attach_operator_sandbox()?;
        }

        Ok(Self {
            session,
            profile: Profile::default(),
            profile_path: None,
        })
    }

    pub fn launch_mode_with_profile_path(
        content: ContentLibrary,
        mode: Mode,
        difficulty: Difficulty,
        settings: Settings,
        profile_path: PathBuf,
    ) -> anyhow::Result<Self> {
        let mut app = Self::launch_mode(content, mode, difficulty, settings)?;
        app.profile = if profile_path.exists() {
            Profile::load_from(&profile_path)?
        } else {
            Profile::default()
        };
        app.profile_path = Some(profile_path);
        Ok(app)
    }

    pub fn submit_command(&mut self, command: &str) -> AttemptOutcome {
        let current_id = self.session.current_objective_id().map(ToString::to_string);
        let outcome = self.session.submit_command(command);
        if matches!(outcome, AttemptOutcome::Correct { .. })
            && let Some(id) = current_id
            && !id.is_empty()
            && !self.profile.completed_objectives.contains(&id)
        {
            self.profile.completed_objectives.push(id);
            if let Some(path) = self.profile_path.as_ref() {
                let _ = self.profile.save_to(path);
            }
        }
        outcome
    }

    pub fn session(&self) -> &GameSession {
        &self.session
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }
}

pub fn launch_session(
    mode: Mode,
    difficulty: Difficulty,
    settings: Settings,
) -> anyhow::Result<App> {
    let content = ContentLibrary::bundled()?;
    if let Some(path) = Profile::default_path() {
        App::launch_mode_with_profile_path(content, mode, difficulty, settings, path)
    } else {
        App::launch_mode(content, mode, difficulty, settings)
    }
}
