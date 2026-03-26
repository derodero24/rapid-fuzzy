mod index;
mod keyed_index;
mod keys;

pub use index::FuzzyIndex;
pub use keyed_index::KeyedFuzzyIndex;
pub use keys::search_keys;

// Re-export core algorithm functions and types for use in submodules and tests.
pub(crate) use rapid_fuzzy_core::search::{
    BigramIndex, PrecomputedSearch, compute_char_mask, compute_max_score, compute_query_mask,
    extract_query_bigrams, intersect_sorted, resolve_case_matching, search_over_precomputed,
    search_over_precomputed_indices,
};

#[cfg(test)]
pub(crate) use rapid_fuzzy_core::search::{bigram_key, extract_bigrams};

use napi::Either;
use napi_derive::napi;
use nucleo_matcher::pattern::CaseMatching;

// -------------------------
// Napi-specific types
// -------------------------

/// Classification of how a query matched an item.
///
/// Derived from the matched character positions:
/// - **Exact**: all positions consecutive from index 0, covering every character in the item.
/// - **Prefix**: all positions consecutive from index 0, but the item is longer.
/// - **Contains**: all positions consecutive (a substring match), not starting at 0.
/// - **Fuzzy**: positions have gaps (character-level fuzzy match).
#[napi(string_enum)]
#[derive(Debug, PartialEq)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

impl From<rapid_fuzzy_core::search::MatchType> for MatchType {
    fn from(mt: rapid_fuzzy_core::search::MatchType) -> Self {
        match mt {
            rapid_fuzzy_core::search::MatchType::Exact => Self::Exact,
            rapid_fuzzy_core::search::MatchType::Prefix => Self::Prefix,
            rapid_fuzzy_core::search::MatchType::Contains => Self::Contains,
            rapid_fuzzy_core::search::MatchType::Fuzzy => Self::Fuzzy,
        }
    }
}

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
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `includePositions` is set to true in SearchOptions.
    /// Derived from positions at zero additional cost.
    pub match_type: Option<MatchType>,
}

impl From<rapid_fuzzy_core::search::SearchResult> for SearchResult {
    fn from(r: rapid_fuzzy_core::search::SearchResult) -> Self {
        Self {
            item: r.item,
            score: r.score,
            index: r.index,
            positions: r.positions,
            match_type: r.match_type.map(Into::into),
        }
    }
}

/// A lightweight search result containing only index and score (no item string).
///
/// Use this when you maintain your own data array and only need the index
/// to look up the original item. Avoids String cloning overhead.
#[napi(object)]
pub struct IndexSearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// Indices of matched characters in the item string.
    /// Empty unless `includePositions` is set to true in SearchOptions.
    pub positions: Vec<u32>,
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `includePositions` is set to true in SearchOptions.
    /// Derived from positions at zero additional cost.
    pub match_type: Option<MatchType>,
}

impl From<rapid_fuzzy_core::search::IndexSearchResult> for IndexSearchResult {
    fn from(r: rapid_fuzzy_core::search::IndexSearchResult) -> Self {
        Self {
            index: r.index,
            score: r.score,
            positions: r.positions,
            match_type: r.match_type.map(Into::into),
        }
    }
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
    /// If true, return all items when the query is empty (or whitespace-only).
    /// Useful for filter-as-you-type UIs where the full list should appear
    /// before the user starts typing. Default is false.
    pub return_all_on_empty: Option<bool>,
}

