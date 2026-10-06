use std::{collections::BTreeSet, fmt::Write as _, path::Path};

use pulldown_cmark::{html, CodeBlockKind, Event, LinkType, Options, Parser, Tag, TagEnd};
use serde_yaml::{Mapping, Value};

use crate::source_link::{normalize_relative_link, SourceLinkResolver};

#[derive(Clone, Debug)]
pub struct Diagram {
    pub source: String,
}

#[derive(Debug)]
pub struct RenderedDocument {
    pub html: String,
    pub diagrams: Vec<Diagram>,
}

enum InterceptedBlock {
    PlantUml(String),
    Mermaid(String),
    Math(String),
}

impl InterceptedBlock {
    fn buffer_mut(&mut self) -> &mut String {
        match self {
            Self::PlantUml(source) | Self::Mermaid(source) | Self::Math(source) => source,
        }
    }
}

const LENS_OPTIONS: Options = Options::ENABLE_TABLES
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_SMART_PUNCTUATION)
    .union(Options::ENABLE_HEADING_ATTRIBUTES)
    .union(Options::ENABLE_MATH);

pub fn render(
    markdown: &str,
    document_id: usize,
    current_document: &str,
    current_document_path: &Path,
    known_documents: &BTreeSet<String>,
    source_links: &SourceLinkResolver,
) -> RenderedDocument {
    let frontmatter = frontmatter(markdown);
    let parser = Parser::new_ext(frontmatter.body, LENS_OPTIONS);
    let mut events = Vec::new();
    let mut diagrams = Vec::new();
    let mut intercepted_block: Option<InterceptedBlock> = None;
    let mut source_link_stack = Vec::new();
    let mut image_depth: usize = 0;

    for event in parser {
        if let Some(block) = intercepted_block.as_mut() {
            match event {
                Event::End(TagEnd::CodeBlock) => match intercepted_block.take().unwrap() {
                    InterceptedBlock::PlantUml(source) => {
                        let diagram_id = diagrams.len();
                        diagrams.push(Diagram {
                            source: source.clone(),
                        });
                        events.push(Event::Html(
                            diagram_placeholder(document_id, diagram_id, &source).into(),
                        ));
                    }
                    InterceptedBlock::Mermaid(source) => {
                        events.push(Event::Html(mermaid_placeholder(&source).into()));
                    }
                    InterceptedBlock::Math(source) => {
                        events.push(Event::Html(math_block_placeholder(&source).into()));
                    }
                },
                Event::Text(text) | Event::Code(text) => block.buffer_mut().push_str(&text),
                Event::SoftBreak | Event::HardBreak => block.buffer_mut().push('\n'),
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language))) => {
                let tag = language.trim();
                if tag.eq_ignore_ascii_case("plantuml") {
                    intercepted_block = Some(InterceptedBlock::PlantUml(String::new()));
                } else if tag.eq_ignore_ascii_case("mermaid") {
                    intercepted_block = Some(InterceptedBlock::Mermaid(String::new()));
                } else if tag.eq_ignore_ascii_case("math") {
                    intercepted_block = Some(InterceptedBlock::Math(String::new()));
                } else {
                    events.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(
                        language,
                    ))));
                }
            }
            Event::Start(Tag::Image { .. }) => {
                image_depth += 1;
                events.push(event);
            }
            Event::End(TagEnd::Image) => {
                image_depth = image_depth.saturating_sub(1);
                events.push(event);
            }
            Event::InlineMath(tex) => events.push(if image_depth > 0 {
                Event::Text(format!("${tex}$").into())
            } else {
                Event::Html(
                    format!(
                        r#"<span class="math-inline" data-math-inline>{}</span>"#,
                        escape_html(&tex)
                    )
                    .into(),
                )
            }),
            Event::DisplayMath(tex) => events.push(if image_depth > 0 {
                Event::Text(format!("$${tex}$$").into())
            } else {
                Event::Html(
                    format!(
                        r#"<span class="math-display" data-math-display>{}</span>"#,
                        escape_html(&tex)
                    )
                    .into(),
                )
            }),
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let resolved = if link_type == LinkType::Email {
                    ResolvedLink {
                        destination: dest_url.to_string(),
                        opens_in_vscode: false,
                    }
                } else {
                    resolve_link(
                        &dest_url,
                        current_document,
                        current_document_path,
                        known_documents,
                        source_links,
                    )
                };
                source_link_stack.push(resolved.opens_in_vscode);
                events.push(Event::Start(Tag::Link {
                    link_type,
                    dest_url: resolved.destination.into(),
                    title,
                    id,
                }));
            }
            Event::End(TagEnd::Link) => {
                if source_link_stack.pop().unwrap_or(false) {
                    events.push(Event::Html(
                        r#"<span class="source-link-indicator"> (opens in VS Code)</span>"#.into(),
                    ));
                }
                events.push(Event::End(TagEnd::Link));
            }
            Event::Start(Tag::Table(alignments)) => {
                let width_class = if alignments.len() >= 4 {
                    " markdown-table-wide"
                } else {
                    ""
                };
                events.push(Event::Html(
                    format!(r#"<div class="markdown-table{width_class}" tabindex="0">"#).into(),
                ));
                events.push(Event::Start(Tag::Table(alignments)));
            }
            Event::End(TagEnd::Table) => {
                events.push(Event::End(TagEnd::Table));
                events.push(Event::Html("</div>".into()));
            }
            Event::Start(Tag::Heading {
                level,
                id,
                classes,
                attrs: _,
            }) => {
                events.push(Event::Start(Tag::Heading {
                    level,
                    id,
                    classes,
                    attrs: Vec::new(),
                }));
            }
            Event::Html(value) | Event::InlineHtml(value) => events.push(Event::Text(value)),
            event => events.push(event),
        }
    }

    let mut html = frontmatter.html;
    html::push_html(&mut html, events.into_iter());
    RenderedDocument { html, diagrams }
}

struct Frontmatter<'a> {
    body: &'a str,
    html: String,
}

