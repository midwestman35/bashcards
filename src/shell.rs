use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

#[derive(Clone, Debug)]
pub struct SimulatedOutput {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug)]
pub struct CommandWorld {
    cwd: String,
    dirs: BTreeSet<String>,
    files: BTreeMap<String, String>,
}

impl CommandWorld {
    pub fn lost_terminal() -> Self {
        let mut dirs = BTreeSet::new();
        dirs.insert("/shell-motel/room-101".to_string());
        dirs.insert("/shell-motel/room-101/lobby".to_string());

        let mut files = BTreeMap::new();
        files.insert(
            "/shell-motel/room-101/lobby/README".to_string(),
            "Lobby README: The vending machine accepts paths, not excuses.".to_string(),
        );
        files.insert(
            "/shell-motel/room-101/report.txt".to_string(),
            "Moonbase report: the missing file was in the first directory all along.".to_string(),
        );

        Self {
            cwd: "/shell-motel/room-101".to_string(),
            dirs,
            files,
        }
    }

    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    fn resolve(&self, path: &str) -> String {
        if path.starts_with('/') {
            normalize(path)
        } else {
            normalize(&format!("{}/{}", self.cwd, path))
        }
    }

    fn list(&self) -> Vec<String> {
        let prefix = format!("{}/", self.cwd);
        let mut entries = BTreeSet::new();
        for dir in &self.dirs {
            if let Some(rest) = dir.strip_prefix(&prefix)
                && !rest.is_empty()
                && !rest.contains('/')
            {
                entries.insert(rest.to_string());
            }
        }
        for file in self.files.keys() {
            if let Some(rest) = file.strip_prefix(&prefix)
                && !rest.is_empty()
                && !rest.contains('/')
            {
                entries.insert(rest.to_string());
            }
        }
        entries.into_iter().collect()
    }
}

pub fn run_guided(world: &mut CommandWorld, command: &str) -> SimulatedOutput {
    let mut parts = command.split_whitespace();
    match (parts.next(), parts.next(), parts.next()) {
        (Some("pwd"), None, None) => SimulatedOutput {
            stdout: format!("{}\n", world.cwd()),
            stderr: String::new(),
        },
        (Some("ls"), None, None) => SimulatedOutput {
            stdout: format!("{}\n", world.list().join("  ")),
            stderr: String::new(),
        },
        (Some("cd"), Some(path), None) => {
            let target = world.resolve(path);
            if world.dirs.contains(&target) {
                world.cwd = target;
                SimulatedOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                }
            } else {
                SimulatedOutput {
                    stdout: String::new(),
                    stderr: format!("cd: {path}: No such directory"),
                }
            }
        }
        (Some("cat"), Some(path), None) => {
            let target = world.resolve(path);
            if let Some(contents) = world.files.get(&target) {
                SimulatedOutput {
                    stdout: format!("{contents}\n"),
                    stderr: String::new(),
                }
            } else {
                SimulatedOutput {
                    stdout: String::new(),
                    stderr: format!("cat: {path}: No such file"),
                }
            }
        }
        _ => SimulatedOutput {
            stdout: String::new(),
            stderr: "unknown command".to_string(),
        },
    }
}

fn normalize(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("/{}", parts.join("/"))
}

#[derive(Debug)]
pub struct Sandbox {
    temp_dir: TempDir,
    root: PathBuf,
}

impl Sandbox {
    pub fn create() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let root = temp_dir.path().to_path_buf();
        Ok(Self { temp_dir, root })
    }

    pub fn create_named(name: &str) -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let root = temp_dir.path().join(name);
        std::fs::create_dir_all(&root)?;
        Ok(Self { temp_dir, root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn write_fixture(&self, relative_path: &str, contents: &str) -> anyhow::Result<()> {
        let path = self.root().join(relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn temp_anchor(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn run_command(&self, command: &str) -> anyhow::Result<ShellOutput> {
        let mut process = shell_command(command);
        let output = process.current_dir(self.root()).output()?;
        Ok(ShellOutput {
            status: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("powershell.exe");
    process
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(command);
    process
}

#[cfg(not(windows))]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("sh");
    process.arg("-c").arg(command);
    process
}
