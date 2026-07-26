use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use reqwest::Client;

use super::known_documents::KnownDocuments;
use crate::{
    markdown::{render, render_standalone_plantuml, RenderedDocument},
    source_link::SourceLinkResolver,
    target::{DocumentKind, MarkdownDocument},
};

const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

pub(super) struct ViewerState {
    pub(super) documents: RwLock<Vec<ViewerDocument>>,
    pub(super) known_documents: KnownDocuments,
    known_document_ids: BTreeSet<String>,
    source_links: SourceLinkResolver,
    pub(super) initial_document: usize,
    pub(super) client: Client,
    pub(super) plantuml_server: String,
}

impl ViewerState {
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
            let Ok(source) = fs::read_to_string(&canonical_path) else {
                continue;
            };
            if source == stored_source {
                continue;
            }

            let rendered = render_document(
                &source,
                document_id,
                &identifier,
                &canonical_path,
                kind,
                &self.known_document_ids,
                &self.source_links,
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
    document_root: PathBuf,
    documents: Vec<MarkdownDocument>,
    initial_document: usize,
    client: Client,
    plantuml_server: String,
) -> Arc<ViewerState> {
    let known_documents = KnownDocuments::new(
        documents
            .iter()
            .enumerate()
            .map(|(index, document)| (document.identifier.clone(), index)),
    );
    let source_links = SourceLinkResolver::new(document_root);
    let known_document_ids = documents
        .iter()
        .map(|document| document.identifier.clone())
        .collect();
    let documents = documents
        .into_iter()
        .enumerate()
        .map(|(document_id, document)| {
            let rendered = render_document(
                &document.source,
                document_id,
                &document.identifier,
                &document.canonical_path,
                document.kind,
                &known_document_ids,
                &source_links,
            );
            ViewerDocument {
                identifier: document.identifier,
                canonical_path: document.canonical_path,
                source: document.source,
                kind: document.kind,
                rendered,
                revision: 0,
            }
        })
        .collect();

    Arc::new(ViewerState {
        documents: RwLock::new(documents),
        known_documents,
        known_document_ids,
        source_links,
        initial_document,
        client,
        plantuml_server,
    })
}

fn render_document(
    source: &str,
    document_id: usize,
    identifier: &str,
    canonical_path: &std::path::Path,
    kind: DocumentKind,
    known_documents: &BTreeSet<String>,
    source_links: &SourceLinkResolver,
) -> RenderedDocument {
    match kind {
        DocumentKind::Markdown => render(
            source,
            document_id,
            identifier,
            canonical_path,
            known_documents,
            source_links,
        ),
        DocumentKind::PlantUml => render_standalone_plantuml(document_id, source),
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
        plantuml::PUBLIC_SERVER,
        target::{DocumentKind, MarkdownDocument},
        viewer::rendering::renderer_client,
    };

    fn test_server() -> String {
        PUBLIC_SERVER.to_owned()
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
            path.parent()
                .expect("test document should have a parent")
                .to_path_buf(),
            vec![file_backed_test_document(path.clone(), "# Before refresh")],
            0,
            renderer_client().expect("test client should initialize"),
            test_server(),
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
            path.parent()
                .expect("test document should have a parent")
                .to_path_buf(),
            vec![file_backed_test_document(
                path.clone(),
                "# Readable document",
            )],
            0,
            renderer_client().expect("test client should initialize"),
            test_server(),
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

    #[test]
    fn changed_document_source_link_then_reuses_session_root() {
        // Arrange
        let root =
            std::env::temp_dir().join(format!("lens-viewer-source-link-{}", std::process::id()));
        let document_path = root.join("docs/README.md");
        let source_path = root.join("src/lib.rs");
        fs::create_dir_all(
            document_path
                .parent()
                .expect("document should have a parent"),
        )
        .expect("document directory should be creatable");
        fs::create_dir_all(source_path.parent().expect("source should have a parent"))
            .expect("source directory should be creatable");
        fs::write(&document_path, "# Before refresh").expect("test document should be writable");
        fs::write(&source_path, "source").expect("test source should be writable");
        let document_path =
            fs::canonicalize(document_path).expect("test document should canonicalize");
        let root = fs::canonicalize(root).expect("test root should canonicalize");
        let state = viewer_state(
            root.clone(),
            vec![file_backed_test_document(
                document_path.clone(),
                "# Before refresh",
            )],
            0,
            renderer_client().expect("test client should initialize"),
            test_server(),
        );
        fs::write(&document_path, "[Source](../src/lib.rs)").expect("test document should update");

        // Act
        state.refresh_known_documents();

        // Assert
        let documents = state
            .documents
            .read()
            .expect("viewer documents lock should not be poisoned");
        assert!(documents[0].rendered.html.contains("href=\"vscode://file/"));
        assert!(documents[0].rendered.html.contains("/src/lib.rs\""));
        drop(documents);
        fs::remove_dir_all(root).expect("test fixture should be removable");
    }
}
