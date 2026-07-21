use std::process::Command;

#[allow(dead_code)] // Each variant is constructed by its supported target build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BrowserPlatform {
    Linux,
    MacOs,
    Windows,
}

#[derive(Debug, Eq, PartialEq)]
struct BrowserCommand {
    program: &'static str,
    arguments: Vec<String>,
}

pub(super) fn open_browser(url: &str) -> std::io::Result<()> {
    let command = browser_command(current_browser_platform()?, url);
    Command::new(command.program)
        .args(command.arguments)
        .spawn()
        .map(|_| ())
}

fn browser_command(platform: BrowserPlatform, url: &str) -> BrowserCommand {
    match platform {
        BrowserPlatform::Linux => BrowserCommand {
            program: "xdg-open",
            arguments: vec![url.to_owned()],
        },
        BrowserPlatform::MacOs => BrowserCommand {
            program: "open",
            arguments: vec![url.to_owned()],
        },
        BrowserPlatform::Windows => BrowserCommand {
            program: "cmd",
            arguments: vec![
                "/C".to_owned(),
                "start".to_owned(),
                String::new(),
                url.to_owned(),
            ],
        },
    }
}

#[cfg(target_os = "linux")]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Ok(BrowserPlatform::Linux)
}

#[cfg(target_os = "macos")]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Ok(BrowserPlatform::MacOs)
}

#[cfg(target_os = "windows")]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Ok(BrowserPlatform::Windows)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Lens does not support automatic browser launch on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::{browser_command, BrowserCommand, BrowserPlatform};

    #[test]
    fn supported_browser_platform_then_uses_its_launch_command() {
        // Arrange
        let url = "http://127.0.0.1:4567";

        // Act
        let commands = [
            (
                BrowserPlatform::Linux,
                browser_command(BrowserPlatform::Linux, url),
            ),
            (
                BrowserPlatform::MacOs,
                browser_command(BrowserPlatform::MacOs, url),
            ),
            (
                BrowserPlatform::Windows,
                browser_command(BrowserPlatform::Windows, url),
            ),
        ];

        // Assert
        assert_eq!(
            commands[0].1,
            BrowserCommand {
                program: "xdg-open",
                arguments: vec![url.to_owned()],
            }
        );
        assert_eq!(
            commands[1].1,
            BrowserCommand {
                program: "open",
                arguments: vec![url.to_owned()],
            }
        );
        assert_eq!(
            commands[2].1,
            BrowserCommand {
                program: "cmd",
                arguments: vec![
                    "/C".to_owned(),
                    "start".to_owned(),
                    String::new(),
                    url.to_owned()
                ],
            }
        );
    }
}
