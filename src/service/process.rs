use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use thiserror::Error;

pub(crate) const SERVICE_ARGUMENT: &str = "--lens-background-service";

#[derive(Debug, Error)]
pub(crate) enum ProcessError {
    #[error("Could not identify the Lens executable: {0}")]
    CurrentExecutable(std::io::Error),
    #[cfg_attr(unix, allow(dead_code))]
    #[error("Could not identify a stable service working directory: {0}")]
    StableWorkingDirectory(std::io::Error),
    #[error("Could not start the Lens background service: {0}")]
    Spawn(std::io::Error),
}

pub(crate) fn spawn_detached_service() -> Result<(), ProcessError> {
    let executable = env::current_exe().map_err(ProcessError::CurrentExecutable)?;
    let working_directory = stable_working_directory()?;
    service_command(&executable, &working_directory)
        .spawn()
        .map(|_| ())
        .map_err(ProcessError::Spawn)
}

fn service_command(executable: &Path, working_directory: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .current_dir(working_directory)
        .arg(SERVICE_ARGUMENT)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    detach(&mut command);
    command
}

#[cfg(unix)]
fn stable_working_directory() -> Result<PathBuf, ProcessError> {
    Ok(PathBuf::from("/"))
}

#[cfg(windows)]
fn stable_working_directory() -> Result<PathBuf, ProcessError> {
    env::var_os("SystemRoot")
        .map(PathBuf::from)
        .or_else(|| env::var_os("windir").map(PathBuf::from))
        .or_else(|| {
            env::var_os("SystemDrive")
                .map(|drive| PathBuf::from(format!("{}\\", drive.to_string_lossy())))
        })
        .ok_or_else(|| {
            ProcessError::StableWorkingDirectory(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine Windows system root",
            ))
        })
}

#[cfg(unix)]
fn detach(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    // SAFETY: this closure calls only the async-signal-safe setsid operation
    // between fork and exec and does not access shared process state.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
}

#[cfg(windows)]
fn detach(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};

    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, path::Path};

    use super::{service_command, stable_working_directory, SERVICE_ARGUMENT};

    #[test]
    fn detached_candidate_then_uses_hidden_service_mode() {
        // Arrange
        let executable = Path::new("lens-test-executable");
        let working_directory =
            stable_working_directory().expect("stable working directory should be available");

        // Act
        let command = service_command(executable, &working_directory);

        // Assert
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec![OsStr::new(SERVICE_ARGUMENT)]
        );
        assert_eq!(command.get_current_dir(), Some(working_directory.as_path()));
    }
}
