use std::path::PathBuf;

use crate::arcade::Cabinet;
use crate::content::ContentLibrary;
use crate::game::{AttemptOutcome, CabinetSession};
use crate::modes::{CabinetGenre, objectives_for_genre};
use crate::profile::Profile;
use crate::settings::{Difficulty, Settings};

pub struct App {
    session: CabinetSession,
    profile: Profile,
    profile_path: Option<PathBuf>,
}

impl App {
    pub fn launch_cabinet(
        content: ContentLibrary,
        cabinet: Cabinet,
        difficulty: Difficulty,
        settings: Settings,
    ) -> anyhow::Result<Self> {
        let manifest = content
            .cabinet()
            .ok_or_else(|| anyhow::anyhow!("cabinet content missing manifest"))?;
        if manifest.id != cabinet.id() {
            anyhow::bail!(
                "content manifest `{}` does not match cabinet `{}`",
                manifest.id,
                cabinet.id()
            );
        }
        let genre = manifest.genre;
        let objectives = objectives_for_genre(genre, &content, &settings);

        let mut session = CabinetSession::new(cabinet, difficulty, settings, objectives);
        if cabinet == Cabinet::MidnightCarnival
            && genre == CabinetGenre::OpsSim
            && crate::modes::ops_sim::OpsSimMode::requires_real_sandbox(difficulty)
        {
            session.attach_operator_sandbox()?;
        }

        Ok(Self {
            session,
            profile: Profile::default(),
            profile_path: None,
        })
    }

    pub fn launch_cabinet_with_profile_path(
        content: ContentLibrary,
        cabinet: Cabinet,
        difficulty: Difficulty,
        settings: Settings,
        profile_path: PathBuf,
    ) -> anyhow::Result<Self> {
        let mut app = Self::launch_cabinet(content, cabinet, difficulty, settings)?;
        app.profile = if profile_path.exists() {
            Profile::load_from(&profile_path)?
        } else {
            Profile::default()
        };
        app.profile_path = Some(profile_path);
        Ok(app)
    }

    pub fn submit_command(&mut self, command: &str) -> AttemptOutcome {
        let is_sandbox_command = command.trim().starts_with('!');
        let current_id = self.session.current_objective_id().map(ToString::to_string);
        let outcome = self.session.submit_command(command);
        if is_sandbox_command {
            return outcome;
        }
        let score = self.session.score().max(0) as u32;
        let streak = self.session.streak() as u32;
        let attempts = self.session.attempts() as u32;

        if matches!(outcome, AttemptOutcome::Correct { .. })
            && let Some(id) = current_id
            && !id.is_empty()
        {
            self.profile
                .record_objective(self.session.cabinet(), &id, score, streak, attempts);
        } else {
            self.profile
                .record_session_stats(self.session.cabinet(), score, streak, attempts);
        }

        if let Some(path) = self.profile_path.as_ref() {
            let _ = self.profile.save_to(path);
        }
        outcome
    }

    pub fn session(&self) -> &CabinetSession {
        &self.session
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }
}

pub fn launch_session(
    cabinet: Cabinet,
    difficulty: Difficulty,
    settings: Settings,
) -> anyhow::Result<App> {
    let content = ContentLibrary::bundled_for(cabinet)?;
    if let Some(path) = Profile::default_path() {
        App::launch_cabinet_with_profile_path(content, cabinet, difficulty, settings, path)
    } else {
        App::launch_cabinet(content, cabinet, difficulty, settings)
    }
}
