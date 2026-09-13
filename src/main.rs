use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    about = "Open a Markdown file with PlantUML diagrams in a browser",
    after_help = "Examples:\n  lens\n  lens docs\n  lens stop\n  lens --scope target .hidden/docs    Limit discovery to a visible directory below a hidden parent"
)]
struct Arguments {
    #[arg(value_name = "TARGET")]
    target: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = lens::TargetScope::Repository)]
    scope: lens::TargetScope,

    #[arg(long = "lens-background-service", hide = true)]
    background_service: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
enum Command {
    /// Stop the background Lens service for the current user
    Stop,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    if arguments.background_service {
        lens::run_background_service().await
    } else if let Some(Command::Stop) = arguments.command {
        lens::stop().await
    } else {
        lens::open(arguments.target, arguments.scope).await
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use super::{Arguments, Command};

    #[test]
    fn stop_argument_then_parses_as_stop_command() {
        // Arrange
        let args = ["lens", "stop"];

        // Act
        let parsed = Arguments::try_parse_from(args).expect("arguments should parse");

        // Assert
        assert_eq!(parsed.command, Some(Command::Stop));
        assert_eq!(parsed.target, None);
    }

    #[test]
    fn target_argument_then_parses_as_target_without_command() {
        // Arrange
        let args = ["lens", "docs/spec.md"];

        // Act
        let parsed = Arguments::try_parse_from(args).expect("arguments should parse");

        // Assert
        assert_eq!(parsed.target, Some(PathBuf::from("docs/spec.md")));
        assert_eq!(parsed.command, None);
    }

    #[test]
    fn empty_arguments_then_target_and_command_are_none() {
        // Arrange
        let args = ["lens"];

        // Act
        let parsed = Arguments::try_parse_from(args).expect("arguments should parse");

        // Assert
        assert_eq!(parsed.target, None);
        assert_eq!(parsed.command, None);
    }
}
