use std::{future::Future, net::TcpListener, path::PathBuf};

use anyhow::{Context, Result};
use tokio::{sync::oneshot, task::JoinHandle};

mod known_documents;
mod page;
mod rendering;
mod routes;
mod state;

use rendering::renderer_client;
use routes::router;
use state::{viewer_state, watch_documents};

use crate::browser::open_browser;
use crate::target::MarkdownTarget;

pub(crate) struct ViewerSession {
    view_url: String,
    initial_path: PathBuf,
    server_task: JoinHandle<Result<()>>,
    watcher_task: JoinHandle<()>,
    shutdown_sender: Option<oneshot::Sender<()>>,
}

impl ViewerSession {
    pub(crate) fn view_url(&self) -> &str {
        &self.view_url
    }

    async fn run_until<F>(mut self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()>,
    {
        tokio::pin!(shutdown);
        let result = tokio::select! {
            result = &mut self.server_task => joined_server_result(result),
            () = &mut shutdown => {
                if let Some(shutdown_sender) = self.shutdown_sender.take() {
                    let _ = shutdown_sender.send(());
                }
                joined_server_result((&mut self.server_task).await)
            }
        };
        self.watcher_task.abort();
        result
    }
}

pub(crate) async fn start_session(
    target: MarkdownTarget,
    plantuml_server: String,
) -> Result<ViewerSession> {
    let (document_root, documents, initial_document) = target.into_parts();
    let initial_path = documents[initial_document].canonical_path.clone();
    let state = viewer_state(
        document_root,
        documents,
        initial_document,
        renderer_client()?,
        plantuml_server,
    );
    let listener =
        TcpListener::bind("127.0.0.1:0").context("Could not start the loopback viewer")?;
    let address = listener
        .local_addr()
        .context("Could not determine the loopback viewer address")?;
    let server = axum::Server::from_tcp(listener)
        .context("Could not serve the loopback viewer")?
        .serve(router(state.clone()).into_make_service());
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let server_task = tokio::spawn(async move {
        server
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            })
            .await
            .context("The loopback viewer stopped unexpectedly")
    });
    let watcher_task = tokio::spawn(watch_documents(state));

    Ok(ViewerSession {
        view_url: format!("http://{address}"),
        initial_path,
        server_task,
        watcher_task,
        shutdown_sender: Some(shutdown_sender),
    })
}

pub async fn serve(target: MarkdownTarget) -> Result<()> {
    let session = start_session(target, crate::plantuml::server()).await?;

    println!(
        "Lens is serving {} at {}",
        session.initial_path.display(),
        session.view_url()
    );
    if let Err(error) = open_browser(session.view_url()) {
        eprintln!("Could not open a browser automatically: {error}");
        eprintln!("Open {} manually.", session.view_url());
    }

    session.run_until(shutdown_signal()).await
}

impl Drop for ViewerSession {
    fn drop(&mut self) {
        self.server_task.abort();
        self.watcher_task.abort();
    }
}

fn joined_server_result(result: Result<Result<()>, tokio::task::JoinError>) -> Result<()> {
    result.context("The loopback viewer task stopped unexpectedly")?
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Could not listen for Ctrl-C: {error}");
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, net::TcpStream, time::Duration};

    use super::start_session;
    use crate::{load_markdown_target, plantuml::PUBLIC_SERVER};

    #[tokio::test]
    async fn started_session_then_serves_selected_document_while_handle_is_retained() {
        // Arrange
        let root = std::env::temp_dir().join(format!("lens-viewer-session-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("stale test fixture should be removable");
        }
        fs::create_dir(&root).expect("test document root should be creatable");
        let document = root.join("README.md");
        fs::write(&document, "# Retained viewer session")
            .expect("test document should be writable");
        let target = load_markdown_target(Some(&document)).expect("test target should load");

        // Act
        let session = start_session(target, PUBLIC_SERVER.to_owned())
            .await
            .expect("viewer session should start");
        let response = reqwest::get(session.view_url())
            .await
            .expect("retained viewer session should respond");
        let status = response.status();
        let body = response
            .text()
            .await
            .expect("viewer response should be readable");

        // Assert
        assert!(status.is_success());
        assert!(body.contains("Retained viewer session"));
        drop(session);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }

    #[tokio::test]
    async fn dropped_session_then_releases_loopback_listener() {
        // Arrange
        let root = std::env::temp_dir().join(format!(
            "lens-dropped-viewer-session-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("stale test fixture should be removable");
        }
        fs::create_dir(&root).expect("test document root should be creatable");
        let document = root.join("README.md");
        fs::write(&document, "# Dropped viewer session").expect("test document should be writable");
        let target = load_markdown_target(Some(&document)).expect("test target should load");
        let session = start_session(target, PUBLIC_SERVER.to_owned())
            .await
            .expect("viewer session should start");
        let address = session
            .view_url()
            .strip_prefix("http://")
            .expect("viewer URL should use HTTP")
            .parse()
            .expect("viewer URL should contain a socket address");

        // Act
        drop(session);
        let released = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                tokio::task::yield_now().await;
                if TcpStream::connect_timeout(&address, Duration::from_millis(10)).is_err() {
                    break;
                }
            }
        })
        .await
        .is_ok();

        // Assert
        assert!(released, "dropping the session should release its listener");
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }
}
