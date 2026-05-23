use bashcards::app::App;
use bashcards::arcade::Cabinet;
use bashcards::content::ContentLibrary;
use bashcards::game::AttemptOutcome;
use bashcards::settings::{Difficulty, PressureProfile, Settings};

#[test]
fn beginner_escape_room_teaches_orientation_listing_movement_and_inspection() {
    let content = ContentLibrary::bundled_for(Cabinet::ShellMotel).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::ShellMotel,
        Difficulty::Beginner,
        Settings::default(),
    )
    .expect("launch escape room");

    assert_eq!(
        app.session().current_objective_prompt(),
        Some("State your location.")
    );
    assert!(matches!(
        app.submit_command("ls"),
        AttemptOutcome::Incorrect { .. }
    ));
    assert!(matches!(
        app.submit_command("pwd"),
        AttemptOutcome::Correct { .. }
    ));
    assert!(
        app.session()
            .last_command_output()
            .contains("/shell-motel/room-101")
    );
    assert_eq!(
        app.session().current_objective_prompt(),
        Some("List what the room contains.")
    );
    assert!(matches!(
        app.submit_command("ls"),
        AttemptOutcome::Correct { .. }
    ));
    assert!(app.session().last_command_output().contains("lobby"));
    assert_eq!(
        app.session().current_objective_prompt(),
        Some("Move into the lobby directory.")
    );
    assert!(matches!(
        app.submit_command("cd lobby"),
        AttemptOutcome::Correct { .. }
    ));
    assert_eq!(
        app.session().current_objective_prompt(),
        Some("Read the lobby README.")
    );
    assert!(matches!(
        app.submit_command("cat README"),
        AttemptOutcome::Correct { .. }
    ));
    assert!(app.session().completed());
}

#[test]
fn pressure_profiles_change_scoring_without_blocking_beginner_completion() {
    let settings = Settings {
        pressure: PressureProfile::CozyNoTimer,
        ..Settings::default()
    };
    let content = ContentLibrary::bundled_for(Cabinet::MonasteryOfForms).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::MonasteryOfForms,
        Difficulty::Beginner,
        settings,
    )
    .expect("launch dojo");

    assert!(matches!(
        app.submit_command("pwd"),
        AttemptOutcome::Correct { .. }
    ));
    assert_eq!(app.session().score(), 100);

    let arcade = Settings {
        pressure: PressureProfile::ArcadePressure,
        ..Settings::default()
    };
    let content = ContentLibrary::bundled_for(Cabinet::MonasteryOfForms).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::MonasteryOfForms,
        Difficulty::Beginner,
        arcade,
    )
    .expect("launch dojo");

    assert!(matches!(
        app.submit_command("pwd"),
        AttemptOutcome::Correct { .. }
    ));
    assert!(app.session().score() > 100);
}

#[test]
fn dojo_tracks_streaks_missed_cards_and_session_length() {
    let settings = Settings {
        session_length_target: 3,
        ..Settings::default()
    };
    let content = ContentLibrary::bundled_for(Cabinet::MonasteryOfForms).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::MonasteryOfForms,
        Difficulty::Beginner,
        settings,
    )
    .expect("launch dojo");

    assert!(matches!(
        app.submit_command("pwd"),
        AttemptOutcome::Correct { .. }
    ));
    assert_eq!(app.session().streak(), 1);
    assert!(matches!(
        app.submit_command("cat README"),
        AttemptOutcome::Incorrect { .. }
    ));
    assert_eq!(app.session().streak(), 0);
    assert_eq!(
        app.session().missed_objectives(),
        &["ls-recall".to_string()]
    );
}

#[test]
fn dojo_clean_path_completes_with_full_streak_and_no_missed_cards() {
    let content = ContentLibrary::bundled_for(Cabinet::MonasteryOfForms).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::MonasteryOfForms,
        Difficulty::Beginner,
        Settings::default(),
    )
    .expect("launch dojo");

    for command in ["pwd", "ls", "cat README"] {
        assert!(matches!(
            app.submit_command(command),
            AttemptOutcome::Correct { .. }
        ));
    }

    assert!(app.session().completed());
    assert_eq!(app.session().streak(), 3);
    assert!(app.session().missed_objectives().is_empty());
}

#[test]
fn operator_ops_sim_starts_with_real_sandbox_infrastructure() {
    let content = ContentLibrary::bundled_for(Cabinet::MidnightCarnival).expect("bundled content");
    let app = App::launch_cabinet(
        content,
        Cabinet::MidnightCarnival,
        Difficulty::Operator,
        Settings::default(),
    )
    .expect("launch operator ops sim");

    assert!(app.session().uses_real_sandbox());
    let root = app.session().sandbox_root().expect("sandbox root");
    assert!(root.ends_with("bashcards-operator"));
    assert!(root.join("report.txt").exists());
}

