mod index;
mod keyed_index;
mod keys;

pub use index::FuzzyIndex;
pub use keyed_index::KeyedFuzzyIndex;
pub use keys::search_keys;

// Re-export core algorithm functions and types for use in submodules and tests.
pub(crate) use rapid_fuzzy_core::search::resolve_case_matching;

#[cfg(test)]
pub(crate) use rapid_fuzzy_core::search::compute_char_mask;

use napi::bindgen_prelude::{FromNapiValue, Object, TypeName, Unknown, ValidateNapiValue, sys};
use napi::{Status, ValueType};
use napi_derive::napi;
use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::{
    KeyMatchMode, KeyScoreMode, SearchKeysOptions, check_max_results, invalid_match_mode,
    invalid_score_mode, is_empty_query,
};

// -------------------------
// Napi-specific types
// -------------------------

/// Classification of how a query matched an item.
///
/// Derived from the matched character positions:
/// - **Exact**: all positions consecutive from index 0, covering every character in the item.
/// - **Prefix**: all positions consecutive from index 0, but the item is longer.
/// - **Contains**: all positions consecutive (a substring match), not starting at 0.
/// - **Fuzzy**: positions have gaps (character-level fuzzy match).
#[napi(string_enum)]
#[derive(Debug, PartialEq)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

impl From<rapid_fuzzy_core::search::MatchType> for MatchType {
    fn from(mt: rapid_fuzzy_core::search::MatchType) -> Self {
        match mt {
            rapid_fuzzy_core::search::MatchType::Exact => Self::Exact,
            rapid_fuzzy_core::search::MatchType::Prefix => Self::Prefix,
            rapid_fuzzy_core::search::MatchType::Contains => Self::Contains,
            rapid_fuzzy_core::search::MatchType::Fuzzy => Self::Fuzzy,
        }
    }
}

/// A single fuzzy search result with the matched item and its score.
#[napi(object)]
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
    /// Derived from positions at zero additional cost.
    pub match_type: Option<MatchType>,
}

impl From<rapid_fuzzy_core::search::SearchResult> for SearchResult {
    fn from(r: rapid_fuzzy_core::search::SearchResult) -> Self {
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
///
/// Use this when you maintain your own data array and only need the index
/// to look up the original item. Avoids String cloning overhead.
#[napi(object)]
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
    /// Derived from positions at zero additional cost.
    pub match_type: Option<MatchType>,
}

impl From<rapid_fuzzy_core::search::IndexSearchResult> for IndexSearchResult {
    fn from(r: rapid_fuzzy_core::search::IndexSearchResult) -> Self {
        Self {
            index: r.index,
            score: r.score,
            positions: r.positions,
            match_type: r.match_type.map(Into::into),
        }
    }
}

/// Options for the search function.
#[napi(object, object_from_js = false)]
pub struct SearchOptions {
    /// Maximum number of results to return: a non-negative integer, or
    /// `Infinity` for no limit. NaN, negative and fractional values throw.
    pub max_results: Option<u32>,
    /// Minimum normalized score (0.0-1.0) to include in results.
    pub min_score: Option<f64>,
    /// If true, include matched character positions in results.
    pub include_positions: Option<bool>,
    /// If true, matching is case-sensitive. When false or omitted, matching
    /// is smart case: case-insensitive while the query is all lower-case, and
    /// case-sensitive once it contains an upper-case letter. `false` does not
    /// force case-insensitive matching; lower-case the query for that.
    pub is_case_sensitive: Option<bool>,
    /// If true, return all items when the query has no search term: empty,
    /// whitespace-only, or only query syntax such as `^` or `!`.
    /// Useful for filter-as-you-type UIs where the full list should appear
    /// before the user starts typing. Default is false.
    pub return_all_on_empty: Option<bool>,
}

/// Validate a `maxResults` value coming from JavaScript.
///
/// napi's `u32` conversion wraps numbers modulo 2^32 (`Infinity`, `NaN` and
/// `0.5` became 0, `-1` became 4294967295), so the value is read as a double
/// and checked instead: non-negative integers are accepted (values beyond
/// `u32::MAX` exceed any array length and mean "no limit"), `Infinity` means
/// no limit, and NaN, negative or fractional values are rejected.
///
/// The check itself is [`check_max_results`], shared with the WebAssembly
/// binding.
pub(crate) fn resolve_max_results(value: f64) -> napi::Result<Option<u32>> {
    check_max_results(value).map_err(|message| napi::Error::new(Status::InvalidArg, message))
}

/// Read an optional property of the options object `type_name`, treating
/// `undefined` (or a missing property) as `None`, like the conversion
/// `#[napi(object)]` generates.
fn optional_field<T: FromNapiValue>(
    obj: &Object<'_>,
    type_name: &str,
    field: &str,
) -> napi::Result<Option<T>> {
    obj.get::<T>(field).map_err(|err| {
        napi::Error::new(err.status, format!("{} on {type_name}.{field}", err.reason))
    })
}

/// An options object read from JavaScript by hand (`object_from_js = false`),
/// so that its fields are validated.
pub(crate) trait OptionsObject: Sized {
    /// The TypeScript name of the options type.
    const NAME: &'static str;
    /// The TypeScript type of a `number | options` argument.
    const ARG_TYPE: &'static str;

