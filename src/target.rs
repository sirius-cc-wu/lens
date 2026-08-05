use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug)]
pub struct MarkdownDocument {
    pub(crate) identifier: String,
    pub(crate) canonical_path: PathBuf,
    pub(crate) source: String,
    pub(crate) kind: DocumentKind,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DocumentKind {
    Markdown,
    PlantUml,
}

#[derive(Debug)]
pub struct MarkdownTarget {
    document_root: PathBuf,
    documents: Vec<MarkdownDocument>,
    initial_document: usize,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum TargetScope {
    #[default]
    Repository,
    Target,
}

enum InitialSelection {
    Document(PathBuf),
    Directory(PathBuf),
}

impl MarkdownTarget {
    pub(crate) fn into_parts(self) -> (PathBuf, Vec<MarkdownDocument>, usize) {
        (self.document_root, self.documents, self.initial_document)
    }
}

#[derive(Debug, Error)]
pub enum TargetError {
    #[error("Target {path} does not exist")]
    Missing { path: PathBuf },
    #[error("Target {path} is not readable: {source}")]
    Unreadable {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "Target {path} is not a directory or supported document; expected .md, .markdown, or .puml"
    )]
    UnsupportedTarget { path: PathBuf },
    #[error("Target {path} contains no discoverable Markdown or PlantUML documents")]
    NoMarkdownDocuments { path: PathBuf },
    #[error("Target {path} is a symbolic link; choose a directory or Markdown file directly")]
    SymbolicLinkTarget { path: PathBuf },
    #[error("Target {path} is hidden; choose a visible directory or Markdown file")]
    HiddenTarget { path: PathBuf },
}

pub fn load_markdown_target(path: Option<&Path>) -> Result<MarkdownTarget, TargetError> {
    load_markdown_target_with_scope(path, TargetScope::Repository)
}

pub fn load_markdown_target_with_scope(
    path: Option<&Path>,
    scope: TargetScope,
) -> Result<MarkdownTarget, TargetError> {
    let requested_path = match path {
        Some(path) => path.to_path_buf(),
        None => env::current_dir().map_err(|source| TargetError::Unreadable {
            path: PathBuf::from("."),
            source,
        })?,
    };
    load_resolved_target(requested_path, scope)
}

pub(crate) fn load_markdown_target_from(
    invocation_directory: &Path,
    path: Option<&Path>,
    scope: TargetScope,
) -> Result<MarkdownTarget, TargetError> {
    let requested_path = match path {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => invocation_directory.join(path),
        None => invocation_directory.to_path_buf(),
    };
    load_resolved_target(requested_path, scope)
}

fn load_resolved_target(
    requested_path: PathBuf,
    scope: TargetScope,
) -> Result<MarkdownTarget, TargetError> {
    let metadata = target_metadata(&requested_path)?;
    if metadata.file_type().is_symlink() {
        return Err(TargetError::SymbolicLinkTarget {
            path: requested_path,
        });
    }
    if is_hidden_target(&requested_path) {
        return Err(TargetError::HiddenTarget {
            path: requested_path,
        });
    }
    let canonical_target = canonicalize(&requested_path)?;

    let (target_root, initial_selection) = if metadata.is_dir() {
        (
            canonical_target.clone(),
            InitialSelection::Directory(canonical_target.clone()),
        )
    } else if metadata.is_file() && document_kind(&canonical_target).is_some() {
        let parent = canonical_target
            .parent()
            .expect("a regular file has a parent directory")
            .to_path_buf();
        (parent, InitialSelection::Document(canonical_target.clone()))
    } else {
        return Err(TargetError::UnsupportedTarget {
            path: canonical_target,
        });
    };

    let document_root = match scope {
        TargetScope::Repository => {
            nearest_repository_root(&target_root).unwrap_or_else(|| target_root.clone())
        }
        TargetScope::Target => target_root,
    };
    if contains_hidden_entry(&document_root, &canonical_target) {
        return Err(TargetError::HiddenTarget {
            path: canonical_target,
        });
    }

    let documents = discover_documents(&document_root)?;
    if documents.is_empty() {
        return Err(TargetError::NoMarkdownDocuments {
            path: document_root,
        });
    }
    let initial_document = select_initial_document(&documents, &initial_selection)
        .expect("a non-empty document set must have an initial document");

    Ok(MarkdownTarget {
        document_root,
        documents,
        initial_document,
    })
}

fn discover_documents(root: &Path) -> Result<Vec<MarkdownDocument>, TargetError> {
    let mut documents = Vec::new();
    discover_documents_in(root, root, &mut documents)?;
    documents.sort_by(|left, right| left.identifier.cmp(&right.identifier));
    Ok(documents)
}

fn discover_documents_in(
    root: &Path,
    directory: &Path,
    documents: &mut Vec<MarkdownDocument>,
) -> Result<(), TargetError> {
    let entries = fs::read_dir(directory).map_err(|source| TargetError::Unreadable {
        path: directory.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| TargetError::Unreadable {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| TargetError::Unreadable {
                path: path.clone(),
                source,
            })?;

        if file_type.is_symlink() || is_hidden(&entry.file_name()) {
            continue;
        }
        if file_type.is_dir() {
            discover_documents_in(root, &path, documents)?;
            continue;
        }
        let Some(kind) = file_type.is_file().then(|| document_kind(&path)).flatten() else {
            continue;
        };

        let canonical_path = canonicalize(&path)?;
        if !canonical_path.starts_with(root) {
            continue;
        }
        let source =
            fs::read_to_string(&canonical_path).map_err(|source| TargetError::Unreadable {
                path: canonical_path.clone(),
                source,
            })?;
        documents.push(MarkdownDocument {
            identifier: document_identifier(root, &path),
            canonical_path,
            source,
            kind,
        });
    }

    Ok(())
}

fn select_initial_document(
    documents: &[MarkdownDocument],
    initial_selection: &InitialSelection,
) -> Option<usize> {
    match initial_selection {
        InitialSelection::Document(initial_path) => documents
            .iter()
            .position(|document| document.canonical_path == *initial_path),
        InitialSelection::Directory(directory) => {
            select_initial_document_in_directory(documents, directory)
                .or_else(|| select_repository_initial_document(documents))
        }
    }
}

fn select_initial_document_in_directory(
    documents: &[MarkdownDocument],
    directory: &Path,
) -> Option<usize> {
    documents
        .iter()
        .position(|document| {
            identifier_relative_to(directory, &document.canonical_path)
                .is_some_and(|identifier| is_root_readme(&identifier))
        })
        .or_else(|| {
            documents.iter().position(|document| {
                identifier_relative_to(directory, &document.canonical_path)
                    .is_some_and(|identifier| is_document_index(&identifier))
            })
        })
        .or_else(|| {
            documents
                .iter()
                .position(|document| document.canonical_path.starts_with(directory))
        })
}

fn select_repository_initial_document(documents: &[MarkdownDocument]) -> Option<usize> {
    documents
        .iter()
        .position(|document| is_root_readme(&document.identifier))
        .or_else(|| {
            documents
                .iter()
                .position(|document| is_document_index(&document.identifier))
        })
        .or(Some(0))
}

fn identifier_relative_to(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root).ok().map(|relative| {
        relative
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/")
    })
}

fn is_root_readme(identifier: &str) -> bool {
    identifier.eq_ignore_ascii_case("README.md")
        || identifier.eq_ignore_ascii_case("README.markdown")
}

fn is_document_index(identifier: &str) -> bool {
    identifier.eq_ignore_ascii_case("docs/index.md")
        || identifier.eq_ignore_ascii_case("docs/index.markdown")
}

fn is_hidden(file_name: &std::ffi::OsStr) -> bool {
    file_name
        .to_str()
        .is_some_and(|file_name| file_name.starts_with('.'))
}

fn is_hidden_target(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|file_name| file_name != "." && file_name != ".." && is_hidden(file_name))
}

