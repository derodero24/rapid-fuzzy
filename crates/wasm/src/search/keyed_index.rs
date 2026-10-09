use rapid_fuzzy_core::search::KeyedFuzzyIndexCore;
use rapid_fuzzy_core::search::serialization::{deserialize_keyed_index, serialize_keyed_index};
use wasm_bindgen::prelude::*;

use super::keys::KeySearchResult;
use super::{KeyClosestOptions, KeySearchOptions};
use crate::convert::{error, from_js, or_null, string_matrix_from_js, to_js};

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
    /// `keyTexts[k]` is an array of strings for key `k`, one per item.
    /// All inner arrays must have the same length (the number of items).
    /// `weights` holds one finite, non-negative weight per key.
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(js_name = "keyTexts", unchecked_param_type = "string[][]")]
        key_texts: JsValue,
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] weights: Vec<f64>,
    ) -> Result<KeyedFuzzyIndex, JsValue> {
        let key_texts = string_matrix_from_js(key_texts, "keyTexts")?;
        KeyedFuzzyIndexCore::new(key_texts, weights)
            .map(|core| Self { core })
            .map_err(|e| error(&e))
    }

    /// Return the number of items in the index.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.core.size()
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined score (best match first), exactly
    /// like `searchKeys()` on the same key texts and weights. The second
    /// argument accepts either a number (maxResults) or a KeySearchOptions
    /// object, whose `matchMode` selects how the query is matched against
    /// the keys and `scoreMode` how the per-key scores are combined.
    #[wasm_bindgen(unchecked_return_type = "KeySearchResult[]")]
    pub fn search(
        &self,
        query: String,
        #[wasm_bindgen(unchecked_optional_param_type = "number | KeySearchOptions | null")]
        options: Option<JsValue>,
    ) -> Result<JsValue, JsValue> {
        let opts = KeySearchOptions::from_js_or_max_results(options)?.to_core();
        let results: Vec<KeySearchResult> = self
            .core
            .search_with_options(&query, opts)
            .into_iter()
            .map(KeySearchResult::from)
            .collect();
        to_js(&results)
    }

    /// Find the index of the closest matching item.
    ///
    /// Returns the index of the best match, or null if no match is found:
    /// the index of the first result of
    /// `search(query, { maxResults: 1, minScore, scoreMode, matchMode })`.
    ///
    /// The second argument accepts either a number (minScore shorthand) or a
    /// KeyClosestOptions object: `minScore` makes it return null when the
    /// best match scores below the threshold, and `scoreMode` and
    /// `matchMode` work like the `search()` options of the same names
    /// (defaults `"weighted"` and `"perKey"`).
    #[wasm_bindgen(unchecked_return_type = "number | null")]
    pub fn closest(
        &self,
        query: String,
        #[wasm_bindgen(unchecked_optional_param_type = "number | KeyClosestOptions | null")]
        options: Option<JsValue>,
    ) -> Result<JsValue, JsValue> {
        let options = KeyClosestOptions::from_js_or_min_score(options)?.to_core();
        let results = self.core.search_with_options(&query, options);
        Ok(or_null(results.into_iter().next().map(|r| r.index)))
    }

    /// Add a single item to the index.
    ///
    /// `keyValues` must have the same length as the number of keys.
    /// Throws if the length does not match.
    pub fn add(
        &mut self,
        #[wasm_bindgen(js_name = "keyValues", unchecked_param_type = "string[]")]
        key_values: JsValue,
    ) -> Result<(), JsValue> {
        let key_values: Vec<String> = from_js(key_values, "keyValues")?;
        self.core.add(key_values).map_err(|e| error(&e))
    }

    /// Add multiple items to the index at once.
    ///
    /// Each element of `itemsKeyValues` is an array of key values for one item.
    /// Throws if any element has the wrong number of key values.
    #[wasm_bindgen(js_name = "addMany")]
    pub fn add_many(
        &mut self,
        #[wasm_bindgen(js_name = "itemsKeyValues", unchecked_param_type = "string[][]")]
        items_key_values: JsValue,
    ) -> Result<(), JsValue> {
        let items = string_matrix_from_js(items_key_values, "itemsKeyValues")?;
        self.core.add_many(items).map_err(|e| error(&e))
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    pub fn remove(&mut self, index: u32) -> bool {
        self.core.remove(index)
    }

    /// Free the internal data. After calling this, the index is empty.
    ///
    /// The key configuration is kept, so the index stays usable: it behaves
    /// as an empty index and `add()` / `addMany()` work as before.
    pub fn destroy(&mut self) {
        self.core.destroy();
    }

    /// Serialize the index to a compact binary format (Uint8Array).
    pub fn serialize(&self) -> Vec<u8> {
        serialize_keyed_index(&self.core)
    }

    /// Reconstruct a KeyedFuzzyIndex from a previously serialized Uint8Array.
    pub fn deserialize(data: &[u8]) -> Result<KeyedFuzzyIndex, JsValue> {
        deserialize_keyed_index(data)
            .map(|core| Self { core })
            .map_err(|e| error(&e))
    }
}
