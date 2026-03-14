use napi_derive::napi;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

/// A single fuzzy search result with the matched item and its score.
#[napi(object)]
pub struct SearchResult {
    /// The original string that matched.
    pub item: String,
    /// The match score (higher is better). 0 means no match.
    pub score: u32,
    /// The index of the item in the original input array.
    pub index: u32,
}

/// Perform fuzzy search over a list of strings.
///
/// Returns matches sorted by score (best match first).
/// Uses the nucleo algorithm (same as Helix editor), which is
/// significantly faster than fzf/skim for large datasets.
#[napi]
pub fn search(query: String, items: Vec<String>, max_results: Option<u32>) -> Vec<SearchResult> {
    if query.is_empty() || items.is_empty() {
        return Vec::new();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(&query, CaseMatching::Smart, Normalization::Smart);

    let mut results: Vec<SearchResult> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let mut buf = Vec::new();
            let atoms = nucleo_matcher::Utf32Str::new(item, &mut buf);
            pattern
                .score(atoms, &mut matcher)
                .map(|score| SearchResult {
                    item: item.clone(),
                    score,
                    index: index as u32,
                })
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));

    if let Some(max) = max_results {
        results.truncate(max as usize);
    }

    results
}

/// Find the closest matching string from a list.
///
/// Returns the best match, or null if no match is found.
#[napi]
pub fn closest(query: String, items: Vec<String>) -> Option<String> {
    let results = search(query, items, Some(1));
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
        let results = search("typscript".to_string(), items, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].item, "TypeScript");
    }

    #[test]
    fn test_search_empty_query() {
        let items = vec!["foo".to_string()];
        let results = search("".to_string(), items, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_closest() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
        ];
        let result = closest("app".to_string(), items);
        assert!(result.is_some());
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
                let results = search(query, items, Some(max));
                prop_assert!(results.len() <= max as usize);
            }

            #[test]
            fn search_scores_sorted_descending(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search(query, items, None);
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
            fn closest_in_search_results(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let closest_result = closest(query.clone(), items.clone());
                let search_results = search(query, items, None);

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
                let results = search(query, items, None);
                for r in &results {
                    prop_assert!((r.index as usize) < len, "index {} out of bounds (len={})", r.index, len);
                }
            }
        }
    }
}
