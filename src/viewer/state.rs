use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

use reqwest::Client;

use super::catalog::DocumentCatalog;
use crate::{
    markdown::{render, render_standalone_plantuml, RenderedDocument},
    plantuml::DiagramRenderer,
    target::{DocumentKind, MarkdownDocument},
};

const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

pub(super) struct ViewerState {
    pub(super) documents: RwLock<Vec<ViewerDocument>>,
    pub(super) catalog: DocumentCatalog,
    known_documents: BTreeSet<String>,
    pub(super) initial_document: usize,
    pub(super) client: Client,
    pub(super) renderer: DiagramRenderer,
    rendering_disabled: AtomicBool,
}

impl ViewerState {
    pub(super) fn rendering_enabled(&self) -> bool {
        self.renderer.is_enabled() && !self.rendering_disabled.load(Ordering::Acquire)
    }

    pub(super) fn disable_rendering(&self) {
        self.rendering_disabled.store(true, Ordering::Release);
    }

    pub(super) fn document_revision(&self, document_id: usize) -> Option<u64> {
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

pub(super) struct ViewerDocument {
    pub(super) identifier: String,
    pub(super) canonical_path: PathBuf,
    source: String,
    kind: DocumentKind,
    pub(super) rendered: RenderedDocument,
    pub(super) revision: u64,
}

impl ViewerDocument {
    fn replace(&mut self, source: String, rendered: RenderedDocument) {
        self.source = source;
        self.rendered = rendered;
        self.revision += 1;
    }
}

pub(super) fn viewer_state(
    documents: Vec<MarkdownDocument>,
    initial_document: usize,
    client: Client,
    renderer: DiagramRenderer,
) -> Arc<ViewerState> {
    let catalog = DocumentCatalog::new(
        documents
            .iter()
            .enumerate()
            .map(|(index, document)| (document.identifier.clone(), index)),
    );
    let known_documents = catalog.known_document_ids();
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
        catalog,
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

pub(super) async fn watch_documents(state: Arc<ViewerState>) {
    let mut interval = tokio::time::interval(REFRESH_INTERVAL);
    loop {
        interval.tick().await;
        state.refresh_known_documents();
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::viewer_state;
    use crate::{
        plantuml::{DiagramRenderer, RendererMode},
        target::{DocumentKind, MarkdownDocument},
        viewer::rendering::renderer_client,
    };

    fn test_renderer() -> DiagramRenderer {
        DiagramRenderer::from_mode(RendererMode::Public)
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
}