fn frontmatter(markdown: &str) -> Frontmatter<'_> {
    let Some((opening_line, opening_end)) = next_line(markdown, 0) else {
        return Frontmatter {
            body: markdown,
            html: String::new(),
        };
    };
    if opening_line != "---" {
        return Frontmatter {
            body: markdown,
            html: String::new(),
        };
    }

    let mut position = opening_end;
    while let Some((line, line_end)) = next_line(markdown, position) {
        if matches!(line, "---" | "...") {
            let source = &markdown[opening_end..position];
            let body = &markdown[line_end..];
            return parsed_frontmatter(source, body);
        }
        position = line_end;
    }

    Frontmatter {
        body: markdown,
        html: frontmatter_error("A closing `---` or `...` delimiter is required."),
    }
}

fn next_line(source: &str, start: usize) -> Option<(&str, usize)> {
    (start < source.len()).then(|| {
        let remaining = &source[start..];
        let content_end = remaining.find('\n').unwrap_or(remaining.len());
        let line_end = start + content_end;
        let line = source[start..line_end]
            .strip_suffix('\r')
            .unwrap_or(&source[start..line_end]);
        let next_start = if line_end < source.len() {
            line_end + 1
        } else {
            line_end
        };
        (line, next_start)
    })
}

fn parsed_frontmatter<'a>(source: &str, body: &'a str) -> Frontmatter<'a> {
    let html = match serde_yaml::from_str::<Value>(source) {
        Ok(Value::Mapping(metadata)) => metadata_html(&metadata),
        Ok(Value::Null) => empty_metadata_html(),
        Ok(_) => frontmatter_error("YAML frontmatter must be a mapping of metadata fields."),
        Err(error) => frontmatter_error(&error.to_string()),
    };
    Frontmatter { body, html }
}

fn metadata_html(metadata: &Mapping) -> String {
    let mut html = String::from(
        r#"<section class="document-metadata" aria-label="Document metadata"><table><caption>Document metadata</caption><tbody>"#,
    );
    render_metadata_table(metadata, &mut html);
    html.push_str("</tbody></table></section>");
    html
}

fn empty_metadata_html() -> String {
    r#"<section class="document-metadata" aria-label="Document metadata"><table><caption>Document metadata</caption><tbody><tr><td class="document-metadata-empty" colspan="4">No metadata fields were supplied.</td></tr></tbody></table></section>"#.to_owned()
}

fn render_metadata_table(metadata: &Mapping, html: &mut String) {
    let mut fields = metadata.iter();
    while let Some((key, value)) = fields.next() {
        html.push_str("<tr>");
        let second_field = fields.next();
        render_metadata_table_field(key, value, second_field.is_none(), html);
        if let Some((second_key, second_value)) = second_field {
            render_metadata_table_field(second_key, second_value, false, html);
        }
        html.push_str("</tr>");
    }
}

fn render_metadata_table_field(
    key: &Value,
    value: &Value,
    spans_remaining_columns: bool,
    html: &mut String,
) {
    write!(
        html,
        "<th scope=\"row\">{}</th><td{}>",
        escape_html(&metadata_key(key)),
        if spans_remaining_columns {
            " colspan=\"3\""
        } else {
            ""
        },
    )
    .expect("writing metadata markup to a string cannot fail");
    render_metadata_value(value, html);
    html.push_str("</td>");
}

fn render_metadata_mapping(metadata: &Mapping, html: &mut String) {
    for (key, value) in metadata {
        write!(
            html,
            "<div><dt>{}</dt><dd>",
            escape_html(&metadata_key(key))
        )
        .expect("writing metadata markup to a string cannot fail");
        render_metadata_value(value, html);
        html.push_str("</dd></div>");
    }
}

fn metadata_key(key: &Value) -> String {
    match key {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Sequence(_) | Value::Mapping(_) => "complex key".to_owned(),
        Value::Tagged(tagged) => metadata_key(&tagged.value),
    }
}

fn render_metadata_value(value: &Value, html: &mut String) {
    match value {
        Value::Null => html.push_str("None"),
        Value::Bool(value) => html.push_str(&value.to_string()),
        Value::Number(value) => html.push_str(&value.to_string()),
        Value::String(value) => html.push_str(&escape_html(value)),
        Value::Sequence(values) => {
            html.push_str("<ul>");
            for value in values {
                html.push_str("<li>");
                render_metadata_value(value, html);
                html.push_str("</li>");
            }
            html.push_str("</ul>");
        }
        Value::Mapping(values) => {
            html.push_str("<dl>");
            render_metadata_mapping(values, html);
            html.push_str("</dl>");
        }
        Value::Tagged(tagged) => render_metadata_value(&tagged.value, html),
    }
}

fn frontmatter_error(problem: &str) -> String {
    format!(
        r#"<aside class="frontmatter-error" role="alert"><p><strong>Could not parse YAML frontmatter.</strong></p><p>{}</p><p>Fix the YAML between the opening and closing delimiters.</p></aside>"#,
        escape_html(problem),
    )
}

pub fn render_standalone_plantuml(document_id: usize, source: &str) -> RenderedDocument {
    RenderedDocument {
        html: format!(
            r#"<p class="standalone-plantuml">Standalone PlantUML file.</p>{}"#,
            diagram_placeholder(document_id, 0, source)
        ),
        diagrams: vec![Diagram {
            source: source.to_owned(),
        }],
    }
}

fn diagram_placeholder(document_id: usize, diagram_id: usize, source: &str) -> String {
    format!(
        r#"<figure class="diagram" data-diagram-container><img src="/diagrams/{document_id}/{diagram_id}" alt="Rendered PlantUML diagram" data-diagram><p class="diagram-error" hidden>PlantUML rendering failed. The source is shown below.</p><button type="button" data-diagram-retry hidden>Retry diagram rendering</button><details class="diagram-source"><summary>PlantUML source</summary><pre><code>{}</code></pre></details></figure>"#,
        escape_html(source),
    )
}

