use serde::{Deserialize, Serialize};

use crate::{arcade::Cabinet, modes::CabinetGenre};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentLibrary {
    #[serde(default)]
    cabinet: Option<CabinetManifest>,
    #[serde(default)]
    chapters: Vec<Chapter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CabinetManifest {
    pub id: String,
    pub display_name: String,
    pub tagline: String,
    pub glyph: String,
    pub genre: CabinetGenre,
    pub theme: CabinetTheme,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CabinetTheme {
    pub accent_primary: String,
    pub accent_secondary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub rooms: Vec<Room>,
    #[serde(default)]
    pub decks: Vec<Deck>,
    #[serde(default)]
    pub incidents: Vec<Incident>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Room {
    pub id: String,
    pub title: String,
    pub scene: String,
    #[serde(default)]
    pub objectives: Vec<Objective>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deck {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub cards: Vec<ChallengeCard>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChallengeCard {
    pub id: String,
    pub prompt: String,
    pub success: String,
    #[serde(default)]
    pub hints: Vec<String>,
    pub validator: ValidatorSpec,
}

impl From<ChallengeCard> for Objective {
    fn from(card: ChallengeCard) -> Self {
        Self {
            id: card.id,
            prompt: card.prompt,
            success: card.success,
            hints: card.hints,
            validator: card.validator,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub objective: Objective,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Objective {
    pub id: String,
    pub prompt: String,
    pub success: String,
    #[serde(default)]
    pub hints: Vec<String>,
    pub validator: ValidatorSpec,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ValidatorSpec {
    ExactCommand { expected: String },
    CommandPattern { patterns: Vec<String> },
    FilesystemState { path: String },
    AnyOf { validators: Vec<ValidatorSpec> },
}

impl ContentLibrary {
    pub fn from_toml_str(source: &str) -> anyhow::Result<Self> {
        let library: Self = toml::from_str(source)?;
        library.validate()?;
        Ok(library)
    }

    pub fn bundled_for(cabinet: Cabinet) -> anyhow::Result<Self> {
        Self::from_toml_str(cabinet.manifest_toml())
    }

    pub fn bundled() -> anyhow::Result<Self> {
        Self::bundled_for(Cabinet::ShellMotel)
    }

    pub fn cabinet(&self) -> Option<&CabinetManifest> {
        self.cabinet.as_ref()
    }

    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    pub fn first_escape_room_objectives(&self) -> Vec<Objective> {
        self.chapters
            .first()
            .and_then(|chapter| chapter.rooms.first())
            .map(|room| room.objectives.clone())
            .unwrap_or_default()
    }

    pub fn first_dojo_objectives(&self) -> Vec<Objective> {
        self.chapters
            .first()
            .and_then(|chapter| chapter.decks.first())
            .map(|deck| {
                deck.cards
                    .clone()
                    .into_iter()
                    .map(Objective::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn first_ops_objectives(&self) -> Vec<Objective> {
        self.chapters
            .first()
            .map(|chapter| {
                chapter
                    .incidents
                    .iter()
                    .map(|incident| incident.objective.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn validate(&self) -> anyhow::Result<()> {
        let Some(cabinet) = self.cabinet.as_ref() else {
            anyhow::bail!("cabinet manifest must include a [cabinet] block");
        };
        if Cabinet::from_id(&cabinet.id).is_none() {
            anyhow::bail!("unknown cabinet id `{}`", cabinet.id);
        }
        if self.chapters.is_empty() {
            anyhow::bail!("content library must include at least one chapter");
        }
        for chapter in &self.chapters {
            match cabinet.genre {
                CabinetGenre::EscapeRoom if chapter.rooms.is_empty() => {
                    anyhow::bail!("escape-room cabinet `{}` must include rooms", cabinet.id)
                }
                CabinetGenre::Dojo if chapter.decks.is_empty() => {
                    anyhow::bail!("dojo cabinet `{}` must include decks", cabinet.id)
                }
                CabinetGenre::OpsSim if chapter.incidents.is_empty() => {
                    anyhow::bail!("ops-sim cabinet `{}` must include incidents", cabinet.id)
                }
                _ => {}
            }
        }
        Ok(())
    }
}
