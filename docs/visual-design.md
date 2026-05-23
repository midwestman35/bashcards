# Bash Arcade · Visual Design Philosophy

A durable reference for the arcade's look and feel. Edit this file when you want to change the visual language — the design spec at `docs/superpowers/specs/2026-05-23-bashcards-arcade-overhaul-design.md` references this document as the source of truth for visuals.

This is not a spec. It is a style guide. Code in `src/theme/` mirrors what is written here; if the two disagree, change one or the other deliberately, not by drift.

---

## 1. The blend

Three thematic layers stack:

1. **Vapor primary** — the resting palette and the chrome. Pastel pink, cyan, violet, mint, soft yellow. This is what the arcade *feels like* most of the time.
2. **Liminal field** — the background and the negative space. Deep purple→indigo gradient. The room *breathes*; it doesn't blink. Empty pastel mall at 3am, not a poster of one.
3. **Famicom punctuation** — used sparingly. Neon magenta + deep cyan + navy inset. Only on selected states, flicker cues, and boot status tokens. This is what gives "you picked this cabinet" the JRPG-menu kick.

Rule of thumb: **at least 80% vapor, at most 20% Famicom punctuation, in any given frame.** Liminal field is wallpaper — it doesn't count toward the ratio.

---

## 2. Palette tokens

These are the only colors the app should use. Every other RGB value is a bug.

### Vapor primary

| Token         | Hex       | Use                                          |
|---            |---        |---                                           |
| `pink`        | `#ff9ad2` | accents, frames, Shell Motel primary         |
| `cyan`        | `#8af0ff` | accents, sub-frames, Monastery primary       |
| `violet`      | `#c8a8ff` | secondary text, hint copy                    |
| `mint`        | `#c9ffb3` | success cues, Carnival primary, "OK" tokens  |
| `yellow`      | `#ffe8a3` | streak / score numerals, warning tokens      |
| `dim_prose`   | `#9f87c8` | tertiary prose, dim hints                    |

### Liminal field

| Token         | Hex       | Use                                          |
|---            |---        |---                                           |
| `deep_field`  | `#1b0a36` | gradient top                                 |
| `mid_field`   | `#2a1052` | gradient middle                              |
| `low_field`   | `#1f0a44` | gradient bottom                              |

The field is rendered as a **vertical gradient**, never a horizontal one and never a single solid. Solid `#000` is reserved for high-contrast mode.

### Famicom punctuation

| Token            | Hex       | Use                                                  |
|---               |---        |---                                                   |
| `neon_magenta`   | `#ff3e9d` | selected cabinet border, incorrect flicker          |
| `deep_cyan`      | `#00d4ff` | "press enter" prompt, selected status               |
| `navy_inset`     | `#2e3aa0` | selected cabinet background inset                   |

Famicom tokens never appear on idle UI. If you find yourself reaching for them in a static layout, switch to a vapor token instead.

---

## 3. Liminal mood, concretely

The "empty pastel mall at 3am" feeling is three render-time choices, not vibes:

1. **Vertical purple gradient field** (`deep_field → mid_field → low_field`). Subtle enough that text contrast holds, atmospheric enough to feel like a room.
2. **Cabinet cards float.** Whitespace around them; the arcade frame is intentionally larger than its contents. Never crowd to the edges.
3. **Idle breath.** When no cue is firing, `Cue::LobbyHover` runs at a slow 6-second cycle on the selected cabinet only. No flashing elsewhere. The room *breathes*; it doesn't blink.

Avoid:

- Solid black backgrounds (reserved for high contrast).
- Borders touching the terminal edges.
- More than one element animating at the same time at rest.
- ASCII illustrations of malls, sunsets, or statues. We are not a poster.

---

## 4. Per-cabinet identity

Each cabinet keeps its own voice through (a) glyph, (b) accent token, and (c) tagline. The chrome around them stays vapor. Internally, the in-game screen uses the cabinet's accent color for its header bar and prompt cursor, so the cabinet feels like a different room without becoming a different app.

### Shell Motel · `▦`

