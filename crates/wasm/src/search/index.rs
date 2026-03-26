use std::cell::RefCell;

use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher, Utf32String};
use wasm_bindgen::prelude::*;

use super::{
    BigramIndex, IndexSearchResult, PrecomputedSearch, SearchOptions, SearchResult,
    compute_char_mask, extract_query_bigrams, resolve_case_matching, search_over_precomputed,
    search_over_precomputed_indices, to_js,
};

const BIGRAM_THRESHOLD: usize = 5_000;
const SERIALIZE_MAGIC: &[u8] = b"RFUZ";
const SERIALIZE_VERSION: u32 = 1;

/// A persistent fuzzy search index backed by Rust-side data.
///
/// Holds items in memory on the Rust side, avoiding repeated FFI overhead
/// for applications that search the same dataset multiple times.
#[wasm_bindgen]
pub struct FuzzyIndex {
    items: Vec<String>,
    utf32_items: Vec<Utf32String>,
    char_masks: Vec<u64>,
    bigram_index: BigramIndex,
    matcher: RefCell<Matcher>,
    last_query: RefCell<String>,
    last_matching_indices: RefCell<Vec<u32>>,
}

#[wasm_bindgen]
impl FuzzyIndex {
    /// Create a new FuzzyIndex from an array of strings.
    #[wasm_bindgen(constructor)]
    pub fn new(items: Vec<String>) -> Self {
        let utf32_items: Vec<Utf32String> = items
            .iter()
            .map(|s| Utf32String::from(s.as_str()))
            .collect();
        let char_masks: Vec<u64> = items.iter().map(|s| compute_char_mask(s)).collect();
        let bigram_index = BigramIndex::new(&items);
        Self {
            items,
            utf32_items,
            char_masks,
            bigram_index,
            matcher: RefCell::new(Matcher::new(Config::DEFAULT)),
            last_query: RefCell::new(String::new()),
            last_matching_indices: RefCell::new(Vec::new()),
        }
    }

    /// Return the number of items in the index.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.items.len() as u32
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
            let limit = max_results.unwrap_or(self.items.len() as u32) as usize;
            let results: Vec<SearchResult> = self
                .items
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

