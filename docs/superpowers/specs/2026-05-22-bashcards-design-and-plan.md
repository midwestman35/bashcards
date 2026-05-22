# bashcards Design and Implementation Plan

## Purpose

`bashcards` is a Rust terminal game for learning bash by doing. It teaches real terminal fluency through playful, repeatable practice instead of passive lectures. The app should feel like a strange little training console: practical, visually alive, funny without being noisy, and structured enough to build discipline.

The game has three first-class modes:

- **Escape Room**: puzzle rooms where commands unlock clues, doors, panels, machines, and weird terminal artifacts.
- **Dojo Drills**: fast replayable challenge cards for repetition, recall, streaks, and muscle memory.
- **Ops Sim**: practical incidents where the player investigates and fixes realistic-but-silly terminal problems.

The first release should not be a toy prototype. It should implement the full product skeleton, all three modes, shared challenge infrastructure, beginner-first content, settings, scoring, progress tracking, and the visual effect system. Content depth can grow over time, but the shipped architecture should be the real architecture.

## Product Principles

- **Doing beats watching**: every lesson turns quickly into a command the player must type.
- **Beginner-first, not babyish**: start from `pwd`, `ls`, `cd`, `cat`, paths, files, and safe destructive-command habits, but keep the tone clever and game-like.
- **ADHD-friendly by default**: short loops, visible progress, opt-in pressure, quick wins, low shame, and clear next actions.
- **Playful narratives, not factory chores**: challenges should feel like odd missions, rooms, rituals, mysteries, or incidents, not bland workplace tickets.
- **Real fluency matters**: simulated validation is allowed early, but advanced play should graduate into disposable real shell sandboxes.
- **Effects reinforce state**: animation and color should guide attention, reveal story, and reward completion, never obscure command input or output.

## Player Model

The default player is a true beginner who wants terminal confidence from the ground up. They may have seen commands before but do not yet have automatic recall or a clear mental model of where they are in the filesystem.

The app should also support growth through explicit difficulty selection:

- **Beginner**: teaches commands directly, provides generous hints, validates expected commands, and avoids fail states.
- **Builder**: combines concepts, reduces hints, introduces alternatives, and asks for small command chains.
- **Operator**: uses real temporary sandboxes and validates final filesystem state.
- **Chaos**: adds timers, distractors, broken states, recovery tasks, and stricter scoring.

Difficulty is separate from game mode. A player can play Beginner Escape Room, Builder Dojo, Operator Ops Sim, and so on.

## Game Modes

### Escape Room

Escape Room is the narrative puzzle mode. Each chapter is a sequence of rooms, each room containing terminal objects, clues, locked exits, and command-driven interactions.

Example narrative flavor:

- The player wakes up inside an abandoned terminal called the **Shell Motel**, where each room is a directory.
- A melodramatic help daemon named **Manny Page** keeps appearing with suspiciously useful hints.
- Locked doors are files, broken lights are permissions, and secret panels are hidden dotfiles.
- Rooms can be silly: a directory full of fake cursed receipts, a vending machine that only accepts correct glob patterns, a door that refuses to open until the player reads its `README`.

Beginner chapters teach:

- Orientation: `pwd`, `ls`, current directory, home directory.
- Movement: `cd`, `..`, `.`, absolute and relative paths.
- Inspection: `cat`, `less` later, file names with spaces, extensions.
- Creation and movement: `touch`, `mkdir`, `cp`, `mv`.
- Safe deletion: `rm` warnings, trash/sandbox framing, why destructive commands matter.
- Pattern basics: `*`, simple globs, hidden files.

Escape Room should prioritize curiosity. The player should wonder, "What happens if I inspect this?" The command is the tool that answers.

### Dojo Drills

Dojo Drills is the repetition mode. It uses short cards organized by topic, with streaks, speed feedback, and immediate correction. It should feel like a slightly ridiculous command-line martial arts academy, not a corporate training quiz.

Example narrative flavor:

- The player trains at the **Monastery of Questionable Aliases**.
- Instructors have names like **Sensei Subshell**, **Grandmaster Glob**, and **The Pipe Whisperer**.
- Challenge cards are "forms": Path Form, File Form, Copy Form, Rename Form, Inspection Form.
- Wrong answers are treated as training moments: "The scroll remains unimpressed, but it is patient."