- **Genre:** Escape Room
- **Accent primary:** `pink` (`#ff9ad2`)
- **Accent secondary:** `cyan`
- **Tagline:** *a hallway asks where you are*
- **Voice:** Curious, slightly haunted, instructive. Manny Page (the help daemon) is melodramatic but never cruel.
- **Visual hook:** Frame borders use pink. Room transitions fade pink → cyan → pink. Doors are pink-bordered. Successfully unlocking something sweeps mint across the prompt bar.

### Monastery of Forms · `⛩`

- **Genre:** Dojo Drills
- **Accent primary:** `cyan` (`#8af0ff`)
- **Accent secondary:** `violet`
- **Tagline:** *repeat until the scroll smiles*
- **Voice:** Patient, koan-like, never punishing. Wrong answers are training moments, not failures.
- **Visual hook:** Frame borders use cyan. Streak counters pulse cyan→mint when streak grows. Speed-round timer (Operator/Chaos) is rendered as a thin violet bar, never red.

### Midnight Carnival · `◉`

- **Genre:** Ops Sim
- **Accent primary:** `mint` (`#c9ffb3`)
- **Accent secondary:** `yellow`
- **Tagline:** *find the lost logs*
- **Voice:** Slightly noir, dryly funny, occasional fairground absurdity. Incidents are odd but the work is real.
- **Visual hook:** Frame borders use mint. Investigation results (file lists, grep matches) render with yellow line numbers. Sandbox commands are prefixed with a deep_cyan `!`.

### Future cabinets

When adding a summer cabinet:

1. Pick a glyph from the Unicode box/symbol blocks. Single character. No emoji.
2. Pick `accent_primary` from the vapor tokens above. Don't invent new colors.
3. Pick a tagline ≤ 6 words, lowercase except proper nouns.
4. Pick a voice in one sentence and add it as a comment in the cabinet's manifest.
5. Update Section 4 of this doc with the new cabinet.

---

## 4.5 Carafe imprint splash

The arcade boots in two stages. The first is **carafe** — the imprint logo, in the style of a PS1 / Dreamcast / N64 boot screen. It plays once per process invocation and is skippable by Enter or Esc at any moment.

**Why a separate stage.** The imprint signals identity, not friction. It is intentionally short, atmospheric, and unbranded by the bash arcade itself. Treat it the way Sony's "Sony Computer Entertainment Presents" preceded every PS1 game.

**Visual specifics.**

- **Field:** radial gradient on the deep-field tokens (`#160830` center → `#060212` edge). Liminal, mysterious, not flat black. This is one of two cases where the field is radial; the other is high-contrast mode (solid `#000`).
- **Logomark:** "carafe" rendered in low-rez braille glyphs (Unicode block U+2800–U+28FF). Hand-designed as a 6-character × 2-row const glyph table in `src/ui/splash/carafe.rs`. The braille rez gives the pseudo-pixel, sub-character resolution that PS1/Dreamcast boots had.
- **Color:** dots resolve from dim violet (`#6a7eb8`) to soft cyan (`#c8e8ff`) with a faint cyan glow. Tagline ("arcade imprint") in dim slate (`#5f7aa8`) at 0.3em letter spacing — the only typography in the app that uses tracking like that.
- **Layout:** centered, lots of negative space. Never crowd to the edges.

**Animation cue.** A new `Cue::ImprintMaterialize { progress: f32 }` is added to the animation runtime. It animates *glyph content* (sparse dots → resolved logomark) rather than color or position. The animator drives `progress` from 0.0 → 1.0 over 1.0 s with ease-out timing. The renderer samples `progress` and chooses how many of each letter's braille dots to render.

**Timing budget (normal motion).**

| Stage              | Duration |
|---                 |---       |
| Field idle (dot motes drift) | 0.8 s |
| Materialize (dots → glyphs)  | 1.0 s |
| Hold                         | 1.5 s |
| Fade out                     | 0.2 s |
| **Total**                    | **3.5 s** |

**Reduced motion.** Shows the resolved logomark for 1.0 s and then advances to the bash arcade boot. Dot motes don't drift; materialize doesn't tween.

**Skip.** Enter or Esc at any frame jumps straight to the bash arcade boot console.

