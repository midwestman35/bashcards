# Bash Arcade · Design Overhaul

**Status:** approved design, ready for implementation plan
**Date:** 2026-05-23
**Companion documents:**

- `docs/visual-design.md` — durable visual philosophy and per-cabinet identity
- `docs/roadmap.md` — out-of-scope items for v1 with eventual landing targets
- `docs/superpowers/specs/2026-05-22-bashcards-design-and-plan.md` — original product spec; still load-bearing for gameplay design
- `docs/superpowers/specs/2026-05-22-bashcards-arcade-splash-addendum.md` — superseded by this spec, kept for history

---

## Purpose

Evolve `bashcards` from a single game with three modes into **Bash Arcade** — an umbrella terminal app whose first three cabinets are the existing modes (Shell Motel = Escape Room, Monastery of Forms = Dojo Drills, Midnight Carnival = Ops Sim). New summer cabinets drop in alongside them as new topics are studied. Existing core gameplay, validators, sandbox, and content stay intact; what changes is the chrome, the entry surface, the animation runtime, and how progress is namespaced.

Driving constraints:

- **Keep the core of each game.** Existing cabinet gameplay (validators, scoring, streak, objective progression) should not be touched in this overhaul.
- **Premium, animated feel.** Adopt the `vyfor/animate` crate as the foundation for both arcade chrome and in-cabinet prompt animations.
- **Vapor + liminal + Famicom punctuation.** Visual language locked in `docs/visual-design.md`.
- **Carafe imprint splash.** PS1-style boot screen identifies the publishing imprint before the arcade itself loads.
- **Cabinet velocity.** Adding a new summer cabinet should cost a few hours of Rust + TOML, plus content authoring time.

---

## Decisions locked during brainstorm

| Decision                                  | Choice                                                                     |
|---                                        |---                                                                         |
| Arcade scope                              | Modes → cabinets (3 cabinets at v1, more added over summer)                |
| Old splash addendum                       | Superseded with deprecation note (header added on the old file)            |
| Visual aesthetic                          | Vapor primary + liminal field + Famicom punctuation                        |
| Profile model                             | Per-cabinet progress, surfaced together in lobby                           |
| Animation scope                           | Chrome + in-game prompts; `effects.rs` rewritten on the new runtime        |
| Cabinet system architecture               | Static `Cabinet` enum + per-cabinet TOML manifest                          |
| Imprint splash                            | Two-stage boot: carafe imprint (3.5 s) → bash arcade boot console (~2.2 s) |
| Imprint skip                              | Both Enter and Esc skip                                                    |
| Default animation speed                   | `Normal`; settings already exposes `Calm` / `Normal` / `Snappy` (existing field, currently inert) |
| `time_scale` wiring                       | Implemented if `vyfor/animate` exposes a time-scale primitive; else commented-out with TODO marker |
| Audio in v1                               | None; moved to roadmap                                                     |
| Network features                          | Explicitly omitted (not on roadmap)                                        |

---

## Architecture

### Cabinet system

A `Cabinet` enum replaces today's `Mode` enum and is the single source of truth for "what cabinets exist." Each variant carries:

- A stable `id` (kebab-case, used in profile keys and content paths)
- A `Theme` (palette accents + cabinet glyph), resolved from the manifest
- A `ContentRef` pointing at a `content/cabinets/<id>.toml` file bundled via `include_str!`

```rust
// src/arcade/cabinet.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cabinet {
    ShellMotel,
    MonasteryOfForms,
    MidnightCarnival,
    // future summer cabinets get added as new variants
}

impl Cabinet {
    pub const fn all() -> &'static [Cabinet] {
        &[
            Cabinet::ShellMotel,
            Cabinet::MonasteryOfForms,
            Cabinet::MidnightCarnival,
        ]
    }

    pub const fn id(&self) -> &'static str { /* "shell-motel" | "monastery-of-forms" | "midnight-carnival" */ }
    pub const fn manifest_toml(&self) -> &'static str { /* include_str!("...") per variant */ }
    pub fn manifest(&self) -> &'static CabinetManifest { /* lazily parsed */ }
}
```

