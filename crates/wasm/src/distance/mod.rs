//! String distance and similarity functions.
//!
//! Mirrors the Node.js binding (`crates/core/src/distance`): same algorithms,
//! parameter names and `null` results. Fixed-width results are returned as
//! typed arrays (`Uint32Array` / `Float64Array`).

use rapid_fuzzy_core::distance as core_dist;
use rapid_fuzzy_core::distance::DistanceError;
use wasm_bindgen::prelude::*;

use crate::convert::{error, nullable_array, or_null, string_matrix_from_js};

/// Malformed pairs and `NaN` thresholds throw an `Error`.
#[allow(clippy::needless_pass_by_value)]
fn dist_error(err: DistanceError) -> JsValue {
    error(&err.to_string())
}

/// Compute the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string
/// into the other.
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "levenshtein")]
pub fn levenshtein(a: String, b: String) -> u32 {
    core_dist::levenshtein(&a, &b)
}

/// Compute the Levenshtein distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
/// Each pair must be an array of exactly two strings `[a, b]`.
#[wasm_bindgen(js_name = "levenshteinBatch")]
pub fn levenshtein_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<u32>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::levenshtein_batch(&pairs).map_err(dist_error)
}

/// Compute the Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// If `maxDistance` is provided, candidates with distance exceeding the threshold
/// will return `maxDistance + 1` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "levenshteinMany")]
pub fn levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "maxDistance")] max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::levenshtein_many(&reference, &candidates, max_distance)
}

/// Compute the Damerau-Levenshtein distance between two strings.
///
/// Like Levenshtein, but also considers transpositions of two adjacent
/// characters as a single edit.
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
/// Takes time proportional to the product of the two lengths: about 0.4 s
/// for two 10,000-character strings with the native addon.
#[wasm_bindgen(js_name = "damerauLevenshtein")]
pub fn damerau_levenshtein(a: String, b: String) -> u32 {
    core_dist::damerau_levenshtein(&a, &b)
}

/// Compute the Damerau-Levenshtein distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
#[wasm_bindgen(js_name = "damerauLevenshteinBatch")]
pub fn damerau_levenshtein_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<u32>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::damerau_levenshtein_batch(&pairs).map_err(dist_error)
}

/// Compute the Damerau-Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// If `maxDistance` is provided, candidates with distance exceeding the threshold
/// will return `maxDistance + 1` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "damerauLevenshteinMany")]
pub fn damerau_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "maxDistance")] max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::damerau_levenshtein_many(&reference, &candidates, max_distance)
}

/// Compute the Hamming distance between two strings.
///
/// The Hamming distance counts the number of positions at which the corresponding
/// characters differ. It is only defined for strings of equal length.
/// Returns `null` if the strings have different lengths.
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "hamming", unchecked_return_type = "number | null")]
pub fn hamming(a: String, b: String) -> JsValue {
    or_null(core_dist::hamming(&a, &b))
}

/// Compute the Hamming distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
/// Each pair must be an array of exactly two strings `[a, b]`.
/// Returns `null` for pairs with different lengths.
#[wasm_bindgen(js_name = "hammingBatch", unchecked_return_type = "(number | null)[]")]
pub fn hamming_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<js_sys::Array, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    Ok(nullable_array(
        &core_dist::hamming_batch(&pairs).map_err(dist_error)?,
    ))
}

/// Compute the Hamming distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// Returns `null` for candidates with a different length than the reference.
/// If `maxDistance` is provided, candidates with distance exceeding the threshold
/// will also return `null` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "hammingMany", unchecked_return_type = "(number | null)[]")]
pub fn hamming_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "maxDistance")] max_distance: Option<u32>,
) -> js_sys::Array {
    nullable_array(&core_dist::hamming_many(
        &reference,
        &candidates,
        max_distance,
    ))
}

/// Compute the normalized Hamming similarity between two strings.
///
/// Returns `null` if the strings have different lengths.
/// Returns a value between 0.0 (no matching characters) and 1.0 (identical).
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "normalizedHamming", unchecked_return_type = "number | null")]
pub fn normalized_hamming(a: String, b: String) -> JsValue {
    or_null(core_dist::normalized_hamming(&a, &b))
}

/// Compute the normalized Hamming similarity for multiple pairs of strings in a single call.
///
/// Returns an array of scores in the same order as the input pairs.
/// Returns `null` for pairs with different lengths.
#[wasm_bindgen(
    js_name = "normalizedHammingBatch",
    unchecked_return_type = "(number | null)[]"
)]
pub fn normalized_hamming_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<js_sys::Array, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    Ok(nullable_array(
        &core_dist::normalized_hamming_batch(&pairs).map_err(dist_error)?,
    ))
}

/// Compute the normalized Hamming similarity from one reference string to many candidates.
///
/// Returns an array of scores, one per candidate, in the same order as the input.
/// Returns `null` for candidates with a different length than the reference.
/// If `minSimilarity` is provided, candidates with similarity below the threshold
/// will also return `null` (enabling early termination for better performance).
#[wasm_bindgen(
    js_name = "normalizedHammingMany",
    unchecked_return_type = "(number | null)[]"
)]
pub fn normalized_hamming_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<js_sys::Array, JsValue> {
    Ok(nullable_array(
        &core_dist::normalized_hamming_many(&reference, &candidates, min_similarity)
            .map_err(dist_error)?,
    ))
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "jaro")]
pub fn jaro(a: String, b: String) -> f64 {
    core_dist::jaro(&a, &b)
}

