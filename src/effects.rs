use crate::settings::Settings;

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
            GradientCell {
                ch,
                rgb: (r as u8, g as u8, b as u8),
            }
        })
        .collect()
}

pub fn cues_for_success(_settings: &Settings) -> Vec<EffectCue> {
    if _settings.reduced_motion {
        return vec![
            EffectCue {
                kind: EffectKind::ObjectiveGradient,
                reduced_motion_safe: true,
            },
            EffectCue {
                kind: EffectKind::SuccessGradient,
                reduced_motion_safe: true,
            },
        ];
    }

    vec![
        EffectCue {
            kind: EffectKind::ObjectiveGradient,
            reduced_motion_safe: true,
        },
        EffectCue {
            kind: EffectKind::Reveal,
            reduced_motion_safe: false,
        },
        EffectCue {
            kind: EffectKind::Flicker,
            reduced_motion_safe: false,
        },
        EffectCue {
            kind: EffectKind::Sweep,
            reduced_motion_safe: false,
        },
        EffectCue {
            kind: EffectKind::SuccessGradient,
            reduced_motion_safe: true,
        },
    ]
}
