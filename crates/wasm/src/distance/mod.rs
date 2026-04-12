use rapid_fuzzy_core::distance as core_dist;
use wasm_bindgen::prelude::*;

fn batch_apply_wasm<T: Default + serde::Serialize, F: Fn(&str, &str) -> T>(
    pairs: JsValue,
    f: F,
) -> JsValue {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return JsValue::from(js_sys::Array::new()),
    };
    let results: Vec<T> = pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                f(&pair[0], &pair[1])
            } else {
                T::default()
            }
        })
        .collect();
    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

// ─── Levenshtein ────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "levenshtein")]
pub fn levenshtein(a: String, b: String) -> u32 {
    core_dist::levenshtein(&a, &b)
}

#[wasm_bindgen(js_name = "levenshteinBatch")]
pub fn levenshtein_batch(pairs: JsValue) -> Vec<u32> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::levenshtein_batch(&pairs)
}

#[wasm_bindgen(js_name = "levenshteinMany")]
pub fn levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::levenshtein_many(&reference, &candidates, max_distance)
}

// ─── Damerau-Levenshtein ────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "damerauLevenshtein")]
pub fn damerau_levenshtein(a: String, b: String) -> u32 {
    core_dist::damerau_levenshtein(&a, &b)
}

#[wasm_bindgen(js_name = "damerauLevenshteinBatch")]
pub fn damerau_levenshtein_batch(pairs: JsValue) -> Vec<u32> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::damerau_levenshtein_batch(&pairs)
}

#[wasm_bindgen(js_name = "damerauLevenshteinMany")]
pub fn damerau_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::damerau_levenshtein_many(&reference, &candidates, max_distance)
}

// ─── Hamming ─────────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "hamming")]
pub fn hamming(a: String, b: String) -> JsValue {
    match core_dist::hamming(&a, &b) {
        Some(d) => JsValue::from_f64(d as f64),
        None => JsValue::NULL,
    }
}

#[wasm_bindgen(js_name = "hammingBatch")]
pub fn hamming_batch(pairs: JsValue) -> JsValue {
    batch_apply_wasm::<Option<u32>, _>(pairs, core_dist::hamming)
}

#[wasm_bindgen(js_name = "hammingMany")]
pub fn hamming_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> JsValue {
    let results = core_dist::hamming_many(&reference, &candidates, max_distance);
    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

// ─── Normalized Hamming ──────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "normalizedHamming")]
pub fn normalized_hamming(a: String, b: String) -> JsValue {
    match core_dist::normalized_hamming(&a, &b) {
        Some(s) => JsValue::from_f64(s),
        None => JsValue::NULL,
    }
}

#[wasm_bindgen(js_name = "normalizedHammingBatch")]
pub fn normalized_hamming_batch(pairs: JsValue) -> JsValue {
    batch_apply_wasm::<Option<f64>, _>(pairs, core_dist::normalized_hamming)
}

#[wasm_bindgen(js_name = "normalizedHammingMany")]
pub fn normalized_hamming_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> JsValue {
    let results = core_dist::normalized_hamming_many(&reference, &candidates, score_cutoff);
    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

// ─── Jaro ────────────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "jaro")]
pub fn jaro(a: String, b: String) -> f64 {
    core_dist::jaro(&a, &b)
}

#[wasm_bindgen(js_name = "jaroBatch")]
pub fn jaro_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::jaro_batch(&pairs)
}

#[wasm_bindgen(js_name = "jaroMany")]
pub fn jaro_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::jaro_many(&reference, &candidates, score_cutoff)
}

// ─── Jaro-Winkler ────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "jaroWinkler")]
pub fn jaro_winkler(a: String, b: String) -> f64 {
    core_dist::jaro_winkler(&a, &b)
}

#[wasm_bindgen(js_name = "jaroWinklerBatch")]
pub fn jaro_winkler_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::jaro_winkler_batch(&pairs)
}

#[wasm_bindgen(js_name = "jaroWinklerMany")]
pub fn jaro_winkler_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::jaro_winkler_many(&reference, &candidates, score_cutoff)
}

