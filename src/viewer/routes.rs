use std::sync::Arc;

use axum::{
    extract::{Path, RawQuery, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};

use super::{
    catalog::NavigationRequest,
    page::{
        app_script, app_stylesheet, content_security_policy, deferred_navigation_page,
        navigation_pane, page, renderer_controls,
    },
    rendering::request_diagram,
    state::ViewerState,
};

pub(super) fn router(state: Arc<ViewerState>) -> Router {
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
    let navigation = navigation_pane(state.catalog.search(request), document_id, current_route);
    let renderer_controls = renderer_controls(state.renderer.label(), state.rendering_enabled());
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
        app_stylesheet(),
    )
}

async fn script() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        app_script(),
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use super::router;
    use crate::{
        plantuml::{DiagramRenderer, RendererMode},
        target::{DocumentKind, MarkdownDocument},
        viewer::{rendering::renderer_client, state::viewer_state},
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
