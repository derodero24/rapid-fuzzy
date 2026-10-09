use napi::Env;
use napi::bindgen_prelude::{Buffer, ObjectFinalize};
use napi_derive::napi;
use rapid_fuzzy_core::search::serialization::{deserialize_keyed_index, serialize_keyed_index};
use rapid_fuzzy_core::search::{
    KeyMatchMode, KeyScoreMode, KeyedFuzzyIndexCore, SearchKeysOptions,
};

use super::keys::KeySearchResult;
use super::{KeySearchOptionsArg, MatchModeArg, ScoreModeArg};

/// A persistent multi-key fuzzy search index backed by Rust-side data.
///
/// Holds key text arrays and weights in memory on the Rust side,
/// avoiding repeated FFI overhead for applications that search the
/// same dataset multiple times with multiple keys.
/// Pre-computes the search representation of every key text, eliminating
/// per-search string conversion overhead.
/// Memory is freed when the JavaScript garbage collector collects the instance
/// or when `destroy()` is called explicitly.
///
/// Typically wrapped by a JS-side `FuzzyObjectIndex` class that maps
/// results back to original objects.
#[napi(custom_finalize)]
pub struct KeyedFuzzyIndex {
    core: KeyedFuzzyIndexCore,
    /// Native heap bytes currently reported to the JavaScript engine (see
    /// [`KeyedFuzzyIndex::report_memory`]).
    reported_bytes: i64,
}

impl KeyedFuzzyIndex {
    fn from_core(core: KeyedFuzzyIndexCore) -> Self {
        Self {
            core,
            reported_bytes: 0,
        }
    }

    fn with_reported_memory(mut self, env: Env) -> napi::Result<Self> {
        self.report_memory(env)?;
        Ok(self)
    }

    /// Tell the JavaScript engine how much native memory this index holds,
    /// like `FuzzyIndex` does: without it the garbage collector does not know
    /// that collecting an unreachable index frees memory. Called whenever the
    /// index's size changes; the finalizer releases the amount again.
    fn report_memory(&mut self, env: Env) -> napi::Result<()> {
        let bytes = i64::try_from(self.core.heap_size()).unwrap_or(i64::MAX);
        let delta = bytes - self.reported_bytes;
        if delta != 0 {
            env.adjust_external_memory(delta)?;
            self.reported_bytes = bytes;
        }
        Ok(())
    }
}

impl ObjectFinalize for KeyedFuzzyIndex {
    fn finalize(self, env: Env) -> napi::Result<()> {
        if self.reported_bytes != 0 {
            env.adjust_external_memory(-self.reported_bytes)?;
        }
        Ok(())
    }
}

