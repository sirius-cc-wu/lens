mod markdown;
mod plantuml;
mod service;
mod source_link;
mod target;
mod viewer;

pub use target::{
    load_markdown_target, load_markdown_target_with_scope, MarkdownTarget, TargetScope,
};
pub use viewer::serve;

#[doc(hidden)]
/// Internal CLI entry point for opening a target through the reusable background service.
/// Library consumers should use [`serve`] for supported foreground document viewing.
pub async fn open(target: Option<std::path::PathBuf>, scope: TargetScope) -> anyhow::Result<()> {
    let invocation = service::client::OpenInvocation::capture(target, scope)?;
    let service::client::ClientOutcome::Ready { view_url } =
        service::client::request_target_view(invocation).await?;
    println!("{view_url}");
    Ok(())
}

#[doc(hidden)]
pub async fn run_background_service() -> anyhow::Result<()> {
    service::server::run_background_service().await?;
    Ok(())
}

#[doc(hidden)]
/// Internal CLI entry point for stopping the reusable background service.
pub async fn stop() -> anyhow::Result<()> {
    match service::client::stop_background_service().await? {
        service::client::StopOutcome::Stopped => {
            println!("Lens background service stopped.");
        }
        service::client::StopOutcome::NotRunning => {
            println!("No Lens background service is running.");
        }
    }
    Ok(())
}
