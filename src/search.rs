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
    /// Check if search is currently active
    pub fn is_active(&self) -> bool {
        matches!(self, SearchMode::Active { .. })
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

    /// Get the current query string, if active
    #[allow(dead_code)]
    pub fn query(&self) -> Option<&str> {
        match self {
            SearchMode::Active { query, .. } => Some(query.as_str()),
            SearchMode::Inactive => None,
        }
    }
}

/// Fuzzy match scoring for command bar filtering.
/// Returns Some(score) if query matches target, None otherwise.
/// Higher scores = better matches.
pub fn _fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query = query.to_lowercase();
    let target = target.to_lowercase();

    let mut score = 0i32;
    let mut query_chars = query.chars().peekable();
    let mut last_match_idx: Option<usize> = None;

    for (i, tc) in target.chars().enumerate() {
        if let Some(&qc) = query_chars.peek() {
            if tc == qc {
                query_chars.next();
                score += 10; // base match score

                // Bonus for matching at start
                if i == 0 {
                    score += 15;
                }

                // Bonus for consecutive matches
                if let Some(last) = last_match_idx {
                    if i == last + 1 {
                        score += 5;
                    }
                }

                last_match_idx = Some(i);
            }
        }
    }

    // All query chars must be consumed for a match
    if query_chars.peek().is_none() {
        // Prefer shorter targets (penalty for length)
        Some(score - target.len() as i32)
    } else {
        None
    }
}

/// Filter commands using fuzzy matching, returning (index, score) pairs sorted by score descending
pub fn _filter_commands_fuzzy<'a>(
    commands: impl Iterator<Item = (usize, &'a str)>,
    query: &str,
) -> Vec<(usize, i32)> {
    let mut matches: Vec<(usize, i32)> = commands
        .filter_map(|(idx, name)| _fuzzy_score(query, name).map(|score| (idx, score)))
        .collect();
    matches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by score descending
    matches
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
    fn test_search_mode_current_match() {
        let search: SearchMode = SearchMode::Inactive;
        assert!(!search.is_active());
        assert_eq!(search.match_count(), 0);
        assert_eq!(search.current_match(), None);
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
