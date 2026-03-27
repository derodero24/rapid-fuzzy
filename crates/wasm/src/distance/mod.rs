use rapidfuzz::distance::damerau_levenshtein as rapid_damerau;
use rapidfuzz::distance::hamming as rapid_hamming;
use rapidfuzz::distance::indel as rapid_indel;
use rapidfuzz::distance::jaro as rapid_jaro;
use rapidfuzz::distance::jaro_winkler as rapid_jw;
use rapidfuzz::distance::levenshtein as rapid_lev;
use wasm_bindgen::prelude::*;

fn batch_apply<T: Default + serde::Serialize, F: Fn(&str, &str) -> T>(
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
    rapid_lev::distance(a.chars(), b.chars()) as u32
}

#[wasm_bindgen(js_name = "levenshteinBatch")]
pub fn levenshtein_batch(pairs: JsValue) -> Vec<u32> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_lev::distance(pair[0].chars(), pair[1].chars()) as u32
            } else {
                0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "levenshteinMany")]
pub fn levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    let scorer = rapid_lev::BatchComparator::new(reference.chars());
    match max_distance {
        Some(cutoff) => {
            let args = rapid_lev::Args::default().score_cutoff(cutoff as usize);
            let sentinel = cutoff + 1;
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .map_or(sentinel, |d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()) as u32)
            .collect(),
    }
}

// ─── Damerau-Levenshtein ────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "damerauLevenshtein")]
pub fn damerau_levenshtein(a: String, b: String) -> u32 {
    rapid_damerau::distance(a.chars(), b.chars()) as u32
}

#[wasm_bindgen(js_name = "damerauLevenshteinBatch")]
pub fn damerau_levenshtein_batch(pairs: JsValue) -> Vec<u32> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_damerau::distance(pair[0].chars(), pair[1].chars()) as u32
            } else {
                0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "damerauLevenshteinMany")]
pub fn damerau_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    let scorer = rapid_damerau::BatchComparator::new(reference.chars());
    match max_distance {
        Some(cutoff) => {
            let args = rapid_damerau::Args::default().score_cutoff(cutoff as usize);
            let sentinel = cutoff + 1;
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .map_or(sentinel, |d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()) as u32)
            .collect(),
    }
}

// ─── Hamming ─────────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "hamming")]
pub fn hamming(a: String, b: String) -> JsValue {
    match rapid_hamming::distance(a.chars(), b.chars()) {
        Ok(d) => JsValue::from_f64(d as f64),
        Err(_) => JsValue::NULL,
    }
}

#[wasm_bindgen(js_name = "hammingBatch")]
pub fn hamming_batch(pairs: JsValue) -> JsValue {
    batch_apply::<Option<u32>, _>(pairs, |a, b| {
        rapid_hamming::distance(a.chars(), b.chars())
            .ok()
            .map(|d| d as u32)
    })
}

#[wasm_bindgen(js_name = "hammingMany")]
pub fn hamming_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> JsValue {
    let scorer = rapid_hamming::BatchComparator::new(reference.chars());
    let results: Vec<Option<u32>> = match max_distance {
        Some(cutoff) => {
            let args = rapid_hamming::Args::default().score_cutoff(cutoff as usize);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .ok()
                        .flatten()
                        .map(|d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()).ok().map(|d| d as u32))
            .collect(),
    };
    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

// ─── Normalized Hamming ──────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "normalizedHamming")]
pub fn normalized_hamming(a: String, b: String) -> JsValue {
    match rapid_hamming::normalized_similarity(a.chars(), b.chars()) {
        Ok(s) => JsValue::from_f64(s),
        Err(_) => JsValue::NULL,
    }
}

#[wasm_bindgen(js_name = "normalizedHammingBatch")]
pub fn normalized_hamming_batch(pairs: JsValue) -> JsValue {
    batch_apply::<Option<f64>, _>(pairs, |a, b| {
        rapid_hamming::normalized_similarity(a.chars(), b.chars()).ok()
    })
}

#[wasm_bindgen(js_name = "normalizedHammingMany")]
pub fn normalized_hamming_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> JsValue {
    let scorer = rapid_hamming::BatchComparator::new(reference.chars());
    let results: Vec<Option<f64>> = match score_cutoff {
        Some(cutoff) => {
            let args = rapid_hamming::Args::default().score_cutoff(cutoff);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .normalized_similarity_with_args(c.chars(), &args)
                        .ok()
                        .flatten()
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.normalized_similarity(c.chars()).ok())
            .collect(),
    };
    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

// ─── Jaro ────────────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "jaro")]
pub fn jaro(a: String, b: String) -> f64 {
    rapid_jaro::normalized_similarity(a.chars(), b.chars())
}

#[wasm_bindgen(js_name = "jaroBatch")]
pub fn jaro_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_jaro::normalized_similarity(pair[0].chars(), pair[1].chars())
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "jaroMany")]
pub fn jaro_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let scorer = rapid_jaro::BatchComparator::new(reference.chars());
    match score_cutoff {
        Some(cutoff) => {
            let args = rapid_jaro::Args::default().score_cutoff(cutoff);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .normalized_similarity_with_args(c.chars(), &args)
                        .unwrap_or(0.0)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.normalized_similarity(c.chars()))
            .collect(),
    }
}