/// Compute the Jaro similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "jaroBatch")]
pub fn jaro_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::jaro_batch(&pairs).map_err(dist_error)
}

/// Compute the Jaro similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "jaroMany")]
pub fn jaro_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::jaro_many(&reference, &candidates, min_similarity).map_err(dist_error)
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// A modification of Jaro that gives more weight to common prefixes.
/// Returns a value between 0.0 and 1.0.
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "jaroWinkler")]
pub fn jaro_winkler(a: String, b: String) -> f64 {
    core_dist::jaro_winkler(&a, &b)
}

/// Compute the Jaro-Winkler similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "jaroWinklerBatch")]
pub fn jaro_winkler_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::jaro_winkler_batch(&pairs).map_err(dist_error)
}

/// Compute the Jaro-Winkler similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "jaroWinklerMany")]
pub fn jaro_winkler_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::jaro_winkler_many(&reference, &candidates, min_similarity).map_err(dist_error)
}

/// Compute the Sorensen-Dice coefficient between two strings.
///
/// Compares the bigrams (pairs of consecutive characters) of the two strings
/// after removing all whitespace; case and Unicode normalization are not
/// adjusted. Identical strings score 1.0; otherwise a string with fewer than
/// two characters left scores 0.0. Returns a value between 0.0 and 1.0.
#[wasm_bindgen(js_name = "sorensenDice")]
pub fn sorensen_dice(a: String, b: String) -> f64 {
    core_dist::sorensen_dice(&a, &b)
}

/// Compute the Sorensen-Dice coefficient for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "sorensenDiceBatch")]
pub fn sorensen_dice_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::sorensen_dice_batch(&pairs).map_err(dist_error)
}

/// Compute the Sorensen-Dice coefficient from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
/// Reference bigrams are pre-computed once and reused for all candidates.
#[wasm_bindgen(js_name = "sorensenDiceMany")]
pub fn sorensen_dice_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::sorensen_dice_many(&reference, &candidates, min_similarity).map_err(dist_error)
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// `1 - levenshtein(a, b) / max(length of a, length of b)`, counted in
/// characters. Returns a value between 0.0 (completely different) and 1.0
/// (identical).
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "normalizedLevenshtein")]
pub fn normalized_levenshtein(a: String, b: String) -> f64 {
    core_dist::normalized_levenshtein(&a, &b)
}

/// Compute the normalized Levenshtein similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "normalizedLevenshteinBatch")]
pub fn normalized_levenshtein_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::normalized_levenshtein_batch(&pairs).map_err(dist_error)
}

/// Compute the normalized Levenshtein similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "normalizedLevenshteinMany")]
pub fn normalized_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::normalized_levenshtein_many(&reference, &candidates, min_similarity)
        .map_err(dist_error)
}

/// Compute the Indel distance between two strings.
///
/// The Indel distance counts the minimum number of insertions and deletions
/// (no substitutions) required to transform one string into the other.
/// It equals `len(a) + len(b) - 2 * LCS_length(a, b)`.
///
/// Useful when substitutions are semantically two operations (one deletion +
/// one insertion), such as in DNA sequence alignment.
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "indel")]
pub fn indel(a: String, b: String) -> u32 {
    core_dist::indel(&a, &b)
}

/// Compute the Indel distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
/// Each pair must be an array of exactly two strings `[a, b]`.
#[wasm_bindgen(js_name = "indelBatch")]
pub fn indel_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<u32>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::indel_batch(&pairs).map_err(dist_error)
}

/// Compute the Indel distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// If `maxDistance` is provided, candidates with distance exceeding the threshold
/// will return `maxDistance + 1` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "indelMany")]
pub fn indel_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "maxDistance")] max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::indel_many(&reference, &candidates, max_distance)
}

/// Compute the normalized Indel similarity between two strings.
///
/// `1 - indel(a, b) / (length of a + length of b)`, counted in characters:
/// the measure behind `fuzz.ratio` in RapidFuzz and fuzzball, on a 0.0-1.0
/// scale (fuzzball also lower-cases and strips punctuation by default; this
/// function does not). Returns a value between 0.0 (completely different)
/// and 1.0 (identical).
///
/// Compares the Unicode code points of the strings as given: case, whitespace
/// and Unicode normalization (NFC vs NFD) are not adjusted.
#[wasm_bindgen(js_name = "normalizedIndel")]
pub fn normalized_indel(a: String, b: String) -> f64 {
    core_dist::normalized_indel(&a, &b)
}

/// Compute the normalized Indel similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "normalizedIndelBatch")]
pub fn normalized_indel_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::normalized_indel_batch(&pairs).map_err(dist_error)
}

