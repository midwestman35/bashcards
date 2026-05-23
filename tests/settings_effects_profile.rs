use bashcards::animation::Cue;
use bashcards::arcade::Cabinet;
use bashcards::effects::{cues_for_success, gradient_cells};
use bashcards::profile::Profile;
use bashcards::settings::{
    AnimationSpeed, ExplainAfterSuccess, HintStyle, PressureProfile, Settings,
};

#[test]
fn default_settings_match_first_build_product_defaults() {
    let settings = Settings::default();

    assert_eq!(settings.pressure, PressureProfile::SoftUrgency);
    assert!(!settings.reduced_motion);
    assert!(!settings.high_contrast);
    assert_eq!(settings.animation_speed, AnimationSpeed::Normal);
    assert_eq!(settings.hint_style, HintStyle::Nudging);
    assert_eq!(
        settings.explain_after_success,
        ExplainAfterSuccess::FirstTimeOnly
    );
    assert_eq!(settings.session_length_target, 10);
    assert!(settings.theme_overrides.is_empty());
}

#[test]
fn reduced_motion_replaces_sweep_flicker_and_reveal_with_static_success_cues() {
    let animated = cues_for_success(&Settings::default());
    assert!(animated.contains(&Cue::CorrectGlow));
    assert!(animated.contains(&Cue::TitleSweep));
    assert!(animated.contains(&Cue::PromptReveal));
    assert!(animated.contains(&Cue::IncorrectFlicker));

    let reduced = Settings {
        reduced_motion: true,
        ..Settings::default()
    };
    let cues = cues_for_success(&reduced);
    assert!(cues.contains(&Cue::CorrectGlow));
    assert!(!cues.contains(&Cue::TitleSweep));
    assert!(!cues.contains(&Cue::PromptReveal));
    assert!(!cues.contains(&Cue::IncorrectFlicker));
}

#[test]
fn gradient_cells_are_deterministic_and_have_multiple_truecolor_steps() {
    let cells = gradient_cells("Objective complete");

    assert_eq!(cells.len(), "Objective complete".chars().count());
    assert_eq!(cells[0].ch, 'O');
    assert_ne!(cells.first().unwrap().rgb, cells.last().unwrap().rgb);
    assert!(
        cells
            .iter()
            .map(|cell| cell.rgb)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            > 3
    );
}

#[test]
fn profile_round_trips_progress_and_weak_topics() {
    let dir = assert_fs::TempDir::new().expect("temp dir");
    let path = dir.path().join("profile.toml");
    let mut profile = Profile::default();
    profile.record_objective(Cabinet::ShellMotel, "state-location", 110, 1, 1);
    profile.weak_topics.push("paths".to_string());

    profile.save_to(&path).expect("save profile");
    let loaded = Profile::load_from(&path).expect("load profile");

    assert!(
        loaded
            .cabinet_progress(Cabinet::ShellMotel)
            .expect("shell progress")
            .completed_objectives
            .contains("state-location")
    );
    assert_eq!(loaded.weak_topics, vec!["paths"]);
}
