mod index;
mod keyed_index;
mod keys;

pub use index::FuzzyIndex;
pub use keyed_index::KeyedFuzzyIndex;
pub use keys::search_keys;

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search as core;
use serde::de::{self, DeserializeOwned};
use serde::{Deserialize, Deserializer, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::convert::{
    from_js, optional_number_from_js, or_null, strings_from_js, to_js, type_error,
};

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
    /// The matched item. Strings are converted to UTF-8 on the way into
    /// Rust, so this is the same string as `items[index]` unless that one
    /// contains a lone UTF-16 surrogate (for example from slicing an emoji in
    /// half), which becomes U+FFFD; `index` always identifies your string.
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
    /// Minimum normalized score (0.0-1.0) to include in results. NaN throws.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_min_score")]
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
        options_from_js(options, "SearchOptions", |number| {
            Ok(Self {
                max_results: core::check_max_results(number)?,
                ..Self::default()
            })
        })
    }
}

/// Read an `options` argument: `undefined`/`null` (the defaults), a number
/// (a shorthand for one of the options, read by `from_number`) or an options
/// object `T` named `what` in errors. Errors are `TypeError`s.
fn options_from_js<T: DeserializeOwned + Default>(
    options: Option<JsValue>,
    what: &str,
    from_number: impl FnOnce(f64) -> Result<T, String>,
) -> Result<T, JsValue> {
    match options {
        None => Ok(T::default()),
        Some(value) => match value.as_f64() {
            Some(number) => from_number(number).map_err(|e| type_error(&e)),
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
    /// The mode named `name` (see [`core::KeyScoreMode::from_name`]).
    fn from_name(name: &str) -> Result<Self, String> {
        core::KeyScoreMode::from_name(name).map(Self::from)
    }
}

/// How multi-key search matches the query against the keys of an item (see
/// `KeySearchOptions.matchMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum KeyMatchMode {
    /// Every key is matched against the whole query (the default).
    PerKey,
    /// Every term of the query is matched against the keys on its own.
    CrossKey,
}

impl From<core::KeyMatchMode> for KeyMatchMode {
    fn from(mode: core::KeyMatchMode) -> Self {
        match mode {
            core::KeyMatchMode::PerKey => Self::PerKey,
            core::KeyMatchMode::CrossKey => Self::CrossKey,
        }
    }
}

impl From<KeyMatchMode> for core::KeyMatchMode {
    fn from(mode: KeyMatchMode) -> Self {
        match mode {
            KeyMatchMode::PerKey => Self::PerKey,
            KeyMatchMode::CrossKey => Self::CrossKey,
        }
    }
}

impl KeyMatchMode {
    /// The mode named `name` (see [`core::KeyMatchMode::from_name`]).
    fn from_name(name: &str) -> Result<Self, String> {
        core::KeyMatchMode::from_name(name).map(Self::from)
    }
}

/// Parse a mode value like the Node.js binding does: a string is parsed by
/// `from_name`, and any other value is reported by `invalid` with its
/// `typeof` (`null` for null).
fn parse_mode<T>(
    value: &JsValue,
    from_name: fn(&str) -> Result<T, String>,
    invalid: fn(&str) -> String,
) -> Result<T, String> {
    match value.as_string() {
        Some(name) => from_name(&name),
        None if value.is_null() => Err(invalid("null")),
        None => Err(invalid(&value.js_typeof().as_string().unwrap_or_default())),
    }
}

/// Deserialize a mode field of an options object like the Node.js binding
/// reads it: `undefined` (or a missing field) leaves it unset, and any other
/// value goes through [`parse_mode`], so that `null` is rejected like the
/// other values that are not a mode. The field is read as the `JsValue`
/// itself: serde-wasm-bindgen reports `null` and `undefined` alike as a
/// unit, and other types with its own messages.
fn deserialize_mode<'de, D: Deserializer<'de>, T>(
    deserializer: D,
    from_name: fn(&str) -> Result<T, String>,
    invalid: fn(&str) -> String,
) -> Result<Option<T>, D::Error> {
    let value: JsValue = serde_wasm_bindgen::preserve::deserialize(deserializer)?;
    if value.is_undefined() {
        return Ok(None);
    }
    parse_mode(&value, from_name, invalid)
        .map(Some)
        .map_err(de::Error::custom)
}

/// `KeySearchOptions.scoreMode` and `KeyClosestOptions.scoreMode` (see
/// [`deserialize_mode`]).
fn deserialize_score_mode<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<KeyScoreMode>, D::Error> {
    deserialize_mode(
        deserializer,
        KeyScoreMode::from_name,
        core::invalid_score_mode,
    )
}

/// `KeySearchOptions.matchMode` and `KeyClosestOptions.matchMode` (see
/// [`deserialize_mode`]).
fn deserialize_match_mode<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<KeyMatchMode>, D::Error> {
    deserialize_mode(
        deserializer,
        KeyMatchMode::from_name,
        core::invalid_match_mode,
    )
}

/// Options for multi-key search: `searchKeys()`, `KeyedFuzzyIndex.search()`
/// and the object search built on them (`searchObjects()`,
/// `FuzzyObjectIndex.search()`). The `SearchOptions` fields, plus
/// `scoreMode` and `matchMode`.
#[derive(Debug, Clone, Default, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct KeySearchOptions {
    // The `SearchOptions` fields are declared here rather than flattened in:
    // with `#[serde(flatten)]`, serde-wasm-bindgen reads the object as a map
    // of its own enumerable entries, so options read from a getter or the
    // prototype chain (which the Node.js binding reads) were ignored and the
    // entries of a `Map` were taken. Declared fields are read by name, like
    // `SearchOptions` reads them.
    /// Maximum number of results to return: a non-negative integer, or
    /// `Infinity` for no limit. NaN, negative and fractional values throw.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_max_results")]
    pub max_results: Option<u32>,
    /// Minimum combined score (0.0-1.0, see `scoreMode`) to include in
    /// results. NaN throws.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_min_score")]
    pub min_score: Option<f64>,
    /// Accepted for compatibility with `SearchOptions`, but has no effect:
    /// multi-key results have no match positions.
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
    /// If true, return all items when the query has no search term: empty,
    /// whitespace-only, or only query syntax such as `^` or `!`. Every item
    /// then scores 1, in every `scoreMode`. Default is false.
    #[tsify(optional)]
    #[serde(default)]
    pub return_all_on_empty: Option<bool>,
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
    /// apply to the combined score. Equal scores are ordered by the UTF-8
    /// byte length of the best-matching key's text (the key contributing
    /// most to the score: highest `weight * keyScore`, or highest `keyScore`
    /// in `"max"` mode; the first one on a tie), then by index. Any other
    /// value throws a `TypeError`.
    ///
    /// With `matchMode: "crossKey"`, `"matched"` counts the weight of each
    /// key in proportion to the share of the query it matches, and `"max"`
    /// scores each term by the key it matches best (see `matchMode`).
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_score_mode")]
    pub score_mode: Option<KeyScoreMode>,
    /// How the query is matched against the keys of an item:
    ///
    /// - `"perKey"` (default): every key is matched against the whole query,
    ///   like `search()` matches an item. A key scores 0 unless it matches
    ///   every term of the query and none of its `!term` exclusions.
    /// - `"crossKey"`: every term is matched against the keys on its own, so
    ///   the terms may match different keys: `"john tokyo"` finds an item
    ///   whose name is "John Smith" and whose city is "Tokyo". Every term
    ///   must match at least one key, and a `!term` matching any key
    ///   excludes the item. A key's score (`keyScores`) is the share of the
    ///   query it matches: the scores of the terms it matches, each counting
    ///   in proportion to the score of a perfect match of the term (which
    ///   grows with its length), so a key matching every term perfectly
    ///   scores 1. `scoreMode` combines these key scores: `"weighted"` as
    ///   `sum(weight * keyScore) / sum(weight)`, as in `"perKey"` mode;
    ///   `"matched"` as `sum(weight * keyScore) / sum(weight * coverage)`,
    ///   where a key's coverage is the share of the query made up by the
    ///   terms it matches (in `"perKey"` mode, 1 for a key that matches and
    ///   0 otherwise); `"max"` by taking each term's score on the key it
    ///   matches best. In `"max"` mode, an item whose every term matches some
    ///   key perfectly scores 1. In `"matched"` mode, only an item whose
    ///   every term matches perfectly each key it matches scores 1: a term
    ///   that also matches another key partially lowers the score, so
    ///   `"john tokyo"` scores 0.89 on an item whose name is "John Smith",
    ///   whose city is "Tokyo" and whose email "jtokyo@example.com" matches
    ///   `tokyo` partially.
    ///
    /// Only keys with a positive weight take part in either mode; keys whose
    /// weight is 0 still get `keyScores`. For a query of a single term
    /// without exclusions, both modes return the same results. Any other
    /// value throws a `TypeError`.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_match_mode")]
    pub match_mode: Option<KeyMatchMode>,
}

