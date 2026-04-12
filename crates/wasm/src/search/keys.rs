use rapid_fuzzy_core::search::SearchKeysOptions;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::{SearchOptions, to_js};

/// A single result from multi-key fuzzy search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeySearchResult {
    pub index: u32,
    pub score: f64,
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

    let core_opts = options.map(|opts| SearchKeysOptions {
        max_results: opts.max_results,
        min_score: opts.min_score,
        is_case_sensitive: opts.is_case_sensitive,
        return_all_on_empty: opts.return_all_on_empty,
    });

    let results: Vec<KeySearchResult> =
        rapid_fuzzy_core::search::search_keys_impl(&query, &key_texts, &weights, core_opts)
            .into_iter()
            .map(KeySearchResult::from)
            .collect();
    to_js(&results)
}
