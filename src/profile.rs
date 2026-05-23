use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Profile {
    pub completed_objectives: Vec<String>,
    pub weak_topics: Vec<String>,
}

impl Profile {
    pub fn default_path() -> Option<PathBuf> {
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
        Ok(toml::from_str(&data)?)
    }
}
