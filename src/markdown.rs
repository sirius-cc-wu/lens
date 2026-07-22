use std::{collections::BTreeSet, fmt::Write as _};

use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use serde_yaml::{Mapping, Value};

use crate::plantuml::DiagramRenderer;

#[derive(Clone, Debug)]
pub struct Diagram {
    pub source: String,
}

#[derive(Debug)]
pub struct RenderedDocument {
    pub html: String,
    pub diagrams: Vec<Diagram>,
}

pub fn render(
    markdown: &str,
    document_id: usize,
    current_document: &str,
    known_documents: &BTreeSet<String>,
    renderer: &DiagramRenderer,
) -> RenderedDocument {
    let frontmatter = frontmatter(markdown);
    let parser = Parser::new_ext(frontmatter.body, Options::all());
    let mut events = Vec::new();
    let mut diagrams = Vec::new();
    let mut plantuml_source: Option<String> = None;

    for event in parser {
        if let Some(source) = plantuml_source.as_mut() {
            match event {
                Event::End(Tag::CodeBlock(_)) => {
                    let source = plantuml_source.take().expect("PlantUML source is active");
                    let diagram_id = diagrams.len();
                    diagrams.push(Diagram {
                        source: source.clone(),
                    });
                    events.push(Event::Html(
                        diagram_placeholder(
                            document_id,
                            diagram_id,
                            &source,
                            renderer.is_enabled(),
                        )
                        .into(),
                    ));
                }
                Event::Text(text) | Event::Code(text) => source.push_str(&text),
                Event::SoftBreak | Event::HardBreak => source.push('\n'),
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))
                if language.trim().eq_ignore_ascii_case("plantuml") =>
            {
                plantuml_source = Some(String::new());
            }
            Event::Start(Tag::Link(link_type, destination, title)) => {
                events.push(Event::Start(Tag::Link(
                    link_type,
                    resolve_document_link(&destination, current_document, known_documents).into(),
                    title,
                )));
            }
            Event::Html(value) => events.push(Event::Text(value)),
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

pub fn render_standalone_plantuml(
    document_id: usize,
    source: &str,
    renderer: &DiagramRenderer,
) -> RenderedDocument {
    RenderedDocument {
        html: format!(
            r#"<p class="standalone-plantuml">Standalone PlantUML file.</p>{}"#,
            diagram_placeholder(document_id, 0, source, renderer.is_enabled())
        ),
        diagrams: vec![Diagram {
            source: source.to_owned(),
        }],
    }
}

fn diagram_placeholder(
    document_id: usize,
    diagram_id: usize,
    source: &str,
    rendering_enabled: bool,
) -> String {
    let image = rendering_enabled
        .then(|| {
            format!(
                r#"<img src="/diagrams/{document_id}/{diagram_id}" alt="Rendered PlantUML diagram" data-diagram>"#
            )
        })
        .unwrap_or_default();
    let disabled_status = rendering_enabled.then_some(" hidden").unwrap_or_default();
    format!(
        r#"<figure class="diagram" data-diagram-container>{image}<p class="diagram-error" hidden>PlantUML rendering failed. The source is shown below.</p><p class="diagram-disabled" data-diagram-disabled{disabled_status}>PlantUML rendering is disabled for this viewing session.</p><button type="button" data-diagram-retry hidden>Retry diagram rendering</button><details class="diagram-source"><summary>PlantUML source</summary><pre><code>{}</code></pre></details></figure>"#,
        escape_html(source),
    )
}

fn resolve_document_link(
    destination: &str,
    current_document: &str,
    known_documents: &BTreeSet<String>,
) -> String {
    let (path, suffix) = split_link_suffix(destination);
    if path.is_empty() || !is_markdown_path(path) {
        return destination.to_owned();
    }

    let Some(candidate) = normalize_document_path(current_document, path) else {
        return destination.to_owned();
    };
    if known_documents.contains(&candidate) {
        format!("/documents/{candidate}{suffix}")
    } else {
        destination.to_owned()
    }
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

fn is_markdown_path(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
    })
}

fn normalize_document_path(current_document: &str, destination: &str) -> Option<String> {
    if destination.starts_with('/') || destination.contains('\u{005C}') {
        return None;
    }

    let mut components = current_document
        .split('/')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    components.pop();

    for component in destination.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            component => components.push(component.to_owned()),
        }
    }

    (!components.is_empty()).then(|| components.join("/"))
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
    use std::collections::BTreeSet;

    use super::{render, render_standalone_plantuml};
    use crate::plantuml::{DiagramRenderer, RendererMode};

    fn public_renderer() -> DiagramRenderer {
        DiagramRenderer::from_mode(RendererMode::Public)
    }

    #[test]
    fn plantuml_block_then_adds_document_scoped_diagram_endpoint() {
        // Arrange
        let markdown = "```plantuml\n@startuml\nAlice -> Bob: hello\n@enduml\n```";

        // Act
        let document = render(
            markdown,
            3,
            "guides/intro.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

        // Assert
        assert_eq!(document.diagrams.len(), 1);
        assert!(document.html.contains("src=\"/diagrams/3/0\""));
        assert_eq!(
            document.diagrams[0].source,
            "@startuml\nAlice -> Bob: hello\n@enduml\n"
        );
    }

    #[test]
    fn disabled_renderer_then_keeps_plantuml_source_without_an_image_request() {
        // Arrange
        let markdown = "```plantuml\n@startuml\nAlice -> Bob: private\n@enduml\n```";
        let renderer = DiagramRenderer::from_mode(RendererMode::Disabled);

        // Act
        let document = render(markdown, 0, "document.md", &BTreeSet::new(), &renderer);

        // Assert
        assert!(!document.html.contains("<img"));
        assert!(document
            .html
            .contains("PlantUML rendering is disabled for this viewing session."));
        assert!(document.html.contains("Alice -&gt; Bob: private"));
    }

    #[test]
    fn standalone_plantuml_source_then_renders_a_document_scoped_diagram() {
        // Arrange
        let source = "@startuml\nAlice -> Bob: standalone\n@enduml";

        // Act
        let document = render_standalone_plantuml(2, source, &public_renderer());

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
        let document = render(
            markdown,
            0,
            "guides/intro.md",
            &known_documents,
            &public_renderer(),
        );

        // Assert
        assert!(document
            .html
            .contains("href=\"/documents/README.md#install\""));
    }

    #[test]
    fn unknown_or_external_link_then_preserves_original_destination() {
        // Arrange
        let markdown = "[Unknown](../../secret.md) [External](https://example.com/guide.md)";
        let known_documents = BTreeSet::from(["guides/intro.md".to_owned()]);

        // Act
        let document = render(
            markdown,
            0,
            "guides/intro.md",
            &known_documents,
            &public_renderer(),
        );

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
        let document = render(
            markdown,
            0,
            "document.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

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
        let document = render(
            markdown,
            0,
            "document.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

        // Assert
        assert!(!document.html.contains("<script>"));
        assert!(document.html.contains("&lt;script&gt;"));
    }

    #[test]
    fn valid_frontmatter_then_renders_nested_metadata_before_markdown_body() {
        // Arrange
        let markdown = "---\ntitle: Lens guide\nauthor: Ada\ntags:\n  - rust\n  - docs\npublication:\n  audience: maintainers\n---\n# Guide\n\nBody text.";

        // Act
        let document = render(
            markdown,
            0,
            "guide.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

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
        let document = render(
            markdown,
            0,
            "guide.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

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
        let document = render(
            markdown,
            0,
            "guide.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

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
        let document = render(
            markdown,
            0,
            "guide.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

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
        let document = render(
            markdown,
            0,
            "guide.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

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
        let document = render(
            markdown,
            0,
            "document.md",
            &BTreeSet::new(),
            &public_renderer(),
        );

        // Assert
        assert!(document.html.contains("&lt;unsafe&gt;"));
    }
}