`App::launch_cabinet(Cabinet)` is the entry point for any cabinet, replacing `App::launch_mode`. Internally, `CabinetSession` (renamed from `GameSession`) holds the same scoring/streak/objective state as today, plus a reference to the cabinet's manifest for theme and content.

The existing `EscapeRoom` / `Dojo` / `OpsSim` thin extraction structs in `src/modes/` are repurposed as **objective extractors per cabinet genre**. The manifest's `genre` field selects which extractor a cabinet uses. This keeps per-genre logic intact while letting future cabinets compose from existing extractors.

**Difficulty stays orthogonal.** `Difficulty` (Beginner / Builder / Operator / Chaos) remains independent of cabinet. The existing rule that only certain (cabinet, difficulty) combos attach a real `Sandbox` is preserved — specifically, Midnight Carnival at Operator or Chaos still uses a real shell sandbox.

### Module layout

```
src/
├─ main.rs               # unchanged — calls ui::run_tui
├─ lib.rs                # adds: arcade, animation, theme
├─ app.rs                # launch_cabinet(Cabinet) instead of launch_mode(Mode)
├─ game.rs               # GameSession → CabinetSession (rename + per-cabinet profile wiring)
├─ content.rs            # adds CabinetManifest (theme + content in one TOML)
├─ validation.rs         # unchanged
├─ shell.rs              # unchanged (CommandWorld + Sandbox)
├─ profile.rs            # reshaped: cabinets: HashMap<String, CabinetProgress>
├─ settings.rs           # adds animation_speed; existing fields stay
├─ effects.rs            # rewritten on top of animation runtime
├─ input.rs              # unchanged
├─ telemetry_local.rs    # unchanged
├─ ui.rs                 # removed; becomes ui/ directory
├─ arcade/               # NEW
│   ├─ mod.rs            # state machine: Splash → Lobby → CabinetEnter → InCabinet → CabinetExit → Settings
│   ├─ cabinet.rs        # Cabinet enum + manifest accessors
│   └─ session.rs        # arcade ↔ cabinet routing
├─ animation/            # NEW
│   ├─ mod.rs            # Animator, Cue enum, sample API
│   ├─ timeline.rs       # named timelines, eased durations per Cue variant
│   └─ reduced_motion.rs # static-substitution helpers
├─ theme/                # NEW
│   ├─ mod.rs            # Palette, CabinetAccent, adapt::to_ansi
│   ├─ vapor.rs          # VaporPalette consts
│   └─ famicom.rs        # FamicomAccents consts
└─ ui/                   # NEW directory (formerly single ui.rs)
    ├─ mod.rs            # run_tui, frame loop, top-level state dispatch
    ├─ splash/
    │   ├─ mod.rs
    │   ├─ carafe.rs     # imprint stage rendering + glyph table
    │   └─ boot_console.rs # bash arcade boot console rendering
    ├─ lobby.rs          # cabinet grid + selected-session panel
    ├─ cabinet.rs        # in-cabinet play screen (replaces Playing screen)
    ├─ settings.rs       # settings modal overlay
    └─ widgets.rs        # shared vapor widgets (panel, glyph row, glow bar)
```

The existing `src/modes/` directory is kept, repurposed as per-genre objective extractors:

```
src/modes/
├─ mod.rs                # CabinetGenre enum, extractor dispatch
├─ escape_room.rs        # extractor for cabinets with genre = "escape_room"
├─ dojo.rs               # extractor for genre = "dojo"
└─ ops_sim.rs            # extractor for genre = "ops_sim"
```

### Cabinet manifest schema

One TOML per cabinet under `content/cabinets/<id>.toml`. The schema extends today's chapter format with a `[cabinet]` block carrying metadata and theme:

```toml
[cabinet]
id = "shell-motel"
display_name = "Shell Motel"
tagline = "a hallway asks where you are"
glyph = "▦"
genre = "escape_room"        # selects which modes/*.rs extractor handles this cabinet

[cabinet.theme]
accent_primary = "#ff9ad2"   # pink — used for cabinet frame borders + selected state
accent_secondary = "#8af0ff" # cyan — used for sub-frames and accents

[[chapters]]
title = "Lost Terminal"
# ... unchanged from existing lost_terminal.toml structure ...

[[chapters.rooms]]
# ... unchanged ...

[[chapters.rooms.objectives]]
# ... unchanged ...
```

For genres that use different content shapes (dojo uses `chapters.decks.cards`, ops_sim uses `chapters.incidents`), the manifest carries only the blocks relevant to that genre. The current `content/lost_terminal.toml` co-locates all three; the migration splits it into three separate manifests.

### Top-level state machine

Owned by `TuiApp` in `src/ui/mod.rs`. Six states; two are transient transition states owned by the animator.

```
[CarafeImprint] ─ complete or skip ─► [Splash] ─ complete or skip ─► [Lobby]
                                                                       │
                                                                  enter │ ◄─── exit ───┐
                                                                       ▼              │
                                                                  [CabinetEnter*] ─► [InCabinet]
                                                                                       │
                                                                                   pause│
                                                                                       ▼
                                                                                  [Settings] ─ esc ─► back to caller
                                                                                       │
                                                                                       ▼
                                                                                  [CabinetExit*] ─► back to Lobby

  * CabinetEnter and CabinetExit are transient states owned by the animator.
    They pin input handling during the wipe and yield when the cue completes.
```

---

## Animation runtime

### Cue catalog

Every animation in the app is one of these, defined in `src/animation/mod.rs::Cue`. Cues are the only animation API the rest of the app sees.

| Cue                              | Trigger                                | Feel                                  |
|---                               |---                                     |---                                    |
| `ImprintMaterialize { progress }` | carafe splash dots → glyph             | sparse dots resolve, eased-out, 1.0s  |
| `BootLineReveal { line_idx }`    | arcade splash, per boot-log line       | type-on, ~120ms per line              |
| `TitleSweep`                     | arcade splash, after boot lines done   | pastel sweep across title, ~1.2 s     |
| `StatusFlicker { token }`        | arcade splash, on `OK`/`READY`/`NO GUILT` | one flicker before settling          |
| `CabinetEnter { cabinet }`       | lobby → cabinet                        | scanline wipe + glow, ~400 ms         |
| `CabinetExit`                    | cabinet → lobby                        | reverse wipe, ~300 ms                 |
| `LobbyHover { cabinet }`         | idle on selected cabinet               | slow pulse, 6 s cycle                 |
| `ObjectiveGradient`              | in-game, new objective revealed        | gradient sweep on objective text      |
| `PromptReveal`                   | in-game, new prompt shown              | typed-on reveal, ~80 ms/char          |
| `HintFlash`                      | in-game, hint summoned                 | violet flash on hint line             |
| `CorrectGlow`                    | in-game, correct answer                | mint pulse on prompt bar              |
| `IncorrectFlicker`               | in-game, incorrect answer              | one Famicom-magenta flicker           |
| `StreakSurge { streak }`         | in-game, streak ≥ 3                    | stronger pulse, cabinet-accent color  |

### Animator API

```rust
// src/animation/mod.rs
pub struct Animator {
    inner: animate::Engine,
    motion: MotionBudget,
}

pub struct MotionBudget {
    pub reduced: bool,
    pub speed: f32,             // 0.5 / 1.0 / 1.5
}

impl Animator {
    pub fn fire(&mut self, cue: Cue);
    pub fn fire_set(&mut self, cues: &[Cue]);
    pub fn tick(&mut self, dt: Duration);
    pub fn sample(&self, cue: &Cue) -> Option<Sample>;
    pub fn is_complete(&self, cue: &Cue) -> bool;
    #[cfg(test)] pub fn tick_to(&mut self, t: Duration);  // deterministic test helper
}
```

