use std::cell::RefCell;

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};

use super::{KeySearchResult, compute_char_mask, compute_max_score, compute_query_mask};

fn to_utf32(texts: &[String]) -> Vec<Utf32String> {
    texts
        .iter()
        .map(|s| Utf32String::from(s.as_str()))
        .collect()
}

/// Core state and logic for a persistent multi-key fuzzy search index.
///
/// This struct contains all platform-independent state and methods.
/// Binding crates (napi, wasm) wrap this with their own FFI layer.
pub struct KeyedFuzzyIndexCore {
    key_texts: Vec<Vec<String>>,
    utf32_keys: Vec<Vec<Utf32String>>,
    key_char_masks: Vec<Vec<u64>>,
    weights: Vec<f64>,
    total_weight: f64,
    matcher: RefCell<Matcher>,
}

impl KeyedFuzzyIndexCore {
    /// Create a new KeyedFuzzyIndexCore.
    ///
    /// `key_texts[k]` is an array of strings for key `k`, one per item.
    /// All inner arrays must have the same length (the number of items).
    pub fn new(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> Result<Self, String> {
        let num_keys = key_texts.len();

        if let Some(num_items) = key_texts.first().map(Vec::len) {
            for (k, col) in key_texts.iter().enumerate().skip(1) {
                if col.len() != num_items {
                    return Err(format!(
                        "All key_texts columns must have the same length; key 0 has {}, key {} has {}",
                        num_items,
                        k,
                        col.len()
                    ));
                }
            }
        }

        if weights.len() != num_keys {
            return Err(format!(
                "Expected {} weights, got {}",
                num_keys,
                weights.len()
            ));
        }

        if weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
            return Err("Weights must be finite non-negative numbers".to_string());
        }

        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return Err("Total weight must be greater than zero".to_string());
        }

        let utf32_keys: Vec<Vec<Utf32String>> = key_texts.iter().map(|t| to_utf32(t)).collect();
        let key_char_masks: Vec<Vec<u64>> = key_texts
            .iter()
            .map(|texts| texts.iter().map(|s| compute_char_mask(s)).collect())
            .collect();
        Ok(Self {
            key_texts,
            utf32_keys,
            key_char_masks,
            weights,
            total_weight,
            matcher: RefCell::new(Matcher::new(Config::DEFAULT)),
        })
    }

    /// Return the number of items in the index.
    pub fn size(&self) -> u32 {
        self.key_texts.first().map_or(0, |v| v.len() as u32)
    }

    /// Access the key_texts.
    pub fn key_texts(&self) -> &[Vec<String>] {
        &self.key_texts
    }

    /// Access the weights.
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined weighted score (best match first).
    pub fn search(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        case_matching: CaseMatching,
        return_all_on_empty: bool,
    ) -> Vec<KeySearchResult> {
        let num_keys = self.key_texts.len();

        if num_keys == 0 {
            return Vec::new();
        }

        let num_items = self.size() as usize;
        if num_items == 0 {
            return Vec::new();
        }

        if return_all_on_empty && query.trim().is_empty() {
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

        let threshold = min_score.unwrap_or(0.0);

        let mut matcher = self.matcher.borrow_mut();
        let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
        let max_score = compute_max_score(query, &pattern, &mut matcher);
        let query_mask = compute_query_mask(query);

        // Identify keys with non-zero weight to skip unnecessary scoring.
        let active_keys: Vec<usize> = (0..num_keys).filter(|&k| self.weights[k] > 0.0).collect();

        // Pre-compute the sum of active weights for early exit upper-bound.
        let active_total_weight: f64 = active_keys.iter().map(|&k| self.weights[k]).sum();

        // Per-item scoring with early exit and per-key char_mask pre-filtering.
        let mut per_key_scores: Vec<Vec<f64>> = vec![vec![0.0; num_items]; num_keys];

        let threshold_weighted = threshold * self.total_weight;

        let mut scored: Vec<(u32, f64)> = (0..num_items)
            .filter_map(|i| {
                let mut weighted_sum = 0.0;
                let mut matched_any = false;
                let mut remaining_weight = active_total_weight;

                for &k in &active_keys {
                    let w = self.weights[k];
                    remaining_weight -= w;

                    // Per-key char_mask pre-filter: skip expensive scoring if
                    // the item for this key cannot contain the query characters.
                    if query_mask != 0 && (self.key_char_masks[k][i] & query_mask) != query_mask {
                        // Upper bound check: even if all remaining keys score 1.0,
                        // can we still reach the threshold?
                        if threshold > 0.0 && weighted_sum + remaining_weight < threshold_weighted {
                            return None;
                        }
                        continue;
                    }

                    let atoms = self.utf32_keys[k][i].slice(..);
                    let score = match pattern.score(atoms, &mut matcher) {
                        Some(raw) => ((raw as f64) / max_score).min(1.0),
                        None => 0.0,
                    };
                    per_key_scores[k][i] = score;

                    if score > 0.0 {
                        weighted_sum += score * w;
                        matched_any = true;
                    }

                    // Early exit: if even perfect scores on remaining keys
                    // cannot lift the combined score above the threshold,
                    // skip the remaining keys for this item.
                    if threshold > 0.0 && weighted_sum + remaining_weight < threshold_weighted {
                        return None;
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

        // Sort by score descending, with original index as tiebreaker
        // for deterministic ordering.
        let cmp = |a: &(u32, f64), b: &(u32, f64)| {
            let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
            if score_ord != std::cmp::Ordering::Equal {
                return score_ord;
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
    pub fn add(&mut self, key_values: Vec<String>) -> Result<(), String> {
        let num_keys = self.key_texts.len();
        if key_values.len() != num_keys {
            return Err(format!(
                "Expected {num_keys} key values, got {}",
                key_values.len()
            ));
        }
        for (k, value) in key_values.into_iter().enumerate() {
            self.utf32_keys[k].push(Utf32String::from(value.as_str()));
            self.key_char_masks[k].push(compute_char_mask(&value));
            self.key_texts[k].push(value);
        }
        Ok(())
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    pub fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        let num_items = self.size() as usize;
        if idx >= num_items {
            return false;
        }
        for ((texts, utf32), masks) in self
            .key_texts
            .iter_mut()
            .zip(self.utf32_keys.iter_mut())
            .zip(self.key_char_masks.iter_mut())
        {
            texts.swap_remove(idx);
            utf32.swap_remove(idx);
            masks.swap_remove(idx);
        }
        true
    }

    /// Free the internal data. After calling this, the index is empty.
    pub fn destroy(&mut self) {
        self.key_texts = Vec::new();
        self.utf32_keys = Vec::new();
        self.key_char_masks = Vec::new();
        self.weights = Vec::new();
        self.total_weight = 0.0;
    }
}
