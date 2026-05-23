use crate::{
    animation::Cue,
    settings::Settings,
    theme::{Rgb, ThemeToken},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GradientCell {
    pub ch: char,
    pub rgb: (u8, u8, u8),
}

pub fn gradient_cells(text: &str) -> Vec<GradientCell> {
    let chars: Vec<char> = text.chars().collect();
    let steps = chars.len().saturating_sub(1).max(1) as f32;
    let start = ThemeToken::Cyan.rgb();
    let end = ThemeToken::Mint.rgb();
    chars
        .into_iter()
        .enumerate()
        .map(|(index, ch)| {
            let rgb = lerp_rgb(start, end, index as f32 / steps);
            GradientCell {
                ch,
                rgb: (rgb.0, rgb.1, rgb.2),
            }
        })
        .collect()
}

pub fn cues_for_success(settings: &Settings) -> Vec<Cue> {
    if settings.reduced_motion {
        return vec![Cue::ObjectiveGradient, Cue::CorrectGlow];
    }

    vec![
        Cue::ObjectiveGradient,
        Cue::PromptReveal,
        Cue::IncorrectFlicker,
        Cue::TitleSweep,
        Cue::CorrectGlow,
    ]
}

fn lerp_rgb(start: Rgb, end: Rgb, t: f32) -> Rgb {
    let lerp = |a: u8, b: u8| a as f32 + (b as f32 - a as f32) * t;
    Rgb(
        lerp(start.0, end.0).round() as u8,
        lerp(start.1, end.1).round() as u8,
        lerp(start.2, end.2).round() as u8,
    )
}
