mod index;
mod keyed_index;
mod keys;

pub use index::FuzzyIndex;
pub use keyed_index::KeyedFuzzyIndex;
pub use keys::search_keys;

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search as core;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::convert::{from_js, or_null, to_js};

// ─── Shared wasm types ──────────────────────────────────────────────────────

/// Classification of how a query matched an item.
///
/// Derived from the matched character positions:
/// - **Exact**: all positions consecutive from index 0, covering every character in the item.
/// - **Prefix**: all positions consecutive from index 0, but the item is longer.
/// - **Contains**: all positions consecutive (a substring match), not starting at 0.
/// - **Fuzzy**: positions have gaps (character-level fuzzy match).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
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
#[derive(Debug, Clone, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// The original string that matched.
    pub item: String,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// The index of the item in the original input array.
    pub index: u32,
    /// Indices of matched characters in the item string.
    /// Empty unless `includePositions` is set to true in SearchOptions.
    pub positions: Vec<u32>,
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `includePositions` is set to true in SearchOptions.
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// A lightweight search result containing only index and score (no item string).
#[derive(Debug, Clone, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct IndexSearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// Indices of matched characters in the item string.
    /// Empty unless `includePositions` is set to true in SearchOptions.
    pub positions: Vec<u32>,
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `includePositions` is set to true in SearchOptions.
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[derive(Debug, Clone, Default, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    /// Maximum number of results to return.
    #[tsify(optional)]
    #[serde(default)]
    pub max_results: Option<u32>,
    /// Minimum normalized score (0.0-1.0) to include in results.
    #[tsify(optional)]
    #[serde(default)]
    pub min_score: Option<f64>,
    /// If true, include matched character positions in results.
    #[tsify(optional)]
    #[serde(default)]
    pub include_positions: Option<bool>,
    /// If true, matching is case-sensitive. When false or omitted, matching
    /// is smart case: case-insensitive while the query is all lower-case, and
    /// case-sensitive once it contains an upper-case letter. `false` does not
    /// force case-insensitive matching; lower-case the query for that.
    #[tsify(optional)]
    #[serde(default)]
    pub is_case_sensitive: Option<bool>,
    /// If true, return all items when the query is empty (or whitespace-only).
    /// Useful for filter-as-you-type UIs where the full list should appear
    /// before the user starts typing. Default is false.
    #[tsify(optional)]
    #[serde(default)]
    pub return_all_on_empty: Option<bool>,
}

impl SearchOptions {
    /// Read the `options` argument of `search`, `FuzzyIndex.search` and
    /// `FuzzyIndex.searchIndices`: `undefined`/`null`, a number (shorthand for
    /// `{ maxResults }`, as in the Node.js binding) or a `SearchOptions` object.
    pub(crate) fn from_js_or_max_results(options: Option<JsValue>) -> Result<Self, JsValue> {
        match options {
            None => Ok(Self::default()),
            Some(max_results) if max_results.as_f64().is_some() => Ok(Self {
                max_results: Some(from_js(max_results, "maxResults")?),
                ..Self::default()
            }),
            Some(options) => from_js(options, "SearchOptions"),
        }
    }

    /// Read an optional `SearchOptions` argument.
    pub(crate) fn from_ts(options: Option<tsify::Ts<Self>>) -> Result<Self, JsValue> {
        options.map_or_else(
            || Ok(Self::default()),
            |opts| from_js(opts.into(), "SearchOptions"),
        )
    }
}

pub(crate) use rapid_fuzzy_core::search::resolve_case_matching;

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
///
/// The third argument accepts either a number (maxResults for backward
/// compatibility) or a SearchOptions object.
#[wasm_bindgen(unchecked_return_type = "SearchResult[]")]
pub fn search(
    query: String,
    items: Vec<String>,
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

    if return_all_on_empty && core::is_empty_query(&query) {
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
/// If `minScore` is provided, returns null when the best match scores below the threshold.
#[wasm_bindgen(unchecked_return_type = "string | null")]
pub fn closest(
    query: String,
    items: Vec<String>,
    #[wasm_bindgen(js_name = "minScore")] min_score: Option<f64>,
) -> JsValue {
    let results = search_impl(query, items, Some(1), min_score, false, CaseMatching::Smart);
    or_null(results.into_iter().next().map(|r| r.item))
}
