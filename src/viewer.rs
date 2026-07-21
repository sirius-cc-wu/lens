use std::{fmt::Write as _, net::TcpListener, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    extract::{Path, RawQuery, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
mod browser;
mod catalog;
mod rendering;
mod state;

use browser::open_browser;
use catalog::{CatalogPage, CatalogResults, NavigationRequest, MAX_QUERY_BYTES, RESULT_LIMIT};
use rendering::{renderer_client, request_diagram};
use state::{viewer_state, watch_documents, ViewerState};

use crate::{
    markdown::escape_html,
    plantuml::{DiagramRenderer, RendererMode},
    target::MarkdownTarget,
};

impl ViewerState {
    fn navigation_pane(
        &self,
        current_document: usize,
        request: &NavigationRequest,
        current_route: &str,
    ) -> String {
        navigation_pane(
            self.catalog.search(request),
            current_document,
            current_route,
        )
    }

    fn renderer_controls(&self) -> String {
        if self.rendering_enabled() {
            format!(
                r#"<section class="renderer-controls" data-renderer-controls><p role="status" data-renderer-status>Diagram renderer: {}.</p><button type="button" data-disable-renderer>Disable diagram rendering for this session</button></section>"#,
                self.renderer.label()
            )
        } else {
            r#"<section class="renderer-controls" data-renderer-controls><p role="status" data-renderer-status>Diagram rendering is disabled for this viewing session.</p></section>"#.to_owned()
        }
    }
}

pub async fn serve(target: MarkdownTarget, renderer_mode: RendererMode) -> Result<()> {
    let (documents, initial_document) = target.into_parts();
    let initial_path = documents[initial_document].canonical_path.clone();
    let diagram_renderer = DiagramRenderer::from_mode(renderer_mode);
    let state = viewer_state(
        documents,
        initial_document,
        renderer_client()?,
        diagram_renderer,
    );
    tokio::spawn(watch_documents(state.clone()));
    let listener =
        TcpListener::bind("127.0.0.1:0").context("Could not start the loopback viewer")?;
    let address = listener
        .local_addr()
        .context("Could not determine the loopback viewer address")?;
    let url = format!("http://{address}");

    println!("Lens is serving {} at {url}", initial_path.display());
    if let Err(error) = open_browser(&url) {
        eprintln!("Could not open a browser automatically: {error}");
        eprintln!("Open {url} manually.");
    }

    axum::Server::from_tcp(listener)
        .context("Could not serve the loopback viewer")?
        .serve(router(state).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("The loopback viewer stopped unexpectedly")
}

fn router(state: Arc<ViewerState>) -> Router {
    Router::new()
        .route("/", get(initial_document_view))
        .route("/documents/*document_id", get(document_view))
        .route("/revisions/*document_id", get(document_revision))
        .route("/app.css", get(stylesheet))
        .route("/app.js", get(script))
        .route("/diagrams/:document_id/:diagram_id", get(diagram))
        .route("/renderer/disable", post(disable_renderer))
        .fallback(not_found)
        .with_state(state)
}

async fn initial_document_view(
    State(state): State<Arc<ViewerState>>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let request = NavigationRequest::from_raw_query(raw_query.as_deref());
    rendered_document_response(&state, state.initial_document, &request, "/")
}

async fn document_view(
    State(state): State<Arc<ViewerState>>,
    Path(document_id): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let document_id = document_id.trim_start_matches('/');
    let request = NavigationRequest::from_raw_query(raw_query.as_deref());
    match state.catalog.known_document_index(document_id) {
        Some(known_document) => {
            let route = format!("/documents/{document_id}");
            rendered_document_response(&state, known_document, &request, &route)
        }
        None => not_found().await.into_response(),
    }
}

async fn document_revision(
    State(state): State<Arc<ViewerState>>,
    Path(document_id): Path<String>,
) -> Response {
    let document_id = document_id.trim_start_matches('/');
    match state.catalog.known_document_index(document_id) {
        Some(document_id) => (
            [(header::CACHE_CONTROL, "no-store")],
            state
                .document_revision(document_id)
                .expect("known document index should remain valid")
                .to_string(),
        )
            .into_response(),
        None => not_found().await.into_response(),
    }
}

fn rendered_document_response(
    state: &ViewerState,
    document_id: usize,
    request: &NavigationRequest,
    current_route: &str,
) -> Response {
    let navigation = state.navigation_pane(document_id, request, current_route);
    let renderer_controls = state.renderer_controls();
    let documents = state
        .documents
        .read()
        .expect("viewer documents lock should not be poisoned");
    let document = &documents[document_id];
    (
        [(header::CONTENT_SECURITY_POLICY, content_security_policy())],
        Html(page(
            &document.canonical_path.display().to_string(),
            document.rendered.html.clone(),
            navigation,
            renderer_controls,
            !state.rendering_enabled(),
            Some((&document.identifier, document.revision)),
        )),
    )
        .into_response()
}

fn navigation_pane(page: CatalogPage, current_document: usize, current_route: &str) -> String {
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

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        APP_STYLESHEET,
    )
}

async fn script() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        APP_SCRIPT,
    )
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_SECURITY_POLICY, content_security_policy())],
        Html(deferred_navigation_page()),
    )
}

