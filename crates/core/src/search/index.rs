use napi::bindgen_prelude::{AsyncTask, Buffer};
use napi::{Either, Task};
use napi_derive::napi;
use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::FuzzyIndexCore;
use rapid_fuzzy_core::search::serialization::{
    FUZZY_INDEX_MAGIC, deserialize_items, serialize_items,
};

use super::{IndexSearchResult, SearchOptions, SearchResult, resolve_case_matching};

pub struct BuildFuzzyIndexTask {
    items: Vec<String>,
}

impl Task for BuildFuzzyIndexTask {
    type Output = FuzzyIndexCore;
    type JsValue = FuzzyIndex;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(FuzzyIndexCore::new(std::mem::take(&mut self.items)))
    }

    fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(FuzzyIndex { core: output })
    }
}

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
    core: FuzzyIndexCore,
}

#[napi]
impl FuzzyIndex {
    /// Create a new FuzzyIndex from an array of strings.
    #[napi(constructor)]
    pub fn new(items: Vec<String>) -> Self {
        Self {
            core: FuzzyIndexCore::new(items),
        }
    }

    /// Construct a FuzzyIndex on the libuv thread pool, returning a Promise.
    ///
    /// For large datasets this keeps the JavaScript event loop unblocked during
    /// index construction. The synchronous constructor is fine for small datasets.
    #[napi(ts_return_type = "Promise<FuzzyIndex>")]
    pub fn from_async(items: Vec<String>) -> AsyncTask<BuildFuzzyIndexTask> {
        AsyncTask::new(BuildFuzzyIndexTask { items })
    }

    /// Return the number of items in the index.
    #[napi(getter)]
    pub fn size(&self) -> u32 {
        self.core.size()
    }

