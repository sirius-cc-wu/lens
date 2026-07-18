use std::{
    collections::BTreeMap, net::TcpListener, path::PathBuf, process::Command, sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::StreamExt;
use reqwest::Client;

use crate::{
    markdown::{escape_html, render, Diagram, RenderedDocument},
    plantuml::renderer_server,
    target::{MarkdownDocument, MarkdownTarget},
};

const MAX_DIAGRAM_BYTES: usize = 2 * 1024 * 1024;
const RENDER_TIMEOUT: Duration = Duration::from_secs(10);

struct ViewerState {
    documents: Vec<ViewerDocument>,
    document_ids: BTreeMap<String, usize>,
    initial_document: usize,
    client: Client,
}

struct ViewerDocument {
    canonical_path: PathBuf,
    rendered: RenderedDocument,
}

pub async fn serve(target: MarkdownTarget) -> Result<()> {
    let (documents, initial_document) = target.into_parts();
    let initial_path = documents[initial_document].canonical_path.clone();
    let diagram_renderer = renderer_server();
    let state = viewer_state(
        documents,
        initial_document,
        renderer_client()?,
        &diagram_renderer,
    );
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

#[cfg(target_os = "linux")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("open").arg(url).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
}

fn viewer_state(
    documents: Vec<MarkdownDocument>,
    initial_document: usize,
    client: Client,
    renderer_server: &str,
) -> Arc<ViewerState> {
    let document_ids = documents
        .iter()
        .enumerate()
        .map(|(index, document)| (document.identifier.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let known_documents = document_ids.keys().cloned().collect();
    let documents = documents
        .into_iter()
        .enumerate()
        .map(|(document_id, document)| ViewerDocument {
            canonical_path: document.canonical_path,
            rendered: render(
                &document.source,
                document_id,
                &document.identifier,
                &known_documents,
                renderer_server,
            ),
        })
        .collect();

    Arc::new(ViewerState {
        documents,
        document_ids,
        initial_document,
        client,
    })
}

fn router(state: Arc<ViewerState>) -> Router {
    Router::new()
        .route("/", get(initial_document_view))
        .route("/documents/*document_id", get(document_view))
        .route("/app.css", get(stylesheet))
        .route("/app.js", get(script))
        .route("/diagrams/:document_id/:diagram_id", get(diagram))
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

fn rendered_document_response(state: &ViewerState, document_id: usize) -> Response {
    let document = &state.documents[document_id];
    (
        [(header::CONTENT_SECURITY_POLICY, content_security_policy())],
        Html(page(
            &document.canonical_path.display().to_string(),
            document.rendered.html.clone(),
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
    let Some(document) = state.documents.get(document_id) else {
        return (StatusCode::NOT_FOUND, "Diagram not found").into_response();
    };
    let Some(diagram) = document.rendered.diagrams.get(diagram_id) else {
        return (StatusCode::NOT_FOUND, "Diagram not found").into_response();
    };

    match request_diagram(&state.client, diagram).await {
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

async fn request_diagram(client: &Client, diagram: &Diagram) -> Result<Vec<u8>> {
    let response = client
        .get(&diagram.url)
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

fn page(title: &str, document_html: String) -> String {
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
  <main>
    <header><p class="eyebrow">Lens</p><h1>{}</h1></header>
    <article>{document_html}</article>
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
    )
}

fn content_security_policy() -> &'static str {
    "default-src 'self'; base-uri 'none'; img-src 'self'; object-src 'none'; script-src 'self'; style-src 'self'"
}

const APP_SCRIPT: &str = r#"for (const image of document.querySelectorAll('[data-diagram]')) {
  const revealFailure = () => {
    const figure = image.closest('.diagram');
    figure.querySelector('.diagram-error').hidden = false;
    figure.querySelector('.diagram-source').open = true;
  };
  image.addEventListener('error', revealFailure);
  if (image.complete && image.naturalWidth === 0) {
    revealFailure();
  }
}"#;

const APP_STYLESHEET: &str = r#"* { box-sizing: border-box; }
body { margin: 0; background: #f4f1ea; color: #1d2826; font-family: Georgia, serif; line-height: 1.55; }
main { width: min(920px, calc(100% - 2rem)); margin: 3rem auto 5rem; }
header { border-bottom: 3px solid #1d2826; margin-bottom: 2rem; }
h1 { font-size: clamp(2rem, 5vw, 3.5rem); line-height: 1.05; margin: 0 0 1.2rem; overflow-wrap: anywhere; }
.eyebrow { color: #8b3f21; font-family: system-ui, sans-serif; font-size: .75rem; font-weight: 800; letter-spacing: .14em; margin: 0 0 .4rem; text-transform: uppercase; }
article > :first-child { margin-top: 0; }
pre { overflow: auto; padding: 1rem; background: #e5e0d7; }
code { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }
.diagram { margin: 1.5rem 0; padding: 1rem; border: 1px solid #b6b0a4; background: #fffdf8; }
.diagram img { display: block; width: 100%; height: auto; }
.diagram-error { color: #9c2f19; font-family: system-ui, sans-serif; font-weight: 700; }
.diagram-source { margin-top: .75rem; }
@media (max-width: 600px) { main { width: min(100% - 1rem, 920px); margin-top: 1.5rem; } .diagram { padding: .5rem; } }"#;

#[cfg(test)]
mod tests {
    use std::{net::TcpListener, path::PathBuf, time::Duration};

    use axum::{
        body::Body,
        http::{header, Request},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use super::{
        deferred_navigation_page, renderer_client, renderer_client_with_timeout, request_diagram,
        router, viewer_state,
    };
    use crate::{markdown::Diagram, plantuml::PUBLIC_SERVER, target::MarkdownDocument};

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
            PUBLIC_SERVER,
        ))
    }

    fn test_document(identifier: &str, source: &str) -> MarkdownDocument {
        MarkdownDocument {
            identifier: identifier.to_owned(),
            canonical_path: PathBuf::from(identifier),
            source: source.to_owned(),
        }
    }

    async fn mock_renderer_url(renderer: Router) -> String {
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
        format!("http://{address}/svg")
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
        let renderer = Router::new().route(
            "/svg",
            get(|| async { ([(header::CONTENT_TYPE, "image/svg+xml")], "<svg></svg>") }),
        );
        let diagram = Diagram {
            url: mock_renderer_url(renderer).await,
        };

        // Act
        let response = request_diagram(
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
        let renderer = Router::new().route(
            "/svg",
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
            url: mock_renderer_url(renderer).await,
        };

        // Act
        let result = request_diagram(
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
        let renderer = Router::new().route(
            "/svg",
            get(|| async { (axum::http::StatusCode::SERVICE_UNAVAILABLE, "unavailable") }),
        );
        let diagram = Diagram {
            url: mock_renderer_url(renderer).await,
        };

        // Act
        let result = request_diagram(
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
        let renderer = Router::new().route(
            "/svg",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                ([(header::CONTENT_TYPE, "image/svg+xml")], "<svg></svg>")
            }),
        );
        let diagram = Diagram {
            url: mock_renderer_url(renderer).await,
        };
        let client = renderer_client_with_timeout(Duration::from_millis(10))
            .expect("test client should initialize");

        // Act
        let result = request_diagram(&client, &diagram).await;

        // Assert
        assert!(result.is_err());
    }
}