`Sample` carries values the renderer needs (rgb, alpha, position offset, character substitution for reveal-by-character, glyph density for braille materialize). The TUI render path queries the animator and translates the sample into ratatui `Style` and `Span` overrides. `vyfor/animate` never touches ratatui directly — the wrapper isolates it.

### `vyfor/animate` integration

Cargo dependency:

```toml
animate = { version = "x.y", features = ["ratatui"] }
```

The `ratatui` feature provides interpolators for `ratatui::style::Color`, enabling smooth color tweens on truecolor terminals. Version pinned during implementation. No other dependency changes.

### Motion budget

Three settings interact:

- **`reduced_motion: true`** → animator returns final-state samples immediately; no tweens, no flickers, no idle breath. Functionally equivalent to no animation.
- **`animation_speed`** (already in `Settings` today as `AnimationSpeed::Calm | Normal | Snappy`, currently unused by the renderer) → maps internally to cue-duration multipliers. Default `Normal` (1.0×); `Calm` is ~0.6×, `Snappy` is ~1.4×. The enum names are kept; the multipliers are an internal animator detail.
- **`high_contrast: true`** → cues that depend on color tween (e.g., `CorrectGlow` going mint → cyan) snap to one solid token instead of tweening.

**`time_scale` wiring caveat.** The `animation_speed` field already exists in `Settings` but is inert today — no code reads it. v1 wires it to the animator. The wiring routes through `vyfor/animate`'s time-scale primitive if one is exposed; otherwise the wiring is left as `// TODO: enable when animate exposes time_scale` and the setting persists but has no runtime effect until that primitive lands. The settings panel surfaces the caveat (e.g., a dim note next to the field) until the wiring activates.

### Replacing `effects.rs`

The current `EffectCue` enum and `gradient_cells()` get rewritten on top of `Cue` and `Animator`. One-to-one mapping for existing cues:

| Today (`effects.rs`)     | New (`animation::Cue`)                       |
|---                       |---                                           |
| `ObjectiveGradient`      | `ObjectiveGradient`                          |
| `Reveal`                 | `PromptReveal`                               |
| `Flicker`                | `IncorrectFlicker`                           |
| `Sweep`                  | `TitleSweep` (splash) / `CorrectGlow` (in-game) |
| `SuccessGradient`        | `CorrectGlow`                                |
| `cues_for_success()`     | `Animator::fire_set(&[...])`                 |

The reduced-motion branch in `cues_for_success` lifts to `MotionBudget`, which the `Animator` consults globally — every call site stops caring about reduced motion.

### Frame loop

Today `ui.rs` blocks on `event::read()` with no animation tick. The new loop in `ui/mod.rs`:

```rust
loop {
    terminal.draw(|f| renderer.render(f, &state, &animator))?;
    if event::poll(frame_budget)? {
        match event::read()? { /* handle input */ }
    }
    animator.tick(elapsed);
    if state.should_quit { break; }
}
```

`frame_budget` defaults to ~16 ms (≈60 fps) and drops to `Duration::ZERO` (no animation tick, only event-driven repaints) when `reduced_motion = true`, so reduced-motion mode costs essentially nothing.

---

## Visual direction

The full visual philosophy lives in `docs/visual-design.md` and is the source of truth. Summary for this spec:

- **Vapor primary** (resting palette, chrome): `#ff9ad2` pink, `#8af0ff` cyan, `#c8a8ff` violet, `#c9ffb3` mint, `#ffe8a3` yellow, `#9f87c8` dim prose.
- **Liminal field** (background gradient): `#1b0a36 → #2a1052 → #1f0a44` vertical gradient.
- **Famicom punctuation** (selected state, flicker, boot tokens): `#ff3e9d` magenta, `#00d4ff` deep cyan, `#2e3aa0` navy inset.
- **Ratio rule**: at least 80% vapor / at most 20% Famicom per frame.

Cabinet identity (glyph + accent + voice):

