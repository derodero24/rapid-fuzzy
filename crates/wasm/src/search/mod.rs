mod index;
mod keyed_index;
mod keys;

pub use index::FuzzyIndex;
pub use keyed_index::KeyedFuzzyIndex;
pub use keys::search_keys;

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search as core;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

// ─── Shared wasm types ──────────────────────────────────────────────────────

/// Classification of how a query matched an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

impl From<core::MatchType> for MatchType {
    fn from(mt: core::MatchType) -> Self {
        match mt {
            core::MatchType::Exact => Self::Exact,
            core::MatchType::Prefix => Self::Prefix,
            core::MatchType::Contains => Self::Contains,
            core::MatchType::Fuzzy => Self::Fuzzy,
        }
    }
}

/// A single fuzzy search result with the matched item and its score.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub item: String,
    pub score: f64,
    pub index: u32,
    pub positions: Vec<u32>,
    pub match_type: Option<MatchType>,
}

impl From<core::SearchResult> for SearchResult {
    fn from(r: core::SearchResult) -> Self {
        Self {
            item: r.item,
            score: r.score,
            index: r.index,
            positions: r.positions,
            match_type: r.match_type.map(Into::into),
        }
    }
}

/// A lightweight search result containing only index and score.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexSearchResult {
    pub index: u32,
    pub score: f64,
    pub positions: Vec<u32>,
    pub match_type: Option<MatchType>,
}

impl From<core::IndexSearchResult> for IndexSearchResult {
    fn from(r: core::IndexSearchResult) -> Self {
        Self {
            index: r.index,
            score: r.score,
            positions: r.positions,
            match_type: r.match_type.map(Into::into),
        }
    }
}

/// Options for search functions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(from_wasm_abi)]
pub struct SearchOptions {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub include_positions: Option<bool>,
    pub is_case_sensitive: Option<bool>,
    pub return_all_on_empty: Option<bool>,
}

pub(crate) fn to_js<T: Serialize>(value: &T) -> JsValue {
    serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL)
}

pub(crate) use rapid_fuzzy_core::search::{
    BigramIndex, PrecomputedSearch, compute_char_mask, compute_max_score, compute_query_mask,
    extract_query_bigrams, resolve_case_matching, search_over_precomputed,
    search_over_precomputed_indices,
};

// ─── Standalone search functions ────────────────────────────────────────────

pub(crate) fn search_impl(
    query: String,
    items: Vec<String>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    core::search_impl(
        query,
        items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    )
    .into_iter()
    .map(SearchResult::from)
    .collect()
}

/// Perform fuzzy search over a list of strings.
///
/// Returns matches sorted by score (best match first).
/// Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
#[wasm_bindgen]
pub fn search(query: String, items: Vec<String>, options: Option<SearchOptions>) -> JsValue {
    let opts = options.unwrap_or_default();
    let (max_results, min_score, include_positions, case_matching, return_all_on_empty) = (
        opts.max_results,
        opts.min_score,
        opts.include_positions.unwrap_or(false),
        resolve_case_matching(opts.is_case_sensitive),
        opts.return_all_on_empty.unwrap_or(false),
    );

    if return_all_on_empty && query.trim().is_empty() {
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

    let results = search_impl(
        query,
        items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    );
    to_js(&results)
}

/// Find the closest matching string from a list.
///
/// Returns the best match, or null if no match is found.
#[wasm_bindgen]
pub fn closest(query: String, items: Vec<String>, min_score: Option<f64>) -> Option<String> {
    let results = search_impl(query, items, Some(1), min_score, false, CaseMatching::Smart);
    results.into_iter().next().map(|r| r.item)
}
