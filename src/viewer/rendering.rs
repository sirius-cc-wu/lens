use std::time::Duration;

use anyhow::{Context, Result};
use axum::http::header;
use futures_util::StreamExt;
use reqwest::Client;

use crate::markdown::Diagram;

const MAX_DIAGRAM_BYTES: usize = 2 * 1024 * 1024;
const RENDER_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn request_diagram(
    client: &Client,
    server: &str,
    diagram: &Diagram,
) -> Result<Vec<u8>> {
    let response = client
        .get(crate::plantuml::svg_url(server, &diagram.source))
        .send()
        .await
        .context("Could not contact the PlantUML server")?;
    if !response.status().is_success() {
        anyhow::bail!("The PlantUML server returned {}", response.status());
    }
    if response.headers().contains_key("x-plantuml-diagram-error") {
        anyhow::bail!("The PlantUML server reported an invalid diagram");
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !content_type.starts_with("image/svg+xml") {
        anyhow::bail!("The PlantUML server did not return SVG content");
    }
    if response
        .content_length()
        .is_some_and(|length| length as usize > MAX_DIAGRAM_BYTES)
    {
        anyhow::bail!("The PlantUML server returned an oversized diagram");
    }

    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Could not read the PlantUML response")?;
        if bytes.len() + chunk.len() > MAX_DIAGRAM_BYTES {
            anyhow::bail!("The PlantUML server returned an oversized diagram");
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub(super) fn renderer_client() -> Result<Client> {
    renderer_client_with_timeout(RENDER_TIMEOUT)
}

fn renderer_client_with_timeout(timeout: Duration) -> Result<Client> {
    Client::builder()
        .timeout(timeout)
        .build()
        .context("Could not configure the PlantUML client")
}

#[cfg(test)]
mod tests {
    use std::{net::TcpListener, time::Duration};

    use axum::{http::header, routing::get, Router};

    use super::{renderer_client, renderer_client_with_timeout, request_diagram};
    use crate::markdown::Diagram;

    async fn mock_plantuml_server(server: Router) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
        let address = listener
            .local_addr()
            .expect("mock server should have an address");
        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .expect("mock server should serve")
                .serve(server.into_make_service())
                .await
                .expect("mock server should not fail");
        });
        format!("http://{address}")
    }

    #[tokio::test]
    async fn configured_plantuml_server_then_uses_only_that_server() {
        // Arrange
        let server = Router::new().route(
            "/svg/*encoded",
            get(|| async { ([(header::CONTENT_TYPE, "image/svg+xml")], "<svg></svg>") }),
        );
        let diagram = Diagram {
            source: "@startuml\n@enduml".to_owned(),
        };
        let server = mock_plantuml_server(server).await;

        // Act
        let response = request_diagram(
            &renderer_client().expect("test client should initialize"),
            &server,
            &diagram,
        )
        .await
        .expect("valid SVG should render");

        // Assert
        assert_eq!(response, b"<svg></svg>");
    }

    #[tokio::test]
    async fn plantuml_error_header_then_returns_error() {
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
        let server = mock_plantuml_server(server).await;

        // Act
        let result = request_diagram(
            &renderer_client().expect("test client should initialize"),
            &server,
            &diagram,
        )
        .await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn unavailable_configured_server_then_does_not_contact_default_server() {
        // Arrange
        let server = Router::new().route(
            "/svg/*encoded",
            get(|| async { (axum::http::StatusCode::SERVICE_UNAVAILABLE, "unavailable") }),
        );
        let diagram = Diagram {
            source: "@startuml\n@enduml".to_owned(),
        };
        let server = mock_plantuml_server(server).await;

        // Act
        let result = request_diagram(
            &renderer_client().expect("test client should initialize"),
            &server,
            &diagram,
        )
        .await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delayed_plantuml_server_then_times_out() {
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
        let server = mock_plantuml_server(server).await;
        let client = renderer_client_with_timeout(Duration::from_millis(10))
            .expect("test client should initialize");

        // Act
        let result = request_diagram(&client, &server, &diagram).await;

        // Assert
        assert!(result.is_err());
    }
}
