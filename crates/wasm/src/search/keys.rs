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

/// A JavaScript `Error` with the given message.
fn js_error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}

/// Validate a numeric `maxResults` like the Node.js binding does:
/// non-negative integers are accepted (values beyond `u32::MAX` mean no
/// limit), `Infinity` means no limit, and NaN, negative or fractional values
/// are rejected.
fn max_results_from_number(value: f64) -> Result<Option<u32>, JsValue> {
    if value == f64::INFINITY {
        return Ok(None);
    }
    if value.is_nan() || value < 0.0 || value.fract() != 0.0 {
        let shown = if value.is_nan() {
            "NaN".to_string()
        } else if value.is_infinite() {
            "-Infinity".to_string()
        } else {
            value.to_string()
        };
        return Err(js_error(&format!(
            "maxResults must be a non-negative integer or Infinity, got {shown}"
        )));
    }
    Ok(Some(if value >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        value as u32
    }))
}

/// Read the `options` argument of `searchKeys`: `undefined`/`null`, a
/// number (shorthand for `{ maxResults }`) or a `SearchOptions` object.
fn search_keys_options(options: Option<JsValue>) -> Result<SearchKeysOptions, JsValue> {
    let opts = match options {
        None => SearchOptions::default(),
        Some(value) => match value.as_f64() {
            Some(max_results) => SearchOptions {
                max_results: max_results_from_number(max_results)?,
                ..SearchOptions::default()
            },
            None => serde_wasm_bindgen::from_value(value)
                .map_err(|e| js_error(&format!("Invalid SearchOptions: {e}")))?,
        },
    };
    Ok(SearchKeysOptions {
        max_results: opts.max_results,
        min_score: opts.min_score,
        is_case_sensitive: opts.is_case_sensitive,
        return_all_on_empty: opts.return_all_on_empty,
    })
}

/// Perform fuzzy search across multiple text keys with weights.
///
/// `key_texts` is a JS Array of Arrays of strings (one inner array per key,
/// each inner array has one string per item).
/// `weights` is a JS Array of numbers specifying the relative importance of each key.
/// `options` is a `SearchOptions` object or a number (maxResults).
///
/// Returns results sorted by combined weighted score as a JS Array, exactly
/// like `KeyedFuzzyIndex.search` on the same key texts and weights. Throws an
/// `Error` for invalid input (key texts of different lengths, a weight count
/// that differs from the key count, negative, NaN or infinite weights, or
/// weights summing to 0 or Infinity), like the `KeyedFuzzyIndex` constructor.
#[wasm_bindgen(js_name = "searchKeys")]
pub fn search_keys(
    query: String,
    key_texts: JsValue,
    weights: Vec<f64>,
    #[wasm_bindgen(unchecked_optional_param_type = "number | SearchOptions | null")]
    options: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let key_texts: Vec<Vec<String>> =
        serde_wasm_bindgen::from_value(key_texts).map_err(|e| js_error(&e.to_string()))?;
    let core_opts = search_keys_options(options)?;

    let results: Vec<KeySearchResult> =
        rapid_fuzzy_core::search::search_keys_impl(&query, &key_texts, &weights, Some(core_opts))
            .map_err(|e| js_error(&e))?
            .into_iter()
            .map(KeySearchResult::from)
            .collect();
    Ok(to_js(&results))
}
