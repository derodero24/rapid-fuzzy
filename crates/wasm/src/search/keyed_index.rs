use std::cell::RefCell;

use nucleo_matcher::pattern::{Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};
use wasm_bindgen::prelude::*;

use super::{
    SearchOptions, compute_char_mask, compute_max_score, compute_query_mask, resolve_case_matching,
    to_js,
};

use super::keys::KeySearchResult;

const SERIALIZE_MAGIC: &[u8; 4] = b"RFKI";
const SERIALIZE_VERSION: u32 = 1;

fn to_utf32(texts: &[String]) -> Vec<Utf32String> {
    texts
        .iter()
        .map(|s| Utf32String::from(s.as_str()))
        .collect()
}

/// A persistent multi-key fuzzy search index backed by Rust-side data.
///
/// Holds key text arrays and weights in memory on the Rust side.
#[wasm_bindgen]
pub struct KeyedFuzzyIndex {
    key_texts: Vec<Vec<String>>,
    utf32_keys: Vec<Vec<Utf32String>>,
    key_char_masks: Vec<Vec<u64>>,
    weights: Vec<f64>,
    total_weight: f64,
    matcher: RefCell<Matcher>,
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
        Self::new_impl(key_texts, weights).map_err(|e| JsValue::from_str(&e))
    }

    fn new_impl(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> Result<Self, String> {
        let num_keys = key_texts.len();
        if let Some(num_items) = key_texts.first().map(Vec::len) {
            for (k, col) in key_texts.iter().enumerate().skip(1) {
                if col.len() != num_items {
                    return Err(format!(
                        "All key_texts columns must have the same length; key 0 has {}, key {} has {}",
                        num_items,
                        k,
                        col.len()
                    ));
                }
            }
        }
        if weights.len() != num_keys {
            return Err(format!(
                "Expected {} weights, got {}",
                num_keys,
                weights.len()
            ));
        }
        if weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
            return Err("Weights must be finite non-negative numbers".to_string());
        }
        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            return Err("Total weight must be greater than zero".to_string());
        }
        let utf32_keys: Vec<Vec<Utf32String>> = key_texts.iter().map(|t| to_utf32(t)).collect();
        let key_char_masks: Vec<Vec<u64>> = key_texts
            .iter()
            .map(|texts| texts.iter().map(|s| compute_char_mask(s)).collect())
            .collect();
        Ok(Self {
            key_texts,
            utf32_keys,
            key_char_masks,
            weights,
            total_weight,
            matcher: RefCell::new(Matcher::new(Config::DEFAULT)),
        })
    }

    /// Return the number of items in the index.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.key_texts.first().map_or(0, |v| v.len() as u32)
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined weighted score as a JS Array.
    pub fn search(&self, query: String, options: Option<SearchOptions>) -> JsValue {
        let results = self.search_impl(query, options);
        to_js(&results)
    }

    /// Find the index of the closest matching item.
    pub fn closest(&self, query: String, min_score: Option<f64>) -> Option<u32> {
        let results = self.search_impl(
            query,
            Some(SearchOptions {
                max_results: Some(1),
                min_score,
                ..Default::default()
            }),
        );
        results.into_iter().next().map(|r| r.index)
    }

    /// Add a single item to the index.
    ///
    /// `key_values` must be a JS Array of strings with one value per key.
    pub fn add(&mut self, key_values: JsValue) -> Result<(), JsValue> {
        let key_values: Vec<String> = serde_wasm_bindgen::from_value(key_values)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.add_impl(key_values).map_err(|e| JsValue::from_str(&e))
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
            self.add_impl(key_values)
                .map_err(|e| JsValue::from_str(&e))?;
        }
        Ok(())
    }

    fn add_impl(&mut self, key_values: Vec<String>) -> Result<(), String> {
        let num_keys = self.key_texts.len();
        if key_values.len() != num_keys {
            return Err(format!(
                "Expected {num_keys} key values, got {}",
                key_values.len()
            ));
        }
        for (k, value) in key_values.into_iter().enumerate() {
            self.utf32_keys[k].push(Utf32String::from(value.as_str()));
            self.key_char_masks[k].push(compute_char_mask(&value));
            self.key_texts[k].push(value);
        }
        Ok(())
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    pub fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        let num_items = self.size() as usize;
        if idx >= num_items {
            return false;
        }
        for ((texts, utf32), masks) in self
            .key_texts
            .iter_mut()
            .zip(self.utf32_keys.iter_mut())
            .zip(self.key_char_masks.iter_mut())
        {
            texts.swap_remove(idx);
            utf32.swap_remove(idx);
            masks.swap_remove(idx);
        }
        true
    }

    /// Free the internal data. After calling this, the index is empty.
    pub fn destroy(&mut self) {
        self.key_texts = Vec::new();
        self.utf32_keys = Vec::new();
        self.key_char_masks = Vec::new();
        self.weights = Vec::new();
        self.total_weight = 0.0;
    }

    /// Serialize the index to a compact binary format (Uint8Array).
    pub fn serialize(&self) -> Vec<u8> {
        let num_keys = self.weights.len();
        let num_items = self.size() as usize;
        let mut buf = Vec::new();
        buf.extend_from_slice(SERIALIZE_MAGIC);
        buf.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
        buf.extend_from_slice(&(num_keys as u32).to_le_bytes());
        buf.extend_from_slice(&(num_items as u32).to_le_bytes());
        for &w in &self.weights {
            buf.extend_from_slice(&w.to_le_bytes());
        }
        for key_col in &self.key_texts {
            for item in key_col {
                buf.extend_from_slice(&(item.len() as u32).to_le_bytes());
                buf.extend_from_slice(item.as_bytes());
            }
        }
        buf
    }

    /// Reconstruct a KeyedFuzzyIndex from a previously serialized Uint8Array.
    pub fn deserialize(data: &[u8]) -> Result<KeyedFuzzyIndex, JsValue> {
        Self::deserialize_impl(data).map_err(|e| JsValue::from_str(&e))
    }

    fn deserialize_impl(bytes: &[u8]) -> Result<Self, String> {
        let header_size = 4 + 4 + 4 + 4;
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
        let num_keys = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| "Invalid data: truncated header".to_string())?,
        ) as usize;
        let num_items = u32::from_le_bytes(
            bytes[12..16]
                .try_into()
                .map_err(|_| "Invalid data: truncated header".to_string())?,
        ) as usize;
        let mut offset = header_size;
        let weights_size = num_keys * 8;
        if offset + weights_size > bytes.len() {
            return Err("Invalid data: truncated weights".into());
        }
        let mut weights = Vec::with_capacity(num_keys);
        for _ in 0..num_keys {
            let w = f64::from_le_bytes(
                bytes[offset..offset + 8]
                    .try_into()
                    .map_err(|_| "Invalid data: truncated weight".to_string())?,
            );
            weights.push(w);
            offset += 8;
        }
        let mut key_texts: Vec<Vec<String>> = Vec::with_capacity(num_keys);
        for _ in 0..num_keys {
            let mut col = Vec::with_capacity(num_items);
            for _ in 0..num_items {
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
                col.push(s.to_owned());
                offset += len;
            }
            key_texts.push(col);
        }
        if offset != bytes.len() {
            return Err("Invalid data: trailing bytes".into());
        }
        Self::new_impl(key_texts, weights)
    }

    fn search_impl(&self, query: String, options: Option<SearchOptions>) -> Vec<KeySearchResult> {
        let opts = options.unwrap_or_default();
        let num_keys = self.key_texts.len();
        if num_keys == 0 {
            return Vec::new();
        }
        let num_items = self.size() as usize;
        if num_items == 0 {
            return Vec::new();
        }
        let return_all_on_empty = opts.return_all_on_empty.unwrap_or(false);
        if return_all_on_empty && query.trim().is_empty() {
            let limit = opts.max_results.unwrap_or(num_items as u32) as usize;
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
        if self.weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
            return Vec::new();
        }
        let mut matcher = self.matcher.borrow_mut();
        let pattern = Pattern::parse(&query, case_matching, Normalization::Smart);
        let max_score = compute_max_score(&query, &pattern, &mut matcher);
        let query_mask = compute_query_mask(&query);
        let active_keys: Vec<usize> = (0..num_keys).filter(|&k| self.weights[k] > 0.0).collect();
        let active_total_weight: f64 = active_keys.iter().map(|&k| self.weights[k]).sum();
        let mut per_key_scores: Vec<Vec<f64>> = vec![vec![0.0; num_items]; num_keys];
        let threshold_weighted = threshold * self.total_weight;
        let mut scored: Vec<(u32, f64)> = (0..num_items)
            .filter_map(|i| {
                let mut weighted_sum = 0.0;
                let mut matched_any = false;
                let mut remaining_weight = active_total_weight;
                for &k in &active_keys {
                    let w = self.weights[k];
                    remaining_weight -= w;
                    if query_mask != 0 && (self.key_char_masks[k][i] & query_mask) != query_mask {
                        if threshold > 0.0 && weighted_sum + remaining_weight < threshold_weighted {
                            return None;
                        }
                        continue;
                    }
                    let atoms = self.utf32_keys[k][i].slice(..);
                    let score = match pattern.score(atoms, &mut matcher) {
                        Some(raw) => ((raw as f64) / max_score).min(1.0),
                        None => 0.0,
                    };
                    per_key_scores[k][i] = score;
                    if score > 0.0 {
                        weighted_sum += score * w;
                        matched_any = true;
                    }
                    if threshold > 0.0 && weighted_sum + remaining_weight < threshold_weighted {
                        return None;
                    }
                }
                if !matched_any {
                    return None;
                }
                let combined = weighted_sum / self.total_weight;
                if combined >= threshold {
                    Some((i as u32, combined))
                } else {
                    None
                }
            })
            .collect();
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
}