Drill types:

- **Recall**: given an objective, type the command.
- **Complete the Command**: fill the missing piece.
- **Spot the Safer Command**: choose between destructive and safer alternatives.
- **Fix the Command**: repair a broken command.
- **Speed Round**: complete several tiny objectives under optional pressure.
- **Combo Card**: combine two or three commands in sequence.

Dojo should be highly replayable. It is the main home for spaced practice, streaks, and short ADHD-friendly sessions.

### Ops Sim

Ops Sim is the applied practice mode. It presents practical incidents in odd, memorable wrappers. The goal is not to mimic a factory line; it is to make real workflows feel like miniature stories.

Example narrative flavor:

- **The Midnight Server Carnival**: logs are hiding in concession stands.
- **The Haunted Backup Closet**: old reports must be found, verified, compressed, and moved before the closet gets dramatic.
- **The Moonbase Helpdesk**: astronauts keep saving files in the wrong directories.
- **The Museum of Misnamed Artifacts**: files have bad names, weird extensions, and duplicate copies.

Ops Sim teaches:

- Finding things: `find`, `grep`, `ls -la`, `du`.
- Inspecting state: file sizes, timestamps, permissions.
- Pipelines: `|`, `sort`, `head`, `wc`, command composition.
- Environment basics: `$PATH`, variables, `export`.
- Scripts: shebangs, executable bits, arguments.
- Recovery: reading errors, undoing safe mistakes inside a sandbox.

Operator and Chaos difficulties should run Ops Sim tasks in real temporary sandboxes where the app validates final state.

## Curriculum and Challenge Decks

`bashcards` uses both a progression path and replayable decks.

### Curriculum Path

The curriculum is chapter-based. Chapters unlock in a rough beginner-to-advanced order:

1. **Lost Terminal**: orientation, `pwd`, `ls`, `cd`, `cat`, paths.
2. **File Forge**: `touch`, `mkdir`, `cp`, `mv`, safe `rm`.
3. **Glob Garden**: `*`, `?`, hidden files, file matching.
4. **Pipeworks**: `grep`, `sort`, `uniq`, `wc`, `head`, `tail`, pipes.
5. **Permission Parlor**: `chmod`, executable files, ownership concepts.
6. **Script Crypt**: shell scripts, arguments, exit codes.
7. **Env Observatory**: environment variables, `$PATH`, aliases.

Each chapter should include at least:

- One Escape Room sequence.
- One Dojo deck unlocked from the commands taught.
- One Ops Sim that applies the chapter's skills.

### Challenge Decks

Decks are replayable sets of cards grouped by command, concept, mode, and difficulty. Decks should support short sessions:

- 3-card micro session.
- 10-card focused drill.
- Review due cards.
- Shuffle by weak topic.
- Retry missed cards.

Decks should be stored as data, not hard-coded, so new cards can be added without rewriting app logic.

## Command Input Model

The app uses hybrid command input.

### Guided TUI Prompt

Beginner and Builder challenges use an in-game command panel. It looks like a shell prompt, but commands are validated through the app's challenge engine.

Benefits:

- Deterministic output for teaching.
- Helpful error explanations.
- Safer handling of destructive commands.
- Better integration with hints, scoring, narrative reveals, and animations.

The guided prompt should support:

- Command submission.
- Command history.
- Basic autocomplete preview where appropriate.
- Error feedback.
- Hint requests.
- Expected-output rendering.

### Real Sandbox Shell

Operator and Chaos challenges can launch or embed a real shell inside a temporary sandbox. The app then validates the final filesystem state when the player returns.

Sandbox rules:

- Use a temporary directory per challenge attempt.
- Populate fixtures from challenge definitions.
- Set the working directory to the challenge root.
- Never run tasks against user directories.
- Validate expected final files, contents, permissions, and command effects.
- Treat dangerous commands as contained by the sandbox.
- Record enough metadata for scoring, but do not log sensitive user shell history outside the game.

## Settings

Settings are first-class and should be available from the launcher and in-game pause/settings screen.

### Pressure Profile

Default: **Soft Urgency**.

