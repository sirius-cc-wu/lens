use std::{
    env,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Mutex, MutexGuard,
    },
    time::Duration,
};

static SERVICE_TESTS: Mutex<()> = Mutex::new(());
static CLI_SERVICE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

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

#[test]
fn stop_command_when_service_running_then_stops_service_and_exits_zero() {
    // Arrange
    let mut service = BackgroundService::start("stop-active-service");
    let mut command = service.command();
    command.arg("stop");

    // Act
    let output = command.output().expect("lens stop command should run");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("output should be UTF-8");
    assert!(stdout.contains("Lens background service stopped."));

    let wait_result = wait_for_child_exit(&mut service.child, Duration::from_secs(3));
    assert!(
        wait_result,
        "background service process should exit after stop"
    );

    // Act
    let second_output = service
        .command()
        .arg("stop")
        .output()
        .expect("subsequent lens stop command should run");

    // Assert
    assert!(second_output.status.success());
    let second_stdout = String::from_utf8(second_output.stdout).expect("output should be UTF-8");
    assert!(second_stdout.contains("No Lens background service is running."));
}

#[test]
fn concurrent_stop_commands_when_service_running_then_all_succeed_and_exit_zero() {
    // Arrange
    let mut service = BackgroundService::start("concurrent-cli-stops");

    // Act
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let mut command = service.command();
            command.arg("stop");
            std::thread::spawn(move || command.output().expect("lens stop command should run"))
        })
        .collect();

    let mut outputs = Vec::new();
    for handle in handles {
        outputs.push(handle.join().expect("thread should join"));
    }

    // Assert
    for output in outputs {
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("output should be UTF-8");
        assert!(
            stdout.contains("Lens background service stopped.")
                || stdout.contains("No Lens background service is running.")
        );
    }

    let wait_result = wait_for_child_exit(&mut service.child, Duration::from_secs(3));
    assert!(
        wait_result,
        "background service process should exit after stop"
    );
}

#[test]
fn stop_command_when_service_not_running_then_reports_inactive_and_exits_zero() {
    // Arrange
    let _guard = SERVICE_TESTS
        .lock()
        .expect("service test lock should be available");
    let runtime_directory = unique_runtime_directory();
    create_private_directory(&runtime_directory);
    let mut command = lens_command();
    command.env("XDG_RUNTIME_DIR", &runtime_directory);
    command.arg("stop");

    // Act
    let output = command.output().expect("lens stop command should run");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("output should be UTF-8");
    assert!(stdout.contains("No Lens background service is running."));

    if runtime_directory.exists() {
        let _ = std::fs::remove_dir_all(&runtime_directory);
    }
}

#[test]
fn stop_command_help_flag_then_describes_stop_command() {
    // Arrange
    let mut command = lens_command();
    command.args(["stop", "--help"]);

    // Act
    let output = command.output().expect("lens stop help should run");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Stop the background Lens service for the current user"));
}

#[cfg(unix)]
#[test]
fn stop_command_when_stale_socket_exists_then_removes_socket_and_exits_zero() {
    // Arrange
    let _guard = SERVICE_TESTS
        .lock()
        .expect("service test lock should be available");
    let runtime_directory = unique_runtime_directory();
    create_private_directory(&runtime_directory);
    let lens_dir = runtime_directory.join("lens");
    create_private_directory(&lens_dir);
    let endpoint = lens_dir.join("service-v1.sock");
    let stale = std::os::unix::net::UnixListener::bind(&endpoint)
        .expect("stale endpoint should be creatable");
    drop(stale);
    assert!(
        endpoint.exists(),
        "stale socket file must exist before test"
    );

    let mut command = lens_command();
    command.env("XDG_RUNTIME_DIR", &runtime_directory);
    command.arg("stop");

    // Act
    let output = command.output().expect("lens stop command should run");

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("output should be UTF-8");
    assert!(stdout.contains("No Lens background service is running."));
    assert!(!endpoint.exists(), "stale socket file must be unlinked");

    if runtime_directory.exists() {
        let _ = std::fs::remove_dir_all(&runtime_directory);
    }
}

fn wait_for_child_exit(child: &mut Child, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn unique_path(name: &str) -> PathBuf {
    env::temp_dir().join(format!("lens-cli-{}-{name}", std::process::id()))
}

fn unique_runtime_directory() -> PathBuf {
    let sequence = CLI_SERVICE_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    env::temp_dir().join(format!("lcr-{}-{sequence}", std::process::id()))
}

struct BackgroundService {
    child: Child,
    runtime_directory: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl BackgroundService {
    fn start(name: &str) -> Self {
        let _ = name;
        let guard = SERVICE_TESTS
            .lock()
            .expect("background service test lock should be available");
        let runtime_directory = unique_runtime_directory();
        if runtime_directory.exists() {
            std::fs::remove_dir_all(&runtime_directory)
                .expect("stale runtime directory should be removable");
        }
        create_private_directory(&runtime_directory);
        #[cfg(unix)]
        {
            let socket_path = runtime_directory.join("lens").join("service-v1.sock");
            assert!(
                socket_path.as_os_str().len() < 104,
                "CLI fixture socket path exceeds Unix sockaddr_un.sun_path capacity: {}",
                socket_path.display()
            );
        }
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
        let stderr = child
            .stderr
            .take()
            .expect("service standard error should be captured");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
            let _ = sender.send(result);
        });
        let stderr_handle = std::thread::spawn(move || {
            let mut err_output = String::new();
            let _ = BufReader::new(stderr).read_to_string(&mut err_output);
            err_output
        });
        let readiness = match receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(line)) => line,
            Ok(Err(err)) => {
                let _ = child.kill();
                let err_output = stderr_handle.join().unwrap_or_default();
                panic!("Lens background service readiness should be readable: {err}; child stderr: {err_output}");
            }
            Err(err) => {
                let _ = child.kill();
                let err_output = stderr_handle.join().unwrap_or_default();
                panic!("Lens background service should report readiness: {err}; child stderr: {err_output}");
            }
        };
        if readiness.trim() != "Lens background service is ready" {
            let _ = child.kill();
            let err_output = stderr_handle.join().unwrap_or_default();
            panic!(
                "Lens background service failed to report ready: observed readiness: {:?}; child stderr: {}",
                readiness.trim(),
                err_output
            );
        }
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

#[cfg(unix)]
#[test]
fn cli_fixture_socket_path_then_fits_within_unix_sun_path_limit() {
    // Arrange
    let deep_temp = PathBuf::from("/var/folders/zz/zyxvpxvq6csfxvn_n0000000000000/T");
    let sequence = 999;
    let pid = 99999;

    // Act
    let directory = deep_temp.join(format!("lcr-{pid}-{sequence}"));
    let socket_path = directory.join("lens").join("service-v1.sock");

    // Assert
    assert!(
        socket_path.as_os_str().len() < 104,
        "simulated macOS CLI fixture socket path must fit within 104 bytes capacity: {}",
        socket_path.display()
    );
}
