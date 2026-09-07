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
    session_token: &str,
) -> String {
    let refresh_attributes = document_revision
        .map(|(document_id, revision)| {
            format!(
                r#" data-document-id="{}" data-document-revision="{revision}""#,
                escape_html(document_id),
            )
        })
        .unwrap_or_default();
    let document_html = inject_capability(&document_html, session_token);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="referrer" content="no-referrer">
  <title>Lens: {}</title>
  <link rel="stylesheet" href="/app.css?token={}">
</head>
<body>
  <main{refresh_attributes} data-session-token="{}">
    <section class="document-content">
      <header class="document-header"><p class="eyebrow">Lens</p><h1>{}</h1></header>
      <article>{document_html}</article>
    </section>
  </main>
  <script src="/app.js?token={}"></script>
</body>
</html>"#,
        escape_html(title),
        session_token,
        session_token,
        escape_html(title),
        session_token,
    )
}

pub(super) fn document_unavailable_page(session_token: &str) -> String {
    page(
        "Document unavailable",
        format!(
            "<p>Lens can display the selected document, but the requested document is not part of this viewing session.</p><p><a href=\"/?token={session_token}\">Return to the initial document</a></p>"
        ),
        None,
        session_token,
    )
}

pub(super) fn inject_capability(html: &str, token: &str) -> String {
    let mut result = String::with_capacity(html.len() + 128);
    let mut cursor = 0;

    while cursor < html.len() {
        let remainder = &html[cursor..];
        let next_match = [
            remainder
                .find(r#"href="/documents/"#)
                .map(|pos| (pos, 6, '"')),
            remainder
                .find(r#"src="/diagrams/"#)
                .map(|pos| (pos, 5, '"')),
        ]
        .into_iter()
        .flatten()
        .min_by_key(|&(pos, _, _)| pos);

        match next_match {
            Some((pos, prefix_len, quote)) => {
                let match_start = cursor + pos;
                let url_start = match_start + prefix_len;
                result.push_str(&html[cursor..url_start]);

                if let Some(quote_offset) = html[url_start..].find(quote) {
                    let url_end = url_start + quote_offset;
                    let url = &html[url_start..url_end];
                    let (base, fragment) = match url.split_once('#') {
                        Some((b, f)) => (b, Some(f)),
                        None => (url, None),
                    };
                    result.push_str(base);
                    if base.contains('?') {
                        result.push_str("&token=");
                    } else {
                        result.push_str("?token=");
                    }
                    result.push_str(token);
                    if let Some(f) = fragment {
                        result.push('#');
                        result.push_str(f);
                    }
                    cursor = url_end;
                } else {
                    cursor = url_start;
                }
            }
            None => {
                result.push_str(&html[cursor..]);
                break;
            }
        }
    }

    result
}

pub(super) fn content_security_policy() -> &'static str {
    "default-src 'self'; base-uri 'none'; img-src 'self'; object-src 'none'; script-src 'self'; style-src 'self'"
}

#[cfg(test)]
mod tests {
    use super::{document_unavailable_page, inject_capability, page};

    const TEST_TOKEN: &str = "test-token";

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
        let document_page = page("README.md", document_content.to_owned(), None, TEST_TOKEN);

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
        let document_page = page("README.md", expected_content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(document_page.contains(expected_content));
        assert!(!document_page.contains("PlantUML server rendering"));
        assert!(!document_page.contains("rendering-status"));
        assert!(!document_page.contains("data-disable-renderer"));
    }

    #[test]
    fn document_page_then_includes_session_token_and_no_referrer_policy() {
        // Arrange
        let content = "<p>Content</p>";

        // Act
        let rendered = page("Test", content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(rendered.contains(r#"<meta name="referrer" content="no-referrer">"#));
        assert!(rendered.contains(r#"<link rel="stylesheet" href="/app.css?token=test-token">"#));
        assert!(rendered.contains(r#"<script src="/app.js?token=test-token"></script>"#));
        assert!(rendered.contains(r#"data-session-token="test-token""#));
    }

    #[test]
    fn inject_capability_then_attaches_token_to_document_and_diagram_urls() {
        // Arrange
        let html = r#"<p><a href="/documents/README.md#install">Install</a></p><img src="/diagrams/0/0" alt="Diagram"><a href="https://example.com">External</a>"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert!(injected.contains(r#"href="/documents/README.md?token=test-token#install""#));
        assert!(injected.contains(r#"src="/diagrams/0/0?token=test-token""#));
        assert!(injected.contains(r#"href="https://example.com""#));
    }

    #[test]
    fn unavailable_document_then_explains_how_to_return() {
        // Arrange
        let expected_message = "requested document is not part of this viewing session";

        // Act
        let page = document_unavailable_page(TEST_TOKEN);

        // Assert
        assert!(page.contains("<title>Lens: Document unavailable</title>"));
        assert!(page.contains(expected_message));
        assert!(page.contains(r#"href="/?token=test-token""#));
    }
}
