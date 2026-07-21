use std::fmt::Write as _;

use super::catalog::{CatalogPage, CatalogResults, MAX_QUERY_BYTES, RESULT_LIMIT};
use crate::markdown::escape_html;

const APP_SCRIPT: &str = include_str!("assets/app.js");
const APP_STYLESHEET: &str = include_str!("assets/app.css");

pub(super) fn app_script() -> &'static str {
    APP_SCRIPT.strip_suffix('\n').unwrap_or(APP_SCRIPT)
}

pub(super) fn app_stylesheet() -> &'static str {
    APP_STYLESHEET.strip_suffix('\n').unwrap_or(APP_STYLESHEET)
}

pub(super) fn page(
    title: &str,
    document_html: String,
    navigation_html: String,
    renderer_controls: String,
    rendering_disabled: bool,
    document_revision: Option<(&str, u64)>,
) -> String {
    let navigation_control = if navigation_html.is_empty() {
        String::new()
    } else {
        document_navigation_control().to_owned()
    };
    let refresh_attributes = document_revision
        .map(|(document_id, revision)| {
            format!(
                r#" data-document-id="{}" data-document-revision="{revision}""#,
                escape_html(document_id),
            )
        })
        .unwrap_or_default();
    let rendering_disabled_attribute = rendering_disabled
        .then_some(r#" data-diagram-rendering-disabled="true""#)
        .unwrap_or_default();
    format!(
        r#"<!doctype html>
<html lang="en"{rendering_disabled_attribute}>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Lens: {}</title>
  <link rel="stylesheet" href="/app.css">
</head>
<body>
  <main{refresh_attributes}>
    {navigation_control}
    {navigation_html}
    <section class="document-content">
      <header><p class="eyebrow">Lens</p><h1>{}</h1></header>
      {renderer_controls}
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

pub(super) fn navigation_pane(
    page: CatalogPage,
    current_document: usize,
    current_route: &str,
) -> String {
    let (query, status, document_links, page_links) = match page {
        CatalogPage::QueryTooLong { query } => (
            query,
            format!("Search queries are limited to {MAX_QUERY_BYTES} UTF-8 bytes."),
            String::new(),
            String::new(),
        ),
        CatalogPage::Results(results) => {
            let status = catalog_status(&results);
            let document_links = catalog_result_links(&results, current_document);
            let page_links = catalog_page_links(&results, current_route);
            (results.query, status, document_links, page_links)
        }
    };

    format!(
        r#"<nav id="document-navigation" class="document-navigation" aria-label="Discovered documents"><h2>Documents</h2>{}<p role="status">{status}</p><ul id="document-catalog">{document_links}</ul>{page_links}</nav>"#,
        catalog_search_form(&query, current_route),
    )
}

pub(super) fn renderer_controls(renderer_label: &str, rendering_enabled: bool) -> String {
    if rendering_enabled {
        format!(
            r#"<section class="renderer-controls" data-renderer-controls><p role="status" data-renderer-status>Diagram renderer: {renderer_label}.</p><button type="button" data-disable-renderer>Disable diagram rendering for this session</button></section>"#,
        )
    } else {
        r#"<section class="renderer-controls" data-renderer-controls><p role="status" data-renderer-status>Diagram rendering is disabled for this viewing session.</p></section>"#.to_owned()
    }
}

pub(super) fn deferred_navigation_page() -> String {
    page(
        "Document navigation unavailable",
        "<p>Lens can display the selected document, but the requested document is not part of this viewing session.</p><p><a href=\"/\">Return to the initial document</a></p>".to_owned(),
        String::new(),
        String::new(),
        false,
        None,
    )
}

pub(super) fn content_security_policy() -> &'static str {
    "default-src 'self'; base-uri 'none'; img-src 'self'; object-src 'none'; script-src 'self'; style-src 'self'"
}

fn catalog_search_form(query: &str, current_route: &str) -> String {
    format!(
        r#"<form class="document-search" method="get" action="{}"><label for="document-search">Search discovered documents</label><input id="document-search" name="query" type="search" value="{}" maxlength="{MAX_QUERY_BYTES}"><button type="submit">Search</button></form>"#,
        escape_html(current_route),
        escape_html(query),
    )
}

fn catalog_status(results: &CatalogResults) -> String {
    if results.total == 0 {
        return "No discovered documents match the search.".to_owned();
    }

    let first_result = (results.page - 1) * RESULT_LIMIT + 1;
    let last_result = first_result + results.entries.len() - 1;
    if results.query.is_empty() {
        format!(
            "Showing {first_result}–{last_result} of {} discovered documents.",
            results.total
        )
    } else {
        format!(
            "Showing {first_result}–{last_result} of {} discovered documents matching \"{}\".",
            results.total,
            escape_html(&results.query),
        )
    }
}

fn catalog_result_links(results: &CatalogResults, current_document: usize) -> String {
    let mut document_links = String::new();
    let page_query = escape_html(&results.page_query(results.page));
    for entry in &results.entries {
        let current = (entry.document_index == current_document)
            .then_some(r#" aria-current="page""#)
            .unwrap_or_default();
        let identifier = escape_html(&entry.identifier);
        write!(
            document_links,
            r#"<li data-document-navigation-item><a href="/documents/{identifier}?{page_query}"{current}>{identifier}</a></li>"#
        )
        .expect("writing navigation markup to a string cannot fail");
    }
    document_links
}

fn catalog_page_links(results: &CatalogResults, current_route: &str) -> String {
    let mut page_links = String::new();
    if results.has_previous_page() {
        write!(
            page_links,
            r#"<a href="{}?{}" rel="prev">Previous results</a>"#,
            escape_html(current_route),
            escape_html(&results.page_query(results.page - 1)),
        )
        .expect("writing navigation markup to a string cannot fail");
    }
    if results.has_next_page() {
        if !page_links.is_empty() {
            page_links.push(' ');
        }
        write!(
            page_links,
            r#"<a href="{}?{}" rel="next">Next results</a>"#,
            escape_html(current_route),
            escape_html(&results.page_query(results.page + 1)),
        )
        .expect("writing navigation markup to a string cannot fail");
    }

    page_links
        .is_empty()
        .then(String::new)
        .unwrap_or_else(|| format!(r#"<p class="document-result-pages">{page_links}</p>"#))
}

fn document_navigation_control() -> &'static str {
    r#"<div class="document-navigation-control" data-document-navigation-control hidden><button type="button" data-document-navigation-toggle aria-controls="document-navigation" aria-expanded="true">Hide documents</button></div>"#
}

#[cfg(test)]
mod tests {
    use super::{deferred_navigation_page, navigation_pane, page, renderer_controls};
    use crate::viewer::catalog::{DocumentCatalog, NavigationRequest};

    fn test_navigation(
        identifiers: impl IntoIterator<Item = String>,
        current_document: usize,
        current_route: &str,
    ) -> String {
        let catalog = DocumentCatalog::new(
            identifiers
                .into_iter()
                .enumerate()
                .map(|(index, identifier)| (identifier, index)),
        );
        let request = NavigationRequest::from_raw_query(None);
        navigation_pane(catalog.search(&request), current_document, current_route)
    }

    #[test]
    fn document_navigation_pane_then_lists_known_documents_and_marks_current() {
        // Arrange
        let identifiers = ["README.md".to_owned(), "guides/intro.md".to_owned()];

        // Act
        let navigation = test_navigation(identifiers, 1, "/documents/guides/intro.md");

        // Assert
        assert!(navigation.contains("aria-label=\"Discovered documents\""));
        assert!(navigation.contains("href=\"/documents/README.md?query=&amp;page=1\""));
        assert!(navigation.contains(
            "href=\"/documents/guides/intro.md?query=&amp;page=1\" aria-current=\"page\""
        ));
        assert!(!navigation.contains(".private.md"));
        assert_eq!(navigation.matches("aria-current=\"page\"").count(), 1);
    }

    #[test]
    fn document_page_with_navigation_then_exposes_an_accessible_visibility_control() {
        // Arrange
        let navigation = test_navigation(["README.md".to_owned()], 0, "/");

        // Act
        let document_page = page(
            "README.md",
            String::new(),
            navigation,
            String::new(),
            false,
            None,
        );

        // Assert
        assert!(document_page.contains("data-document-navigation-control hidden"));
        assert!(document_page.contains("data-document-navigation-toggle"));
        assert!(document_page.contains("aria-controls=\"document-navigation\""));
        assert!(document_page.contains("aria-expanded=\"true\""));
        assert!(document_page.contains("<nav id=\"document-navigation\""));
    }

    #[test]
    fn document_navigation_pane_with_html_identifier_then_escapes_identifier() {
        // Arrange
        let identifiers = ["guides/<unsafe>.md".to_owned()];

        // Act
        let navigation = test_navigation(identifiers, 0, "/");

        // Assert
        assert!(navigation.contains("/documents/guides/&lt;unsafe&gt;.md"));
        assert!(!navigation.contains("/documents/guides/<unsafe>.md"));
    }

    #[test]
    fn enabled_renderer_then_exposes_its_status_and_disable_control() {
        // Arrange
        let renderer_label = "public";

        // Act
        let controls = renderer_controls(renderer_label, true);

        // Assert
        assert!(controls.contains("Diagram renderer: public."));
        assert!(controls.contains("data-disable-renderer"));
    }

    #[test]
    fn more_than_result_limit_then_shows_first_page_and_next_link() {
        // Arrange
        let identifiers = (0..=50).map(|index| format!("guides/{index:03}.md"));

        // Act
        let navigation = test_navigation(identifiers, 0, "/");

        // Assert
        assert_eq!(
            navigation.matches("data-document-navigation-item").count(),
            50
        );
        assert!(navigation.contains("Next results"));
        assert!(!navigation.contains("guides/050.md"));
    }

    #[test]
    fn deferred_document_navigation_then_explains_how_to_return() {
        // Arrange
        let expected_message = "requested document is not part of this viewing session";

        // Act
        let page = deferred_navigation_page();

        // Assert
        assert!(page.contains(expected_message));
        assert!(page.contains("href=\"/\""));
    }
}
