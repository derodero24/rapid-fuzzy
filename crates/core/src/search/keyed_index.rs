use std::cell::RefCell;

use napi_derive::napi;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};

use super::keys::KeySearchResult;
use super::{SearchOptions, compute_max_score, resolve_case_matching};

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
    key_texts: Vec<Vec<String>>,
    utf32_keys: Vec<Vec<Utf32String>>,
    weights: Vec<f64>,
    total_weight: f64,
    matcher: RefCell<Matcher>,
}

fn to_utf32(texts: &[String]) -> Vec<Utf32String> {
    texts
        .iter()
        .map(|s| Utf32String::from(s.as_str()))
        .collect()
}

#[napi]
impl KeyedFuzzyIndex {
    /// Create a new KeyedFuzzyIndex.
    ///
    /// `key_texts[k]` is an array of strings for key `k`, one per item.
    /// All inner arrays must have the same length (the number of items).
    #[napi(constructor)]
    pub fn new(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> Self {
        let utf32_keys: Vec<Vec<Utf32String>> = key_texts.iter().map(|t| to_utf32(t)).collect();
        let total_weight: f64 = weights.iter().sum();
        Self {
            key_texts,
            utf32_keys,
            weights,
            total_weight,
            matcher: RefCell::new(Matcher::new(Config::DEFAULT)),
        }
    }

    /// Return the number of items in the index.
    #[napi(getter)]
    pub fn size(&self) -> u32 {
        self.key_texts.first().map_or(0, |v| v.len() as u32)
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined weighted score (best match first).
    #[napi]
    pub fn search(&self, query: String, options: Option<SearchOptions>) -> Vec<KeySearchResult> {
        let num_keys = self.key_texts.len();

        if num_keys == 0 {
            return Vec::new();
        }

        let num_items = self.size() as usize;
        if num_items == 0 {
            return Vec::new();
        }

        let return_all_on_empty = options
            .as_ref()
            .and_then(|o| o.return_all_on_empty)
            .unwrap_or(false);

        if return_all_on_empty && query.trim().is_empty() {
            let max_results = options.as_ref().and_then(|o| o.max_results);
            let limit = max_results.unwrap_or(num_items as u32) as usize;
            return (0..num_items)
                .take(limit)
                .map(|i| KeySearchResult {
                    index: i as u32,
                    score: 1.0,
                    key_scores: vec![1.0; num_keys],
                })
                .collect();
        }

        if query.is_empty() {
            return Vec::new();
        }

        let (max_results, min_score, case_matching) = match &options {
            Some(opts) => (
                opts.max_results,
                opts.min_score,
                resolve_case_matching(opts.is_case_sensitive),
            ),
            None => (None, None, CaseMatching::Smart),
        };

        let threshold = min_score.unwrap_or(0.0);

        let mut matcher = self.matcher.borrow_mut();
        let pattern = Pattern::parse(&query, case_matching, Normalization::Smart);
        let max_score = compute_max_score(&query, &pattern, &mut matcher);

        let mut per_key_scores: Vec<Vec<f64>> = Vec::with_capacity(num_keys);

        for utf32_texts in &self.utf32_keys {
            let mut scores = Vec::with_capacity(num_items);
            for utf32_item in utf32_texts {
                let atoms = utf32_item.slice(..);
                let score = match pattern.score(atoms, &mut matcher) {
                    Some(raw) => ((raw as f64) / max_score).min(1.0),
                    None => 0.0,
                };
                scores.push(score);
            }
            per_key_scores.push(scores);
        }

        // Pass 1: Compute combined scores, collect (index, score) only.
        let mut scored: Vec<(u32, f64)> = (0..num_items)
            .filter_map(|i| {
                let mut weighted_sum = 0.0;
                let mut matched_any = false;

                for (k, key_score_vec) in per_key_scores.iter().enumerate() {
                    let score = key_score_vec[i];
                    if score > 0.0 {
                        weighted_sum += score * self.weights[k];
                        matched_any = true;
                    }
                }

                if !matched_any {
                    return None;
                }

                let combined = weighted_sum / self.total_weight;
                if combined >= threshold {
                    Some((i as u32, combined))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(max) = max_results {
            scored.truncate(max as usize);
        }

        // Pass 2: Construct KeySearchResult only for the final top-k items.
        scored
            .into_iter()
            .map(|(index, score)| {
                let key_scores: Vec<f64> = per_key_scores
                    .iter()
                    .map(|scores| scores[index as usize])
                    .collect();
                KeySearchResult {
                    index,
                    score,
                    key_scores,
                }
            })
            .collect()
    }

    /// Add a single item to the index.
    ///
    /// `key_values` must have the same length as the number of keys.
    /// Throws if the length does not match.
    #[napi]
    pub fn add(&mut self, key_values: Vec<String>) -> napi::Result<()> {
        self.add_impl(key_values).map_err(napi::Error::from_reason)
    }

    /// Add multiple items to the index at once.
    ///
    /// Each element of `items_key_values` is an array of key values for one item.
    /// Throws if any element has the wrong number of key values.
    #[napi]
    pub fn add_many(&mut self, items_key_values: Vec<Vec<String>>) -> napi::Result<()> {
        for key_values in items_key_values {
            self.add_impl(key_values)
                .map_err(napi::Error::from_reason)?;
        }
        Ok(())
    }

    fn add_impl(&mut self, key_values: Vec<String>) -> Result<(), String> {
        let num_keys = self.key_texts.len();
        if key_values.len() != num_keys {
            return Err(format!(
                "Expected {num_keys} key values, got {}",
                key_values.len()
            ));
        }
        for (k, value) in key_values.into_iter().enumerate() {
            self.utf32_keys[k].push(Utf32String::from(value.as_str()));
            self.key_texts[k].push(value);
        }
        Ok(())
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    #[napi]
    pub fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        let num_items = self.size() as usize;
        if idx >= num_items {
            return false;
        }
        for (texts, utf32) in self.key_texts.iter_mut().zip(self.utf32_keys.iter_mut()) {
            texts.swap_remove(idx);
            utf32.swap_remove(idx);
        }
        true
    }

    /// Free the internal data. After calling this, the index is empty.
    #[napi]
    pub fn destroy(&mut self) {
        self.key_texts = Vec::new();
        self.utf32_keys = Vec::new();
        self.weights = Vec::new();
        self.total_weight = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index() -> KeyedFuzzyIndex {
        KeyedFuzzyIndex::new(
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
}
