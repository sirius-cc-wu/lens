use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

use super::{
    page::{app_script, app_stylesheet, content_security_policy, document_unavailable_page, page},
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
        .fallback(not_found)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_auth_middleware,
        ))
        .with_state(state)
}

async fn session_auth_middleware<B>(
    State(state): State<Arc<ViewerState>>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let query_token = request.uri().query().and_then(|query| {
        query.split('&').find_map(|param| {
            let mut parts = param.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some("token"), Some(val)) => Some(val),
                _ => None,
            }
        })
    });

    if let Some(token) = query_token {
        if token == state.session_token {
            let mut response = next.run(request).await;
            response.headers_mut().insert(
                header::REFERRER_POLICY,
                HeaderValue::from_static("no-referrer"),
            );
            return response;
        }
    }

    StatusCode::UNAUTHORIZED.into_response()
}

async fn initial_document_view(State(state): State<Arc<ViewerState>>) -> Response {
    rendered_document_response(&state, state.initial_document)
}

async fn document_view(
    State(state): State<Arc<ViewerState>>,
    Path(document_id): Path<String>,
) -> Response {
    let document_id = document_id.trim_start_matches('/');
    match state.known_documents.index(document_id) {
        Some(known_document) => rendered_document_response(&state, known_document),
        None => not_found(State(state)).await.into_response(),
    }
}

async fn document_revision(
    State(state): State<Arc<ViewerState>>,
    Path(document_id): Path<String>,
) -> Response {
    let document_id = document_id.trim_start_matches('/');
    match state.known_documents.index(document_id) {
        Some(document_id) => (
            [(header::CACHE_CONTROL, "no-store")],
            state
                .document_revision(document_id)
                .expect("known document index should remain valid")
                .to_string(),
        )
            .into_response(),
        None => not_found(State(state)).await.into_response(),
    }
}

fn rendered_document_response(state: &ViewerState, document_id: usize) -> Response {
    let documents = state
        .documents
        .read()
        .expect("viewer documents lock should not be poisoned");
    let document = &documents[document_id];
    (
        [(header::CONTENT_SECURITY_POLICY, content_security_policy())],
        Html(page(
            &document.identifier,
            document.rendered.html.clone(),
            Some((&document.identifier, document.revision)),
            &state.session_token,
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

async fn not_found(State(state): State<Arc<ViewerState>>) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_SECURITY_POLICY, content_security_policy())],
        Html(document_unavailable_page(&state.session_token)),
    )
}

async fn diagram(
    State(state): State<Arc<ViewerState>>,
    Path((document_id, diagram_id)): Path<(usize, usize)>,
) -> Response {
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

    match request_diagram(&state.client, &state.plantuml_server, &diagram).await {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use axum::{
        body::Body,
        http::{header, Request},
    };
    use tower::ServiceExt;

    use super::router;
    use crate::{
        plantuml::PUBLIC_SERVER,
        target::{DocumentKind, MarkdownDocument},
        viewer::{rendering::renderer_client, state::viewer_state},
    };

    const TEST_TOKEN: &str = "test-session-token";

    fn test_server() -> String {
        PUBLIC_SERVER.to_owned()
    }

    fn test_router() -> axum::Router {
        test_router_with_documents(vec![test_document("README.md", "# Lens")], 0)
    }

    fn test_router_with_documents(
        documents: Vec<MarkdownDocument>,
        initial_document: usize,
    ) -> axum::Router {
        router(viewer_state(
            std::env::current_dir().expect("test root should be available"),
            documents,
            initial_document,
            renderer_client().expect("test client should initialize"),
            test_server(),
            TEST_TOKEN.to_owned(),
        ))
    }

    fn authed_request(uri: &str) -> Request<Body> {
        let authed_uri = if uri.contains('?') {
            format!("{uri}&token={TEST_TOKEN}")
        } else {
            format!("{uri}?token={TEST_TOKEN}")
        };
        Request::builder()
            .uri(authed_uri)
            .body(Body::empty())
            .expect("test request should build")
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
    async fn unauthenticated_request_without_token_then_returns_unauthorized() {
        // Arrange
        let app = test_router();

        // Act & Assert
        for path in [
            "/",
            "/documents/README.md",
            "/revisions/README.md",
            "/app.css",
            "/app.js",
            "/diagrams/0/0",
        ] {
            let request = Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("test request should build");
            let response = app
                .clone()
                .oneshot(request)
                .await
                .expect("router should respond");
            assert_eq!(
                response.status(),
                axum::http::StatusCode::UNAUTHORIZED,
                "unauthenticated request to {path} should be rejected"
            );
        }
    }

    #[tokio::test]
    async fn request_with_token_query_then_returns_ok_with_no_referrer_and_no_cookie() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .uri(format!("/?token={TEST_TOKEN}"))
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert!(response.headers().get(header::SET_COOKIE).is_none());
        assert_eq!(
            response
                .headers()
                .get(header::REFERRER_POLICY)
                .expect("referrer policy should be set"),
            "no-referrer"
        );
    }

    #[tokio::test]
    async fn request_with_mismatched_token_then_returns_unauthorized() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .uri("/?token=wrong-token")
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn request_with_cookie_only_then_returns_unauthorized() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, format!("lens-session={TEST_TOKEN}"))
            .body(Body::empty())
            .expect("test request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn unknown_document_path_then_returns_not_found() {
        // Arrange
        let app = test_router();
        let request = authed_request("/documents/../../etc/passwd");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_document_revision_path_then_returns_not_found() {
        // Arrange
        let app = test_router();
        let request = authed_request("/revisions/.private.md");

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
        let request = authed_request("/documents/guides/intro.md");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_diagram_then_returns_not_found() {
        // Arrange
        let app = test_router();
        let request = authed_request("/diagrams/99/0");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn renderer_disable_request_then_returns_not_found() {
        // Arrange
        let app = test_router();
        let request = Request::builder()
            .method("POST")
            .uri(format!("/renderer/disable?token={TEST_TOKEN}"))
            .body(Body::empty())
            .expect("disable request should build");

        // Act
        let response = app.oneshot(request).await.expect("router should respond");

        // Assert
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn document_request_then_sets_restrictive_content_security_policy() {
        // Arrange
        let app = test_router();
        let request = authed_request("/");

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
