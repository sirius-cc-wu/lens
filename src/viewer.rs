use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    net::TcpListener,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use futures_util::StreamExt;
use reqwest::Client;
use tokio::{io::AsyncWriteExt, process::Command as TokioCommand};

use crate::{
    markdown::{escape_html, render, render_standalone_plantuml, Diagram, RenderedDocument},
    plantuml::{DiagramRenderer, RendererMode},
    target::{DocumentKind, MarkdownDocument, MarkdownTarget},
};

const MAX_DIAGRAM_BYTES: usize = 2 * 1024 * 1024;
const RENDER_TIMEOUT: Duration = Duration::from_secs(10);
const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

struct ViewerState {
    documents: RwLock<Vec<ViewerDocument>>,
    document_ids: BTreeMap<String, usize>,
    known_documents: BTreeSet<String>,
    initial_document: usize,
    client: Client,
    renderer: DiagramRenderer,
    rendering_disabled: AtomicBool,
}

#[allow(dead_code)] // Each variant is constructed by its supported target build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BrowserPlatform {
    Linux,
    MacOs,
    Windows,
}

#[derive(Debug, Eq, PartialEq)]
struct BrowserCommand {
    program: &'static str,
    arguments: Vec<String>,
}

impl ViewerState {
    fn navigation_pane(&self, current_document: usize) -> String {
        let mut document_links = String::new();
        for (identifier, &document_index) in &self.document_ids {
            let current = (document_index == current_document)
                .then_some(r#" aria-current="page""#)
                .unwrap_or_default();
            let identifier = escape_html(identifier);
            write!(
                document_links,
                r#"<li data-document-navigation-item><a href="/documents/{identifier}"{current}>{identifier}</a></li>"#
            )
            .expect("writing navigation markup to a string cannot fail");
        }

        format!(
            r#"<nav class="document-navigation" aria-label="Discovered documents" data-document-navigation><h2>Documents</h2><label for="document-filter">Filter discovered documents</label><input id="document-filter" type="search" data-document-filter aria-controls="document-catalog"><noscript><p>Filtering requires JavaScript; all discovered documents are shown.</p></noscript><p data-document-filter-empty role="status" hidden>No discovered documents match the filter.</p><ul id="document-catalog">{document_links}</ul></nav>"#
        )
    }

    fn rendering_enabled(&self) -> bool {
        self.renderer.is_enabled() && !self.rendering_disabled.load(Ordering::Acquire)
    }

    fn disable_rendering(&self) {
        self.rendering_disabled.store(true, Ordering::Release);
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

    fn document_revision(&self, document_id: usize) -> Option<u64> {
        self.documents
            .read()
            .expect("viewer documents lock should not be poisoned")
            .get(document_id)
            .map(|document| document.revision)
    }

    fn refresh_known_documents(&self) {
        let documents = self
            .documents
            .read()
            .expect("viewer documents lock should not be poisoned")
            .iter()
            .enumerate()
            .map(|(document_id, document)| {
                (
                    document_id,
                    document.identifier.clone(),
                    document.canonical_path.clone(),
                    document.source.clone(),
                    document.kind,
                )
            })
            .collect::<Vec<_>>();

        for (document_id, identifier, canonical_path, stored_source, kind) in documents {
            let Ok(source) = fs::read_to_string(canonical_path) else {
                continue;
            };
            if source == stored_source {
                continue;
            }

            let rendered = render_document(
                &source,
                document_id,
                &identifier,
                kind,
                &self.known_documents,
                &self.renderer,
            );
            let mut documents = self
                .documents
                .write()
                .expect("viewer documents lock should not be poisoned");
            let document = &mut documents[document_id];
            if document.source == stored_source {
                document.replace(source, rendered);
            }
        }
    }
}

struct ViewerDocument {
    identifier: String,
    canonical_path: PathBuf,
    source: String,
    kind: DocumentKind,
    rendered: RenderedDocument,
    revision: u64,
}

impl ViewerDocument {
    fn replace(&mut self, source: String, rendered: RenderedDocument) {
        self.source = source;
        self.rendered = rendered;
        self.revision += 1;
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

fn open_browser(url: &str) -> std::io::Result<()> {
    let command = browser_command(current_browser_platform()?, url);
    Command::new(command.program)
        .args(command.arguments)
        .spawn()
        .map(|_| ())
}

fn browser_command(platform: BrowserPlatform, url: &str) -> BrowserCommand {
    match platform {
        BrowserPlatform::Linux => BrowserCommand {
            program: "xdg-open",
            arguments: vec![url.to_owned()],
        },
        BrowserPlatform::MacOs => BrowserCommand {
            program: "open",
            arguments: vec![url.to_owned()],
        },
        BrowserPlatform::Windows => BrowserCommand {
            program: "cmd",
            arguments: vec![
                "/C".to_owned(),
                "start".to_owned(),
                String::new(),
                url.to_owned(),
            ],
        },
    }
}

#[cfg(target_os = "linux")]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Ok(BrowserPlatform::Linux)
}

#[cfg(target_os = "macos")]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Ok(BrowserPlatform::MacOs)
}

#[cfg(target_os = "windows")]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Ok(BrowserPlatform::Windows)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn current_browser_platform() -> std::io::Result<BrowserPlatform> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Lens does not support automatic browser launch on this platform",
    ))
}