fn document_identifier(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("discovered documents are inside the document root")
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn canonicalize(path: &Path) -> Result<PathBuf, TargetError> {
    fs::canonicalize(path).map_err(|source| {
        if source.kind() == ErrorKind::NotFound {
            TargetError::Missing {
                path: path.to_path_buf(),
            }
        } else {
            TargetError::Unreadable {
                path: path.to_path_buf(),
                source,
            }
        }
    })
}

fn target_metadata(path: &Path) -> Result<fs::Metadata, TargetError> {
    fs::symlink_metadata(path).map_err(|source| {
        if source.kind() == ErrorKind::NotFound {
            TargetError::Missing {
                path: path.to_path_buf(),
            }
        } else {
            TargetError::Unreadable {
                path: path.to_path_buf(),
                source,
            }
        }
    })
}

fn nearest_repository_root(directory: &Path) -> Option<PathBuf> {
    directory
        .ancestors()
        .find(|ancestor| is_repository_root(ancestor))
        .map(Path::to_path_buf)
}

fn is_repository_root(directory: &Path) -> bool {
    fs::symlink_metadata(directory.join(".git"))
        .ok()
        .is_some_and(|metadata| {
            let file_type = metadata.file_type();
            !file_type.is_symlink() && (file_type.is_dir() || file_type.is_file())
        })
}

fn contains_hidden_entry(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root).ok().is_some_and(|relative| {
        relative.components().any(|component| {
            let std::path::Component::Normal(name) = component else {
                return false;
            };
            is_hidden(name)
        })
    })
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}

