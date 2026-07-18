use std::collections::{BTreeMap, BTreeSet};

pub(super) const MAX_QUERY_BYTES: usize = 256;
pub(super) const RESULT_LIMIT: usize = 50;

pub(super) struct DocumentCatalog {
    document_ids: BTreeMap<String, usize>,
}

pub(super) struct NavigationRequest {
    query: Option<String>,
    page: Option<usize>,
}

pub(super) enum CatalogPage {
    Results(CatalogResults),
    QueryTooLong { query: String },
}

pub(super) struct CatalogResults {
    pub(super) query: String,
    pub(super) page: usize,
    pub(super) total: usize,
    pub(super) entries: Vec<CatalogEntry>,
}

pub(super) struct CatalogEntry {
    pub(super) identifier: String,
    pub(super) document_index: usize,
}

impl DocumentCatalog {
    pub(super) fn new(document_ids: impl IntoIterator<Item = (String, usize)>) -> Self {
        Self {
            document_ids: document_ids.into_iter().collect(),
        }
    }

    pub(super) fn known_document_index(&self, identifier: &str) -> Option<usize> {
        self.document_ids.get(identifier).copied()
    }

    pub(super) fn known_document_ids(&self) -> BTreeSet<String> {
        self.document_ids.keys().cloned().collect()
    }

    pub(super) fn search(&self, request: &NavigationRequest) -> CatalogPage {
        let raw_query = request.query.as_deref().unwrap_or_default();
        if raw_query.len() > MAX_QUERY_BYTES {
            return CatalogPage::QueryTooLong {
                query: raw_query.to_owned(),
            };
        }
        let query = raw_query.trim();

        let normalized_query = query.to_lowercase();
        let matches = |identifier: &String| identifier.to_lowercase().contains(&normalized_query);
        let total = self
            .document_ids
            .keys()
            .filter(|identifier| matches(identifier))
            .count();
        let page_count = total / RESULT_LIMIT + usize::from(total % RESULT_LIMIT != 0);
        let requested_page = request.page.filter(|page| *page > 0).unwrap_or(1);
        let page = if requested_page <= page_count.max(1) {
            requested_page
        } else {
            1
        };
        let first_result = (page - 1) * RESULT_LIMIT;
        let entries = self
            .document_ids
            .iter()
            .filter(|(identifier, _)| matches(identifier))
            .skip(first_result)
            .take(RESULT_LIMIT)
            .map(|(identifier, &document_index)| CatalogEntry {
                identifier: identifier.clone(),
                document_index,
            })
            .collect();

        CatalogPage::Results(CatalogResults {
            query: query.to_owned(),
            page,
            total,
            entries,
        })
    }
}

impl NavigationRequest {
    pub(super) fn from_raw_query(raw_query: Option<&str>) -> Self {
        let mut query = None;
        let mut page = None;

        if let Some(raw_query) = raw_query {
            for (name, value) in form_urlencoded::parse(raw_query.as_bytes()) {
                match name.as_ref() {
                    "query" => query = Some(value.into_owned()),
                    "page" => page = value.parse().ok(),
                    _ => {}
                }
            }
        }

        Self { query, page }
    }
}

impl CatalogResults {
    pub(super) fn has_previous_page(&self) -> bool {
        self.page > 1
    }

    pub(super) fn has_next_page(&self) -> bool {
        self.page * RESULT_LIMIT < self.total
    }

    pub(super) fn page_query(&self, page: usize) -> String {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer.append_pair("query", &self.query);
        serializer.append_pair("page", &page.to_string());
        serializer.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{CatalogPage, DocumentCatalog, NavigationRequest, RESULT_LIMIT};

    fn test_catalog(identifiers: impl IntoIterator<Item = String>) -> DocumentCatalog {
        DocumentCatalog::new(
            identifiers
                .into_iter()
                .enumerate()
                .map(|(index, identifier)| (identifier, index)),
        )
    }

    #[test]
    fn more_than_result_limit_then_returns_first_page_and_next_link() {
        // Arrange
        let catalog = test_catalog((0..=50).map(|index| format!("guides/{index:03}.md")));
        let request = NavigationRequest::from_raw_query(None);

        // Act
        let CatalogPage::Results(results) = catalog.search(&request) else {
            panic!("an empty query should return results");
        };

        // Assert
        assert_eq!(results.entries.len(), RESULT_LIMIT);
        assert_eq!(results.entries[0].identifier, "guides/000.md");
        assert_eq!(results.entries[49].identifier, "guides/049.md");
        assert!(results.has_next_page());
    }

    #[test]
    fn case_insensitive_query_then_returns_matching_identifier_page() {
        // Arrange
        let catalog = test_catalog([
            "README.md".to_owned(),
            "guides/Alpha.md".to_owned(),
            "guides/beta.md".to_owned(),
        ]);
        let request = NavigationRequest::from_raw_query(Some("query=ALPHA&page=1"));

        // Act
        let CatalogPage::Results(results) = catalog.search(&request) else {
            panic!("a short query should return results");
        };

        // Assert
        assert_eq!(results.total, 1);
        assert_eq!(results.entries.len(), 1);
        assert_eq!(results.entries[0].identifier, "guides/Alpha.md");
        assert_eq!(results.page_query(1), "query=ALPHA&page=1");
    }

    #[test]
    fn over_limit_query_then_returns_limit_result_without_entries() {
        // Arrange
        let catalog = test_catalog(["README.md".to_owned()]);
        let request =
            NavigationRequest::from_raw_query(Some(&format!("query={}", "a".repeat(257))));

        // Act
        let result = catalog.search(&request);

        // Assert
        assert!(matches!(result, CatalogPage::QueryTooLong { .. }));
    }

    #[test]
    fn invalid_page_then_returns_first_matching_page() {
        // Arrange
        let catalog = test_catalog((0..=50).map(|index| format!("guides/{index:03}.md")));
        let request = NavigationRequest::from_raw_query(Some("page=99"));

        // Act
        let CatalogPage::Results(results) = catalog.search(&request) else {
            panic!("an empty query should return results");
        };

        // Assert
        assert_eq!(results.page, 1);
        assert_eq!(results.entries[0].identifier, "guides/000.md");
        assert!(!results.has_previous_page());
    }

    #[test]
    fn malformed_page_then_returns_first_matching_page() {
        // Arrange
        let catalog = test_catalog(["README.md".to_owned(), "guides/intro.md".to_owned()]);
        let request = NavigationRequest::from_raw_query(Some("page=not-a-number"));

        // Act
        let CatalogPage::Results(results) = catalog.search(&request) else {
            panic!("an empty query should return results");
        };

        // Assert
        assert_eq!(results.page, 1);
        assert_eq!(results.entries[0].identifier, "README.md");
    }
}
