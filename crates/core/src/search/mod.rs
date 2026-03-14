mod index;
mod keyed_index;
mod keys;

pub use index::FuzzyIndex;
pub use keyed_index::KeyedFuzzyIndex;
pub use keys::search_keys;

use std::cell::RefCell;

use napi::Either;
use napi_derive::napi;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};

/// A single fuzzy search result with the matched item and its score.
#[napi(object)]
pub struct SearchResult {
    /// The original string that matched.
    pub item: String,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// The index of the item in the original input array.
    pub index: u32,
    /// Indices of matched characters in the item string.
    /// Empty unless `includePositions` is set to true in SearchOptions.
    pub positions: Vec<u32>,
}

/// Options for the search function.
#[napi(object)]
pub struct SearchOptions {
    /// Maximum number of results to return.
    pub max_results: Option<u32>,
    /// Minimum normalized score (0.0-1.0) to include in results.
    pub min_score: Option<f64>,
    /// If true, include matched character positions in results.
    pub include_positions: Option<bool>,
    /// If true, matching is case-sensitive. Default is smart case
    /// (case-insensitive unless the query contains uppercase characters).
    pub is_case_sensitive: Option<bool>,
}

/// Compute the maximum possible score for a given pattern by scoring
/// the query against itself (exact match = theoretical maximum).
pub(crate) fn compute_max_score(query: &str, pattern: &Pattern, matcher: &mut Matcher) -> f64 {
    let mut buf = Vec::new();
    let atoms = nucleo_matcher::Utf32Str::new(query, &mut buf);
    pattern.score(atoms, matcher).unwrap_or(1) as f64
}

/// Convert the `is_case_sensitive` flag into a `CaseMatching` variant.
pub(crate) fn resolve_case_matching(is_case_sensitive: Option<bool>) -> CaseMatching {
    match is_case_sensitive {
        Some(true) => CaseMatching::Respect,
        _ => CaseMatching::Smart,
    }
}

