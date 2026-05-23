#[derive(Clone, Debug, Default)]
pub struct CommandHistory {
    entries: Vec<String>,
}

impl CommandHistory {
    pub fn push(&mut self, command: impl Into<String>) {
        self.entries.push(command.into());
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}