impl KeySearchOptions {
    /// Read the `options` argument of `searchKeys` and
    /// `KeyedFuzzyIndex.search`: `undefined`/`null`, a number (shorthand for
    /// `{ maxResults }`) or a `KeySearchOptions` object.
    pub(crate) fn from_js_or_max_results(options: Option<JsValue>) -> Result<Self, JsValue> {
        options_from_js(options, "KeySearchOptions", |number| {
            Ok(Self {
                max_results: core::check_max_results(number)?,
                ..Self::default()
            })
        })
    }

    /// The options of the shared core-lib search.
    pub(crate) fn to_core(&self) -> core::SearchKeysOptions {
        core::SearchKeysOptions {
            max_results: self.max_results,
            min_score: self.min_score,
            is_case_sensitive: self.is_case_sensitive,
            return_all_on_empty: self.return_all_on_empty,
            score_mode: self.score_mode.map(Into::into),
            match_mode: self.match_mode.map(Into::into),
        }
    }
}

/// Options for `KeyedFuzzyIndex.closest()` and `FuzzyObjectIndex.closest()`:
/// the `KeySearchOptions` fields that apply to finding the best match. The
/// result is the first result of `search()` with these options and
/// `maxResults: 1`. Other `KeySearchOptions` fields are not read.
#[derive(Debug, Clone, Default, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct KeyClosestOptions {
    // Each field is read exactly like the `KeySearchOptions` field of the same
    // name (the same serde attributes and deserializers).
    /// Minimum combined score (0.0-1.0, see `scoreMode`): `closest()`
    /// returns null when the best match scores below it. NaN throws.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_min_score")]
    pub min_score: Option<f64>,
    /// How the per-key scores of an item are combined into its score:
    /// `"weighted"` (default), `"matched"` or `"max"`, as in
    /// `KeySearchOptions.scoreMode`. Any other value throws a `TypeError`.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_score_mode")]
    pub score_mode: Option<KeyScoreMode>,
    /// How the query is matched against the keys of an item: `"perKey"`
    /// (default) or `"crossKey"`, as in `KeySearchOptions.matchMode`. Any
    /// other value throws a `TypeError`.
    #[tsify(optional)]
    #[serde(default, deserialize_with = "deserialize_match_mode")]
    pub match_mode: Option<KeyMatchMode>,
}