async fn diagram(
    State(state): State<Arc<ViewerState>>,
    Path((document_id, diagram_id)): Path<(usize, usize)>,
) -> Response {
    if !state.rendering_enabled() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "PlantUML rendering is disabled for this viewing session.",
        )
            .into_response();
    }
    let diagram = state
        .documents
        .read()
        .expect("viewer documents lock should not be poisoned")
        .get(document_id)
        .and_then(|document| document.rendered.diagrams.get(diagram_id))
        .cloned();
    let Some(diagram) = diagram else {
        return (StatusCode::NOT_FOUND, "Diagram not found").into_response();
    };

    match request_diagram(&state.renderer, &state.client, &diagram).await {
        Ok(svg) => (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("image/svg+xml"),
            )],
            svg,
        )
            .into_response(),
        Err(error) => {
            eprintln!("PlantUML rendering failed: {error}");
            (
                StatusCode::BAD_GATEWAY,
                "PlantUML rendering failed. See the source shown in the document.",
            )
                .into_response()
        }
    }
}

async fn disable_renderer(State(state): State<Arc<ViewerState>>) -> StatusCode {
    state.disable_rendering();
    StatusCode::NO_CONTENT
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Could not listen for Ctrl-C: {error}");
    }
}

fn page(
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

fn document_navigation_control() -> &'static str {
    r#"<div class="document-navigation-control" data-document-navigation-control hidden><button type="button" data-document-navigation-toggle aria-controls="document-navigation" aria-expanded="true">Hide documents</button></div>"#
}

fn deferred_navigation_page() -> String {
    page(
        "Document navigation unavailable",
        "<p>Lens can display the selected document, but the requested document is not part of this viewing session.</p><p><a href=\"/\">Return to the initial document</a></p>".to_owned(),
        String::new(),
        String::new(),
        false,
        None,
    )
}

fn content_security_policy() -> &'static str {
    "default-src 'self'; base-uri 'none'; img-src 'self'; object-src 'none'; script-src 'self'; style-src 'self'"
}

const APP_SCRIPT: &str = r#"const markDiagramDisabled = (figure) => {
  const image = figure.querySelector('[data-diagram]');
  if (image) image.removeAttribute('src');
  figure.querySelector('.diagram-error').hidden = true;
  figure.querySelector('[data-diagram-retry]').hidden = true;
  figure.querySelector('[data-diagram-disabled]').hidden = false;
  figure.querySelector('.diagram-source').open = true;
};

const navigationControl = document.querySelector('[data-document-navigation-control]');
const navigationToggle = document.querySelector('[data-document-navigation-toggle]');
const navigationPane = document.querySelector('#document-navigation');
const documentLayout = document.querySelector('main');
if (navigationControl && navigationToggle && navigationPane && documentLayout) {
  const navigationPaneStateKey = 'lens.documentNavigationCollapsed';
  const setNavigationPaneCollapsed = (collapsed) => {
    navigationPane.hidden = collapsed;
    documentLayout.dataset.documentNavigationCollapsed = String(collapsed);
    navigationToggle.setAttribute('aria-expanded', String(!collapsed));
    navigationToggle.textContent = collapsed ? 'Show documents' : 'Hide documents';
  };
  let collapsed = false;
  try {
    collapsed = sessionStorage.getItem(navigationPaneStateKey) === 'true';
  } catch {
    // Keep the navigation pane visible when browser session storage is unavailable.
  }
  setNavigationPaneCollapsed(collapsed);
  navigationControl.hidden = false;
  navigationToggle.addEventListener('click', () => {
    collapsed = !navigationPane.hidden;
    setNavigationPaneCollapsed(collapsed);
    try {
      sessionStorage.setItem(navigationPaneStateKey, String(collapsed));
    } catch {
      // Retain the current page's visibility when browser session storage is unavailable.
    }
  });
}

