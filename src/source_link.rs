use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};

const VSCODE_PATH_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

pub(crate) struct SourceLinkResolver {
    document_root: PathBuf,
}

impl SourceLinkResolver {
    pub(crate) fn new(document_root: PathBuf) -> Self {
        debug_assert!(document_root.is_absolute());
        Self { document_root }
    }

    pub(crate) fn resolve(&self, current_document: &Path, destination: &str) -> Option<String> {
        let current_document = identifier_relative_to(&self.document_root, current_document)?;
        let target_identifier = normalize_relative_link(&current_document, destination)?;
        let mut candidate = self.document_root.clone();
        let components = target_identifier.split('/').collect::<Vec<_>>();

        for (index, component) in components.iter().enumerate() {
            if component.starts_with('.') {
                return None;
            }
            candidate.push(component);
            let metadata = fs::symlink_metadata(&candidate).ok()?;
            if metadata.file_type().is_symlink() {
                return None;
            }
            let is_target = index + 1 == components.len();
            if (!is_target && !metadata.is_dir()) || (is_target && !metadata.is_file()) {
                return None;
            }
        }

        File::open(&candidate).ok()?;
        let canonical_target = fs::canonicalize(candidate).ok()?;
        canonical_target
            .starts_with(&self.document_root)
            .then(|| vscode_url(&canonical_target))
            .flatten()
    }
}

pub(crate) fn normalize_relative_link(current_document: &str, destination: &str) -> Option<String> {
    if !has_valid_percent_encoding(destination) {
        return None;
    }
    let decoded = percent_decode_str(destination).decode_utf8().ok()?;
    if decoded.is_empty()
        || decoded.starts_with('/')
        || decoded.contains('\\')
        || has_uri_scheme(&decoded)
    {
        return None;
    }

    let mut components = current_document.split('/').collect::<Vec<_>>();
    components.pop();

    for component in decoded.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            component => components.push(component),
        }
    }

    (!components.is_empty()).then(|| components.join("/"))
}

fn has_valid_percent_encoding(destination: &str) -> bool {
    let bytes = destination.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return false;
            }
            index += 3;
        } else {
            index += 1;
        }
    }
    true
}

fn identifier_relative_to(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()?
        .to_str()
        .map(|identifier| identifier.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn has_uri_scheme(destination: &str) -> bool {
    let Some((scheme, _)) = destination.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, character)| {
            if index == 0 {
                character.is_ascii_alphabetic()
            } else {
                character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
            }
        })
}

