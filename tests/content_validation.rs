use bashcards::arcade::Cabinet;
use bashcards::content::ContentLibrary;
use bashcards::validation::{ValidationResult, validate_exact};

#[test]
fn parses_chapter_rooms_decks_incidents_from_toml() {
    let content = ContentLibrary::from_toml_str(
        r##"
        [cabinet]
        id = "shell-motel"
        display_name = "Shell Motel"
        tagline = "a hallway asks where you are"
        glyph = "C"
        genre = "escape_room"

        [cabinet.theme]
        accent_primary = "#ff9ad2"
        accent_secondary = "#8af0ff"

        [[chapters]]
        id = "lost-terminal"
        title = "Lost Terminal"

        [[chapters.rooms]]
        id = "shell-motel-101"
        title = "Shell Motel Room 101"
        scene = "The door has no handle. It has a prompt."

        [[chapters.rooms.objectives]]
        id = "state-location"
        prompt = "State your location."
        success = "Objective complete: you found your current location."
        hints = ["pwd prints the current directory."]
        validator = { type = "ExactCommand", expected = "pwd" }

        [[chapters.decks]]
        id = "path-form"
        title = "Path Form"

        [[chapters.decks.cards]]
        id = "pwd-recall"
        prompt = "Print the current directory."
        success = "The scroll nods."
        hints = ["Three letters: pwd."]
        validator = { type = "ExactCommand", expected = "pwd" }

        [[chapters.incidents]]
        id = "moonbase-misfile"
        title = "Moonbase Helpdesk"
        prompt = "Find the misplaced report."
        objective = { id = "inspect-report", prompt = "Read report.txt", success = "Report found.", hints = ["cat reads files."], validator = { type = "ExactCommand", expected = "cat report.txt" } }
        "##,
    )
    .expect("content parses");

    assert_eq!(content.chapters().len(), 1);
    let chapter = &content.chapters()[0];
    assert_eq!(chapter.id, "lost-terminal");
    assert_eq!(chapter.rooms[0].objectives[0].id, "state-location");
    assert_eq!(chapter.decks[0].cards[0].id, "pwd-recall");
    assert_eq!(chapter.incidents[0].id, "moonbase-misfile");
}

#[test]
fn exact_command_validation_trims_whitespace_and_explains_near_miss() {
    assert_eq!(validate_exact("pwd", "  pwd  "), ValidationResult::Pass);
    assert_eq!(
        validate_exact("pwd", "ls"),
        ValidationResult::Fail(
            "You are close. `ls` shows what is here; `pwd` shows where here is.".to_string()
        )
    );
}

#[test]
fn bundled_content_covers_lost_terminal_beginner_commands() {
    let content = ContentLibrary::bundled_for(Cabinet::ShellMotel).expect("bundled content");
    let chapter = &content.chapters()[0];
    let room_commands: Vec<_> = chapter.rooms[0]
        .objectives
        .iter()
        .map(|objective| format!("{:?}", objective.validator))
        .collect();

    assert!(
        room_commands
            .iter()
            .any(|validator| validator.contains("pwd"))
    );
    assert!(
        room_commands
            .iter()
            .any(|validator| validator.contains("ls"))
    );
    assert!(
        room_commands
            .iter()
            .any(|validator| validator.contains("cd lobby"))
    );
    assert!(
        room_commands
            .iter()
            .any(|validator| validator.contains("cat README"))
    );
}