    fn read(obj: &Object<'_>) -> napi::Result<Self>;
}

/// Convert a `number | options` argument: a number with `from_number`, an
/// options object `T` with `from_options`. Any other value is rejected with
/// an `InvalidArg` error naming `shorthand`, the option a number sets.
///
/// # Safety
///
/// `env` and `napi_val` must be valid, as for [`FromNapiValue::from_napi_value`].
unsafe fn number_or_options<T: OptionsObject, R>(
    env: sys::napi_env,
    napi_val: sys::napi_value,
    shorthand: &str,
    from_number: impl FnOnce(f64) -> napi::Result<R>,
    from_options: impl FnOnce(T) -> R,
) -> napi::Result<R> {
    let value_type = unsafe { Unknown::from_napi_value(env, napi_val)? }.get_type()?;
    match value_type {
        ValueType::Number => from_number(unsafe { f64::from_napi_value(env, napi_val)? }),
        ValueType::Object => Ok(from_options(T::read(&unsafe {
            Object::from_napi_value(env, napi_val)?
        })?)),
        _ => Err(napi::Error::new(
            Status::InvalidArg,
            format!(
                "Expected a number ({shorthand}) or a {} object, got {value_type}",
                T::NAME
            ),
        )),
    }
}

/// Read the `SearchOptions` fields of `obj`, an options object `type_name`.
fn read_search_options(obj: &Object<'_>, type_name: &str) -> napi::Result<SearchOptions> {
    let max_results = match optional_field::<f64>(obj, type_name, "maxResults")? {
        Some(value) => resolve_max_results(value)?,
        None => None,
    };
    Ok(SearchOptions {
        max_results,
        min_score: optional_field(obj, type_name, "minScore")?,
        include_positions: optional_field(obj, type_name, "includePositions")?,
        is_case_sensitive: optional_field(obj, type_name, "isCaseSensitive")?,
        return_all_on_empty: optional_field(obj, type_name, "returnAllOnEmpty")?,
    })
}

impl OptionsObject for SearchOptions {
    const NAME: &'static str = "SearchOptions";
    const ARG_TYPE: &'static str = "number | SearchOptions";