- **Cozy No-Timer**: no countdowns, gentle hints, no score decay, no fail state.
- **Soft Urgency**: ambient progress, optional streaks, no beginner fail state, replayable clean-run goals.
- **Arcade Pressure**: timers, score decay, combo streaks, faster animations, stronger replay incentives.

Pressure changes pacing and scoring, not the command curriculum.

### Motion and Accessibility

- Reduced motion toggle.
- High contrast toggle.
- Animation speed: calm, normal, snappy.
- Sound should not be required. Initial version can be silent.
- Effects must never obscure command input or command output.

### Learning Preferences

- Hint style: direct, nudging, minimal.
- Explain after success: always, first time only, off.
- Session length target: 3, 10, or 20 cards.

## Visual and Animation System

The core UI is built with Rust, `ratatui`, and `crossterm`.

Candidate visual libraries:

- `ratatui`: primary full-screen TUI rendering.
- `crossterm`: terminal backend and input events.
- `tachyonfx`: Ratatui-compatible effects and animations.
- `cli_animate`: optional support for progress bars, loading indicators, and menu flourishes outside the main Ratatui room renderer.

The design should assume terminal-native visuals, not browser or GPU shaders. Standard terminals render character cells, so true blur and glow are hard limitations in the normal CLI path. The game should create atmosphere through color, timing, Unicode blocks, layout, and animation.

### Effects Vocabulary

- **Truecolor gradients for objectives**: active objectives use readable gradient text to pull attention.
- **Flicker, sweep, and reveal animations**: used for narrative text, timers, room reveals, system boot sequences, and clue discovery.
- **Truecolor success gradients**: completed objectives receive a rewarding gradient pass.
- **Breathing lighting**: room panels and ASCII art can pulse subtly through color brightness changes.
- **Progress fills**: system power, door unlocks, and drill streaks use animated bars.

Guardrails:

- Never animate command input while the player is typing.
- Never obscure or distort command output.
- Reduced motion must replace animated reveals with instant or minimal transitions.
- Beginner mode should use fewer simultaneous effects.
- Effects must indicate state changes: objective active, clue revealed, success, warning, or failure.

### TUI Layout

Main in-game layout:

- Header: mode, chapter, difficulty, pressure profile, streak/progress.
- Room or challenge view: ASCII/Unicode scene, interactive objects, state indicators.
- Lesson card: command concept, short explanation, current hint, optional example.
- Command panel: prompt, output, error feedback, command history.
- Footer: keybindings and settings hint.

Launcher layout:

- Mode selector: Escape Room, Dojo Drills, Ops Sim.
- Difficulty selector: Beginner, Builder, Operator, Chaos.
- Curriculum path and decks.
- Settings.
- Profile/progress.

The interface should feel visually interesting but still legible under repeated use.

## Data Model

Challenge content should be data-driven. The first build should use TOML. Rust structs deserialize challenge definitions into game content.

Core entities:

- `Profile`: player progress, settings, streaks, weak topics.
- `Mode`: Escape Room, Dojo, Ops Sim.
- `Difficulty`: Beginner, Builder, Operator, Chaos.
- `PressureProfile`: Cozy, Soft Urgency, Arcade.
- `Chapter`: curriculum unit with rooms, decks, and incidents.
- `ChallengeCard`: atomic drill or objective.
- `Room`: escape-room scene with objects, objectives, and transitions.
- `Objective`: task with prompt, hints, validation, success text.
- `Fixture`: files/directories used for simulated or real sandbox tasks.
- `Validator`: command-pattern validation or state validation.
- `EffectCue`: animation intent triggered by game events.

Validation types:

- `ExactCommand`: expected command string for early beginner cards.
- `CommandPattern`: allowed command variants.
- `OutputMatch`: expected simulated output.
- `FilesystemState`: expected files, directories, contents, or permissions.
- `Sequence`: ordered objective chain.
- `AnyOf`: multiple valid solutions.

## Architecture

Recommended Rust module structure:

