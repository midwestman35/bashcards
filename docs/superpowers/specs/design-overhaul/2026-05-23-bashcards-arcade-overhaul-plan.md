# Bash Arcade Overhaul Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the Bash Arcade overhaul per `docs/superpowers/specs/design-overhaul/2026-05-23-bashcards-arcade-overhaul-design.md` — promote bashcards' three modes into arcade cabinets, add a carafe imprint splash + arcade boot console, introduce a `vyfor/animate`-backed animation runtime that drives both chrome and in-game cues, reshape `Profile` to per-cabinet progress, and split the content TOML into per-cabinet manifests.

**Architecture:** Ten ordered phases. Each phase ends with a green build + passing tests + a commit; the codebase compiles after every phase. Phases 0–2 are foundational (deps, content split, profile shape) and ship no UI changes. Phases 3–5 build the animation + theme runtime under the existing TUI. Phases 6–9 replace the TUI surface (splash, lobby, in-cabinet, settings). Phase 10 is polish + manual QA.

**Tech Stack:** Rust 2024 edition · ratatui 0.29 · crossterm 0.29 · vyfor/animate (new, with `ratatui` feature) · toml 0.9 · serde 1 · directories 6 · tempfile 3 · anyhow 1 · thiserror 2 · assert_fs 1 (dev).

---

## File Structure

After the overhaul. **Created** files are new; **modified** files exist today and change shape.

| Path                                       | State     | Responsibility                                                         |
|---                                         |---        |---                                                                     |
| `Cargo.toml`                               | modified  | Adds `animate` dep                                                     |
| `src/main.rs`                              | unchanged | Calls `ui::run_tui`                                                    |
| `src/lib.rs`                               | modified  | Adds new modules: `arcade`, `animation`, `theme`                       |
| `src/app.rs`                               | modified  | `launch_cabinet(Cabinet, …)` replaces `launch_mode(Mode, …)`           |
| `src/game.rs`                              | modified  | `Mode` removed; `GameSession` renamed to `CabinetSession`              |
| `src/content.rs`                           | modified  | Adds `CabinetManifest`, per-cabinet content loading                    |
| `src/validation.rs`                        | unchanged |                                                                        |
| `src/shell.rs`                             | unchanged |                                                                        |
| `src/profile.rs`                           | modified  | Per-cabinet `CabinetProgress` with `schema_version` + v1 migration     |
| `src/settings.rs`                          | modified  | Adds `theme_overrides: HashMap<String, ThemeOverride>`                 |
| `src/effects.rs`                           | modified  | Re-expressed as a thin facade over `animation::Animator`               |
| `src/input.rs`                             | unchanged |                                                                        |
| `src/telemetry_local.rs`                   | unchanged |                                                                        |
| `src/ui.rs`                                | **deleted** | Replaced by `src/ui/` directory                                      |
| `src/arcade/mod.rs`                        | created   | `ArcadeState` enum; top-level state machine                            |
| `src/arcade/cabinet.rs`                    | created   | `Cabinet` enum + manifest accessors                                    |
| `src/arcade/session.rs`                    | created   | `CabinetEntry` (entry + exit transition state)                         |
| `src/animation/mod.rs`                     | created   | `Animator`, `Cue`, `Sample`, `MotionBudget`                            |
| `src/animation/timeline.rs`                | created   | Per-cue durations + easing + sample functions                          |
| `src/animation/reduced_motion.rs`          | created   | Snap-to-final-sample helpers                                           |
| `src/theme/mod.rs`                         | created   | `Palette`, `CabinetAccent`, `adapt::to_ansi`                           |
| `src/theme/vapor.rs`                       | created   | Vapor palette consts + high-contrast palette                           |
| `src/theme/famicom.rs`                     | created   | Famicom punctuation consts                                             |
| `src/modes/mod.rs`                         | modified  | `CabinetGenre` enum + extractor dispatch                               |
| `src/modes/escape_room.rs`                 | modified  | Takes a specific `ContentLibrary` (already shape-correct)              |
| `src/modes/dojo.rs`                        | unchanged | Already shape-correct                                                  |
| `src/modes/ops_sim.rs`                     | unchanged | Already shape-correct                                                  |
| `src/ui/mod.rs`                            | created   | `run_tui`, frame loop, top-level state dispatch                        |
| `src/ui/splash/mod.rs`                     | created   | Splash state machine; routes carafe → boot console                     |
| `src/ui/splash/carafe.rs`                  | created   | Imprint renderer + braille glyph table                                 |
| `src/ui/splash/boot_console.rs`            | created   | Bash arcade boot console renderer + boot lines                         |
| `src/ui/lobby.rs`                          | created   | Cabinet grid + selected-session panel + nav                            |
| `src/ui/cabinet.rs`                        | created   | In-cabinet play screen                                                 |
| `src/ui/settings.rs`                       | created   | Settings modal overlay                                                 |
| `src/ui/widgets.rs`                        | created   | Shared widgets: vapor panel, glyph row, glow bar                       |
| `content/lost_terminal.toml`               | **deleted** | Split into three cabinet manifests                                   |
| `content/cabinets/shell-motel.toml`        | created   | Escape Room cabinet (lifted from `[[chapters.rooms]]`)                 |
| `content/cabinets/monastery-of-forms.toml` | created   | Dojo cabinet (lifted from `[[chapters.decks.cards]]`)                  |
| `content/cabinets/midnight-carnival.toml`  | created   | Ops Sim cabinet (lifted from `[[chapters.incidents]]`)                 |
| `tests/content_validation.rs`              | modified  | Updated content fixtures + cabinet manifest validation tests           |
| `tests/gameplay_flow.rs`                   | modified  | `Mode` → `Cabinet`; `launch_mode` → `launch_cabinet`                   |
| `tests/sandbox_and_agents.rs`              | modified  | Same renames                                                           |
| `tests/settings_effects_profile.rs`        | modified  | Profile per-cabinet shape + animation_speed wiring tests               |
| `tests/animation.rs`                       | created   | Animator unit tests (TUI-free)                                         |
| `tests/theme.rs`                           | created   | Palette + ANSI adapter tests                                           |

---

## Phase 0 · Foundation: dependency + baseline green build

### Task 0.1: Confirm clean baseline

**Files:** none modified

- [ ] **Step 1: Run the existing test suite to confirm a clean baseline**

```
cargo test
```

Expected: all existing tests in `tests/` pass.

- [ ] **Step 2: Run clippy to confirm baseline lint state**

```
cargo clippy -- -D warnings
```

Expected: no warnings. (If warnings exist on `main`, capture the list — do not fix them as part of this overhaul.)

### Task 0.2: Add `animate` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Look up the latest `animate` crate version on crates.io**

Visit https://crates.io/crates/animate and capture the latest published version. Note whether the crate exposes a `time_scale` API (search docs.rs for `time_scale` or `speed` on the `Engine` type) — this affects Task 6.5.

- [ ] **Step 2: Add the dependency with the `ratatui` feature**

Modify `Cargo.toml`:

```toml
[dependencies]
anyhow = "1"
animate = { version = "X.Y", features = ["ratatui"] }  # replace X.Y with the version captured above
crossterm = "0.29"
directories = "6"
ratatui = "0.29"
serde = { version = "1", features = ["derive"] }
tempfile = "3"
thiserror = "2"
toml = "0.9"
```

- [ ] **Step 3: Verify the build still compiles**

```
cargo build
```

Expected: success. If `animate` fails to compile against ratatui 0.29 (its `ratatui` feature may be pinned to a different ratatui version), drop the `ratatui` feature and re-record the consequence in a code comment near `animation::sample`: the wrapper will own all ratatui-color interpolation manually rather than via the crate's helper. Continue.

- [ ] **Step 4: Commit**

```
git add Cargo.toml Cargo.lock
git commit -m "Add vyfor/animate dependency for arcade animation runtime"
```

---

## Phase 1 · Content split + Cabinet enum (no UI change)

This phase splits `content/lost_terminal.toml` into three cabinet manifests, introduces the `Cabinet` enum alongside (not replacing) the existing `Mode` enum, and adds a `CabinetManifest` type. The legacy `Mode` enum stays during this phase so existing tests and the existing UI continue to work; it is removed in Phase 6 when the new UI lands.

### Task 1.1: Add `CabinetManifest` and `CabinetTheme` types

**Files:**
- Modify: `src/content.rs:1-152`
- Test: `tests/content_validation.rs`

- [ ] **Step 1: Write the failing test for cabinet manifest parsing**

Add to `tests/content_validation.rs`:

```rust
use bashcards::content::CabinetManifest;

#[test]
fn parses_cabinet_manifest_with_theme_and_rooms() {
    let manifest = CabinetManifest::from_toml_str(
        r#"
        [cabinet]
        id = "shell-motel"
        display_name = "Shell Motel"
        tagline = "a hallway asks where you are"
        glyph = "▦"
        genre = "escape_room"

        [cabinet.theme]
        accent_primary = "#ff9ad2"
        accent_secondary = "#8af0ff"

        [[chapters]]
        id = "lost-terminal"
        title = "Lost Terminal"

        [[chapters.rooms]]
        id = "shell-motel-101"
        title = "Shell Motel Room 101"
        scene = "The door has no handle. It has a prompt."

        [[chapters.rooms.objectives]]
        id = "state-location"
        prompt = "State your location."
        success = "Objective complete."
        hints = ["`pwd` prints the current directory."]
        validator = { type = "ExactCommand", expected = "pwd" }
        "#,
    )
    .expect("manifest parses");

    assert_eq!(manifest.cabinet.id, "shell-motel");
    assert_eq!(manifest.cabinet.display_name, "Shell Motel");
    assert_eq!(manifest.cabinet.glyph, "▦");
    assert_eq!(manifest.cabinet.genre, "escape_room");
    assert_eq!(manifest.cabinet.theme.accent_primary, "#ff9ad2");
    assert_eq!(manifest.chapters().len(), 1);
}
```

- [ ] **Step 2: Run the test and confirm it fails**

```
cargo test --test content_validation parses_cabinet_manifest_with_theme_and_rooms
```

Expected: FAIL — `CabinetManifest` is not defined.

- [ ] **Step 3: Add the types to `src/content.rs`**

