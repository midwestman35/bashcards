use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::theme::ThemeToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Builder,
    Operator,
    Chaos,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Builder => "Builder",
            Self::Operator => "Operator",
            Self::Chaos => "Chaos",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureProfile {
    CozyNoTimer,
    SoftUrgency,
    ArcadePressure,
}

impl PressureProfile {
    pub fn label(self) -> &'static str {
        match self {
            Self::CozyNoTimer => "Cozy No-Timer",
            Self::SoftUrgency => "Soft Urgency",
            Self::ArcadePressure => "Arcade Pressure",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::CozyNoTimer => Self::SoftUrgency,
            Self::SoftUrgency => Self::ArcadePressure,
            Self::ArcadePressure => Self::CozyNoTimer,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationSpeed {
    Calm,
    Normal,
    Snappy,
}

impl AnimationSpeed {
    pub fn label(self) -> &'static str {
        match self {
            Self::Calm => "calm",
            Self::Normal => "normal",
            Self::Snappy => "snappy",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Calm => Self::Normal,
            Self::Normal => Self::Snappy,
            Self::Snappy => Self::Calm,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HintStyle {
    Direct,
    Nudging,
    Minimal,
}

impl HintStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Nudging => "nudging",
            Self::Minimal => "minimal",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Direct => Self::Nudging,
            Self::Nudging => Self::Minimal,
            Self::Minimal => Self::Direct,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplainAfterSuccess {
    Always,
    FirstTimeOnly,
    Off,
}

impl ExplainAfterSuccess {
    pub fn label(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::FirstTimeOnly => "first time",
            Self::Off => "off",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Always => Self::FirstTimeOnly,
            Self::FirstTimeOnly => Self::Off,
            Self::Off => Self::Always,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeOverride {
    pub accent_primary: ThemeToken,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
