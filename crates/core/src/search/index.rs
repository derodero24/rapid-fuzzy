use std::cell::RefCell;

use napi::Either;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher, Utf32String};

use super::{
    IndexSearchResult, PrecomputedSearch, SearchOptions, SearchResult, compute_char_mask,
    resolve_case_matching, search_over_precomputed, search_over_precomputed_indices,
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
    /// Incremental search cache: the query from the last search.
    last_query: RefCell<String>,
    /// Incremental search cache: indices of all items that matched the last query.
    last_matching_indices: RefCell<Vec<u32>>,
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
            last_query: RefCell::new(String::new()),
            last_matching_indices: RefCell::new(Vec::new()),
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
            let limit = max_results.unwrap_or(self.items.len() as u32) as usize;
            return self
                .items
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
            let limit = max_results.unwrap_or(self.items.len() as u32) as usize;
            return (0..self.items.len())
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
        self.utf32_items.push(Utf32String::from(item.as_str()));
        self.char_masks.push(compute_char_mask(&item));
        self.items.push(item);
        self.invalidate_cache();
    }

    /// Add multiple items to the index at once.
    #[napi]
    pub fn add_many(&mut self, items: Vec<String>) {
        for item in &items {
            self.utf32_items.push(Utf32String::from(item.as_str()));
            self.char_masks.push(compute_char_mask(item));
        }
        self.items.extend(items);
        self.invalidate_cache();
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
            self.invalidate_cache();
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
        self.invalidate_cache();
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
        // Format: [magic 4B] [version u32 LE] [count u32 LE] [items...]
        // Each item: [len u32 LE] [utf-8 bytes]
        let mut buf = Vec::new();
        buf.extend_from_slice(SERIALIZE_MAGIC);
        buf.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
        buf.extend_from_slice(&(self.items.len() as u32).to_le_bytes());
        for item in &self.items {
            buf.extend_from_slice(&(item.len() as u32).to_le_bytes());
            buf.extend_from_slice(item.as_bytes());
        }
        buf
    }

    fn deserialize_impl(bytes: &[u8]) -> Result<Self, String> {
        let header_size = SERIALIZE_MAGIC.len() + 4 + 4; // magic + version + count

        if bytes.len() < header_size {
            return Err("Invalid data: too short".into());
        }

        if &bytes[0..4] != SERIALIZE_MAGIC {
            return Err("Invalid data: bad magic bytes".into());
        }

        let version = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .map_err(|_| "Invalid data: truncated header".to_string())?,
        );
        if version != SERIALIZE_VERSION {
            return Err(format!(
                "Unsupported format version: expected {SERIALIZE_VERSION}, got {version}"
            ));
        }

        let count = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| "Invalid data: truncated header".to_string())?,
        ) as usize;
        let mut offset = header_size;

        // Reject obviously invalid counts before allocating.
        // Each item needs at least 4 bytes (length field), so count cannot
        // exceed the remaining payload divided by 4.
        let max_possible = bytes.len().saturating_sub(header_size) / 4;
        if count > max_possible {
            return Err("Invalid data: item count exceeds payload size".into());
        }

        let mut items = Vec::with_capacity(count);

        for _ in 0..count {
            if offset + 4 > bytes.len() {
                return Err("Invalid data: truncated".into());
            }
            let len = u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .map_err(|_| "Invalid data: truncated".to_string())?,
            ) as usize;
            offset += 4;
            if offset + len > bytes.len() {
                return Err("Invalid data: truncated".into());
            }
            let s = std::str::from_utf8(&bytes[offset..offset + len])
                .map_err(|e| format!("Invalid UTF-8: {e}"))?;
            items.push(s.to_owned());
            offset += len;
        }

        if offset != bytes.len() {
            return Err("Invalid data: trailing bytes".into());
        }

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
        // Determine if we can narrow the search to cached matching indices.
        // Conditions: new query is a prefix extension of the cached query,
        // the cache has a non-empty candidate set, and neither query uses
        // inverted terms (which break monotonicity).
        let candidates: Option<Vec<u32>> = {
            let last_q = self.last_query.borrow();
            let last_idx = self.last_matching_indices.borrow();
            if !last_q.is_empty()
                && !last_idx.is_empty()
                && query.len() > last_q.len()
                && query.starts_with(last_q.as_str())
                && !query.contains('!')
                && !last_q.contains('!')
            {
                Some(last_idx.clone())
            } else {
                None
            }
        };

        let ctx = PrecomputedSearch {
            items: &self.items,
            utf32_items: &self.utf32_items,
            char_masks: &self.char_masks,
            candidate_indices: candidates.as_deref(),
            matcher: &self.matcher,
        };

        let outcome = search_over_precomputed(
            query,
            &ctx,
            max_results,
            min_score,
            include_positions,
            case_matching,
        );

        // Update incremental cache.
        *self.last_query.borrow_mut() = query.to_owned();
        *self.last_matching_indices.borrow_mut() = outcome.all_matching_indices;

        outcome.results
    }

    fn search_indices_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<IndexSearchResult> {
        let candidates: Option<Vec<u32>> = {
            let last_q = self.last_query.borrow();
            let last_idx = self.last_matching_indices.borrow();
            if !last_q.is_empty()
                && !last_idx.is_empty()
                && query.len() > last_q.len()
                && query.starts_with(last_q.as_str())
                && !query.contains('!')
                && !last_q.contains('!')
            {
                Some(last_idx.clone())
            } else {
                None
            }
        };

        let ctx = PrecomputedSearch {
            items: &self.items,
            utf32_items: &self.utf32_items,
            char_masks: &self.char_masks,
            candidate_indices: candidates.as_deref(),
            matcher: &self.matcher,
        };

        let outcome = search_over_precomputed_indices(
            query,
            &ctx,
            max_results,
            min_score,
            include_positions,
            case_matching,
        );

        // Update incremental cache.
        *self.last_query.borrow_mut() = query.to_owned();
        *self.last_matching_indices.borrow_mut() = outcome.all_matching_indices;

        outcome.results
    }

    fn invalidate_cache(&self) {
        self.last_query.borrow_mut().clear();
        self.last_matching_indices.borrow_mut().clear();
    }
}

/// Magic bytes identifying a serialized FuzzyIndex.
const SERIALIZE_MAGIC: &[u8; 4] = b"RFZI";
/// Current serialization format version.
const SERIALIZE_VERSION: u32 = 1;

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