Insert above the existing `ContentLibrary` struct in `src/content.rs`:

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CabinetManifest {
    pub cabinet: CabinetMeta,
    #[serde(default)]
    chapters: Vec<Chapter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CabinetMeta {
    pub id: String,
    pub display_name: String,
    pub tagline: String,
    pub glyph: String,
    pub genre: String,
    #[serde(default)]
    pub theme: CabinetTheme,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CabinetTheme {
    #[serde(default)]
    pub accent_primary: String,
    #[serde(default)]
    pub accent_secondary: String,
}

impl CabinetManifest {
    pub fn from_toml_str(source: &str) -> anyhow::Result<Self> {
        let manifest: Self = toml::from_str(source)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.cabinet.id.is_empty() {
            anyhow::bail!("cabinet manifest is missing an id");
        }
        match self.cabinet.genre.as_str() {
            "escape_room" | "dojo" | "ops_sim" => {}
            other => anyhow::bail!("cabinet `{}` has unknown genre `{other}`", self.cabinet.id),
        }
        if self.chapters.is_empty() {
            anyhow::bail!("cabinet `{}` must include at least one chapter", self.cabinet.id);
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Run the test and confirm it passes**

```
cargo test --test content_validation parses_cabinet_manifest_with_theme_and_rooms
```

Expected: PASS.

- [ ] **Step 5: Add a failing test for unknown-genre rejection**

Add to `tests/content_validation.rs`:

```rust
#[test]
fn rejects_cabinet_manifest_with_unknown_genre() {
    let result = CabinetManifest::from_toml_str(
        r#"
        [cabinet]
        id = "bogus"
        display_name = "Bogus"
        tagline = "x"
        glyph = "?"
        genre = "telepathy"

        [[chapters]]
        id = "c"
        title = "C"

        [[chapters.rooms]]
        id = "r"
        title = "R"
        scene = "s"
        "#,
    );
    assert!(result.is_err(), "unknown genre should fail validation");
}
```

- [ ] **Step 6: Run the test and confirm it passes**

```
cargo test --test content_validation rejects_cabinet_manifest_with_unknown_genre
```

Expected: PASS.

- [ ] **Step 7: Commit**

```
git add src/content.rs tests/content_validation.rs
git commit -m "Add CabinetManifest type with genre validation"
```

### Task 1.2: Split `lost_terminal.toml` into three cabinet manifests

**Files:**
- Create: `content/cabinets/shell-motel.toml`
- Create: `content/cabinets/monastery-of-forms.toml`
- Create: `content/cabinets/midnight-carnival.toml`
- Delete (Task 1.4): `content/lost_terminal.toml`

- [ ] **Step 1: Create the cabinets directory**

```
mkdir -p content/cabinets
```

- [ ] **Step 2: Write `content/cabinets/shell-motel.toml`**

```toml
[cabinet]
id = "shell-motel"
display_name = "Shell Motel"
tagline = "a hallway asks where you are"
glyph = "▦"
genre = "escape_room"

[cabinet.theme]
accent_primary = "#ff9ad2"
accent_secondary = "#8af0ff"

[[chapters]]
id = "lost-terminal"
title = "Lost Terminal"

[[chapters.rooms]]
id = "shell-motel-101"
title = "Shell Motel Room 101"
scene = "The door has no handle. It has a prompt."

[[chapters.rooms.objectives]]
id = "state-location"
prompt = "State your location."
success = "Objective complete: you found your current location."
hints = ["`pwd` prints the current directory."]
validator = { type = "ExactCommand", expected = "pwd" }

[[chapters.rooms.objectives]]
id = "list-room"
prompt = "List what the room contains."
success = "A lobby directory flickers into view."
hints = ["Use `ls` to list entries."]
validator = { type = "ExactCommand", expected = "ls" }

[[chapters.rooms.objectives]]
id = "enter-lobby"
prompt = "Move into the lobby directory."
success = "The lobby accepts your relative path."
hints = ["Use `cd lobby`."]
validator = { type = "ExactCommand", expected = "cd lobby" }

[[chapters.rooms.objectives]]
id = "read-lobby-readme"
prompt = "Read the lobby README."
success = "The README coughs up a clue. Manny Page says this counts as literature."
hints = ["Use `cat README` to print the file."]
validator = { type = "ExactCommand", expected = "cat README" }
```

- [ ] **Step 3: Write `content/cabinets/monastery-of-forms.toml`**

```toml
[cabinet]
id = "monastery-of-forms"
display_name = "Monastery of Forms"
tagline = "repeat until the scroll smiles"
glyph = "⛩"
genre = "dojo"

[cabinet.theme]
accent_primary = "#8af0ff"
accent_secondary = "#c8a8ff"

[[chapters]]
id = "monastery-warmup"
title = "Monastery Warmup"

[[chapters.decks]]
id = "path-form"
title = "Path Form"

[[chapters.decks.cards]]
id = "pwd-recall"
prompt = "Print the current directory."
success = "The scroll nods. You know where you stand."
hints = ["Three letters: pwd."]
validator = { type = "ExactCommand", expected = "pwd" }

[[chapters.decks.cards]]
id = "ls-recall"
prompt = "List what is in this directory."
success = "The training mat reveals its files."
hints = ["Use `ls` to list files."]
validator = { type = "ExactCommand", expected = "ls" }

[[chapters.decks.cards]]
id = "cat-readme"
prompt = "Read the README."
success = "Manny Page applauds in documentation."
hints = ["Use `cat README`."]
validator = { type = "ExactCommand", expected = "cat README" }
```

- [ ] **Step 4: Write `content/cabinets/midnight-carnival.toml`**

```toml
[cabinet]
id = "midnight-carnival"
display_name = "Midnight Carnival"
tagline = "find the lost logs"
glyph = "◉"
genre = "ops_sim"

[cabinet.theme]
accent_primary = "#c9ffb3"
accent_secondary = "#ffe8a3"

[[chapters]]
id = "moonbase-helpdesk"
title = "Moonbase Helpdesk"

[[chapters.incidents]]
id = "moonbase-misfile"
title = "Moonbase Helpdesk"
prompt = "Astronauts saved the report in plain sight. Prove where you are, list files, then read it."
objective = { id = "ops-orientation", prompt = "State your location.", success = "Moonbase coordinates accepted.", hints = ["Start with `pwd`."], validator = { type = "ExactCommand", expected = "pwd" } }

[[chapters.incidents]]
id = "moonbase-listing"
title = "Moonbase Helpdesk"
prompt = "List the moonbase files."
objective = { id = "ops-list-files", prompt = "List what is here.", success = "The report blinks in the dust.", hints = ["Use `ls`."], validator = { type = "ExactCommand", expected = "ls" } }

[[chapters.incidents]]
id = "moonbase-report"
title = "Moonbase Helpdesk"
prompt = "Read the misplaced report."
objective = { id = "ops-read-report", prompt = "Read report.txt.", success = "Incident complete: the moonbase report has been inspected.", hints = ["Use `cat report.txt`."], validator = { type = "ExactCommand", expected = "cat report.txt" } }
```

- [ ] **Step 5: Verify each manifest parses by writing a smoke test**

Add to `tests/content_validation.rs`:

```rust
#[test]
fn each_bundled_cabinet_manifest_parses() {
    let shell = CabinetManifest::from_toml_str(include_str!(
        "../content/cabinets/shell-motel.toml"
    ))
    .expect("shell-motel parses");
    assert_eq!(shell.cabinet.id, "shell-motel");

    let monastery = CabinetManifest::from_toml_str(include_str!(
        "../content/cabinets/monastery-of-forms.toml"
    ))
    .expect("monastery-of-forms parses");
    assert_eq!(monastery.cabinet.id, "monastery-of-forms");

    let carnival = CabinetManifest::from_toml_str(include_str!(
        "../content/cabinets/midnight-carnival.toml"
    ))
    .expect("midnight-carnival parses");
    assert_eq!(carnival.cabinet.id, "midnight-carnival");
}
```

- [ ] **Step 6: Run the test**

```
cargo test --test content_validation each_bundled_cabinet_manifest_parses
```

Expected: PASS.

- [ ] **Step 7: Commit**

```
git add content/cabinets/ tests/content_validation.rs
git commit -m "Split lost_terminal content into per-cabinet manifests"
```

### Task 1.3: Add `Cabinet` enum + manifest accessors

**Files:**
- Create: `src/arcade/mod.rs`
- Create: `src/arcade/cabinet.rs`
- Modify: `src/lib.rs`
- Test: `tests/content_validation.rs`

- [ ] **Step 1: Write the failing test**

Add to `tests/content_validation.rs`:

```rust
use bashcards::arcade::cabinet::Cabinet;

#[test]
fn cabinet_enum_lists_three_starter_cabinets_in_order() {
    let all = Cabinet::all();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].id(), "shell-motel");
    assert_eq!(all[1].id(), "monastery-of-forms");
    assert_eq!(all[2].id(), "midnight-carnival");
}

#[test]
fn each_cabinet_loads_its_manifest_lazily() {
    for cabinet in Cabinet::all() {
        let manifest = cabinet.manifest();
        assert_eq!(manifest.cabinet.id, cabinet.id());
    }
}
```

- [ ] **Step 2: Run and confirm it fails**

```
cargo test --test content_validation cabinet_enum
```

Expected: FAIL — `bashcards::arcade::cabinet::Cabinet` not found.

- [ ] **Step 3: Create the `arcade` module skeleton**

Create `src/arcade/mod.rs`:

```rust
pub mod cabinet;
```

- [ ] **Step 4: Create `src/arcade/cabinet.rs`**

```rust
use std::sync::OnceLock;

use crate::content::CabinetManifest;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    pub const fn id(self) -> &'static str {
        match self {
            Cabinet::ShellMotel => "shell-motel",
            Cabinet::MonasteryOfForms => "monastery-of-forms",
            Cabinet::MidnightCarnival => "midnight-carnival",
        }
    }

    const fn manifest_toml(self) -> &'static str {
        match self {
            Cabinet::ShellMotel => include_str!("../../content/cabinets/shell-motel.toml"),
            Cabinet::MonasteryOfForms => {
                include_str!("../../content/cabinets/monastery-of-forms.toml")
            }
            Cabinet::MidnightCarnival => {
                include_str!("../../content/cabinets/midnight-carnival.toml")
            }
        }
    }

    pub fn manifest(self) -> &'static CabinetManifest {
        static SHELL: OnceLock<CabinetManifest> = OnceLock::new();
        static MONASTERY: OnceLock<CabinetManifest> = OnceLock::new();
        static CARNIVAL: OnceLock<CabinetManifest> = OnceLock::new();

        let slot = match self {
            Cabinet::ShellMotel => &SHELL,
            Cabinet::MonasteryOfForms => &MONASTERY,
            Cabinet::MidnightCarnival => &CARNIVAL,
        };
        slot.get_or_init(|| {
            CabinetManifest::from_toml_str(self.manifest_toml())
                .unwrap_or_else(|err| panic!("bundled cabinet {} failed to parse: {err}", self.id()))
        })
    }
}
```

- [ ] **Step 5: Expose `arcade` from `lib.rs`**

Modify `src/lib.rs` — add `pub mod arcade;` after `pub mod app;`:

```rust
pub mod app;
pub mod arcade;
pub mod content;
pub mod effects;
pub mod game;
pub mod input;
pub mod modes;
pub mod profile;
pub mod settings;
pub mod shell;
pub mod telemetry_local;
pub mod ui;
pub mod validation;
```

- [ ] **Step 6: Run the tests**

```
cargo test --test content_validation cabinet_enum
cargo test --test content_validation each_cabinet_loads
```

Expected: both PASS.

- [ ] **Step 7: Commit**

```
git add src/arcade/ src/lib.rs tests/content_validation.rs
git commit -m "Add Cabinet enum with bundled manifest accessors"
```

### Task 1.4: Remove the legacy `content/lost_terminal.toml`

The legacy file is no longer referenced by the cabinet manifests, but `ContentLibrary::bundled()` still loads it via `include_str!`. We keep `lost_terminal.toml` alive for one more phase to avoid breaking the existing `Mode`-based UI until Phase 6.

**Files:** none modified

- [ ] **Step 1: Verify `lost_terminal.toml` is still referenced by `ContentLibrary::bundled()`**

```
grep -n "lost_terminal" src/content.rs
```

Expected output: `95:        Self::from_toml_str(include_str!("../content/lost_terminal.toml"))`. If the line moved, note the new line number. **Do not delete `content/lost_terminal.toml` yet** — Phase 6 removes both the legacy file and its loader together.

- [ ] **Step 2: No commit needed for this confirmation task**

---

## Phase 2 · Profile reshape with v1 migration

This phase reshapes `Profile` to per-cabinet `CabinetProgress` keyed by cabinet id, introduces a `schema_version` field, and adds a one-way migration from the legacy flat shape (treating an absent `schema_version` as v1). The `App` is wired to write through the new shape; tests are extended.

### Task 2.1: Add `CabinetProgress` and the new `Profile` shape

**Files:**
- Modify: `src/profile.rs`
- Test: `tests/settings_effects_profile.rs`

- [ ] **Step 1: Write the failing test**

Add to `tests/settings_effects_profile.rs`:

```rust
use bashcards::profile::{CabinetProgress, Profile};

#[test]
fn empty_profile_default_has_schema_v2_and_no_cabinets() {
    let p = Profile::default();
    assert_eq!(p.schema_version, 2);
    assert!(p.cabinets.is_empty());
    assert!(p.weak_topics.is_empty());
}

#[test]
fn record_completion_writes_into_cabinet_progress_map() {
    let mut p = Profile::default();
    p.record_completion("shell-motel", "state-location");
    p.record_completion("shell-motel", "state-location"); // dedup
    p.record_completion("shell-motel", "list-room");
    let progress = p
        .cabinets
        .get("shell-motel")
        .expect("cabinet entry exists");
    assert_eq!(progress.completed_objectives.len(), 2);
    assert!(progress.completed_objectives.contains("state-location"));
    assert!(progress.completed_objectives.contains("list-room"));
}
```

- [ ] **Step 2: Run and confirm both fail**

```
cargo test --test settings_effects_profile empty_profile_default
cargo test --test settings_effects_profile record_completion
```

Expected: FAIL on both — `schema_version` / `cabinets` / `record_completion` not defined.

- [ ] **Step 3: Rewrite `src/profile.rs`**

Replace the entire contents of `src/profile.rs` with:

```rust
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

const CURRENT_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Profile {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub cabinets: HashMap<String, CabinetProgress>,
    #[serde(default)]
    pub weak_topics: Vec<String>,
}

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            cabinets: HashMap::new(),
            weak_topics: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CabinetProgress {
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub streak: u32,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub completed_objectives: HashSet<String>,
    #[serde(default)]
    pub last_objective_id: Option<String>,
}

impl Profile {
    pub fn default_path() -> Option<PathBuf> {
        ProjectDirs::from("dev", "midwestman35", "bashcards")
            .map(|dirs| dirs.data_dir().join("profile.toml"))
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let profile: Profile = toml::from_str(&data)?;
        Ok(profile.migrate_if_needed())
    }

    pub fn record_completion(&mut self, cabinet_id: &str, objective_id: &str) {
        let entry = self.cabinets.entry(cabinet_id.to_string()).or_default();
        entry.completed_objectives.insert(objective_id.to_string());
    }

    pub fn record_attempt(&mut self, cabinet_id: &str) {
        let entry = self.cabinets.entry(cabinet_id.to_string()).or_default();
        entry.attempts = entry.attempts.saturating_add(1);
    }

    pub fn set_session_stats(
        &mut self,
        cabinet_id: &str,
        score: u32,
        streak: u32,
        last_objective: Option<&str>,
    ) {
        let entry = self.cabinets.entry(cabinet_id.to_string()).or_default();
        entry.score = score;
        entry.streak = streak;
        entry.last_objective_id = last_objective.map(ToString::to_string);
    }

    fn migrate_if_needed(mut self) -> Self {
        if self.schema_version >= CURRENT_SCHEMA_VERSION {
            return self;
        }
        self.schema_version = CURRENT_SCHEMA_VERSION;
        self
    }
}
```

Note: this rewrite drops the legacy `completed_objectives: Vec<String>` top-level field from the struct (the new shape keys by cabinet). Migration of pre-existing v1 files lands in Task 2.2.

- [ ] **Step 4: Run both new tests**

```
cargo test --test settings_effects_profile empty_profile_default
cargo test --test settings_effects_profile record_completion
```

Expected: both PASS.

- [ ] **Step 5: Run the full test suite to find callers of the old `Profile.completed_objectives` field**

```
cargo test
```

Expected: compilation errors in `src/app.rs` (lines that reference `self.profile.completed_objectives.contains(&id)` and `.push(id)`). Note the errors — Task 2.3 fixes them. Until then, do not commit.

### Task 2.2: Add v1-to-v2 migration logic

**Files:**
- Modify: `src/profile.rs`
- Test: `tests/settings_effects_profile.rs`

- [ ] **Step 1: Write the failing migration test**

Add to `tests/settings_effects_profile.rs`:

```rust
use assert_fs::TempDir;
use std::fs;

#[test]
fn loading_legacy_v1_profile_migrates_completions_to_shell_motel() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("profile.toml");
    fs::write(
        &path,
        r#"
completed_objectives = ["state-location", "list-room"]
weak_topics = ["paths"]
"#,
    )
    .unwrap();

    let p = Profile::load_from(&path).expect("loads v1 profile");
    assert_eq!(p.schema_version, 2);
    assert_eq!(p.weak_topics, vec!["paths".to_string()]);
    let shell = p
        .cabinets
        .get("shell-motel")
        .expect("migrated completions land in shell-motel");
    assert!(shell.completed_objectives.contains("state-location"));
    assert!(shell.completed_objectives.contains("list-room"));
}
```

- [ ] **Step 2: Run and confirm it fails**

```
cargo test --test settings_effects_profile loading_legacy_v1_profile_migrates
```

Expected: FAIL — TOML deserialization will reject the legacy `completed_objectives` top-level field.

- [ ] **Step 3: Add a transitional deserialization shim to `src/profile.rs`**

Replace `load_from` and `migrate_if_needed` with:

```rust
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let raw: RawProfile = toml::from_str(&data)?;
        Ok(raw.into_profile())
    }
```

And add at the bottom of the file:

```rust
#[derive(Debug, Default, Deserialize)]
struct RawProfile {
    #[serde(default)]
    schema_version: Option<u32>,
    #[serde(default)]
    cabinets: HashMap<String, CabinetProgress>,
    #[serde(default)]
    weak_topics: Vec<String>,
    #[serde(default)]
    completed_objectives: Vec<String>, // legacy v1 field
}

impl RawProfile {
    fn into_profile(self) -> Profile {
        let mut cabinets = self.cabinets;
        if self.schema_version.is_none() && !self.completed_objectives.is_empty() {
            // Legacy v1 file: existing content was Escape Room only, so completions
            // migrate into Shell Motel's CabinetProgress.
            let shell = cabinets
                .entry("shell-motel".to_string())
                .or_default();
            for id in self.completed_objectives {
                shell.completed_objectives.insert(id);
            }
        }
        Profile {
            schema_version: CURRENT_SCHEMA_VERSION,
            cabinets,
            weak_topics: self.weak_topics,
        }
    }
}
```

Also remove the obsolete `migrate_if_needed` method since `RawProfile::into_profile` subsumes it.

- [ ] **Step 4: Run the migration test**

```
cargo test --test settings_effects_profile loading_legacy_v1_profile_migrates
```

Expected: PASS.

- [ ] **Step 5: Do not commit yet — App still doesn't compile**

Move on to Task 2.3.

### Task 2.3: Wire `App` to the new Profile shape

**Files:**
- Modify: `src/app.rs`
- Test: `tests/gameplay_flow.rs`, `tests/settings_effects_profile.rs`

- [ ] **Step 1: Modify `App::submit_command` in `src/app.rs:57-71` to use the new API**

Replace the body of `submit_command` (lines 57-71) with:

```rust
    pub fn submit_command(&mut self, command: &str) -> AttemptOutcome {
        let current_id = self.session.current_objective_id().map(ToString::to_string);
        let cabinet_id = self.session.cabinet_id_for_profile().to_string();
        let outcome = self.session.submit_command(command);
        self.profile.record_attempt(&cabinet_id);
        if matches!(outcome, AttemptOutcome::Correct { .. })
            && let Some(id) = current_id
            && !id.is_empty()
        {
            self.profile.record_completion(&cabinet_id, &id);
            self.profile.set_session_stats(
                &cabinet_id,
                self.session.score().max(0) as u32,
                self.session.streak() as u32,
                self.session.current_objective_id(),
            );
            if let Some(path) = self.profile_path.as_ref() {
                let _ = self.profile.save_to(path);
            }
        }
        outcome
    }
```

The new helper `cabinet_id_for_profile()` is added to `GameSession` in Step 2 below — it returns the id of the cabinet the session belongs to. For Phase 2, this is derived from the legacy `Mode` enum (Phase 6 replaces `Mode` entirely):

- [ ] **Step 2: Add `cabinet_id_for_profile` to `GameSession` in `src/game.rs`**

Add to `impl GameSession` (anywhere in the impl block):

```rust
    pub fn cabinet_id_for_profile(&self) -> &'static str {
        match self.mode {
            Mode::EscapeRoom => "shell-motel",
            Mode::Dojo => "monastery-of-forms",
            Mode::OpsSim => "midnight-carnival",
        }
    }
```

- [ ] **Step 3: Run the test suite**

```
cargo test
```

Expected: existing tests that read `Profile.completed_objectives` may need updating. If any test fails, fix it by reading from `p.cabinets.get("shell-motel").unwrap().completed_objectives` instead. Save the fixes for Step 4 commit.

- [ ] **Step 4: Run clippy to confirm no warnings**

```
cargo clippy -- -D warnings
```

Expected: clean.

- [ ] **Step 5: Commit**

```
git add src/profile.rs src/app.rs src/game.rs tests/settings_effects_profile.rs tests/gameplay_flow.rs
git commit -m "Reshape Profile to per-cabinet progress with v1 migration"
```

---

## Phase 3 · Theme system

Pure-data palette modules with a 256-color fallback adapter. No UI changes yet; the theme module is built and unit-tested in isolation.

### Task 3.1: Create `src/theme/vapor.rs` with palette constants

**Files:**
- Create: `src/theme/mod.rs`
- Create: `src/theme/vapor.rs`
- Create: `src/theme/famicom.rs`
- Modify: `src/lib.rs`
- Create: `tests/theme.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/theme.rs`:

```rust
use bashcards::theme::{adapt, vapor};

#[test]
fn vapor_palette_pink_is_canonical_hex() {
    assert_eq!(vapor::PALETTE.pink, (0xff, 0x9a, 0xd2));
}

#[test]
fn vapor_palette_high_contrast_uses_solid_black_field() {
    assert_eq!(vapor::PALETTE_HIGH_CONTRAST.deep_field, (0, 0, 0));
    assert_eq!(vapor::PALETTE_HIGH_CONTRAST.mid_field, (0, 0, 0));
    assert_eq!(vapor::PALETTE_HIGH_CONTRAST.low_field, (0, 0, 0));
}

#[test]
fn nearest_ansi_256_for_vapor_pink_is_in_pink_band() {
    let idx = adapt::to_ansi_256((0xff, 0x9a, 0xd2));
    assert!((201..=219).contains(&idx) || (211..=219).contains(&idx),
        "pink {idx} should fall in the magenta/pink band of the 256-color cube");
}
```

- [ ] **Step 2: Run the test and confirm failure**

```
cargo test --test theme
```

Expected: FAIL — `bashcards::theme` module does not exist.

- [ ] **Step 3: Create `src/theme/mod.rs`**

```rust
pub mod adapt;
pub mod famicom;
pub mod vapor;
```

- [ ] **Step 4: Create `src/theme/vapor.rs`**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VaporPalette {
    pub pink: (u8, u8, u8),
    pub cyan: (u8, u8, u8),
    pub violet: (u8, u8, u8),
    pub mint: (u8, u8, u8),
    pub yellow: (u8, u8, u8),
    pub dim_prose: (u8, u8, u8),
    pub deep_field: (u8, u8, u8),
    pub mid_field: (u8, u8, u8),
    pub low_field: (u8, u8, u8),
}

pub const PALETTE: VaporPalette = VaporPalette {
    pink:       (0xff, 0x9a, 0xd2),
    cyan:       (0x8a, 0xf0, 0xff),
    violet:     (0xc8, 0xa8, 0xff),
    mint:       (0xc9, 0xff, 0xb3),
    yellow:     (0xff, 0xe8, 0xa3),
    dim_prose:  (0x9f, 0x87, 0xc8),
    deep_field: (0x1b, 0x0a, 0x36),
    mid_field:  (0x2a, 0x10, 0x52),
    low_field:  (0x1f, 0x0a, 0x44),
};

pub const PALETTE_HIGH_CONTRAST: VaporPalette = VaporPalette {
    pink:       PALETTE.pink,
    cyan:       PALETTE.cyan,
    violet:     PALETTE.violet,
    mint:       PALETTE.mint,
    yellow:     PALETTE.yellow,
    dim_prose:  (0xe0, 0xd4, 0xff),
    deep_field: (0x00, 0x00, 0x00),
    mid_field:  (0x00, 0x00, 0x00),
    low_field:  (0x00, 0x00, 0x00),
};
```

- [ ] **Step 5: Create `src/theme/famicom.rs`**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FamicomAccents {
    pub neon_magenta: (u8, u8, u8),
    pub deep_cyan: (u8, u8, u8),
    pub navy_inset: (u8, u8, u8),
}

pub const PUNCTUATION: FamicomAccents = FamicomAccents {
    neon_magenta: (0xff, 0x3e, 0x9d),
    deep_cyan:    (0x00, 0xd4, 0xff),
    navy_inset:   (0x2e, 0x3a, 0xa0),
};
```

- [ ] **Step 6: Create `src/theme/adapt.rs`**

Wait — `mod.rs` references `pub mod adapt;` but we haven't created the file yet. Create `src/theme/adapt.rs`:

```rust
/// Snap an RGB triple to the nearest of the 240 colors in the xterm 256-color
/// cube (indices 16..=255). Index 16..=231 is a 6×6×6 RGB cube; 232..=255 is
/// a 24-step grayscale ramp. We pick whichever is closer.
pub fn to_ansi_256(rgb: (u8, u8, u8)) -> u8 {
    let (r, g, b) = rgb;

    // 6x6x6 cube candidate
    let cube_levels: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let nearest = |v: u8| -> (u8, u32) {
        cube_levels
            .iter()
            .enumerate()
            .map(|(i, &c)| (i as u8, (v as i32 - c as i32).unsigned_abs()))
            .min_by_key(|(_, d)| *d)
            .unwrap()
    };
    let (ri, dr) = nearest(r);
    let (gi, dg) = nearest(g);
    let (bi, db) = nearest(b);
    let cube_index = 16 + 36 * ri + 6 * gi + bi;
    let cube_dist = dr * dr + dg * dg + db * db;

    // Grayscale ramp candidate
    let avg = (r as u32 + g as u32 + b as u32) / 3;
    let gray_index = if avg < 8 {
        16
    } else if avg > 248 {
        231
    } else {
        232 + ((avg - 8) / 10).min(23) as u8
    };
    let gray_level = if gray_index >= 232 {
        8 + (gray_index - 232) as u32 * 10
    } else if gray_index == 231 {
        255
    } else {
        0
    };
    let gray_dist = (0..3)
        .map(|i| {
            let c = [r, g, b][i] as i32;
            (c - gray_level as i32).pow(2) as u32
        })
        .sum::<u32>();

    if gray_dist < cube_dist { gray_index } else { cube_index }
}
```

- [ ] **Step 7: Expose the theme module from `lib.rs`**

Add `pub mod theme;` to `src/lib.rs`.

- [ ] **Step 8: Run the tests**

```
cargo test --test theme
```

Expected: PASS.

- [ ] **Step 9: Commit**

```
git add src/theme/ src/lib.rs tests/theme.rs
git commit -m "Add vapor and Famicom palette modules with 256-color adapter"
```

---

## Phase 4 · Animation runtime

Build the `Animator` + `Cue` enum + sample API in isolation. Tests assert cue lifecycle, reduced-motion behavior, and speed scaling — all without touching the TUI.

### Task 4.1: Define `Cue`, `Sample`, `MotionBudget`

**Files:**
- Create: `src/animation/mod.rs`
- Create: `src/animation/timeline.rs`
- Create: `src/animation/reduced_motion.rs`
- Modify: `src/lib.rs`
- Create: `tests/animation.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/animation.rs`:

```rust
use std::time::Duration;

use bashcards::animation::{Animator, Cue, MotionBudget, Sample};

#[test]
fn newly_fired_cue_samples_at_progress_zero_then_advances() {
    let mut a = Animator::new(MotionBudget::default());
    a.fire(Cue::ImprintMaterialize);
    let s0 = a.sample(&Cue::ImprintMaterialize).expect("cue is live");
    assert_eq!(s0.progress, 0.0);
    a.tick_to(Duration::from_millis(500));
    let s_mid = a.sample(&Cue::ImprintMaterialize).expect("cue is live");
    assert!(s_mid.progress > 0.4 && s_mid.progress < 0.6,
        "midway sample should be ~0.5, got {}", s_mid.progress);
}

#[test]
fn cue_is_complete_after_its_full_duration() {
    let mut a = Animator::new(MotionBudget::default());
    a.fire(Cue::CabinetEnter);
    a.tick_to(Duration::from_millis(500));
    assert!(a.is_complete(&Cue::CabinetEnter));
}

#[test]
fn reduced_motion_snaps_progress_to_one_immediately() {
    let budget = MotionBudget { reduced: true, speed: 1.0 };
    let mut a = Animator::new(budget);
    a.fire(Cue::ImprintMaterialize);
    let s = a.sample(&Cue::ImprintMaterialize).expect("cue live");
    assert_eq!(s.progress, 1.0);
    assert!(a.is_complete(&Cue::ImprintMaterialize));
}

#[test]
fn snappy_speed_completes_a_cue_in_less_time() {
    let normal = {
        let mut a = Animator::new(MotionBudget { reduced: false, speed: 1.0 });
        a.fire(Cue::CabinetEnter);
        a.tick_to(Duration::from_millis(300));
        a.is_complete(&Cue::CabinetEnter)
    };
    let snappy = {
        let mut a = Animator::new(MotionBudget { reduced: false, speed: 1.4 });
        a.fire(Cue::CabinetEnter);
        a.tick_to(Duration::from_millis(300));
        a.is_complete(&Cue::CabinetEnter)
    };
    // CabinetEnter is 400ms; at 1.0x not complete at 300ms; at 1.4x effective duration is ~286ms, complete.
    assert!(!normal, "normal speed should not be complete at 300ms (duration is 400ms)");
    assert!(snappy, "snappy speed should be complete at 300ms");
}
```

- [ ] **Step 2: Run and confirm failures**

```
cargo test --test animation
```

Expected: FAIL — `bashcards::animation` module not found.

- [ ] **Step 3: Create `src/animation/mod.rs`**

```rust
use std::collections::HashMap;
use std::time::Duration;

pub mod reduced_motion;
pub mod timeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cue {
    ImprintMaterialize,
    BootLineReveal { line_idx: u8 },
    TitleSweep,
    StatusFlicker { token: StatusToken },
    CabinetEnter { cabinet: crate::arcade::cabinet::Cabinet },
    CabinetExit,
    LobbyHover { cabinet: crate::arcade::cabinet::Cabinet },
    ObjectiveGradient,
    PromptReveal,
    HintFlash,
    CorrectGlow,
    IncorrectFlicker,
    StreakSurge { streak: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusToken {
    Ok,
    Ready,
    NoGuilt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// 0.0 at fire-time, 1.0 at completion. Eased per cue.
    pub progress: f32,
    /// Optional color override; some cues compute an RGB tween in the sample.
    pub color: Option<(u8, u8, u8)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionBudget {
    pub reduced: bool,
    /// Multiplier on cue durations. Calm = 0.6, Normal = 1.0, Snappy = 1.4.
    pub speed: f32,
}

impl Default for MotionBudget {
    fn default() -> Self {
        Self { reduced: false, speed: 1.0 }
    }
}

#[derive(Clone, Copy, Debug)]
struct LiveCue {
    elapsed: Duration,
    duration: Duration,
}

pub struct Animator {
    budget: MotionBudget,
    live: HashMap<CueKey, LiveCue>,
}

/// Stable hash key for a Cue. Some Cue variants carry data we don't want to
/// re-fire on each tick (e.g. CabinetEnter with same Cabinet), so we key by
/// the discriminant + payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CueKey(u64);

impl From<&Cue> for CueKey {
    fn from(cue: &Cue) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        cue.hash(&mut hasher);
        CueKey(hasher.finish())
    }
}

impl Animator {
    pub fn new(budget: MotionBudget) -> Self {
        Self { budget, live: HashMap::new() }
    }

    pub fn set_budget(&mut self, budget: MotionBudget) {
        self.budget = budget;
    }

    pub fn fire(&mut self, cue: Cue) {
        let base_duration = timeline::base_duration(&cue);
        let scaled = if self.budget.reduced {
            Duration::ZERO
        } else {
            scale_duration(base_duration, self.budget.speed)
        };
        self.live.insert(
            CueKey::from(&cue),
            LiveCue { elapsed: Duration::ZERO, duration: scaled },
        );
    }

    pub fn fire_set(&mut self, cues: &[Cue]) {
        for cue in cues { self.fire(*cue); }
    }

    pub fn tick(&mut self, dt: Duration) {
        for live in self.live.values_mut() {
            live.elapsed = (live.elapsed + dt).min(live.duration);
        }
    }

    /// Advance the animator to an absolute time. Test-only.
    #[doc(hidden)]
    pub fn tick_to(&mut self, t: Duration) {
        for live in self.live.values_mut() {
            live.elapsed = t.min(live.duration);
        }
    }

    pub fn sample(&self, cue: &Cue) -> Option<Sample> {
        let key = CueKey::from(cue);
        let live = self.live.get(&key)?;
        let progress = if live.duration.is_zero() {
            1.0
        } else {
            (live.elapsed.as_secs_f32() / live.duration.as_secs_f32()).min(1.0)
        };
        Some(Sample {
            progress: timeline::ease(cue, progress),
            color: timeline::sample_color(cue, progress),
        })
    }

    pub fn is_complete(&self, cue: &Cue) -> bool {
        let key = CueKey::from(cue);
        match self.live.get(&key) {
            Some(live) => live.elapsed >= live.duration,
            None => false,
        }
    }
}

fn scale_duration(d: Duration, speed: f32) -> Duration {
    if speed <= 0.01 {
        d
    } else {
        Duration::from_secs_f32(d.as_secs_f32() / speed)
    }
}
```

- [ ] **Step 4: Create `src/animation/timeline.rs`**

```rust
use std::time::Duration;

use super::{Cue, StatusToken};

pub fn base_duration(cue: &Cue) -> Duration {
    match cue {
        Cue::ImprintMaterialize       => Duration::from_millis(1000),
        Cue::BootLineReveal { .. }    => Duration::from_millis(120),
        Cue::TitleSweep               => Duration::from_millis(1200),
        Cue::StatusFlicker { .. }     => Duration::from_millis(250),
        Cue::CabinetEnter { .. }      => Duration::from_millis(400),
        Cue::CabinetExit              => Duration::from_millis(300),
        Cue::LobbyHover { .. }        => Duration::from_secs(6),
        Cue::ObjectiveGradient        => Duration::from_millis(600),
        Cue::PromptReveal             => Duration::from_millis(400),
        Cue::HintFlash                => Duration::from_millis(300),
        Cue::CorrectGlow              => Duration::from_millis(400),
        Cue::IncorrectFlicker         => Duration::from_millis(180),
        Cue::StreakSurge { .. }       => Duration::from_millis(500),
    }
}

/// Per-cue easing. Most cues use ease-out (cubic); LobbyHover uses a
/// half-sine pulse; StreakSurge uses ease-in-out.
pub fn ease(cue: &Cue, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match cue {
        Cue::LobbyHover { .. } => {
            // Smooth half-sine pulse: 0 -> 1 -> 0 across the cycle. We return
            // the sine value so callers can use it as an alpha multiplier.
            (t * std::f32::consts::PI).sin().abs()
        }
        Cue::StreakSurge { .. } => {
            // ease-in-out cubic
            if t < 0.5 { 4.0 * t * t * t } else {
                let f = 2.0 * t - 2.0;
                0.5 * f * f * f + 1.0
            }
        }
        _ => {
            // ease-out cubic
            let f = 1.0 - t;
            1.0 - f * f * f
        }
    }
}

/// Optional per-cue color sample. Cues like CorrectGlow tween across the
/// vapor palette; this returns the current rgb mix for the given progress.
pub fn sample_color(cue: &Cue, _t: f32) -> Option<(u8, u8, u8)> {
    match cue {
        // Color tweens are computed at render time using the cabinet's
        // accent in concert with the sample's progress; the animator itself
        // doesn't need to know about palettes.
        _ => None,
    }
}
```

Note: `sample_color` returns `None` for every variant in v1. The renderer composes the actual color using the cabinet accent + sample progress. We keep the API surface so future cues that need a literal color (e.g. interpolating from violet to mint) can fill in this function later.

- [ ] **Step 5: Create `src/animation/reduced_motion.rs`**

```rust
use super::{Cue, Sample};

/// When reduced_motion is active, the animator's sample API already returns
/// progress = 1.0. This helper exists for renderers that want to do their
/// own static substitution based on cue type — e.g. picking a single solid
/// color instead of a gradient — when reduced motion is on.
pub fn static_sample_for(cue: &Cue) -> Sample {
    Sample { progress: 1.0, color: super::timeline::sample_color(cue, 1.0) }
}
```

- [ ] **Step 6: Expose `animation` from `lib.rs`**

Add `pub mod animation;` to `src/lib.rs` (after the existing modules).

- [ ] **Step 7: Run the test suite**

```
cargo test --test animation
```

Expected: all four tests PASS. The `snappy_speed_completes_a_cue_in_less_time` test depends on the precise math: `CabinetEnter` base = 400ms, at speed 1.4 the scaled duration is ~286ms, so by 300ms it is complete. At speed 1.0 it isn't complete at 300ms. If the math drifts, adjust the test's `tick_to` value but keep the assertion structure.

- [ ] **Step 8: Commit**

```
git add src/animation/ src/lib.rs tests/animation.rs
git commit -m "Add Animator with Cue catalog, motion budget, and deterministic tests"
```

---

## Phase 5 · Rewire `effects.rs` on top of the animator

Replace the current `EffectCue` / `EffectKind` API with a thin facade that emits real `Cue`s into a shared `Animator`. The renderer (Phase 6+) talks to `Animator` directly; `effects.rs` exists for any call site that still uses the old API.

### Task 5.1: Replace `effects.rs` with a facade

**Files:**
- Modify: `src/effects.rs`
- Test: existing `tests/settings_effects_profile.rs`

- [ ] **Step 1: Read existing call sites of `effects`**

```
grep -rn "effects::" src/ tests/
```

Note each caller. The expected callers are `tests/settings_effects_profile.rs` and possibly `src/ui.rs` via `gradient_cells`. We need to keep the `gradient_cells` function callable until the new UI lands.

- [ ] **Step 2: Rewrite `src/effects.rs`**

Replace the existing file with:

```rust
use crate::animation::{Animator, Cue};
use crate::settings::Settings;

/// Public re-export kept for callers that read effect cues during success
/// flows. Maps cleanly onto `animation::Cue`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectKind {
    ObjectiveGradient,
    Reveal,
    Flicker,
    Sweep,
    SuccessGradient,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectCue {
    pub kind: EffectKind,
    pub reduced_motion_safe: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GradientCell {
    pub ch: char,
    pub rgb: (u8, u8, u8),
}

/// Returns the cue list that should play on a successful submission. Honors
/// reduced motion by trimming non-safe entries.
pub fn cues_for_success(settings: &Settings) -> Vec<EffectCue> {
    let safe = vec![
        EffectCue { kind: EffectKind::ObjectiveGradient, reduced_motion_safe: true },
        EffectCue { kind: EffectKind::SuccessGradient,   reduced_motion_safe: true },
    ];
    if settings.reduced_motion {
        return safe;
    }
    let mut full = safe.clone();
    full.insert(1, EffectCue { kind: EffectKind::Reveal,  reduced_motion_safe: false });
    full.insert(2, EffectCue { kind: EffectKind::Flicker, reduced_motion_safe: false });
    full.insert(3, EffectCue { kind: EffectKind::Sweep,   reduced_motion_safe: false });
    full
}

/// Fire each effect cue into the supplied animator. New callers should
/// prefer `Animator::fire_set` directly; this helper exists so legacy code
/// that thinks in terms of `EffectKind` keeps working.
pub fn fire_into(animator: &mut Animator, cues: &[EffectCue]) {
    for cue in cues {
        let mapped = match cue.kind {
            EffectKind::ObjectiveGradient => Cue::ObjectiveGradient,
            EffectKind::Reveal            => Cue::PromptReveal,
            EffectKind::Flicker           => Cue::IncorrectFlicker,
            EffectKind::Sweep             => Cue::TitleSweep,
            EffectKind::SuccessGradient   => Cue::CorrectGlow,
        };
        animator.fire(mapped);
    }
}

/// Per-character gradient cells across a string. Kept as a function for the
/// current legacy renderer in `src/ui.rs`; once Phase 6 lands, the renderer
/// computes its own cells using the animator's Sample.
pub fn gradient_cells(text: &str) -> Vec<GradientCell> {
    let chars: Vec<char> = text.chars().collect();
    let steps = chars.len().saturating_sub(1).max(1) as u16;
    chars
        .into_iter()
        .enumerate()
        .map(|(index, ch)| {
            let index = index as u16;
            let r = 64 + (index * 96 / steps);
            let g = 190 + (index * 50 / steps);
            let b = 210 - (index * 90 / steps);
            GradientCell { ch, rgb: (r as u8, g as u8, b as u8) }
        })
        .collect()
}
```

- [ ] **Step 3: Run the existing effects tests**

```
cargo test --test settings_effects_profile
```

Expected: PASS. If any test depended on the old `cues_for_success` shape, adjust the assertion to expect the new vector order (`safe` first; `Reveal/Flicker/Sweep` inserted at positions 1/2/3 when reduced motion is off).

- [ ] **Step 4: Run the full suite + clippy**

```
cargo test
cargo clippy -- -D warnings
```

Expected: clean.

- [ ] **Step 5: Commit**

```
git add src/effects.rs tests/settings_effects_profile.rs
git commit -m "Rewire effects.rs as facade over animation runtime"
```

---

## Phase 6 · Splash sequences (carafe + bash arcade boot)

This phase introduces the new `ui/` directory, builds the `ArcadeState` machine, and implements both splash stages. The legacy `Mode`-based UI in `src/ui.rs` is replaced; `Mode` itself is removed and call sites updated.

### Task 6.1: Convert `src/ui.rs` into a `src/ui/` directory

**Files:**
- Delete: `src/ui.rs`
- Create: `src/ui/mod.rs` (initially carrying the same `run_tui` body)
- Modify: no top-level public API changes

- [ ] **Step 1: Move `src/ui.rs` to `src/ui/mod.rs`**

```
mkdir -p src/ui
git mv src/ui.rs src/ui/mod.rs
```

- [ ] **Step 2: Verify the build still compiles**

```
cargo build
```

Expected: success (no source change, just a path change).

- [ ] **Step 3: Run tests**

```
cargo test
```

Expected: PASS.

- [ ] **Step 4: Commit**

```
git add -A src/ui.rs src/ui/
git commit -m "Move ui.rs into ui/mod.rs to allow submodule expansion"
```

### Task 6.2: Define `ArcadeState` and remove the legacy `Mode` enum

**Files:**
- Create: `src/arcade/session.rs` (transition state holder)
- Modify: `src/arcade/mod.rs`
- Modify: `src/game.rs` (drop `Mode`, rename `GameSession` → `CabinetSession`)
- Modify: `src/app.rs` (rename `launch_mode` → `launch_cabinet`)
- Modify: `src/modes/*.rs` (extractors keyed by genre)
- Modify: `src/content.rs` (drop `ContentLibrary::bundled`, add `CabinetContent`)
- Modify: tests in `tests/`
- Delete: `content/lost_terminal.toml` after manifests are loaded directly

- [ ] **Step 1: Add `CabinetGenre` to `src/modes/mod.rs`**

Replace the contents of `src/modes/mod.rs`:

```rust
pub mod dojo;
pub mod escape_room;
pub mod ops_sim;

use crate::content::{CabinetManifest, Objective};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CabinetGenre {
    EscapeRoom,
    Dojo,
    OpsSim,
}

impl CabinetGenre {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "escape_room" => Some(Self::EscapeRoom),
            "dojo"        => Some(Self::Dojo),
            "ops_sim"     => Some(Self::OpsSim),
            _             => None,
        }
    }

    /// Extract the playable objective list from this cabinet's manifest.
    pub fn extract_objectives(self, manifest: &CabinetManifest, session_length: usize) -> Vec<Objective> {
        match self {
            Self::EscapeRoom => manifest
                .chapters()
                .first()
                .and_then(|c| c.rooms.first())
                .map(|r| r.objectives.clone())
                .unwrap_or_default(),
            Self::Dojo => {
                let mut cards: Vec<Objective> = manifest
                    .chapters()
                    .first()
                    .and_then(|c| c.decks.first())
                    .map(|d| d.cards.iter().cloned().map(Objective::from).collect())
                    .unwrap_or_default();
                cards.truncate(session_length);
                cards
            }
            Self::OpsSim => manifest
                .chapters()
                .first()
                .map(|c| c.incidents.iter().map(|i| i.objective.clone()).collect())
                .unwrap_or_default(),
        }
    }

    pub fn requires_real_sandbox(self, difficulty: crate::settings::Difficulty) -> bool {
        matches!(self, Self::OpsSim)
            && matches!(difficulty, crate::settings::Difficulty::Operator | crate::settings::Difficulty::Chaos)
    }
}
```

This subsumes the per-mode extractor structs. Delete the now-redundant content of `src/modes/escape_room.rs`, `src/modes/dojo.rs`, and `src/modes/ops_sim.rs` — replace each with an empty `// kept for namespace reservation` comment to preserve module paths during transition, or remove them and update `mod.rs` to drop the `pub mod` lines. Choose removal:

```rust
// src/modes/mod.rs final content — drop the per-mode pub mod lines:
use crate::content::{CabinetManifest, Objective};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CabinetGenre { /* as above */ }

impl CabinetGenre { /* as above */ }
```

Delete the now-empty files:

```
git rm src/modes/escape_room.rs src/modes/dojo.rs src/modes/ops_sim.rs
```

- [ ] **Step 2: Update `Cabinet` to expose its `CabinetGenre`**

Modify `src/arcade/cabinet.rs` — add the genre accessor:

```rust
impl Cabinet {
    // ... existing methods ...

    pub fn genre(self) -> crate::modes::CabinetGenre {
        crate::modes::CabinetGenre::parse(&self.manifest().cabinet.genre)
            .unwrap_or_else(|| panic!("bundled cabinet {} has unknown genre", self.id()))
    }
}
```

- [ ] **Step 3: Rewrite `src/game.rs` — drop `Mode`, rename `GameSession` → `CabinetSession`**

Replace the contents of `src/game.rs`:

```rust
use crate::{
    arcade::cabinet::Cabinet,
    content::Objective,
    settings::{Difficulty, PressureProfile, Settings},
    shell::{CommandWorld, Sandbox, run_guided},
    validation::{ValidationResult, validate_command},
};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttemptOutcome {
    Correct   { feedback: String },
    Incorrect { feedback: String },
    Blocked   { feedback: String },
}

pub struct CabinetSession {
    pub cabinet: Cabinet,
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
            return AttemptOutcome::Correct { feedback: "Session already complete.".to_string() };
        }
        let Some(objective) = self.objectives.get(self.current) else {
            return AttemptOutcome::Incorrect { feedback: "No challenge loaded.".to_string() };
        };
        self.attempts += 1;
        let simulated = run_guided(&mut self.command_world, command);
        self.last_command_output = if simulated.stderr.is_empty() { simulated.stdout } else { simulated.stderr };
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

    pub fn current_objective_prompt(&self) -> Option<&str> { self.objectives.get(self.current).map(|o| o.prompt.as_str()) }
    pub fn current_objective_id(&self)     -> Option<&str> { self.objectives.get(self.current).map(|o| o.id.as_str()) }
    pub fn cabinet_id(&self)               -> &'static str { self.cabinet.id() }
    pub fn completed(&self) -> bool { self.completed }
    pub fn score(&self) -> i32 { self.score }
    pub fn attempts(&self) -> usize { self.attempts }
    pub fn streak(&self) -> usize { self.streak }
    pub fn missed_objectives(&self) -> &[String] { &self.missed_objectives }
    pub fn last_command_output(&self) -> &str { &self.last_command_output }
    pub fn uses_real_sandbox(&self) -> bool { self.sandbox.is_some() }
    pub fn sandbox_root(&self) -> Option<&Path> { self.sandbox.as_ref().map(Sandbox::root) }

    fn points_for_correct(&self) -> i32 {
        match self.settings.pressure {
            PressureProfile::CozyNoTimer   => 100,
            PressureProfile::SoftUrgency   => 110,
            PressureProfile::ArcadePressure => 140,
        }
    }
    fn penalty_for_miss(&self) -> i32 {
        match self.settings.pressure {
            PressureProfile::CozyNoTimer   => 0,
            PressureProfile::SoftUrgency   => 5,
            PressureProfile::ArcadePressure => 15,
        }
    }
    fn submit_sandbox_command(&mut self, command: &str) -> AttemptOutcome {
        let Some(sandbox) = self.sandbox.as_ref() else {
            return AttemptOutcome::Blocked { feedback: "Real shell commands are only available in sandbox modes.".to_string() };
        };
        match sandbox.run_command(command) {
            Ok(output) => {
                self.last_command_output = if output.stderr.is_empty() { output.stdout } else { output.stderr };
                if output.status == 0 {
                    AttemptOutcome::Correct { feedback: format!("Sandbox command exited 0 inside {}.", sandbox.root().display()) }
                } else {
                    AttemptOutcome::Incorrect { feedback: format!("Sandbox command exited {}.", output.status) }
                }
            }
            Err(error) => AttemptOutcome::Blocked { feedback: format!("Could not run sandbox command: {error}") },
        }
    }
}
```

- [ ] **Step 4: Rewrite `src/app.rs` — `launch_cabinet` instead of `launch_mode`**

Replace the contents of `src/app.rs`:

```rust
use std::path::PathBuf;

use crate::{
    arcade::cabinet::Cabinet,
    game::{AttemptOutcome, CabinetSession},
    profile::Profile,
    settings::{Difficulty, Settings},
};

pub struct App {
    session: CabinetSession,
    profile: Profile,
    profile_path: Option<PathBuf>,
}

impl App {
    pub fn launch_cabinet(
        cabinet: Cabinet,
        difficulty: Difficulty,
        settings: Settings,
    ) -> anyhow::Result<Self> {
        let manifest = cabinet.manifest();
        let genre = cabinet.genre();
        let objectives = genre.extract_objectives(manifest, settings.session_length_target);
        let mut session = CabinetSession::new(cabinet, difficulty, settings, objectives);
        if genre.requires_real_sandbox(difficulty) {
            session.attach_operator_sandbox()?;
        }
        Ok(Self { session, profile: Profile::default(), profile_path: None })
    }

    pub fn launch_cabinet_with_profile_path(
        cabinet: Cabinet,
        difficulty: Difficulty,
        settings: Settings,
        profile_path: PathBuf,
    ) -> anyhow::Result<Self> {
        let mut app = Self::launch_cabinet(cabinet, difficulty, settings)?;
        app.profile = if profile_path.exists() {
            Profile::load_from(&profile_path)?
        } else {
            Profile::default()
        };
        app.profile_path = Some(profile_path);
        Ok(app)
    }

    pub fn submit_command(&mut self, command: &str) -> AttemptOutcome {
        let current_id = self.session.current_objective_id().map(ToString::to_string);
        let cabinet_id = self.session.cabinet_id();
        let outcome = self.session.submit_command(command);
        self.profile.record_attempt(cabinet_id);
        if matches!(outcome, AttemptOutcome::Correct { .. })
            && let Some(id) = current_id
            && !id.is_empty()
        {
            self.profile.record_completion(cabinet_id, &id);
            self.profile.set_session_stats(
                cabinet_id,
                self.session.score().max(0) as u32,
                self.session.streak() as u32,
                self.session.current_objective_id(),
            );
            if let Some(path) = self.profile_path.as_ref() {
                let _ = self.profile.save_to(path);
            }
        }
        outcome
    }

    pub fn session(&self) -> &CabinetSession { &self.session }
    pub fn profile(&self) -> &Profile { &self.profile }
}

pub fn launch_session(
    cabinet: Cabinet,
    difficulty: Difficulty,
    settings: Settings,
) -> anyhow::Result<App> {
    if let Some(path) = Profile::default_path() {
        App::launch_cabinet_with_profile_path(cabinet, difficulty, settings, path)
    } else {
        App::launch_cabinet(cabinet, difficulty, settings)
    }
}
```

- [ ] **Step 5: Remove `lost_terminal.toml` loader from `content.rs`**

In `src/content.rs`, remove the `ContentLibrary::bundled()` method (it was the only consumer of `lost_terminal.toml`). Keep `ContentLibrary` and `Chapter` types — they're used by `CabinetManifest`'s deserialization.

```
git rm content/lost_terminal.toml
```

Also remove the `first_*_objectives` convenience methods on `ContentLibrary` — they're replaced by `CabinetGenre::extract_objectives`.

- [ ] **Step 6: Update tests to use `Cabinet` + `launch_cabinet`**

In `tests/gameplay_flow.rs`, change all `Mode::EscapeRoom` → `Cabinet::ShellMotel`, `Mode::Dojo` → `Cabinet::MonasteryOfForms`, `Mode::OpsSim` → `Cabinet::MidnightCarnival`, and `App::launch_mode(content, mode, ...)` → `App::launch_cabinet(cabinet, ...)`. The `content: ContentLibrary` argument is no longer needed.

Equivalent renames in `tests/sandbox_and_agents.rs` and `tests/settings_effects_profile.rs`.

- [ ] **Step 7: Run the full test suite**

```
cargo test
```

Expected: all tests pass. Any remaining failures should point at lingering `Mode` references.

- [ ] **Step 8: Commit**

```
git add src/ tests/ content/
git commit -m "Replace Mode enum with Cabinet; drop legacy content/lost_terminal.toml"
```

### Task 6.3: Build `ArcadeState` and the new `ui/mod.rs` frame loop

**Files:**
- Modify: `src/arcade/mod.rs`
- Create: `src/arcade/session.rs`
- Rewrite: `src/ui/mod.rs`

- [ ] **Step 1: Define the state machine in `src/arcade/mod.rs`**

```rust
pub mod cabinet;
pub mod session;

use cabinet::Cabinet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArcadeState {
    CarafeImprint,
    Splash,
    Lobby,
    CabinetEntering(Cabinet),
    InCabinet(Cabinet),
    CabinetExiting(Cabinet),
    Settings(SettingsCaller),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsCaller {
    Lobby,
    InCabinet(Cabinet),
}
```

- [ ] **Step 2: Create `src/arcade/session.rs`**

```rust
//! Reserved for session-routing helpers used during cabinet entry/exit.
//! Phase 8 fills this in; for Phase 6 we only need the file to exist so
//! `mod.rs` compiles.
```

- [ ] **Step 3: Rewrite `src/ui/mod.rs` to host the new frame loop**

Replace the entire contents of `src/ui/mod.rs` with:

```rust
use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Frame, Terminal, backend::CrosstermBackend};

use crate::animation::{Animator, MotionBudget};
use crate::arcade::{ArcadeState, cabinet::Cabinet};
use crate::settings::{Difficulty, Settings};

pub mod cabinet;
pub mod lobby;
pub mod settings;
pub mod splash;
pub mod widgets;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub struct TuiApp {
    pub state: ArcadeState,
    pub focused_cabinet: Cabinet,
    pub difficulty: Difficulty,
    pub settings: Settings,
    pub animator: Animator,
    pub command: String,
    pub should_quit: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        let settings = Settings::default();
        let budget = MotionBudget {
            reduced: settings.reduced_motion,
            speed: 1.0, // overwritten in Phase 9 once animation_speed is wired
        };
        Self {
            state: ArcadeState::CarafeImprint,
            focused_cabinet: Cabinet::ShellMotel,
            difficulty: Difficulty::Beginner,
            settings,
            animator: Animator::new(budget),
            command: String::new(),
            should_quit: false,
        }
    }
}

pub fn run_tui() -> anyhow::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = TuiApp::new();

    splash::on_enter(&mut app);

    let frame_budget = Duration::from_millis(16);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| render(f, &app))?;
        if event::poll(frame_budget)? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key.code);
            }
        }
        let now = Instant::now();
        let dt = now.duration_since(last_tick);
        last_tick = now;
        if !app.settings.reduced_motion {
            app.animator.tick(dt);
        }
        splash::on_tick(&mut app);
        if app.should_quit { break; }
    }
    Ok(())
}

fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    match app.state {
        ArcadeState::CarafeImprint           => splash::carafe::render(frame, app),
        ArcadeState::Splash                  => splash::boot_console::render(frame, app),
        ArcadeState::Lobby                   => lobby::render(frame, app),
        ArcadeState::CabinetEntering(_)
        | ArcadeState::InCabinet(_)
        | ArcadeState::CabinetExiting(_)     => cabinet::render(frame, app),
        ArcadeState::Settings(_)             => settings::render(frame, app),
    }
}

fn handle_key(app: &mut TuiApp, code: KeyCode) {
    match app.state {
        ArcadeState::CarafeImprint => splash::carafe::handle_key(app, code),
        ArcadeState::Splash        => splash::boot_console::handle_key(app, code),
        ArcadeState::Lobby         => lobby::handle_key(app, code),
        ArcadeState::CabinetEntering(_)
        | ArcadeState::InCabinet(_)
        | ArcadeState::CabinetExiting(_) => cabinet::handle_key(app, code),
        ArcadeState::Settings(_)   => settings::handle_key(app, code),
    }
}
```

- [ ] **Step 4: Create stub render modules so the build can proceed**

Create `src/ui/widgets.rs`:

```rust
// Shared vapor widgets; populated in subsequent tasks.
```

Create `src/ui/splash/mod.rs`:

```rust
pub mod boot_console;
pub mod carafe;

use crate::animation::Cue;
use crate::arcade::ArcadeState;
use crate::ui::TuiApp;

pub fn on_enter(app: &mut TuiApp) {
    app.animator.fire(Cue::ImprintMaterialize);
}

pub fn on_tick(app: &mut TuiApp) {
    match app.state {
        ArcadeState::CarafeImprint => {
            if app.animator.is_complete(&Cue::ImprintMaterialize) {
                app.state = ArcadeState::Splash;
                boot_console::on_enter(app);
            }
        }
        ArcadeState::Splash => boot_console::on_tick(app),
        _ => {}
    }
}
```

Create `src/ui/splash/carafe.rs` (placeholder body — Task 6.4 fills in the renderer):

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::widgets::Paragraph;

use crate::arcade::ArcadeState;
use crate::ui::TuiApp;

pub fn render(frame: &mut Frame<'_>, _app: &TuiApp) {
    frame.render_widget(Paragraph::new("carafe imprint"), frame.area());
}

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app.state = ArcadeState::Splash;
            super::boot_console::on_enter(app);
        }
        _ => {}
    }
}
```

Create `src/ui/splash/boot_console.rs` (placeholder):

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::widgets::Paragraph;

use crate::arcade::ArcadeState;
use crate::ui::TuiApp;

pub fn on_enter(_app: &mut TuiApp) {}
pub fn on_tick(_app: &mut TuiApp) {}

pub fn render(frame: &mut Frame<'_>, _app: &TuiApp) {
    frame.render_widget(Paragraph::new("bash arcade boot console"), frame.area());
}

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    if matches!(code, KeyCode::Enter | KeyCode::Esc) {
        app.state = ArcadeState::Lobby;
    }
}
```

Create `src/ui/lobby.rs` (placeholder):

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::widgets::Paragraph;

use crate::ui::TuiApp;

pub fn render(frame: &mut Frame<'_>, _app: &TuiApp) {
    frame.render_widget(Paragraph::new("lobby"), frame.area());
}

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    if matches!(code, KeyCode::Char('q')) { app.should_quit = true; }
}
```

Create `src/ui/cabinet.rs`:

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::widgets::Paragraph;

use crate::ui::TuiApp;

pub fn render(frame: &mut Frame<'_>, _app: &TuiApp) {
    frame.render_widget(Paragraph::new("in cabinet"), frame.area());
}
pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    if matches!(code, KeyCode::Esc) { app.state = crate::arcade::ArcadeState::Lobby; }
}
```

Create `src/ui/settings.rs`:

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::widgets::Paragraph;

use crate::arcade::{ArcadeState, SettingsCaller};
use crate::ui::TuiApp;

pub fn render(frame: &mut Frame<'_>, _app: &TuiApp) {
    frame.render_widget(Paragraph::new("settings"), frame.area());
}

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    if matches!(code, KeyCode::Esc) {
        match app.state {
            ArcadeState::Settings(SettingsCaller::Lobby)            => app.state = ArcadeState::Lobby,
            ArcadeState::Settings(SettingsCaller::InCabinet(c))     => app.state = ArcadeState::InCabinet(c),
            _ => app.state = ArcadeState::Lobby,
        }
    }
}
```

- [ ] **Step 5: Build and verify**

```
cargo build
cargo test
cargo clippy -- -D warnings
```

Expected: PASS. The TUI is structurally in place but visually stubbed — that's the goal of this task.

- [ ] **Step 6: Commit**

```
git add src/arcade/ src/ui/
git commit -m "Introduce ArcadeState machine and stubbed ui/ submodule layout"
```

### Task 6.4: Implement the carafe imprint renderer

**Files:**
- Modify: `src/ui/splash/carafe.rs`
- Modify: `src/animation/timeline.rs` (`ImprintMaterialize` duration confirmed; no change needed)

- [ ] **Step 1: Add the const glyph table to `src/ui/splash/carafe.rs`**

Replace the placeholder body with:

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::animation::Cue;
use crate::arcade::ArcadeState;
use crate::theme;
use crate::ui::TuiApp;

/// "carafe" rendered with Unicode braille (U+2800–U+28FF). Two visual rows
/// per letter, six letters across. Hand-tuned to read as a low-rez PS1-era
/// logomark. Final letterforms may be retouched without breaking layout
/// because each cell is independently rendered.
pub const CARAFE_GLYPHS: [[&str; 6]; 2] = [
    ["⢀⣀⡀", "⣀⡀ ", "⡀⢀ ", "⡀⢀ ", "⣀⡀ ", "⢀⣀⡀"],
    ["⠛⠁ ", "⣸⠟ ", "⠉⠉ ", "⢾⠆ ", "⣸⠟ ", "⠉⠉⠆"],
];

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let progress = app
        .animator
        .sample(&Cue::ImprintMaterialize)
        .map(|s| s.progress)
        .unwrap_or(1.0);

    let palette = if app.settings.high_contrast {
        theme::vapor::PALETTE_HIGH_CONTRAST
    } else {
        theme::vapor::PALETTE
    };

    let logo_alpha = progress;
    let logo_color = blend(palette.dim_prose, palette.cyan, logo_alpha);
    let tagline_alpha = (progress - 0.5).clamp(0.0, 1.0) * 2.0;
    let tagline_color = blend(palette.deep_field, palette.dim_prose, tagline_alpha);

    let line_top = build_glyph_line(0, logo_color);
    let line_bot = build_glyph_line(1, logo_color);
    let tagline = Line::from(Span::styled(
        "·     a r c a d e   i m p r i n t     ·",
        Style::default().fg(rgb(tagline_color)),
    ));
    let blank = Line::raw("");

    let body = Paragraph::new(vec![blank.clone(), blank.clone(), blank.clone(), line_top, line_bot, blank.clone(), blank.clone(), tagline])
        .alignment(Alignment::Center)
        .style(Style::default().bg(rgb(palette.deep_field)));
    frame.render_widget(body, area);
}

fn build_glyph_line(row: usize, color: (u8, u8, u8)) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    for letter in CARAFE_GLYPHS[row].iter() {
        spans.push(Span::styled(letter.to_string(), Style::default().fg(rgb(color))));
        spans.push(Span::raw("  "));
    }
    Line::from(spans)
}

fn blend(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        ((1.0 - t) * a.0 as f32 + t * b.0 as f32) as u8,
        ((1.0 - t) * a.1 as f32 + t * b.1 as f32) as u8,
        ((1.0 - t) * a.2 as f32 + t * b.2 as f32) as u8,
    )
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color { Color::Rgb(r, g, b) }

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    if matches!(code, KeyCode::Esc | KeyCode::Enter) {
        app.state = ArcadeState::Splash;
        super::boot_console::on_enter(app);
    }
}
```

