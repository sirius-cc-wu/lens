use std::collections::BTreeSet;

use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};

use crate::plantuml::svg_url;

#[derive(Clone, Debug)]
pub struct Diagram {
    pub url: String,
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
    renderer_server: &str,
) -> RenderedDocument {
    let parser = Parser::new_ext(markdown, Options::all());
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
                        url: svg_url(renderer_server, &source),
                    });
                    events.push(Event::Html(
                        diagram_placeholder(document_id, diagram_id, &source).into(),
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

    let mut html = String::new();
    html::push_html(&mut html, events.into_iter());
    RenderedDocument { html, diagrams }
}

fn diagram_placeholder(document_id: usize, diagram_id: usize, source: &str) -> String {
    format!(
        r#"<figure class="diagram"><img src="/diagrams/{document_id}/{diagram_id}" alt="Rendered PlantUML diagram" data-diagram><p class="diagram-error" hidden>PlantUML rendering failed. The source is shown below.</p><details class="diagram-source"><summary>PlantUML source</summary><pre><code>{}</code></pre></details></figure>"#,
        escape_html(source)
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

    use super::render;
    use crate::plantuml::PUBLIC_SERVER;

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
            PUBLIC_SERVER,
        );

        // Assert
        assert_eq!(document.diagrams.len(), 1);
        assert!(document.html.contains("src=\"/diagrams/3/0\""));
        assert!(document.diagrams[0].url.contains("/svg/"));
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
            PUBLIC_SERVER,
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
            PUBLIC_SERVER,
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
        let document = render(markdown, 0, "document.md", &BTreeSet::new(), PUBLIC_SERVER);

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
        let document = render(markdown, 0, "document.md", &BTreeSet::new(), PUBLIC_SERVER);

        // Assert
        assert!(!document.html.contains("<script>"));
        assert!(document.html.contains("&lt;script&gt;"));
    }

    #[test]
    fn plantuml_source_with_html_then_escapes_source_in_fallback() {
        // Arrange
        let markdown = "```plantuml\nAlice -> Bob: <unsafe>\n```";

        // Act
        let document = render(markdown, 0, "document.md", &BTreeSet::new(), PUBLIC_SERVER);

        // Assert
        assert!(document.html.contains("&lt;unsafe&gt;"));
    }
}
