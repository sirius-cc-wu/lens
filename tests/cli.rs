use std::{env, path::PathBuf, process::Command};

fn lens_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lens"))
}

#[test]
fn help_flag_then_describes_optional_target() {
    // Arrange
    let mut command = lens_command();
    command.arg("--help");

    // Act
    let output = command.output().expect("Lens help command should run");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Usage: lens [OPTIONS] [TARGET]"));
    assert!(stdout.contains("--renderer <RENDERER>"));
    assert!(stdout.contains("lens .hidden/docs"));
}

#[test]
fn missing_target_then_reports_actionable_error() {
    // Arrange
    let missing_target = unique_path("missing-target.md");
    let mut command = lens_command();
    command.arg(&missing_target);

    // Act
    let output = command.output().expect("Lens command should run");

    // Assert
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("does not exist"));
}

#[test]
fn empty_current_directory_then_reports_no_documents_error() {
    // Arrange
    let directory = unique_path("empty-document-root");
    std::fs::create_dir(&directory).expect("test directory should be creatable");
    let mut command = lens_command();
    command.current_dir(&directory);

    // Act
    let output = command.output().expect("Lens command should run");

    // Assert
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("contains no discoverable Markdown or PlantUML documents"));
    std::fs::remove_dir(directory).expect("test directory should be removable");
}

fn unique_path(name: &str) -> PathBuf {
    env::temp_dir().join(format!("lens-cli-{}-{name}", std::process::id()))
}