    fn read(obj: &Object<'_>) -> napi::Result<Self> {
        read_search_options(obj, Self::NAME)
    }
}

impl FromNapiValue for SearchOptions {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        Self::read(&unsafe { Object::from_napi_value(env, napi_val)? })
    }
}

impl ValidateNapiValue for SearchOptions {}

/// Options for multi-key search: `searchKeys()`, `KeyedFuzzyIndex.search()`
/// and the object search built on them (`searchObjects()`,
/// `FuzzyObjectIndex.search()`). The `SearchOptions` fields, plus
/// `scoreMode` and `matchMode`.
#[napi(object, object_from_js = false, object_to_js = false)]
pub struct KeySearchOptions {
    /// Maximum number of results to return: a non-negative integer, or
    /// `Infinity` for no limit. NaN, negative and fractional values throw.
    pub max_results: Option<u32>,
    /// Minimum combined score (0.0-1.0, see `scoreMode`) to include in
    /// results.
    pub min_score: Option<f64>,
    /// Accepted for compatibility with `SearchOptions`, but has no effect:
    /// multi-key results have no match positions.
    pub include_positions: Option<bool>,
    /// If true, matching is case-sensitive. When false or omitted, matching
    /// is smart case: case-insensitive while the query is all lower-case, and
    /// case-sensitive once it contains an upper-case letter. `false` does not
    /// force case-insensitive matching; lower-case the query for that.
    pub is_case_sensitive: Option<bool>,
    /// If true, return all items when the query has no search term: empty,
    /// whitespace-only, or only query syntax such as `^` or `!`. Every item
    /// then scores 1, in every `scoreMode`. Default is false.
    pub return_all_on_empty: Option<bool>,
    /// How the per-key scores (`keyScores`) of an item are combined into its
    /// `score`. Only keys with a positive weight take part:
    ///
    /// - `'weighted'` (default): the weighted mean over all keys,
    ///   `sum(weight * keyScore) / sum(weight)`. A key that does not match
    ///   counts as 0, so an exact match on one key out of several scores only
    ///   that key's share of the total weight.
    /// - `'matched'`: the weighted mean over the keys that match
    ///   (`keyScore > 0`) only. An item whose only matching key matches
    ///   exactly scores 1.
    /// - `'max'`: the highest score of any key. Weights then only select the
    ///   keys that take part (weight > 0).
    ///
    /// `keyScores` are the same in every mode; `minScore` and `maxResults`
    /// apply to the combined score. Equal scores are ordered by the length of
    /// the best-matching key's text (the key contributing most to the score:
    /// highest `weight * keyScore`, or highest `keyScore` in `'max'` mode;
    /// the first one on a tie), then by index. Any other value throws an
    /// `InvalidArg` error.
    ///
    /// With `matchMode: 'crossKey'`, `'matched'` counts the weight of each
    /// key in proportion to the share of the query it matches, and `'max'`
    /// scores each term by the key it matches best (see `matchMode`).
    #[napi(ts_type = "KeyScoreMode")]
    pub score_mode: Option<KeyScoreMode>,
    /// How the query is matched against the keys of an item:
    ///
    /// - `'perKey'` (default): every key is matched against the whole query,
    ///   like `search()` matches an item. A key scores 0 unless it matches
    ///   every term of the query and none of its `!term` exclusions.
    /// - `'crossKey'`: every term is matched against the keys on its own, so
    ///   the terms may match different keys: `'john tokyo'` finds an item
    ///   whose name is "John Smith" and whose city is "Tokyo". Every term
    ///   must match at least one key, and a `!term` matching any key
    ///   excludes the item. A key's score (`keyScores`) is the share of the
    ///   query it matches: the scores of the terms it matches, each counting
    ///   in proportion to the score of a perfect match of the term (which
    ///   grows with its length), so a key matching every term perfectly
    ///   scores 1. `scoreMode` combines these key scores: `'weighted'` as
    ///   `sum(weight * keyScore) / sum(weight)`, as in `'perKey'` mode;
    ///   `'matched'` as `sum(weight * keyScore) / sum(weight * coverage)`,
    ///   where a key's coverage is the share of the query made up by the
    ///   terms it matches (in `'perKey'` mode, 1 for a key that matches and
    ///   0 otherwise); `'max'` by taking each term's score on the key it
    ///   matches best. In `'max'` mode, an item whose every term matches some
    ///   key perfectly scores 1. In `'matched'` mode, only an item whose
    ///   every term matches perfectly each key it matches scores 1: a term
    ///   that also matches another key partially lowers the score, so
    ///   `'john tokyo'` scores 0.89 on an item whose name is "John Smith",
    ///   whose city is "Tokyo" and whose email "jtokyo@example.com" matches
    ///   `tokyo` partially.
    ///
    /// Only keys with a positive weight take part in either mode; keys whose
    /// weight is 0 still get `keyScores`. For a query of a single term
    /// without exclusions, both modes return the same results. Any other
    /// value throws an `InvalidArg` error.
    #[napi(ts_type = "KeyMatchMode")]
    pub match_mode: Option<KeyMatchMode>,
}

impl OptionsObject for KeySearchOptions {
    const NAME: &'static str = "KeySearchOptions";
    const ARG_TYPE: &'static str = "number | KeySearchOptions";

    fn read(obj: &Object<'_>) -> napi::Result<Self> {
        let base = read_search_options(obj, Self::NAME)?;
        Ok(Self {
            score_mode: read_score_mode(obj)?,
            match_mode: read_match_mode(obj)?,
            ..base.into()
        })
    }
}

/// Read the `scoreMode` field of a multi-key options object.
fn read_score_mode(obj: &Object<'_>) -> napi::Result<Option<KeyScoreMode>> {
    Ok(obj.get::<ScoreModeArg>("scoreMode")?.map(|arg| arg.0))
}

/// Read the `matchMode` field of a multi-key options object.
fn read_match_mode(obj: &Object<'_>) -> napi::Result<Option<KeyMatchMode>> {
    Ok(obj.get::<MatchModeArg>("matchMode")?.map(|arg| arg.0))
}

/// Options for `KeyedFuzzyIndex.closest()` and `FuzzyObjectIndex.closest()`:
/// the `KeySearchOptions` fields that apply to finding the best match. The
/// result is the first result of `search()` with these options and
/// `maxResults: 1`. Other `KeySearchOptions` fields are not read.
#[napi(object, object_from_js = false, object_to_js = false)]
#[derive(Default)]
pub struct KeyClosestOptions {
    /// Minimum combined score (0.0-1.0, see `scoreMode`): `closest()`
    /// returns null when the best match scores below it.
    pub min_score: Option<f64>,
    /// How the per-key scores of an item are combined into its score:
    /// `'weighted'` (default), `'matched'` or `'max'`, as in
    /// `KeySearchOptions.scoreMode`. Any other value throws an `InvalidArg`
    /// error.
    #[napi(ts_type = "KeyScoreMode")]
    pub score_mode: Option<KeyScoreMode>,
    /// How the query is matched against the keys of an item: `'perKey'`
    /// (default) or `'crossKey'`, as in `KeySearchOptions.matchMode`. Any
    /// other value throws an `InvalidArg` error.
    #[napi(ts_type = "KeyMatchMode")]
    pub match_mode: Option<KeyMatchMode>,
}

impl OptionsObject for KeyClosestOptions {
    const NAME: &'static str = "KeyClosestOptions";
    const ARG_TYPE: &'static str = "number | KeyClosestOptions";

