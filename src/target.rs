use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug)]
pub struct MarkdownDocument {
    pub(crate) identifier: String,
    pub(crate) canonical_path: PathBuf,
    pub(crate) source: String,
}

#[derive(Debug)]
pub struct MarkdownTarget {
    documents: Vec<MarkdownDocument>,
    initial_document: usize,
}

impl MarkdownTarget {
    pub(crate) fn into_parts(self) -> (Vec<MarkdownDocument>, usize) {
        (self.documents, self.initial_document)
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
    #[error("Target {path} is not a directory or Markdown file; expected .md or .markdown")]
    UnsupportedTarget { path: PathBuf },
    #[error("Target {path} contains no discoverable Markdown documents")]
    NoMarkdownDocuments { path: PathBuf },
    #[error("Target {path} is a symbolic link; choose a directory or Markdown file directly")]
    SymbolicLinkTarget { path: PathBuf },
    #[error("Target {path} is hidden; choose a visible directory or Markdown file")]
    HiddenTarget { path: PathBuf },
}

pub fn load_markdown_target(path: Option<&Path>) -> Result<MarkdownTarget, TargetError> {
    let requested_path = match path {
        Some(path) => path.to_path_buf(),
        None => env::current_dir().map_err(|source| TargetError::Unreadable {
            path: PathBuf::from("."),
            source,
        })?,
    };
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

    let (document_root, initial_path) = if metadata.is_dir() {
        (canonical_target, None)
    } else if metadata.is_file() && is_markdown_file(&canonical_target) {
        let document_root = canonical_target
            .parent()
            .expect("a regular file has a parent directory")
            .to_path_buf();
        (document_root, Some(canonical_target))
    } else {
        return Err(TargetError::UnsupportedTarget {
            path: canonical_target,
        });
    };

    let documents = discover_documents(&document_root)?;
    if documents.is_empty() {
        return Err(TargetError::NoMarkdownDocuments {
            path: document_root,
        });
    }
    let initial_document = select_initial_document(&documents, initial_path.as_deref())
        .expect("the selected file must be present in its discovered document set");

    Ok(MarkdownTarget {
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
        if !file_type.is_file() || !is_markdown_file(&path) {
            continue;
        }

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
        });
    }

    Ok(())
}

fn select_initial_document(
    documents: &[MarkdownDocument],
    initial_path: Option<&Path>,
) -> Option<usize> {
    if let Some(initial_path) = initial_path {
        return documents
            .iter()
            .position(|document| document.canonical_path == initial_path);
    }

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

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::{is_hidden_target, load_markdown_target, MarkdownTarget, TargetError};

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
    fn non_markdown_file_then_returns_unsupported_target_error() {
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
    fn directory_without_markdown_then_returns_no_documents_error() {
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
    fn direct_markdown_file_then_selects_file_and_discovers_siblings() {
        // Arrange
        let directory = temporary_directory("file-document-root");
        let selected = directory.join("selected.md");
        fs::write(&selected, "# Selected\n").expect("selected fixture should be writable");
        fs::write(directory.join("sibling.md"), "# Sibling\n")
            .expect("sibling fixture should be writable");

        // Act
        let target = load_markdown_target(Some(&selected)).expect("Markdown file should load");

        // Assert
        assert_target(&target, 0, &["selected.md", "sibling.md"]);
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