- **Shell Motel** — `▦` · `#ff9ad2` pink · *a hallway asks where you are*
- **Monastery of Forms** — `⛩` · `#8af0ff` cyan · *repeat until the scroll smiles*
- **Midnight Carnival** — `◉` · `#c9ffb3` mint · *find the lost logs*

---

## Splash, lobby, and cabinet flow

### Stage 1 · Carafe imprint splash

Plays once per process invocation. Skippable by Enter or Esc at any frame.

**Field.** Radial gradient on the deep-field tokens (`#160830` center → `#060212` edge). One of two cases where the field is radial; the other is high-contrast mode (solid `#000`).

**Logomark.** "carafe" rendered in low-rez braille glyphs (Unicode U+2800–U+28FF). Hand-designed as a 6-character × 2-row const glyph table in `src/ui/splash/carafe.rs`:

```rust
// 6 letters × 2 braille-row "lines" × ~4 braille cells per letter, hand-tuned
pub const CARAFE_GLYPHS: [[&str; 6]; 2] = [
    ["⢀⣀⡀", "⣀⡀", "⡀⢀", "⡀⢀", "⣀⡀", "⢀⣀⡀"],   // top row
    ["⠛⠁",  "⣸⠟", "⠉⠉", "⢾⠆", "⣸⠟", "⠉⠉⠆"],   // bottom row
];
```

Final glyph patterns are tuned during implementation.

**Reveal.** `Cue::ImprintMaterialize { progress }` drives sparse-dots → resolved-glyph over 1.0 s with eased-out timing. The renderer samples `progress` and chooses how many of each letter's braille dots to render.

**Timing budget.**

| Stage                         | Duration |
|---                            |---       |
| Field idle (dot motes drift)  | 0.8 s    |
| Materialize                   | 1.0 s    |
| Hold                          | 1.5 s    |
| Fade                          | 0.2 s    |
| **Total**                     | **3.5 s** |

Reduced motion: holds resolved logomark for 1.0 s and advances.

### Stage 2 · Bash arcade boot console

Plays after carafe. Skippable by Esc.

| Step | What renders                                              | Cue                  | Duration   |
|---   |---                                                        |---                   |---         |
| 1    | Title block (`BASH ARCADE` ASCII letters) appears static  | none                 | instant    |
| 2    | Title sweep (pastel sweep across letters)                 | `TitleSweep`         | ~1.2 s     |
| 3    | Boot lines reveal one at a time                           | `BootLineReveal{n}`  | ~120 ms ea |
| 4    | Status tokens (`OK` / `NO GUILT` / `READY`) flicker once  | `StatusFlicker`      | ~250 ms    |
| 5    | "press enter to insert coin" prompt with blinking cursor  | static + blink       | until input |

Total minimum-to-prompt: ~2.2 s in normal motion; immediate in reduced motion.

Boot lines (defined in `src/ui/splash/boot_console.rs::BOOT_LINES`) name the actual cabinets being mounted. Generated from `Cabinet::all()`, so new summer cabinets are advertised automatically:

```
initializing vapor console
mounting cabinet shell-motel
mounting cabinet monastery-of-forms
mounting cabinet midnight-carnival
loading profile
checking reset protocol
```

### Lobby

Three cabinet cards arranged horizontally with per-cabinet profile stats (streak, score, last objective) surfaced under each. Below the cabinet row, a "selected session" detail panel shows the focused cabinet's tagline, next objective, and completion progress, pulled from the cabinet's `CabinetProgress` and manifest.

**Default focus.** Whichever cabinet has the fewest completed objectives. On a fresh profile, that is Shell Motel (the most beginner-friendly). This nudges the player toward growth without locking them out of replay.

**Key bindings (lobby).**

| Key              | Action                                                              |
|---               |---                                                                  |
| `h` / `←`        | focus previous cabinet (wraps)                                      |
| `l` / `→`        | focus next cabinet (wraps)                                          |
| `j` / `k`        | scroll the "selected session" detail panel (when content overflows) |
| `1` / `2` / `3`  | jump-focus by cabinet index                                         |
| `Enter`          | launch focused cabinet                                              |
| `s`              | open settings                                                       |
| `?`              | open help / explanation overlay                                     |
| `q` / `Ctrl+C`   | quit                                                                |

