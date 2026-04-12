use std::cell::RefCell;

use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher, Utf32String};

use super::{
    BigramIndex, IndexSearchResult, PrecomputedSearch, SearchResult, compute_char_mask,
    extract_query_bigrams, intersect_sorted, search_over_precomputed,
    search_over_precomputed_indices,
};

/// Minimum dataset size for bigram pre-filtering at query time.
/// Below this threshold, the overhead of bigram intersection exceeds the
/// benefit of reducing pattern.score() calls.
const BIGRAM_THRESHOLD: usize = 5_000;

/// Core state and logic for a persistent fuzzy search index.
///
/// This struct contains all platform-independent state and methods.
/// Binding crates (napi, wasm) wrap this with their own FFI layer.
pub struct FuzzyIndexCore {
    items: Vec<String>,
    utf32_items: Vec<Utf32String>,
    char_masks: Vec<u64>,
    bigram_index: BigramIndex,
    matcher: RefCell<Matcher>,
    /// Incremental search cache: the query from the last search.
    last_query: RefCell<String>,
    /// Incremental search cache: indices of all items that matched the last query.
    last_matching_indices: RefCell<Vec<u32>>,
}

impl FuzzyIndexCore {
    /// Create a new FuzzyIndexCore from a list of items.
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
    pub fn size(&self) -> u32 {
        self.items.len() as u32
    }

    /// Access the items slice.
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Search the index for items matching the query.
    pub fn search_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<SearchResult> {
        // Determine if we can narrow the search to cached matching indices.
        // Conditions: new query is a prefix extension of the cached query,
        // the cache has a non-empty candidate set, and neither query uses
        // inverted terms (which break monotonicity).
        let cache_candidates: Option<Vec<u32>> = {
            let last_q = self.last_query.borrow();
            let last_idx = self.last_matching_indices.borrow();
            if !last_q.is_empty()
                && !last_idx.is_empty()
                && query.len() > last_q.len()
                && query.starts_with(last_q.as_str())
                && !query.contains('!')
                && !last_q.contains('!')
            {
                Some(last_idx.clone())
            } else {
                None
            }
        };

        // Get bigram candidates (only for large datasets).
        let bigram_candidates = if self.items.len() >= BIGRAM_THRESHOLD {
            let query_bigrams = extract_query_bigrams(query);
            self.bigram_index.candidates(&query_bigrams)
        } else {
            None
        };

        // Compose candidate lists: intersect cache and bigram when both available.
        let candidates: Option<Vec<u32>> = match (cache_candidates, bigram_candidates) {
            (Some(cache), Some(bigram)) => Some(intersect_sorted(&cache, &bigram)),
            (Some(cache), None) => Some(cache),
            (None, Some(bigram)) => Some(bigram),
            (None, None) => None,
        };

        let ctx = PrecomputedSearch {
            items: &self.items,
            utf32_items: &self.utf32_items,
            char_masks: &self.char_masks,
            candidate_indices: candidates.as_deref(),
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

        // Update incremental cache.
        *self.last_query.borrow_mut() = query.to_owned();
        *self.last_matching_indices.borrow_mut() = outcome.all_matching_indices;

        outcome.results
    }

    /// Search the index, returning only indices and scores (no item strings).
    pub fn search_indices_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<IndexSearchResult> {
        let cache_candidates: Option<Vec<u32>> = {
            let last_q = self.last_query.borrow();
            let last_idx = self.last_matching_indices.borrow();
            if !last_q.is_empty()
                && !last_idx.is_empty()
                && query.len() > last_q.len()
                && query.starts_with(last_q.as_str())
                && !query.contains('!')
                && !last_q.contains('!')
            {
                Some(last_idx.clone())
            } else {
                None
            }
        };

        let bigram_candidates = if self.items.len() >= BIGRAM_THRESHOLD {
            let query_bigrams = extract_query_bigrams(query);
            self.bigram_index.candidates(&query_bigrams)
        } else {
            None
        };

        let candidates: Option<Vec<u32>> = match (cache_candidates, bigram_candidates) {
            (Some(cache), Some(bigram)) => Some(intersect_sorted(&cache, &bigram)),
            (Some(cache), None) => Some(cache),
            (None, Some(bigram)) => Some(bigram),
            (None, None) => None,
        };

        let ctx = PrecomputedSearch {
            items: &self.items,
            utf32_items: &self.utf32_items,
            char_masks: &self.char_masks,
            candidate_indices: candidates.as_deref(),
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

        // Update incremental cache.
        *self.last_query.borrow_mut() = query.to_owned();
        *self.last_matching_indices.borrow_mut() = outcome.all_matching_indices;

        outcome.results
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

    fn invalidate_cache(&self) {
        self.last_query.borrow_mut().clear();
        self.last_matching_indices.borrow_mut().clear();
    }
}