// ─── Jaro-Winkler ────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "jaroWinkler")]
pub fn jaro_winkler(a: String, b: String) -> f64 {
    rapid_jw::normalized_similarity(a.chars(), b.chars())
}

#[wasm_bindgen(js_name = "jaroWinklerBatch")]
pub fn jaro_winkler_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_jw::normalized_similarity(pair[0].chars(), pair[1].chars())
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "jaroWinklerMany")]
pub fn jaro_winkler_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let scorer = rapid_jw::BatchComparator::new(reference.chars());
    match score_cutoff {
        Some(cutoff) => {
            let args = rapid_jw::Args::default().score_cutoff(cutoff);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .normalized_similarity_with_args(c.chars(), &args)
                        .unwrap_or(0.0)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.normalized_similarity(c.chars()))
            .collect(),
    }
}

// ─── Sorensen-Dice ───────────────────────────────────────────────────────────

fn sorensen_dice_impl(a: &str, b: &str) -> f64 {
    strsim::sorensen_dice(a, b)
}

#[wasm_bindgen(js_name = "sorensenDice")]
pub fn sorensen_dice(a: String, b: String) -> f64 {
    sorensen_dice_impl(&a, &b)
}

#[wasm_bindgen(js_name = "sorensenDiceBatch")]
pub fn sorensen_dice_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                sorensen_dice_impl(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "sorensenDiceMany")]
pub fn sorensen_dice_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let threshold = score_cutoff.unwrap_or(0.0);
    candidates
        .iter()
        .map(|c| {
            let s = sorensen_dice_impl(&reference, c);
            if s >= threshold { s } else { 0.0 }
        })
        .collect()
}

// ─── Normalized Levenshtein ──────────────────────────────────────────────────

fn normalized_levenshtein_impl(a: &str, b: &str) -> f64 {
    strsim::normalized_levenshtein(a, b)
}

#[wasm_bindgen(js_name = "normalizedLevenshtein")]
pub fn normalized_levenshtein(a: String, b: String) -> f64 {
    normalized_levenshtein_impl(&a, &b)
}

#[wasm_bindgen(js_name = "normalizedLevenshteinBatch")]
pub fn normalized_levenshtein_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                normalized_levenshtein_impl(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "normalizedLevenshteinMany")]
pub fn normalized_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let threshold = score_cutoff.unwrap_or(0.0);
    candidates
        .iter()
        .map(|c| {
            let s = normalized_levenshtein_impl(&reference, c);
            if s >= threshold { s } else { 0.0 }
        })
        .collect()
}

// ─── Indel ───────────────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "indel")]
pub fn indel(a: String, b: String) -> u32 {
    rapid_indel::distance(a.chars(), b.chars()) as u32
}

#[wasm_bindgen(js_name = "indelBatch")]
pub fn indel_batch(pairs: JsValue) -> Vec<u32> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_indel::distance(pair[0].chars(), pair[1].chars()) as u32
            } else {
                0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "indelMany")]
pub fn indel_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    let scorer = rapid_indel::BatchComparator::new(reference.chars());
    match max_distance {
        Some(cutoff) => {
            let args = rapid_indel::Args::default().score_cutoff(cutoff as usize);
            let sentinel = cutoff + 1;
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .map_or(sentinel, |d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()) as u32)
            .collect(),
    }
}

// ─── Normalized Indel ────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = "normalizedIndel")]
pub fn normalized_indel(a: String, b: String) -> f64 {
    rapid_indel::normalized_similarity(a.chars(), b.chars())
}

#[wasm_bindgen(js_name = "normalizedIndelBatch")]
pub fn normalized_indel_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_indel::normalized_similarity(pair[0].chars(), pair[1].chars())
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "normalizedIndelMany")]
pub fn normalized_indel_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let scorer = rapid_indel::BatchComparator::new(reference.chars());
    match score_cutoff {
        Some(cutoff) => {
            let args = rapid_indel::Args::default().score_cutoff(cutoff);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .normalized_similarity_with_args(c.chars(), &args)
                        .unwrap_or(0.0)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.normalized_similarity(c.chars()))
            .collect(),
    }
}

// ─── Token Sort Ratio ────────────────────────────────────────────────────────

fn token_sort_ratio_impl(a: &str, b: &str) -> f64 {
    let mut a_tokens: Vec<&str> = a.split_whitespace().collect();
    let mut b_tokens: Vec<&str> = b.split_whitespace().collect();
    a_tokens.sort_unstable();
    b_tokens.sort_unstable();
    let a_sorted = a_tokens.join(" ");
    let b_sorted = b_tokens.join(" ");
    strsim::normalized_levenshtein(&a_sorted, &b_sorted)
}

#[wasm_bindgen(js_name = "tokenSortRatio")]
pub fn token_sort_ratio(a: String, b: String) -> f64 {
    token_sort_ratio_impl(&a, &b)
}

