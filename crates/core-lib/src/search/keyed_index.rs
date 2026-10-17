use nucleo_matcher::pattern::CaseMatching;

use super::keys::{
    IndexedKeys, KeyMatchMode, KeyScoreMode, KeyedSearchParams, SearchKeysOptions,
    keyed_search_core, validate_keyed_input,
};
use super::{KeySearchResult, compute_char_mask, index_haystack, with_matcher};

/// The haystack of a key text stored in an index (see [`index_haystack`]).
type Haystack = Option<Box<[char]>>;

fn to_haystacks(texts: &[String]) -> Vec<Haystack> {
    texts.iter().map(|s| index_haystack(s)).collect()
}

/// Core state and logic for a persistent multi-key fuzzy search index.
///
/// This struct contains all platform-independent state and methods.
/// Binding crates (napi, wasm) wrap this with their own FFI layer.
///
/// Like `FuzzyIndexCore`, the index owns no nucleo matcher (searches use the
/// shared per-thread one, see [`with_matcher`]), so
/// [`destroy`](Self::destroy) frees all of its item data.
pub struct KeyedFuzzyIndexCore {
    key_texts: Vec<Vec<String>>,
    /// Haystack of every non-ASCII key text; `None` for ASCII text, which is
    /// matched as its own bytes instead of being stored twice (like
    /// `FuzzyIndexCore` stores its items).
    haystacks: Vec<Vec<Haystack>>,
    key_char_masks: Vec<Vec<u64>>,
    weights: Vec<f64>,
    total_weight: f64,
}

impl KeyedFuzzyIndexCore {
    /// Create a new KeyedFuzzyIndexCore.
    ///
    /// `key_texts[k]` is an array of strings for key `k`, one per item.
    /// All inner arrays must have the same length (the number of items).
    pub fn new(key_texts: Vec<Vec<String>>, weights: Vec<f64>) -> Result<Self, String> {
        let total_weight = validate_keyed_input(&key_texts, &weights)?;

        let haystacks: Vec<Vec<Haystack>> = key_texts.iter().map(|t| to_haystacks(t)).collect();
        let key_char_masks: Vec<Vec<u64>> = key_texts
            .iter()
            .map(|texts| texts.iter().map(|s| compute_char_mask(s)).collect())
            .collect();
        Ok(Self {
            key_texts,
            haystacks,
            key_char_masks,
            weights,
            total_weight,
        })
    }

    /// An index with no keys and no items.
    ///
    /// Before `destroy()` kept the key configuration, a destroyed index
    /// serialized to a payload with zero keys. Loading such a payload gives
    /// this state back: it is empty, searches return nothing, `add` rejects
    /// any values, and it serializes to the same zero-key payload.
    pub(crate) fn without_keys() -> Self {
        Self {
            key_texts: Vec::new(),
            haystacks: Vec::new(),
            key_char_masks: Vec::new(),
            weights: Vec::new(),
            total_weight: 0.0,
        }
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

    /// Approximate number of heap bytes the index owns: its key texts, their
    /// pre-computed search data and the vectors holding them.
    ///
    /// Bindings report this to their JavaScript engine so that its garbage
    /// collector accounts for the index's native memory.
    pub fn heap_size(&self) -> usize {
        let columns = self.key_texts.capacity() * size_of::<Vec<String>>()
            + self.haystacks.capacity() * size_of::<Vec<Haystack>>()
            + self.key_char_masks.capacity() * size_of::<Vec<u64>>()
            + self.weights.capacity() * size_of::<f64>();
        let texts: usize = self
            .key_texts
            .iter()
            .map(|col| {
                col.capacity() * size_of::<String>()
                    + col.iter().map(String::capacity).sum::<usize>()
            })
            .sum();
        let haystacks: usize = self
            .haystacks
            .iter()
            .map(|col| {
                col.capacity() * size_of::<Haystack>()
                    + col
                        .iter()
                        .map(|haystack| haystack.as_deref().map_or(0, size_of_val))
                        .sum::<usize>()
            })
            .sum();
        let masks: usize = self
            .key_char_masks
            .iter()
            .map(|col| col.capacity() * size_of::<u64>())
            .sum();
        columns + texts + haystacks + masks
    }

    /// Search the index for items matching the query, matching every key
    /// against the whole query ([`KeyMatchMode::PerKey`]).
    ///
    /// Returns results sorted by combined score (best match first), exactly
    /// like [`search_keys_impl`](super::search_keys_impl) on the same key
    /// texts and weights (see the `keys` module for the semantics, and
    /// [`KeyScoreMode`] for how `score_mode` combines the key scores).
    /// [`search_with_options`](Self::search_with_options) also takes a
    /// [`KeyMatchMode`].
    pub fn search(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        case_matching: CaseMatching,
        return_all_on_empty: bool,
        score_mode: KeyScoreMode,
    ) -> Vec<KeySearchResult> {
        self.search_with_params(
            query,
            KeyedSearchParams {
                max_results,
                min_score,
                case_matching,
                return_all_on_empty,
                score_mode,
                match_mode: KeyMatchMode::PerKey,
            },
        )
    }

    /// Search the index for items matching the query.
    ///
    /// Returns exactly what [`search_keys_impl`](super::search_keys_impl)
    /// returns for the same key texts, weights and options (see the `keys`
    /// module for the semantics); unset options take the same defaults.
    pub fn search_with_options(
        &self,
        query: &str,
        options: SearchKeysOptions,
    ) -> Vec<KeySearchResult> {
        self.search_with_params(query, KeyedSearchParams::from(options))
    }

    fn search_with_params(&self, query: &str, params: KeyedSearchParams) -> Vec<KeySearchResult> {
        with_matcher(|matcher| {
            keyed_search_core(
                query,
                &IndexedKeys {
                    key_texts: &self.key_texts,
                    haystacks: &self.haystacks,
                    char_masks: &self.key_char_masks,
                },
                &self.weights,
                self.total_weight,
                params,
                matcher,
            )
        })
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
        for ((texts, haystacks), masks) in self
            .key_texts
            .iter_mut()
            .zip(self.haystacks.iter_mut())
            .zip(self.key_char_masks.iter_mut())
        {
            texts.reserve(additional);
            haystacks.reserve(additional);
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
            self.haystacks[k].push(index_haystack(&value));
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
        for ((texts, haystacks), masks) in self
            .key_texts
            .iter_mut()
            .zip(self.haystacks.iter_mut())
            .zip(self.key_char_masks.iter_mut())
        {
            texts.swap_remove(idx);
            haystacks.swap_remove(idx);
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
        for ((texts, haystacks), masks) in self
            .key_texts
            .iter_mut()
            .zip(self.haystacks.iter_mut())
            .zip(self.key_char_masks.iter_mut())
        {
            *texts = Vec::new();
            *haystacks = Vec::new();
            *masks = Vec::new();
        }
    }
}
