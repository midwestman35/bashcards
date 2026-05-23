use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ContentLibrary {
    #[serde(default)]
    chapters: Vec<Chapter>,
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

    pub fn bundled() -> anyhow::Result<Self> {
        Self::from_toml_str(include_str!("../content/lost_terminal.toml"))
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
        if self.chapters.is_empty() {
            anyhow::bail!("content library must include at least one chapter");
        }
        for chapter in &self.chapters {
            if chapter.rooms.is_empty() || chapter.decks.is_empty() || chapter.incidents.is_empty()
            {
                anyhow::bail!(
                    "chapter `{}` must include rooms, decks, and incidents",
                    chapter.id
                );
            }
        }
        Ok(())
    }
}
