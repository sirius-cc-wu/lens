use std::net::TcpListener;

use anyhow::{Context, Result};

mod browser;
mod catalog;
mod page;
mod rendering;
mod routes;
mod state;

use browser::open_browser;
use rendering::renderer_client;
use routes::router;
use state::{viewer_state, watch_documents};

use crate::{
    plantuml::{DiagramRenderer, RendererMode},
    target::MarkdownTarget,
};

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

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Could not listen for Ctrl-C: {error}");
    }
}
