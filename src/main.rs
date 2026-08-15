use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    about = "Open a Markdown file with PlantUML diagrams in a browser",
    after_help = "Examples:\n  lens\n  lens docs\n  lens stop\n  lens --scope target .hidden/docs    Limit discovery to a visible directory below a hidden parent",
    args_conflicts_with_subcommands = true,
    subcommand_precedence_over_arg = true
)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(value_name = "TARGET")]
    target: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = lens::TargetScope::Repository)]
    scope: lens::TargetScope,

    #[arg(long = "lens-background-service", hide = true)]
    background_service: bool,
}

#[derive(Debug, Eq, PartialEq, Subcommand)]
enum Command {
    /// Stop the current user's Lens background service
    Stop,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    if arguments.background_service {
        lens::run_background_service().await
    } else {
        match arguments.command {
            Some(Command::Stop) => lens::stop().await,
            None => lens::open(arguments.target, arguments.scope).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Arguments, Command};

    #[test]
    fn stop_argument_then_parses_as_command_without_target() {
        // Arrange
        let command = ["lens", "stop"];

        // Act
        let arguments = Arguments::try_parse_from(command).expect("stop command should parse");

        // Assert
        assert_eq!(arguments.command, Some(Command::Stop));
        assert!(arguments.target.is_none());
    }

    #[test]
    fn ordinary_path_argument_then_remains_an_open_target() {
        // Arrange
        let command = ["lens", "docs"];

        // Act
        let arguments = Arguments::try_parse_from(command).expect("target should parse");

        // Assert
        assert!(arguments.command.is_none());
        assert_eq!(
            arguments.target.as_deref(),
            Some(std::path::Path::new("docs"))
        );
    }
}