#[napi]
impl KeyedFuzzyIndex {
    /// Create a new KeyedFuzzyIndex.
    ///
    /// `keyTexts[k]` is an array of strings for key `k`, one per item.
    /// All inner arrays must have the same length (the number of items).
    #[napi(constructor)]
    pub fn new(env: Env, key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> napi::Result<Self> {
        Self::new_impl(key_texts, weights)
            .map_err(napi::Error::from_reason)?
            .with_reported_memory(env)
    }

    fn new_impl(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> Result<Self, String> {
        KeyedFuzzyIndexCore::new(key_texts, weights).map(Self::from_core)
    }

    /// Return the number of items in the index.
    #[napi(getter)]
    pub fn size(&self) -> u32 {
        self.core.size()
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined score (best match first), exactly
    /// like `searchKeys()` on the same key texts and weights.
    ///
    /// The second argument accepts either a number (maxResults shorthand) or a
    /// KeySearchOptions object, whose `matchMode` selects how the query is
    /// matched against the keys and `scoreMode` how the per-key scores are
    /// combined. `maxResults` must be a non-negative integer or `Infinity`.
    #[napi]
    pub fn search(
        &self,
        query: String,
        #[napi(ts_arg_type = "number | KeySearchOptions | undefined | null")] options: Option<
            KeySearchOptionsArg,
        >,
    ) -> Vec<KeySearchResult> {
        self.core
            .search_with_options(&query, KeySearchOptionsArg::resolve(options))
            .into_iter()
            .map(KeySearchResult::from)
            .collect()
    }

    /// Find the index of the closest matching item.
    ///
    /// Returns the index of the best match, or null if no match is found.
    /// If `minScore` is provided, returns null when the best match scores below the threshold.
    /// `scoreMode` and `matchMode` work like the `search()` options of the
    /// same names (defaults `'weighted'` and `'perKey'`): the result is the
    /// first result of
    /// `search(query, { maxResults: 1, minScore, scoreMode, matchMode })`.
    ///
    /// Use the returned index to look up the item in your own data array.
    #[napi]
    pub fn closest(
        &self,
        query: String,
        min_score: Option<f64>,
        #[napi(ts_arg_type = "KeyScoreMode | undefined | null")] score_mode: Option<ScoreModeArg>,
        #[napi(ts_arg_type = "KeyMatchMode | undefined | null")] match_mode: Option<MatchModeArg>,
    ) -> Option<u32> {
        let score_mode = score_mode.map_or(KeyScoreMode::Weighted, |arg| arg.0);
        let match_mode = match_mode.map_or(KeyMatchMode::PerKey, |arg| arg.0);
        self.closest_impl(&query, min_score, score_mode, match_mode)
    }

    /// Add a single item to the index.
    ///
    /// `keyValues` must have the same length as the number of keys.
    /// Throws if the length does not match.
    #[napi]
    pub fn add(&mut self, env: Env, key_values: Vec<String>) -> napi::Result<()> {
        self.core
            .add(key_values)
            .map_err(napi::Error::from_reason)?;
        self.report_memory(env)
    }

    /// Add multiple items to the index at once.
    ///
    /// Each element of `itemsKeyValues` is an array of key values for one item.
    /// Throws if any element has the wrong number of key values; every element
    /// is checked first, so on error no item is added.
    #[napi]
    pub fn add_many(&mut self, env: Env, items_key_values: Vec<Vec<String>>) -> napi::Result<()> {
        self.core
            .add_many(items_key_values)
            .map_err(napi::Error::from_reason)?;
        self.report_memory(env)
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    #[napi]
    pub fn remove(&mut self, env: Env, index: u32) -> napi::Result<bool> {
        let removed = self.core.remove(index);
        self.report_memory(env)?;
        Ok(removed)
    }

    /// Free the internal data. After calling this, the index is empty.
    ///
    /// The key configuration is kept, so the index stays usable: it behaves
    /// as an empty index and `add()` / `addMany()` work as before.
    #[napi]
    pub fn destroy(&mut self, env: Env) -> napi::Result<()> {
        self.core.destroy();
        self.report_memory(env)
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
    pub fn deserialize(env: Env, data: Buffer) -> napi::Result<Self> {
        Self::deserialize_impl(&data)
            .map_err(napi::Error::from_reason)?
            .with_reported_memory(env)
    }
}

/// Non-napi helper methods.
impl KeyedFuzzyIndex {
    fn closest_impl(
        &self,
        query: &str,
        min_score: Option<f64>,
        score_mode: KeyScoreMode,
        match_mode: KeyMatchMode,
    ) -> Option<u32> {
        let options = SearchKeysOptions {
            max_results: Some(1),
            min_score,
            score_mode: Some(score_mode),
            match_mode: Some(match_mode),
            ..SearchKeysOptions::default()
        };
        let results = self.core.search_with_options(query, options);
        results.into_iter().next().map(|r| r.index)
    }

    fn serialize_impl(&self) -> Vec<u8> {
        serialize_keyed_index(&self.core)
    }

    fn deserialize_impl(bytes: &[u8]) -> Result<Self, String> {
        deserialize_keyed_index(bytes).map(Self::from_core)
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
    use crate::search::KeySearchOptions;

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
        assert!(index.core.remove(1)); // Remove Jane Doe
        assert_eq!(index.size(), 2);
        assert!(!index.core.remove(10)); // Out of bounds
    }

    #[test]
    fn test_destroy() {
        let mut index = make_index();
        index.core.destroy();
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
            Some(KeySearchOptionsArg::Options(KeySearchOptions {
                max_results: None,
                min_score: Some(0.9),
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
                score_mode: None,
                match_mode: None,
            })),
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
            Some(KeySearchOptionsArg::Options(KeySearchOptions {
                max_results: Some(1),
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
                score_mode: None,
                match_mode: None,
            })),
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
            Some(KeySearchOptionsArg::Options(KeySearchOptions {
                max_results: None,
                min_score: Some(0.9),
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
                score_mode: None,
                match_mode: None,
            })),
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
        let result = index.closest_impl("john", None, KeyScoreMode::Weighted, KeyMatchMode::PerKey);
        assert_eq!(result, Some(0)); // John Smith is at index 0
    }

    #[test]
    fn test_closest_returns_none_when_min_score_too_high() {
        let index = make_index();
        let result = index.closest_impl(
            "john",
            Some(1.1),
            KeyScoreMode::Weighted,
            KeyMatchMode::PerKey,
        );
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

    fn with_score_mode(score_mode: KeyScoreMode, min_score: Option<f64>) -> KeySearchOptionsArg {
        KeySearchOptionsArg::Options(KeySearchOptions {
            max_results: None,
            min_score,
            include_positions: None,
            is_case_sensitive: None,
            return_all_on_empty: None,
            score_mode: Some(score_mode),
            match_mode: None,
        })
    }

    #[test]
    fn test_score_modes() {
        // "Smith" matches only the name of item 0, exactly; the email does
        // not contain it.
        let index = KeyedFuzzyIndex::new_impl(
            vec![
                vec!["John Smith".to_string(), "Jane Doe".to_string()],
                vec![
                    "john@example.com".to_string(),
                    "jane@example.com".to_string(),
                ],
            ],
            vec![2.0, 1.0],
        )
        .unwrap();
        let score = |options: Option<KeySearchOptionsArg>| {
            let results = index.search("smith".to_string(), options);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].index, 0);
            assert_eq!(results[0].key_scores, [1.0, 0.0]);
            results[0].score
        };
        assert_eq!(score(None), 2.0 / 3.0);
        assert_eq!(
            score(Some(with_score_mode(KeyScoreMode::Weighted, None))),
            2.0 / 3.0
        );
        assert_eq!(
            score(Some(with_score_mode(KeyScoreMode::Matched, None))),
            1.0
        );
        assert_eq!(score(Some(with_score_mode(KeyScoreMode::Max, None))), 1.0);

        // minScore applies to the combined score of the mode.
        for (score_mode, expected) in [
            (KeyScoreMode::Weighted, 0),
            (KeyScoreMode::Matched, 1),
            (KeyScoreMode::Max, 1),
        ] {
            let results = index.search(
                "smith".to_string(),
                Some(with_score_mode(score_mode, Some(0.9))),
            );
            assert_eq!(results.len(), expected, "{score_mode:?}");
            assert_eq!(
                index.closest_impl("smith", Some(0.9), score_mode, KeyMatchMode::PerKey),
                (expected == 1).then_some(0),
                "{score_mode:?}"
            );
        }
    }

    #[test]
    fn test_match_modes() {
        // name, city (#782): the terms of "john tokyo" are in different keys.
        let index = KeyedFuzzyIndex::new_impl(
            vec![
                vec!["John Smith".to_string(), "John Doe".to_string()],
                vec!["Tokyo".to_string(), "Osaka".to_string()],
            ],
            vec![1.0, 1.0],
        )
        .unwrap();
        let search = |query: &str, match_mode: Option<KeyMatchMode>| {
            let options = KeySearchOptionsArg::Options(KeySearchOptions {
                max_results: None,
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
                score_mode: Some(KeyScoreMode::Max),
                match_mode,
            });
            index
                .search(query.to_string(), Some(options))
                .iter()
                .map(|r| (r.index, r.score))
                .collect::<Vec<_>>()
        };
        for per_key in [None, Some(KeyMatchMode::PerKey)] {
            assert_eq!(search("john tokyo", per_key), []);
            assert_eq!(search("john !tokyo", per_key), [(1, 1.0), (0, 1.0)]);
        }
        let cross_key = Some(KeyMatchMode::CrossKey);
        assert_eq!(search("john tokyo", cross_key), [(0, 1.0)]);
        assert_eq!(search("john !tokyo", cross_key), [(1, 1.0)]);
        for (match_mode, expected) in [
            (KeyMatchMode::PerKey, None),
            (KeyMatchMode::CrossKey, Some(0)),
        ] {
            assert_eq!(
                index.closest_impl("john tokyo", None, KeyScoreMode::Weighted, match_mode),
                expected
            );
        }
    }
}
