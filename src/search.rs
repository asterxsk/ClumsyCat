/// Search mode state for the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMode {
    Inactive,
    Active {
        query: String,
        filtered_indices: Vec<usize>,
        current_match_index: usize,
    },
}

impl SearchMode {
    /// Create an active search mode with the given query and entries
    pub fn new_active(query: String, entries: &[impl AsRef<str>]) -> Self {
        let filtered_indices = filter_entries(entries, &query);
        SearchMode::Active {
            query,
            filtered_indices,
            current_match_index: 0,
        }
    }

    /// Check if search is currently active
    pub fn is_active(&self) -> bool {
        matches!(self, SearchMode::Active { .. })
    }

    /// Get the query string if active
    pub fn query(&self) -> Option<&str> {
        match self {
            SearchMode::Active { query, .. } => Some(query),
            SearchMode::Inactive => None,
        }
    }

    /// Get the current match count if active
    pub fn match_count(&self) -> usize {
        match self {
            SearchMode::Active {
                filtered_indices, ..
            } => filtered_indices.len(),
            SearchMode::Inactive => 0,
        }
    }

    /// Get the current matched index, if any
    pub fn current_match(&self) -> Option<usize> {
        match self {
            SearchMode::Active {
                filtered_indices,
                current_match_index,
                ..
            } => {
                if *current_match_index < filtered_indices.len() {
                    Some(filtered_indices[*current_match_index])
                } else {
                    None
                }
            }
            SearchMode::Inactive => None,
        }
    }
}

/// Filter entries by a query string using case-insensitive substring matching
/// Returns indices of matching entries
pub fn filter_entries(entries: &[impl AsRef<str>], query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter_map(|(idx, entry)| {
            if entry.as_ref().to_lowercase().contains(&query_lower) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_entries_basic() {
        let entries = vec!["Document.txt", "Docs", "Design", "Picture.png"];
        let results = filter_entries(&entries, "doc");
        assert_eq!(results, vec![0, 1]); // "Document.txt" and "Docs"
    }

    #[test]
    fn test_filter_entries_case_insensitive() {
        let entries = vec!["README", "Readme", "readme"];
        let results = filter_entries(&entries, "readme");
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[test]
    fn test_filter_entries_no_matches() {
        let entries = vec!["file1", "file2", "file3"];
        let results = filter_entries(&entries, "xyz");
        assert_eq!(results, Vec::<usize>::new());
    }

    #[test]
    fn test_filter_entries_empty_query() {
        let entries = vec!["a", "b", "c"];
        let results = filter_entries(&entries, "");
        assert_eq!(results, vec![0, 1, 2]); // Empty query matches all
    }

    #[test]
    fn test_search_mode_new_active() {
        let entries = vec!["test1", "test2", "other"];
        let search = SearchMode::new_active("test".to_string(), &entries);
        assert!(search.is_active());
        assert_eq!(search.query(), Some("test"));
        assert_eq!(search.match_count(), 2);
    }

    #[test]
    fn test_search_mode_current_match() {
        let entries = vec!["apple", "apricot", "banana"];
        let search = SearchMode::new_active("ap".to_string(), &entries);
        assert_eq!(search.current_match(), Some(0));

        // After advancing, would point to index 1
        if let SearchMode::Active {
            current_match_index,
            filtered_indices,
            ..
        } = search
        {
            if current_match_index + 1 < filtered_indices.len() {
                let next_index = current_match_index + 1;
                assert_eq!(filtered_indices[next_index], 1);
            }
        }
    }

    #[test]
    fn test_search_mode_inactive() {
        let search: SearchMode = SearchMode::Inactive;
        assert!(!search.is_active());
        assert_eq!(search.query(), None);
        assert_eq!(search.match_count(), 0);
        assert_eq!(search.current_match(), None);
    }
}