    /// Read the fields like [`KeySearchOptions`] reads them, in the same order.
    fn read(obj: &Object<'_>) -> napi::Result<Self> {
        Ok(Self {
            min_score: optional_field(obj, Self::NAME, "minScore")?,
            score_mode: read_score_mode(obj)?,
            match_mode: read_match_mode(obj)?,
        })
    }
}

impl KeyClosestOptions {
    /// The options of the search whose first result `closest()` returns.
    pub(crate) fn to_core(&self) -> SearchKeysOptions {
        SearchKeysOptions {
            max_results: Some(1),
            min_score: self.min_score,
            score_mode: self.score_mode,
            match_mode: self.match_mode,
            ..SearchKeysOptions::default()
        }
    }
}

/// The `options` argument of `KeyedFuzzyIndex.closest`: a `KeyClosestOptions`
/// object, or a number as a shorthand for `{ minScore }`.
pub struct KeyClosestOptionsArg(pub KeyClosestOptions);

impl TypeName for KeyClosestOptionsArg {
    fn type_name() -> &'static str {
        KeyClosestOptions::ARG_TYPE
    }

    fn value_type() -> ValueType {
        ValueType::Unknown
    }
}

impl FromNapiValue for KeyClosestOptionsArg {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        let options = unsafe {
            number_or_options(
                env,
                napi_val,
                "minScore",
                |min_score| {
                    Ok(KeyClosestOptions {
                        min_score: Some(min_score),
                        ..KeyClosestOptions::default()
                    })
                },
                |options| options,
            )
        }?;
        Ok(Self(options))
    }
}

impl From<SearchOptions> for KeySearchOptions {
    fn from(opts: SearchOptions) -> Self {
        Self {
            max_results: opts.max_results,
            min_score: opts.min_score,
            include_positions: opts.include_positions,
            is_case_sensitive: opts.is_case_sensitive,
            return_all_on_empty: opts.return_all_on_empty,
            score_mode: None,
            match_mode: None,
        }
    }
}

/// A `scoreMode` value: `'weighted'`, `'matched'` or `'max'`. Anything else
/// (including other types) is rejected with an `InvalidArg` error.
pub struct ScoreModeArg(pub KeyScoreMode);

impl TypeName for ScoreModeArg {
    fn type_name() -> &'static str {
        "KeyScoreMode"
    }

    fn value_type() -> ValueType {
        ValueType::String
    }
}

impl FromNapiValue for ScoreModeArg {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        let mode = match unsafe { Unknown::from_napi_value(env, napi_val)? }.get_type()? {
            ValueType::String => {
                KeyScoreMode::from_name(&unsafe { String::from_napi_value(env, napi_val)? })
            }
            // `typeof`-style names: number, null, object, ...
            other => Err(invalid_score_mode(&other.to_string().to_lowercase())),
        };
        mode.map(Self)
            .map_err(|message| napi::Error::new(Status::InvalidArg, message))
    }
}

/// A `matchMode` value: `'perKey'` or `'crossKey'`. Anything else (including
/// other types) is rejected with an `InvalidArg` error.
pub struct MatchModeArg(pub KeyMatchMode);

impl TypeName for MatchModeArg {
    fn type_name() -> &'static str {
        "KeyMatchMode"
    }

    fn value_type() -> ValueType {
        ValueType::String
    }
}

impl FromNapiValue for MatchModeArg {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        let mode = match unsafe { Unknown::from_napi_value(env, napi_val)? }.get_type()? {
            ValueType::String => {
                KeyMatchMode::from_name(&unsafe { String::from_napi_value(env, napi_val)? })
            }
            // `typeof`-style names: number, null, object, ...
            other => Err(invalid_match_mode(&other.to_string().to_lowercase())),
        };
        mode.map(Self)
            .map_err(|message| napi::Error::new(Status::InvalidArg, message))
    }
}

/// The `options` argument of the search functions: a `maxResults` number or
/// an options object `T`.
pub enum OptionsArg<T> {
    MaxResults(Option<u32>),
    Options(T),
}

/// The `options` argument of `search`, `FuzzyIndex.search` and
/// `FuzzyIndex.searchIndices`: a `maxResults` number or a `SearchOptions`.
pub type SearchOptionsArg = OptionsArg<SearchOptions>;

