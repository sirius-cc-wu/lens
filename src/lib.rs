mod markdown;
mod plantuml;
mod target;
mod viewer;

pub use target::{load_markdown_target, MarkdownTarget};
pub use viewer::serve;
