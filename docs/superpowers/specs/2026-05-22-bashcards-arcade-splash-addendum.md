> **Superseded by `docs/superpowers/specs/design-overhaul/2026-05-23-bashcards-arcade-overhaul-design.md`.**
>
> This document predates the arcade overhaul. Its weekly-schedule framing
> (Mon–Sun cabinet menu) was rejected in favor of a topic-cabinet arcade
> model where each cabinet is a standalone topic game. The vapor-console
> palette and boot-line concept survive — pulled forward into the new spec
> and `docs/visual-design.md`. Kept on disk for history.

# bashcards Arcade Splash and Lobby Addendum

## Decision

The Bashcards launcher should use the **Vapor Console** direction: a restrained vaporwave boot console rendered as a real terminal UI.

This replaces the earlier idea of a large scenic arcade lobby. Bashcards can still frame the weekly study schedule as an arcade, but the first implementation should feel like a stylized TUI startup screen, not a poster, landing page, or decorative full-screen scene.

The locked flow is:

1. **Boot Console Splash**
2. **Weekly Console Menu**
3. **Selected Daily Mode Launch**

The visual target is represented by the local brainstorming mockup:

`.superpowers/brainstorm/73942-1779429191/content/actual-terminal-render.html`

That mockup is not a committed artifact because `.superpowers/` is ignored. This addendum is the committed source of truth.

## Visual Direction

The style should be vaporwave-inspired but terminal-faithful.

Use:

- Pastel pink, cyan, mint, violet, muted lavender, and soft yellow.
- Monospace type only.
- Ratatui-style borders and panels.
- Truecolor text spans for title, boot lines, selected rows, objectives, and status.
- Faux retro terminal/window language through labels and frame text, not browser chrome.
- Minimal ASCII accents such as path pyramids, palm-file motifs, or small system icons.
- Optional scanline/flicker impression through brief cell color changes.

Avoid:

- Real logos, real brand marks, or specific copyrighted characters.
- Large scenic vaporwave elements such as giant sunsets, statues, grid landscapes, or poster-like compositions.
- Browser-only visual effects that cannot map to a TUI.
- Heavy decorative art that competes with the menu.
- Effects that obscure command input, command output, or selected menu state.

The screen should feel like a pastel terminal program from a strange 1990s training computer.

## Splash Screen

The splash is a short boot sequence. It should be skippable.

Elements:

- Top border/title: `bashcards boot console`.
- Large `bashcards` ASCII title using gradient-colored rows.
- Subtitle: `PASTEL TERMINAL TRAINING SYSTEM // 199X // SOFT URGENCY // TRUE BEGINNER`.
- Boot log lines:
  - `initializing weekly study loop`
  - `loading bash game cabinet`
  - `mounting python clone harness`
  - `indexing rust trace paths`
  - `checking reset protocol`
  - `opening practice console`
- Status tokens such as `OK`, `NO GUILT`, and `READY`.
- A prompt line such as `> select Monday Bash Game`.

Animation behavior:

- Boot lines reveal one at a time.
- Status tokens can flicker once before settling.
- The title can receive a subtle sweep of pink, cyan, mint, and violet.
- The prompt cursor can blink.
- `Esc` skips the splash and lands on the weekly console menu.
- Reduced motion shows the complete boot state immediately.

## Weekly Console Menu

After the splash, the same terminal frame becomes the launcher.

The layout should use three main regions:

- **Weekly Cabinets**: a vertical list of Monday through Sunday rituals.
- **Selected Session**: description, objective, expected learning target, and launch text.
- **Profile**: difficulty, pressure, motion, hint style, and explanation setting.

The weekly menu is called an arcade in the product language, but it renders as a console menu.

Default daily entries:

- `MON  Bash Game          30-45m`
- `TUE  Python Clone       tiny`
- `WED  Code Reading       trace`
- `THU  Tiny Patch         guard`
- `FRI  Public Artifact    devlog`
- `SAT  Build Session      90m`
- `SUN  Reset              review`

The current day should be selected by default. If the user launches on a different day, that day gets the active row. Manual navigation can move away from the current day.

Selected row styling:

- Use a readable background highlight.
- Use a left marker or reversed color cell.
- Keep text fully legible on dark terminals.
- Do not rely on glow alone to indicate selection.

## Initial Selected Session Copy

For Monday, the selected session should read:

- Title: `Shell Motel Orientation`
- Summary: `Learn by doing: pwd, ls, cd, cat`
- Narrative text:
  - `A hallway asks where you are.`
  - `The correct answer is not confidence.`
  - `It is a command.`
- Objective: `state your current location`
- Expected command family: `pwd`

This copy can evolve as content grows, but it defines the tone: short, playful, instructional, and not over-lored.

## Controls

Launcher controls:

- `j` / `k` or arrow keys: move selection.
- `Enter`: launch selected daily mode.
- `s`: settings.
- `?`: contextual hint or explanation.
- `Esc`: skip boot when splash is active; back from settings when in launcher.
- `q`: quit from launcher.

Controls should appear in a footer line.

## Settings Integration

The launcher should show the current settings summary without forcing a settings visit.

Default profile display:

- Difficulty: `Beginner`
- Pressure: `Soft`
- Motion: `Normal`
- Hints: `Direct`
- Explain: `On`

Settings remain the same as the main Bashcards spec:

- Pressure profile: Cozy No-Timer, Soft Urgency, Arcade Pressure.
- Difficulty: Beginner, Builder, Operator, Chaos.
- Motion: reduced motion toggle and animation speed.
- Accessibility: high contrast.
- Learning preferences: hint style and explanation behavior.

The Vapor Console theme must respect high contrast and reduced motion.

## Implementation Requirements

The splash and launcher should be implemented inside the Rust/Ratatui app.

Recommended modules:

- `ui::splash`: boot console rendering and splash animation state.
- `ui::launcher`: weekly console menu rendering and navigation.
- `ui::theme::vapor_console`: palette, text styles, selected row styles, and border styles.
- `effects`: title sweep, boot line reveal, status flicker, cursor blink, reduced-motion substitutions.
- `settings`: theme selection and accessibility behavior.

The launcher should be data-backed enough that daily entries are not hard-coded across multiple UI files. A small static list is acceptable for the first build if it is represented as structured data.

## Terminal Constraints

The design must remain faithful to terminal rendering.

Allowed:

- Colored text spans.
- Unicode box borders.
- ASCII title art.
- Highlighted rows.
- Simple panel divisions.
- Timed reveal/flicker/sweep effects.
- Color brightness shifts that imply scanlines or glow.

Not required:

- True blur.
- Pixel glow.
- Image rendering.
- GPU rendering.
- Full-screen vaporwave illustration.

The build should look good in a standard modern terminal with truecolor support. If truecolor is unavailable, the UI should degrade to a simpler high-contrast palette.

## Definition of Done

The arcade splash and launcher are complete when:

- Bashcards launches into the Vapor Console splash.
- The splash can be skipped.
- Reduced motion bypasses animated reveals.
- The weekly console menu lists all seven study rituals.
- The current day is selected by default.
- The settings summary is visible.
- The selected session panel updates when moving through the daily entries.
- Monday can launch the Bash Game flow from the existing main spec.
- The UI remains legible at common terminal sizes.
- No decorative effect obscures text, input, output, or selection state.