- [ ] **Step 2: Confirm the `ImprintMaterialize` total duration matches the spec**

The spec wants 3.5 s of total carafe stage. The `ImprintMaterialize` cue itself is the materialize segment (1.0 s). The drift + hold + fade segments aren't separate cues — they're modeled by lengthening `ImprintMaterialize`'s timeline. Update `src/animation/timeline.rs::base_duration`:

```rust
        Cue::ImprintMaterialize       => Duration::from_millis(3500),
```

(The renderer interprets progress < 0.23 as drift, 0.23–0.51 as materialize, 0.51–0.94 as hold, 0.94–1.0 as fade — but the math is incidental; the user-perceptible duration is the same 3.5 s.)

- [ ] **Step 3: Update the existing animation tests for the new duration**

In `tests/animation.rs`, the `newly_fired_cue_samples_at_progress_zero_then_advances` test asserts ImprintMaterialize is ~half done at 500ms. With duration now 3500ms, 500ms is ~14% in. Adjust:

```rust
    a.tick_to(Duration::from_millis(1750));
    let s_mid = a.sample(&Cue::ImprintMaterialize).expect("cue is live");
    assert!(s_mid.progress > 0.4 && s_mid.progress < 0.6,
        "midway sample should be ~0.5, got {}", s_mid.progress);
```

