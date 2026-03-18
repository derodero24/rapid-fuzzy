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

/// Classify the match type from the matched character positions.
pub(crate) fn classify_match(positions: &[u32], item_char_count: usize) -> MatchType {
    if positions.is_empty() {
        return MatchType::Fuzzy;
    }

    let is_consecutive = positions.len() == 1 || positions.windows(2).all(|w| w[1] == w[0] + 1);

    if is_consecutive {
        if positions[0] == 0 && positions.len() == item_char_count {
            MatchType::Exact
        } else if positions[0] == 0 {
            MatchType::Prefix
        } else {
            MatchType::Contains
        }
    } else {
        MatchType::Fuzzy
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

thread_local! {
    /// Reusable Matcher for standalone search/closest calls.
    /// Avoids allocating internal scoring matrices on every invocation.
    /// FuzzyIndex has its own Matcher, so this is only for the standalone path.
    static STANDALONE_MATCHER: RefCell<Matcher> = RefCell::new(Matcher::new(Config::DEFAULT));
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

    STANDALONE_MATCHER.with(|cell| {
        let mut matcher = cell.borrow_mut();
        let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
        let max_score = compute_max_score(query, &pattern, &mut matcher);
        let threshold = min_score.unwrap_or(0.0);

        let mut buf = Vec::new();

        // Pass 1: Score all items, collect (index, score) only — no String cloning.
        let mut scored: Vec<(u32, f64)> = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                buf.clear();
                let atoms = nucleo_matcher::Utf32Str::new(item, &mut buf);
                let raw_score = pattern.score(atoms, &mut matcher)?;
                let normalized = (raw_score as f64 / max_score).min(1.0);
                if normalized >= threshold {
                    Some((index as u32, normalized))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending, with shorter items first as tiebreaker,
        // then by original index for fully deterministic ordering.
        let cmp = |a: &(u32, f64), b: &(u32, f64)| {
            let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
            if score_ord != std::cmp::Ordering::Equal {
                return score_ord;
            }
            let len_ord = items[a.0 as usize].len().cmp(&items[b.0 as usize].len());
            if len_ord != std::cmp::Ordering::Equal {
                return len_ord;
            }
            a.0.cmp(&b.0)
        };

        // Top-k selection: use quickselect O(n) + sort O(k log k) instead of
        // full sort O(n log n) when maxResults is set.
        if let Some(max) = max_results {
            let k = max as usize;
            if scored.len() > k {
                scored.select_nth_unstable_by(k, cmp);
                scored.truncate(k);
            }
        }
        scored.sort_unstable_by(cmp);

        // Pass 2: Construct results only for the final top-k items.
        scored
            .into_iter()
            .map(|(index, score)| {
                let (positions, match_type) = if include_positions {
                    buf.clear();
                    let atoms = nucleo_matcher::Utf32Str::new(&items[index as usize], &mut buf);
                    let mut indices = Vec::new();
                    pattern.indices(atoms, &mut matcher, &mut indices);
                    indices.sort_unstable();
                    indices.dedup();
                    let item_char_count = items[index as usize].chars().count();
                    let mt = classify_match(&indices, item_char_count);
                    (indices, Some(mt))
                } else {
                    (Vec::new(), None)
                };

                SearchResult {
                    item: items[index as usize].clone(),
                    score,
                    index,
                    positions,
                    match_type,
                }
            })
            .collect()
    })
}

/// Pre-computed search context for FuzzyIndex.
pub(crate) struct PrecomputedSearch<'a> {
    pub items: &'a [String],
    pub utf32_items: &'a [Utf32String],
    pub char_masks: &'a [u64],
    /// Optional subset of item indices to search (for incremental narrowing).
    /// When `None`, all items are searched.
    pub candidate_indices: Option<&'a [u32]>,
    pub matcher: &'a RefCell<Matcher>,
}

/// Compute a character-presence bitmask for quick pre-filtering.
///
/// Bit layout: 0–25 = a–z (case-insensitive), 26–35 = 0–9.
/// Characters outside this range are ignored (conservative — no false negatives).
pub(crate) fn compute_char_mask(s: &str) -> u64 {
    let mut mask = 0u64;
    for c in s.chars() {
        let bit = match c {
            'a'..='z' => Some(c as u32 - 'a' as u32),
            'A'..='Z' => Some(c as u32 - 'A' as u32),
            '0'..='9' => Some(26 + c as u32 - '0' as u32),
            _ => None,
        };
        if let Some(b) = bit {
            mask |= 1u64 << b;
        }
    }
    mask
}

/// Extract a character-presence bitmask from a query string, considering
/// only positive (non-inverted) terms. Inverted terms (`!term`) and
/// syntax prefixes/suffixes (`^`, `'`, `$`) are stripped.
pub(crate) fn compute_query_mask(query: &str) -> u64 {
    let mut mask = 0u64;
    for term in query.split_whitespace() {
        if term.starts_with('!') {
            continue;
        }
        let term = term.trim_start_matches(['^', '\'']);
        let term = term.trim_end_matches('$');
        mask |= compute_char_mask(term);
    }
    mask
}

/// Result of `search_over_precomputed`: the top-k results plus all matching indices.
pub(crate) struct PrecomputedSearchResult {
    /// Top-k search results (sorted, truncated).
    pub results: Vec<SearchResult>,
    /// Indices of ALL items that matched (before truncation), for incremental cache.
    pub all_matching_indices: Vec<u32>,
}

/// Lightweight result of `search_over_precomputed_indices`: index-only results.
pub(crate) struct PrecomputedIndexSearchResult {
    /// Top-k index-only results (sorted, truncated).
    pub results: Vec<IndexSearchResult>,
    /// Indices of ALL items that matched (before truncation), for incremental cache.
    pub all_matching_indices: Vec<u32>,
}

/// Search over pre-computed Utf32String items with a reusable Matcher.
///
/// Used by FuzzyIndex to avoid per-search string conversion and Matcher allocation.
/// When `ctx.candidate_indices` is set, only those items are scored (incremental search).
pub(crate) fn search_over_precomputed(
    query: &str,
    ctx: &PrecomputedSearch<'_>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> PrecomputedSearchResult {
    let items = ctx.items;
    let utf32_items = ctx.utf32_items;
    let matcher_cell = ctx.matcher;
    if query.is_empty() || items.is_empty() {
        return PrecomputedSearchResult {
            results: Vec::new(),
            all_matching_indices: Vec::new(),
        };
    }

    let mut matcher = matcher_cell.borrow_mut();
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(query, &pattern, &mut matcher);
    let threshold = min_score.unwrap_or(0.0);
    let query_mask = compute_query_mask(query);
    let char_masks = ctx.char_masks;

    // Scoring closure shared by both full-scan and candidate paths.
    let mut score_item = |index: u32| -> Option<(u32, f64)> {
        let idx = index as usize;
        if query_mask != 0 && (char_masks[idx] & query_mask) != query_mask {
            return None;
        }
        let atoms = utf32_items[idx].slice(..);
        let raw_score = pattern.score(atoms, &mut matcher)?;
        let normalized = (raw_score as f64 / max_score).min(1.0);
        if normalized >= threshold {
            Some((index, normalized))
        } else {
            None
        }
    };

    // Pass 1: Score items. Use candidate_indices if provided (incremental search).
    let mut scored: Vec<(u32, f64)> = match ctx.candidate_indices {
        Some(candidates) => candidates.iter().filter_map(|&i| score_item(i)).collect(),
        None => (0..utf32_items.len() as u32)
            .filter_map(&mut score_item)
            .collect(),
    };

    // Collect matching indices for incremental cache, but only when the match set
    // is meaningfully smaller than the full dataset. When most items match (e.g.,
    // short queries), the cache overhead outweighs the narrowing benefit.
    let total = utf32_items.len();
    let all_matching_indices: Vec<u32> = if total > 0 && scored.len() < total / 2 {
        scored.iter().map(|&(index, _)| index).collect()
    } else {
        Vec::new()
    };

    // Sort by score descending, with shorter items first as tiebreaker,
    // then by original index for fully deterministic ordering.
    let cmp = |a: &(u32, f64), b: &(u32, f64)| {
        let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
        if score_ord != std::cmp::Ordering::Equal {
            return score_ord;
        }
        let len_ord = items[a.0 as usize].len().cmp(&items[b.0 as usize].len());
        if len_ord != std::cmp::Ordering::Equal {
            return len_ord;
        }
        a.0.cmp(&b.0)
    };

    // Top-k selection: use quickselect O(n) + sort O(k log k) instead of
    // full sort O(n log n) when maxResults is set.
    if let Some(max) = max_results {
        let k = max as usize;
        if scored.len() > k {
            scored.select_nth_unstable_by(k, cmp);
            scored.truncate(k);
        }
    }
    scored.sort_unstable_by(cmp);

    // Pass 2: Construct results only for the final top-k items.
    let results = scored
        .into_iter()
        .map(|(index, score)| {
            let (positions, match_type) = if include_positions {
                let atoms = utf32_items[index as usize].slice(..);
                let mut indices = Vec::new();
                pattern.indices(atoms, &mut matcher, &mut indices);
                indices.sort_unstable();
                indices.dedup();
                let item_char_count = items[index as usize].chars().count();
                let mt = classify_match(&indices, item_char_count);
                (indices, Some(mt))
            } else {
                (Vec::new(), None)
            };

            SearchResult {
                item: items[index as usize].clone(),
                score,
                index,
                positions,
                match_type,
            }
        })
        .collect();

    PrecomputedSearchResult {
        results,
        all_matching_indices,
    }
}

/// Index-only variant of `search_over_precomputed`.
///
/// Reuses the same Pass 1 (scoring, filtering, top-k selection) but builds
/// lightweight `IndexSearchResult` objects in Pass 2 — no String cloning.
pub(crate) fn search_over_precomputed_indices(
    query: &str,
    ctx: &PrecomputedSearch<'_>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> PrecomputedIndexSearchResult {
    let items = ctx.items;
    let utf32_items = ctx.utf32_items;
    let matcher_cell = ctx.matcher;
    if query.is_empty() || items.is_empty() {
        return PrecomputedIndexSearchResult {
            results: Vec::new(),
            all_matching_indices: Vec::new(),
        };
    }

    let mut matcher = matcher_cell.borrow_mut();
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(query, &pattern, &mut matcher);
    let threshold = min_score.unwrap_or(0.0);
    let query_mask = compute_query_mask(query);
    let char_masks = ctx.char_masks;

    let mut score_item = |index: u32| -> Option<(u32, f64)> {
        let idx = index as usize;
        if query_mask != 0 && (char_masks[idx] & query_mask) != query_mask {
            return None;
        }
        let atoms = utf32_items[idx].slice(..);
        let raw_score = pattern.score(atoms, &mut matcher)?;
        let normalized = (raw_score as f64 / max_score).min(1.0);
        if normalized >= threshold {
            Some((index, normalized))
        } else {
            None
        }
    };

    let mut scored: Vec<(u32, f64)> = match ctx.candidate_indices {
        Some(candidates) => candidates.iter().filter_map(|&i| score_item(i)).collect(),
        None => (0..utf32_items.len() as u32)
            .filter_map(&mut score_item)
            .collect(),
    };

    let total = utf32_items.len();
    let all_matching_indices: Vec<u32> = if total > 0 && scored.len() < total / 2 {
        scored.iter().map(|&(index, _)| index).collect()
    } else {
        Vec::new()
    };

    let cmp = |a: &(u32, f64), b: &(u32, f64)| {
        let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
        if score_ord != std::cmp::Ordering::Equal {
            return score_ord;
        }
        let len_ord = items[a.0 as usize].len().cmp(&items[b.0 as usize].len());
        if len_ord != std::cmp::Ordering::Equal {
            return len_ord;
        }
        a.0.cmp(&b.0)
    };

    if let Some(max) = max_results {
        let k = max as usize;
        if scored.len() > k {
            scored.select_nth_unstable_by(k, cmp);
            scored.truncate(k);
        }
    }
    scored.sort_unstable_by(cmp);

    // Pass 2: Build IndexSearchResult without String cloning.
    let results = scored
        .into_iter()
        .map(|(index, score)| {
            let (positions, match_type) = if include_positions {
                let atoms = utf32_items[index as usize].slice(..);
                let mut indices = Vec::new();
                pattern.indices(atoms, &mut matcher, &mut indices);
                indices.sort_unstable();
                indices.dedup();
                let item_char_count = items[index as usize].chars().count();
                let mt = classify_match(&indices, item_char_count);
                (indices, Some(mt))
            } else {
                (Vec::new(), None)
            };

            IndexSearchResult {
                index,
                score,
                positions,
                match_type,
            }
        })
        .collect();

    PrecomputedIndexSearchResult {
        results,
        all_matching_indices,
    }
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