    /// Search the index for items matching the query.
    ///
    /// Returns matches sorted by score (best match first).
    /// Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
    ///
    /// The second argument accepts either a number (maxResults shorthand) or a
    /// SearchOptions object.
    #[napi]
    pub fn search(
        &self,
        query: String,
        options: Option<Either<u32, SearchOptions>>,
    ) -> Vec<SearchResult> {
        let (max_results, min_score, include_positions, case_matching, return_all_on_empty) =
            match options {
                Some(Either::A(max)) => (Some(max), None, false, CaseMatching::Smart, false),
                Some(Either::B(opts)) => (
                    opts.max_results,
                    opts.min_score,
                    opts.include_positions.unwrap_or(false),
                    resolve_case_matching(opts.is_case_sensitive),
                    opts.return_all_on_empty.unwrap_or(false),
                ),
                None => (None, None, false, CaseMatching::Smart, false),
            };

        if return_all_on_empty && query.trim().is_empty() {
            let items = self.core.items();
            let limit = max_results.unwrap_or(items.len() as u32) as usize;
            return items
                .iter()
                .enumerate()
                .take(limit)
                .map(|(i, item)| SearchResult {
                    item: item.clone(),
                    score: 1.0,
                    index: i as u32,
                    positions: Vec::new(),
                    match_type: None,
                })
                .collect();
        }

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

    /// Search the index, returning only indices and scores (no item strings).
    ///
    /// This is more efficient than `search()` when you maintain your own data
    /// array and only need the index to look up the original item. Avoids
    /// String cloning overhead for each result.
    ///
    /// The second argument accepts either a number (maxResults shorthand) or a
    /// SearchOptions object.
    #[napi]
    pub fn search_indices(
        &self,
        query: String,
        options: Option<Either<u32, SearchOptions>>,
    ) -> Vec<IndexSearchResult> {
        let (max_results, min_score, include_positions, case_matching, return_all_on_empty) =
            match options {
                Some(Either::A(max)) => (Some(max), None, false, CaseMatching::Smart, false),
                Some(Either::B(opts)) => (
                    opts.max_results,
                    opts.min_score,
                    opts.include_positions.unwrap_or(false),
                    resolve_case_matching(opts.is_case_sensitive),
                    opts.return_all_on_empty.unwrap_or(false),
                ),
                None => (None, None, false, CaseMatching::Smart, false),
            };

        if return_all_on_empty && query.trim().is_empty() {
            let num_items = self.core.size() as usize;
            let limit = max_results.unwrap_or(num_items as u32) as usize;
            return (0..num_items)
                .take(limit)
                .map(|i| IndexSearchResult {
                    index: i as u32,
                    score: 1.0,
                    positions: Vec::new(),
                    match_type: None,
                })
                .collect();
        }

        self.search_indices_impl(
            &query,
            max_results,
            min_score,
            include_positions,
            case_matching,
        )
    }

    /// Add a single item to the index.
    #[napi]
    pub fn add(&mut self, item: String) {
        self.core.add(item);
    }

    /// Add multiple items to the index at once.
    #[napi]
    pub fn add_many(&mut self, items: Vec<String>) {
        self.core.add_many(items);
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    #[napi]
    pub fn remove(&mut self, index: u32) -> bool {
        self.core.remove(index)
    }

    /// Free the internal data. After calling this, the index is empty.
    #[napi]
    pub fn destroy(&mut self) {
        self.core.destroy();
    }

    /// Serialize the index to a compact binary format.
    ///
    /// The returned Buffer can be written to disk, stored in IndexedDB,
    /// or transferred over the network. Use `FuzzyIndex.deserialize()` to
    /// reconstruct the index.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        self.serialize_impl().into()
    }

    /// Reconstruct a FuzzyIndex from a previously serialized Buffer.
    ///
    /// Pre-computes Utf32String and character masks from the stored items,
    /// so the returned index is immediately ready for searching.
    #[napi(factory)]
    pub fn deserialize(data: Buffer) -> napi::Result<Self> {
        Self::deserialize_impl(&data).map_err(napi::Error::from_reason)
    }

    fn serialize_impl(&self) -> Vec<u8> {
        serialize_items(self.core.items(), FUZZY_INDEX_MAGIC)
    }

    fn deserialize_impl(bytes: &[u8]) -> Result<Self, String> {
        let items = deserialize_items(bytes, FUZZY_INDEX_MAGIC)?;
        Ok(Self::new(items))
    }

    fn search_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<SearchResult> {
        self.core
            .search_impl(
                query,
                max_results,
                min_score,
                include_positions,
                case_matching,
            )
            .into_iter()
            .map(SearchResult::from)
            .collect()
    }

    fn search_indices_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<IndexSearchResult> {
        self.core
            .search_indices_impl(
                query,
                max_results,
                min_score,
                include_positions,
                case_matching,
            )
            .into_iter()
            .map(IndexSearchResult::from)
            .collect()
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

    #[test]
    fn test_serialize_roundtrip() {
        let items = vec!["apple".into(), "banana".into(), "cherry".into()];
        let index = FuzzyIndex::new(items);
        let data = index.serialize_impl();
        let restored = FuzzyIndex::deserialize_impl(&data).unwrap();

        assert_eq!(restored.size(), 3);
        let results = restored.search_impl("apple", None, None, false, CaseMatching::Smart);
        assert!(!results.is_empty());
        assert_eq!(results[0].item, "apple");
    }

    #[test]
    fn test_serialize_empty() {
        let index = FuzzyIndex::new(vec![]);
        let data = index.serialize_impl();
        let restored = FuzzyIndex::deserialize_impl(&data).unwrap();
        assert_eq!(restored.size(), 0);
    }

    #[test]
    fn test_serialize_unicode() {
        let items = vec!["café".into(), "naïve".into(), "東京".into()];
        let index = FuzzyIndex::new(items);
        let data = index.serialize_impl();
        let restored = FuzzyIndex::deserialize_impl(&data).unwrap();

        assert_eq!(restored.size(), 3);
        // Verify Unicode strings survived the roundtrip by searching for an exact match.
        let results = restored.search_impl("café", None, None, false, CaseMatching::Smart);
        assert!(!results.is_empty());
        assert_eq!(results[0].item, "café");
    }

    #[test]
    fn test_serialize_search_consistency() {
        let items: Vec<String> = (0..100).map(|i| format!("item_{i}")).collect();
        let original = FuzzyIndex::new(items);
        let data = original.serialize_impl();
        let restored = FuzzyIndex::deserialize_impl(&data).unwrap();

        let orig_results = original.search_impl("item_5", None, None, false, CaseMatching::Smart);
        let rest_results = restored.search_impl("item_5", None, None, false, CaseMatching::Smart);

        assert_eq!(orig_results.len(), rest_results.len());
        for (a, b) in orig_results.iter().zip(rest_results.iter()) {
            assert_eq!(a.item, b.item);
            assert!((a.score - b.score).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_deserialize_invalid_magic() {
        let data = b"XXXX\x01\x00\x00\x00\x00\x00\x00\x00";
        assert!(FuzzyIndex::deserialize_impl(data).is_err());
    }

    #[test]
    fn test_deserialize_truncated() {
        let data = b"RFZI\x01\x00\x00\x00";
        assert!(FuzzyIndex::deserialize_impl(data).is_err());
    }

    #[test]
    fn test_deserialize_bad_version() {
        let data = b"RFZI\xFF\x00\x00\x00\x00\x00\x00\x00";
        assert!(FuzzyIndex::deserialize_impl(data).is_err());
    }

    #[test]
    fn test_search_indices_basic() {
        let index = FuzzyIndex::new(vec![
            "TypeScript".into(),
            "JavaScript".into(),
            "Python".into(),
        ]);
        let results =
            index.search_indices_impl("typscript", None, None, false, CaseMatching::Smart);
        assert!(!results.is_empty());
        assert_eq!(results[0].index, 0); // TypeScript
    }

    #[test]
    fn test_search_indices_no_item_field() {
        let index = FuzzyIndex::new(vec!["hello".into(), "world".into()]);
        let results = index.search_indices_impl("hello", None, None, false, CaseMatching::Smart);
        assert!(!results.is_empty());
        // IndexSearchResult has index and score but no item field
        assert_eq!(results[0].index, 0);
        assert!((results[0].score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_search_indices_consistent_with_search() {
        let items = vec![
            "apple".into(),
            "application".into(),
            "banana".into(),
            "grape".into(),
        ];
        let index = FuzzyIndex::new(items);
        let full = index.search_impl("apple", None, None, false, CaseMatching::Smart);
        let indices_only =
            index.search_indices_impl("apple", None, None, false, CaseMatching::Smart);
        assert_eq!(full.len(), indices_only.len());
        for (f, i) in full.iter().zip(indices_only.iter()) {
            assert_eq!(f.index, i.index);
            assert!((f.score - i.score).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_search_indices_with_positions() {
        let index = FuzzyIndex::new(vec!["hello".into()]);
        let results = index.search_indices_impl("hello", None, None, true, CaseMatching::Smart);
        assert!(!results.is_empty());
        assert_eq!(results[0].positions, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_search_indices_min_score() {
        let index = FuzzyIndex::new(vec!["apple".into(), "xyz".into()]);
        let results =
            index.search_indices_impl("apple", None, Some(0.5), false, CaseMatching::Smart);
        for r in &results {
            assert!(r.score >= 0.5);
        }
    }

    #[test]
    fn test_search_indices_max_results() {
        let index = FuzzyIndex::new(vec![
            "apple".into(),
            "application".into(),
            "appetizer".into(),
        ]);
        let results = index.search_indices_impl("app", Some(2), None, false, CaseMatching::Smart);
        assert!(results.len() <= 2);
    }
}
