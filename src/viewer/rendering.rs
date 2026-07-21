use std::{process::Stdio, time::Duration};

use anyhow::{Context, Result};
use axum::http::header;
use futures_util::StreamExt;
use reqwest::Client;
use tokio::{io::AsyncWriteExt, process::Command as TokioCommand};

use crate::{markdown::Diagram, plantuml::DiagramRenderer};

const MAX_DIAGRAM_BYTES: usize = 2 * 1024 * 1024;
const RENDER_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn request_diagram(
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
    use crate::{markdown::Diagram, plantuml::DiagramRenderer};

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
            fs,
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
