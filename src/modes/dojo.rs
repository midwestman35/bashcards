use crate::content::{ContentLibrary, Objective};

pub struct DojoMode;

impl DojoMode {
    pub fn objectives(content: &ContentLibrary, session_length_target: usize) -> Vec<Objective> {
        let mut objectives = content.first_dojo_objectives();
        objectives.truncate(session_length_target);
        objectives
    }

    pub fn review_deck<'a>(
        objectives: &'a [Objective],
        missed_objectives: &[String],
    ) -> Vec<&'a Objective> {
        objectives
            .iter()
            .filter(|objective| missed_objectives.contains(&objective.id))
            .collect()
    }
}