#[test]
fn operator_ops_sim_can_run_commands_inside_sandbox_and_record_output() {
    let content = ContentLibrary::bundled_for(Cabinet::MidnightCarnival).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::MidnightCarnival,
        Difficulty::Operator,
        Settings::default(),
    )
    .expect("launch operator ops sim");

    let outcome = app.submit_command("!cat report.txt");

    assert!(matches!(outcome, AttemptOutcome::Correct { .. }));
    assert!(
        app.session()
            .last_command_output()
            .contains("Moonbase report")
    );
    assert_eq!(
        app.session().current_objective_prompt(),
        Some("State your location.")
    );
}

#[test]
fn operator_sandbox_commands_do_not_overwrite_saved_cabinet_progress() {
    let dir = assert_fs::TempDir::new().expect("temp dir");
    let profile_path = dir.path().join("profile.toml");
    let content = ContentLibrary::bundled_for(Cabinet::MidnightCarnival).expect("bundled content");
    let mut beginner = App::launch_cabinet_with_profile_path(
        content,
        Cabinet::MidnightCarnival,
        Difficulty::Beginner,
        Settings::default(),
        profile_path.clone(),
    )
    .expect("launch beginner ops");

    for command in ["pwd", "ls", "cat report.txt"] {
        assert!(matches!(
            beginner.submit_command(command),
            AttemptOutcome::Correct { .. }
        ));
    }

    let before = bashcards::profile::Profile::load_from(&profile_path)
        .expect("profile saved")
        .cabinet_progress(Cabinet::MidnightCarnival)
        .expect("carnival progress")
        .clone();

    let content = ContentLibrary::bundled_for(Cabinet::MidnightCarnival).expect("bundled content");
    let mut operator = App::launch_cabinet_with_profile_path(
        content,
        Cabinet::MidnightCarnival,
        Difficulty::Operator,
        Settings::default(),
        profile_path.clone(),
    )
    .expect("launch operator ops");

    assert!(matches!(
        operator.submit_command("!cat report.txt"),
        AttemptOutcome::Correct { .. }
    ));

    let after = bashcards::profile::Profile::load_from(&profile_path)
        .expect("profile saved")
        .cabinet_progress(Cabinet::MidnightCarnival)
        .expect("carnival progress")
        .clone();

    assert_eq!(after.score, before.score);
    assert_eq!(after.streak, before.streak);
    assert_eq!(after.attempts, before.attempts);
    assert_eq!(after.completed_objectives, before.completed_objectives);
}

#[test]
fn beginner_ops_sim_reads_report_from_simulated_world() {
    let content = ContentLibrary::bundled_for(Cabinet::MidnightCarnival).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::MidnightCarnival,
        Difficulty::Beginner,
        Settings::default(),
    )
    .expect("launch ops sim");

    for command in ["pwd", "ls", "cat report.txt"] {
        assert!(matches!(
            app.submit_command(command),
            AttemptOutcome::Correct { .. }
        ));
    }

    assert!(app.session().completed());
    assert!(
        app.session()
            .last_command_output()
            .contains("Moonbase report")
    );
}

#[test]
fn app_records_completed_objectives_in_profile() {
    let content = ContentLibrary::bundled_for(Cabinet::ShellMotel).expect("bundled content");
    let mut app = App::launch_cabinet(
        content,
        Cabinet::ShellMotel,
        Difficulty::Beginner,
        Settings::default(),
    )
    .expect("launch escape room");

    assert!(matches!(
        app.submit_command("pwd"),
        AttemptOutcome::Correct { .. }
    ));

    assert!(
        app.profile()
            .cabinet_progress(Cabinet::ShellMotel)
            .expect("shell progress")
            .completed_objectives
            .contains("state-location")
    );
}

#[test]
fn app_persists_profile_progress_when_profile_path_is_configured() {
    let dir = assert_fs::TempDir::new().expect("temp dir");
    let profile_path = dir.path().join("profile.toml");
    let content = ContentLibrary::bundled_for(Cabinet::ShellMotel).expect("bundled content");
    let mut app = App::launch_cabinet_with_profile_path(
        content,
        Cabinet::ShellMotel,
        Difficulty::Beginner,
        Settings::default(),
        profile_path.clone(),
    )
    .expect("launch escape room with profile");

    assert!(matches!(
        app.submit_command("pwd"),
        AttemptOutcome::Correct { .. }
    ));

    let loaded = bashcards::profile::Profile::load_from(&profile_path).expect("profile saved");
    assert!(
        loaded
            .cabinet_progress(Cabinet::ShellMotel)
            .expect("shell progress")
            .completed_objectives
            .contains("state-location")
    );
}