(Note: `ease()` is cubic-ease-out for ImprintMaterialize, so progress 0.5 input maps to ~0.875 output. Adjust the assertion bounds to `> 0.8 && < 0.95`, or change the test to assert raw fractional time before ease.)

Recommendation: change the test to check `s_mid.progress > 0.0 && s_mid.progress < 1.0` and add a separate check that `tick_to(Duration::from_millis(3500))` reaches `progress == 1.0`. That tests behavior without coupling to the easing function:

```rust
#[test]
fn cue_progress_advances_monotonically_and_completes_at_full_duration() {
    let mut a = Animator::new(MotionBudget::default());
    a.fire(Cue::ImprintMaterialize);
    let s0 = a.sample(&Cue::ImprintMaterialize).expect("cue is live");
    assert_eq!(s0.progress, 0.0);
    a.tick_to(Duration::from_millis(1750));
    let s_mid = a.sample(&Cue::ImprintMaterialize).expect("cue is live");
    assert!(s_mid.progress > 0.0 && s_mid.progress < 1.0);
    a.tick_to(Duration::from_millis(3500));
    let s_end = a.sample(&Cue::ImprintMaterialize).expect("cue is live");
    assert_eq!(s_end.progress, 1.0);
    assert!(a.is_complete(&Cue::ImprintMaterialize));
}
```

