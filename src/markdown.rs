use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};

use crate::plantuml::svg_url;

#[derive(Debug, Clone)]
pub struct Diagram {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct RenderedDocument {
    pub html: String,
    pub diagrams: Vec<Diagram>,
}

pub fn render(markdown: &str) -> RenderedDocument {
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
                        url: svg_url(&source),
                    });
                    events.push(Event::Html(diagram_placeholder(diagram_id, &source).into()));
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
            Event::Html(value) => events.push(Event::Text(value)),
            event => events.push(event),
        }
    }

    let mut html = String::new();
    html::push_html(&mut html, events.into_iter());
    RenderedDocument { html, diagrams }
}

fn diagram_placeholder(diagram_id: usize, source: &str) -> String {
    format!(
        r#"<figure class="diagram"><img src="/diagrams/{diagram_id}" alt="Rendered PlantUML diagram" data-diagram><p class="diagram-error" hidden>PlantUML rendering failed. The source is shown below.</p><details class="diagram-source"><summary>PlantUML source</summary><pre><code>{}</code></pre></details></figure>"#,
        escape_html(source)
    )
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
    use super::render;

    #[test]
    fn plantuml_block_then_adds_local_diagram_endpoint() {
        // Arrange
        let markdown = "```plantuml\n@startuml\nAlice -> Bob: hello\n@enduml\n```";

        // Act
        let document = render(markdown);

        // Assert
        assert_eq!(document.diagrams.len(), 1);
        assert!(document.html.contains("src=\"/diagrams/0\""));
        assert!(document.diagrams[0].url.contains("/svg/"));
    }

    #[test]
    fn other_fenced_block_then_remains_code_block() {
        // Arrange
        let markdown = "```rust\nlet answer = 42;\n```";

        // Act
        let document = render(markdown);

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
        let document = render(markdown);

        // Assert
        assert!(!document.html.contains("<script>"));
        assert!(document.html.contains("&lt;script&gt;"));
    }

    #[test]
    fn plantuml_source_with_html_then_escapes_source_in_fallback() {
        // Arrange
        let markdown = "```plantuml\nAlice -> Bob: <unsafe>\n```";

        // Act
        let document = render(markdown);

        // Assert
        assert!(document.html.contains("&lt;unsafe&gt;"));
    }
}