fn viewer_state(
    documents: Vec<MarkdownDocument>,
    initial_document: usize,
    client: Client,
    renderer: DiagramRenderer,
) -> Arc<ViewerState> {
    let document_ids = documents
        .iter()
        .enumerate()
        .map(|(index, document)| (document.identifier.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let known_documents = document_ids.keys().cloned().collect::<BTreeSet<_>>();
    let documents = documents
        .into_iter()
        .enumerate()
        .map(|(document_id, document)| ViewerDocument {
            identifier: document.identifier.clone(),
            canonical_path: document.canonical_path,
            source: document.source.clone(),
            kind: document.kind,
            rendered: render_document(
                &document.source,
                document_id,
                &document.identifier,
                document.kind,
                &known_documents,
                &renderer,
            ),
            revision: 0,
        })
        .collect();

    Arc::new(ViewerState {
        documents: RwLock::new(documents),
        document_ids,
        known_documents,
        initial_document,
        client,
        renderer,
        rendering_disabled: AtomicBool::new(false),
    })
}

fn render_document(
    source: &str,
    document_id: usize,
    identifier: &str,
    kind: DocumentKind,
    known_documents: &BTreeSet<String>,
    renderer: &DiagramRenderer,
) -> RenderedDocument {
    match kind {
        DocumentKind::Markdown => {
            render(source, document_id, identifier, known_documents, renderer)
        }
        DocumentKind::PlantUml => render_standalone_plantuml(document_id, source, renderer),
    }
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

async fn initial_document_view(State(state): State<Arc<ViewerState>>) -> Response {
    rendered_document_response(&state, state.initial_document)
}

async fn document_view(
    State(state): State<Arc<ViewerState>>,
    Path(document_id): Path<String>,
) -> Response {
    let document_id = document_id.trim_start_matches('/');
    match state.document_ids.get(document_id) {
        Some(&document_id) => rendered_document_response(&state, document_id),
        None => not_found().await.into_response(),
    }
}

async fn document_revision(
    State(state): State<Arc<ViewerState>>,
    Path(document_id): Path<String>,
) -> Response {
    let document_id = document_id.trim_start_matches('/');
    match state.document_ids.get(document_id) {
        Some(&document_id) => (
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

fn rendered_document_response(state: &ViewerState, document_id: usize) -> Response {
    let navigation = state.navigation_pane(document_id);
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

async fn request_diagram(
    renderer: &DiagramRenderer,
    client: &Client,
    diagram: &Diagram,
) -> Result<Vec<u8>> {
    match renderer {
        DiagramRenderer::Public { server } => request_public_diagram(client, server, diagram).await,
        DiagramRenderer::Local { command } => request_local_diagram(command, diagram).await,
        DiagramRenderer::Disabled => {
            anyhow::bail!("PlantUML rendering is disabled for this viewing session")
        }
    }
}

async fn request_public_diagram(
    client: &Client,
    server: &str,
    diagram: &Diagram,
) -> Result<Vec<u8>> {
    let response = client
        .get(crate::plantuml::svg_url(server, &diagram.source))
        .send()
        .await
        .context("Could not contact the public PlantUML server")?;
    if !response.status().is_success() {
        anyhow::bail!("The public PlantUML server returned {}", response.status());
    }
    if response.headers().contains_key("x-plantuml-diagram-error") {
        anyhow::bail!("The public PlantUML server reported an invalid diagram");
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !content_type.starts_with("image/svg+xml") {
        anyhow::bail!("The public PlantUML server did not return SVG content");
    }
    if response
        .content_length()
        .is_some_and(|length| length as usize > MAX_DIAGRAM_BYTES)
    {
        anyhow::bail!("The public PlantUML server returned an oversized diagram");
    }

    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Could not read the PlantUML response")?;
        if bytes.len() + chunk.len() > MAX_DIAGRAM_BYTES {
            anyhow::bail!("The public PlantUML server returned an oversized diagram");
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn request_local_diagram(command: &std::path::Path, diagram: &Diagram) -> Result<Vec<u8>> {
    let mut command_builder = TokioCommand::new(command);
    command_builder
        .args(["-tsvg", "-pipe"])
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command_builder.spawn().with_context(|| {
        format!(
            "Could not start the local PlantUML command {}",
            command.display()
        )
    })?;
    let mut input = child
        .stdin
        .take()
        .expect("piped PlantUML command should provide standard input");
    input
        .write_all(diagram.source.as_bytes())
        .await
        .context("Could not write PlantUML source to the local renderer")?;
    drop(input);

    let output = tokio::time::timeout(RENDER_TIMEOUT, child.wait_with_output())
        .await
        .context("The local PlantUML command timed out")?
        .context("Could not wait for the local PlantUML command")?;
    if !output.status.success() {
        anyhow::bail!("The local PlantUML command exited with {}", output.status);
    }
    if output.stdout.len() > MAX_DIAGRAM_BYTES {
        anyhow::bail!("The local PlantUML command returned an oversized diagram");
    }
    if !output.stdout.windows(4).any(|bytes| bytes == b"<svg") {
        anyhow::bail!("The local PlantUML command did not return SVG content");
    }
    Ok(output.stdout)
}

fn renderer_client() -> Result<Client> {
    renderer_client_with_timeout(RENDER_TIMEOUT)
}

fn renderer_client_with_timeout(timeout: Duration) -> Result<Client> {
    Client::builder()
        .timeout(timeout)
        .build()
        .context("Could not configure the PlantUML client")
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Could not listen for Ctrl-C: {error}");
    }
}

async fn watch_documents(state: Arc<ViewerState>) {
    let mut interval = tokio::time::interval(REFRESH_INTERVAL);
    loop {
        interval.tick().await;
        state.refresh_known_documents();
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

for (const navigation of document.querySelectorAll('[data-document-navigation]')) {
  const filter = navigation.querySelector('[data-document-filter]');
  const empty = navigation.querySelector('[data-document-filter-empty]');
  const entries = navigation.querySelectorAll('[data-document-navigation-item]');
  if (!filter || !empty) continue;

  filter.addEventListener('input', () => {
    const query = filter.value.trim().toLocaleLowerCase();
    let visibleEntries = 0;
    for (const entry of entries) {
      const matches = entry.textContent.toLocaleLowerCase().includes(query);
      entry.hidden = !matches;
      if (matches) visibleEntries += 1;
    }
    empty.hidden = visibleEntries !== 0;
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
.document-navigation { position: sticky; top: 1rem; padding: 1rem; border: 1px solid #b6b0a4; background: #fffdf8; font-family: system-ui, sans-serif; }
.document-navigation h2 { margin: 0 0 .75rem; font-size: 1rem; }
.document-navigation label { display: block; font-size: .8rem; font-weight: 700; }
.document-navigation input { width: 100%; margin: .25rem 0 .75rem; padding: .4rem; border: 1px solid #8d897e; font: inherit; }
.document-navigation ul { margin: 0; padding: 0; list-style: none; }
.document-navigation li + li { margin-top: .35rem; }
.document-navigation a { color: #1d2826; overflow-wrap: anywhere; }
.document-navigation a[aria-current="page"] { color: #8b3f21; font-weight: 800; text-decoration-thickness: .18em; }
.document-navigation [role="status"] { margin: .75rem 0; color: #8b3f21; font-size: .875rem; }
.document-content { min-width: 0; }
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
@media (max-width: 760px) { main { width: min(100% - 1rem, 920px); margin-top: 1.5rem; display: block; } .document-navigation { position: static; margin-bottom: 1.5rem; } .diagram { padding: .5rem; } }"#;

#[cfg(test)]
mod tests {
    use std::{fs, net::TcpListener, path::PathBuf, time::Duration};

    use axum::{
        body::Body,
        http::{header, Request},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use super::{
        browser_command, deferred_navigation_page, renderer_client, renderer_client_with_timeout,
        request_diagram, router, viewer_state, BrowserCommand, BrowserPlatform,
    };
    use crate::{
        markdown::Diagram,
        plantuml::{DiagramRenderer, RendererMode},
        target::{DocumentKind, MarkdownDocument},
    };

    fn test_renderer() -> DiagramRenderer {
        DiagramRenderer::from_mode(RendererMode::Public)
    }

    #[test]
    fn supported_browser_platform_then_uses_its_launch_command() {
        // Arrange
        let url = "http://127.0.0.1:4567";

        // Act
        let commands = [
            (
                BrowserPlatform::Linux,
                browser_command(BrowserPlatform::Linux, url),
            ),
            (
                BrowserPlatform::MacOs,
                browser_command(BrowserPlatform::MacOs, url),
            ),
            (
                BrowserPlatform::Windows,
                browser_command(BrowserPlatform::Windows, url),
            ),
        ];

        // Assert
        assert_eq!(
            commands[0].1,
            BrowserCommand {
                program: "xdg-open",
                arguments: vec![url.to_owned()],
            }
        );
        assert_eq!(
            commands[1].1,
            BrowserCommand {
                program: "open",
                arguments: vec![url.to_owned()],
            }
        );
        assert_eq!(
            commands[2].1,
            BrowserCommand {
                program: "cmd",
                arguments: vec![
                    "/C".to_owned(),
                    "start".to_owned(),
                    String::new(),
                    url.to_owned()
                ],
            }
        );
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

    fn file_backed_test_document(path: PathBuf, source: &str) -> MarkdownDocument {
        MarkdownDocument {
            identifier: "README.md".to_owned(),
            canonical_path: path,
            source: source.to_owned(),
            kind: DocumentKind::Markdown,
        }
    }

    fn temporary_document_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("lens-viewer-{}-{name}.md", std::process::id()))
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
        let navigation = state.navigation_pane(1);

        // Assert
        assert!(navigation.contains("aria-label=\"Discovered documents\""));
        assert!(navigation.contains("href=\"/documents/README.md\""));
        assert!(navigation.contains("href=\"/documents/guides/intro.md\" aria-current=\"page\""));
        assert!(!navigation.contains(".private.md"));
        assert_eq!(navigation.matches("aria-current=\"page\"").count(), 1);
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
        let navigation = state.navigation_pane(0);

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
    fn changed_known_document_then_updates_rendering_and_revision() {
        // Arrange
        let path = temporary_document_path("changed-document");
        fs::write(&path, "# Before refresh").expect("test document should be writable");
        let state = viewer_state(
            vec![file_backed_test_document(path.clone(), "# Before refresh")],
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );
        fs::write(&path, "# After refresh\n\nChanged content.")
            .expect("test document should update");

        // Act
        state.refresh_known_documents();

        // Assert
        let revision = state.document_revision(0);
        let documents = state
            .documents
            .read()
            .expect("viewer documents lock should not be poisoned");
        assert_eq!(revision, Some(1));
        assert!(documents[0].rendered.html.contains("After refresh"));
        assert!(documents[0].rendered.html.contains("Changed content."));
        fs::remove_file(path).expect("test document should be removable");
    }

    #[test]
    fn unreadable_known_document_then_retains_last_rendering_and_revision() {
        // Arrange
        let path = temporary_document_path("unreadable-document");
        fs::write(&path, "# Readable document").expect("test document should be writable");
        let state = viewer_state(
            vec![file_backed_test_document(
                path.clone(),
                "# Readable document",
            )],
            0,
            renderer_client().expect("test client should initialize"),
            test_renderer(),
        );
        fs::remove_file(&path).expect("test document should be removable");

        // Act
        state.refresh_known_documents();

        // Assert
        let revision = state.document_revision(0);
        let documents = state
            .documents
            .read()
            .expect("viewer documents lock should not be poisoned");
        assert_eq!(revision, Some(0));
        assert!(documents[0].rendered.html.contains("Readable document"));
    }

    async fn mock_renderer_server(renderer: Router) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock renderer should bind");
        let address = listener
            .local_addr()
            .expect("mock renderer should have an address");
        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .expect("mock renderer should serve")
                .serve(renderer.into_make_service())
                .await
                .expect("mock renderer should not fail");
        });
        format!("http://{address}")
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

    #[tokio::test]
    async fn valid_svg_then_returns_public_renderer_response() {
        // Arrange
        let server = Router::new().route(
            "/svg/*encoded",
            get(|| async { ([(header::CONTENT_TYPE, "image/svg+xml")], "<svg></svg>") }),
        );
        let diagram = Diagram {
            source: "@startuml\n@enduml".to_owned(),
        };
        let renderer = DiagramRenderer::Public {
            server: mock_renderer_server(server).await,
        };

        // Act
        let response = request_diagram(
            &renderer,
            &renderer_client().expect("test client should initialize"),
            &diagram,
        )
        .await
        .expect("valid SVG should render");

        // Assert
        assert_eq!(response, b"<svg></svg>");
    }

    #[tokio::test]
    async fn renderer_error_header_then_returns_error() {
        // Arrange
        let server = Router::new().route(
            "/svg/*encoded",
            get(|| async {
                (
                    [
                        (header::CONTENT_TYPE, "image/svg+xml"),
                        (
                            header::HeaderName::from_static("x-plantuml-diagram-error"),
                            "Syntax Error?",
                        ),
                    ],
                    "<svg></svg>",
                )
            }),
        );
        let diagram = Diagram {
            source: "@startuml\n@enduml".to_owned(),
        };
        let renderer = DiagramRenderer::Public {
            server: mock_renderer_server(server).await,
        };

        // Act
        let result = request_diagram(
            &renderer,
            &renderer_client().expect("test client should initialize"),
            &diagram,
        )
        .await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn unavailable_renderer_then_returns_error() {
        // Arrange
        let server = Router::new().route(
            "/svg/*encoded",
            get(|| async { (axum::http::StatusCode::SERVICE_UNAVAILABLE, "unavailable") }),
        );
        let diagram = Diagram {
            source: "@startuml\n@enduml".to_owned(),
        };
        let renderer = DiagramRenderer::Public {
            server: mock_renderer_server(server).await,
        };

        // Act
        let result = request_diagram(
            &renderer,
            &renderer_client().expect("test client should initialize"),
            &diagram,
        )
        .await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delayed_renderer_then_times_out() {
        // Arrange
        let server = Router::new().route(
            "/svg/*encoded",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                ([(header::CONTENT_TYPE, "image/svg+xml")], "<svg></svg>")
            }),
        );
        let diagram = Diagram {
            source: "@startuml\n@enduml".to_owned(),
        };
        let renderer = DiagramRenderer::Public {
            server: mock_renderer_server(server).await,
        };
        let client = renderer_client_with_timeout(Duration::from_millis(10))
            .expect("test client should initialize");

        // Act
        let result = request_diagram(&renderer, &client, &diagram).await;

        // Assert
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn local_renderer_command_then_returns_its_svg_output() {
        use std::{
            os::unix::fs::PermissionsExt,
            time::{SystemTime, UNIX_EPOCH},
        };

        // Arrange
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the Unix epoch")
            .as_nanos();
        let command = std::env::temp_dir().join(format!(
            "lens-local-renderer-{}-{timestamp}",
            std::process::id()
        ));
        fs::write(
            &command,
            "#!/bin/sh\n[ \"$(cat)\" = '@startuml\n@enduml' ] || exit 1\nprintf '%s' '<svg></svg>'\n",
        )
        .expect("local renderer command should be writable");
        fs::set_permissions(&command, fs::Permissions::from_mode(0o755))
            .expect("local renderer command should be executable");
        let renderer = DiagramRenderer::local_with_command(command.clone());
        let diagram = Diagram {
            source: "@startuml\n@enduml".to_owned(),
        };

        // Act
        let result = request_diagram(
            &renderer,
            &renderer_client().expect("test client should initialize"),
            &diagram,
        )
        .await;

        // Assert
        assert_eq!(
            result.expect("local renderer should produce SVG"),
            b"<svg></svg>"
        );
        fs::remove_file(command).expect("local renderer command should be removable");
    }
}
