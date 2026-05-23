# Bash Arcade · Roadmap

A durable list of features and improvements that are **deliberately out of scope for v1** but planned for later. Edit this file as priorities shift.

This is not a backlog of nice-to-haves. It is a list of things we said "yes to eventually" while explicitly leaving them out of the initial overhaul, so v1 stays bounded.

The companion documents:

- `docs/visual-design.md` — visual philosophy, durable
- `docs/superpowers/specs/design-overhaul/2026-05-23-bashcards-arcade-overhaul-design.md` — the v1 overhaul spec
- `docs/superpowers/specs/2026-05-22-bashcards-design-and-plan.md` — the original product spec (still load-bearing for gameplay design)

---

## Explicitly omitted (not on roadmap)

Some things are not "later" — they're "no." Listed here so we don't accidentally relitigate.

- **Network leaderboards, multiplayer, shared profiles.** Bash Arcade is a single-player local app. Cross-machine state is not a goal.
- **Weekly study schedule surface.** Superseded by the topic-cabinet arcade model. The original Mon–Sun framing in the deprecated `2026-05-22-bashcards-arcade-splash-addendum.md` is not coming back.

---

## v1.5 — quality and polish

Things that would make v1 feel more finished without changing its shape.

### Audio

- Carafe imprint chime (subtle, vapor-tonal, sub-1-second).
- Cabinet entry / exit transition sounds, distinct per cabinet accent.
- Correct / incorrect feedback tones, tied to existing `Cue::CorrectGlow` / `Cue::IncorrectFlicker`.
- Streak surge audio pulse.
- Optional ambient bed per cabinet (Shell Motel: distant elevator hum; Monastery: occasional bamboo wind; Carnival: faint carousel reel).
- Settings toggle: master volume, mute, ambient on/off.
- Implementation note: likely `rodio` or `cpal`. Audio is cross-platform-fragile; budget time for QA on macOS, Linux, Windows.

### "Resume where you left off"

- Profile remembers last cabinet played and last objective reached.
- On launch (after splash), focus that cabinet by default instead of the "least-played" rule from v1.
- A "continue" hint appears on the focused cabinet card if mid-progress.
- Opt-out: settings toggle that returns to v1 behavior.

---

## v2 — new mechanics

Things that change the gameplay surface.

### Cabinet unlock progression

- Optional gating: some cabinets unlock after others reach a completion threshold.
- Locked cabinets render dim with a small lock glyph and a hint about the unlock condition.
- Toggleable in settings — "free play" mode keeps everything available, "campaign" mode enables gating.
- Per-cabinet unlock conditions live in the manifest:
  ```toml
  [cabinet.unlock]
  requires = ["shell-motel"]
  threshold = "first-chapter-complete"
  ```

### Theme overrides for the carafe imprint

- Currently the imprint is fixed (deep field, soft cyan logomark).
- Add support for swapping the imprint to a "holiday" or "study-arc-themed" variant for special occasions.
- Lives in `src/ui/splash/carafe/variants.rs` as a const set of glyph + palette pairs.
- Settings toggle to pick imprint variant; default is the standard one.

---

## v3 — authoring and extensibility

Things that make the arcade easier to grow.

### Cabinet authoring tooling

- An in-app editor (or a separate CLI helper) for drafting new cabinet manifests without hand-editing TOML.
- Validates schema as you type.
- Live-preview the cabinet card and a sample objective.
- Possible separate binary: `cargo run --bin bashcards-author`.
- Honest assessment: this is high-effort and only worth building if cabinet velocity bottlenecks on TOML friction. Re-evaluate after the summer.

---

## Reviewing this doc

When v1 ships, walk through this list and:

1. Promote items that should land next into a concrete plan.
2. Demote or delete items that no longer feel worth doing.
3. Add anything that emerged from playing v1.

This is a living document. Edit it with the same energy as `docs/visual-design.md`.
