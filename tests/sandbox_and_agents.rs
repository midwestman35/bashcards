use bashcards::app::App;
use bashcards::arcade::Cabinet;
use bashcards::content::ContentLibrary;
use bashcards::game::AttemptOutcome;
use bashcards::settings::{Difficulty, Settings};
use bashcards::shell::Sandbox;
use bashcards::shell::{CommandWorld, run_guided};
use bashcards::validation::{ValidationResult, validate_filesystem_path};

#[test]
fn sandbox_fixtures_are_created_and_filesystem_validation_stays_inside_root() {
    let sandbox = Sandbox::create().expect("sandbox");
    sandbox
        .write_fixture("logs/report.txt", "moonbase report")
        .expect("fixture");

    assert!(sandbox.root().join("logs/report.txt").exists());
    assert_eq!(
        validate_filesystem_path(sandbox.root(), "logs/report.txt"),
        ValidationResult::Pass
    );
    assert!(matches!(
        validate_filesystem_path(sandbox.root(), "../outside.txt"),
        ValidationResult::Blocked(_)
    ));
}

#[test]
fn guided_shell_simulates_beginner_commands_without_touching_user_filesystem() {
    let mut world = CommandWorld::lost_terminal();

    let pwd = run_guided(&mut world, "pwd");
    assert_eq!(pwd.stdout.trim(), "/shell-motel/room-101");

    let ls = run_guided(&mut world, "ls");
    assert!(ls.stdout.contains("lobby"));

    let cd = run_guided(&mut world, "cd lobby");
    assert_eq!(cd.stderr, "");
    assert_eq!(world.cwd(), "/shell-motel/room-101/lobby");

    let read = run_guided(&mut world, "cat README");
    assert!(read.stdout.contains("Lobby README"));
}

#[test]
fn simulated_agents_complete_all_three_beginner_modes_end_to_end() {
    let scripts = [
        (
            Cabinet::ShellMotel,
            vec!["pwd", "ls", "cd lobby", "cat README"],
        ),
        (Cabinet::MonasteryOfForms, vec!["pwd", "ls", "cat README"]),
        (
            Cabinet::MidnightCarnival,
            vec!["pwd", "ls", "cat report.txt"],
        ),
    ];

    for (cabinet, commands) in scripts {
        let content = ContentLibrary::bundled_for(cabinet).expect("bundled content");
        let mut app =
            App::launch_cabinet(content, cabinet, Difficulty::Beginner, Settings::default())
                .expect("launch cabinet");

        for command in commands {
            let outcome = app.submit_command(command);
            if app.session().completed() {
                assert!(matches!(outcome, AttemptOutcome::Correct { .. }));
                break;
            }
        }

        assert!(app.session().completed(), "{cabinet:?} did not complete");
    }
}