#[wasm_bindgen(js_name = "tokenSortRatioBatch")]
pub fn token_sort_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                token_sort_ratio_impl(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "tokenSortRatioMany")]
pub fn token_sort_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let threshold = score_cutoff.unwrap_or(0.0);
    let mut ref_tokens: Vec<&str> = reference.split_whitespace().collect();
    ref_tokens.sort_unstable();
    let ref_sorted = ref_tokens.join(" ");
    candidates
        .iter()
        .map(|c| {
            let mut c_tokens: Vec<&str> = c.split_whitespace().collect();
            c_tokens.sort_unstable();
            let c_sorted = c_tokens.join(" ");
            let s = strsim::normalized_levenshtein(&ref_sorted, &c_sorted);
            if s >= threshold { s } else { 0.0 }
        })
        .collect()
}

// ─── Token Set Ratio ─────────────────────────────────────────────────────────

fn token_set_ratio_impl(a: &str, b: &str) -> f64 {
    use std::collections::BTreeSet;
    let a_set: BTreeSet<&str> = a.split_whitespace().collect();
    let b_set: BTreeSet<&str> = b.split_whitespace().collect();

    let intersection: Vec<&&str> = a_set.intersection(&b_set).collect();
    if intersection.is_empty() {
        return 0.0;
    }

    let mut intersection_sorted: Vec<&str> = intersection.iter().map(|&&s| s).collect();
    intersection_sorted.sort_unstable();
    let base = intersection_sorted.join(" ");

    let mut a_diff: Vec<&str> = a_set.difference(&b_set).copied().collect();
    a_diff.sort_unstable();
    let mut b_diff: Vec<&str> = b_set.difference(&a_set).copied().collect();
    b_diff.sort_unstable();

    let a_rest = a_diff.join(" ");
    let b_rest = b_diff.join(" ");

    let s1 = if a_rest.is_empty() {
        base.clone()
    } else {
        format!("{base} {a_rest}")
    };
    let s2 = if b_rest.is_empty() {
        base.clone()
    } else {
        format!("{base} {b_rest}")
    };

    strsim::normalized_levenshtein(&s1, &s2)
        .max(strsim::normalized_levenshtein(&base, &s1))
        .max(strsim::normalized_levenshtein(&base, &s2))
}

#[wasm_bindgen(js_name = "tokenSetRatio")]
pub fn token_set_ratio(a: String, b: String) -> f64 {
    token_set_ratio_impl(&a, &b)
}

#[wasm_bindgen(js_name = "tokenSetRatioBatch")]
pub fn token_set_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                token_set_ratio_impl(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "tokenSetRatioMany")]
pub fn token_set_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let threshold = score_cutoff.unwrap_or(0.0);
    candidates
        .iter()
        .map(|c| {
            let s = token_set_ratio_impl(&reference, c);
            if s >= threshold { s } else { 0.0 }
        })
        .collect()
}

// ─── Partial Ratio ───────────────────────────────────────────────────────────

fn partial_ratio_impl(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return if a == b { 1.0 } else { 0.0 };
    }
    let (shorter, longer) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let window = shorter.len();
    (0..=(longer.len().saturating_sub(window)))
        .map(|i| strsim::normalized_levenshtein(shorter, &longer[i..i + window]))
        .fold(0.0f64, f64::max)
}

#[wasm_bindgen(js_name = "partialRatio")]
pub fn partial_ratio(a: String, b: String) -> f64 {
    partial_ratio_impl(&a, &b)
}

#[wasm_bindgen(js_name = "partialRatioBatch")]
pub fn partial_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                partial_ratio_impl(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "partialRatioMany")]
pub fn partial_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let threshold = score_cutoff.unwrap_or(0.0);
    candidates
        .iter()
        .map(|c| {
            let s = partial_ratio_impl(&reference, c);
            if s >= threshold { s } else { 0.0 }
        })
        .collect()
}

// ─── Weighted Ratio ──────────────────────────────────────────────────────────

fn weighted_ratio_impl(a: &str, b: &str) -> f64 {
    let base = strsim::normalized_levenshtein(a, b);
    let partial = partial_ratio_impl(a, b);
    let token_sort = token_sort_ratio_impl(a, b);
    let token_set = token_set_ratio_impl(a, b);
    base.max(partial).max(token_sort).max(token_set)
}

#[wasm_bindgen(js_name = "weightedRatio")]
pub fn weighted_ratio(a: String, b: String) -> f64 {
    weighted_ratio_impl(&a, &b)
}

#[wasm_bindgen(js_name = "weightedRatioBatch")]
pub fn weighted_ratio_batch(pairs: JsValue) -> Vec<f64> {
    let pairs: Vec<Vec<String>> = match serde_wasm_bindgen::from_value(pairs) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                weighted_ratio_impl(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

#[wasm_bindgen(js_name = "weightedRatioMany")]
pub fn weighted_ratio_many(
    reference: String,
    candidates: Vec<String>,
    score_cutoff: Option<f64>,
) -> Vec<f64> {
    let threshold = score_cutoff.unwrap_or(0.0);
    candidates
        .iter()
        .map(|c| {
            let s = weighted_ratio_impl(&reference, c);
            if s >= threshold { s } else { 0.0 }
        })
        .collect()
}
