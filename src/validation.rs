use std::path::{Component, Path};

use crate::content::ValidatorSpec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationResult {
    Pass,
    Fail(String),
    Blocked(String),
}

pub fn validate_exact(expected: &str, actual: &str) -> ValidationResult {
    let actual = actual.trim();
    if expected == actual {
        ValidationResult::Pass
    } else if expected == "pwd" && actual == "ls" {
        ValidationResult::Fail(
            "You are close. `ls` shows what is here; `pwd` shows where here is.".to_string(),
        )
    } else {
        ValidationResult::Fail(format!(
            "Try a different command. Hint: `{expected}` is the form this objective expects."
        ))
    }
}

pub fn validate_command(spec: &ValidatorSpec, command: &str) -> ValidationResult {
    match spec {
        ValidatorSpec::ExactCommand { expected } => validate_exact(expected, command),
        ValidatorSpec::CommandPattern { patterns } => {
            let trimmed = command.trim();
            if patterns.iter().any(|pattern| pattern == trimmed) {
                ValidationResult::Pass
            } else {
                ValidationResult::Fail(
                    "That is a real move, but not the one this prompt needs.".to_string(),
                )
            }
        }
        ValidatorSpec::AnyOf { validators } => validators
            .iter()
            .map(|validator| validate_command(validator, command))
            .find(|result| *result == ValidationResult::Pass)
            .unwrap_or_else(|| {
                ValidationResult::Fail("None of the accepted forms matched.".to_string())
            }),
        ValidatorSpec::FilesystemState { .. } => {
            ValidationResult::Fail("Filesystem validators run after sandbox work.".to_string())
        }
    }
}

pub fn validate_filesystem_path(root: &Path, relative_path: &str) -> ValidationResult {
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return ValidationResult::Blocked(
            "That path tries to leave the challenge sandbox.".to_string(),
        );
    }

    if root.join(relative).exists() {
        ValidationResult::Pass
    } else {
        ValidationResult::Fail(format!("Missing sandbox path `{relative_path}`."))
    }
}
