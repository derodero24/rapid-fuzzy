use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

use super::{KeySearchResult, compute_max_score, resolve_case_matching};

/// Search options for the `search_keys_impl` function.
pub struct SearchKeysOptions {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub is_case_sensitive: Option<bool>,
    pub return_all_on_empty: Option<bool>,
}

/// Perform fuzzy search across multiple text keys with weights.
///
/// `key_texts[k]` is an array of strings for key `k`, one per item.
/// All inner arrays must have the same length (the number of items).
/// `weights` specifies the relative importance of each key.
///
/// Returns results sorted by combined weighted score (best match first).
pub fn search_keys_impl(
    query: &str,
    key_texts: &[Vec<String>],
    weights: &[f64],
    options: Option<SearchKeysOptions>,
) -> Vec<KeySearchResult> {
    let num_keys = key_texts.len();

    if num_keys == 0 || weights.len() != num_keys {
        return Vec::new();
    }

    let num_items = key_texts[0].len();
    if num_items == 0 {
        return Vec::new();
    }

    // Validate all key_texts have the same length
    for texts in key_texts {
        if texts.len() != num_items {
            return Vec::new();
        }
    }

    let (max_results, min_score, case_matching, return_all_on_empty) = match &options {
        Some(opts) => (
            opts.max_results,
            opts.min_score,
            resolve_case_matching(opts.is_case_sensitive),
            opts.return_all_on_empty.unwrap_or(false),
        ),
        None => (None, None, CaseMatching::Smart, false),
    };

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

    // Reject negative, NaN, or infinite weights to guarantee scores stay in [0.0, 1.0]
    if weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
        return Vec::new();
    }

    let total_weight: f64 = weights.iter().sum();
    if total_weight <= 0.0 {
        return Vec::new();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(query, &pattern, &mut matcher);

    // Compute per-key scores for all items
    let mut per_key_scores: Vec<Vec<f64>> = Vec::with_capacity(num_keys);
    let mut buf = Vec::new();

    for texts in key_texts {
        let mut scores = Vec::with_capacity(num_items);
        for text in texts {
            buf.clear();
            let atoms = nucleo_matcher::Utf32Str::new(text, &mut buf);
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

            for k in 0..num_keys {
                let score = per_key_scores[k][i];
                if score > 0.0 {
                    weighted_sum += score * weights[k];
                    matched_any = true;
                }
            }

            if !matched_any {
                return None;
            }

            let combined = weighted_sum / total_weight;
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