/// Internal search implementation wrapping the core-lib algorithm.
/// Returns napi-compatible `SearchResult` types.
pub(crate) fn search_impl(
    query: String,
    items: Vec<String>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    rapid_fuzzy_core::search::search_impl(
        query,
        items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    )
    .into_iter()
    .map(SearchResult::from)
    .collect()
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
    fn test_match_type_exact() {
        let items = vec!["apple".to_string(), "pineapple".to_string()];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        let exact = results.iter().find(|r| r.item == "apple").unwrap();
        assert_eq!(exact.match_type, Some(MatchType::Exact));
    }

    #[test]
    fn test_match_type_prefix() {
        let items = vec!["application".to_string()];
        let results = search_impl(
            "app".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Prefix));
    }

    #[test]
    fn test_match_type_contains() {
        let items = vec!["pineapple".to_string()];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Contains));
    }

    #[test]
    fn test_match_type_fuzzy() {
        let items = vec!["abcdef".to_string()];
        let results = search_impl(
            "adf".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Fuzzy));
    }

    #[test]
    fn test_match_type_none_without_positions() {
        let items = vec!["hello".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, None);
    }

    #[test]
    fn test_match_type_present_with_positions() {
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
        assert_eq!(results[0].match_type, Some(MatchType::Exact));
    }

    #[test]
    fn test_match_type_case_insensitive_exact() {
        let items = vec!["Apple".to_string()];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Exact));
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

    mod bigram_tests {
        use super::*;

        #[test]
        fn test_bigram_key_ascii_case_folding() {
            assert_eq!(bigram_key('a', 'b'), bigram_key('A', 'B'));
            assert_eq!(bigram_key('a', 'b'), bigram_key('A', 'b'));
            assert_eq!(bigram_key('Z', 'a'), bigram_key('z', 'A'));
        }

        #[test]
        fn test_bigram_key_digits() {
            assert_ne!(bigram_key('a', '0'), bigram_key('a', 'b'));
            assert_eq!(bigram_key('0', '1'), bigram_key('0', '1'));
        }

        #[test]
        fn test_extract_bigrams_basic() {
            let bg = extract_bigrams("abc");
            assert_eq!(bg.len(), 2); // "ab", "bc"
        }

        #[test]
        fn test_extract_bigrams_case_insensitive() {
            let lower = extract_bigrams("abc");
            let upper = extract_bigrams("ABC");
            assert_eq!(lower, upper);
        }

        #[test]
        fn test_extract_bigrams_single_char() {
            assert!(extract_bigrams("a").is_empty());
        }

        #[test]
        fn test_extract_bigrams_empty() {
            assert!(extract_bigrams("").is_empty());
        }

        #[test]
        fn test_extract_bigrams_unicode() {
            let bg = extract_bigrams("東京都");
            assert_eq!(bg.len(), 2); // "東京", "京都"
        }

        #[test]
        fn test_extract_query_bigrams_skips_inverted() {
            let bg = extract_query_bigrams("hello !world");
            let plain = extract_bigrams("hello");
            assert_eq!(bg, plain);
        }

        #[test]
        fn test_extract_query_bigrams_strips_syntax() {
            let bg = extract_query_bigrams("^hello$");
            let plain = extract_bigrams("hello");
            assert_eq!(bg, plain);
        }

        #[test]
        fn test_extract_query_bigrams_single_char_terms() {
            // Single-char terms produce no bigrams
            let bg = extract_query_bigrams("a b");
            assert!(bg.is_empty());
        }

        #[test]
        fn test_intersect_sorted_basic() {
            let result = intersect_sorted(&[1, 3, 5, 7], &[2, 3, 5, 8]);
            assert_eq!(result, vec![3, 5]);
        }

        #[test]
        fn test_intersect_sorted_no_overlap() {
            let result = intersect_sorted(&[1, 3], &[2, 4]);
            assert!(result.is_empty());
        }

        #[test]
        fn test_intersect_sorted_empty() {
            let result = intersect_sorted(&[], &[1, 2]);
            assert!(result.is_empty());
        }

        #[test]
        fn test_bigram_index_basic() {
            let items: Vec<String> = vec!["hello".into(), "world".into(), "help".into()];
            let idx = BigramIndex::new(&items);
            // "he" appears in "hello" and "help"
            let bg = extract_query_bigrams("he");
            let candidates = idx.candidates(&bg);
            assert!(candidates.is_some());
            let cands = candidates.unwrap();
            assert!(cands.contains(&0)); // hello
            assert!(cands.contains(&2)); // help
            assert!(!cands.contains(&1)); // world
        }

        #[test]
        fn test_bigram_index_no_match() {
            let items: Vec<String> = vec!["hello".into(), "world".into()];
            let idx = BigramIndex::new(&items);
            let bg = extract_query_bigrams("zz");
            let candidates = idx.candidates(&bg);
            assert_eq!(candidates, Some(vec![]));
        }

        #[test]
        fn test_bigram_index_add_item() {
            // Add enough items so matching ones are below the 80% skip threshold.
            let mut idx = BigramIndex::new(&[]);
            idx.add_item(0, "hello");
            idx.add_item(1, "help");
            idx.add_item(2, "world");
            idx.add_item(3, "rust");
            idx.add_item(4, "code");
            // "he" matches only items 0,1 = 2/5 = 40% < 80%
            let bg = extract_query_bigrams("he");
            let candidates = idx.candidates(&bg);
            assert!(candidates.is_some());
            let cands = candidates.unwrap();
            assert!(cands.contains(&0));
            assert!(cands.contains(&1));
            assert!(!cands.contains(&2));
        }

        #[test]
        fn test_bigram_index_remove_and_swap() {
            let items: Vec<String> = vec!["abc".into(), "def".into(), "abx".into()];
            let mut idx = BigramIndex::new(&items);
            // Remove item 0 ("abc"), swapping in item 2 ("abx")
            idx.remove_item(0, 2, "abc", Some("abx"));
            // "abx" should now be at index 0
            let bg = extract_query_bigrams("ab");
            let candidates = idx.candidates(&bg);
            assert!(candidates.is_some());
            let cands = candidates.unwrap();
            assert!(cands.contains(&0)); // "abx" was swapped to index 0
            assert!(!cands.contains(&2)); // old index 2 no longer valid
        }

        #[test]
        fn test_bigram_index_empty_query() {
            let items: Vec<String> = vec!["hello".into()];
            let idx = BigramIndex::new(&items);
            assert_eq!(idx.candidates(&[]), None);
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
        fn test_tiebreaker_consistent_for_unicode() {
            // Verify standalone search and indexed search produce identical
            // ordering for Unicode items where byte length != char count.
            // "hello世界" = 11 bytes / 7 chars (CJK chars are 3 bytes each)
            // "helloab" = 7 bytes / 7 chars
            // Same char count (7) but different byte lengths (11 vs 7).
            // Using .chars().count() would treat them as equal length;
            // using .len() (byte length) correctly differentiates them.
            // Both items contain the query's ASCII letters (h,e,l,o)
            // so the bitmask pre-filter in the indexed path matches both.
            use std::cell::RefCell;

            use nucleo_matcher::{Config, Matcher, Utf32String};

            let items = vec!["hello世界".to_string(), "helloab".to_string()];

            let standalone = search_impl(
                "hello".to_string(),
                items.clone(),
                None,
                None,
                false,
                CaseMatching::Smart,
            );

            // Build precomputed context to test the indexed path.
            let utf32_items: Vec<Utf32String> = items
                .iter()
                .map(|s| Utf32String::from(s.as_str()))
                .collect();
            let char_masks: Vec<u64> = items.iter().map(|s| compute_char_mask(s)).collect();
            let matcher = RefCell::new(Matcher::new(Config::DEFAULT));
            let ctx = PrecomputedSearch {
                items: &items,
                utf32_items: &utf32_items,
                char_masks: &char_masks,
                candidate_indices: None,
                matcher: &matcher,
            };
            let indexed =
                search_over_precomputed("hello", &ctx, None, None, false, CaseMatching::Smart);

            assert_eq!(
                standalone.len(),
                indexed.results.len(),
                "result count differs between standalone and indexed search"
            );
            for (s, i) in standalone.iter().zip(indexed.results.iter()) {
                assert_eq!(
                    s.item, i.item,
                    "ordering differs between standalone and indexed search"
                );
            }
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