### Cabinet entry / exit

- **`CabinetEnter`** — ~400 ms scanline wipe + glow in focused cabinet's accent color. Input locked during the wipe. At completion, renderer swaps to `InCabinet` and fires `ObjectiveGradient` on the first objective.
- **`CabinetExit`** — ~300 ms reverse wipe back to lobby. Triggered by `Esc` from inside a cabinet, with a "quit cabinet? your progress is saved" confirmation only on Operator/Chaos difficulty.

### In-cabinet view

Replaces today's `Playing` screen. Game logic in `CabinetSession` is unchanged. Layout:

- **Top bar:** cabinet glyph + display_name + objective progress (e.g., `form 1 / 5`) + streak counter.
- **Body:** objective text with `ObjectiveGradient` on entry.
- **Hint slot:** collapsed until `?`; violet, with `HintFlash` on summon.
- **Input box:** cabinet-accent-bordered with `PromptReveal` for typed-on system prompts.
- **Footer:** key bindings + back-to-arcade hint.

The renderer subscribes to `AttemptOutcome` and fires the corresponding cue:

- `Correct` → `CorrectGlow` and `ObjectiveGradient` on the next objective
- `Incorrect` → `IncorrectFlicker`
- `Blocked` → no animation (the message stands alone)
- Streak crosses 3 → `StreakSurge`

### Settings overlay