fn vscode_url(path: &Path) -> Option<String> {
    let mut normalized = path.to_str()?.replace('\\', "/");
    if let Some(verbatim) = normalized.strip_prefix("//?/") {
        normalized = verbatim
            .strip_prefix("UNC/")
            .map_or_else(|| verbatim.to_owned(), |unc| format!("//{unc}"));
    }
    let normalized = normalized.strip_prefix('/').unwrap_or(&normalized);
    Some(format!(
        "vscode://file/{}",
        utf8_percent_encode(normalized, VSCODE_PATH_ENCODE_SET)
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use super::{normalize_relative_link, vscode_url, SourceLinkResolver};

    struct Fixture {
        root: PathBuf,
        outside_file: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir()
                .join(format!("lens-source-link-{}-{name}", std::process::id()));
            let outside_file = root.with_extension("outside.rs");
            fs::create_dir_all(root.join("docs")).expect("source-link fixture should create docs");
            fs::create_dir_all(root.join("src"))
                .expect("source-link fixture should create source directory");
            fs::write(root.join("docs/design.md"), "# Design")
                .expect("source-link fixture should create its document");
            fs::write(&outside_file, "outside")
                .expect("source-link fixture should create its outside file");
            Self { root, outside_file }
        }

        fn write(&self, relative: &str) -> PathBuf {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("source-link fixture should create file parent");
            }
            fs::write(&path, "source").expect("source-link fixture file should be writable");
            path
        }

        fn resolver(&self) -> SourceLinkResolver {
            SourceLinkResolver::new(
                fs::canonicalize(&self.root).expect("fixture root should canonicalize"),
            )
        }

        fn document(&self) -> PathBuf {
            fs::canonicalize(self.root.join("docs/design.md"))
                .expect("fixture document should canonicalize")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
            let _ = fs::remove_file(&self.outside_file);
        }
    }

    #[test]
    fn source_link_inside_root_then_emits_vscode_file_url() {
        // Arrange
        let fixture = Fixture::new("inside-root");
        fixture.write("src/lib.rs");
        let resolver = fixture.resolver();
        #[cfg(not(windows))]
        let canonical_source =
            fs::canonicalize(fixture.root.join("src/lib.rs")).expect("source should canonicalize");

        // Act
        let resolved = resolver.resolve(&fixture.document(), "../src/lib.rs");

        // Assert
        let resolved = resolved.expect("qualifying source should resolve");
        assert!(resolved.starts_with("vscode://file/"));
        assert!(resolved.ends_with("/src/lib.rs"));
        #[cfg(not(windows))]
        assert_eq!(
            resolved,
            format!("vscode://file{}", canonical_source.display())
        );
    }

    #[test]
    fn source_link_with_spaces_then_percent_encodes_canonical_path() {
        // Arrange
        let fixture = Fixture::new("encoded-path");
        fixture.write("src/source file-測試.rs");
        let resolver = fixture.resolver();

        // Act
        let resolved = resolver
            .resolve(
                &fixture.document(),
                "../src/source%20file-%E6%B8%AC%E8%A9%A6.rs",
            )
            .expect("qualifying source should resolve");

        // Assert
        assert!(resolved.ends_with("/src/source%20file-%E6%B8%AC%E8%A9%A6.rs"));
    }

    #[test]
    fn source_link_outside_root_then_omits_vscode_url() {
        // Arrange
        let fixture = Fixture::new("outside-root");
        fixture.write("src/inside.rs");
        let resolver = fixture.resolver();
        let outside_name = fixture
            .outside_file
            .file_name()
            .expect("outside file should have a name")
            .to_string_lossy();

        // Act
        let inside = resolver.resolve(&fixture.document(), "../src/inside.rs");
        let outside = resolver.resolve(&fixture.document(), &format!("../../{}", outside_name));

        // Assert
        assert!(inside.is_some());
        assert!(outside.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn source_link_through_symlink_then_omits_vscode_url() {
        use std::os::unix::fs::symlink;

        // Arrange
        let fixture = Fixture::new("symlink");
        let source = fixture.write("src/real.rs");
        let file_link = fixture.root.join("src/linked.rs");
        let directory_link = fixture.root.join("linked-src");
        symlink(source, file_link).expect("source-link fixture should create file symlink");
        symlink(fixture.root.join("src"), directory_link)
            .expect("source-link fixture should create directory symlink");
        let resolver = fixture.resolver();

        // Act
        let direct = resolver.resolve(&fixture.document(), "../src/real.rs");
        let linked_file = resolver.resolve(&fixture.document(), "../src/linked.rs");
        let linked_directory = resolver.resolve(&fixture.document(), "../linked-src/real.rs");

        // Assert
        assert!(direct.is_some());
        assert!(linked_file.is_none());
        assert!(linked_directory.is_none());
    }

    #[test]
    fn source_link_beneath_hidden_directory_then_omits_vscode_url() {
        // Arrange
        let fixture = Fixture::new("hidden");
        fixture.write("src/visible.rs");
        fixture.write(".hidden/source.rs");
        fixture.write("src/.hidden.rs");
        let resolver = fixture.resolver();

        // Act
        let visible = resolver.resolve(&fixture.document(), "../src/visible.rs");
        let hidden_directory = resolver.resolve(&fixture.document(), "../.hidden/source.rs");
        let hidden_file = resolver.resolve(&fixture.document(), "../src/.hidden.rs");

        // Assert
        assert!(visible.is_some());
        assert!(hidden_directory.is_none());
        assert!(hidden_file.is_none());
    }

    #[test]
    fn missing_or_directory_source_link_then_omits_vscode_url() {
        // Arrange
        let fixture = Fixture::new("non-file");
        fixture.write("src/visible.rs");
        let resolver = fixture.resolver();

        // Act
        let visible = resolver.resolve(&fixture.document(), "../src/visible.rs");
        let missing = resolver.resolve(&fixture.document(), "../src/missing.rs");
        let directory = resolver.resolve(&fixture.document(), "../src");

        // Assert
        assert!(visible.is_some());
        assert!(missing.is_none());
        assert!(directory.is_none());
    }

    #[test]
    fn absolute_source_link_then_omits_vscode_url() {
        // Arrange
        let fixture = Fixture::new("absolute");
        fixture.write("src/visible.rs");
        let resolver = fixture.resolver();

        // Act
        let visible = resolver.resolve(&fixture.document(), "../src/visible.rs");
        let absolute = resolver.resolve(&fixture.document(), "/etc/hosts");
        let windows_absolute = resolver.resolve(&fixture.document(), r"C:\source\main.rs");

        // Assert
        assert!(visible.is_some());
        assert!(absolute.is_none());
        assert!(windows_absolute.is_none());
    }

    #[test]
    fn encoded_relative_link_then_normalizes_without_suffix() {
        // Arrange
        let current_document = "docs/design.md";

        // Act
        let normalized = normalize_relative_link(
            current_document,
            "../src/source%20file-%E6%B8%AC%E8%A9%A6.rs",
        );

        // Assert
        assert_eq!(normalized.as_deref(), Some("src/source file-測試.rs"));
    }

    #[test]
    fn invalidly_encoded_source_link_then_omits_vscode_url() {
        // Arrange
        let fixture = Fixture::new("invalid-encoding");
        fixture.write("src/visible.rs");
        fixture.write("src/bad%ZZ.rs");
        let resolver = fixture.resolver();

        // Act
        let visible = resolver.resolve(&fixture.document(), "../src/visible.rs");
        let invalid = resolver.resolve(&fixture.document(), "../src/bad%ZZ.rs");

        // Assert
        assert!(visible.is_some());
        assert!(invalid.is_none());
    }

    #[test]
    fn windows_path_then_uses_forward_separators_and_preserves_drive_letter() {
        // Arrange
        let path = Path::new(r"C:\Users\Ada Lovelace\專案\main.rs");

        // Act
        let url = vscode_url(path);

        // Assert
        assert_eq!(
            url.as_deref(),
            Some("vscode://file/C:/Users/Ada%20Lovelace/%E5%B0%88%E6%A1%88/main.rs")
        );
    }
}
