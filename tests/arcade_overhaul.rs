use std::collections::HashSet;

use bashcards::animation::{Animator, Cue, MotionBudget};
use bashcards::app::App;
use bashcards::arcade::Cabinet;
use bashcards::content::ContentLibrary;
use bashcards::game::AttemptOutcome;
use bashcards::profile::{CabinetProgress, Profile};
use bashcards::settings::{AnimationSpeed, Difficulty, Settings};

#[test]
fn cabinet_registry_exposes_three_v1_cabinets_with_valid_manifests() {
    let cabinets = Cabinet::all();

    assert_eq!(
        cabinets.iter().map(Cabinet::id).collect::<Vec<_>>(),
        vec!["shell-motel", "monastery-of-forms", "midnight-carnival"]
    );

    for cabinet in cabinets {
        let content = ContentLibrary::bundled_for(*cabinet).expect("cabinet content");
        let manifest = content.cabinet().expect("cabinet manifest");

        assert_eq!(manifest.id, cabinet.id());
        assert_eq!(manifest.display_name, cabinet.display_name());
        assert!(!manifest.tagline.is_empty());
        assert!(!content.chapters().is_empty());
    }
}

#[test]
fn cabinet_launches_keep_existing_gameplay_but_namespace_progress() {
    let content = ContentLibrary::bundled_for(Cabinet::ShellMotel).expect("content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::ShellMotel,
        Difficulty::Beginner,
        Settings::default(),
    )
    .expect("launch cabinet");

    assert_eq!(app.session().cabinet(), Cabinet::ShellMotel);
    assert_eq!(
        app.session().current_objective_prompt(),
        Some("State your location.")
    );

    assert!(matches!(
        app.submit_command("pwd"),
        AttemptOutcome::Correct { .. }
    ));

    let progress = app
        .profile()
        .cabinet_progress(Cabinet::ShellMotel)
        .expect("shell motel progress");
    assert!(progress.completed_objectives.contains("state-location"));
    assert_eq!(progress.score, app.session().score() as u32);
    assert_eq!(progress.streak, app.session().streak() as u32);
    assert_eq!(progress.attempts, app.session().attempts() as u32);

    assert!(
        app.profile()
            .cabinet_progress(Cabinet::MonasteryOfForms)
            .is_none()
    );
}

#[test]
fn v1_profile_load_migrates_completed_objectives_to_shell_motel() {
    let dir = assert_fs::TempDir::new().expect("temp dir");
    let path = dir.path().join("profile.toml");
    std::fs::write(
        &path,
        r#"
        completed_objectives = ["state-location", "state-location", "list-room"]
        weak_topics = ["paths"]
        "#,
    )
    .expect("write v1 profile");

    let migrated = Profile::load_from(&path).expect("migrate profile");

    assert_eq!(migrated.schema_version, 2);
    assert_eq!(migrated.weak_topics, vec!["paths"]);
    let expected = HashSet::from(["state-location".to_string(), "list-room".to_string()]);
    assert_eq!(
        migrated
            .cabinet_progress(Cabinet::ShellMotel)
            .expect("shell progress")
            .completed_objectives,
        expected
    );

    let raw = std::fs::read_to_string(path).expect("profile rewritten");
    assert!(raw.contains("schema_version = 2"));
    assert!(raw.contains("[cabinets.shell-motel]"));
}

#[test]
fn v2_profile_round_trips_multiple_cabinet_progress_records() {
    let dir = assert_fs::TempDir::new().expect("temp dir");
    let path = dir.path().join("profile.toml");
    let mut profile = Profile::default();
    profile.set_cabinet_progress(
        Cabinet::ShellMotel,
        CabinetProgress {
            score: 220,
            streak: 2,
            attempts: 2,
            completed_objectives: HashSet::from(["state-location".to_string()]),
            last_objective_id: Some("state-location".to_string()),
            last_played: None,
        },
    );
    profile.set_cabinet_progress(
        Cabinet::MidnightCarnival,
        CabinetProgress {
            score: 140,
            streak: 1,
            attempts: 1,
            completed_objectives: HashSet::from(["ops-orientation".to_string()]),
            last_objective_id: Some("ops-orientation".to_string()),
            last_played: None,
        },
    );

    profile.save_to(&path).expect("save v2 profile");
    let loaded = Profile::load_from(&path).expect("load v2 profile");

    assert_eq!(
        loaded
            .cabinet_progress(Cabinet::ShellMotel)
            .expect("shell")
            .score,
        220
    );
    assert_eq!(
        loaded
            .cabinet_progress(Cabinet::MidnightCarnival)
            .expect("carnival")
            .completed_objectives
            .len(),
        1
    );
}

#[test]
fn animator_samples_reduced_motion_and_animation_speed_deterministically() {
    let mut normal = Animator::new(MotionBudget::from_settings(&Settings::default()));
    normal.fire(Cue::TitleSweep);
    normal.tick_to(std::time::Duration::from_millis(600));
    let normal_progress = normal.sample(&Cue::TitleSweep).expect("sample").progress;

    let snappy_settings = Settings {
        animation_speed: AnimationSpeed::Snappy,
        ..Settings::default()
    };
    let mut snappy = Animator::new(MotionBudget::from_settings(&snappy_settings));
    snappy.fire(Cue::TitleSweep);
    snappy.tick_to(std::time::Duration::from_millis(600));
    let snappy_progress = snappy.sample(&Cue::TitleSweep).expect("sample").progress;

    assert!(snappy_progress > normal_progress);

    let reduced_settings = Settings {
        reduced_motion: true,
        ..Settings::default()
    };
    let mut reduced = Animator::new(MotionBudget::from_settings(&reduced_settings));
    reduced.fire(Cue::IncorrectFlicker);
    reduced.tick(std::time::Duration::from_millis(1));
    let sample = reduced.sample(&Cue::IncorrectFlicker).expect("sample");

    assert_eq!(sample.progress, 1.0);
    assert!(reduced.is_complete(&Cue::IncorrectFlicker));
}