fn mermaid_placeholder(source: &str) -> String {
    format!(
        r#"<figure class="diagram mermaid-diagram" data-mermaid-container><div class="mermaid-target"></div><a class="diagram-open-link" data-mermaid-open target="_blank" rel="noopener noreferrer" hidden>Open SVG</a><p class="diagram-error" hidden>Mermaid rendering failed. The source is shown below.</p><details class="diagram-source"><summary>Mermaid source</summary><pre><code>{}</code></pre></details></figure>"#,
        escape_html(source),
    )
}

fn math_block_placeholder(source: &str) -> String {
    format!(
        r#"<div class="math-block" data-math-block><div class="math-target"></div><p class="math-error" hidden>Formula rendering failed. The source is shown below.</p><details class="math-source"><summary>Formula source</summary><pre><code>{}</code></pre></details></div>"#,
        escape_html(source),
    )
}

struct ResolvedLink {
    destination: String,
    opens_in_vscode: bool,
}

enum SourceLocation {
    File,
    Line(u32),
}

fn resolve_link(
    destination: &str,
    current_document: &str,
    current_document_path: &Path,
    known_documents: &BTreeSet<String>,
    source_links: &SourceLinkResolver,
) -> ResolvedLink {
    let (path, suffix) = split_link_suffix(destination);
    if path.is_empty() {
        return ResolvedLink {
            destination: destination.to_owned(),
            opens_in_vscode: false,
        };
    }

    if let Some(candidate) = normalize_relative_link(current_document, path) {
        if known_documents.contains(&candidate) {
            return ResolvedLink {
                destination: format!("/documents/{candidate}{suffix}"),
                opens_in_vscode: false,
            };
        }
    }

    let Some(source_location) = source_location(suffix) else {
        return ResolvedLink {
            destination: destination.to_owned(),
            opens_in_vscode: false,
        };
    };

    match source_links.resolve(current_document_path, path) {
        Some(destination) => ResolvedLink {
            destination: match source_location {
                SourceLocation::File => destination,
                SourceLocation::Line(line) => format!("{destination}:{line}:1"),
            },
            opens_in_vscode: true,
        },
        None => ResolvedLink {
            destination: destination.to_owned(),
            opens_in_vscode: false,
        },
    }
}

fn source_location(suffix: &str) -> Option<SourceLocation> {
    if suffix.is_empty() {
        return Some(SourceLocation::File);
    }

    suffix
        .strip_prefix("#L")?
        .parse::<u32>()
        .ok()
        .filter(|line| *line > 0)
        .map(SourceLocation::Line)
}

fn split_link_suffix(destination: &str) -> (&str, &str) {
    let suffix_start = destination
        .char_indices()
        .find_map(|(index, character)| matches!(character, '#' | '?').then_some(index));
    match suffix_start {
        Some(index) => destination.split_at(index),
        None => (destination, ""),
    }
}