for (const image of document.querySelectorAll('[data-diagram]')) {
  const revealFailure = () => {
    const figure = image.closest('.diagram');
    if (document.documentElement.dataset.diagramRenderingDisabled === 'true') {
      markDiagramDisabled(figure);
      return;
    }
    figure.querySelector('.diagram-error').hidden = false;
    figure.querySelector('[data-diagram-retry]').hidden = false;
    figure.querySelector('.diagram-source').open = true;
  };
  const retry = image.closest('.diagram').querySelector('[data-diagram-retry]');
  retry.addEventListener('click', () => {
    image.closest('.diagram').querySelector('.diagram-error').hidden = true;
    retry.hidden = true;
    const retryUrl = new URL(image.src, window.location.origin);
    retryUrl.searchParams.set('retry', Date.now().toString());
    image.src = retryUrl.toString();
  });
  image.addEventListener('error', revealFailure);
  if (image.complete && image.naturalWidth === 0) {
    revealFailure();
  }
}

const disableRenderer = document.querySelector('[data-disable-renderer]');
if (disableRenderer) {
  disableRenderer.addEventListener('click', async () => {
    disableRenderer.disabled = true;
    try {
      const response = await fetch('/renderer/disable', { method: 'POST' });
      if (!response.ok) throw new Error('disable failed');
      document.documentElement.dataset.diagramRenderingDisabled = 'true';
      document.querySelector('[data-renderer-status]').textContent =
        'Diagram rendering is disabled for this viewing session.';
      for (const figure of document.querySelectorAll('[data-diagram-container]')) {
        markDiagramDisabled(figure);
      }
      disableRenderer.remove();
    } catch {
      disableRenderer.disabled = false;
    }
  });
}

const documentView = document.querySelector('[data-document-id][data-document-revision]');
if (documentView) {
  const documentId = documentView.dataset.documentId;
  let revision = documentView.dataset.documentRevision;
  let reloading = false;

  window.setInterval(async () => {
    try {
      const response = await fetch(`/revisions/${encodeURIComponent(documentId)}`, { cache: 'no-store' });
      if (!response.ok) return;
      const currentRevision = await response.text();
      if (currentRevision !== revision && !reloading) {
        reloading = true;
        window.location.reload();
      }
    } catch {
      // Retain the readable document and try again on the next interval.
    }
  }, 500);
}"#;