// ─── Sorensen-Dice ───────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "sorensenDice")]
pub fn sorensen_dice(a: String, b: String) -> f64 {
    core_dist::sorensen_dice(&a, &b)
}

#[wasm_bindgen(js_name = "sorensenDiceBatch")]
pub fn sorensen_dice_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::sorensen_dice_batch(&pairs)
}

#[wasm_bindgen(js_name = "sorensenDiceMany")]
pub fn sorensen_dice_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::sorensen_dice_many(&reference, &candidates, score_cutoff)
}

// ─── Normalized Levenshtein ──────────────────────────────────────────────────

#[wasm_bindgen(js_name = "normalizedLevenshtein")]
pub fn normalized_levenshtein(a: String, b: String) -> f64 {
    core_dist::normalized_levenshtein(&a, &b)
}

#[wasm_bindgen(js_name = "normalizedLevenshteinBatch")]
pub fn normalized_levenshtein_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::normalized_levenshtein_batch(&pairs)
}

#[wasm_bindgen(js_name = "normalizedLevenshteinMany")]
pub fn normalized_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::normalized_levenshtein_many(&reference, &candidates, score_cutoff)
}

// ─── Indel ───────────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "indel")]
pub fn indel(a: String, b: String) -> u32 {
    core_dist::indel(&a, &b)
}

#[wasm_bindgen(js_name = "indelBatch")]
pub fn indel_batch(pairs: JsValue) -> Vec<u32> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::indel_batch(&pairs)
}

#[wasm_bindgen(js_name = "indelMany")]
pub fn indel_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::indel_many(&reference, &candidates, max_distance)
}

// ─── Normalized Indel ────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "normalizedIndel")]
pub fn normalized_indel(a: String, b: String) -> f64 {
    core_dist::normalized_indel(&a, &b)
}

#[wasm_bindgen(js_name = "normalizedIndelBatch")]
pub fn normalized_indel_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::normalized_indel_batch(&pairs)
}

#[wasm_bindgen(js_name = "normalizedIndelMany")]
pub fn normalized_indel_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::normalized_indel_many(&reference, &candidates, score_cutoff)
}

// ─── Token Sort Ratio ────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "tokenSortRatio")]
pub fn token_sort_ratio(a: String, b: String) -> f64 {
    core_dist::token_sort_ratio(&a, &b)
}

#[wasm_bindgen(js_name = "tokenSortRatioBatch")]
pub fn token_sort_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::token_sort_ratio_batch(&pairs)
}

#[wasm_bindgen(js_name = "tokenSortRatioMany")]
pub fn token_sort_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::token_sort_ratio_many(&reference, &candidates, score_cutoff)
}

// ─── Token Set Ratio ─────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "tokenSetRatio")]
pub fn token_set_ratio(a: String, b: String) -> f64 {
    core_dist::token_set_ratio(&a, &b)
}

#[wasm_bindgen(js_name = "tokenSetRatioBatch")]
pub fn token_set_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::token_set_ratio_batch(&pairs)
}

#[wasm_bindgen(js_name = "tokenSetRatioMany")]
pub fn token_set_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::token_set_ratio_many(&reference, &candidates, score_cutoff)
}

// ─── Partial Ratio ───────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "partialRatio")]
pub fn partial_ratio(a: String, b: String) -> f64 {
    core_dist::partial_ratio(&a, &b)
}

#[wasm_bindgen(js_name = "partialRatioBatch")]
pub fn partial_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::partial_ratio_batch(&pairs)
}

#[wasm_bindgen(js_name = "partialRatioMany")]
pub fn partial_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::partial_ratio_many(&reference, &candidates, score_cutoff)
}

// ─── Weighted Ratio ──────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "weightedRatio")]
pub fn weighted_ratio(a: String, b: String) -> f64 {
    core_dist::weighted_ratio(&a, &b)
}

#[wasm_bindgen(js_name = "weightedRatioBatch")]
pub fn weighted_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    core_dist::weighted_ratio_batch(&pairs)
}

#[wasm_bindgen(js_name = "weightedRatioMany")]
pub fn weighted_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    core_dist::weighted_ratio_many(&reference, &candidates, score_cutoff)
}
