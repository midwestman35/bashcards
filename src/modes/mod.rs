use serde::{Deserialize, Serialize};

use crate::{content::Objective, settings::Settings};

pub mod dojo;
pub mod escape_room;
pub mod ops_sim;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CabinetGenre {
    EscapeRoom,
    Dojo,
    OpsSim,
}

pub fn objectives_for_genre(
    genre: CabinetGenre,
    content: &crate::content::ContentLibrary,
    settings: &Settings,
) -> Vec<Objective> {
    match genre {
        CabinetGenre::EscapeRoom => escape_room::EscapeRoomMode::objectives(content),
        CabinetGenre::Dojo => dojo::DojoMode::objectives(content, settings.session_length_target),
        CabinetGenre::OpsSim => ops_sim::OpsSimMode::objectives(content),
    }
}