/// The `options` argument of `searchKeys` and `KeyedFuzzyIndex.search`: a
/// `maxResults` number or a `KeySearchOptions`.
pub type KeySearchOptionsArg = OptionsArg<KeySearchOptions>;

impl<T: OptionsObject> TypeName for OptionsArg<T> {
    fn type_name() -> &'static str {
        T::ARG_TYPE
    }

    fn value_type() -> ValueType {
        ValueType::Unknown
    }
}

impl<T: OptionsObject> FromNapiValue for OptionsArg<T> {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        unsafe {
            number_or_options(
                env,
                napi_val,
                "maxResults",
                |value| resolve_max_results(value).map(Self::MaxResults),
                Self::Options,
            )
        }
    }
}

impl KeySearchOptionsArg {
    /// The core-lib options of a `number | KeySearchOptions` argument.
    pub(crate) fn resolve(options: Option<Self>) -> SearchKeysOptions {
        match options {
            Some(OptionsArg::Options(opts)) => SearchKeysOptions {
                max_results: opts.max_results,
                min_score: opts.min_score,
                is_case_sensitive: opts.is_case_sensitive,
                return_all_on_empty: opts.return_all_on_empty,
                score_mode: opts.score_mode,
                match_mode: opts.match_mode,
            },
            Some(OptionsArg::MaxResults(max_results)) => SearchKeysOptions {
                max_results,
                ..SearchKeysOptions::default()
            },
            None => SearchKeysOptions::default(),
        }
    }
}

/// Options resolved from a `number | SearchOptions` argument.
pub(crate) struct ResolvedSearchOptions {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub include_positions: bool,
    pub case_matching: CaseMatching,
    pub return_all_on_empty: bool,
}

impl ResolvedSearchOptions {
    pub(crate) fn new(options: Option<SearchOptionsArg>) -> Self {
        match options {
            Some(SearchOptionsArg::Options(opts)) => Self {
                max_results: opts.max_results,
                min_score: opts.min_score,
                include_positions: opts.include_positions.unwrap_or(false),
                case_matching: resolve_case_matching(opts.is_case_sensitive),
                return_all_on_empty: opts.return_all_on_empty.unwrap_or(false),
            },
            Some(SearchOptionsArg::MaxResults(max_results)) => Self {
                max_results,
                ..Self::new(None)
            },
            None => Self {
                max_results: None,
                min_score: None,
                include_positions: false,
                case_matching: CaseMatching::Smart,
                return_all_on_empty: false,
            },
        }
    }
}