        let results = self.search_impl(
            &query,
            max_results,
            min_score,
            include_positions,
            case_matching,
        );
        to_js(&results)
    }

    /// Find the closest matching string in the index.
    ///
    /// Returns the best match, or null if no match is found.
    pub fn closest(&self, query: String, min_score: Option<f64>) -> Option<String> {
        let results = self.search_impl(&query, Some(1), min_score, false, CaseMatching::Smart);
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
            let limit = max_results.unwrap_or(self.items.len() as u32) as usize;
            let results: Vec<IndexSearchResult> = (0..self.items.len())
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

        let results = self.search_indices_impl(
            &query,
            max_results,
            min_score,
            include_positions,
            case_matching,
        );
        to_js(&results)
    }

    /// Add a single item to the index.
    pub fn add(&mut self, item: String) {
        let index = self.items.len() as u32;
        self.utf32_items.push(Utf32String::from(item.as_str()));
        self.char_masks.push(compute_char_mask(&item));
        self.bigram_index.add_item(index, &item);
        self.items.push(item);
        self.invalidate_cache();
    }

    /// Add multiple items to the index at once.
    #[wasm_bindgen(js_name = "addMany")]
    pub fn add_many(&mut self, items: Vec<String>) {
        let base = self.items.len() as u32;
        for (i, item) in items.iter().enumerate() {
            self.utf32_items.push(Utf32String::from(item.as_str()));
            self.char_masks.push(compute_char_mask(item));
            self.bigram_index.add_item(base + i as u32, item);
        }
        self.items.extend(items);
        self.invalidate_cache();
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    pub fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        if idx < self.items.len() {
            let last_index = (self.items.len() - 1) as u32;
            let removed_item = self.items[idx].clone();
            let last_item = if idx != last_index as usize {
                Some(self.items[last_index as usize].clone())
            } else {
                None
            };
            self.bigram_index
                .remove_item(index, last_index, &removed_item, last_item.as_deref());
            self.items.swap_remove(idx);
            self.utf32_items.swap_remove(idx);
            self.char_masks.swap_remove(idx);
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    /// Free the internal data. After calling this, the index is empty.
    pub fn destroy(&mut self) {
        self.items = Vec::new();
        self.utf32_items = Vec::new();
        self.char_masks = Vec::new();
        self.bigram_index.clear();
        self.invalidate_cache();
    }

    /// Serialize the index to a compact binary format (Uint8Array).
    pub fn serialize(&self) -> Vec<u8> {
        self.serialize_impl()
    }

    /// Reconstruct a FuzzyIndex from a previously serialized Uint8Array.
    pub fn deserialize(data: &[u8]) -> Result<FuzzyIndex, JsValue> {
        Self::deserialize_impl(data).map_err(|e| JsValue::from_str(&e))
    }

    fn serialize_impl(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(SERIALIZE_MAGIC);
        buf.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
        buf.extend_from_slice(&(self.items.len() as u32).to_le_bytes());
        for item in &self.items {
            buf.extend_from_slice(&(item.len() as u32).to_le_bytes());
            buf.extend_from_slice(item.as_bytes());
        }
        buf
    }

    fn deserialize_impl(bytes: &[u8]) -> Result<Self, String> {
        let header_size = SERIALIZE_MAGIC.len() + 4 + 4;
        if bytes.len() < header_size {
            return Err("Invalid data: too short".into());
        }
        if &bytes[0..4] != SERIALIZE_MAGIC {
            return Err("Invalid data: bad magic bytes".into());
        }
        let version = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .map_err(|_| "Invalid data: truncated header".to_string())?,
        );
        if version != SERIALIZE_VERSION {
            return Err(format!(
                "Unsupported format version: expected {SERIALIZE_VERSION}, got {version}"
            ));
        }
        let count = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| "Invalid data: truncated header".to_string())?,
        ) as usize;
        let mut offset = header_size;
        let max_possible = bytes.len().saturating_sub(header_size) / 4;
        if count > max_possible {
            return Err("Invalid data: item count exceeds payload size".into());
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            if offset + 4 > bytes.len() {
                return Err("Invalid data: truncated".into());
            }
            let len = u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .map_err(|_| "Invalid data: truncated".to_string())?,
            ) as usize;
            offset += 4;
            if offset + len > bytes.len() {
                return Err("Invalid data: truncated".into());
            }
            let s = std::str::from_utf8(&bytes[offset..offset + len])
                .map_err(|e| format!("Invalid UTF-8: {e}"))?;
            items.push(s.to_owned());
            offset += len;
        }
        if offset != bytes.len() {
            return Err("Invalid data: trailing bytes".into());
        }
        Ok(Self::new(items))
    }

    fn search_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<SearchResult> {
        // Determine incremental search cache candidates.
        let cache_candidates: Option<Vec<u32>> = {
            let last_q = self.last_query.borrow();
            let last_indices = self.last_matching_indices.borrow();
            if !query.is_empty()
                && !last_q.is_empty()
                && query.starts_with(last_q.as_str())
                && !last_indices.is_empty()
                && !last_q.contains('!')
                && !query.contains('!')
            {
                Some(last_indices.clone())
            } else {
                None
            }
        };

        // Apply bigram pre-filtering when the dataset is large enough.
        let bigram_candidates: Option<Vec<u32>> =
            if self.items.len() >= BIGRAM_THRESHOLD && cache_candidates.is_none() {
                let qbigrams = extract_query_bigrams(query);
                if qbigrams.is_empty() {
                    None
                } else {
                    self.bigram_index.candidates(&qbigrams)
                }
            } else {
                None
            };

        let candidate_indices: Option<&[u32]> =
            cache_candidates.as_deref().or(bigram_candidates.as_deref());

        let ctx = PrecomputedSearch {
            items: &self.items,
            utf32_items: &self.utf32_items,
            char_masks: &self.char_masks,
            candidate_indices,
            matcher: &self.matcher,
        };

        let outcome = search_over_precomputed(
            query,
            &ctx,
            max_results,
            min_score,
            include_positions,
            case_matching,
        );

        *self.last_query.borrow_mut() = query.to_owned();
        *self.last_matching_indices.borrow_mut() = outcome.all_matching_indices;

        outcome
            .results
            .into_iter()
            .map(SearchResult::from)
            .collect()
    }

    fn search_indices_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<IndexSearchResult> {
        let bigram_candidates: Option<Vec<u32>> = if self.items.len() >= BIGRAM_THRESHOLD {
            let qbigrams = extract_query_bigrams(query);
            if qbigrams.is_empty() {
                None
            } else {
                self.bigram_index.candidates(&qbigrams)
            }
        } else {
            None
        };

        let ctx = PrecomputedSearch {
            items: &self.items,
            utf32_items: &self.utf32_items,
            char_masks: &self.char_masks,
            candidate_indices: bigram_candidates.as_deref(),
            matcher: &self.matcher,
        };

        let outcome = search_over_precomputed_indices(
            query,
            &ctx,
            max_results,
            min_score,
            include_positions,
            case_matching,
        );

        *self.last_query.borrow_mut() = query.to_owned();
        *self.last_matching_indices.borrow_mut() = outcome.all_matching_indices;

        outcome
            .results
            .into_iter()
            .map(IndexSearchResult::from)
            .collect()
    }

    fn invalidate_cache(&self) {
        self.last_query.borrow_mut().clear();
        self.last_matching_indices.borrow_mut().clear();
    }
}
