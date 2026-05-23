use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::arcade::Cabinet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Profile {
    pub schema_version: u32,
    pub cabinets: HashMap<String, CabinetProgress>,
    pub weak_topics: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CabinetProgress {
    pub score: u32,
    pub streak: u32,
    pub attempts: u32,
    pub completed_objectives: HashSet<String>,
    pub last_objective_id: Option<String>,
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct ProfileV1 {
    #[serde(default)]
    completed_objectives: Vec<String>,
    #[serde(default)]
    weak_topics: Vec<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            schema_version: 2,
            cabinets: HashMap::new(),
            weak_topics: Vec::new(),
        }
    }
}

impl Profile {
    pub fn default_path() -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("BASHCARDS_PROFILE_PATH") {
            return Some(PathBuf::from(path));
        }
        ProjectDirs::from("dev", "midwestman35", "bashcards")
            .map(|dirs| dirs.data_dir().join("profile.toml"))
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let value: toml::Value = toml::from_str(&data)?;
        if value.get("schema_version").is_some() {
            return Ok(toml::from_str(&data)?);
        }

        let old: ProfileV1 = toml::from_str(&data)?;
        let mut profile = Profile {
            weak_topics: old.weak_topics,
            ..Profile::default()
        };
        profile.set_cabinet_progress(
            Cabinet::ShellMotel,
            CabinetProgress {
                completed_objectives: old.completed_objectives.into_iter().collect(),
                ..CabinetProgress::default()
            },
        );
        profile.save_to(path)?;
        Ok(profile)
    }

    pub fn cabinet_progress(&self, cabinet: Cabinet) -> Option<&CabinetProgress> {
        self.cabinets.get(cabinet.id())
    }

    pub fn cabinet_progress_mut(&mut self, cabinet: Cabinet) -> &mut CabinetProgress {
        self.cabinets.entry(cabinet.id().to_string()).or_default()
    }

    pub fn set_cabinet_progress(&mut self, cabinet: Cabinet, progress: CabinetProgress) {
        self.cabinets.insert(cabinet.id().to_string(), progress);
    }

    pub fn record_objective(
        &mut self,
        cabinet: Cabinet,
        objective_id: &str,
        score: u32,
        streak: u32,
        attempts: u32,
    ) {
        let progress = self.cabinet_progress_mut(cabinet);
        progress.score = score;
        progress.streak = streak;
        progress.attempts = attempts;
        progress
            .completed_objectives
            .insert(objective_id.to_string());
        progress.last_objective_id = Some(objective_id.to_string());
        progress.last_played = Some(Utc::now());
    }

    pub fn record_session_stats(
        &mut self,
        cabinet: Cabinet,
        score: u32,
        streak: u32,
        attempts: u32,
    ) {
        let progress = self.cabinet_progress_mut(cabinet);
        progress.score = score;
        progress.streak = streak;
        progress.attempts = attempts;
        progress.last_played = Some(Utc::now());
    }

    pub fn default_focus(&self) -> Cabinet {
        Cabinet::all()
            .iter()
            .copied()
            .min_by_key(|cabinet| {
                self.cabinet_progress(*cabinet)
                    .map(|progress| progress.completed_objectives.len())
                    .unwrap_or(0)
            })
            .unwrap_or(Cabinet::ShellMotel)
    }
}