impl KeyClosestOptions {
    /// Read the `options` argument of `KeyedFuzzyIndex.closest`:
    /// `undefined`/`null`, a number (shorthand for `{ minScore }`, as in the
    /// Node.js binding) or a `KeyClosestOptions` object.
    pub(crate) fn from_js_or_min_score(options: Option<JsValue>) -> Result<Self, JsValue> {
        options_from_js(options, "KeyClosestOptions", |min_score| {
            Ok(Self {
                min_score: core::check_min_score(Some(min_score))?,
                ..Self::default()
            })
        })
    }

    /// The options of the search whose first result `closest()` returns.
    pub(crate) fn to_core(&self) -> core::SearchKeysOptions {
        core::SearchKeysOptions {
            max_results: Some(1),
            min_score: self.min_score,
            score_mode: self.score_mode.map(Into::into),
            match_mode: self.match_mode.map(Into::into),
            ..core::SearchKeysOptions::default()
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

/// Read a `minScore` option like the Node.js binding does (see
/// [`core::check_min_score`]): NaN, which used to filter out every match
/// silently, is rejected.
fn deserialize_min_score<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f64>, D::Error> {
    core::check_min_score(Option::<f64>::deserialize(deserializer)?)
        .map_err(serde::de::Error::custom)
}

/// Read the `minScore` argument of `closest()` and `FuzzyIndex.closest()`
/// like the Node.js binding does: a `TypeError` for a value that is not a
/// number (see [`optional_number_from_js`]) and for NaN (see
/// [`core::check_min_score`]).
pub(crate) fn min_score_arg(min_score: Option<JsValue>) -> Result<Option<f64>, JsValue> {
    let min_score = optional_number_from_js(min_score, "minScore")?;
    core::check_min_score(min_score).map_err(|message| type_error(&message))
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
    #[wasm_bindgen(unchecked_param_type = "string[]")] items: JsValue,
    #[wasm_bindgen(unchecked_optional_param_type = "number | SearchOptions | null")]
    options: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let items = strings_from_js(&items)?;
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
/// Returns the best match, or null if no match is found. Like
/// `SearchResult.item`, the returned string is converted to UTF-8: a lone
/// UTF-16 surrogate in it becomes U+FFFD.
/// If `minScore` is provided, returns null when the best match scores below
/// the threshold. A NaN `minScore`, or one that is not a number, throws a
/// `TypeError`.
#[wasm_bindgen(unchecked_return_type = "string | null")]
pub fn closest(
    query: String,
    #[wasm_bindgen(unchecked_param_type = "string[]")] items: JsValue,
    #[wasm_bindgen(js_name = "minScore", unchecked_optional_param_type = "number | null")]
    min_score: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let items = strings_from_js(&items)?;
    let min_score = min_score_arg(min_score)?;
    let results = search_impl(query, items, Some(1), min_score, false, CaseMatching::Smart);
    Ok(or_null(results.into_iter().next().map(|r| r.item)))
}
