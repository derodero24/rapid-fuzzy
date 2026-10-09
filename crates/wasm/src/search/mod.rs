mod index;
mod keyed_index;
mod keys;

pub use index::FuzzyIndex;
pub use keyed_index::KeyedFuzzyIndex;
pub use keys::search_keys;

use std::fmt;

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search as core;
use serde::de::{self, DeserializeOwned, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::convert::{from_js, or_null, to_js, type_error};

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
    /// Indices of the matched characters of the item, in ascending order.
    /// An ASCII item is counted by character; any other item by grapheme
    /// cluster (a user-perceived character, such as an emoji with its
    /// modifiers or a letter with its combining marks), so these are not
    /// UTF-16 string offsets. `highlight()` and `highlightRanges()` convert
    /// them. Empty unless `includePositions` is set to true in SearchOptions.
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
    /// Indices of the matched characters of the item, in ascending order.
    /// An ASCII item is counted by character; any other item by grapheme
    /// cluster (a user-perceived character, such as an emoji with its
    /// modifiers or a letter with its combining marks), so these are not
    /// UTF-16 string offsets. `highlight()` and `highlightRanges()` convert
    /// them. Empty unless `includePositions` is set to true in SearchOptions.
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
    /// Maximum number of results to return: a non-negative integer, or
    /// `Infinity` for no limit. NaN, negative and fractional values throw.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_max_results")]
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
    /// Read the `options` argument of `search` and `FuzzyIndex`'s `search` /
    /// `searchIndices`: `undefined`/`null`, a number (shorthand for
    /// `{ maxResults }`, as in the Node.js binding) or a `SearchOptions`
    /// object.
    pub(crate) fn from_js_or_max_results(options: Option<JsValue>) -> Result<Self, JsValue> {
        options_from_js(options, "SearchOptions", |max_results| Self {
            max_results,
            ..Self::default()
        })
    }
}

/// Read an `options` argument: `undefined`/`null` (the defaults), a number
/// (shorthand for `{ maxResults }`, built by `with_max_results`) or an
/// options object `T` named `what` in errors.
fn options_from_js<T: DeserializeOwned + Default>(
    options: Option<JsValue>,
    what: &str,
    with_max_results: impl FnOnce(Option<u32>) -> T,
) -> Result<T, JsValue> {
    match options {
        None => Ok(T::default()),
        Some(value) => match value.as_f64() {
            Some(number) => Ok(with_max_results(
                core::check_max_results(number).map_err(|e| type_error(&e))?,
            )),
            None => from_js(value, what),
        },
    }
}

/// How multi-key search combines the per-key scores (`keyScores`) of an item
/// into its `score` (see `KeySearchOptions.scoreMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum KeyScoreMode {
    /// The weighted mean over all keys (the default).
    Weighted,
    /// The weighted mean over the keys that match.
    Matched,
    /// The highest score of any key with a positive weight.
    Max,
}

impl From<core::KeyScoreMode> for KeyScoreMode {
    fn from(mode: core::KeyScoreMode) -> Self {
        match mode {
            core::KeyScoreMode::Weighted => Self::Weighted,
            core::KeyScoreMode::Matched => Self::Matched,
            core::KeyScoreMode::Max => Self::Max,
        }
    }
}

impl From<KeyScoreMode> for core::KeyScoreMode {
    fn from(mode: KeyScoreMode) -> Self {
        match mode {
            KeyScoreMode::Weighted => Self::Weighted,
            KeyScoreMode::Matched => Self::Matched,
            KeyScoreMode::Max => Self::Max,
        }
    }
}

impl KeyScoreMode {
    /// Read a `scoreMode` argument, rejecting anything but `"weighted"`,
    /// `"matched"` and `"max"` with a `TypeError` (the message of the Node.js
    /// binding).
    pub(crate) fn from_js(value: &JsValue) -> Result<Self, JsValue> {
        let mode = match value.as_string() {
            Some(name) => core::KeyScoreMode::from_name(&name),
            None => Err(core::invalid_score_mode(
                &value.js_typeof().as_string().unwrap_or_default(),
            )),
        };
        mode.map(Self::from).map_err(|e| type_error(&e))
    }
}

impl<'de> Deserialize<'de> for KeyScoreMode {
    /// Parsed by [`core::KeyScoreMode::from_name`], so that an unknown mode
    /// is reported with the message of the Node.js binding.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ScoreModeVisitor;

