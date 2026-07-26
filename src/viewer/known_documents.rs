use std::collections::BTreeMap;

pub(super) struct KnownDocuments {
    document_indices: BTreeMap<String, usize>,
}

impl KnownDocuments {
    pub(super) fn new(document_ids: impl IntoIterator<Item = (String, usize)>) -> Self {
        Self {
            document_indices: document_ids.into_iter().collect(),
        }
    }

    pub(super) fn index(&self, identifier: &str) -> Option<usize> {
        self.document_indices.get(identifier).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::KnownDocuments;

    #[test]
    fn known_identifier_then_returns_document_index() {
        // Arrange
        let known_documents = KnownDocuments::new([
            ("README.md".to_owned(), 0),
            ("guides/intro.md".to_owned(), 1),
        ]);

        // Act
        let document_index = known_documents.index("guides/intro.md");

        // Assert
        assert_eq!(document_index, Some(1));
    }

    #[test]
    fn unknown_identifier_then_returns_no_document_index() {
        // Arrange
        let known_documents = KnownDocuments::new([("README.md".to_owned(), 0)]);

        // Act
        let document_index = known_documents.index("../private.md");

        // Assert
        assert_eq!(document_index, None);
    }
}