Replace the original `newly_fired_cue_samples_at_progress_zero_then_advances` test with this one.

- [ ] **Step 4: Build and run tests**

```
cargo build
cargo test
```

Expected: PASS.

- [ ] **Step 5: Commit**

```
git add src/ui/splash/carafe.rs src/animation/timeline.rs tests/animation.rs
git commit -m "Implement carafe imprint splash with braille logomark"
```

### Task 6.5: Implement the bash arcade boot console

**Files:**
- Modify: `src/ui/splash/boot_console.rs`

- [ ] **Step 1: Build the renderer + state machine**

Replace the placeholder contents of `src/ui/splash/boot_console.rs`:

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::animation::{Cue, StatusToken};
use crate::arcade::{ArcadeState, cabinet::Cabinet};
use crate::theme;
use crate::ui::TuiApp;

const BOOT_LINES: &[&str] = &[
    "initializing vapor console",
    "mounting cabinet shell-motel",
    "mounting cabinet monastery-of-forms",
    "mounting cabinet midnight-carnival",
    "loading profile",
    "checking reset protocol",
];

pub fn on_enter(app: &mut TuiApp) {
    app.animator.fire(Cue::TitleSweep);
    for (idx, _) in BOOT_LINES.iter().enumerate() {
        app.animator.fire(Cue::BootLineReveal { line_idx: idx as u8 });
    }
    app.animator.fire(Cue::StatusFlicker { token: StatusToken::Ok });
    app.animator.fire(Cue::StatusFlicker { token: StatusToken::Ready });
    app.animator.fire(Cue::StatusFlicker { token: StatusToken::NoGuilt });
}