pub fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs, path::PathBuf};

    use super::{render, render_standalone_plantuml, RenderedDocument};
    use crate::source_link::SourceLinkResolver;

    fn render_test(
        markdown: &str,
        document_id: usize,
        current_document: &str,
        known_documents: &BTreeSet<String>,
    ) -> RenderedDocument {
        let document_root = std::env::current_dir().expect("test root should be available");
        let current_document_path = document_root.join(current_document);
        let source_links = SourceLinkResolver::new(document_root);
        render(
            markdown,
            document_id,
            current_document,
            &current_document_path,
            known_documents,
            &source_links,
        )
    }

    #[test]
    fn plantuml_block_then_adds_document_scoped_diagram_endpoint() {
        // Arrange
        let markdown = "```plantuml\n@startuml\nAlice -> Bob: hello\n@enduml\n```";

        // Act
        let document = render_test(markdown, 3, "guides/intro.md", &BTreeSet::new());

        // Assert
        assert_eq!(document.diagrams.len(), 1);
        assert!(document.html.contains("src=\"/diagrams/3/0\""));
        assert_eq!(
            document.diagrams[0].source,
            "@startuml\nAlice -> Bob: hello\n@enduml\n"
        );
    }

    #[test]
    fn plantuml_block_then_omits_rendering_disable_status() {
        // Arrange
        let markdown = "```plantuml\n@startuml\nAlice -> Bob: server\n@enduml\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("src=\"/diagrams/0/0\""));
        assert!(!document.html.contains("data-diagram-disabled"));
        assert!(!document
            .html
            .contains("PlantUML rendering is disabled for this viewing session."));
    }

    #[test]
    fn standalone_plantuml_source_then_renders_a_document_scoped_diagram() {
        // Arrange
        let source = "@startuml\nAlice -> Bob: standalone\n@enduml";

        // Act
        let document = render_standalone_plantuml(2, source);

        // Assert
        assert_eq!(document.diagrams.len(), 1);
        assert_eq!(document.diagrams[0].source, source);
        assert!(document.html.contains("Standalone PlantUML file."));
        assert!(document.html.contains("src=\"/diagrams/2/0\""));
    }

    #[test]
    fn known_relative_markdown_link_then_targets_document_route() {
        // Arrange
        let markdown = "[Read the overview](../README.md#install)";
        let known_documents =
            BTreeSet::from(["README.md".to_owned(), "guides/intro.md".to_owned()]);

        // Act
        let document = render_test(markdown, 0, "guides/intro.md", &known_documents);

        // Assert
        assert!(document
            .html
            .contains("href=\"/documents/README.md#install\""));
        assert!(!document.html.contains("source-link-indicator"));
    }

    #[test]
    fn known_relative_plantuml_link_then_targets_document_route() {
        // Arrange
        let markdown = "[Architecture](../architecture.puml)";
        let known_documents =
            BTreeSet::from(["architecture.puml".to_owned(), "guides/intro.md".to_owned()]);

        // Act
        let document = render_test(markdown, 0, "guides/intro.md", &known_documents);

        // Assert
        assert!(document
            .html
            .contains("href=\"/documents/architecture.puml\""));
        assert!(!document.html.contains("source-link-indicator"));
    }

    #[test]
    fn source_link_with_line_fragment_then_opens_vscode_at_that_line() {
        // Arrange
        let root = temporary_source_link_root("suffix");
        let document_path = root.join("docs/design.md");
        let source_path = root.join("src/lib.rs");
        fs::create_dir_all(source_path.parent().expect("source should have a parent"))
            .expect("source directory should be created");
        fs::create_dir_all(
            document_path
                .parent()
                .expect("document should have a parent"),
        )
        .expect("document directory should be created");
        fs::write(&document_path, "# Design").expect("document should be created");
        fs::write(&source_path, "source").expect("source should be created");
        let document_path = fs::canonicalize(document_path).expect("document should canonicalize");
        let source_links =
            SourceLinkResolver::new(fs::canonicalize(&root).expect("root should canonicalize"));

        // Act
        let document = render(
            "[Source](../src/lib.rs#L10)",
            0,
            "docs/design.md",
            &document_path,
            &BTreeSet::from(["docs/design.md".to_owned()]),
            &source_links,
        );

        // Assert
        assert!(document.html.contains("href=\"vscode://file/"));
        assert!(document.html.contains("/src/lib.rs:10:1\""));
        assert!(document
            .html
            .contains(r#"<span class="source-link-indicator"> (opens in VS Code)</span>"#));
        assert!(!document.html.contains("#L10"));
        fs::remove_dir_all(root).expect("source-link fixture should be removable");
    }

    #[test]
    fn source_link_with_invalid_line_fragment_then_preserves_authored_destination() {
        // Arrange
        let root = temporary_source_link_root("invalid-line");
        let document_path = root.join("docs/design.md");
        let source_path = root.join("src/lib.rs");
        fs::create_dir_all(source_path.parent().expect("source should have a parent"))
            .expect("source directory should be created");
        fs::create_dir_all(
            document_path
                .parent()
                .expect("document should have a parent"),
        )
        .expect("document directory should be created");
        fs::write(&document_path, "# Design").expect("document should be created");
        fs::write(&source_path, "source").expect("source should be created");
        let document_path = fs::canonicalize(document_path).expect("document should canonicalize");
        let source_links =
            SourceLinkResolver::new(fs::canonicalize(&root).expect("root should canonicalize"));

        // Act
        let document = render(
            "[Zero](../src/lib.rs#L0) [Malformed](../src/lib.rs#Lx)",
            0,
            "docs/design.md",
            &document_path,
            &BTreeSet::from(["docs/design.md".to_owned()]),
            &source_links,
        );

        // Assert
        assert!(document.html.contains("href=\"../src/lib.rs#L0\""));
        assert!(document.html.contains("href=\"../src/lib.rs#Lx\""));
        assert!(!document.html.contains("source-link-indicator"));
        fs::remove_dir_all(root).expect("source-link fixture should be removable");
    }

    #[test]
    fn email_autolink_with_colliding_source_file_then_preserves_email_destination() {
        // Arrange
        let root = temporary_source_link_root("email");
        let document_path = root.join("docs/design.md");
        let colliding_source_path = root.join("docs/team@example.com");
        fs::create_dir_all(
            document_path
                .parent()
                .expect("document should have a parent"),
        )
        .expect("document directory should be created");
        fs::write(&document_path, "# Design").expect("document should be created");
        fs::write(colliding_source_path, "source").expect("colliding source should be created");
        let document_path = fs::canonicalize(document_path).expect("document should canonicalize");
        let source_links =
            SourceLinkResolver::new(fs::canonicalize(&root).expect("root should canonicalize"));

        // Act
        let document = render(
            "<team@example.com>",
            0,
            "docs/design.md",
            &document_path,
            &BTreeSet::from(["docs/design.md".to_owned()]),
            &source_links,
        );

        // Assert
        assert!(document.html.contains("href=\"mailto:team@example.com\""));
        assert!(!document.html.contains("vscode://"));
        assert!(!document.html.contains("source-link-indicator"));
        fs::remove_dir_all(root).expect("source-link fixture should be removable");
    }

    #[test]
    fn unknown_or_external_link_then_preserves_original_destination() {
        // Arrange
        let markdown = "[Unknown](../../secret.md) [External](https://example.com/guide.md)";
        let known_documents = BTreeSet::from(["guides/intro.md".to_owned()]);

        // Act
        let document = render_test(markdown, 0, "guides/intro.md", &known_documents);

        // Assert
        assert!(document.html.contains("href=\"../../secret.md\""));
        assert!(document
            .html
            .contains("href=\"https://example.com/guide.md\""));
    }

    #[test]
    fn other_fenced_block_then_remains_code_block() {
        // Arrange
        let markdown = "```rust\nlet answer = 42;\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.diagrams.is_empty());
        assert!(document.html.contains("language-rust"));
        assert!(document.html.contains("let answer = 42;"));
    }

    #[test]
    fn raw_html_then_is_escaped() {
        // Arrange
        let markdown = "<script>alert('unsafe')</script>";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<script>"));
        assert!(document.html.contains("&lt;script&gt;"));
    }

    #[test]
    fn valid_frontmatter_then_renders_nested_metadata_before_markdown_body() {
        // Arrange
        let markdown = "---\ntitle: Lens guide\nauthor: Ada\ntags:\n  - rust\n  - docs\npublication:\n  audience: maintainers\n---\n# Guide\n\nBody text.";

        // Act
        let document = render_test(markdown, 0, "guide.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("class=\"document-metadata\""));
        assert!(document
            .html
            .contains("<caption>Document metadata</caption>"));
        assert!(document
            .html
            .contains("<th scope=\"row\">title</th><td>Lens guide</td>"));
        assert!(document.html.contains("<li>rust</li>"));
        assert!(document.html.contains(">audience</dt><dd>maintainers</dd>"));
        assert!(document.html.contains("<h1>Guide</h1>"));
        assert!(!document.html.contains("<hr"));
    }

    #[test]
    fn alternate_frontmatter_delimiter_then_excludes_metadata_from_markdown_body() {
        // Arrange
        let markdown = "---\ntitle: Alternate delimiter\n...\n# Guide";

        // Act
        let document = render_test(markdown, 0, "guide.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("<th scope=\"row\">title</th><td colspan=\"3\">Alternate delimiter</td>"));
        assert!(document.html.contains("<h1>Guide</h1>"));
        assert!(!document.html.contains("<p>title: Alternate delimiter</p>"));
    }

    #[test]
    fn malformed_frontmatter_then_shows_actionable_error_and_renders_body() {
        // Arrange
        let markdown = "---\ntitle: [missing bracket\n---\n# Guide";

        // Act
        let document = render_test(markdown, 0, "guide.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("class=\"frontmatter-error\""));
        assert!(document.html.contains("Could not parse YAML frontmatter."));
        assert!(document
            .html
            .contains("Fix the YAML between the opening and closing delimiters."));
        assert!(document.html.contains("<h1>Guide</h1>"));
    }

    #[test]
    fn unknown_nested_frontmatter_value_then_escapes_its_html() {
        // Arrange
        let markdown = "---\ncustom:\n  note: <unsafe>\n---\n# Guide";

        // Act
        let document = render_test(markdown, 0, "guide.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("<th scope=\"row\">custom</th><td colspan=\"3\"><dl>"));
        assert!(document.html.contains("&lt;unsafe&gt;"));
        assert!(!document.html.contains("<unsafe>"));
    }

    #[test]
    fn unclosed_frontmatter_then_explains_delimiter_and_preserves_document_source() {
        // Arrange
        let markdown = "---\ntitle: Unclosed metadata\n# Guide";

        // Act
        let document = render_test(markdown, 0, "guide.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("A closing `---` or `...` delimiter is required."));
        assert!(document.html.contains("title: Unclosed metadata"));
        assert!(document.html.contains("<h1>Guide</h1>"));
    }

    #[test]
    fn plantuml_source_with_html_then_escapes_source_in_fallback() {
        // Arrange
        let markdown = "```plantuml\nAlice -> Bob: <unsafe>\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("&lt;unsafe&gt;"));
    }

    #[test]
    fn mermaid_block_then_emits_mermaid_placeholder_and_no_server_diagram() {
        // Arrange
        let markdown = "```mermaid\ngraph TD;\nA-->B;\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert_eq!(document.diagrams.len(), 0);
        assert!(document.html.contains(r#"class="diagram mermaid-diagram""#));
        assert!(document.html.contains(r#"data-mermaid-container"#));
        assert!(document
            .html
            .contains(r#"<div class="mermaid-target"></div>"#));
        assert!(document
            .html
            .contains(r#"<p class="diagram-error" hidden>Mermaid rendering failed. The source is shown below.</p>"#));
        assert!(document
            .html
            .contains(r#"<details class="diagram-source"><summary>Mermaid source</summary><pre><code>graph TD;"#));
        assert!(document.html.contains("A--&gt;B;"));
    }

    #[test]
    fn mermaid_block_with_mixed_case_language_then_emits_mermaid_placeholder() {
        // Arrange
        let markdown = "```MeRmAiD\nflowchart LR\nStart --> Stop\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(r#"class="diagram mermaid-diagram""#));
        assert!(document.html.contains("flowchart LR"));
    }

    #[test]
    fn mermaid_source_with_html_then_escapes_source_in_details() {
        // Arrange
        let markdown = "```mermaid\ngraph TD;\nA[<div id=\"danger\">] --> B;\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("&lt;div id=&quot;danger&quot;&gt;"));
        assert!(!document.html.contains("<div id=\"danger\">"));
    }

    #[test]
    fn mixed_plantuml_and_mermaid_document_then_emits_both_diagram_types() {
        // Arrange
        let markdown = "# System\n\n```plantuml\n@startuml\nnode Server\n@enduml\n```\n\n```mermaid\nsequenceDiagram\nClient->>Server: ping\n```";

        // Act
        let document = render_test(markdown, 1, "doc.md", &BTreeSet::new());

        // Assert
        assert_eq!(document.diagrams.len(), 1);
        assert!(document.html.contains(r#"src="/diagrams/1/0""#));
        assert!(document.html.contains(r#"class="diagram mermaid-diagram""#));
        assert!(document.html.contains("Client-&gt;&gt;Server: ping"));
    }

    #[test]
    fn mermaid_block_then_emits_open_svg_link_in_placeholder() {
        // Arrange
        let markdown = "```mermaid\ngraph TD;\nA-->B;\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            r#"<a class="diagram-open-link" data-mermaid-open target="_blank" rel="noopener noreferrer" hidden>Open SVG</a>"#
        ));
    }

    #[test]
    fn mixed_plantuml_and_mermaid_document_then_emits_open_svg_only_on_mermaid() {
        // Arrange
        let markdown = "# System\n\n```plantuml\n@startuml\nnode Server\n@enduml\n```\n\n```mermaid\nsequenceDiagram\nClient->>Server: ping\n```";

        // Act
        let document = render_test(markdown, 1, "doc.md", &BTreeSet::new());

        // Assert
        assert_eq!(document.diagrams.len(), 1);
        assert!(document.html.contains(r#"src="/diagrams/1/0""#));
        assert!(document.html.contains(r#"data-diagram"#));
        assert!(document.html.contains(
            r#"<a class="diagram-open-link" data-mermaid-open target="_blank" rel="noopener noreferrer" hidden>Open SVG</a>"#
        ));
        assert_eq!(document.html.matches("diagram-open-link").count(), 1);
        assert_eq!(document.html.matches("data-mermaid-open").count(), 1);
    }

    #[test]
    fn raw_inline_html_and_html_block_then_escape_verbatim() {
        // Arrange
        let markdown = "text <img src=x onerror=alert(1)> text\n\n<div onclick=\"x\">hi</div>";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<img"));
        assert!(document.html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(!document.html.contains("<div onclick"));
        assert!(document
            .html
            .contains("&lt;div onclick=\"x\"&gt;hi&lt;/div&gt;"));
    }

    #[test]
    fn inline_raw_html_then_is_escaped() {
        // Arrange
        let markdown = "text <img src=x onerror=alert(1)> text";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<img"));
        assert!(document.html.contains("&lt;img src=x onerror=alert(1)&gt;"));
    }

    #[test]
    fn block_raw_html_with_event_handler_then_is_escaped() {
        // Arrange
        let markdown = "<div onclick=\"x\">hi</div>";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<div onclick"));
        assert!(document
            .html
            .contains("&lt;div onclick=\"x\"&gt;hi&lt;/div&gt;"));
    }

    #[test]
    fn raw_script_tag_in_markdown_then_is_escaped() {
        // Arrange
        let markdown = "<script>alert('unsafe')</script>";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<script>"));
        assert!(document
            .html
            .contains("&lt;script&gt;alert('unsafe')&lt;/script&gt;"));
    }

    #[test]
    fn forged_math_span_in_raw_markdown_then_is_escaped() {
        // Arrange
        let markdown = "Inline <span data-math-inline>x</span> math";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<span data-math-inline>"));
        assert!(document
            .html
            .contains("&lt;span data-math-inline&gt;x&lt;/span&gt;"));
    }

    #[test]
    fn heading_attribute_block_with_custom_attributes_then_keeps_only_id_and_classes() {
        // Arrange
        let markdown =
            "# Title {onclick=alert(1) style=position:fixed data-diagram href=/documents/a.md}";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("onclick"));
        assert!(!document.html.contains("style"));
        assert!(!document.html.contains("data-diagram"));
        assert!(!document.html.contains("href"));
        assert!(document.html.contains("<h1>Title</h1>"));
    }

    #[test]
    fn subscript_and_superscript_markers_then_render_literally() {
        // Arrange
        let markdown = "~sub~ and ^sup^ and H~2~O, x^2^";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<sub>"));
        assert!(!document.html.contains("<sup>"));
        assert!(document.html.contains("<del>sub</del>"));
        assert!(document.html.contains("^sup^ and H~2~O, x^2^"));
    }

    #[test]
    fn wikilink_syntax_then_renders_literally() {
        // Arrange
        let markdown = "[[Page]]";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<a"));
        assert!(document.html.contains("[[Page]]"));
    }

    #[test]
    fn definition_list_syntax_then_renders_no_definition_list() {
        // Arrange
        let markdown = "Term\n: definition";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<dl>"));
        assert!(!document.html.contains("<dt>"));
        assert!(!document.html.contains("<dd>"));
    }

    #[test]
    fn gfm_alert_marker_then_renders_plain_blockquote() {
        // Arrange
        let markdown = "> [!NOTE]\n> text";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("markdown-alert"));
        assert!(document.html.contains("<blockquote>"));
    }

    #[test]
    fn metadata_blocks_plus_and_minus_then_render_literally() {
        // Arrange
        let plus_markdown = "Body\n\n+++\ntitle = \"test\"\n+++\n\nMore";
        let minus_markdown = "Body\n\n---\ntitle: test\n---\n\nMore";

        // Act
        let plus_doc = render_test(plus_markdown, 0, "document.md", &BTreeSet::new());
        let minus_doc = render_test(minus_markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(plus_doc.html.contains("+++"));
        assert!(plus_doc.html.contains("title = “test”"));
        assert!(!minus_doc.html.contains("document-metadata"));
        assert!(minus_doc.html.contains("title: test"));
        assert!(minus_doc.html.contains("<hr"));
    }

    #[test]
    fn defined_footnote_then_renders_reference_and_definition() {
        // Arrange
        let markdown = "Note reference[^1].\n\n[^1]: Note content.";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("<sup class=\"footnote-reference\"><a href=\"#1\">1</a></sup>"));
        assert!(document
            .html
            .contains("<div class=\"footnote-definition\" id=\"1\"><sup class=\"footnote-definition-label\">1</sup>"));
        assert!(document.html.contains("<p>Note content.</p>"));
    }

    #[test]
    fn undefined_footnote_reference_then_renders_literal_text() {
        // Arrange
        let markdown = "Here is an [^undefined] reference.";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<sup class=\"footnote-reference\">"));
        assert!(document
            .html
            .contains("<p>Here is an [^undefined] reference.</p>"));
    }

    #[test]
    fn consecutive_footnote_definitions_then_render_as_separate_items() {
        // Arrange
        let markdown = "Two notes[^a][^b].\n\n[^a]: First note\n[^b]: Second note";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("<div class=\"footnote-definition\" id=\"a\">"));
        assert!(document
            .html
            .contains("<div class=\"footnote-definition\" id=\"b\">"));
        assert!(document.html.contains("<p>First note</p>"));
        assert!(document.html.contains("<p>Second note</p>"));
    }

    #[test]
    fn indented_footnote_continuation_then_stays_inside_footnote() {
        // Arrange
        let markdown = "A note[^1].\n\n[^1]: First paragraph.\n\n    Continuation paragraph.\n\nAfter footnote.";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            "<div class=\"footnote-definition\" id=\"1\"><sup class=\"footnote-definition-label\">1</sup>\n<p>First paragraph.</p>\n<p>Continuation paragraph.</p>\n</div>"
        ));
        assert!(document.html.contains("<p>After footnote.</p>"));
    }

    #[test]
    fn heading_attribute_block_with_valid_id_and_classes_then_preserves_them() {
        // Arrange
        let markdown = "# Title {#intro .lead}";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("<h1 id=\"intro\" class=\"lead\">Title</h1>"));
    }

    #[test]
    fn html_comment_with_math_then_renders_escaped_comment_text() {
        // Arrange
        let markdown = "<!-- $x$ -->";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<!--"));
        assert!(document.html.contains("&lt;!-- $x$ --&gt;"));
    }

    #[test]
    fn strikethrough_tasklist_and_smart_quotes_then_render_as_before() {
        // Arrange
        let markdown = "~~strike~~\n\n- [ ] task\n\n\"quoted\"";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("<del>strike</del>"));
        assert!(document.html.contains(r#"type="checkbox""#));
        assert!(document.html.contains("“quoted”"));
    }

    #[test]
    fn inline_math_with_underscores_then_emits_span_with_verbatim_tex() {
        // Arrange
        let markdown = "The frame takes $T_{\\text{frame}}$.";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains(r#"<span class="math-inline" data-math-inline>T_{\text{frame}}</span>"#));
        assert!(!document.html.contains("<em>"));
    }

    #[test]
    fn inline_math_with_asterisks_then_emits_no_emphasis() {
        // Arrange
        let markdown = "$a*b*c$ and $x_1_2$";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains(r#"<span class="math-inline" data-math-inline>a*b*c</span>"#));
        assert!(document
            .html
            .contains(r#"<span class="math-inline" data-math-inline>x_1_2</span>"#));
        assert!(!document.html.contains("<em>"));
        assert!(!document.html.contains("<strong>"));
    }

    #[test]
    fn display_math_multiline_then_emits_display_span_preserving_newlines() {
        // Arrange
        let markdown = "$$\n\\begin{aligned}\na &= b\n\\end{aligned}\n$$";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            "<span class=\"math-display\" data-math-display>\n\\begin{aligned}\na &amp;= b\n\\end{aligned}\n</span>"
        ));
    }

    #[test]
    fn display_math_spanning_lines_then_keeps_line_breaks_in_span_text() {
        // Arrange
        let markdown = "$$\n\\begin{aligned}\na &= b\n\\end{aligned}\n$$";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            "<span class=\"math-display\" data-math-display>\n\\begin{aligned}\na &amp;= b\n\\end{aligned}\n</span>"
        ));
    }

    #[test]
    fn math_in_heading_then_emits_span_inside_heading() {
        // Arrange
        let markdown = "# Chapter $1$: $x + y$";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            r#"<h1>Chapter <span class="math-inline" data-math-inline>1</span>: <span class="math-inline" data-math-inline>x + y</span></h1>"#
        ));
    }

    #[test]
    fn math_in_blockquote_and_list_item_then_emits_spans_in_place() {
        // Arrange
        let markdown = "> $x$\n\n- $y$";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            r#"<blockquote>
<p><span class="math-inline" data-math-inline>x</span></p>
</blockquote>"#
        ));
        assert!(document
            .html
            .contains(r#"<li><span class="math-inline" data-math-inline>y</span></li>"#));
    }

    #[test]
    fn math_in_heading_link_footnote_and_table_cell_then_emits_spans_in_place() {
        // Arrange
        let markdown = "# Heading $h$\n\n[$l$](https://example.com)\n\nNote[^1].\n\n[^1]: Footnote $f$\n\n| Col |\n| --- |\n| $c$ |";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains(r#"<h1>Heading <span class="math-inline" data-math-inline>h</span></h1>"#));
        assert!(document.html.contains(
            r#"<a href="https://example.com"><span class="math-inline" data-math-inline>l</span></a>"#
        ));
        assert!(document
            .html
            .contains(r#"Footnote <span class="math-inline" data-math-inline>f</span>"#));
        assert!(document
            .html
            .contains(r#"<td><span class="math-inline" data-math-inline>c</span></td>"#));
    }

    #[test]
    fn currency_range_without_escapes_then_renders_math_per_parser_rule() {
        // Arrange
        let markdown = "Range $5-$10";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        // D1: "5-" renders as math, followed by literal "10"
        assert!(document
            .html
            .contains(r#"Range <span class="math-inline" data-math-inline>5-</span>10"#));
    }

    #[test]
    fn digit_after_closing_dollar_then_renders_math_followed_by_digit() {
        // Arrange
        let markdown = "$y$2";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        // D2: "y" renders as math, followed by literal "2"
        assert!(document
            .html
            .contains(r#"<span class="math-inline" data-math-inline>y</span>2"#));
    }

    #[test]
    fn shell_expression_in_prose_then_renders_math_per_parser_rule() {
        // Arrange
        let markdown = "$(CC)$(FLAGS)";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        // D6: "(CC)" renders as math, followed by literal "(FLAGS)"
        assert!(document
            .html
            .contains(r#"<span class="math-inline" data-math-inline>(CC)</span>(FLAGS)"#));
    }

    #[test]
    fn math_tex_with_html_characters_then_escapes_span_text() {
        // Arrange
        let markdown = r#"$a<b \text{"q"}$"#;

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            r#"<span class="math-inline" data-math-inline>a&lt;b \text{&quot;q&quot;}</span>"#
        ));
    }

    #[test]
    fn display_math_in_table_cell_then_emits_display_span_inside_cell() {
        // Arrange
        let markdown = "| Col |\n| --- |\n| $$x$$ |";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains(r#"<td><span class="math-display" data-math-display>x</span></td>"#));
    }

    #[test]
    fn display_math_paragraph_then_contains_no_block_element_inside_paragraph() {
        // Arrange
        let markdown = "$$x$$";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains(r#"<p><span class="math-display" data-math-display>x</span></p>"#));
        assert!(!document.html.contains("<div"));
    }

    #[test]
    fn fenced_math_block_then_emits_math_block_container_with_escaped_source() {
        // Arrange
        let markdown = "```math\n\\frac{1}{2}\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            r#"<div class="math-block" data-math-block><div class="math-target"></div><p class="math-error" hidden>Formula rendering failed. The source is shown below.</p><details class="math-source"><summary>Formula source</summary><pre><code>\frac{1}{2}
</code></pre></details></div>"#
        ));
        assert!(!document.html.contains("language-math"));
    }

    #[test]
    fn fenced_math_block_with_mixed_case_info_then_emits_math_block_container() {
        // Arrange
        let markdown = "```Math\n\\sum x\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            r#"<div class="math-block" data-math-block><div class="math-target"></div><p class="math-error" hidden>Formula rendering failed. The source is shown below.</p><details class="math-source"><summary>Formula source</summary><pre><code>\sum x
</code></pre></details></div>"#
        ));
        assert!(!document.html.contains("language-Math"));
    }

    #[test]
    fn fenced_math_block_inside_list_item_and_blockquote_then_emits_container_inside() {
        // Arrange
        let markdown = "> ```math\n> x\n> ```\n\n- ```math\n  y\n  ```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(
            "<blockquote>\n<div class=\"math-block\" data-math-block><div class=\"math-target\"></div><p class=\"math-error\" hidden>Formula rendering failed. The source is shown below.</p><details class=\"math-source\"><summary>Formula source</summary><pre><code>x\n</code></pre></details></div></blockquote>"
        ));
        assert!(document.html.contains(
            "<li><div class=\"math-block\" data-math-block><div class=\"math-target\"></div><p class=\"math-error\" hidden>Formula rendering failed. The source is shown below.</p><details class=\"math-source\"><summary>Formula source</summary><pre><code>y\n</code></pre></details></div></li>"
        ));
    }

    #[test]
    fn fenced_math_source_with_script_tag_then_escapes_source() {
        // Arrange
        let markdown = "```math\n<script>alert(1)</script>\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(!document.html.contains("<script>"));
        assert!(document
            .html
            .contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn spaced_dollar_delimiters_then_render_literal_text() {
        // Arrange
        let markdown = "$ 100 and 200 $";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert_eq!(document.html, "<p>$ 100 and 200 $</p>\n");
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn currency_amounts_then_render_literal_text() {
        // Arrange
        let markdown = "It costs $10 and $20.";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert_eq!(document.html, "<p>It costs $10 and $20.</p>\n");
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn escaped_dollar_signs_then_render_literal_dollars() {
        // Arrange
        let markdown = r"\$50 and \$100";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert_eq!(document.html, "<p>$50 and $100</p>\n");
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn code_span_with_dollars_then_preserves_code_without_math_span() {
        // Arrange
        let markdown = "`echo $PATH` and `$x$`";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("<p><code>echo $PATH</code> and <code>$x$</code></p>"));
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn code_span_and_code_block_with_dollar_math_then_emit_no_math_span() {
        // Arrange
        let markdown = "`$x$`\n\n```\n$y$\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("<code>$x$</code>"));
        assert!(document.html.contains("<pre><code>$y$\n</code></pre>"));
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn nested_list_and_blockquote_fences_with_dollar_math_then_emit_no_math_span() {
        // Arrange
        let markdown = "> ```\n> $x$\n> ```\n\n- ```\n  $y$\n  ```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("<pre><code>$x$\n</code></pre>"));
        assert!(document.html.contains("<pre><code>$y$\n</code></pre>"));
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn link_destination_and_autolink_with_dollars_then_preserve_urls() {
        // Arrange
        let markdown = "[l](https://a.example/$x$) and <https://a.example/$x$>";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains(r#"<a href="https://a.example/$x$">l</a>"#));
        assert!(document
            .html
            .contains(r#"<a href="https://a.example/$x$">https://a.example/$x$</a>"#));
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn diagram_fences_with_dollar_math_then_preserve_source_without_math_span() {
        // Arrange
        let markdown = "```mermaid\ngraph TD\n  A[$a$] --> B\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains("A[$a$] --&gt; B"));
        assert!(!document.html.contains("data-math"));
    }

    #[test]
    fn table_with_unescaped_pipe_in_math_then_splits_cell_per_parser_rule() {
        // Arrange
        let markdown = "| Parameter | Formula | Description |\n| :--- | :--- | :--- |\n| Normal | $|x|$ | Absolute value |";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        // D3: unescaped pipe inside $|x|$ splits the cell before math is parsed.
        // It produces separate cells for "$" and "x", dropping trailing content beyond column count.
        assert!(document.html.contains(
            "<td style=\"text-align: left\">$</td><td style=\"text-align: left\">x</td>"
        ));
        assert!(!document.html.contains("data-math-inline"));
    }

    #[test]
    fn table_with_escaped_pipe_in_math_then_preserves_cell_and_strips_escape() {
        // Arrange
        let markdown = "| Parameter | Formula | Description |\n| :--- | :--- | :--- |\n| Normal | $\\|x\\|$ | Absolute value |";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        // D3: escaped pipe keeps single cell, stripping backslash.
        assert!(document.html.contains(
            r#"<td style="text-align: left"><span class="math-inline" data-math-inline>|x|</span></td>"#
        ));
    }

    #[test]
    fn display_math_with_block_starter_then_terminates_paragraph() {
        // Arrange
        let markdown = "$$\n> quote\n$$";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        // D7: block starter line inside $$...$$ terminates paragraph; no display math
        assert!(!document.html.contains("data-math-display"));
        assert!(document.html.contains("<blockquote>"));
    }

    #[test]
    fn image_alt_with_inline_math_then_contains_literal_dollar_source() {
        // Arrange
        let markdown = "![speed $v$](plot.png)";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(r#"alt="speed $v$""#));
        assert!(!document.html.contains("data-math-inline"));
    }

    #[test]
    fn image_alt_with_display_math_then_contains_literal_double_dollar_source() {
        // Arrange
        let markdown = "![area $$r^2$$](a.png)";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(r#"alt="area $$r^2$$""#));
        assert!(!document.html.contains("data-math-display"));
    }

    #[test]
    fn image_alt_with_html_and_math_then_escapes_without_math_span() {
        // Arrange
        let markdown = "![a <b> $x$](i.png)";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document.html.contains(r#"alt="a &lt;b&gt; $x$""#));
        assert!(!document.html.contains("data-math-inline"));
    }

    #[test]
    fn fenced_math_extra_block_then_remains_code_block() {
        // Arrange
        let markdown = "```math extra\nfoo\n```";

        // Act
        let document = render_test(markdown, 0, "document.md", &BTreeSet::new());

        // Assert
        assert!(document
            .html
            .contains("<pre><code class=\"language-math\">foo\n</code></pre>"));
        assert!(!document.html.contains("data-math-block"));
    }

    fn temporary_source_link_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lens-markdown-source-link-{}-{name}",
            std::process::id()
        ))
    }
}