/// Compute the normalized Indel similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[wasm_bindgen(js_name = "normalizedIndelMany")]
pub fn normalized_indel_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::normalized_indel_many(&reference, &candidates, min_similarity).map_err(dist_error)
}

/// Compute the token sort ratio between two strings.
///
/// Lower-cases both strings, splits them on whitespace, sorts the tokens and
/// joins them with single spaces, then returns the normalized Levenshtein
/// similarity of the two results, so word order does not matter. Punctuation
/// is kept: `Smith,` and `Smith` are different tokens.
/// Returns a value between 0.0 (completely different) and 1.0 (identical after sorting).
#[wasm_bindgen(js_name = "tokenSortRatio")]
pub fn token_sort_ratio(a: String, b: String) -> f64 {
    core_dist::token_sort_ratio(&a, &b)
}

/// Compute the token sort ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "tokenSortRatioBatch")]
pub fn token_sort_ratio_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::token_sort_ratio_batch(&pairs).map_err(dist_error)
}

/// Compute the token sort ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
#[wasm_bindgen(js_name = "tokenSortRatioMany")]
pub fn token_sort_ratio_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::token_sort_ratio_many(&reference, &candidates, min_similarity).map_err(dist_error)
}

/// Compute the token set ratio between two strings.
///
/// Lower-cases both strings and splits them into sets of whitespace-separated
/// tokens (duplicates count once). With the shared tokens sorted and joined
/// as `common`, returns the highest normalized Levenshtein similarity among
/// `common + rest of a` vs `common + rest of b`, `common` vs
/// `common + rest of a`, and `common` vs `common + rest of b`. It is 1.0
/// when the tokens of one string are a subset of the other's.
/// Returns a value between 0.0 and 1.0.
#[wasm_bindgen(js_name = "tokenSetRatio")]
pub fn token_set_ratio(a: String, b: String) -> f64 {
    core_dist::token_set_ratio(&a, &b)
}

/// Compute the token set ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "tokenSetRatioBatch")]
pub fn token_set_ratio_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::token_set_ratio_batch(&pairs).map_err(dist_error)
}

/// Compute the token set ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
#[wasm_bindgen(js_name = "tokenSetRatioMany")]
pub fn token_set_ratio_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::token_set_ratio_many(&reference, &candidates, min_similarity).map_err(dist_error)
}

/// Compute the partial ratio between two strings.
///
/// Lower-cases both strings and collapses whitespace runs into single
/// spaces, then compares the shorter string with every window of the same
/// length in the longer one and returns the highest normalized Levenshtein
/// similarity. Useful when one string is a substring or truncation of the
/// other; it does not match abbreviations (`MSFT` vs `Microsoft` scores low).
/// Scores can differ from fuzzball's / RapidFuzz's `partial_ratio`, which
/// use a different alignment. Returns a value between 0.0 and 1.0.
///
/// Takes time proportional to the length of the longer string times the
/// square of the length of the shorter one: about 0.4 s for a
/// 1,000-character string against a 10,000-character one with the native
/// addon.
#[wasm_bindgen(js_name = "partialRatio")]
pub fn partial_ratio(a: String, b: String) -> f64 {
    core_dist::partial_ratio(&a, &b)
}

/// Compute the partial ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "partialRatioBatch")]
pub fn partial_ratio_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::partial_ratio_batch(&pairs).map_err(dist_error)
}

/// Compute the partial ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
#[wasm_bindgen(js_name = "partialRatioMany")]
pub fn partial_ratio_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::partial_ratio_many(&reference, &candidates, min_similarity).map_err(dist_error)
}

/// Compute the weighted ratio between two strings.
///
/// Returns the highest of: the normalized Levenshtein similarity of the
/// strings as given and after lower-casing and collapsing whitespace,
/// `tokenSortRatio`, `tokenSetRatio` and `partialRatio`. Unlike `WRatio` in
/// fuzzball / RapidFuzz, no score is scaled down or weighted by the length
/// ratio of the strings, so scores are often higher than `WRatio`'s.
/// Includes the cost of `partialRatio` (see there) when the strings differ
/// in length. Returns a value between 0.0 and 1.0.
#[wasm_bindgen(js_name = "weightedRatio")]
pub fn weighted_ratio(a: String, b: String) -> f64 {
    core_dist::weighted_ratio(&a, &b)
}

/// Compute the weighted ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[wasm_bindgen(js_name = "weightedRatioBatch")]
pub fn weighted_ratio_batch(
    #[wasm_bindgen(unchecked_param_type = "string[][]")] pairs: JsValue,
) -> Result<Vec<f64>, JsValue> {
    let pairs = string_matrix_from_js(pairs, "pairs")?;
    core_dist::weighted_ratio_batch(&pairs).map_err(dist_error)
}

/// Compute the weighted ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
#[wasm_bindgen(js_name = "weightedRatioMany")]
pub fn weighted_ratio_many(
    reference: String,
    candidates: Vec<String>,
    #[wasm_bindgen(js_name = "minSimilarity")] min_similarity: Option<f64>,
) -> Result<Vec<f64>, JsValue> {
    core_dist::weighted_ratio_many(&reference, &candidates, min_similarity).map_err(dist_error)
}