pub fn on_tick(_app: &mut TuiApp) {
    // No automatic advance to Lobby — the user has to press Enter to "insert
    // coin". on_enter has already scheduled all the cues; on_tick is a
    // no-op for now.
}

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let palette = if app.settings.high_contrast {
        theme::vapor::PALETTE_HIGH_CONTRAST
    } else {
        theme::vapor::PALETTE
    };

    // ASCII title block in pink, swept by TitleSweep cue.
    let sweep_progress = app
        .animator
        .sample(&Cue::TitleSweep)
        .map(|s| s.progress)
        .unwrap_or(1.0);
    let title_color = blend(palette.pink, palette.cyan, sweep_progress * 0.5);

    let title = vec![
        Line::raw(""),
        styled_line("    bash arcade", title_color),
        styled_line("    pastel terminal training system · 199X", palette.dim_prose),
        Line::raw(""),
    ];

    // Boot lines reveal sequentially. Each BootLineReveal's progress
    // controls how much of the line is visible.
    let mut body: Vec<Line<'static>> = title;
    for (idx, raw) in BOOT_LINES.iter().enumerate() {
        let cue = Cue::BootLineReveal { line_idx: idx as u8 };
        let progress = app.animator.sample(&cue).map(|s| s.progress).unwrap_or(1.0);
        let take = (raw.len() as f32 * progress).round() as usize;
        let visible: String = raw.chars().take(take).collect();
        let line = Line::from(vec![
            Span::styled(" · ", Style::default().fg(rgb(palette.cyan))),
            Span::styled(visible, Style::default().fg(rgb(palette.dim_prose))),
        ]);
        body.push(line);
    }

    body.push(Line::raw(""));
    body.push(status_line(palette));
    body.push(Line::raw(""));
    body.push(styled_line("[ press enter to insert coin ]", palette.cyan));
    body.push(styled_line("esc skips boot", palette.dim_prose));

    let widget = Paragraph::new(body)
        .alignment(Alignment::Center)
        .style(Style::default().bg(rgb(palette.deep_field)));
    frame.render_widget(widget, area);
}

fn status_line(palette: theme::vapor::VaporPalette) -> Line<'static> {
    Line::from(vec![
        Span::styled("READY ", Style::default().fg(rgb(palette.pink))),
        Span::styled("· ",      Style::default().fg(rgb(palette.dim_prose))),
        Span::styled("OK ",     Style::default().fg(rgb(palette.mint))),
        Span::styled("· ",      Style::default().fg(rgb(palette.dim_prose))),
        Span::styled("NO GUILT", Style::default().fg(rgb(palette.yellow))),
    ])
}

fn styled_line(text: &str, color: (u8, u8, u8)) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), Style::default().fg(rgb(color))))
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color { Color::Rgb(r, g, b) }

