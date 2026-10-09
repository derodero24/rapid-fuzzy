use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::serialization::{deserialize_fuzzy_index, serialize_fuzzy_index};
use rapid_fuzzy_core::search::{FuzzyIndexCore, is_empty_query};
use wasm_bindgen::prelude::*;

use super::{IndexSearchResult, SearchOptions, SearchResult, min_score_arg, resolve_case_matching};
use crate::convert::{error, or_null, remove_index_from_js, strings_from_js, to_js};

/// A persistent fuzzy search index backed by Rust-side data.
///
/// Holds items in memory on the Rust side, avoiding repeated FFI overhead
/// for applications that search the same dataset multiple times.
#[wasm_bindgen]
pub struct FuzzyIndex {
    core: FuzzyIndexCore,
}

impl FuzzyIndex {
    fn from_items(items: Vec<String>) -> Self {
        Self {
            core: FuzzyIndexCore::new(items),
        }
    }
}

#[wasm_bindgen]
impl FuzzyIndex {
    /// Create a new FuzzyIndex from an array of strings.
    ///
    /// Throws a `TypeError` for anything but an array of strings.
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(unchecked_param_type = "string[]")] items: JsValue,
    ) -> Result<FuzzyIndex, JsValue> {
        Ok(Self::from_items(strings_from_js(&items)?))
    }

    /// Return the number of items in the index.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.core.size()
    }

    /// Create a new FuzzyIndex, returning a Promise (parity with the Node.js binding).
    ///
    /// The WebAssembly build has no worker thread, so the index is built
    /// synchronously on the calling thread and the returned Promise is already
    /// resolved. Prefer the constructor when you do not need a Promise.
    ///
    /// Invalid input (anything but an array of strings) rejects the returned
    /// Promise with a `TypeError` instead of throwing synchronously.
    #[wasm_bindgen(js_name = "fromAsync", unchecked_return_type = "Promise<FuzzyIndex>")]
    pub fn from_async(
        #[wasm_bindgen(unchecked_param_type = "string[]")] items: JsValue,
    ) -> js_sys::Promise {
        match strings_from_js(&items) {
            Ok(items) => js_sys::Promise::resolve(&JsValue::from(Self::from_items(items))),
            Err(err) => js_sys::Promise::reject(&err),
        }
    }

    /// Search the index for items matching the query.
    ///
    /// Returns matches sorted by score (best match first).
    /// The second argument accepts either a number (maxResults) or a SearchOptions object.
    #[wasm_bindgen(unchecked_return_type = "SearchResult[]")]
    pub fn search(
        &self,
        query: String,
        #[wasm_bindgen(unchecked_optional_param_type = "number | SearchOptions | null")]
        options: Option<JsValue>,
    ) -> Result<JsValue, JsValue> {
        let opts = SearchOptions::from_js_or_max_results(options)?;
        let (max_results, min_score, include_positions, case_matching, return_all_on_empty) = (
            opts.max_results,
            opts.min_score,
            opts.include_positions.unwrap_or(false),
            resolve_case_matching(opts.is_case_sensitive),
            opts.return_all_on_empty.unwrap_or(false),
        );

        if return_all_on_empty && is_empty_query(&query) {
            let items = self.core.items();
            let limit = max_results.unwrap_or(items.len() as u32) as usize;
            let results: Vec<SearchResult> = items
                .iter()
                .enumerate()
                .take(limit)
                .map(|(i, item)| SearchResult {
                    item: item.clone(),
                    score: 1.0,
                    index: i as u32,
                    positions: Vec::new(),
                    match_type: None,
                })
                .collect();
            return to_js(&results);
        }

        let results: Vec<SearchResult> = self
            .core
            .search_impl(
                &query,
                max_results,
                min_score,
                include_positions,
                case_matching,
            )
            .into_iter()
            .map(SearchResult::from)
            .collect();
        to_js(&results)
    }

    /// Find the closest matching string in the index.
    ///
    /// Returns the best match, or null if no match is found.
    /// If `minScore` is provided, returns null when the best match scores below
    /// the threshold. A NaN `minScore` throws a `TypeError`.
    #[wasm_bindgen(unchecked_return_type = "string | null")]
    pub fn closest(
        &self,
        query: String,
        #[wasm_bindgen(js_name = "minScore")] min_score: Option<f64>,
    ) -> Result<JsValue, JsValue> {
        let min_score = min_score_arg(min_score)?;
        let results = self
            .core
            .search_impl(&query, Some(1), min_score, false, CaseMatching::Smart);
        Ok(or_null(results.into_iter().next().map(|r| r.item)))
    }

    /// Search the index, returning only indices and scores (no item strings).
    ///
    /// The second argument accepts either a number (maxResults) or a SearchOptions object.
    #[wasm_bindgen(
        js_name = "searchIndices",
        unchecked_return_type = "IndexSearchResult[]"
    )]
    pub fn search_indices(
        &self,
        query: String,
        #[wasm_bindgen(unchecked_optional_param_type = "number | SearchOptions | null")]
        options: Option<JsValue>,
    ) -> Result<JsValue, JsValue> {
        let opts = SearchOptions::from_js_or_max_results(options)?;
        let (max_results, min_score, include_positions, case_matching, return_all_on_empty) = (
            opts.max_results,
            opts.min_score,
            opts.include_positions.unwrap_or(false),
            resolve_case_matching(opts.is_case_sensitive),
            opts.return_all_on_empty.unwrap_or(false),
        );

        if return_all_on_empty && is_empty_query(&query) {
            let num_items = self.core.size() as usize;
            let limit = max_results.unwrap_or(num_items as u32) as usize;
            let results: Vec<IndexSearchResult> = (0..num_items)
                .take(limit)
                .map(|i| IndexSearchResult {
                    index: i as u32,
                    score: 1.0,
                    positions: Vec::new(),
                    match_type: None,
                })
                .collect();
            return to_js(&results);
        }

        let results: Vec<IndexSearchResult> = self
            .core
            .search_indices_impl(
                &query,
                max_results,
                min_score,
                include_positions,
                case_matching,
            )
            .into_iter()
            .map(IndexSearchResult::from)
            .collect();
        to_js(&results)
    }

    /// Add a single item to the index.
    pub fn add(&mut self, item: String) {
        self.core.add(item);
    }

    /// Add multiple items to the index at once.
    ///
    /// Throws a `TypeError`, adding nothing, for anything but an array of
    /// strings.
    #[wasm_bindgen(js_name = "addMany")]
    pub fn add_many(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "string[]")] items: JsValue,
    ) -> Result<(), JsValue> {
        let items = strings_from_js(&items)?;
        self.core.add_many(items);
        Ok(())
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance: the last item moves into the
    /// freed slot. Returns false, removing nothing, if `index` is out of
    /// range (negative, or not less than `size`). Throws a `TypeError` if
    /// `index` is not a number and a `RangeError` if it is not an integer
    /// (`NaN`, `±Infinity` or a fraction), like `FuzzyObjectIndex.remove()`.
    pub fn remove(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "number")] index: JsValue,
    ) -> Result<bool, JsValue> {
        Ok(remove_index_from_js(&index)?.is_some_and(|index| self.core.remove(index)))
    }

    /// Free the internal data. After calling this, the index is empty.
    ///
    /// The index stays usable: it behaves as an empty index (searches
    /// return no results) and `add()` / `addMany()` work as before.
    pub fn destroy(&mut self) {
        self.core.destroy();
    }

    /// Serialize the index to a compact binary format (Uint8Array).
    pub fn serialize(&self) -> Vec<u8> {
        serialize_fuzzy_index(&self.core)
    }

    /// Reconstruct a FuzzyIndex from a previously serialized Uint8Array.
    pub fn deserialize(data: &[u8]) -> Result<FuzzyIndex, JsValue> {
        deserialize_fuzzy_index(data)
            .map(|core| Self { core })
            .map_err(|e| error(&e))
    }
}
