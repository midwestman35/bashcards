use crate::{
    content::{ContentLibrary, Objective},
    settings::Difficulty,
};

pub struct OpsSimMode;

impl OpsSimMode {
    pub fn objectives(content: &ContentLibrary) -> Vec<Objective> {
        content.first_ops_objectives()
    }

    pub fn requires_real_sandbox(difficulty: Difficulty) -> bool {
        matches!(difficulty, Difficulty::Operator | Difficulty::Chaos)
    }
}