fn document_kind(path: &Path) -> Option<DocumentKind> {
    if is_markdown_file(path) {
        Some(DocumentKind::Markdown)
    } else {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("puml"))
            .then_some(DocumentKind::PlantUml)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::{
        is_hidden_target, load_markdown_target, load_markdown_target_from,
        load_markdown_target_with_scope, MarkdownTarget, TargetError, TargetScope,
    };

    #[test]
    fn current_directory_marker_then_is_not_hidden_target() {
        // Arrange
        let path = Path::new(".");

        // Act
        let hidden = is_hidden_target(path);

        // Assert
        assert!(!hidden);
    }

    #[test]
    fn missing_target_then_returns_missing_error() {
        // Arrange
        let path = Path::new("missing-document.md");

        // Act
        let result = load_markdown_target(Some(path));

        // Assert
        assert!(matches!(result, Err(TargetError::Missing { .. })));
    }

    #[test]
    fn relative_target_with_invocation_directory_then_resolves_from_invoking_shell() {
        // Arrange
        let directory = temporary_directory("explicit-invocation-directory");
        fs::create_dir(directory.join("docs")).expect("docs directory should be creatable");
        fs::write(directory.join("docs/guide.md"), "# Guide\n")
            .expect("guide fixture should be writable");

        // Act
        let target = load_markdown_target_from(
            &directory,
            Some(Path::new("docs/guide.md")),
            TargetScope::Target,
        )
        .expect("relative target should load from invocation directory");

        // Assert
        assert_target(&target, 0, &["guide.md"]);
        remove_directory(directory);
    }

    #[test]
    fn unsupported_file_then_returns_unsupported_target_error() {
        // Arrange
        let directory = temporary_directory("unsupported-target");
        let path = directory.join("notes.txt");
        fs::write(&path, "not Markdown").expect("test fixture should be writable");

        // Act
        let result = load_markdown_target(Some(&path));

        // Assert
        assert!(matches!(result, Err(TargetError::UnsupportedTarget { .. })));
        remove_directory(directory);
    }

    #[test]
    fn directory_without_supported_documents_then_returns_no_documents_error() {
        // Arrange
        let directory = temporary_directory("empty-document-root");

        // Act
        let result = load_markdown_target(Some(&directory));

        // Assert
        assert!(matches!(
            result,
            Err(TargetError::NoMarkdownDocuments { .. })
        ));
        remove_directory(directory);
    }

    #[test]
    fn directory_with_root_readme_then_selects_readme_and_discovers_nested_documents() {
        // Arrange
        let directory = temporary_directory("readme-document-root");
        fs::write(directory.join("README.md"), "# Read me\n")
            .expect("README fixture should be writable");
        fs::create_dir(directory.join("guides")).expect("guide directory should be creatable");
        fs::write(directory.join("guides/usage.markdown"), "# Usage\n")
            .expect("guide fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&directory)).expect("document root should load");

        // Assert
        assert_target(&target, 0, &["README.md", "guides/usage.markdown"]);
        remove_directory(directory);
    }

    #[test]
    fn git_directory_with_markdown_then_excludes_git_document() {
        // Arrange
        let directory = temporary_directory("git-document-root");
        fs::write(directory.join("README.md"), "# Read me\n")
            .expect("README fixture should be writable");
        fs::create_dir(directory.join(".git")).expect("git directory should be creatable");
        fs::write(directory.join(".git/private.md"), "# Private\n")
            .expect("git fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&directory)).expect("document root should load");

        // Assert
        assert_target(&target, 0, &["README.md"]);
        remove_directory(directory);
    }

    #[test]
    fn docs_index_without_root_readme_then_selects_document_index_and_excludes_hidden_documents() {
        // Arrange
        let directory = temporary_directory("index-document-root");
        fs::create_dir(directory.join("docs")).expect("docs directory should be creatable");
        fs::write(directory.join("docs/guide.md"), "# Guide\n")
            .expect("guide fixture should be writable");
        fs::write(directory.join("docs/index.md"), "# Index\n")
            .expect("index fixture should be writable");
        fs::create_dir(directory.join(".internal")).expect("hidden directory should be creatable");
        fs::write(directory.join(".internal/notes.md"), "# Internal\n")
            .expect("hidden fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&directory)).expect("document root should load");

        // Assert
        assert_target(&target, 1, &["docs/guide.md", "docs/index.md"]);
        remove_directory(directory);
    }

    #[test]
    fn direct_file_in_repository_then_discovers_repository_documents() {
        // Arrange
        let repository = temporary_directory("direct-file-repository");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir_all(repository.join("docs/features"))
            .expect("feature directory should be creatable");
        fs::create_dir_all(repository.join("docs/iterations"))
            .expect("iteration directory should be creatable");
        let selected = repository.join("docs/features/guide.md");
        fs::write(&selected, "# Guide\n").expect("selected fixture should be writable");
        fs::write(
            repository.join("docs/iterations/evidence.md"),
            "# Evidence\n",
        )
        .expect("iteration fixture should be writable");
        fs::write(repository.join("README.md"), "# Repository\n")
            .expect("README fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&selected)).expect("Markdown file should load");

        // Assert
        assert_eq!(
            target.document_root,
            fs::canonicalize(&repository).expect("repository root should canonicalize")
        );
        assert_target(
            &target,
            1,
            &[
                "README.md",
                "docs/features/guide.md",
                "docs/iterations/evidence.md",
            ],
        );
        remove_directory(repository);
    }

    #[test]
    fn direct_file_in_worktree_then_uses_worktree_root() {
        // Arrange
        let worktree = temporary_directory("direct-file-worktree");
        fs::write(worktree.join(".git"), "gitdir: /tmp/example\n")
            .expect("Git file marker should be writable");
        fs::create_dir_all(worktree.join("docs/guides"))
            .expect("guide directory should be creatable");
        let selected = worktree.join("docs/guides/selected.markdown");
        fs::write(&selected, "# Selected\n").expect("selected fixture should be writable");
        fs::write(worktree.join("README.md"), "# Worktree\n")
            .expect("README fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&selected)).expect("Markdown file should load");

        // Assert
        assert_target(&target, 1, &["README.md", "docs/guides/selected.markdown"]);
        remove_directory(worktree);
    }

    #[test]
    fn direct_file_in_nested_repository_then_uses_nearest_repository_root() {
        // Arrange
        let outer_repository = temporary_directory("nested-repository");
        fs::create_dir(outer_repository.join(".git"))
            .expect("outer Git marker should be creatable");
        fs::write(outer_repository.join("README.md"), "# Outer\n")
            .expect("outer README fixture should be writable");
        let inner_repository = outer_repository.join("vendor/module");
        fs::create_dir_all(inner_repository.join(".git"))
            .expect("inner Git marker should be creatable");
        fs::create_dir(inner_repository.join("docs"))
            .expect("inner docs directory should be creatable");
        let selected = inner_repository.join("docs/architecture.puml");
        fs::write(&selected, "@startuml\n@enduml\n")
            .expect("selected PlantUML fixture should be writable");
        fs::write(inner_repository.join("README.md"), "# Inner\n")
            .expect("inner README fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&selected)).expect("PlantUML file should load");

        // Assert
        assert_target(&target, 1, &["README.md", "docs/architecture.puml"]);
        assert!(matches!(
            target.documents[1].kind,
            super::DocumentKind::PlantUml
        ));
        remove_directory(outer_repository);
    }

    #[test]
    fn direct_file_without_repository_then_discovers_only_parent_documents() {
        // Arrange
        let directory = temporary_directory("file-without-repository");
        fs::create_dir_all(directory.join("docs/features"))
            .expect("feature directory should be creatable");
        fs::create_dir(directory.join("docs/iterations"))
            .expect("iteration directory should be creatable");
        let selected = directory.join("docs/features/selected.md");
        fs::write(&selected, "# Selected\n").expect("selected fixture should be writable");
        fs::write(directory.join("docs/features/sibling.md"), "# Sibling\n")
            .expect("sibling fixture should be writable");
        fs::write(directory.join("docs/iterations/outside.md"), "# Outside\n")
            .expect("outside-parent fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&selected)).expect("Markdown file should load");

        // Assert
        assert_target(&target, 0, &["selected.md", "sibling.md"]);
        remove_directory(directory);
    }

    #[test]
    fn directory_target_inside_repository_then_discovers_repository_documents() {
        // Arrange
        let repository = temporary_directory("directory-inside-repository");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir_all(repository.join("docs/features"))
            .expect("feature directory should be creatable");
        fs::create_dir(repository.join("docs/iterations"))
            .expect("iteration directory should be creatable");
        fs::write(repository.join("docs/features/guide.md"), "# Guide\n")
            .expect("guide fixture should be writable");
        fs::write(
            repository.join("docs/iterations/evidence.md"),
            "# Evidence\n",
        )
        .expect("iteration fixture should be writable");
        fs::write(repository.join("README.md"), "# Repository\n")
            .expect("README fixture should be writable");
        let selected_directory = repository.join("docs/features");

        // Act
        let target =
            load_markdown_target(Some(&selected_directory)).expect("directory should load");

        // Assert
        assert_target(
            &target,
            1,
            &[
                "README.md",
                "docs/features/guide.md",
                "docs/iterations/evidence.md",
            ],
        );
        remove_directory(repository);
    }

    #[test]
    fn empty_selected_directory_then_uses_repository_initial_document() {
        // Arrange
        let repository = temporary_directory("empty-selected-directory");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir(repository.join("empty")).expect("selected directory should be creatable");
        fs::write(repository.join("README.md"), "# Repository\n")
            .expect("README fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&repository.join("empty")))
            .expect("repository document root should load");

        // Assert
        assert_target(&target, 0, &["README.md"]);
        remove_directory(repository);
    }

    #[test]
    fn target_scoped_directory_inside_repository_then_discovers_only_target_documents() {
        // Arrange
        let repository = temporary_directory("target-scoped-directory");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir_all(repository.join("docs/features"))
            .expect("feature directory should be creatable");
        fs::create_dir(repository.join("docs/iterations"))
            .expect("iteration directory should be creatable");
        fs::write(repository.join("docs/features/guide.md"), "# Guide\n")
            .expect("guide fixture should be writable");
        fs::write(
            repository.join("docs/iterations/evidence.md"),
            "# Evidence\n",
        )
        .expect("iteration fixture should be writable");

        // Act
        let target = load_markdown_target_with_scope(
            Some(&repository.join("docs/features")),
            TargetScope::Target,
        )
        .expect("target-scoped directory should load");

        // Assert
        assert_target(&target, 0, &["guide.md"]);
        remove_directory(repository);
    }

    #[test]
    fn target_scoped_file_inside_repository_then_discovers_only_parent_documents() {
        // Arrange
        let repository = temporary_directory("target-scoped-file");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir_all(repository.join("docs/features"))
            .expect("feature directory should be creatable");
        let selected = repository.join("docs/features/selected.md");
        fs::write(&selected, "# Selected\n").expect("selected fixture should be writable");
        fs::write(repository.join("docs/features/sibling.md"), "# Sibling\n")
            .expect("sibling fixture should be writable");
        fs::write(repository.join("README.md"), "# Repository\n")
            .expect("README fixture should be writable");

        // Act
        let target = load_markdown_target_with_scope(Some(&selected), TargetScope::Target)
            .expect("target-scoped file should load");

        // Assert
        assert_target(&target, 0, &["selected.md", "sibling.md"]);
        remove_directory(repository);
    }

    #[test]
    fn repository_scoped_directory_below_hidden_entry_then_returns_hidden_target_error() {
        // Arrange
        let repository = temporary_directory("hidden-repository-directory");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir(repository.join(".private")).expect("hidden directory should be creatable");
        fs::create_dir(repository.join(".private/docs"))
            .expect("selected directory should be creatable");
        fs::write(repository.join(".private/docs/guide.md"), "# Guide\n")
            .expect("guide fixture should be writable");

        // Act
        let result = load_markdown_target(Some(&repository.join(".private/docs")));

        // Assert
        assert!(matches!(result, Err(TargetError::HiddenTarget { .. })));
        remove_directory(repository);
    }

    #[test]
    fn target_scoped_directory_below_hidden_parent_then_discovers_documents() {
        // Arrange
        let repository = temporary_directory("target-scoped-hidden-parent");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir(repository.join(".private")).expect("hidden directory should be creatable");
        let selected = repository.join(".private/docs");
        fs::create_dir(&selected).expect("selected directory should be creatable");
        fs::write(selected.join("guide.md"), "# Guide\n")
            .expect("guide fixture should be writable");

        // Act
        let target = load_markdown_target_with_scope(Some(&selected), TargetScope::Target)
            .expect("target-scoped directory should load");

        // Assert
        assert_target(&target, 0, &["guide.md"]);
        remove_directory(repository);
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_git_marker_then_uses_enclosing_repository_root() {
        use std::os::unix::fs::symlink;

        // Arrange
        let repository = temporary_directory("symbolic-git-marker");
        fs::create_dir(repository.join(".git")).expect("outer Git marker should be creatable");
        fs::write(repository.join("README.md"), "# Outer\n")
            .expect("outer README fixture should be writable");
        let nested = repository.join("nested");
        fs::create_dir(&nested).expect("nested directory should be creatable");
        symlink(repository.join(".git"), nested.join(".git"))
            .expect("symbolic Git marker should be creatable");
        fs::create_dir(nested.join("docs")).expect("nested docs directory should be creatable");
        let selected = nested.join("docs/guide.md");
        fs::write(&selected, "# Guide\n").expect("selected fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&selected)).expect("Markdown file should load");

        // Assert
        assert_target(&target, 1, &["README.md", "nested/docs/guide.md"]);
        remove_directory(repository);
    }

    #[test]
    fn direct_file_below_hidden_repository_entry_then_returns_hidden_target_error() {
        // Arrange
        let repository = temporary_directory("hidden-repository-document");
        fs::create_dir(repository.join(".git")).expect("Git marker should be creatable");
        fs::create_dir(repository.join(".private")).expect("hidden directory should be creatable");
        let selected = repository.join(".private/guide.md");
        fs::write(&selected, "# Guide\n").expect("selected fixture should be writable");

        // Act
        let result = load_markdown_target(Some(&selected));

        // Assert
        assert!(matches!(result, Err(TargetError::HiddenTarget { .. })));
        remove_directory(repository);
    }

    #[test]
    fn direct_plantuml_file_then_selects_file_and_discovers_siblings() {
        // Arrange
        let directory = temporary_directory("plantuml-file-document-root");
        let selected = directory.join("architecture.puml");
        fs::write(&selected, "@startuml\n@enduml\n").expect("PlantUML fixture should be writable");
        fs::write(directory.join("guide.md"), "# Guide\n")
            .expect("Markdown fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&selected)).expect("PlantUML file should load");

        // Assert
        assert_target(&target, 0, &["architecture.puml", "guide.md"]);
        assert!(matches!(
            target.documents[0].kind,
            super::DocumentKind::PlantUml
        ));
        remove_directory(directory);
    }

    #[test]
    fn hidden_markdown_target_then_returns_hidden_target_error() {
        // Arrange
        let directory = temporary_directory("hidden-target");
        let hidden = directory.join(".hidden.md");
        fs::write(&hidden, "# Hidden\n").expect("hidden fixture should be writable");
        fs::write(directory.join("visible.md"), "# Visible\n")
            .expect("visible fixture should be writable");

        // Act
        let result = load_markdown_target(Some(&hidden));

        // Assert
        assert!(matches!(result, Err(TargetError::HiddenTarget { .. })));
        remove_directory(directory);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_markdown_file_then_excludes_target_outside_document_root() {
        use std::os::unix::fs::symlink;

        // Arrange
        let root = temporary_directory("symlink-document-root");
        let outside = temporary_directory("outside-document-root");
        fs::write(root.join("inside.md"), "# Inside\n").expect("inside fixture should be writable");
        let outside_document = outside.join("outside.md");
        fs::write(&outside_document, "# Outside\n").expect("outside fixture should be writable");
        symlink(&outside_document, root.join("outside.md")).expect("symlink should be creatable");

        // Act
        let target = load_markdown_target(Some(&root)).expect("document root should load");

        // Assert
        assert_target(&target, 0, &["inside.md"]);
        remove_directory(root);
        remove_directory(outside);
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_link_target_then_returns_symbolic_link_target_error() {
        use std::os::unix::fs::symlink;

        // Arrange
        let root = temporary_directory("symbolic-link-target");
        let outside = temporary_directory("symbolic-link-outside");
        let outside_document = outside.join("outside.md");
        fs::write(&outside_document, "# Outside\n").expect("outside fixture should be writable");
        let link = root.join("outside.md");
        symlink(&outside_document, &link).expect("symlink should be creatable");

        // Act
        let result = load_markdown_target(Some(&link));

        // Assert
        assert!(matches!(
            result,
            Err(TargetError::SymbolicLinkTarget { .. })
        ));
        remove_directory(root);
        remove_directory(outside);
    }

    fn assert_target(target: &MarkdownTarget, initial_document: usize, identifiers: &[&str]) {
        assert_eq!(target.initial_document, initial_document);
        assert_eq!(
            target
                .documents
                .iter()
                .map(|document| document.identifier.as_str())
                .collect::<Vec<_>>(),
            identifiers
        );
    }

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!("{}-{}", std::process::id(), name));
        fs::create_dir_all(&directory).expect("test directory should be creatable");
        directory
    }

    fn remove_directory(directory: std::path::PathBuf) {
        fs::remove_dir_all(directory).expect("test directory should be removable");
    }
}
