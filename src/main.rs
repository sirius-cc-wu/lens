use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    about = "Open a Markdown file with PlantUML diagrams in a browser",
    after_help = "Examples:\n  lens\n  lens docs\n  lens .agents/skills    View project agent skills in a hidden parent directory"
)]
struct Arguments {
    #[arg(value_name = "TARGET")]
    target: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let target = lens::load_markdown_target(arguments.target.as_deref())?;
    lens::serve(target).await
}