/// Internal search implementation wrapping the core-lib algorithm.
/// Returns napi-compatible `SearchResult` types.
pub(crate) fn search_impl(
    query: String,
    items: Vec<String>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    rapid_fuzzy_core::search::search_impl(
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
/// Uses the nucleo matcher (the one in the Helix editor): every character of
/// each query term must occur in the item in order, so letters missing from
/// the query are tolerated but substituted or swapped letters are not. Matches
/// at the start of the item, at word boundaries and in consecutive runs score
/// higher. Matching is smart case by default (see `SearchOptions`).
///
/// Terms are separated by any whitespace (including the ideographic space
/// U+3000). A query without any search term (empty, whitespace-only or only
/// syntax such as `^`) returns no results, and so does a query containing a
/// single term longer than 2,520 characters, which cannot be scored.
///
/// The third argument accepts either a number (maxResults for backward
/// compatibility) or a SearchOptions object with maxResults and minScore.
/// `maxResults` must be a non-negative integer or `Infinity`.
#[napi]
pub fn search(
    query: String,
    items: Vec<String>,
    #[napi(ts_arg_type = "number | SearchOptions | undefined | null")] options: Option<
        SearchOptionsArg,
    >,
) -> Vec<SearchResult> {
    let ResolvedSearchOptions {
        max_results,
        min_score,
        include_positions,
        case_matching,
        return_all_on_empty,
    } = ResolvedSearchOptions::new(options);

    if return_all_on_empty && is_empty_query(&query) {
        let limit = max_results.unwrap_or(u32::MAX) as usize;
        return items
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
    }

    search_impl(
        query,
        items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    )
}

/// Find the closest matching string from a list.
///
/// Returns the best match, or null if no match is found.
/// If minScore is provided, returns null when the best match scores below the threshold.
#[napi]
pub fn closest(query: String, items: Vec<String>, min_score: Option<f64>) -> Option<String> {
    let results = search_impl(query, items, Some(1), min_score, false, CaseMatching::Smart);
    results.into_iter().next().map(|r| r.item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefilters_fold_diacritics_like_the_matcher() {
        // nucleo matches "cafe" against "café" (Normalization::Smart), so the
        // char-mask prefilter must not reject folded characters.
        let cafe = compute_char_mask("cafe");
        assert_eq!(compute_char_mask("café") & cafe, cafe);
        let arger = compute_char_mask("arger");
        assert_eq!(compute_char_mask("Ärger") & arger, arger);
    }

    #[test]
    fn test_check_max_results() {
        assert_eq!(check_max_results(0.0), Ok(Some(0)));
        assert_eq!(check_max_results(-0.0), Ok(Some(0)));
        assert_eq!(check_max_results(5.0), Ok(Some(5)));
        assert_eq!(check_max_results(f64::INFINITY), Ok(None));
        assert_eq!(check_max_results(4_294_967_296.0), Ok(Some(u32::MAX)));
        assert_eq!(check_max_results(1e300), Ok(Some(u32::MAX)));
        for (invalid, shown) in [
            (f64::NAN, "NaN"),
            (-1.0, "-1"),
            (0.5, "0.5"),
            (-0.5, "-0.5"),
            (f64::NEG_INFINITY, "-Infinity"),
        ] {
            assert_eq!(
                check_max_results(invalid),
                Err(format!(
                    "maxResults must be a non-negative integer or Infinity, got {shown}"
                ))
            );
        }
    }

    #[test]
    fn test_search_basic() {
        let items = vec![
            "TypeScript".to_string(),
            "JavaScript".to_string(),
            "Python".to_string(),
            "TypeSpec".to_string(),
        ];
        let results = search_impl(
            "typscript".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].item, "TypeScript");
    }

    #[test]
    fn test_search_empty_query() {
        let items = vec!["foo".to_string()];
        let results = search_impl(
            "".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_closest() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
        ];
        let result = closest("app".to_string(), items, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_scores_normalized_range() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
            "grape".to_string(),
        ];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        for r in &results {
            assert!(
                r.score >= 0.0 && r.score <= 1.0,
                "score {} out of 0.0-1.0 range for '{}'",
                r.score,
                r.item
            );
        }
    }

    #[test]
    fn test_exact_match_scores_one() {
        let items = vec!["hello".to_string(), "world".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        let exact = results.iter().find(|r| r.item == "hello").unwrap();
        assert!(
            (exact.score - 1.0).abs() < f64::EPSILON,
            "exact match should score 1.0, got {}",
            exact.score
        );
    }

    #[test]
    fn test_partial_match_scores_below_one() {
        let items = vec!["TypeScript".to_string(), "JavaScript".to_string()];
        let results = search_impl(
            "type".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        for r in &results {
            assert!(
                r.score > 0.0 && r.score <= 1.0,
                "partial match score {} should be in (0.0, 1.0] for '{}'",
                r.score,
                r.item
            );
        }
    }

    #[test]
    fn test_min_score_filters_low_quality() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "xyz".to_string(),
        ];
        let all_results = search_impl(
            "apple".to_string(),
            items.clone(),
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        let filtered = search_impl(
            "apple".to_string(),
            items,
            None,
            Some(0.5),
            false,
            CaseMatching::Smart,
        );
        assert!(filtered.len() <= all_results.len());
        for r in &filtered {
            assert!(
                r.score >= 0.5,
                "score {} below min_score 0.5 for '{}'",
                r.score,
                r.item
            );
        }
    }

    #[test]
    fn test_min_score_one_returns_only_exact() {
        let items = vec!["hello".to_string(), "help".to_string(), "world".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "hello");
    }

    #[test]
    fn test_min_score_zero_same_as_none() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "grape".to_string(),
        ];
        let no_threshold = search_impl(
            "ap".to_string(),
            items.clone(),
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        let zero_threshold = search_impl(
            "ap".to_string(),
            items,
            None,
            Some(0.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(no_threshold.len(), zero_threshold.len());
    }

    #[test]
    fn test_min_score_with_max_results() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "appetizer".to_string(),
            "xyz".to_string(),
        ];
        let results = search_impl(
            "apple".to_string(),
            items,
            Some(2),
            Some(0.3),
            false,
            CaseMatching::Smart,
        );
        assert!(results.len() <= 2);
        for r in &results {
            assert!(r.score >= 0.3);
        }
    }

    #[test]
    fn test_closest_with_min_score() {
        let items = vec!["xyz".to_string(), "abc".to_string()];
        // With a very high threshold, closest should return None
        let result = closest("hello".to_string(), items, Some(0.99));
        assert!(result.is_none());
    }

    #[test]
    fn test_case_sensitive_excludes_different_case() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        // Case-sensitive: lowercase query "apple" should not match "Apple" or "APPLE"
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Respect,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "apple");
    }

    #[test]
    fn test_case_sensitive_uppercase_query() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        let results = search_impl(
            "APPLE".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Respect,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "APPLE");
    }

    #[test]
    fn test_smart_case_lowercase_query_matches_any_case() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        // Smart case: all-lowercase query matches any case
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_smart_case_uppercase_query_is_case_sensitive() {
        let items = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];
        // Smart case: query with uppercase becomes case-sensitive
        let results = search_impl(
            "Apple".to_string(),
            items,
            None,
            Some(1.0),
            false,
            CaseMatching::Smart,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item, "Apple");
    }

    #[test]
    fn test_closest_without_min_score() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let result = closest("app".to_string(), items, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_positions_returned_when_requested() {
        let items = vec!["hello world".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert!(
            !results[0].positions.is_empty(),
            "positions should not be empty"
        );
    }

    #[test]
    fn test_positions_empty_when_not_requested() {
        let items = vec!["hello world".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert!(
            results[0].positions.is_empty(),
            "positions should be empty when not requested"
        );
    }

    #[test]
    fn test_positions_are_sorted() {
        let items = vec!["hello world".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        let positions = &results[0].positions;
        for window in positions.windows(2) {
            assert!(
                window[0] <= window[1],
                "positions not sorted: {} > {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn test_positions_within_bounds() {
        let items = vec!["hello".to_string()];
        let results = search_impl(
            "hlo".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        let item_len = results[0].item.chars().count() as u32;
        for &pos in &results[0].positions {
            assert!(
                pos < item_len,
                "position {} out of bounds (len={})",
                pos,
                item_len
            );
        }
    }

    #[test]
    fn test_exact_match_positions() {
        let items = vec!["hello".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].positions, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_match_type_exact() {
        let items = vec!["apple".to_string(), "pineapple".to_string()];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        let exact = results.iter().find(|r| r.item == "apple").unwrap();
        assert_eq!(exact.match_type, Some(MatchType::Exact));
    }

    #[test]
    fn test_match_type_prefix() {
        let items = vec!["application".to_string()];
        let results = search_impl(
            "app".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Prefix));
    }

    #[test]
    fn test_match_type_contains() {
        let items = vec!["pineapple".to_string()];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Contains));
    }

    #[test]
    fn test_match_type_fuzzy() {
        let items = vec!["abcdef".to_string()];
        let results = search_impl(
            "adf".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Fuzzy));
    }

    #[test]
    fn test_match_type_none_without_positions() {
        let items = vec!["hello".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, None);
    }

    #[test]
    fn test_match_type_present_with_positions() {
        let items = vec!["hello".to_string()];
        let results = search_impl(
            "hello".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Exact));
    }

    #[test]
    fn test_match_type_case_insensitive_exact() {
        let items = vec!["Apple".to_string()];
        let results = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, Some(MatchType::Exact));
    }

    #[test]
    fn test_positions_score_consistency() {
        let items = vec![
            "apple".to_string(),
            "application".to_string(),
            "banana".to_string(),
        ];
        let with_pos = search_impl(
            "apple".to_string(),
            items.clone(),
            None,
            None,
            true,
            CaseMatching::Smart,
        );
        let without_pos = search_impl(
            "apple".to_string(),
            items,
            None,
            None,
            false,
            CaseMatching::Smart,
        );
        assert_eq!(with_pos.len(), without_pos.len());
        for (a, b) in with_pos.iter().zip(without_pos.iter()) {
            assert_eq!(a.item, b.item);
            assert!(
                (a.score - b.score).abs() < f64::EPSILON,
                "scores differ: {} vs {}",
                a.score,
                b.score
            );
        }
    }

    mod proptest_search {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn search_max_results_respected(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
                max in 1u32..10
            ) {
                let results = search_impl(query, items, Some(max), None, false, CaseMatching::Smart);
                prop_assert!(results.len() <= max as usize);
            }

            #[test]
            fn search_scores_sorted_descending(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for window in results.windows(2) {
                    prop_assert!(
                        window[0].score >= window[1].score,
                        "results not sorted: {} < {}",
                        window[0].score,
                        window[1].score
                    );
                }
            }

            #[test]
            fn search_scores_normalized(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!(
                        r.score >= 0.0 && r.score <= 1.0,
                        "score {} out of 0.0-1.0 range",
                        r.score
                    );
                }
            }

            #[test]
            fn closest_in_search_results(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let closest_result = closest(query.clone(), items.clone(), None);
                let search_results = search_impl(query, items, None, None, false, CaseMatching::Smart);

                match (closest_result, search_results.first()) {
                    (Some(c), Some(first)) => {
                        prop_assert_eq!(c, first.item.clone());
                    }
                    (None, None) => {} // both empty is fine
                    (Some(_), None) | (None, Some(_)) => {
                        prop_assert!(false, "closest and search disagree on match existence");
                    }
                }
            }

            #[test]
            fn search_indices_valid(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let len = items.len();
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!((r.index as usize) < len, "index {} out of bounds (len={})", r.index, len);
                }
            }

            #[test]
            fn search_positions_within_bounds(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, true, CaseMatching::Smart);
                for r in &results {
                    let item_len = r.item.chars().count() as u32;
                    for &pos in &r.positions {
                        prop_assert!(pos < item_len, "position {} >= item length {} for '{}'", pos, item_len, r.item);
                    }
                }
            }

            #[test]
            fn search_positions_sorted_and_unique(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, true, CaseMatching::Smart);
                for r in &results {
                    for window in r.positions.windows(2) {
                        prop_assert!(window[0] < window[1], "positions not strictly sorted: {} >= {}", window[0], window[1]);
                    }
                }
            }

            #[test]
            fn search_min_score_respected(
                query in "[a-z]{1,5}",
                items in prop::collection::vec("[a-z]{1,10}", 1..20),
                threshold in 0.0f64..1.0
            ) {
                let results = search_impl(query, items, None, Some(threshold), false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!(
                        r.score >= threshold,
                        "score {} below threshold {}",
                        r.score,
                        threshold
                    );
                }
            }
        }
    }

    mod unicode_tests {
        use super::*;

        #[test]
        fn test_search_cjk() {
            let items = vec!["東京".to_string(), "大阪".to_string(), "京都".to_string()];
            let results = search_impl(
                "東".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
        }

        #[test]
        fn test_search_emoji() {
            let items = vec![
                "🎉 party".to_string(),
                "🎊 celebration".to_string(),
                "work".to_string(),
            ];
            let results = search_impl(
                "party".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
            assert!(results[0].item.contains("party"));
        }

        #[test]
        fn test_search_accented() {
            let items = vec![
                "café".to_string(),
                "resume".to_string(),
                "naïve".to_string(),
            ];
            let results = search_impl(
                "cafe".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
        }

        #[test]
        fn test_closest_cjk() {
            let items = vec!["大阪".to_string(), "京都".to_string(), "東京都".to_string()];
            let result = closest("東京".to_string(), items, None);
            assert!(result.is_some());
        }

        #[test]
        fn test_tiebreaker_consistent_for_unicode() {
            // Verify standalone search and indexed search produce identical
            // ordering for Unicode items where byte length != char count.
            // "hello世界" = 11 bytes / 7 chars (CJK chars are 3 bytes each)
            // "helloab" = 7 bytes / 7 chars
            // Same char count (7) but different byte lengths (11 vs 7).
            // Using .chars().count() would treat them as equal length;
            // using .len() (byte length) correctly differentiates them.
            // Both items contain the query's ASCII letters (h,e,l,o)
            // so the bitmask pre-filter in the indexed path matches both.
            use rapid_fuzzy_core::search::FuzzyIndexCore;

            let items = vec!["hello世界".to_string(), "helloab".to_string()];

            let standalone = search_impl(
                "hello".to_string(),
                items.clone(),
                None,
                None,
                false,
                CaseMatching::Smart,
            );

            let index = FuzzyIndexCore::new(items);
            let indexed = index.search_impl("hello", None, None, false, CaseMatching::Smart);

            assert_eq!(
                standalone.len(),
                indexed.len(),
                "result count differs between standalone and indexed search"
            );
            for (s, i) in standalone.iter().zip(indexed.iter()) {
                assert_eq!(
                    s.item, i.item,
                    "ordering differs between standalone and indexed search"
                );
            }
        }

        #[test]
        fn test_search_mixed_scripts() {
            let items = vec![
                "hello世界".to_string(),
                "goodbye世間".to_string(),
                "test".to_string(),
            ];
            let results = search_impl(
                "hello".to_string(),
                items,
                None,
                None,
                false,
                CaseMatching::Smart,
            );
            assert!(!results.is_empty());
            assert!(results[0].item.contains("hello"));
        }
    }

    mod proptest_unicode {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn search_unicode_max_results_respected(
                query in "\\PC{1,5}",
                items in prop::collection::vec("\\PC{1,10}", 1..20),
                max in 1u32..10
            ) {
                let results = search_impl(query, items, Some(max), None, false, CaseMatching::Smart);
                prop_assert!(results.len() <= max as usize);
            }

            #[test]
            fn search_unicode_scores_sorted_descending(
                query in "\\PC{1,5}",
                items in prop::collection::vec("\\PC{1,10}", 1..20),
            ) {
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for window in results.windows(2) {
                    prop_assert!(
                        window[0].score >= window[1].score,
                        "results not sorted: {} < {}",
                        window[0].score,
                        window[1].score
                    );
                }
            }

            #[test]
            fn search_unicode_indices_valid(
                query in "\\PC{1,5}",
                items in prop::collection::vec("\\PC{1,10}", 1..20),
            ) {
                let len = items.len();
                let results = search_impl(query, items, None, None, false, CaseMatching::Smart);
                for r in &results {
                    prop_assert!((r.index as usize) < len, "index {} out of bounds (len={})", r.index, len);
                }
            }
        }
    }
}
