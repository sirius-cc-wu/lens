use std::{
    env,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{mpsc, Mutex, MutexGuard},
    time::Duration,
};

static SERVICE_TESTS: Mutex<()> = Mutex::new(());

fn lens_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lens"))
}

fn accepts_server_configured_serve<F, Fut>(_serve: F)
where
    F: FnOnce(lens::MarkdownTarget) -> Fut,
{
}

#[test]
fn public_serve_entry_point_then_accepts_only_target() {
    // Arrange
    let serve = lens::serve;

    // Act
    accepts_server_configured_serve(serve);

    // Assert
    // The function bound is the compile-time public API assertion.
}

#[test]
fn help_flag_then_describes_optional_target_without_renderer_selection() {
    // Arrange
    let mut command = lens_command();
    command.arg("--help");

    // Act
    let output = command.output().expect("Lens help command should run");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    let executable_name = Path::new(env!("CARGO_BIN_EXE_lens"))
        .file_name()
        .expect("Lens executable path should have a file name")
        .to_string_lossy();
    assert!(stdout.contains(&format!("Usage: {executable_name} [OPTIONS] [TARGET]")));
    assert!(stdout.contains("--scope <SCOPE>"));
    assert!(stdout.contains("[possible values: repository, target]"));
    assert!(!stdout.contains("--renderer"));
    assert!(!stdout.contains("lens-background-service"));
    assert!(stdout.contains("lens --scope target .hidden/docs"));
}

#[test]
fn renderer_argument_then_reports_unknown_argument() {
    // Arrange
    let missing_target = unique_path("renderer-argument-target.md");
    let mut command = lens_command();
    command.args(["--renderer", "public"]);
    command.arg(missing_target);

    // Act
    let output = command.output().expect("Lens command should run");

    // Assert
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("unexpected argument '--renderer' found"));
    assert!(!stderr.contains("does not exist"));
}

#[test]
fn missing_target_then_reports_actionable_error() {
    // Arrange
    let service = BackgroundService::start("missing-target-service");
    let missing_target = unique_path("missing-target.md");
    let mut command = service.command();
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
    let service = BackgroundService::start("empty-directory-service");
    let directory = unique_path("empty-document-root");
    std::fs::create_dir(&directory).expect("test directory should be creatable");
    let mut command = service.command();
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

struct BackgroundService {
    child: Child,
    runtime_directory: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl BackgroundService {
    fn start(name: &str) -> Self {
        let guard = SERVICE_TESTS
            .lock()
            .expect("background service test lock should be available");
        let runtime_directory = unique_path(name);
        if runtime_directory.exists() {
            std::fs::remove_dir_all(&runtime_directory)
                .expect("stale runtime directory should be removable");
        }
        create_private_directory(&runtime_directory);
        let mut command = lens_command();
        command
            .arg("--lens-background-service")
            .env("XDG_RUNTIME_DIR", &runtime_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .expect("Lens background service should start");
        let stdout = child
            .stdout
            .take()
            .expect("service standard output should be captured");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
            let _ = sender.send(result);
        });
        let readiness = receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("Lens background service should report readiness")
            .expect("Lens background service readiness should be readable");
        assert_eq!(readiness.trim(), "Lens background service is ready");
        Self {
            child,
            runtime_directory,
            _guard: guard,
        }
    }

    fn command(&self) -> Command {
        let mut command = lens_command();
        command.env("XDG_RUNTIME_DIR", &self.runtime_directory);
        command
    }
}

impl Drop for BackgroundService {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if self.runtime_directory.exists() {
            std::fs::remove_dir_all(&self.runtime_directory)
                .expect("runtime directory should be removable");
        }
    }
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::DirBuilderExt;

    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(path)
        .expect("private runtime directory should be creatable");
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    std::fs::create_dir(path).expect("private runtime directory should be creatable");
}
