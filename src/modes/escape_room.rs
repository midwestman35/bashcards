use crate::content::{ContentLibrary, Objective};

pub struct EscapeRoomMode;

impl EscapeRoomMode {
    pub fn objectives(content: &ContentLibrary) -> Vec<Objective> {
        content.first_escape_room_objectives()
    }
}
