use bashcards::effects::{EffectKind, cues_for_success, gradient_cells};
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
}

#[test]
fn reduced_motion_replaces_sweep_flicker_and_reveal_with_static_success_cues() {
    let animated = cues_for_success(&Settings::default());
    assert!(
        animated
            .iter()
            .any(|cue| cue.kind == EffectKind::SuccessGradient)
    );
    assert!(animated.iter().any(|cue| cue.kind == EffectKind::Sweep));
    assert!(animated.iter().any(|cue| cue.kind == EffectKind::Reveal));
    assert!(animated.iter().any(|cue| cue.kind == EffectKind::Flicker));

    let mut reduced = Settings::default();
    reduced.reduced_motion = true;
    let cues = cues_for_success(&reduced);
    assert!(cues.iter().all(|cue| cue.reduced_motion_safe));
    assert!(
        cues.iter()
            .any(|cue| cue.kind == EffectKind::SuccessGradient)
    );
    assert!(!cues.iter().any(|cue| cue.kind == EffectKind::Sweep));
    assert!(!cues.iter().any(|cue| cue.kind == EffectKind::Reveal));
    assert!(!cues.iter().any(|cue| cue.kind == EffectKind::Flicker));
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
    profile
        .completed_objectives
        .push("state-location".to_string());
    profile.weak_topics.push("paths".to_string());

    profile.save_to(&path).expect("save profile");
    let loaded = Profile::load_from(&path).expect("load profile");

    assert_eq!(loaded.completed_objectives, vec!["state-location"]);
    assert_eq!(loaded.weak_topics, vec!["paths"]);
}
