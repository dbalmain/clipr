use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};

use super::clip::ClipEntry;

/// Search case sensitivity mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Smart case: case-insensitive unless query contains uppercase letters
    /// This is the default mode
    SmartCase,
    /// Case-sensitive search (always)
    CaseSensitive,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::SmartCase
    }
}

/// Wrapper around nucleo for fuzzy searching clipboard entries
pub struct SearchIndex {
    matcher: Matcher,
    mode: SearchMode,
}

impl SearchIndex {
    /// Create a new search index with default mode (SmartCase)
    pub fn new() -> Self {
        SearchIndex {
            matcher: Matcher::new(Config::DEFAULT),
            mode: SearchMode::default(),
        }
    }

    /// Set the search mode
    pub fn set_mode(&mut self, mode: SearchMode) {
        self.mode = mode;
    }

    /// Get the current search mode
    pub fn mode(&self) -> SearchMode {
        self.mode
    }

    /// Toggle between SmartCase and CaseSensitive modes
    /// (for future Ctrl-? keybinding)
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            SearchMode::SmartCase => SearchMode::CaseSensitive,
            SearchMode::CaseSensitive => SearchMode::SmartCase,
        };
    }

    /// Search clips by query string
    /// Returns vector of (clip_id, score) tuples, sorted by score descending
    pub fn search(&mut self, clips: &[ClipEntry], query: &str) -> Vec<(u64, u32)> {
        if query.is_empty() {
            // Return all clips with max score if query is empty
            return clips.iter().map(|c| (c.id, u32::MAX)).collect();
        }

        // Map SearchMode to nucleo's CaseMatching
        let case_matching = match self.mode {
            SearchMode::SmartCase => CaseMatching::Smart,
            SearchMode::CaseSensitive => CaseMatching::Respect,
        };

        // Create pattern from query
        let pattern = Pattern::parse(query, case_matching, Normalization::Smart);

        let mut results: Vec<(u64, u32)> = clips
            .iter()
            .filter_map(|clip| {
                // Build searchable text from clip
                let search_text = clip.searchable_text();

                // Convert to UTF-32 for nucleo
                let utf32_text = Utf32String::from(search_text.as_str());

                // Match pattern against clip text
                pattern
                    .score(utf32_text.slice(..), &mut self.matcher)
                    .map(|score| (clip.id, score))
            })
            .collect();

        // Sort by score descending (higher scores = better matches)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipEntry {
    /// Get searchable text representation of this clip
    /// Used by fuzzy search to match against
    fn searchable_text(&self) -> String {
        use super::clip::ClipContent;

        let mut text = String::new();

        // Include name if present
        if let Some(name) = &self.name {
            text.push_str(name);
            text.push(' ');
        }

        // Include description if present
        if let Some(description) = &self.description {
            text.push_str(description);
            text.push(' ');
        }

        // Include content preview
        match &self.content {
            ClipContent::Text(t) => {
                text.push_str(t);
            }
            ClipContent::Image { .. } => {
                text.push_str("[image]");
            }
            ClipContent::File { path, .. } => {
                text.push_str("[file: ");
                if let Some(filename) = path.file_name() {
                    text.push_str(&filename.to_string_lossy());
                }
                text.push(']');
            }
        }

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::clip::ClipContent;

    #[test]
    fn test_search_empty_query() {
        let mut index = SearchIndex::new();
        let clips = vec![
            ClipEntry::new_text("hello world".to_string()),
            ClipEntry::new_text("goodbye world".to_string()),
        ];

        let results = index.search(&clips, "");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_basic() {
        let mut index = SearchIndex::new();
        let clips = vec![
            ClipEntry::new_text("hello world".to_string()),
            ClipEntry::new_text("goodbye world".to_string()),
            ClipEntry::new_text("unrelated".to_string()),
        ];

        let results = index.search(&clips, "hello");
        assert!(results.len() >= 1);
        assert_eq!(results[0].0, clips[0].id); // First result should be "hello world"
    }

    #[test]
    fn test_search_smart_case() {
        let mut index = SearchIndex::new();
        let clips = vec![ClipEntry::new_text("Hello World".to_string())];

        // Lowercase query should match (case-insensitive)
        let results = index.search(&clips, "hello");
        assert_eq!(results.len(), 1);

        // Uppercase query should be case-sensitive in smart mode
        index.set_mode(SearchMode::SmartCase);
        let results = index.search(&clips, "Hello");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_toggle_mode() {
        let mut index = SearchIndex::new();
        assert_eq!(index.mode(), SearchMode::SmartCase);

        index.toggle_mode();
        assert_eq!(index.mode(), SearchMode::CaseSensitive);

        index.toggle_mode();
        assert_eq!(index.mode(), SearchMode::SmartCase);
    }

    #[test]
    fn test_search_with_name() {
        let mut index = SearchIndex::new();
        let mut clip = ClipEntry::new_text("content".to_string());
        clip.name = Some("my_clip".to_string());

        let clips = vec![clip];
        let results = index.search(&clips, "my_clip");
        assert_eq!(results.len(), 1);
    }
}
