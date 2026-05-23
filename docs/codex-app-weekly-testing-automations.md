# Codex-App Weekly Testing Automations

These are suggested automations for a Codex app workflow that keeps `bashcards`
playable while the content and TUI grow.

## Weekly Regression Run

- Trigger: every Monday morning and on demand from the Codex app.
- Command: `cargo test`
- Purpose: run unit and integration coverage for content parsing, validators,
  scoring, settings, profile persistence, sandbox safety, and beginner
  end-to-end mode completion.
- Expected artifact: summarized pass/fail report with failing test names and
  the first relevant compiler or assertion error.

## Weekly Format And Compile Gate

- Trigger: after the regression run.
- Commands:
  - `cargo fmt -- --check`
  - `cargo test --no-run`
- Purpose: catch formatting drift and compile breakage before deeper gameplay
  checks spend time.

## Weekly Agent Playthrough

- Trigger: after the compile gate passes.
- Command: `cargo test simulated_agents_complete_all_three_beginner_modes_end_to_end`
- Purpose: dispatch a Codex app agent to run the same Beginner scripts a player
  would use for Escape Room, Dojo Drills, and Ops Sim.
- Expected artifact: a short playthrough note listing each mode, commands sent,
  final score, attempts, and whether the session completed.

## Weekly Sandbox Safety Sweep

- Trigger: after the agent playthrough.
- Command: `cargo test sandbox_fixtures_are_created_and_filesystem_validation_stays_inside_root`
- Purpose: ensure Operator and Chaos sandbox infrastructure keeps fixture
  writes and validation inside disposable challenge roots.
- Expected artifact: pass/fail plus sandbox path examples from the run when
  debug logging is enabled.

## Weekly Non-Interactive Launch Smoke Test

- Trigger: after sandbox safety passes.
- Command: `cargo run --quiet`
- Purpose: verify the binary starts cleanly in non-interactive automation
  contexts and tells the user to run in an interactive terminal for the TUI.

## Weekly Spec Drift Review

- Trigger: after all commands pass.
- Inputs:
  - `docs/superpowers/specs/2026-05-22-bashcards-design-and-plan.md`
  - `tests/*.rs`
  - `content/*.toml`
  - `src/**/*.rs`
- Purpose: have a Codex app reviewer compare the current implementation against
  the first-build definition of done and open a follow-up task for missing
  coverage, weak content depth, layout risks, or sandbox safety gaps.

## Recommended Codex App Output

Each weekly run should produce `docs/reports/YYYY-MM-DD-weekly-test.md` with:

- commit SHA and branch
- commands run
- test counts
- failed checks, if any
- agent playthrough transcript summary
- sandbox safety result
- spec drift notes
- recommended next fixes