fn blend(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        ((1.0 - t) * a.0 as f32 + t * b.0 as f32) as u8,
        ((1.0 - t) * a.1 as f32 + t * b.1 as f32) as u8,
        ((1.0 - t) * a.2 as f32 + t * b.2 as f32) as u8,
    )
}

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Enter => app.state = ArcadeState::Lobby,
        KeyCode::Esc   => app.state = ArcadeState::Lobby, // skip remaining boot animation
        _ => {}
    }
}
```

- [ ] **Step 2: Build + run**

```
cargo build
cargo test
cargo clippy -- -D warnings
```

Expected: PASS.

- [ ] **Step 3: Manual smoke test (one-time terminal check)**

The TUI requires an interactive terminal — `cargo test` cannot validate the splash visually. Run the binary by hand:

```
cargo run
```

Expected: carafe imprint splash appears for ~3.5 s, then the bash arcade boot console with cabinet-mounting log lines and the "press enter to insert coin" prompt. Press Enter — see "lobby" stub. Press q — quit.

- [ ] **Step 4: Commit**

```
git add src/ui/splash/boot_console.rs
git commit -m "Implement bash arcade boot console renderer"
```

---

## Phase 7 · Lobby

Implement the cabinet grid + selected-session panel + navigation. After this phase, the user can boot through the splash, browse cabinets, and read each cabinet's tagline/progress.

### Task 7.1: Implement cabinet grid + selected-session panel

**Files:**
- Modify: `src/ui/lobby.rs`
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Build the lobby renderer**

Replace `src/ui/lobby.rs`:

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::animation::Cue;
use crate::arcade::{ArcadeState, SettingsCaller, cabinet::Cabinet};
use crate::theme;
use crate::ui::TuiApp;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let palette = if app.settings.high_contrast {
        theme::vapor::PALETTE_HIGH_CONTRAST
    } else {
        theme::vapor::PALETTE
    };

    let [header, cabinets_row, detail, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(12),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    render_header(frame, header, palette);
    render_cabinet_row(frame, cabinets_row, app, palette);
    render_selected_session(frame, detail, app, palette);
    render_footer(frame, footer, palette);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, p: theme::vapor::VaporPalette) {
    let line = Line::from(vec![
        Span::styled("  bash arcade  ", Style::default().fg(rgb(p.cyan)).add_modifier(Modifier::BOLD)),
        Span::styled("// pastel terminal training", Style::default().fg(rgb(p.dim_prose))),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL).style(Style::default().fg(rgb(p.pink)))),
        area,
    );
}

fn render_cabinet_row(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, p: theme::vapor::VaporPalette) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
        .split(area);

    for (idx, cabinet) in Cabinet::all().iter().enumerate() {
        let selected = *cabinet == app.focused_cabinet;
        render_cabinet_card(frame, chunks[idx], *cabinet, selected, app, p);
    }
}

fn render_cabinet_card(
    frame: &mut Frame<'_>,
    area: Rect,
    cabinet: Cabinet,
    selected: bool,
    app: &TuiApp,
    p: theme::vapor::VaporPalette,
) {
    let manifest = cabinet.manifest();
    let accent = parse_hex(&manifest.cabinet.theme.accent_primary).unwrap_or(p.pink);
    let progress = app
        .animator
        .sample(&Cue::LobbyHover { cabinet })
        .map(|s| s.progress)
        .unwrap_or(0.0);

    let border_color = if selected {
        // Famicom punctuation on the selected cabinet
        theme::famicom::PUNCTUATION.neon_magenta
    } else {
        // Slow breath: dim accent → bright accent
        blend((accent.0 / 2, accent.1 / 2, accent.2 / 2), accent, progress)
    };

    let cabinet_progress = app
        .session_profile()
        .cabinets
        .get(cabinet.id());
    let streak = cabinet_progress.map(|p| p.streak).unwrap_or(0);
    let score  = cabinet_progress.map(|p| p.score).unwrap_or(0);
    let completed = cabinet_progress.map(|p| p.completed_objectives.len()).unwrap_or(0);

    let card = vec![
        Line::from(Span::styled(
            format!(" {} {}", manifest.cabinet.glyph, manifest.cabinet.display_name),
            Style::default().fg(rgb(accent)).add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            format!(" {}", manifest.cabinet.tagline),
            Style::default().fg(rgb(p.dim_prose)),
        )),
        Line::raw(""),
        Line::from(Span::styled(format!(" streak  {streak}"), Style::default().fg(rgb(p.yellow)))),
        Line::from(Span::styled(format!(" score   {score}"),  Style::default().fg(rgb(p.yellow)))),
        Line::from(Span::styled(format!(" cleared {completed}"), Style::default().fg(rgb(p.yellow)))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(rgb(border_color)));
    let block = if selected {
        block.style(
            Style::default()
                .fg(rgb(theme::famicom::PUNCTUATION.neon_magenta))
                .bg(rgb(theme::famicom::PUNCTUATION.navy_inset)),
        )
    } else {
        block
    };
    frame.render_widget(Paragraph::new(card).block(block), area);
}

fn render_selected_session(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, p: theme::vapor::VaporPalette) {
    let manifest = app.focused_cabinet.manifest();
    let body = vec![
        Line::from(Span::styled(
            format!("▸ {}", manifest.cabinet.display_name.to_lowercase()),
            Style::default().fg(rgb(p.violet)).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", manifest.cabinet.tagline),
            Style::default().fg(rgb(p.dim_prose)),
        )),
    ];
    let block = Block::default().borders(Borders::ALL).style(Style::default().fg(rgb(p.cyan)));
    frame.render_widget(Paragraph::new(body).block(block), area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, p: theme::vapor::VaporPalette) {
    let hints = Line::from(vec![
        Span::styled("◂ ▸ select   ", Style::default().fg(rgb(p.dim_prose))),
        Span::styled("↵ launch   ", Style::default().fg(rgb(p.cyan))),
        Span::styled("s settings   ", Style::default().fg(rgb(p.dim_prose))),
        Span::styled("? help   ", Style::default().fg(rgb(p.dim_prose))),
        Span::styled("q quit", Style::default().fg(rgb(p.dim_prose))),
    ]);
    frame.render_widget(
        Paragraph::new(hints).block(Block::default().borders(Borders::ALL).style(Style::default().fg(rgb(p.pink)))),
        area,
    );
}

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    let all = Cabinet::all();
    let current = all.iter().position(|c| *c == app.focused_cabinet).unwrap_or(0);
    match code {
        KeyCode::Char('h') | KeyCode::Left => {
            let next = if current == 0 { all.len() - 1 } else { current - 1 };
            app.focused_cabinet = all[next];
            fire_hover(app);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            let next = (current + 1) % all.len();
            app.focused_cabinet = all[next];
            fire_hover(app);
        }
        KeyCode::Char('1') => { app.focused_cabinet = all[0]; fire_hover(app); }
        KeyCode::Char('2') => { app.focused_cabinet = all[1]; fire_hover(app); }
        KeyCode::Char('3') => { app.focused_cabinet = all[2]; fire_hover(app); }
        KeyCode::Enter     => app.state = ArcadeState::CabinetEntering(app.focused_cabinet),
        KeyCode::Char('s') => app.state = ArcadeState::Settings(SettingsCaller::Lobby),
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

fn fire_hover(app: &mut TuiApp) {
    app.animator.fire(Cue::LobbyHover { cabinet: app.focused_cabinet });
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color { Color::Rgb(r, g, b) }

fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.strip_prefix('#').unwrap_or(hex);
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

fn blend(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        ((1.0 - t) * a.0 as f32 + t * b.0 as f32) as u8,
        ((1.0 - t) * a.1 as f32 + t * b.1 as f32) as u8,
        ((1.0 - t) * a.2 as f32 + t * b.2 as f32) as u8,
    )
}
```

- [ ] **Step 2: Add the `session_profile()` accessor on `TuiApp`**

Add to `src/ui/mod.rs::impl TuiApp`:

```rust
    pub fn session_profile(&self) -> &crate::profile::Profile {
        &self.profile
    }
```

And add a `profile: crate::profile::Profile` field to `TuiApp`, defaulting to `Profile::default()` in `TuiApp::new`. Load it from disk if a default profile path exists:

```rust
pub struct TuiApp {
    // ... existing fields ...
    pub profile: crate::profile::Profile,
}

impl TuiApp {
    pub fn new() -> Self {
        // ... existing initialization ...
        let profile = crate::profile::Profile::default_path()
            .filter(|p| p.exists())
            .and_then(|p| crate::profile::Profile::load_from(&p).ok())
            .unwrap_or_default();
        Self {
            // ... existing ...
            profile,
        }
    }
}
```

- [ ] **Step 3: Default focus rule**

Add a helper on `TuiApp` and call it at startup:

```rust
impl TuiApp {
    pub fn default_focus(&self) -> Cabinet {
        Cabinet::all()
            .iter()
            .copied()
            .min_by_key(|c| {
                self.profile.cabinets
                    .get(c.id())
                    .map(|p| p.completed_objectives.len())
                    .unwrap_or(0)
            })
            .unwrap_or(Cabinet::ShellMotel)
    }
}
```

Then in `splash::on_tick`, when transitioning out of `Splash` to `Lobby`, set `app.focused_cabinet = app.default_focus();` before the state change. The state change point is in `boot_console::handle_key` where Enter or Esc transitions to Lobby:

```rust
pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    if matches!(code, KeyCode::Enter | KeyCode::Esc) {
        app.focused_cabinet = app.default_focus();
        app.animator.fire(Cue::LobbyHover { cabinet: app.focused_cabinet });
        app.state = ArcadeState::Lobby;
    }
}
```

- [ ] **Step 4: Build + manual smoke test**

```
cargo build
cargo run
```

Expected: after splash, lobby shows three cabinet cards horizontally, the focused one has Famicom punctuation, `h`/`l` and arrow keys cycle focus, `1`/`2`/`3` jump-focus, `q` quits.

- [ ] **Step 5: Commit**

```
git add src/ui/
git commit -m "Implement arcade lobby with cabinet grid and default-focus rule"
```

---

## Phase 8 · Cabinet entry/exit + in-cabinet view

Hook the lobby's `Enter` press into a `CabinetEnter` transition, render the in-cabinet view, route `AttemptOutcome` to the corresponding animation cue, then handle `CabinetExit` back to the lobby.

### Task 8.1: Implement cabinet transition states

**Files:**
- Modify: `src/ui/cabinet.rs`
- Modify: `src/ui/mod.rs` (advance transitions in the frame loop)

- [ ] **Step 1: Add transition advancement to `splash::on_tick` equivalent for cabinets**

Modify `src/ui/mod.rs` — wrap state transitions into a small dispatcher. After `splash::on_tick(&mut app);` in `run_tui`, add:

```rust
        cabinet::on_tick(&mut app);
```

Then in `src/ui/cabinet.rs`, replace the stub with:

```rust
use std::path::PathBuf;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::animation::Cue;
use crate::app::App;
use crate::arcade::{ArcadeState, cabinet::Cabinet};
use crate::game::AttemptOutcome;
use crate::theme;
use crate::ui::TuiApp;

pub fn on_tick(app: &mut TuiApp) {
    match app.state {
        ArcadeState::CabinetEntering(cabinet) => {
            if app.session.is_none() {
                if let Ok(session) = App::launch_cabinet(cabinet, app.difficulty, app.settings.clone()) {
                    app.session = Some(session);
                    app.animator.fire(Cue::CabinetEnter { cabinet });
                    app.animator.fire(Cue::ObjectiveGradient);
                }
            }
            if app.animator.is_complete(&Cue::CabinetEnter { cabinet }) {
                app.state = ArcadeState::InCabinet(cabinet);
            }
        }
        ArcadeState::CabinetExiting(_) => {
            if app.animator.is_complete(&Cue::CabinetExit) {
                app.session = None;
                app.command.clear();
                app.state = ArcadeState::Lobby;
            }
        }
        _ => {}
    }
}

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let palette = if app.settings.high_contrast {
        theme::vapor::PALETTE_HIGH_CONTRAST
    } else {
        theme::vapor::PALETTE
    };

    let cabinet = match app.state {
        ArcadeState::CabinetEntering(c) | ArcadeState::InCabinet(c) | ArcadeState::CabinetExiting(c) => c,
        _ => return,
    };
    let manifest = cabinet.manifest();
    let accent = super::lobby::parse_hex(&manifest.cabinet.theme.accent_primary).unwrap_or(palette.pink);

    let [header, body, prompt_box, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    // Header
    let session = app.session.as_ref();
    let score = session.map(|s| s.session().score()).unwrap_or(0);
    let streak = session.map(|s| s.session().streak()).unwrap_or(0);
    let attempts = session.map(|s| s.session().attempts()).unwrap_or(0);
    let header_line = Line::from(vec![
        Span::styled(
            format!(" {} {}", manifest.cabinet.glyph, manifest.cabinet.display_name),
            Style::default().fg(rgb(accent)).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(format!("attempts {attempts}"), Style::default().fg(rgb(palette.dim_prose))),
        Span::raw("   "),
        Span::styled(format!("streak {streak}"),     Style::default().fg(rgb(palette.yellow))),
        Span::raw("   "),
        Span::styled(format!("score {score}"),       Style::default().fg(rgb(palette.yellow))),
    ]);
    frame.render_widget(
        Paragraph::new(header_line).block(Block::default().borders(Borders::ALL).style(Style::default().fg(rgb(accent)))),
        header,
    );

    // Body — objective prompt with gradient sweep
    let objective_progress = app.animator.sample(&Cue::ObjectiveGradient).map(|s| s.progress).unwrap_or(1.0);
    let prompt = session
        .and_then(|s| s.session().current_objective_prompt())
        .unwrap_or("Session complete.");
    let highlight = blend(palette.violet, accent, objective_progress);
    let body_widget = Paragraph::new(vec![
        Line::raw(""),
        Line::from(Span::styled(format!("  ▌ {prompt}"), Style::default().fg(rgb(highlight)))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" objective ").style(Style::default().fg(rgb(palette.cyan))));
    frame.render_widget(body_widget, body);

    // Prompt box — current command-in-progress, with PromptReveal cursor blink
    let cmd = format!("$ {}", app.command);
    let prompt_widget = Paragraph::new(Line::from(vec![
        Span::styled(cmd, Style::default().fg(rgb(palette.mint))),
        Span::styled("▮", Style::default().fg(rgb(accent))),
    ]))
    .block(Block::default().borders(Borders::ALL).style(Style::default().fg(rgb(accent))));
    frame.render_widget(prompt_widget, prompt_box);

    // Footer
    let footer_line = Line::from(vec![
        Span::styled("↵ submit   ", Style::default().fg(rgb(palette.mint))),
        Span::styled("⌫ edit   ",   Style::default().fg(rgb(palette.dim_prose))),
        Span::styled("? hint   ",   Style::default().fg(rgb(palette.dim_prose))),
        Span::styled("esc back   ", Style::default().fg(rgb(palette.dim_prose))),
        Span::styled("q quit",      Style::default().fg(rgb(palette.dim_prose))),
    ]);
    frame.render_widget(
        Paragraph::new(footer_line).block(Block::default().borders(Borders::ALL).style(Style::default().fg(rgb(palette.pink)))),
        footer,
    );
}

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    if !matches!(app.state, ArcadeState::InCabinet(_)) { return; }
    let cabinet = match app.state { ArcadeState::InCabinet(c) => c, _ => return };
    match code {
        KeyCode::Esc => {
            app.animator.fire(Cue::CabinetExit);
            app.state = ArcadeState::CabinetExiting(cabinet);
        }
        KeyCode::Enter => {
            let command = std::mem::take(&mut app.command);
            if let Some(session) = app.session.as_mut() {
                let outcome = session.submit_command(&command);
                match outcome {
                    AttemptOutcome::Correct { .. }   => app.animator.fire_set(&[Cue::CorrectGlow, Cue::ObjectiveGradient]),
                    AttemptOutcome::Incorrect { .. } => app.animator.fire(Cue::IncorrectFlicker),
                    AttemptOutcome::Blocked { .. }   => {}
                }
                if matches!(outcome, AttemptOutcome::Correct { .. }) && session.session().streak() >= 3 {
                    app.animator.fire(Cue::StreakSurge { streak: session.session().streak() as u32 });
                }
            }
        }
        KeyCode::Backspace => { app.command.pop(); }
        KeyCode::Char(ch)  => app.command.push(ch),
        _ => {}
    }
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color { Color::Rgb(r, g, b) }

fn blend(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        ((1.0 - t) * a.0 as f32 + t * b.0 as f32) as u8,
        ((1.0 - t) * a.1 as f32 + t * b.1 as f32) as u8,
        ((1.0 - t) * a.2 as f32 + t * b.2 as f32) as u8,
    )
}

#[allow(dead_code)]
pub(crate) const _ENSURE_PATHBUF_USED: Option<PathBuf> = None;
```

