use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cabinet {
    ShellMotel,
    MonasteryOfForms,
    MidnightCarnival,
}

impl Cabinet {
    pub const fn all() -> &'static [Cabinet] {
        &[
            Cabinet::ShellMotel,
            Cabinet::MonasteryOfForms,
            Cabinet::MidnightCarnival,
        ]
    }

    pub const fn id(&self) -> &'static str {
        match self {
            Self::ShellMotel => "shell-motel",
            Self::MonasteryOfForms => "monastery-of-forms",
            Self::MidnightCarnival => "midnight-carnival",
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::ShellMotel => "Shell Motel",
            Self::MonasteryOfForms => "Monastery of Forms",
            Self::MidnightCarnival => "Midnight Carnival",
        }
    }

    pub const fn tagline(&self) -> &'static str {
        match self {
            Self::ShellMotel => "a hallway asks where you are",
            Self::MonasteryOfForms => "repeat until the scroll smiles",
            Self::MidnightCarnival => "find the lost logs",
        }
    }

    pub const fn glyph(&self) -> char {
        match self {
            Self::ShellMotel => '\u{25a6}',
            Self::MonasteryOfForms => '\u{26e9}',
            Self::MidnightCarnival => '\u{25ce}',
        }
    }

    pub const fn manifest_toml(&self) -> &'static str {
        match self {
            Self::ShellMotel => include_str!("../../content/cabinets/shell-motel.toml"),
            Self::MonasteryOfForms => {
                include_str!("../../content/cabinets/monastery-of-forms.toml")
            }
            Self::MidnightCarnival => {
                include_str!("../../content/cabinets/midnight-carnival.toml")
            }
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|cabinet| cabinet.id() == id)
    }
}
