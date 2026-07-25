use crate::markdown::escape_html;

const APP_SCRIPT: &str = include_str!("assets/app.js");
const APP_STYLESHEET: &str = include_str!("assets/app.css");

pub(super) fn app_script() -> &'static str {
    embedded_asset(APP_SCRIPT)
}

pub(super) fn app_stylesheet() -> &'static str {
    embedded_asset(APP_STYLESHEET)
}

fn embedded_asset(asset: &'static str) -> &'static str {
    asset
        .strip_suffix("\r\n")
        .or_else(|| asset.strip_suffix('\n'))
        .unwrap_or(asset)
}

pub(super) fn page(
    title: &str,
    document_html: String,
    document_revision: Option<(&str, u64)>,
) -> String {
    let refresh_attributes = document_revision
        .map(|(document_id, revision)| {
            format!(
                r#" data-document-id="{}" data-document-revision="{revision}""#,
                escape_html(document_id),
            )
        })
        .unwrap_or_default();
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Lens: {}</title>
  <link rel="stylesheet" href="/app.css">
</head>
<body>
  <main{refresh_attributes}>
    <section class="document-content">
      <header class="document-header"><p class="eyebrow">Lens</p><h1>{}</h1></header>
      <article>{document_html}</article>
    </section>
  </main>
  <script src="/app.js"></script>
</body>
</html>"#,
        escape_html(title),
        escape_html(title),
    )
}

pub(super) fn document_unavailable_page() -> String {
    page(
        "Document unavailable",
        "<p>Lens can display the selected document, but the requested document is not part of this viewing session.</p><p><a href=\"/\">Return to the initial document</a></p>".to_owned(),
        None,
    )
}

pub(super) fn content_security_policy() -> &'static str {
    "default-src 'self'; base-uri 'none'; img-src 'self'; object-src 'none'; script-src 'self'; style-src 'self'"
}

#[cfg(test)]
mod tests {
    use super::{document_unavailable_page, page};

    #[test]
    fn embedded_asset_with_repository_newline_then_omits_only_final_line_ending() {
        // Arrange
        let assets = ["content\n", "content\r\n", "content"];

        // Act
        let bodies = assets.map(super::embedded_asset);

        // Assert
        assert_eq!(bodies, ["content", "content", "content"]);
    }

    #[test]
    fn document_page_then_omits_document_navigation_controls() {
        // Arrange
        let document_content = "<p>Document content</p>";

        // Act
        let document_page = page("README.md", document_content.to_owned(), None);

        // Assert
        assert!(document_page.contains(document_content));
        assert!(!document_page.contains("Discovered documents"));
        assert!(!document_page.contains("document-catalog"));
        assert!(!document_page.contains("document-search"));
        assert!(!document_page.contains("data-document-navigation-control"));
        assert!(!document_page.contains("data-document-navigation-toggle"));
        assert!(!document_page.contains("<nav id=\"document-navigation\""));
    }

    #[test]
    fn document_page_then_omits_rendering_status_and_disable_control() {
        // Arrange
        let expected_content = "<p>Document content</p>";

        // Act
        let document_page = page("README.md", expected_content.to_owned(), None);

        // Assert
        assert!(document_page.contains(expected_content));
        assert!(!document_page.contains("PlantUML server rendering"));
        assert!(!document_page.contains("rendering-status"));
        assert!(!document_page.contains("data-disable-renderer"));
    }

    #[test]
    fn unavailable_document_then_explains_how_to_return() {
        // Arrange
        let expected_message = "requested document is not part of this viewing session";

        // Act
        let page = document_unavailable_page();

        // Assert
        assert!(page.contains("<title>Lens: Document unavailable</title>"));
        assert!(page.contains(expected_message));
        assert!(page.contains("href=\"/\""));
    }
}
