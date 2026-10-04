use std::cell::RefCell;

use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher, Utf32String};

use super::keys::{IndexedKeys, KeyedSearchParams, keyed_search_core, validate_keyed_input};
use super::{KeySearchResult, compute_char_mask};

fn to_utf32(texts: &[String]) -> Vec<Utf32String> {
    texts
        .iter()
        .map(|s| Utf32String::from(s.as_str()))
        .collect()
}

/// Core state and logic for a persistent multi-key fuzzy search index.
///
/// This struct contains all platform-independent state and methods.
/// Binding crates (napi, wasm) wrap this with their own FFI layer.
pub struct KeyedFuzzyIndexCore {
    key_texts: Vec<Vec<String>>,
    utf32_keys: Vec<Vec<Utf32String>>,
    key_char_masks: Vec<Vec<u64>>,
    weights: Vec<f64>,
    total_weight: f64,
    matcher: RefCell<Matcher>,
}

impl KeyedFuzzyIndexCore {
    /// Create a new KeyedFuzzyIndexCore.
    ///
    /// `key_texts[k]` is an array of strings for key `k`, one per item.
    /// All inner arrays must have the same length (the number of items).
    pub fn new(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> Result<Self, String> {
        let total_weight = validate_keyed_input(&key_texts, &weights)?;

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
    pub fn size(&self) -> u32 {
        self.key_texts.first().map_or(0, |v| v.len() as u32)
    }

    /// Access the key_texts.
    pub fn key_texts(&self) -> &[Vec<String>] {
        &self.key_texts
    }

    /// Access the weights.
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    /// Search the index for items matching the query.
    ///
    /// Returns results sorted by combined weighted score (best match first),
    /// exactly like [`search_keys_impl`](super::search_keys_impl) on the same
    /// key texts and weights (see the `keys` module for the semantics).
    pub fn search(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        case_matching: CaseMatching,
        return_all_on_empty: bool,
    ) -> Vec<KeySearchResult> {
        keyed_search_core(
            query,
            &IndexedKeys {
                key_texts: &self.key_texts,
                haystacks: &self.utf32_keys,
                char_masks: &self.key_char_masks,
            },
            &self.weights,
            self.total_weight,
            KeyedSearchParams {
                max_results,
                min_score,
                case_matching,
                return_all_on_empty,
            },
            &mut self.matcher.borrow_mut(),
        )
    }

    /// Add a single item to the index.
    ///
    /// `key_values` must hold one value per key; otherwise an error is
    /// returned and the index is left unchanged.
    pub fn add(&mut self, key_values: Vec<String>) -> Result<(), String> {
        let num_keys = self.key_texts.len();
        if key_values.len() != num_keys {
            return Err(format!(
                "Expected {num_keys} key values, got {}",
                key_values.len()
            ));
        }
        self.push_row(key_values);
        Ok(())
    }

    /// Add multiple items to the index at once.
    ///
    /// Every row is validated before any is added: if one row does not hold
    /// exactly one value per key, an error naming it is returned and the
    /// index is left unchanged.
    pub fn add_many(&mut self, items_key_values: Vec<Vec<String>>) -> Result<(), String> {
        let num_keys = self.key_texts.len();
        if let Some((i, row)) = items_key_values
            .iter()
            .enumerate()
            .find(|(_, row)| row.len() != num_keys)
        {
            return Err(format!(
                "Expected {num_keys} key values for item {i}, got {}",
                row.len()
            ));
        }
        let additional = items_key_values.len();
        for ((texts, utf32), masks) in self
            .key_texts
            .iter_mut()
            .zip(self.utf32_keys.iter_mut())
            .zip(self.key_char_masks.iter_mut())
        {
            texts.reserve(additional);
            utf32.reserve(additional);
            masks.reserve(additional);
        }
        for key_values in items_key_values {
            self.push_row(key_values);
        }
        Ok(())
    }

    /// Append one row whose length has already been checked against the key count.
    fn push_row(&mut self, key_values: Vec<String>) {
        for (k, value) in key_values.into_iter().enumerate() {
            self.utf32_keys[k].push(Utf32String::from(value.as_str()));
            self.key_char_masks[k].push(compute_char_mask(&value));
            self.key_texts[k].push(value);
        }
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

    /// Free the item data. After calling this, the index is empty.
    ///
    /// The key configuration (number of keys and their weights) is kept, so
    /// the index stays usable: items can be added again and it serializes
    /// as a valid empty index.
    pub fn destroy(&mut self) {
        for ((texts, utf32), masks) in self
            .key_texts
            .iter_mut()
            .zip(self.utf32_keys.iter_mut())
            .zip(self.key_char_masks.iter_mut())
        {
            *texts = Vec::new();
            *utf32 = Vec::new();
            *masks = Vec::new();
        }
    }
}
