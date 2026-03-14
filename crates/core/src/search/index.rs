use std::cell::RefCell;

use napi::Either;
use napi_derive::napi;
use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher, Utf32String};

use super::{
    PrecomputedSearch, SearchOptions, SearchResult, compute_char_mask, resolve_case_matching,
    search_over_precomputed,
};

/// A persistent fuzzy search index backed by Rust-side data.
///
/// Holds items in memory on the Rust side, avoiding repeated FFI overhead
/// for applications that search the same dataset multiple times.
/// Pre-computes Utf32String representations for each item, eliminating
/// per-search string conversion overhead.
/// Memory is freed when the JavaScript garbage collector collects the instance
/// or when `destroy()` is called explicitly.
#[napi]
pub struct FuzzyIndex {
    items: Vec<String>,
    utf32_items: Vec<Utf32String>,
    char_masks: Vec<u64>,
    matcher: RefCell<Matcher>,
}

#[napi]
impl FuzzyIndex {
    /// Create a new FuzzyIndex from an array of strings.
    #[napi(constructor)]
    pub fn new(items: Vec<String>) -> Self {
        let utf32_items: Vec<Utf32String> = items
            .iter()
            .map(|s| Utf32String::from(s.as_str()))
            .collect();
        let char_masks: Vec<u64> = items.iter().map(|s| compute_char_mask(s)).collect();
        Self {
            items,
            utf32_items,
            char_masks,
            matcher: RefCell::new(Matcher::new(Config::DEFAULT)),
        }
    }

    /// Return the number of items in the index.
    #[napi(getter)]
    pub fn size(&self) -> u32 {
        self.items.len() as u32
    }

    /// Search the index for items matching the query.
    ///
    /// Returns matches sorted by score (best match first).
    /// Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
    #[napi]
    pub fn search(
        &self,
        query: String,
        options: Option<Either<u32, SearchOptions>>,
    ) -> Vec<SearchResult> {
        let (max_results, min_score, include_positions, case_matching) = match options {
            Some(Either::A(max)) => (Some(max), None, false, CaseMatching::Smart),
            Some(Either::B(opts)) => (
                opts.max_results,
                opts.min_score,
                opts.include_positions.unwrap_or(false),
                resolve_case_matching(opts.is_case_sensitive),
            ),
            None => (None, None, false, CaseMatching::Smart),
        };
        self.search_impl(
            &query,
            max_results,
            min_score,
            include_positions,
            case_matching,
        )
    }

    /// Find the closest matching string in the index.
    ///
    /// Returns the best match, or null if no match is found.
    /// If minScore is provided, returns null when the best match scores below the threshold.
    #[napi]
    pub fn closest(&self, query: String, min_score: Option<f64>) -> Option<String> {
        let results = self.search_impl(&query, Some(1), min_score, false, CaseMatching::Smart);
        results.into_iter().next().map(|r| r.item)
    }

    /// Add a single item to the index.
    #[napi]
    pub fn add(&mut self, item: String) {
        self.utf32_items.push(Utf32String::from(item.as_str()));
        self.char_masks.push(compute_char_mask(&item));
        self.items.push(item);
    }

    /// Add multiple items to the index at once.
    #[napi]
    pub fn add_many(&mut self, items: Vec<String>) {
        for item in &items {
            self.utf32_items.push(Utf32String::from(item.as_str()));
            self.char_masks.push(compute_char_mask(item));
        }
        self.items.extend(items);
    }