const APP_STYLESHEET: &str = r#"* { box-sizing: border-box; }
body { margin: 0; background: #f4f1ea; color: #1d2826; font-family: Georgia, serif; line-height: 1.55; }
main { width: min(1200px, calc(100% - 2rem)); margin: 3rem auto 5rem; display: grid; grid-template-columns: minmax(13rem, .35fr) minmax(0, 920px); gap: 2rem; align-items: start; }
.document-navigation-control { grid-column: 1; grid-row: 1; }
.document-navigation-control button { padding: .35rem .65rem; border: 1px solid #1d2826; background: #1d2826; color: #fffdf8; font: inherit; }
.document-navigation { position: sticky; top: 1rem; grid-column: 1; grid-row: 2; padding: 1rem; border: 1px solid #b6b0a4; background: #fffdf8; font-family: system-ui, sans-serif; }
.document-navigation h2 { margin: 0 0 .75rem; font-size: 1rem; }
.document-navigation label { display: block; font-size: .8rem; font-weight: 700; }
.document-navigation input { width: 100%; margin: .25rem 0 .75rem; padding: .4rem; border: 1px solid #8d897e; font: inherit; }
.document-search button { padding: .35rem .65rem; border: 1px solid #1d2826; background: #1d2826; color: #fffdf8; font: inherit; }
.document-navigation ul { margin: 0; padding: 0; list-style: none; }
.document-navigation li + li { margin-top: .35rem; }
.document-navigation a { color: #1d2826; overflow-wrap: anywhere; }
.document-navigation a[aria-current="page"] { color: #8b3f21; font-weight: 800; text-decoration-thickness: .18em; }
.document-navigation [role="status"] { margin: .75rem 0; color: #8b3f21; font-size: .875rem; }
.document-result-pages { display: flex; gap: .75rem; margin: .75rem 0 0; }
.document-content { grid-column: 2; grid-row: 1 / span 2; min-width: 0; }
main[data-document-navigation-collapsed="true"] { grid-template-columns: auto minmax(0, 1fr); }
header { border-bottom: 3px solid #1d2826; margin-bottom: 2rem; }
h1 { font-size: clamp(2rem, 5vw, 3.5rem); line-height: 1.05; margin: 0 0 1.2rem; overflow-wrap: anywhere; }
.eyebrow { color: #8b3f21; font-family: system-ui, sans-serif; font-size: .75rem; font-weight: 800; letter-spacing: .14em; margin: 0 0 .4rem; text-transform: uppercase; }
article > :first-child { margin-top: 0; }
pre { overflow: auto; padding: 1rem; background: #e5e0d7; }
code { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }
.diagram { margin: 1.5rem 0; padding: 1rem; border: 1px solid #b6b0a4; background: #fffdf8; }
.diagram img { display: block; width: 100%; height: auto; }
.diagram-error { color: #9c2f19; font-family: system-ui, sans-serif; font-weight: 700; }
.diagram-disabled { color: #555147; font-family: system-ui, sans-serif; font-weight: 700; }
.diagram button { margin-top: .75rem; }
.diagram-source { margin-top: .75rem; }
.renderer-controls { margin: 0 0 1.5rem; padding: .75rem 1rem; border-left: 4px solid #8b3f21; background: #fffdf8; font-family: system-ui, sans-serif; }
.renderer-controls p { margin: 0; font-weight: 700; }
.renderer-controls button { margin-top: .65rem; }
.document-metadata { margin: 0 0 1.5rem; padding: 1rem; border: 1px solid #b6b0a4; background: #fffdf8; font-family: system-ui, sans-serif; }
.document-metadata h2 { margin: 0 0 .75rem; font-size: 1rem; }
.document-metadata dl { display: grid; gap: .6rem; margin: 0; }
.document-metadata dl div { display: grid; gap: .15rem; }
.document-metadata dt { color: #5c5a54; font-size: .8rem; font-weight: 800; }
.document-metadata dd { margin: 0; overflow-wrap: anywhere; }
.document-metadata ul { margin: .25rem 0 0; padding-left: 1.2rem; }
.document-metadata dl dl { margin: .25rem 0 0; padding-left: .75rem; border-left: 2px solid #b6b0a4; }
.frontmatter-error { margin: 0 0 1.5rem; padding: .75rem 1rem; border-left: 4px solid #8b3f21; background: #fff4ed; font-family: system-ui, sans-serif; }
.frontmatter-error p { margin: .35rem 0; }
@media (max-width: 760px) { main { width: min(100% - 1rem, 920px); margin-top: 1.5rem; display: block; } .document-navigation-control { margin-bottom: 1rem; } .document-navigation { position: static; margin-bottom: 1.5rem; } .diagram { padding: .5rem; } }"#;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use super::{
        deferred_navigation_page, page, renderer_client, router, viewer_state, NavigationRequest,
    };
    use crate::{
        plantuml::{DiagramRenderer, RendererMode},
        target::{DocumentKind, MarkdownDocument},
    };

    fn test_renderer() -> DiagramRenderer {
        DiagramRenderer::from_mode(RendererMode::Public)
    }

    fn test_router() -> axum::Router {
        test_router_with_documents(vec![test_document("README.md", "# Lens")], 0)
    }

    fn test_router_with_documents(
        documents: Vec<MarkdownDocument>,
        initial_document: usize,
    ) -> axum::Router {
        router(viewer_state(
            documents,
            initial_document,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        ))
    }

    fn test_document(identifier: &str, source: &str) -> MarkdownDocument {
        MarkdownDocument {
            identifier: identifier.to_owned(),
            canonical_path: PathBuf::from(identifier),
            source: source.to_owned(),
            kind: DocumentKind::Markdown,
        }
    }

    #[test]
    fn document_navigation_pane_then_lists_known_documents_and_marks_current() {
        // Arrange
        let state = viewer_state(
            vec![
                test_document("README.md", "# Read me"),
                test_document("guides/intro.md", "# Introduction"),
            ],
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );

        // Act
        let request = NavigationRequest::from_raw_query(None);
        let navigation = state.navigation_pane(1, &request, "/documents/guides/intro.md");

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
        let state = viewer_state(
            vec![test_document("README.md", "# Read me")],
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );
        let request = NavigationRequest::from_raw_query(None);
        let navigation = state.navigation_pane(0, &request, "/");

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
        let state = viewer_state(
            vec![test_document("guides/<unsafe>.md", "# Guide")],
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );

        // Act
        let request = NavigationRequest::from_raw_query(None);
        let navigation = state.navigation_pane(0, &request, "/");

        // Assert
        assert!(navigation.contains("/documents/guides/&lt;unsafe&gt;.md"));
        assert!(!navigation.contains("/documents/guides/<unsafe>.md"));
    }

    #[test]
    fn enabled_renderer_then_exposes_its_status_and_disable_control() {
        // Arrange
        let state = viewer_state(
            vec![test_document("README.md", "# Read me")],
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );

        // Act
        let controls = state.renderer_controls();

        // Assert
        assert!(controls.contains("Diagram renderer: public."));
        assert!(controls.contains("data-disable-renderer"));
    }

    #[test]
    fn more_than_result_limit_then_shows_first_page_and_next_link() {
        // Arrange
        let documents = (0..=50)
            .map(|index| test_document(&format!("guides/{index:03}.md"), "# Guide"))
            .collect();
        let state = viewer_state(
            documents,
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );

        // Act
        let request = NavigationRequest::from_raw_query(None);
        let navigation = state.navigation_pane(0, &request, "/");

        // Assert
        assert_eq!(
            navigation.matches("data-document-navigation-item").count(),
            50
        );
        assert!(navigation.contains("Next results"));
        assert!(!navigation.contains("guides/050.md"));
    }

    #[tokio::test]
    async fn unknown_document_path_then_returns_not_found() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .uri("/documents/../../etc/passwd")
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_document_revision_path_then_returns_not_found() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .uri("/revisions/.private.md")
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn discovered_document_path_then_returns_document() {
        // Arrange
        let app = test_router_with_documents(
            vec![
                test_document("README.md", "# Read me"),
                test_document("guides/intro.md", "# Introduction"),
            ],
            0,
        );
        let request = Request::builder()
            .uri("/documents/guides/intro.md")
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::OK);
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

    #[tokio::test]
    async fn unknown_diagram_then_returns_not_found() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .uri("/diagrams/99/0")
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn renderer_disable_request_then_blocks_diagram_rendering_for_the_session() {
        // Arrange
        let state = viewer_state(
            vec![test_document(
                "README.md",
                "```plantuml\n@startuml\nAlice -> Bob\n@enduml\n```",
            )],
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );
        let disable_request = Request::builder()
            .method("POST")
            .uri("/renderer/disable")
            .body(Body::empty())
            .expect("disable request should build");

        // Act
        let disable_response = router(state.clone())
            .oneshot(disable_request)
            .await
            .expect("router should respond");
        let diagram_request = Request::builder()
            .uri("/diagrams/0/0")
            .body(Body::empty())
            .expect("diagram request should build");
        let diagram_response = router(state)
            .oneshot(diagram_request)
            .await
            .expect("router should respond");

        // Assert
        assert_eq!(
            disable_response.status(),
            axum::http::StatusCode::NO_CONTENT
        );
        assert_eq!(
            diagram_response.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn document_request_then_sets_restrictive_content_security_policy() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-security-policy")
                .expect("CSP should be set"),
            "default-src 'self'; base-uri 'none'; img-src 'self'; object-src 'none'; script-src 'self'; style-src 'self'"
        );
    }
}