- `app`: top-level application state, event loop, screen routing.
- `ui`: Ratatui widgets, layout, style, rendering helpers.
- `effects`: animation clock, effect cues, gradient styling, reduced-motion behavior.
- `input`: key handling, command-line editing, history, mode-specific input.
- `game`: shared game state, scoring, progress, attempts, mode contracts.
- `modes::escape_room`: rooms, objects, navigation, puzzle state.
- `modes::dojo`: decks, drills, streaks, spaced review hooks.
- `modes::ops_sim`: incident flows, sandbox lifecycle, applied validation.
- `content`: loading, parsing, and validating TOML/YAML challenge files.
- `shell`: simulated command runner and real sandbox runner.
- `validation`: validators for commands, outputs, and filesystem state.
- `profile`: persistence for settings, progress, and stats.
- `settings`: pressure, difficulty, accessibility, learning preferences.
- `telemetry_local`: local-only stats for progress and weak-topic detection.

The mode modules should use shared contracts instead of each inventing its own command handling. A mode owns narrative flow; the shared engine owns attempts, validation, scoring, hints, and effects.

## Error Handling and Safety

Command mistakes are part of learning. The game should classify errors and respond helpfully:

- Unknown command.
- Command known but wrong for current objective.
- Right command, wrong path.
- Destructive command blocked in guided mode.
- Sandbox command failed.
- Validation failed after real sandbox task.

Feedback should be specific and kind:

- "You are close. `ls` shows what is here; `pwd` shows where here is."
- "That path points to a file, but `cd` needs a directory."
- "This would delete something. In Beginner mode, deletion only works when the objective explicitly asks for it."

Safety rules:

- No guided-mode destructive command should affect the user's filesystem.
- Real shell tasks run only in disposable temp directories.
- The app should show the sandbox path clearly when entering real shell mode.
- The app should clean up sandboxes by default, with a debug option to preserve them.

## Progress, Scoring, and Feedback

Beginner progress should reward completion, attempts, and return visits, not just speed.

Scoring inputs:

- Objective completion.
- Hint usage.
- Attempts.
- Time only when pressure profile enables it.
- Streaks in Dojo.
- Clean final state in Ops Sim.

Progress outputs:

- Chapter completion.
- Command confidence by topic.
- Missed-card review deck.
- Recent wins.
- Suggested next session.

Completion text should use truecolor success gradients and short celebratory copy:

- "Objective complete: you found your current location."
- "Door logic accepts your offering of `pwd`."
- "The terminal hums approvingly. Suspicious, but helpful."

## Content Tone

The tone should be silly, strange, and memorable, but not exhausting. It should have recurring characters and locations.

Recurring character ideas:

- **Manny Page**: an overdramatic manual-page spirit who explains commands.
- **Captain Alias**: gives shortcut advice, often too confidently.
- **The Permission Clerk**: refuses entry unless bits are set correctly.
- **The Archive Vending Machine**: dispenses files only when patterns match.
- **The Error Oracle**: translates intimidating errors into plain English.
- **The Dotfile Concierge**: appears when hidden files matter, acting far too formal about `ls -a`.

Narrative style:

- Short lines.
- No lore dumps.
- Punchy room descriptions.
- Jokes tied to the command concept.
- Funny failure messages that still teach.

Example room beat:

> The door has no handle. It has a prompt.
> A label blinks: "State your location."
> The terminal waits, smugly.

Expected command: `pwd`.

Success:

> The door accepts that you know where you are. Honestly, more than most doors can say.

## Testing Strategy

Unit tests:

- Challenge parsing.
- Command validators.
- Filesystem validators.
- Scoring rules.
- Settings behavior.
- Gradient/effect cue generation where deterministic.

Integration tests:

- Simulated beginner challenge attempts.
- Sandbox fixture setup and teardown.
- Operator task validation.
- Profile save/load.
- Reduced-motion rendering paths.

Manual verification:

- Run the TUI in a real terminal.
- Check keyboard navigation.
- Check color readability on dark and light terminal themes where possible.
- Verify no animation interferes with typing.
- Verify real shell tasks cannot escape the temp sandbox through normal gameplay paths.

## Implementation Plan

### Phase 1: Project Foundation

- Create Rust binary project.
- Add dependencies: `ratatui`, `crossterm`, serialization crate, temp-dir support, error handling, and test helpers.
- Set up TOML-based content loading and fixture directories.
- Establish module skeleton listed in the architecture section.
- Implement app event loop, screen routing, and graceful terminal restore on panic/error.

### Phase 2: Content and Validation Core