    /// Remove the item at the given index.
    ///
    /// Returns false if the index is out of bounds.
    #[napi]
    pub fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        if idx < self.items.len() {
            self.items.swap_remove(idx);
            self.utf32_items.swap_remove(idx);
            self.char_masks.swap_remove(idx);
            true
        } else {
            false
        }
    }

    /// Free the internal data. After calling this, the index is empty.
    #[napi]
    pub fn destroy(&mut self) {
        self.items = Vec::new();
        self.utf32_items = Vec::new();
        self.char_masks = Vec::new();
    }

    fn search_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<SearchResult> {
        let ctx = PrecomputedSearch {
            items: &self.items,
            utf32_items: &self.utf32_items,
            char_masks: &self.char_masks,
            matcher: &self.matcher,
        };
        search_over_precomputed(
            query,
            &ctx,
            max_results,
            min_score,
            include_positions,
            case_matching,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_size() {
        let index = FuzzyIndex::new(vec!["apple".into(), "banana".into()]);
        assert_eq!(index.size(), 2);
    }

    #[test]
    fn test_empty_index() {
        let index = FuzzyIndex::new(vec![]);
        assert_eq!(index.size(), 0);
        let results = index.search_impl("test", None, None, false, CaseMatching::Smart);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_basic() {
        let index = FuzzyIndex::new(vec![
            "TypeScript".into(),
            "JavaScript".into(),
            "Python".into(),
        ]);
        let results = index.search_impl("typscript", None, None, false, CaseMatching::Smart);
        assert!(!results.is_empty());
        assert_eq!(results[0].item, "TypeScript");
    }

    #[test]
    fn test_search_max_results() {
        let index = FuzzyIndex::new(vec![
            "apple".into(),
            "application".into(),
            "appetizer".into(),
        ]);
        let results = index.search_impl("app", Some(2), None, false, CaseMatching::Smart);
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_search_min_score() {
        let index = FuzzyIndex::new(vec!["apple".into(), "xyz".into()]);
        let results = index.search_impl("apple", None, Some(0.5), false, CaseMatching::Smart);
        for r in &results {
            assert!(r.score >= 0.5);
        }
    }

    #[test]
    fn test_search_with_positions() {
        let index = FuzzyIndex::new(vec!["hello".into()]);
        let results = index.search_impl("hello", None, None, true, CaseMatching::Smart);
        assert!(!results.is_empty());
        assert_eq!(results[0].positions, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_closest() {
        let index = FuzzyIndex::new(vec!["apple".into(), "banana".into()]);
        let result = index.closest("app".into(), None);
        assert_eq!(result, Some("apple".into()));
    }

    #[test]
    fn test_closest_with_min_score() {
        let index = FuzzyIndex::new(vec!["xyz".into()]);
        let result = index.closest("hello".into(), Some(0.99));
        assert!(result.is_none());
    }

    #[test]
    fn test_add() {
        let mut index = FuzzyIndex::new(vec!["apple".into()]);
        assert_eq!(index.size(), 1);
        index.add("banana".into());
        assert_eq!(index.size(), 2);
        let result = index.closest("banana".into(), None);
        assert_eq!(result, Some("banana".into()));
    }

    #[test]
    fn test_add_many() {
        let mut index = FuzzyIndex::new(vec![]);
        index.add_many(vec!["apple".into(), "banana".into(), "grape".into()]);
        assert_eq!(index.size(), 3);
    }

    #[test]
    fn test_remove() {
        let mut index = FuzzyIndex::new(vec!["apple".into(), "banana".into(), "grape".into()]);
        assert!(index.remove(1)); // remove "banana"
        assert_eq!(index.size(), 2);
        assert!(!index.remove(10)); // out of bounds
    }

    #[test]
    fn test_remove_swap_semantics() {
        let mut index = FuzzyIndex::new(vec!["a".into(), "b".into(), "c".into()]);
        index.remove(0); // removes "a", swaps "c" into position 0
        assert_eq!(index.size(), 2);
        // After swap_remove(0): ["c", "b"]
        let results = index.search_impl("c", None, None, false, CaseMatching::Smart);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_destroy() {
        let mut index = FuzzyIndex::new(vec!["apple".into(), "banana".into()]);
        index.destroy();
        assert_eq!(index.size(), 0);
        let results = index.search_impl("apple", None, None, false, CaseMatching::Smart);
        assert!(results.is_empty());
    }

    #[test]
    fn test_scores_normalized() {
        let index = FuzzyIndex::new(vec!["apple".into(), "application".into(), "banana".into()]);
        let results = index.search_impl("apple", None, None, false, CaseMatching::Smart);
        for r in &results {
            assert!(r.score >= 0.0 && r.score <= 1.0);
        }
    }

    #[test]
    fn test_results_sorted_descending() {
        let index = FuzzyIndex::new(vec![
            "apple".into(),
            "application".into(),
            "appetizer".into(),
            "banana".into(),
        ]);
        let results = index.search_impl("apple", None, None, false, CaseMatching::Smart);
        for window in results.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }

    #[test]
    fn test_consistent_with_standalone_search() {
        let items = vec![
            "apple".into(),
            "application".into(),
            "banana".into(),
            "grape".into(),
        ];
        let index = FuzzyIndex::new(items.clone());
        let index_results = index.search_impl("apple", None, None, false, CaseMatching::Smart);
        let standalone_results = crate::search::search_impl(
            "apple".into(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert_eq!(index_results.len(), standalone_results.len());
        for (a, b) in index_results.iter().zip(standalone_results.iter()) {
            assert_eq!(a.item, b.item);
            assert!((a.score - b.score).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_case_sensitive_search() {
        let index = FuzzyIndex::new(vec!["Apple".into(), "apple".into(), "APPLE".into()]);
        let results = index.search_impl("apple", None, Some(1.0), false, CaseMatching::Respect);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "apple");
    }

    #[test]
    fn test_smart_case_search() {
        let index = FuzzyIndex::new(vec!["Apple".into(), "apple".into(), "APPLE".into()]);
        // All-lowercase query with smart case matches all
        let results = index.search_impl("apple", None, Some(1.0), false, CaseMatching::Smart);
        assert_eq!(results.len(), 3);
    }
}
