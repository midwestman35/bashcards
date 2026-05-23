use std::time::Duration;

use crate::{
    arcade::Cabinet,
    settings::{AnimationSpeed, Settings},
    theme::{Rgb, ThemeToken},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Cue {
    ImprintMaterialize { progress: f32 },
    BootLineReveal { line_idx: usize },
    TitleSweep,
    StatusFlicker { token: String },
    CabinetEnter { cabinet: Cabinet },
    CabinetExit,
    LobbyHover { cabinet: Cabinet },
    ObjectiveGradient,
    PromptReveal,
    HintFlash,
    CorrectGlow,
    IncorrectFlicker,
    StreakSurge { streak: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionBudget {
    pub reduced: bool,
    pub high_contrast: bool,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub progress: f32,
    pub alpha: f32,
    pub rgb: Rgb,
    pub offset: i16,
}

#[derive(Clone, Debug)]
struct ActiveCue {
    cue: Cue,
    elapsed: Duration,
}

#[derive(Clone, Debug)]
pub struct Animator {
    active: Vec<ActiveCue>,
    motion: MotionBudget,
}

impl MotionBudget {
    pub fn from_settings(settings: &Settings) -> Self {
        let speed = match settings.animation_speed {
            AnimationSpeed::Calm => 0.6,
            AnimationSpeed::Normal => 1.0,
            AnimationSpeed::Snappy => 1.4,
        };
        Self {
            reduced: settings.reduced_motion,
            high_contrast: settings.high_contrast,
            speed,
        }
    }
}

impl Animator {
    pub fn new(motion: MotionBudget) -> Self {
        Self {
            active: Vec::new(),
            motion,
        }
    }

    pub fn set_motion_budget(&mut self, motion: MotionBudget) {
        self.motion = motion;
    }

    pub fn fire(&mut self, cue: Cue) {
        self.active.retain(|active| !same_cue(&active.cue, &cue));
        self.active.push(ActiveCue {
            elapsed: Duration::ZERO,
            cue,
        });
    }

    pub fn fire_set(&mut self, cues: &[Cue]) {
        for cue in cues {
            self.fire(cue.clone());
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        let scaled_ms = if self.motion.reduced {
            0
        } else {
            (dt.as_millis() as f32 * self.motion.speed).round() as u64
        };
        animate::tick(scaled_ms as usize);

        for active in &mut self.active {
            active.elapsed = if self.motion.reduced {
                duration_for(&active.cue)
            } else {
                active
                    .elapsed
                    .saturating_add(Duration::from_millis(scaled_ms))
            };
        }
        self.active
            .retain(|active| active.elapsed <= duration_for(&active.cue));
    }

    pub fn sample(&self, cue: &Cue) -> Option<Sample> {
        let active = self
            .active
            .iter()
            .find(|active| same_cue(&active.cue, cue))?;
        let duration = duration_for(&active.cue);
        let progress = if self.motion.reduced || duration.is_zero() {
            1.0
        } else {
            (active.elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
        };
        let eased = ease_out(progress);
        let rgb = if self.motion.high_contrast {
            ThemeToken::Cyan.rgb()
        } else {
            rgb_for(&active.cue, eased)
        };
        Some(Sample {
            progress: eased,
            alpha: eased,
            rgb,
            offset: ((1.0 - eased) * 3.0).round() as i16,
        })
    }

    pub fn is_complete(&self, cue: &Cue) -> bool {
        !self.active.iter().any(|active| same_cue(&active.cue, cue))
            || self
                .active
                .iter()
                .find(|active| same_cue(&active.cue, cue))
                .map(|active| active.elapsed >= duration_for(&active.cue))
                .unwrap_or(true)
    }

    pub fn tick_to(&mut self, t: Duration) {
        for active in &mut self.active {
            active.elapsed = if self.motion.reduced {
                duration_for(&active.cue)
            } else {
                Duration::from_secs_f32(t.as_secs_f32() * self.motion.speed)
            };
        }
    }
}

fn same_cue(left: &Cue, right: &Cue) -> bool {
    match (left, right) {
        (Cue::ImprintMaterialize { .. }, Cue::ImprintMaterialize { .. }) => true,
        (Cue::BootLineReveal { line_idx: a }, Cue::BootLineReveal { line_idx: b }) => a == b,
        (Cue::TitleSweep, Cue::TitleSweep) => true,
        (Cue::StatusFlicker { token: a }, Cue::StatusFlicker { token: b }) => a == b,
        (Cue::CabinetEnter { cabinet: a }, Cue::CabinetEnter { cabinet: b }) => a == b,
        (Cue::CabinetExit, Cue::CabinetExit) => true,
        (Cue::LobbyHover { cabinet: a }, Cue::LobbyHover { cabinet: b }) => a == b,
        (Cue::ObjectiveGradient, Cue::ObjectiveGradient) => true,
        (Cue::PromptReveal, Cue::PromptReveal) => true,
        (Cue::HintFlash, Cue::HintFlash) => true,
        (Cue::CorrectGlow, Cue::CorrectGlow) => true,
        (Cue::IncorrectFlicker, Cue::IncorrectFlicker) => true,
        (Cue::StreakSurge { .. }, Cue::StreakSurge { .. }) => true,
        _ => false,
    }
}

fn duration_for(cue: &Cue) -> Duration {
    let ms = match cue {
        Cue::ImprintMaterialize { .. } => 1_000,
        Cue::BootLineReveal { .. } => 120,
        Cue::TitleSweep => 1_200,
        Cue::StatusFlicker { .. } => 250,
        Cue::CabinetEnter { .. } => 400,
        Cue::CabinetExit => 300,
        Cue::LobbyHover { .. } => 6_000,
        Cue::ObjectiveGradient => 700,
        Cue::PromptReveal => 600,
        Cue::HintFlash => 350,
        Cue::CorrectGlow => 500,
        Cue::IncorrectFlicker => 250,
        Cue::StreakSurge { .. } => 800,
    };
    Duration::from_millis(ms)
}

fn rgb_for(cue: &Cue, progress: f32) -> Rgb {
    let start = match cue {
        Cue::IncorrectFlicker | Cue::StatusFlicker { .. } => ThemeToken::NeonMagenta.rgb(),
        Cue::CorrectGlow | Cue::StreakSurge { .. } => ThemeToken::Mint.rgb(),
        Cue::HintFlash => ThemeToken::Violet.rgb(),
        Cue::CabinetEnter { cabinet } | Cue::LobbyHover { cabinet } => cabinet_color(*cabinet),
        _ => ThemeToken::Pink.rgb(),
    };
    let end = match cue {
        Cue::IncorrectFlicker => ThemeToken::DimProse.rgb(),
        Cue::CorrectGlow => ThemeToken::Cyan.rgb(),
        Cue::TitleSweep => ThemeToken::Mint.rgb(),
        _ => ThemeToken::Cyan.rgb(),
    };
    lerp_rgb(start, end, progress)
}

fn cabinet_color(cabinet: Cabinet) -> Rgb {
    match cabinet {
        Cabinet::ShellMotel => ThemeToken::Pink.rgb(),
        Cabinet::MonasteryOfForms => ThemeToken::Cyan.rgb(),
        Cabinet::MidnightCarnival => ThemeToken::Mint.rgb(),
    }
}

fn lerp_rgb(start: Rgb, end: Rgb, t: f32) -> Rgb {
    let lerp = |a: u8, b: u8| a as f32 + (b as f32 - a as f32) * t;
    Rgb(
        lerp(start.0, end.0).round() as u8,
        lerp(start.1, end.1).round() as u8,
        lerp(start.2, end.2).round() as u8,
    )
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
