use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::KeySearchOptions;
use crate::convert::{error, string_matrix_from_js, to_js};

/// A single result from multi-key fuzzy search.
#[derive(Debug, Clone, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct KeySearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The combined score (0.0-1.0) of the key scores, as set by
    /// `scoreMode` (by default the weighted mean over all keys).
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
/// `keyTexts[k]` is an array of strings for key `k`, one per item.
/// `weights` specifies the relative importance of each key.
/// `options` is a `KeySearchOptions` object or a number (maxResults); its
/// `scoreMode` selects how the per-key scores are combined.
///
/// Returns results sorted by combined score (best match first), exactly like
/// `KeyedFuzzyIndex.search` on the same key texts and weights.
/// Throws an `Error` for invalid input (key texts of different lengths, a
/// weight count that differs from the key count, negative, NaN or infinite
/// weights, or weights summing to 0 or Infinity), like the `KeyedFuzzyIndex`
/// constructor, and a `TypeError` for invalid options (such as an unknown
/// `scoreMode`).
#[wasm_bindgen(js_name = "searchKeys", unchecked_return_type = "KeySearchResult[]")]
pub fn search_keys(
    query: String,
    #[wasm_bindgen(js_name = "keyTexts", unchecked_param_type = "string[][]")] key_texts: JsValue,
    #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] weights: Vec<f64>,
    #[wasm_bindgen(unchecked_optional_param_type = "number | KeySearchOptions | null")]
    options: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let key_texts = string_matrix_from_js(key_texts, "keyTexts")?;
    let core_opts = KeySearchOptions::from_js_or_max_results(options)?.to_core();

    let results: Vec<KeySearchResult> =
        rapid_fuzzy_core::search::search_keys_impl(&query, &key_texts, &weights, Some(core_opts))
            .map_err(|e| error(&e))?
            .into_iter()
            .map(KeySearchResult::from)
            .collect();
    to_js(&results)
}
