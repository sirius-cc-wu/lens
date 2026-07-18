use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct MarkdownTarget {
    pub canonical_path: PathBuf,
    pub source: String,
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
    #[error("Target {path} is a directory; E1 accepts a Markdown file")]
    Directory { path: PathBuf },
    #[error("Target {path} is not a Markdown file; expected .md or .markdown")]
    UnsupportedFileType { path: PathBuf },
}

pub fn load_markdown_target(path: &Path) -> Result<MarkdownTarget, TargetError> {
    let canonical_path = fs::canonicalize(path).map_err(|source| {
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
    })?;
    let metadata = fs::metadata(&canonical_path).map_err(|source| TargetError::Unreadable {
        path: canonical_path.clone(),
        source,
    })?;

    if metadata.is_dir() {
        return Err(TargetError::Directory {
            path: canonical_path,
        });
    }
    if !metadata.is_file() || !is_markdown_file(&canonical_path) {
        return Err(TargetError::UnsupportedFileType {
            path: canonical_path,
        });
    }

    let source = fs::read_to_string(&canonical_path).map_err(|source| TargetError::Unreadable {
        path: canonical_path.clone(),
        source,
    })?;

    Ok(MarkdownTarget {
        canonical_path,
        source,
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

    use super::{load_markdown_target, TargetError};

    #[test]
    fn missing_target_then_returns_missing_error() {
        // Arrange
        let path = Path::new("missing-document.md");

        // Act
        let result = load_markdown_target(path);

        // Assert
        assert!(matches!(result, Err(TargetError::Missing { .. })));
    }

    #[test]
    fn directory_target_then_returns_directory_error() {
        // Arrange
        let directory = std::env::temp_dir();

        // Act
        let result = load_markdown_target(&directory);

        // Assert
        assert!(matches!(result, Err(TargetError::Directory { .. })));
    }

    #[test]
    fn non_markdown_file_then_returns_unsupported_file_type_error() {
        // Arrange
        let path = temporary_path("lens-target.txt");
        fs::write(&path, "not Markdown").expect("test fixture should be writable");

        // Act
        let result = load_markdown_target(&path);

        // Assert
        assert!(matches!(
            result,
            Err(TargetError::UnsupportedFileType { .. })
        ));
        fs::remove_file(path).expect("test fixture should be removable");
    }

    #[test]
    fn markdown_file_then_returns_canonical_path_and_source() {
        // Arrange
        let path = temporary_path("lens-target.md");
        fs::write(&path, "# Lens\n").expect("test fixture should be writable");

        // Act
        let target = load_markdown_target(&path).expect("Markdown fixture should load");

        // Assert
        assert!(target.canonical_path.is_absolute());
        assert_eq!(target.source, "# Lens\n");
        fs::remove_file(path).expect("test fixture should be removable");
    }

    #[cfg(unix)]
    #[test]
    fn markdown_symlink_then_resolves_only_its_canonical_document() {
        use std::os::unix::fs::symlink;

        // Arrange
        let document = temporary_path("lens-canonical-document.md");
        let link = temporary_path("lens-document-link.md");
        fs::write(&document, "# Canonical Lens\n").expect("test fixture should be writable");
        symlink(&document, &link).expect("test symlink should be creatable");

        // Act
        let target = load_markdown_target(&link).expect("Markdown symlink should load");

        // Assert
        assert_eq!(target.canonical_path, fs::canonicalize(&document).unwrap());
        assert_eq!(target.source, "# Canonical Lens\n");
        fs::remove_file(link).expect("test symlink should be removable");
        fs::remove_file(document).expect("test fixture should be removable");
    }

    fn temporary_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{}-{}", std::process::id(), name))
    }
}