/// Shared search logic over a borrowed slice of items.
///
/// Both the standalone `search_impl` and `FuzzyIndex::search_impl` delegate
/// to this function, which contains the core scoring/filtering/sorting logic.
pub(crate) fn search_over_items(
    query: &str,
    items: &[String],
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    if query.is_empty() || items.is_empty() {
        return Vec::new();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(query, &pattern, &mut matcher);
    let threshold = min_score.unwrap_or(0.0);

    let mut buf = Vec::new();

    let mut results: Vec<SearchResult> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            buf.clear();
            let atoms = nucleo_matcher::Utf32Str::new(item, &mut buf);

            let (raw_score, positions) = if include_positions {
                let mut indices = Vec::new();
                let score = pattern.indices(atoms, &mut matcher, &mut indices)?;
                indices.sort_unstable();
                indices.dedup();
                (score, indices)
            } else {
                let score = pattern.score(atoms, &mut matcher)?;
                (score, Vec::new())
            };

            let normalized = (raw_score as f64 / max_score).min(1.0);
            if normalized >= threshold {
                Some(SearchResult {
                    item: item.clone(),
                    score: normalized,
                    index: index as u32,
                    positions,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_unstable_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(max) = max_results {
        results.truncate(max as usize);
    }

    results
}

/// Pre-computed search context for FuzzyIndex.
pub(crate) struct PrecomputedSearch<'a> {
    pub items: &'a [String],
    pub utf32_items: &'a [Utf32String],
    pub matcher: &'a RefCell<Matcher>,
}

/// Search over pre-computed Utf32String items with a reusable Matcher.
///
/// Used by FuzzyIndex to avoid per-search string conversion and Matcher allocation.
pub(crate) fn search_over_precomputed(
    query: &str,
    ctx: &PrecomputedSearch<'_>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    let items = ctx.items;
    let utf32_items = ctx.utf32_items;
    let matcher_cell = ctx.matcher;
    if query.is_empty() || items.is_empty() {
        return Vec::new();
    }

    let mut matcher = matcher_cell.borrow_mut();
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(query, &pattern, &mut matcher);
    let threshold = min_score.unwrap_or(0.0);

    let mut results: Vec<SearchResult> = items
        .iter()
        .zip(utf32_items.iter())
        .enumerate()
        .filter_map(|(index, (item, utf32_item))| {
            let atoms = utf32_item.slice(..);

            let (raw_score, positions) = if include_positions {
                let mut indices = Vec::new();
                let score = pattern.indices(atoms, &mut matcher, &mut indices)?;
                indices.sort_unstable();
                indices.dedup();
                (score, indices)
            } else {
                let score = pattern.score(atoms, &mut matcher)?;
                (score, Vec::new())
            };

            let normalized = (raw_score as f64 / max_score).min(1.0);
            if normalized >= threshold {
                Some(SearchResult {
                    item: item.clone(),
                    score: normalized,
                    index: index as u32,
                    positions,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_unstable_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(max) = max_results {
        results.truncate(max as usize);
    }

    results
}

/// Internal search implementation used by both the napi export and tests.
pub(crate) fn search_impl(
    query: String,
    items: Vec<String>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    search_over_items(
        &query,
        &items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    )
}

/// Perform fuzzy search over a list of strings.
///
/// Returns matches sorted by score (best match first).
/// Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
/// Uses the nucleo algorithm (same as Helix editor), which is
/// significantly faster than fzf/skim for large datasets.
///
/// The third argument accepts either a number (maxResults for backward
/// compatibility) or a SearchOptions object with maxResults and minScore.
#[napi]
pub fn search(
    query: String,
    items: Vec<String>,
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
    search_impl(
        query,
        items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    )
}

/// Find the closest matching string from a list.
///
/// Returns the best match, or null if no match is found.
/// If minScore is provided, returns null when the best match scores below the threshold.
#[napi]
pub fn closest(query: String, items: Vec<String>, min_score: Option<f64>) -> Option<String> {
    let results = search_impl(query, items, Some(1), min_score, false, CaseMatching::Smart);
    results.into_iter().next().map(|r| r.item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_basic() {
        let items = vec![
            "TypeScript".to_string(),
            "JavaScript".to_string(),
            "Python".to_string(),
            "TypeSpec".to_string(),
        ];
        let results = search_impl(
            "typscript".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].item, "TypeScript");
    }

    #[test]
    fn test_search_empty_query() {
        let items = vec!["foo".to_string()];
        let results = search_impl(
            "".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_closest() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
        ];
        let result = closest("app".to_string(), items, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_scores_normalized_range() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
            "grape".to_string(),
        ];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        for r in &results {
            assert!(
                r.score >= 0.0 && r.score <= 1.0,
                "score {} out of 0.0-1.0 range for '{}'",
                r.score,
                r.item
            );
        }
    }

    #[test]
    fn test_exact_match_scores_one() {
        let items = vec!["hello".to_string(), "world".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        let exact = results.iter().find(|r| r.item == "hello").unwrap();
        assert!(
            (exact.score - 1.0).abs() < f64::EPSILON,
            "exact match should score 1.0, got {}",
            exact.score
        );
    }

    #[test]
    fn test_partial_match_scores_below_one() {
        let items = vec!["TypeScript".to_string(), "JavaScript".to_string()];
        let results = search_impl(
            "type".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        for r in &results {
            assert!(
                r.score > 0.0 && r.score <= 1.0,
                "partial match score {} should be in (0.0, 1.0] for '{}'",
                r.score,
                r.item
            );
        }
    }

    #[test]
    fn test_min_score_filters_low_quality() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "xyz".to_string(),
        ];
        let all_results = search_impl(
            "apple".to_string(),
            items.clone(),
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        let filtered = search_impl(
            "apple".to_string(),
            items,
            None,
            Some(0.5),
            false,
            CaseMatching::Smart,
        );
        assert!(filtered.len() <= all_results.len());
        for r in &filtered {
            assert!(
                r.score >= 0.5,
                "score {} below min_score 0.5 for '{}'",
                r.score,
                r.item
            );
        }
    }

    #[test]
    fn test_min_score_one_returns_only_exact() {
        let items = vec!["hello".to_string(), "help".to_string(), "world".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "hello");
    }

    #[test]
    fn test_min_score_zero_same_as_none() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "grape".to_string(),
        ];
        let no_threshold = search_impl(
            "ap".to_string(),
            items.clone(),
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        let zero_threshold = search_impl(
            "ap".to_string(),
            items,
            None,
            Some(0.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(no_threshold.len(), zero_threshold.len());
    }

    #[test]
    fn test_min_score_with_max_results() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "appetizer".to_string(),
            "xyz".to_string(),
        ];
        let results = search_impl(
            "apple".to_string(),
            items,
            Some(2),
            Some(0.3),
            false,
            CaseMatching::Smart,
        );
        assert!(results.len() <= 2);
        for r in &results {
            assert!(r.score >= 0.3);
        }
    }

    #[test]
    fn test_closest_with_min_score() {
        let items = vec!["xyz".to_string(), "abc".to_string()];
        // With a very high threshold, closest should return None
        let result = closest("hello".to_string(), items, Some(0.99));
        assert!(result.is_none());
    }

    #[test]
    fn test_case_sensitive_excludes_different_case() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        // Case-sensitive: lowercase query "apple" should not match "Apple" or "APPLE"
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Respect,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "apple");
    }

    #[test]
    fn test_case_sensitive_uppercase_query() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        let results = search_impl(
            "APPLE".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Respect,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "APPLE");
    }

    #[test]
    fn test_smart_case_lowercase_query_matches_any_case() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        // Smart case: all-lowercase query matches any case
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_smart_case_uppercase_query_is_case_sensitive() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        // Smart case: query with uppercase becomes case-sensitive
        let results = search_impl(
            "Apple".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "Apple");
    }

    #[test]
    fn test_closest_without_min_score() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let result = closest("app".to_string(), items, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_positions_returned_when_requested() {
        let items = vec!["hello world".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert!(
            !results[0].positions.is_empty(),
            "positions should not be empty"
        );
    }

    #[test]
    fn test_positions_empty_when_not_requested() {
        let items = vec!["hello world".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert!(
            results[0].positions.is_empty(),
            "positions should be empty when not requested"
        );
    }

    #[test]
    fn test_positions_are_sorted() {
        let items = vec!["hello world".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        let positions = &results[0].positions;
        for window in positions.windows(2) {
            assert!(
                window[0] <= window[1],
                "positions not sorted: {} > {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn test_positions_within_bounds() {
        let items = vec!["hello".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        let item_len = results[0].item.chars().count() as u32;
        for &pos in &results[0].positions {
            assert!(
                pos < item_len,
                "position {} out of bounds (len={})",
                pos,
                item_len
            );
        }
    }

    #[test]
    fn test_exact_match_positions() {
        let items = vec!["hello".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].positions, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_positions_score_consistency() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
        ];
        let with_pos = search_impl(
            "apple".to_string(),
            items.clone(),
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        let without_pos = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert_eq!(with_pos.len(), without_pos.len());
        for (a, b) in with_pos.iter().zip(without_pos.iter()) {
            assert_eq!(a.item, b.item);
            assert!(
                (a.score - b.score).abs() < f64::EPSILON,
                "scores differ: {} vs {}",
                a.score,
                b.score
            );
        }
    }

    mod proptest_search {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn search_max_results_respected(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
                max in 1u32..10
            ) {
                let results = search_impl(query, items, Some(max), None, false, CaseMatching::Smart);
                prop_assert!(results.len() <= max as usize);
            }

            #[test]
            fn search_scores_sorted_descending(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for window in results.windows(2) {
                    prop_assert!(
                        window[0].score >= window[1].score,
                        "results not sorted: {} < {}",
                        window[0].score,
                        window[1].score
                    );
                }
            }

            #[test]
            fn search_scores_normalized(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!(
                        r.score >= 0.0 && r.score <= 1.0,
                        "score {} out of 0.0-1.0 range",
                        r.score
                    );
                }
            }

            #[test]
            fn closest_in_search_results(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let closest_result = closest(query.clone(), items.clone(), None);
                let search_results = search_impl(query, items, None, None, false, CaseMatching::Smart);

                match (closest_result, search_results.first()) {
                    (Some(c), Some(first)) => {
                        prop_assert_eq!(c, first.item.clone());
                    }
                    (None, None) => {} // both empty is fine
                    (Some(_), None) | (None, Some(_)) => {
                        prop_assert!(false, "closest and search disagree on match existence");
                    }
                }
            }

            #[test]
            fn search_indices_valid(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let len = items.len();
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!((r.index as usize) < len, "index {} out of bounds (len={})", r.index, len);
                }
            }

            #[test]
            fn search_positions_within_bounds(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, true, CaseMatching::Smart);
                for r in &results {
                    let item_len = r.item.chars().count() as u32;
                    for &pos in &r.positions {
                        prop_assert!(pos < item_len, "position {} >= item length {} for '{}'", pos, item_len, r.item);
                    }
                }
            }

            #[test]
            fn search_positions_sorted_and_unique(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, true, CaseMatching::Smart);
                for r in &results {
                    for window in r.positions.windows(2) {
                        prop_assert!(window[0] < window[1], "positions not strictly sorted: {} >= {}", window[0], window[1]);
                    }
                }
            }

            #[test]
            fn search_min_score_respected(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
                threshold in 0.0f64..1.0
            ) {
                let results = search_impl(query, items, None, Some(threshold), false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!(
                        r.score >= threshold,
                        "score {} below threshold {}",
                        r.score,
                        threshold
                    );
                }
            }
        }
    }

    mod unicode_tests {
        use super::*;

        #[test]
        fn test_search_cjk() {
            let items = vec!["東京".to_string(), "大阪".to_string(), "京都".to_string()];
            let results = search_impl(
                "東".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
        }

        #[test]
        fn test_search_emoji() {
            let items = vec![
                "🎉 party".to_string(),
                "🎊 celebration".to_string(),
                "work".to_string(),
            ];
            let results = search_impl(
                "party".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
            assert!(results[0].item.contains("party"));
        }

        #[test]
        fn test_search_accented() {
            let items = vec![
                "café".to_string(),
                "resume".to_string(),
                "naïve".to_string(),
            ];
            let results = search_impl(
                "cafe".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
        }

        #[test]
        fn test_closest_cjk() {
            let items = vec!["大阪".to_string(), "京都".to_string(), "東京都".to_string()];
            let result = closest("東京".to_string(), items, None);
            assert!(result.is_some());
        }

        #[test]
        fn test_search_mixed_scripts() {
            let items = vec![
                "hello世界".to_string(),
                "goodbye世間".to_string(),
                "test".to_string(),
            ];
            let results = search_impl(
                "hello".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
            assert!(results[0].item.contains("hello"));
        }
    }

    mod proptest_unicode {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn search_unicode_max_results_respected(
                query in "\\PC{1,5}",
                items in prop::collection::vec("\\PC{1,10}", 1..20),
                max in 1u32..10
            ) {
                let results = search_impl(query, items, Some(max), None, false, CaseMatching::Smart);
                prop_assert!(results.len() <= max as usize);
            }

            #[test]
            fn search_unicode_scores_sorted_descending(
                query in "\\PC{1,5}",
                items in prop::collection::vec("\\PC{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for window in results.windows(2) {
                    prop_assert!(
                        window[0].score >= window[1].score,
                        "results not sorted: {} < {}",
                        window[0].score,
                        window[1].score
                    );
                }
            }

            #[test]
            fn search_unicode_indices_valid(
                query in "\\PC{1,5}",
                items in prop::collection::vec("\\PC{1,10}", 1..20),
            ) {
                let len = items.len();
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!((r.index as usize) < len, "index {} out of bounds (len={})", r.index, len);
                }
            }
        }
    }
}