        impl<'de> Visitor<'de> for ScoreModeVisitor {
            type Value = KeyScoreMode;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(r#""weighted", "matched" or "max""#)
            }

            fn visit_str<E: de::Error>(self, name: &str) -> Result<KeyScoreMode, E> {
                core::KeyScoreMode::from_name(name)
                    .map(KeyScoreMode::from)
                    .map_err(E::custom)
            }

            fn visit_bool<E: de::Error>(self, _: bool) -> Result<KeyScoreMode, E> {
                Err(E::custom(core::invalid_score_mode("boolean")))
            }

            fn visit_i64<E: de::Error>(self, _: i64) -> Result<KeyScoreMode, E> {
                Err(E::custom(core::invalid_score_mode("number")))
            }

            fn visit_u64<E: de::Error>(self, _: u64) -> Result<KeyScoreMode, E> {
                Err(E::custom(core::invalid_score_mode("number")))
            }

            fn visit_f64<E: de::Error>(self, _: f64) -> Result<KeyScoreMode, E> {
                Err(E::custom(core::invalid_score_mode("number")))
            }

            fn visit_map<A: de::MapAccess<'de>>(self, _: A) -> Result<KeyScoreMode, A::Error> {
                Err(de::Error::custom(core::invalid_score_mode("object")))
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, _: A) -> Result<KeyScoreMode, A::Error> {
                Err(de::Error::custom(core::invalid_score_mode("object")))
            }
        }

        // deserialize_any: serde-wasm-bindgen then calls the visitor method of
        // the actual JS type, which reports it like the Node.js binding.
        deserializer.deserialize_any(ScoreModeVisitor)
    }
}

/// Options for multi-key search: `searchKeys()`, `KeyedFuzzyIndex.search()`
/// and the object search built on them (`searchObjects()`,
/// `FuzzyObjectIndex.search()`). `includePositions` has no effect there:
/// multi-key results have no match positions.
#[derive(Debug, Clone, Default, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct KeySearchOptions {
    #[serde(flatten)]
    pub search: SearchOptions,
    /// How the per-key scores (`keyScores`) of an item are combined into its
    /// `score`. Only keys with a positive weight take part:
    ///
    /// - `"weighted"` (default): the weighted mean over all keys,
    ///   `sum(weight * keyScore) / sum(weight)`. A key that does not match
    ///   counts as 0, so an exact match on one key out of several scores only
    ///   that key's share of the total weight.
    /// - `"matched"`: the weighted mean over the keys that match
    ///   (`keyScore > 0`) only. An item whose only matching key matches
    ///   exactly scores 1.
    /// - `"max"`: the highest score of any key. Weights then only select the
    ///   keys that take part (weight > 0).
    ///
    /// `keyScores` are the same in every mode; `minScore` and `maxResults`
    /// apply to the combined score. Equal scores are ordered by the length of
    /// the best-matching key's text (the key contributing most to the score:
    /// highest `weight * keyScore`, or highest `keyScore` in `"max"` mode;
    /// the first one on a tie), then by index. Any other value throws a
    /// `TypeError`.
    #[tsify(optional)]
    #[serde(default)]
    pub score_mode: Option<KeyScoreMode>,
}

impl KeySearchOptions {
    /// Read the `options` argument of `searchKeys` and
    /// `KeyedFuzzyIndex.search`: `undefined`/`null`, a number (shorthand for
    /// `{ maxResults }`) or a `KeySearchOptions` object.
    pub(crate) fn from_js_or_max_results(options: Option<JsValue>) -> Result<Self, JsValue> {
        options_from_js(options, "KeySearchOptions", |max_results| Self {
            search: SearchOptions {
                max_results,
                ..SearchOptions::default()
            },
            score_mode: None,
        })
    }

    /// The options of the shared core-lib search.
    pub(crate) fn to_core(&self) -> core::SearchKeysOptions {
        core::SearchKeysOptions {
            max_results: self.search.max_results,
            min_score: self.search.min_score,
            is_case_sensitive: self.search.is_case_sensitive,
            return_all_on_empty: self.search.return_all_on_empty,
            score_mode: self.score_mode.map(Into::into),
        }
    }
}

/// Read `SearchOptions.maxResults` like the Node.js binding does (see
/// [`core::check_max_results`]): `Infinity` means no limit, and NaN,
/// negative or fractional values are rejected.
fn deserialize_max_results<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    match Option::<f64>::deserialize(deserializer)? {
        Some(value) => core::check_max_results(value).map_err(serde::de::Error::custom),
        None => Ok(None),
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
