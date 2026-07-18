use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Open a Markdown file with PlantUML diagrams in a browser")]
struct Arguments {
    #[arg(value_name = "MARKDOWN_FILE")]
    target: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let target = lens::load_markdown_target(&arguments.target)?;
    lens::serve(target).await
}
