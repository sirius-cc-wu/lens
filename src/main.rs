use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    about = "Open a Markdown file with PlantUML diagrams in a browser",
    after_help = "Examples:\n  lens\n  lens docs\n  lens --scope target .hidden/docs    Limit discovery to a visible directory below a hidden parent"
)]
struct Arguments {
    #[arg(value_name = "TARGET")]
    target: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = lens::TargetScope::Repository)]
    scope: lens::TargetScope,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let target =
        lens::load_markdown_target_with_scope(arguments.target.as_deref(), arguments.scope)?;
    lens::serve(target).await
}