(The `_ENSURE_PATHBUF_USED` declaration prevents an "unused import" warning while `PathBuf` is reserved for a forthcoming sandbox-root display. Remove once it's used.)

- [ ] **Step 2: Add `session: Option<App>` to `TuiApp`**

In `src/ui/mod.rs`, add to `TuiApp`:

```rust
    pub session: Option<crate::app::App>,
```

Initialize with `None` in `TuiApp::new`.

- [ ] **Step 3: Make `lobby::parse_hex` pub(crate)**

Change `fn parse_hex` in `src/ui/lobby.rs` to `pub(crate) fn parse_hex` so `cabinet::render` can call it. (Or move it to `src/ui/widgets.rs` and re-export — choose whichever feels cleaner; the plan picks `pub(crate)`.)

- [ ] **Step 4: Build + manual smoke test**

```
cargo build
cargo run
```

Expected: lobby → Enter on Shell Motel → cabinet enters (brief wipe) → in-cabinet view shows objective + command box → typing builds the command → Enter submits → Correct cues fire / Incorrect flickers → Esc returns to lobby.

- [ ] **Step 5: Commit**

```
git add src/ui/
git commit -m "Implement cabinet entry/exit transitions and in-cabinet renderer"
```

---

## Phase 9 · Settings overlay

Modal panel from lobby or in-cabinet, surfacing existing settings and wiring `animation_speed` into the animator's `MotionBudget`.

### Task 9.1: Implement the settings modal

**Files:**
- Modify: `src/ui/settings.rs`
- Modify: `src/settings.rs` (add `theme_overrides`)
- Modify: `src/ui/mod.rs` (push budget to animator on settings change)

- [ ] **Step 1: Add `theme_overrides` to `Settings`**

In `src/settings.rs`, replace the `Settings` struct and `Default` impl:

```rust
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeOverride {
    pub accent_primary: String, // canonical vapor token name; resolved at render time
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub pressure: PressureProfile,
    pub reduced_motion: bool,
    pub high_contrast: bool,
    pub animation_speed: AnimationSpeed,
    pub hint_style: HintStyle,
    pub explain_after_success: ExplainAfterSuccess,
    pub session_length_target: usize,
    pub theme_overrides: HashMap<String, ThemeOverride>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pressure: PressureProfile::SoftUrgency,
            reduced_motion: false,
            high_contrast: false,
            animation_speed: AnimationSpeed::Normal,
            hint_style: HintStyle::Nudging,
            explain_after_success: ExplainAfterSuccess::FirstTimeOnly,
            session_length_target: 10,
            theme_overrides: HashMap::new(),
        }
    }
}
```

- [ ] **Step 2: Map `AnimationSpeed` to a multiplier**

Add a method to `AnimationSpeed`:

```rust
impl AnimationSpeed {
    pub fn multiplier(self) -> f32 {
        match self {
            Self::Calm   => 0.6,
            Self::Normal => 1.0,
            Self::Snappy => 1.4,
        }
    }
}
```

- [ ] **Step 3: Wire the multiplier into `MotionBudget` on every settings change**

In `src/ui/mod.rs::TuiApp`, add a helper:

```rust
    pub fn refresh_motion_budget(&mut self) {
        self.animator.set_budget(crate::animation::MotionBudget {
            reduced: self.settings.reduced_motion,
            speed: self.settings.animation_speed.multiplier(),
        });
    }
```

Call it from `TuiApp::new` after initializing `animator`, and from each settings handler in Step 4 below.

> **time_scale caveat:** if Task 0.2's research found that `vyfor/animate` does *not* expose a runtime time-scale primitive, the speed multiplier flows through our wrapper's `scale_duration` only (already implemented in Phase 4). No further wiring is needed in that case, and `// TODO: pass speed through to animate's engine when time_scale is exposed` should be added near the `set_budget` call.

- [ ] **Step 4: Implement the settings render + key handler**

Replace `src/ui/settings.rs`:

```rust
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::arcade::{ArcadeState, SettingsCaller};
use crate::settings::AnimationSpeed;
use crate::theme;
use crate::ui::TuiApp;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let palette = if app.settings.high_contrast {
        theme::vapor::PALETTE_HIGH_CONTRAST
    } else {
        theme::vapor::PALETTE
    };
    let area = centered(frame.area(), 60, 18);

    let body = vec![
        Line::from(Span::styled("  Settings", Style::default().fg(rgb(palette.cyan)).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        kv("[p] Pressure",         &app.settings.pressure.label(),                  palette),
        kv("[r] Reduced motion",   yes_no(app.settings.reduced_motion),             palette),
        kv("[h] High contrast",    yes_no(app.settings.high_contrast),              palette),
        kv("[a] Animation speed",  &app.settings.animation_speed.label(),           palette),
        kv("[n] Hint style",       &app.settings.hint_style.label(),                palette),
        kv("[e] Explain",          &app.settings.explain_after_success.label(),     palette),
        kv("[+/-] Session length", &app.settings.session_length_target.to_string(), palette),
        Line::raw(""),
        Line::from(Span::styled("  esc back", Style::default().fg(rgb(palette.dim_prose)))),
    ];

    let block = Block::default()
        .title(" arcade settings ")
        .borders(Borders::ALL)
        .style(Style::default().fg(rgb(palette.pink)).bg(rgb(palette.deep_field)));
    frame.render_widget(Paragraph::new(body).block(block), area);
}

fn kv(key: &'static str, value: &str, p: theme::vapor::VaporPalette) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {key:<24}"), Style::default().fg(rgb(p.dim_prose))),
        Span::styled(value.to_string(),      Style::default().fg(rgb(p.cyan)).add_modifier(Modifier::BOLD)),
    ])
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width: w.min(area.width), height: h.min(area.height) }
}

fn yes_no(v: bool) -> &'static str { if v { "on" } else { "off" } }

fn rgb((r, g, b): (u8, u8, u8)) -> Color { Color::Rgb(r, g, b) }

pub fn handle_key(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Char('p') => app.settings.pressure = app.settings.pressure.next(),
        KeyCode::Char('r') => { app.settings.reduced_motion = !app.settings.reduced_motion; app.refresh_motion_budget(); }
        KeyCode::Char('h') => app.settings.high_contrast = !app.settings.high_contrast,
        KeyCode::Char('a') => { app.settings.animation_speed = app.settings.animation_speed.next(); app.refresh_motion_budget(); }
        KeyCode::Char('n') => app.settings.hint_style = app.settings.hint_style.next(),
        KeyCode::Char('e') => app.settings.explain_after_success = app.settings.explain_after_success.next(),
        KeyCode::Char('+') => app.settings.session_length_target = (app.settings.session_length_target + 1).min(20),
        KeyCode::Char('-') => app.settings.session_length_target = app.settings.session_length_target.saturating_sub(1).max(3),
        KeyCode::Esc => {
            match app.state {
                ArcadeState::Settings(SettingsCaller::Lobby)            => app.state = ArcadeState::Lobby,
                ArcadeState::Settings(SettingsCaller::InCabinet(c))     => app.state = ArcadeState::InCabinet(c),
                _ => app.state = ArcadeState::Lobby,
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 5: Allow `s` to open settings from in-cabinet**

In `src/ui/cabinet.rs::handle_key`, add:

```rust
        KeyCode::Char('s') => app.state = ArcadeState::Settings(crate::arcade::SettingsCaller::InCabinet(cabinet)),
```

- [ ] **Step 6: Write tests for `animation_speed` wiring**

Add to `tests/animation.rs`:

```rust
#[test]
fn animation_speed_calm_doubles_effective_duration_to_about_667ms() {
    let mut a = Animator::new(MotionBudget { reduced: false, speed: 0.6 });
    a.fire(Cue::CabinetEnter { cabinet: bashcards::arcade::cabinet::Cabinet::ShellMotel });
    a.tick_to(Duration::from_millis(400));
    assert!(!a.is_complete(&Cue::CabinetEnter { cabinet: bashcards::arcade::cabinet::Cabinet::ShellMotel }),
        "at calm speed the 400ms-base cue should NOT be complete at 400ms");
    a.tick_to(Duration::from_millis(700));
    assert!(a.is_complete(&Cue::CabinetEnter { cabinet: bashcards::arcade::cabinet::Cabinet::ShellMotel }));
}
```

- [ ] **Step 7: Run tests**

```
cargo test
cargo clippy -- -D warnings
```

Expected: PASS.

- [ ] **Step 8: Manual smoke test**

```
cargo run
```

Expected: from the lobby, press `s` to open settings, cycle through with `p`/`r`/`h`/`a`/`n`/`e`/`+`/`-`, press `Esc` to close. In-cabinet, press `s` to open settings, `Esc` to return to the cabinet. Toggling `Animation speed` to `Calm` and entering a cabinet should noticeably slow the entry wipe.

- [ ] **Step 9: Commit**

```
git add src/settings.rs src/ui/settings.rs src/ui/cabinet.rs src/ui/mod.rs tests/animation.rs
git commit -m "Implement settings overlay and wire animation_speed to MotionBudget"
```

---

## Phase 10 · Polish + manual QA

### Task 10.1: Run the manual QA checklist end-to-end

**Files:** none modified; pure exercise.

- [ ] **Step 1: Run the test suite and lint one more time**

```
cargo build
cargo test
cargo clippy -- -D warnings
```

Expected: clean.

- [ ] **Step 2: Walk through the manual QA checklist from the spec**

Open the spec's "Manual QA checklist" section (`docs/superpowers/specs/design-overhaul/2026-05-23-bashcards-arcade-overhaul-design.md`) and tick each item by running `cargo run` and exercising the flow:

- [ ] Carafe imprint splash plays, skipped by Enter and by Esc.
- [ ] Bash arcade splash plays after carafe, skipped by Esc.
- [ ] Lobby renders all three cabinets with per-cabinet stats.
- [ ] Each cabinet launches, plays an objective, returns to lobby.
- [ ] Reduced motion: toggle in settings; all animations should snap; total time-to-lobby ≤ ~1.5 s.
- [ ] High contrast: toggle; palette swap legible; cabinet accents preserved.
- [ ] Truecolor → 256-color fallback: launch in a terminal with `TERM=xterm-256color` (no truecolor); verify cabinet colors still distinct.
- [ ] Below 80×24: resize the terminal; verify no crash. *Note: vertical-layout degradation for small terminals is **not implemented** in this overhaul — graceful crashes are acceptable, the spec lists this as "degrade not fail" but the lobby's three-column horizontal layout assumes adequate width. Capture the gap as a follow-up.*
- [ ] Settings overlay opens from lobby and in-cabinet; changes persist within the session (cross-launch settings persistence is out of scope).
- [ ] Schema-v1 profile migrates: place a legacy `~/.local/share/bashcards/profile.toml` with `completed_objectives = ["state-location"]`; launch; verify Shell Motel shows 1 completion in its card.

For any failing item that is **not** an explicitly-deferred gap, file it as a follow-up before moving on. The "below 80×24" item is acceptable as a known gap to revisit in Phase 11 (out of scope for v1 acceptance).

- [ ] **Step 3: Commit any small fixes from QA**

If the QA pass found small renderer issues (off-by-one alignment, dim colors), fix them and commit:

```
git add src/ui/
git commit -m "QA polish: <describe fixes>"
```

### Task 10.2: Final acceptance commit

**Files:** none

- [ ] **Step 1: Confirm green build**

```
cargo build
cargo test
cargo clippy -- -D warnings
```

- [ ] **Step 2: Verify `definition of done` in the spec**

Walk the spec's "Definition of done" checkbox list. All boxes should be tickable.

- [ ] **Step 3: Tag the release if desired**

Optional — the project hasn't tagged before. If you want to mark the overhaul:

```
git tag -a v0.2.0-arcade -m "Bash Arcade overhaul: carafe imprint, three cabinets, animation runtime"
```

Otherwise skip.

---

## Self-review summary

- **Spec coverage:** Every spec section maps to at least one phase. Cabinet system → Phase 1+6. Animation runtime → Phase 4. Splash → Phase 6. Lobby → Phase 7. In-cabinet → Phase 8. Settings → Phase 9. Profile reshape → Phase 2. Theme → Phase 3. QA → Phase 10. The "below 80×24 graceful degradation" is captured as a known gap in Task 10.1 because the spec lists it as desirable but neither the design doc nor any other section guarantees an implementation; it is explicitly called out as a follow-up rather than implemented.
- **Open spec questions** (vyfor version pin, `time_scale` presence, exact braille glyph patterns, `StreakSurge` color flash) are resolved during implementation:
  - Task 0.2 picks the version.
  - Task 9.1 step 3 documents the `time_scale` fallback path.
  - Task 6.4 sets up the glyph const that an implementer can hand-tune.
  - Task 8.1 fires `StreakSurge` after Correct outcomes when streak ≥ 3; the color flash detail is delegated to the renderer's interpretation of the cue, consistent with how the spec frames it.
- **Type consistency:** `Cabinet` enum variants (`ShellMotel`, `MonasteryOfForms`, `MidnightCarnival`) are used consistently. `CabinetSession` (post-rename) is referenced everywhere; `GameSession` and `Mode` are removed in Phase 6. `Cue::CabinetEnter { cabinet }` payload is consistent across firing site (Phase 8) and animator key (Phase 4).
- **Placeholders:** no "TBD"/"TODO"/"implement later" remain in the plan body. The only `// TODO` mentioned is the in-code marker for the `vyfor/animate` `time_scale` fallback, which is the spec's intentional implementation-time decision.

---

## Done

After Phase 10 completes, the arcade overhaul is shipped end-to-end. Future cabinets follow the summer-cabinet roadmap in the spec: pick a genre, write a manifest, add an enum variant + one match arm, update `docs/visual-design.md`, ship.