Triggered by `s` from lobby or in-cabinet. Modal panel with cabinet-aware accent (uses current cabinet's color in-game, vapor pink in lobby). Surfaces:

- Difficulty
- Pressure Profile
- Reduced Motion (toggle)
- **Animation Speed (Calm / Normal / Snappy)** — already in `Settings`; v1 wires it to the animator (see `time_scale` caveat above)
- High Contrast (toggle)
- Hint Style
- Explain (on / off)
- **Theme Override per cabinet** — new in v1, optional; lets the player swap a cabinet's accent

Persisted to the existing profile TOML.

### Edge cases

- **First-ever launch.** No profile yet. Splash plays, lobby focuses Shell Motel by default, selected-session panel shows the cabinet's intro tagline and "no progress yet."
- **No content for a cabinet.** Shouldn't happen post-content-split; if it does, the cabinet card renders with a soft `[no content yet]` placeholder and is non-selectable.
- **Terminal too small.** Below 80×24, the splash renders a compact 2-line title and skips the cabinet grid. The lobby renders cabinets vertically (list view) instead of horizontally. We degrade; we don't fail.
- **Truecolor unavailable.** `theme::adapt::to_ansi(rgb)` snaps each token to the nearest 256-color match. The arcade still works; gradients step rather than tween.
- **256-color unavailable.** Fallback to a 16-color "minimal vapor" mapping that preserves cabinet-color distinction.

---

## Data model changes

### Profile

Today's `Profile` is intentionally minimal:

```rust
// current
pub struct Profile {
    pub completed_objectives: Vec<String>,
    pub weak_topics: Vec<String>,
}
```

It does not currently persist score, streak, or attempts — those live in the in-memory `GameSession` and are lost when the session ends. The overhaul reshapes it into a per-cabinet structure that also persists per-session stats so the lobby can render them:

```rust
// after overhaul
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profile {
    pub schema_version: u32,                            // = 2 after the overhaul; absent means v1
    pub cabinets: HashMap<String, CabinetProgress>,     // cabinet id → progress
    pub weak_topics: Vec<String>,                       // kept top-level for now; per-cabinet later
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CabinetProgress {
    pub score: u32,
    pub streak: u32,
    pub attempts: u32,
    pub completed_objectives: HashSet<String>,
    pub last_objective_id: Option<String>,
    pub last_played: Option<DateTime<Utc>>,
}
```

**Migration.** Profile files without a `schema_version` field are treated as v1. On first load post-overhaul:

1. The v1 `completed_objectives: Vec<String>` becomes Shell Motel's `CabinetProgress.completed_objectives` (`HashSet<String>` after dedup). The reasoning: today's content was Escape Room only, so existing completions map to that cabinet.
2. `weak_topics` is preserved verbatim at the top level.
3. `schema_version` is set to `2` and the new file is written back.

Because this is pre-1.0 and the user base is the project author, a "fresh-start" path is acceptable if the migration proves brittle in practice. The implementation may choose either approach; the spec does not require a forced migration.

Profile location stays as it is today (platform data dir via the `directories` crate).

### Settings

`src/settings.rs` already has the fields needed for v1 motion control. The overhaul **wires existing fields** to the animator and adds one new field for cabinet customization:

- `animation_speed: AnimationSpeed` — **exists** as `Calm | Normal | Snappy` but is currently inert; the overhaul connects it to the animator's motion budget.
- `theme_overrides: HashMap<String, ThemeOverride>` — **new**, keyed by cabinet id; optional, lets the player swap a cabinet's `accent_primary` to any vapor token (not arbitrary RGB — restricted to the palette in `theme/vapor.rs` to prevent drift from the visual design philosophy).

Existing fields (`pressure`, `reduced_motion`, `high_contrast`, `hint_style`, `explain_after_success`, `session_length_target`) stay. The settings overlay UI in `src/ui/settings.rs` surfaces all of them, including the previously-inert `animation_speed`.

Settings is currently held by `TuiApp` and not persisted across launches; this spec does not require Settings persistence. If desired, persistence can be added later (likely as a top-level field on `Profile`).

### Content schema

The current single `content/lost_terminal.toml` splits into three cabinet manifests:

- `content/cabinets/shell-motel.toml` — takes the `[[chapters.rooms]]` block
- `content/cabinets/monastery-of-forms.toml` — takes the `[[chapters.decks.cards]]` block
- `content/cabinets/midnight-carnival.toml` — takes the `[[chapters.incidents]]` block

Each gets a `[cabinet]` header per the schema above. Manifests bundled via `include_str!` in `src/arcade/cabinet.rs::manifest_toml()`. The existing `lost_terminal.toml` file is deleted after the split is verified.

`ContentLibrary::validate()` extends to require at least one `[cabinet]` block in each manifest and to validate the manifest's `genre` matches one of the known genres (`escape_room`, `dojo`, `ops_sim`).

---

## Testing strategy

Tests in `tests/` continue to operate on the library API — they bypass the TUI, the splash, the animator, and the lobby.

| Layer                          | Test approach                                                                                            |
|---                             |---                                                                                                       |
| `validation.rs`                | Unchanged. Pure functions; existing unit tests apply.                                                    |
| `shell.rs`                     | Unchanged. `CommandWorld` and `Sandbox` behavior unaffected.                                             |
| `game.rs` (CabinetSession)     | Existing `GameSession` tests follow the rename. Same scoring/streak/objective coverage.                  |
| `profile.rs`                   | New tests for per-cabinet `CabinetProgress`: empty profile, single cabinet, multiple cabinets, schema-v1-to-v2 migration. |
| `content.rs` / `arcade::cabinet` | New tests asserting `Cabinet::all()` returns the expected three variants and each manifest validates. |
| `animation/`                   | Deterministic `Animator::tick_to(t)` lets unit tests assert cue progression. Cover: cue lifecycle, reduced-motion snap, motion-budget scaling, `ImprintMaterialize` progress sampling. |
| `theme/`                       | Unit tests for palette lookups, high-contrast swap, truecolor → 256-color adapter.                       |
| `ui/`                          | Excluded from automated tests (existing convention — TUI requires interactive terminal).                 |

### Manual QA checklist

Run before declaring v1 done:

- [ ] Carafe imprint splash plays, skipped by Enter and by Esc.
- [ ] Bash arcade splash plays after carafe, skipped by Esc.
- [ ] Lobby renders all three cabinets with per-cabinet stats.
- [ ] Each cabinet launches, plays an objective, returns to lobby.
- [ ] Reduced motion: all animations snap; total time-to-lobby ≤ ~1.5 s.
- [ ] High contrast: palette swap is legible, accents preserved.
- [ ] Truecolor → 256-color fallback verified on a non-truecolor terminal.
- [ ] Below 80×24: vertical lobby layout activates, no crash.
- [ ] Settings overlay: opens from lobby and in-cabinet, persists changes.
- [ ] Schema-v1 profile (if any exist) migrates cleanly on first launch.

---

## Out of scope for v1

Five items intentionally deferred to a later release; landing targets documented in `docs/roadmap.md`:

- **Audio** (carafe chime, cabinet transition tones, ambient beds) — v1.5.
- **"Resume where you left off"** behavior on launch — v1.5.
- **Cabinet unlock progression / locked cabinets** — v2.
- **Theme overrides for the carafe imprint** — v2.
- **Cabinet authoring tooling** (in-app or separate CLI editor) — v3, re-evaluated after the summer.

Two items explicitly **omitted** (not on roadmap):

- **Network leaderboards / multiplayer / shared profiles.** Bash Arcade is single-player local.
- **Weekly study schedule surface.** Superseded by the topic-cabinet model.

---

## Summer-cabinet roadmap

The structural payoff of the overhaul: adding a new cabinet during the summer should be a few hours of work, dominated by content authoring.

Steps:

1. **Pick a genre** that matches an existing extractor in `src/modes/` (`escape_room` / `dojo` / `ops_sim`), or write a new extractor file if the gameplay shape is genuinely novel.
2. **Write the cabinet manifest** at `content/cabinets/<id>.toml` — metadata (display name, glyph, tagline, accent colors) + content (chapters with rooms/decks/incidents matching the genre).
3. **Add one variant** to the `Cabinet` enum and one match arm to `id()`, `manifest_toml()`. ~15 lines of Rust.
4. **Update `docs/visual-design.md` Section 4** with the new cabinet's voice, glyph, accent, tagline.
5. **No splash code changes needed** — the bash arcade boot console reads `Cabinet::all()` and auto-includes the new cabinet's "mounting cabinet `<id>`" boot line.

---

## Definition of done

The overhaul is complete when:

- [ ] Carafe imprint splash plays at launch, skippable by Enter and Esc.
- [ ] Bash arcade boot console plays after carafe, skippable by Esc.
- [ ] Lobby surfaces all three cabinets with per-cabinet profile stats.
- [ ] Three cabinets (Shell Motel, Monastery of Forms, Midnight Carnival) are playable with content split from the original `lost_terminal.toml`.
- [ ] `vyfor/animate` (with `ratatui` feature) drives all chrome animations and in-game cues.
- [ ] `effects.rs` is rewritten on top of the animation runtime; no orphan `EffectKind` / `EffectCue` references remain.
- [ ] Per-cabinet profile shape (`schema_version = 2`) is in place; migration path from v1 chosen and implemented (or fresh-start documented).
- [ ] Reduced motion, high contrast, truecolor fallback all functional.
- [ ] Old splash addendum bears the supersession header.
- [ ] `docs/visual-design.md` reflects all final decisions including the carafe imprint section.
- [ ] `docs/roadmap.md` lists all five deferred items with eventual-version targets.
- [ ] Manual QA checklist passes.
- [ ] `cargo test`, `cargo clippy`, `cargo build` all pass.
- [ ] Single-cabinet binary still launches and plays (no regressions in core gameplay).

---

## Open questions

None blocking. The following implementation-time decisions are flagged but do not require pre-implementation resolution:

- Exact `vyfor/animate` version pin and whether `time_scale` is supported.
- Schema-v1 → schema-v2 profile migration vs. fresh-start (pre-1.0 user base is small).
- Final hand-tuned braille glyph patterns for "carafe" letterforms.
- Whether `Cue::StreakSurge` should also use the Famicom magenta as a one-frame flash before the cabinet-accent pulse.

These get resolved during implementation; the design holds either way.
