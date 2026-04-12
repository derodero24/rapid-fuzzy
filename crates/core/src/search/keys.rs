use napi_derive::napi;
use rapid_fuzzy_core::search::SearchKeysOptions;

use super::SearchOptions;

// Re-export CaseMatching so tests can access it via `use super::*`.
#[cfg(test)]
pub(crate) use nucleo_matcher::pattern::CaseMatching;

/// A single result from multi-key fuzzy search.
#[napi(object)]
pub struct KeySearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The combined weighted score normalized to 0.0-1.0 range.
    pub score: f64,
    /// Per-key scores in the same order as the input keys.
    /// A score of 0.0 means the item did not match on that key.
    pub key_scores: Vec<f64>,
}

impl From<rapid_fuzzy_core::search::KeySearchResult> for KeySearchResult {
    fn from(r: rapid_fuzzy_core::search::KeySearchResult) -> Self {
        Self {
            index: r.index,
            score: r.score,
            key_scores: r.key_scores,
        }
    }
}

/// Perform fuzzy search across multiple text keys with weights.
///
/// `key_texts[k]` is an array of strings for key `k`, one per item.
/// All inner arrays must have the same length (the number of items).
/// `weights` specifies the relative importance of each key.
///
/// Returns results sorted by combined weighted score (best match first).
#[napi]
pub fn search_keys(
    query: String,
    key_texts: Vec<Vec<String>>,
    weights: Vec<f64>,
    options: Option<SearchOptions>,
) -> Vec<KeySearchResult> {
    let core_opts = options.map(|opts| SearchKeysOptions {
        max_results: opts.max_results,
        min_score: opts.min_score,
        is_case_sensitive: opts.is_case_sensitive,
        return_all_on_empty: opts.return_all_on_empty,
    });

    rapid_fuzzy_core::search::search_keys_impl(&query, &key_texts, &weights, core_opts)
        .into_iter()
        .map(KeySearchResult::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_multi_key_search() {
        // Items: [{name: "John Smith", email: "john@example.com"},
        //         {name: "Jane Doe", email: "jane@example.com"}]
        let key_texts = vec![
            vec!["John Smith".to_string(), "Jane Doe".to_string()],
            vec![
                "john@example.com".to_string(),
                "jane@example.com".to_string(),
            ],
        ];
        let weights = vec![1.0, 1.0];

        let results = search_keys("john".to_string(), key_texts, weights, None);
        assert!(!results.is_empty());
        // John Smith should rank first (matches on both name and email)
        assert_eq!(results[0].index, 0);
    }

    #[test]
    fn test_weighted_keys() {
        // name has higher weight
        let key_texts = vec![
            vec!["Alice".to_string(), "Bob".to_string()],
            vec!["bob@test.com".to_string(), "alice@test.com".to_string()],
        ];
        let weights = vec![2.0, 1.0]; // name is 2x more important

        let results = search_keys("alice".to_string(), key_texts, weights, None);
        assert!(!results.is_empty());
        // Alice (name match with weight 2) should rank higher than Bob (email match with weight 1)
        assert_eq!(results[0].index, 0);
    }

    #[test]
    fn test_empty_query() {
        let key_texts = vec![vec!["hello".to_string()]];
        let weights = vec![1.0];
        let results = search_keys("".to_string(), key_texts, weights, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_empty_items() {
        let key_texts: Vec<Vec<String>> = vec![vec![]];
        let weights = vec![1.0];
        let results = search_keys("test".to_string(), key_texts, weights, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_mismatched_key_lengths() {
        let key_texts = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()], // different length
        ];
        let weights = vec![1.0, 1.0];
        let results = search_keys("a".to_string(), key_texts, weights, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_mismatched_weights() {
        let key_texts = vec![vec!["a".to_string()]];
        let weights = vec![1.0, 2.0]; // more weights than keys
        let results = search_keys("a".to_string(), key_texts, weights, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_negative_weight_rejected() {
        let key_texts = vec![vec!["apple".to_string()]];
        let weights = vec![-1.0];
        let results = search_keys("apple".to_string(), key_texts, weights, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_nan_weight_rejected() {
        let key_texts = vec![vec!["apple".to_string()]];
        let weights = vec![f64::NAN];
        let results = search_keys("apple".to_string(), key_texts, weights, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_infinity_weight_rejected() {
        let key_texts = vec![vec!["apple".to_string()]];
        let weights = vec![f64::INFINITY];
        let results = search_keys("apple".to_string(), key_texts, weights, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_max_results() {
        let key_texts = vec![vec![
            "apple".to_string(),
            "application".to_string(),
            "appetizer".to_string(),
        ]];
        let weights = vec![1.0];

        let results = search_keys(
            "app".to_string(),
            key_texts,
            weights,
            Some(SearchOptions {
                max_results: Some(2),
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
            }),
        );
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_min_score() {
        let key_texts = vec![vec![
            "apple".to_string(),
            "xyz".to_string(),
            "application".to_string(),
        ]];
        let weights = vec![1.0];

        let results = search_keys(
            "apple".to_string(),
            key_texts,
            weights,
            Some(SearchOptions {
                max_results: None,
                min_score: Some(0.5),
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: None,
            }),
        );
        for r in &results {
            assert!(r.score >= 0.5);
        }
    }

    #[test]
    fn test_scores_sorted_descending() {
        let key_texts = vec![vec![
            "apple".to_string(),
            "application".to_string(),
            "appetizer".to_string(),
            "banana".to_string(),
        ]];
        let weights = vec![1.0];

        let results = search_keys("apple".to_string(), key_texts, weights, None);
        for window in results.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }

    #[test]
    fn test_key_scores_populated() {
        let key_texts = vec![
            vec!["apple".to_string(), "banana".to_string()],
            vec!["fruit".to_string(), "apple pie".to_string()],
        ];
        let weights = vec![1.0, 1.0];

        let results = search_keys("apple".to_string(), key_texts, weights, None);
        for r in &results {
            assert_eq!(r.key_scores.len(), 2);
        }
    }

    #[test]
    fn test_scores_normalized() {
        let key_texts = vec![
            vec!["hello".to_string(), "world".to_string()],
            vec!["help".to_string(), "work".to_string()],
        ];
        let weights = vec![1.0, 1.0];

        let results = search_keys("hello".to_string(), key_texts, weights, None);
        for r in &results {
            assert!(r.score >= 0.0 && r.score <= 1.0);
            for &ks in &r.key_scores {
                assert!(ks >= 0.0 && ks <= 1.0);
            }
        }
    }

    #[test]
    fn test_case_sensitive() {
        let key_texts = vec![vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ]];
        let weights = vec![1.0];

        let results = search_keys(
            "apple".to_string(),
            key_texts,
            weights,
            Some(SearchOptions {
                max_results: None,
                min_score: Some(1.0),
                include_positions: None,
                is_case_sensitive: Some(true),
                return_all_on_empty: None,
            }),
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].index, 1);
    }

    #[test]
    fn test_single_key_matches_standard_search() {
        use super::super::search_impl;

        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
        ];
        let key_texts = vec![items.clone()];
        let weights = vec![1.0];

        let key_results = search_keys("apple".to_string(), key_texts, weights, None);
        let std_results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );

        assert_eq!(key_results.len(), std_results.len());
        for (kr, sr) in key_results.iter().zip(std_results.iter()) {
            assert_eq!(kr.index, sr.index);
            assert!(
                (kr.score - sr.score).abs() < f64::EPSILON,
                "scores differ: {} vs {}",
                kr.score,
                sr.score
            );
        }
    }

    #[test]
    fn test_return_all_on_empty() {
        let key_texts = vec![vec![
            "apple".to_string(),
            "banana".to_string(),
            "grape".to_string(),
        ]];
        let weights = vec![1.0];

        let results = search_keys(
            "".to_string(),
            key_texts,
            weights,
            Some(SearchOptions {
                max_results: None,
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: Some(true),
            }),
        );
        assert_eq!(results.len(), 3);
        for (i, r) in results.iter().enumerate() {
            assert_eq!(r.index, i as u32);
            assert!((r.score - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_return_all_on_empty_respects_max_results() {
        let key_texts = vec![vec!["a".to_string(), "b".to_string(), "c".to_string()]];
        let weights = vec![1.0];

        let results = search_keys(
            "".to_string(),
            key_texts,
            weights,
            Some(SearchOptions {
                max_results: Some(2),
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: Some(true),
            }),
        );
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_return_all_on_empty_false_returns_empty() {
        let key_texts = vec![vec!["apple".to_string()]];
        let weights = vec![1.0];

        let results = search_keys(
            "".to_string(),
            key_texts,
            weights,
            Some(SearchOptions {
                max_results: None,
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: Some(false),
            }),
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_deterministic_ordering_with_equal_scores() {
        let key_texts = vec![
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
        ];
        let weights = vec![1.0, 1.0];

        let results1 = search_keys(
            "alpha".to_string(),
            key_texts.clone(),
            weights.clone(),
            None,
        );
        let results2 = search_keys("alpha".to_string(), key_texts, weights, None);

        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.index, r2.index, "ordering should be deterministic");
        }

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
    fn test_return_all_on_empty_whitespace_query() {
        let key_texts = vec![vec!["apple".to_string(), "banana".to_string()]];
        let weights = vec![1.0];

        let results = search_keys(
            "  ".to_string(),
            key_texts,
            weights,
            Some(SearchOptions {
                max_results: None,
                min_score: None,
                include_positions: None,
                is_case_sensitive: None,
                return_all_on_empty: Some(true),
            }),
        );
        assert_eq!(results.len(), 2);
    }
}
