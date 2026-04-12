use rapid_fuzzy_core::search::KeyedFuzzyIndexCore;
use rapid_fuzzy_core::search::serialization::{
    KEYED_INDEX_MAGIC, deserialize_keyed, serialize_keyed,
};
use wasm_bindgen::prelude::*;

use super::keys::KeySearchResult;
use super::{SearchOptions, resolve_case_matching, to_js};

/// A persistent multi-key fuzzy search index backed by Rust-side data.
///
/// Holds key text arrays and weights in memory on the Rust side.
#[wasm_bindgen]
pub struct KeyedFuzzyIndex {
    core: KeyedFuzzyIndexCore,
}

#[wasm_bindgen]
impl KeyedFuzzyIndex {
    /// Create a new KeyedFuzzyIndex.
    ///
    /// `key_texts` is a JS Array of Arrays of strings (one inner array per key,
    /// each inner array has one string per item).
    /// `weights` is a JS Array of numbers.
    #[wasm_bindgen(constructor)]
    pub fn new(key_texts: JsValue, weights: Vec<f64>) -> Result<KeyedFuzzyIndex, JsValue> {
        let key_texts: Vec<Vec<String>> = serde_wasm_bindgen::from_value(key_texts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        KeyedFuzzyIndexCore::new(key_texts, weights)
            .map(|core| Self { core })
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Return the number of items in the index.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.core.size()
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined weighted score as a JS Array.
    pub fn search(&self, query: String, options: Option<SearchOptions>) -> JsValue {
        let (max_results, min_score, case_matching, return_all_on_empty) = match &options {
            Some(opts) => (
                opts.max_results,
                opts.min_score,
                resolve_case_matching(opts.is_case_sensitive),
                opts.return_all_on_empty.unwrap_or(false),
            ),
            None => (
                None,
                None,
                nucleo_matcher::pattern::CaseMatching::Smart,
                false,
            ),
        };

        let results: Vec<KeySearchResult> = self
            .core
            .search(
                &query,
                max_results,
                min_score,
                case_matching,
                return_all_on_empty,
            )
            .into_iter()
            .map(KeySearchResult::from)
            .collect();
        to_js(&results)
    }

    /// Find the index of the closest matching item.
    pub fn closest(&self, query: String, min_score: Option<f64>) -> Option<u32> {
        let results = self.core.search(
            &query,
            Some(1),
            min_score,
            nucleo_matcher::pattern::CaseMatching::Smart,
            false,
        );
        results.into_iter().next().map(|r| r.index)
    }

    /// Add a single item to the index.
    ///
    /// `key_values` must be a JS Array of strings with one value per key.
    pub fn add(&mut self, key_values: JsValue) -> Result<(), JsValue> {
        let key_values: Vec<String> = serde_wasm_bindgen::from_value(key_values)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.core.add(key_values).map_err(|e| JsValue::from_str(&e))
    }

    /// Add multiple items to the index at once.
    ///
    /// `items_key_values` is a JS Array where each element is an Array of strings
    /// (one per key). Throws if any element has the wrong number of key values.
    #[wasm_bindgen(js_name = "addMany")]
    pub fn add_many(&mut self, items_key_values: JsValue) -> Result<(), JsValue> {
        let items: Vec<Vec<String>> = serde_wasm_bindgen::from_value(items_key_values)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        for key_values in items {
            self.core
                .add(key_values)
                .map_err(|e| JsValue::from_str(&e))?;
        }
        Ok(())
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    pub fn remove(&mut self, index: u32) -> bool {
        self.core.remove(index)
    }

    /// Free the internal data. After calling this, the index is empty.
    pub fn destroy(&mut self) {
        self.core.destroy();
    }

    /// Serialize the index to a compact binary format (Uint8Array).
    pub fn serialize(&self) -> Vec<u8> {
        serialize_keyed(
            self.core.key_texts(),
            self.core.weights(),
            KEYED_INDEX_MAGIC,
        )
    }

    /// Reconstruct a KeyedFuzzyIndex from a previously serialized Uint8Array.
    pub fn deserialize(data: &[u8]) -> Result<KeyedFuzzyIndex, JsValue> {
        let (key_texts, weights) =
            deserialize_keyed(data, KEYED_INDEX_MAGIC).map_err(|e| JsValue::from_str(&e))?;
        KeyedFuzzyIndexCore::new(key_texts, weights)
            .map(|core| Self { core })
            .map_err(|e| JsValue::from_str(&e))
    }
}