**Audio.** None in v1. (PS1 had its chime; we don't ship sound yet — see `docs/roadmap.md`.)

---

## 5. Animation philosophy

Animation reinforces state. It does not decorate idle UI.

### Cue catalog

Every animation in the app is one of these (defined in `src/animation/mod.rs::Cue`):

| Cue                  | Trigger                              | Feel                                  |
|---                   |---                                   |---                                    |
| `BootLineReveal`     | splash, per boot-log line            | type-on, ~120ms per line              |
| `TitleSweep`         | splash, after boot lines complete    | pastel sweep across title, ~1.2s      |
| `StatusFlicker`      | splash, on `OK / READY / NO GUILT`   | one flicker before settling           |
| `CabinetEnter`       | lobby → cabinet                      | scanline wipe + glow, ~400ms          |
| `CabinetExit`        | cabinet → lobby                      | reverse, ~300ms                       |
| `LobbyHover`         | idle on selected cabinet             | slow pulse, 6s cycle                  |
| `ObjectiveGradient`  | new objective revealed               | gradient sweep on objective text      |
| `PromptReveal`       | new prompt shown                     | typed-on reveal, ~80ms/char           |
| `HintFlash`          | hint summoned                        | violet flash on hint line             |
| `CorrectGlow`        | correct answer                       | mint pulse on prompt bar              |
| `IncorrectFlicker`   | incorrect answer                     | one Famicom-magenta flicker           |
| `StreakSurge`        | streak ≥ 3                           | stronger pulse, cabinet-accent color  |

### Motion budget

Three settings interact:

- **`reduced_motion: true`** → animator returns final-state samples immediately. No tween, no flicker. Functionally equivalent to no animation.
- **`animation_speed`** → maps internally to cue-duration multipliers. Settings already exposes this as `Calm | Normal | Snappy` (currently inert); the overhaul wires it to the animator. Default is `Normal`. *Implementation note: routes through `vyfor/animate`'s time-scale primitive if available; otherwise the field persists in settings but has no runtime effect, with a `// TODO: enable when animate exposes time_scale` marker on the wiring site.*
- **`high_contrast: true`** → cues that depend on color tween (e.g., `CorrectGlow` going mint→cyan) snap to one solid token instead of tweening.

Idle UI runs only `LobbyHover` and the cabinet header breath. Nothing else animates at rest.

---

## 6. Selection vs. focus

Two states the navigation can be in. Easy to confuse; the design separates them deliberately.

- **Focused** (keyboard hover, `j` / `k`): vapor accent border (pink for Shell Motel, cyan for Monastery, mint for Carnival). Subtle.
- **Selected** (`Enter` pressed but cabinet not yet launched, or "current" in launcher): Famicom navy inset + magenta border. The one place the JRPG-menu feeling kicks in.

When the player launches a cabinet, the selected state plays `CabinetEnter` and the lobby falls away.

---

## 7. High contrast

`Settings.high_contrast = true` swaps in `PALETTE_HIGH_CONTRAST` — a parallel set of WCAG-AA-compliant pairings. Specifically:

- `deep_field` / `mid_field` / `low_field` → solid `#000`.
- `dim_prose` → `#e0d4ff` (brighter, AA on `#000`).
- Cabinet accents stay (they're already AA on `#000`).
- Famicom magenta stays (AA on `#000`).
- The vertical gradient is disabled.

High contrast is a flag, not a separate theme. The cabinet identities still hold.

---

## 8. Truecolor fallback

If the terminal can't render truecolor, `theme::adapt::to_ansi(rgb)` maps each token to the nearest of the 256-color palette before handing it to ratatui. The arcade still works; gradients step rather than tween. Cabinet accents pick the closest 256-color match, which keeps cabinet identity intact even on degraded terminals.

If the terminal can't render 256 colors either (very rare on modern shells), the app falls back to a 16-color "minimal vapor" mapping that preserves at least cabinet-color distinction.

---

## 9. Editing this file

This doc is intentionally durable. When you want to change the visual language:

1. Edit this file first.
2. Mirror the change in `src/theme/`.
3. Update any in-game cabinet manifests whose `accent_primary` you changed.
4. Bump a small `theme_version` constant in `src/theme/mod.rs` so reduced-motion test snapshots can be regenerated cleanly.

Don't edit `src/theme/` first and let this doc drift. The doc is the source of truth.
