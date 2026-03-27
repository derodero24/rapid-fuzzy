use nucleo_matcher::pattern::{Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::{SearchOptions, compute_max_score, resolve_case_matching, to_js};

/// A single result from multi-key fuzzy search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeySearchResult {
    pub index: u32,
    pub score: f64,
    pub key_scores: Vec<f64>,
}

/// Perform fuzzy search across multiple text keys with weights.
///
/// `key_texts` is a JS Array of Arrays of strings (one inner array per key,
/// each inner array has one string per item).
/// `weights` is a JS Array of numbers specifying the relative importance of each key.
///
/// Returns results sorted by combined weighted score as a JS Array.
#[wasm_bindgen(js_name = "searchKeys")]
pub fn search_keys(
    query: String,
    key_texts: JsValue,
    weights: Vec<f64>,
    options: Option<SearchOptions>,
) -> JsValue {
    let key_texts: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(key_texts) {
        Ok(v) => v,
        Err(_) => return to_js::<Vec<KeySearchResult>>(&Vec::new()),
    };
    let results = search_keys_impl(query, key_texts, weights, options);
    to_js(&results)
}

pub(crate) fn search_keys_impl(
    query: String,
    key_texts: Vec<Vec<String>>,
    weights: Vec<f64>,
    options: Option<SearchOptions>,
) -> Vec<KeySearchResult> {
    let num_keys = key_texts.len();
    if num_keys == 0 || weights.len() != num_keys {
        return Vec::new();
    }
    let num_items = key_texts[0].len();
    if num_items == 0 {
        return Vec::new();
    }
    for texts in &key_texts {
        if texts.len() != num_items {
            return Vec::new();
        }
    }
    let opts = options.unwrap_or_default();
    let return_all_on_empty = opts.return_all_on_empty.unwrap_or(false);
    if return_all_on_empty && query.trim().is_empty() {
        let max_results = opts.max_results;
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
    let max_results = opts.max_results;
    let min_score = opts.min_score;
    let case_matching = resolve_case_matching(opts.is_case_sensitive);
    let threshold = min_score.unwrap_or(0.0);
    if weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
        return Vec::new();
    }
    let total_weight: f64 = weights.iter().sum();
    if total_weight <= 0.0 {
        return Vec::new();
    }
    use nucleo_matcher::Utf32String;
    let utf32_keys: Vec<Vec<Utf32String>> = key_texts
        .iter()
        .map(|texts| {
            texts
                .iter()
                .map(|s| Utf32String::from(s.as_str()))
                .collect()
        })
        .collect();
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(&query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(&query, &pattern, &mut matcher);
    let mut per_key_scores: Vec<Vec<f64>> = Vec::with_capacity(num_keys);
    let mut buf: Vec<u32> = Vec::new();
    #[allow(clippy::needless_range_loop)]
    for k in 0..num_keys {
        let mut scores = Vec::with_capacity(num_items);
        #[allow(clippy::needless_range_loop)]
        for i in 0..num_items {
            let atoms = utf32_keys[k][i].slice(..);
            let score = match pattern.score(atoms, &mut matcher) {
                Some(raw) => ((raw as f64) / max_score).min(1.0),
                None => 0.0,
            };
            scores.push(score);
        }
        per_key_scores.push(scores);
        buf.clear();
    }
    let threshold_weighted = threshold * total_weight;
    let mut scored: Vec<(u32, f64)> = (0..num_items)
        .filter_map(|i| {
            let mut weighted_sum = 0.0;
            let mut matched_any = false;
            for k in 0..num_keys {
                let w = weights[k];
                let s = per_key_scores[k][i];
                if s > 0.0 {
                    weighted_sum += s * w;
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
    let _ = threshold_weighted;
    let cmp = |a: &(u32, f64), b: &(u32, f64)| {
        let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
        if score_ord != std::cmp::Ordering::Equal {
            return score_ord;
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
    scored
        .into_iter()
        .map(|(index, score)| {
            let key_scores: Vec<f64> = per_key_scores.iter().map(|s| s[index as usize]).collect();
            KeySearchResult {
                index,
                score,
                key_scores,
            }
        })
        .collect()
}
