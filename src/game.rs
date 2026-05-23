use crate::{
    arcade::Cabinet,
    content::Objective,
    settings::{Difficulty, PressureProfile, Settings},
    shell::{CommandWorld, Sandbox, run_guided},
    validation::{ValidationResult, validate_command},
};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttemptOutcome {
    Correct { feedback: String },
    Incorrect { feedback: String },
    Blocked { feedback: String },
}

pub struct CabinetSession {
    cabinet: Cabinet,
    pub difficulty: Difficulty,
    pub settings: Settings,
    objectives: Vec<Objective>,
    current: usize,
    attempts: usize,
    score: i32,
    streak: usize,
    missed_objectives: Vec<String>,
    command_world: CommandWorld,
    last_command_output: String,
    sandbox: Option<Sandbox>,
    completed: bool,
}

impl CabinetSession {
    pub fn empty(cabinet: Cabinet, difficulty: Difficulty, settings: Settings) -> Self {
        Self {
            cabinet,
            difficulty,
            settings,
            objectives: Vec::new(),
            current: 0,
            attempts: 0,
            score: 0,
            streak: 0,
            missed_objectives: Vec::new(),
            command_world: CommandWorld::lost_terminal(),
            last_command_output: String::new(),
            sandbox: None,
            completed: false,
        }
    }

    pub fn new(
        cabinet: Cabinet,
        difficulty: Difficulty,
        settings: Settings,
        objectives: Vec<Objective>,
    ) -> Self {
        Self {
            cabinet,
            difficulty,
            settings,
            objectives,
            current: 0,
            attempts: 0,
            score: 0,
            streak: 0,
            missed_objectives: Vec::new(),
            command_world: CommandWorld::lost_terminal(),
            last_command_output: String::new(),
            sandbox: None,
            completed: false,
        }
    }

    pub fn cabinet(&self) -> Cabinet {
        self.cabinet
    }

    pub fn attach_operator_sandbox(&mut self) -> anyhow::Result<()> {
        let sandbox = Sandbox::create_named("bashcards-operator")?;
        sandbox.write_fixture(
            "report.txt",
            "Moonbase report: files are calmer after inspection.",
        )?;
        self.sandbox = Some(sandbox);
        Ok(())
    }

    pub fn submit_command(&mut self, command: &str) -> AttemptOutcome {
        if let Some(shell_command) = command.trim().strip_prefix('!') {
            return self.submit_sandbox_command(shell_command.trim());
        }
        if self.completed {
            return AttemptOutcome::Correct {
                feedback: "Session already complete.".to_string(),
            };
        }
        let Some(objective) = self.objectives.get(self.current) else {
            return AttemptOutcome::Incorrect {
                feedback: "No challenge loaded.".to_string(),
            };
        };

        self.attempts += 1;
        let simulated = run_guided(&mut self.command_world, command);
        self.last_command_output = if simulated.stderr.is_empty() {
            simulated.stdout
        } else {
            simulated.stderr
        };
        match validate_command(&objective.validator, command) {
            ValidationResult::Pass => {
                let success = objective.success.clone();
                self.score += self.points_for_correct();
                self.streak += 1;
                self.current += 1;
                self.completed = self.current >= self.objectives.len();
                AttemptOutcome::Correct { feedback: success }
            }
            ValidationResult::Fail(feedback) => {
                self.score = (self.score - self.penalty_for_miss()).max(0);
                self.streak = 0;
                if !self.missed_objectives.contains(&objective.id) {
                    self.missed_objectives.push(objective.id.clone());
                }
                AttemptOutcome::Incorrect { feedback }
            }
            ValidationResult::Blocked(feedback) => {
                self.streak = 0;
                AttemptOutcome::Blocked { feedback }
            }
        }
    }

    pub fn current_objective_prompt(&self) -> Option<&str> {
        self.objectives
            .get(self.current)
            .map(|objective| objective.prompt.as_str())
    }

    pub fn current_objective_id(&self) -> Option<&str> {
        self.objectives
            .get(self.current)
            .map(|objective| objective.id.as_str())
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn attempts(&self) -> usize {
        self.attempts
    }

    pub fn streak(&self) -> usize {
        self.streak
    }

    pub fn missed_objectives(&self) -> &[String] {
        &self.missed_objectives
    }

    pub fn last_command_output(&self) -> &str {
        &self.last_command_output
    }

    pub fn uses_real_sandbox(&self) -> bool {
        self.sandbox.is_some()
    }

    pub fn sandbox_root(&self) -> Option<&Path> {
        self.sandbox.as_ref().map(Sandbox::root)
    }

    fn points_for_correct(&self) -> i32 {
        match self.settings.pressure {
            PressureProfile::CozyNoTimer => 100,
            PressureProfile::SoftUrgency => 110,
            PressureProfile::ArcadePressure => 140,
        }
    }

    fn penalty_for_miss(&self) -> i32 {
        match self.settings.pressure {
            PressureProfile::CozyNoTimer => 0,
            PressureProfile::SoftUrgency => 5,
            PressureProfile::ArcadePressure => 15,
        }
    }

    fn submit_sandbox_command(&mut self, command: &str) -> AttemptOutcome {
        let Some(sandbox) = self.sandbox.as_ref() else {
            return AttemptOutcome::Blocked {
                feedback: "Real shell commands are only available in sandbox modes.".to_string(),
            };
        };
        match sandbox.run_command(command) {
            Ok(output) => {
                self.last_command_output = if output.stderr.is_empty() {
                    output.stdout
                } else {
                    output.stderr
                };
                if output.status == 0 {
                    AttemptOutcome::Correct {
                        feedback: format!(
                            "Sandbox command exited 0 inside {}.",
                            sandbox.root().display()
                        ),
                    }
                } else {
                    AttemptOutcome::Incorrect {
                        feedback: format!("Sandbox command exited {}.", output.status),
                    }
                }
            }
            Err(error) => AttemptOutcome::Blocked {
                feedback: format!("Could not run sandbox command: {error}"),
            },
        }
    }
}
