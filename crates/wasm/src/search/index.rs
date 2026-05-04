use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::FuzzyIndexCore;
use rapid_fuzzy_core::search::serialization::{
    FUZZY_INDEX_WASM_MAGIC, deserialize_items, serialize_items,
};
use wasm_bindgen::prelude::*;

use super::{IndexSearchResult, SearchOptions, SearchResult, resolve_case_matching, to_js};

/// A persistent fuzzy search index backed by Rust-side data.
///
/// Holds items in memory on the Rust side, avoiding repeated FFI overhead
/// for applications that search the same dataset multiple times.
#[wasm_bindgen]
pub struct FuzzyIndex {
    core: FuzzyIndexCore,
}

#[wasm_bindgen]
impl FuzzyIndex {
    /// Create a new FuzzyIndex from an array of strings.
    #[wasm_bindgen(constructor)]
    pub fn new(items: Vec<String>) -> Self {
        Self {
            core: FuzzyIndexCore::new(items),
        }
    }

    /// Return the number of items in the index.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.core.size()
    }

    /// Search the index for items matching the query.
    ///
    /// Returns matches sorted by score (best match first) as a JS Array.
    pub fn search(&self, query: String, options: Option<SearchOptions>) -> JsValue {
        let opts = options.unwrap_or_default();
        let (max_results, min_score, include_positions, case_matching, return_all_on_empty) = (
            opts.max_results,
            opts.min_score,
            opts.include_positions.unwrap_or(false),
            resolve_case_matching(opts.is_case_sensitive),
            opts.return_all_on_empty.unwrap_or(false),
        );

        if return_all_on_empty && query.trim().is_empty() {
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
    pub fn closest(&self, query: String, min_score: Option<f64>) -> Option<String> {
        let results = self
            .core
            .search_impl(&query, Some(1), min_score, false, CaseMatching::Smart);
        results.into_iter().next().map(|r| r.item)
    }

    /// Search the index, returning only indices and scores (no item strings).
    #[wasm_bindgen(js_name = "searchIndices")]
    pub fn search_indices(&self, query: String, options: Option<SearchOptions>) -> JsValue {
        let opts = options.unwrap_or_default();
        let (max_results, min_score, include_positions, case_matching, return_all_on_empty) = (
            opts.max_results,
            opts.min_score,
            opts.include_positions.unwrap_or(false),
            resolve_case_matching(opts.is_case_sensitive),
            opts.return_all_on_empty.unwrap_or(false),
        );

        if return_all_on_empty && query.trim().is_empty() {
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
    #[wasm_bindgen(js_name = "addMany")]
    pub fn add_many(&mut self, items: Vec<String>) {
        self.core.add_many(items);
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
        serialize_items(self.core.items(), FUZZY_INDEX_WASM_MAGIC)
    }

    /// Reconstruct a FuzzyIndex from a previously serialized Uint8Array.
    pub fn deserialize(data: &[u8]) -> Result<FuzzyIndex, JsValue> {
        let items =
            deserialize_items(data, FUZZY_INDEX_WASM_MAGIC).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self::new(items))
    }
}