- Define content schema for chapters, rooms, cards, objectives, hints, fixtures, validators, and effect cues.
- Implement content loader and schema validation.
- Implement guided command validation.
- Implement simulated command output for beginner commands.
- Add unit tests for command variants and common beginner mistakes.

### Phase 3: Shared Game Engine

- Implement profile, settings, difficulty, pressure profile, scoring, attempts, hints, and progress.
- Build the shared challenge lifecycle: present objective, accept command, validate, emit feedback, trigger effects, advance.
- Add persistence for settings and progress.

### Phase 4: TUI Shell

- Build launcher with mode selector, difficulty selector, curriculum/deck access, settings, and profile summary.
- Build in-game layout: header, room/challenge view, lesson card, command panel, footer.
- Implement command-line editing, command history, hint key, settings key, and pause/back behavior.

### Phase 5: Effects System

- Add animation clock and effect cue pipeline.
- Implement truecolor gradient helper for objective and success text.
- Implement flicker, sweep, and reveal effects for narrative text and timers.
- Implement reduced-motion alternatives.
- Evaluate `tachyonfx` for effects that fit Ratatui rendering cleanly.
- Use `cli_animate` only where it fits outside the main full-screen renderer.

### Phase 6: Escape Room Mode

- Implement room state, objects, objective chains, room transitions, and clue reveals.
- Build the first Escape Room chapter, **Lost Terminal**.
- Include beginner lessons for `pwd`, `ls`, `cd`, `cat`, and basic paths.

### Phase 7: Dojo Mode

- Implement deck selection, drill loops, streaks, missed-card review, and micro-session lengths.
- Build beginner decks matching Lost Terminal skills.
- Add speed/streak feedback that respects pressure settings.

### Phase 8: Ops Sim Mode

- Implement incident flow and applied objective validation.
- Add real sandbox setup and cleanup.
- Build beginner incidents that apply navigation and inspection without overwhelming the player.
- Add Operator-ready filesystem validation even if early content stays beginner-friendly.

### Phase 9: Polish and Safety

- Add friendly error explanations.
- Add high contrast and animation speed settings.
- Add terminal capability checks for truecolor support where possible.
- Add cleanup/debug options for sandboxes.
- Tune copy for clarity, humor, and replayability.

### Phase 10: Verification and Iteration

- Run unit and integration tests.
- Manually play through all modes at Beginner difficulty.
- Test settings combinations: Cozy, Soft Urgency, Arcade, reduced motion, high contrast.
- Fix layout overflow, animation distraction, unclear hints, and unsafe shell behavior.
- Add more cards and chapters based on weak spots discovered during playtesting.

## Implementation Defaults

- Use **TOML** for challenge content in the first build. It fits Rust well, is pleasant for structured hand-authored content, and avoids YAML indentation surprises.
- Launch real shell mode as a clearly marked temporary subshell for the first build, then return to the TUI for validation. Embedded shell interaction can be explored later if needed.
- Store profile and settings data under the platform-appropriate app data directory using a crate such as `directories`, with `bashcards` as the app name.
- Support one default local profile in the first build. The data model should not block future multiple profiles, but the UI does not need profile switching yet.
- Use `cli_animate` only for non-Ratatui startup/loading flourishes or standalone progress moments. Ratatui and `tachyonfx` own the main full-screen animated experience.

## Definition of Done for First Full Build

- `bashcards` launches as a Rust TUI.
- User can choose Escape Room, Dojo, or Ops Sim.
- User can choose Beginner, Builder, Operator, or Chaos difficulty, even if higher difficulties initially reuse some content with stricter settings.
- Settings include pressure profile, reduced motion, high contrast, animation speed, hint style, and explanation behavior.
- Escape Room includes a playable Lost Terminal chapter.
- Dojo includes replayable beginner decks for the Lost Terminal commands.
- Ops Sim includes at least one applied beginner incident.
- Guided command input works with helpful validation and feedback.
- Real sandbox shell infrastructure exists for Operator/Chaos tasks.
- Visual effects include objective gradients, reveal/flicker/sweep animations, and success gradients.
- Effects obey guardrails and reduced-motion settings.
- Tests cover content parsing, validators, scoring, settings, and sandbox validation.
