use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use rapid_fuzzy_core::search::KeyedFuzzyIndexCore;
use rapid_fuzzy_core::search::serialization::{
    KEYED_INDEX_MAGIC, deserialize_keyed, serialize_keyed,
};

use super::keys::KeySearchResult;
use super::{SearchOptions, resolve_case_matching};

/// A persistent multi-key fuzzy search index backed by Rust-side data.
///
/// Holds key text arrays and weights in memory on the Rust side,
/// avoiding repeated FFI overhead for applications that search the
/// same dataset multiple times with multiple keys.
/// Pre-computes Utf32String representations and reuses the Matcher
/// instance for optimal repeated-search performance.
///
/// Typically wrapped by a JS-side `FuzzyObjectIndex` class that maps
/// results back to original objects.
#[napi]
pub struct KeyedFuzzyIndex {
    core: KeyedFuzzyIndexCore,
}

#[napi]
impl KeyedFuzzyIndex {
    /// Create a new KeyedFuzzyIndex.
    ///
    /// `key_texts[k]` is an array of strings for key `k`, one per item.
    /// All inner arrays must have the same length (the number of items).
    #[napi(constructor)]
    pub fn new(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> napi::Result<Self> {
        KeyedFuzzyIndexCore::new(key_texts, weights)
            .map(|core| Self { core })
            .map_err(napi::Error::from_reason)
    }

    fn new_impl(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> Result<Self, String> {
        KeyedFuzzyIndexCore::new(key_texts, weights).map(|core| Self { core })
    }

    /// Return the number of items in the index.
    #[napi(getter)]
    pub fn size(&self) -> u32 {
        self.core.size()
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined weighted score (best match first).
    #[napi]
    pub fn search(&self, query: String, options: Option<SearchOptions>) -> Vec<KeySearchResult> {
        let (max_results, min_score, case_matching, return_all_on_empty) = match &options {
            Some(opts) => (
                opts.max_results,
                opts.min_score,
                resolve_case_matching(opts.is_case_sensitive),
                opts.return_all_on_empty.unwrap_or(false),
            ),
            None => (
                None,
                None,
                nucleo_matcher::pattern::CaseMatching::Smart,
                false,
            ),
        };

        self.core
            .search(
                &query,
                max_results,
                min_score,
                case_matching,
                return_all_on_empty,
            )
            .into_iter()
            .map(KeySearchResult::from)
            .collect()
    }

    /// Find the index of the closest matching item.
    ///
    /// Returns the index of the best match, or null if no match is found.
    /// If `min_score` is provided, returns null when the best match scores below the threshold.
    ///
    /// Use the returned index to look up the item in your own data array.
    #[napi]
    pub fn closest(&self, query: String, min_score: Option<f64>) -> Option<u32> {
        let results = self.core.search(
            &query,
            Some(1),
            min_score,
            nucleo_matcher::pattern::CaseMatching::Smart,
            false,
        );
        results.into_iter().next().map(|r| r.index)
    }

    /// Add a single item to the index.
    ///
    /// `key_values` must have the same length as the number of keys.
    /// Throws if the length does not match.
    #[napi]
    pub fn add(&mut self, key_values: Vec<String>) -> napi::Result<()> {
        self.core.add(key_values).map_err(napi::Error::from_reason)
    }

    /// Add multiple items to the index at once.
    ///
    /// Each element of `items_key_values` is an array of key values for one item.
    /// Throws if any element has the wrong number of key values.
    #[napi]
    pub fn add_many(&mut self, items_key_values: Vec<Vec<String>>) -> napi::Result<()> {
        for key_values in items_key_values {
            self.core
                .add(key_values)
                .map_err(napi::Error::from_reason)?;
        }
        Ok(())
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
    /// or transferred over the network. Use `KeyedFuzzyIndex.deserialize()` to
    /// reconstruct the index.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        self.serialize_impl().into()
    }

    /// Reconstruct a KeyedFuzzyIndex from a previously serialized Buffer.
    #[napi(factory)]
    pub fn deserialize(data: Buffer) -> napi::Result<Self> {
        Self::deserialize_impl(&data).map_err(napi::Error::from_reason)
    }
}

/// Non-napi helper methods.
impl KeyedFuzzyIndex {
    fn serialize_impl(&self) -> Vec<u8> {
        serialize_keyed(
            self.core.key_texts(),
            self.core.weights(),
            KEYED_INDEX_MAGIC,
        )
    }

    fn deserialize_impl(bytes: &[u8]) -> Result<Self, String> {
        let (key_texts, weights) = deserialize_keyed(bytes, KEYED_INDEX_MAGIC)?;
        Self::new_impl(key_texts, weights)
    }
}

#[cfg(test)]
impl KeyedFuzzyIndex {
    fn add_impl(&mut self, key_values: Vec<String>) -> Result<(), String> {
        self.core.add(key_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index() -> KeyedFuzzyIndex {
        KeyedFuzzyIndex::new_impl(
            vec![
                vec![
                    "John Smith".to_string(),
                    "Jane Doe".to_string(),
                    "Bob Johnson".to_string(),
                ],
                vec![
                    "john@example.com".to_string(),
                    "jane@example.com".to_string(),
                    "bob@example.com".to_string(),
                ],
            ],
            vec![2.0, 1.0],
        )
        .unwrap()
    }

    #[test]
    fn test_basic_search() {
        let index = make_index();
        let results = index.search("john".to_string(), None);
        assert!(!results.is_empty());
        assert_eq!(results[0].index, 0); // John Smith
    }

    #[test]
    fn test_size() {
        let index = make_index();
        assert_eq!(index.size(), 3);
    }

    #[test]
    fn test_add_and_search() {
        let mut index = make_index();
        index
            .add_impl(vec![
                "John Wick".to_string(),
                "wick@example.com".to_string(),
            ])
            .unwrap();
        assert_eq!(index.size(), 4);

        let results = index.search("wick".to_string(), None);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_add_wrong_key_count() {
        let mut index = make_index(); // 2 keys

        // Too few keys
        let result = index.add_impl(vec!["only_one".to_string()]);
        assert!(result.is_err());
        assert_eq!(index.size(), 3); // Unchanged

        // Too many keys
        let result = index.add_impl(vec!["a".into(), "b".into(), "c".into()]);
        assert!(result.is_err());
        assert_eq!(index.size(), 3); // Unchanged
    }

    #[test]
    fn test_add_many_validates_key_count() {
        let mut index = make_index();

        // Correct keys
        index
            .add_impl(vec!["Alice".into(), "alice@example.com".into()])
            .unwrap();
        assert_eq!(index.size(), 4);

        // Wrong count is rejected
        let result = index.add_impl(vec!["Bad".into()]);
        assert!(result.is_err());
        assert_eq!(index.size(), 4); // Unchanged
    }

    #[test]
    fn test_remove() {
        let mut index = make_index();
        assert!(index.remove(1)); // Remove Jane Doe
        assert_eq!(index.size(), 2);
        assert!(!index.remove(10)); // Out of bounds
    }

    #[test]
    fn test_destroy() {
        let mut index = make_index();
        index.destroy();
        assert_eq!(index.size(), 0);
    }

    #[test]
    fn test_empty_query() {
        let index = make_index();
        let results = index.search("".to_string(), None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_min_score() {
        let index = make_index();
        let results = index.search(
            "john".to_string(),
            Some(SearchOptions {
                max_results: None,
                min_score: Some(0.9),
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
            }),
        );
        for r in &results {
            assert!(r.score >= 0.9);
        }
    }

    #[test]
    fn test_max_results() {
        let index = make_index();
        let results = index.search(
            "o".to_string(),
            Some(SearchOptions {
                max_results: Some(1),
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
            }),
        );
        assert!(results.len() <= 1);
    }

    #[test]
    fn test_key_scores_populated() {
        let index = make_index();
        let results = index.search("john".to_string(), None);
        assert!(!results.is_empty());
        assert_eq!(results[0].key_scores.len(), 2);
    }

    #[test]
    fn test_mismatched_weights_rejected() {
        // Too few weights
        let result = KeyedFuzzyIndex::new_impl(vec![vec!["a".into()], vec!["b".into()]], vec![1.0]);
        assert!(result.is_err());

        // Too many weights
        let result = KeyedFuzzyIndex::new_impl(vec![vec!["a".into()]], vec![1.0, 2.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_weight_rejected() {
        let result = KeyedFuzzyIndex::new_impl(vec![vec!["a".into()]], vec![-1.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_nan_weight_rejected() {
        let result = KeyedFuzzyIndex::new_impl(vec![vec!["a".into()]], vec![f64::NAN]);
        assert!(result.is_err());
    }

    #[test]
    fn test_infinity_weight_rejected() {
        let result = KeyedFuzzyIndex::new_impl(vec![vec!["a".into()]], vec![f64::INFINITY]);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_total_weight_rejected() {
        let result = KeyedFuzzyIndex::new_impl(vec![vec!["a".into()]], vec![0.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_ordering_with_equal_scores() {
        // Create items where multiple entries will have the same combined score
        let index = KeyedFuzzyIndex::new_impl(
            vec![
                vec![
                    "alpha".to_string(),
                    "alpha".to_string(),
                    "alpha".to_string(),
                ],
                vec![
                    "x@test.com".to_string(),
                    "y@test.com".to_string(),
                    "z@test.com".to_string(),
                ],
            ],
            vec![1.0, 1.0],
        )
        .unwrap();

        // Run the same search twice — results should be identical
        let results1 = index.search("alpha".to_string(), None);
        let results2 = index.search("alpha".to_string(), None);

        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.index, r2.index, "ordering should be deterministic");
        }

        // With equal scores, items should be sorted by original index ascending
        for window in results1.windows(2) {
            if (window[0].score - window[1].score).abs() < f64::EPSILON {
                assert!(
                    window[0].index < window[1].index,
                    "equal scores should be ordered by index: {} vs {}",
                    window[0].index,
                    window[1].index
                );
            }
        }
    }

    #[test]
    fn test_zero_weight_key_skipped() {
        // Key 1 has weight 0, so it should be skipped entirely.
        // Only key 0 ("name") should affect scoring.
        let index = KeyedFuzzyIndex::new_impl(
            vec![
                vec!["Alice".to_string(), "Bob".to_string()],
                vec!["zzzzz".to_string(), "zzzzz".to_string()],
            ],
            vec![1.0, 0.0],
        )
        .unwrap();
        let results = index.search("Alice".to_string(), None);
        assert!(!results.is_empty());
        assert_eq!(results[0].index, 0);
        // key_scores[1] should be 0.0 since weight=0 key is skipped
        assert_eq!(results[0].key_scores[1], 0.0);
    }

    #[test]
    fn test_early_exit_with_high_threshold() {
        // With min_score=0.9, items that can't reach the threshold
        // even with perfect scores on remaining keys should be pruned early.
        let index = KeyedFuzzyIndex::new_impl(
            vec![
                vec!["apple".to_string(), "xyz".to_string()],
                vec!["banana".to_string(), "xyz".to_string()],
            ],
            vec![1.0, 1.0],
        )
        .unwrap();
        let results = index.search(
            "apple".to_string(),
            Some(SearchOptions {
                max_results: None,
                min_score: Some(0.9),
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
            }),
        );
        // "xyz" should not appear since it can't reach 0.9 on any key
        for r in &results {
            assert!(r.score >= 0.9);
        }
    }

    #[test]
    fn test_char_mask_prefilter() {
        // Items whose key text doesn't contain query characters
        // should be filtered out by char_mask before expensive scoring.
        let index = KeyedFuzzyIndex::new_impl(
            vec![vec![
                "hello".to_string(),
                "world".to_string(),
                "xyz".to_string(),
            ]],
            vec![1.0],
        )
        .unwrap();
        let results = index.search("hello".to_string(), None);
        // "xyz" shares no characters with "hello", should be filtered
        assert!(results.iter().all(|r| r.index != 2 || r.score == 0.0));
    }

    #[test]
    fn test_mismatched_key_texts_lengths_rejected() {
        let result = KeyedFuzzyIndex::new_impl(
            vec![
                vec!["a".into(), "b".into()],
                vec!["c".into()], // different length
            ],
            vec![1.0, 1.0],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_closest_returns_correct_index() {
        let index = make_index();
        let result = index.closest("john".to_string(), None);
        assert_eq!(result, Some(0)); // John Smith is at index 0
    }

    #[test]
    fn test_closest_returns_none_when_min_score_too_high() {
        let index = make_index();
        let result = index.closest("john".to_string(), Some(1.1));
        assert_eq!(result, None);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let index = make_index();
        let buf = index.serialize_impl();
        let restored = KeyedFuzzyIndex::deserialize_impl(&buf).unwrap();

        assert_eq!(restored.size(), index.size());

        let original_results = index.search("john".to_string(), None);
        let restored_results = restored.search("john".to_string(), None);

        assert_eq!(original_results.len(), restored_results.len());
        for (orig, rest) in original_results.iter().zip(restored_results.iter()) {
            assert_eq!(orig.index, rest.index);
            assert!((orig.score - rest.score).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_deserialize_fails_on_bad_magic() {
        let mut buf = b"XXXX".to_vec();
        buf.extend_from_slice(&1u32.to_le_bytes()); // version
        buf.extend_from_slice(&0u32.to_le_bytes()); // num_keys
        buf.extend_from_slice(&0u32.to_le_bytes()); // num_items
        let result = KeyedFuzzyIndex::deserialize_impl(&buf);
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("bad magic bytes"));
    }
}
