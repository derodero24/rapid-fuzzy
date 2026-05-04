use std::collections::BTreeSet;
use std::collections::HashMap;

use rapidfuzz::distance::damerau_levenshtein as rapid_damerau;
use rapidfuzz::distance::hamming as rapid_hamming;
use rapidfuzz::distance::indel as rapid_indel;
use rapidfuzz::distance::jaro as rapid_jaro;
use rapidfuzz::distance::jaro_winkler as rapid_jw;
use rapidfuzz::distance::levenshtein as rapid_lev;

pub fn batch_apply<T: Default, F: Fn(&str, &str) -> T>(pairs: &[Vec<String>], f: F) -> Vec<T> {
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                f(&pair[0], &pair[1])
            } else {
                T::default()
            }
        })
        .collect()
}

// ─── Levenshtein ────────────────────────────────────────────────────────────

pub fn levenshtein(a: &str, b: &str) -> u32 {
    rapid_lev::distance(a.chars(), b.chars()) as u32
}

pub fn levenshtein_batch(pairs: &[Vec<String>]) -> Vec<u32> {
    batch_apply(pairs, |a, b| {
        rapid_lev::distance(a.chars(), b.chars()) as u32
    })
}

pub fn levenshtein_many(
    reference: &str,
    candidates: &[String],
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

pub fn damerau_levenshtein(a: &str, b: &str) -> u32 {
    rapid_damerau::distance(a.chars(), b.chars()) as u32
}

pub fn damerau_levenshtein_batch(pairs: &[Vec<String>]) -> Vec<u32> {
    batch_apply(pairs, |a, b| {
        rapid_damerau::distance(a.chars(), b.chars()) as u32
    })
}

pub fn damerau_levenshtein_many(
    reference: &str,
    candidates: &[String],
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

pub fn hamming(a: &str, b: &str) -> Option<u32> {
    rapid_hamming::distance(a.chars(), b.chars())
        .ok()
        .map(|d| d as u32)
}

pub fn hamming_batch(pairs: &[Vec<String>]) -> Vec<Option<u32>> {
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_hamming::distance(pair[0].chars(), pair[1].chars())
                    .ok()
                    .map(|d| d as u32)
            } else {
                None
            }
        })
        .collect()
}

pub fn hamming_many(
    reference: &str,
    candidates: &[String],
    max_distance: Option<u32>,
) -> Vec<Option<u32>> {
    let scorer = rapid_hamming::BatchComparator::new(reference.chars());
    match max_distance {
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
    }
}

// ─── Normalized Hamming ──────────────────────────────────────────────────────

pub fn normalized_hamming(a: &str, b: &str) -> Option<f64> {
    rapid_hamming::normalized_similarity(a.chars(), b.chars()).ok()
}

pub fn normalized_hamming_batch(pairs: &[Vec<String>]) -> Vec<Option<f64>> {
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                rapid_hamming::normalized_similarity(pair[0].chars(), pair[1].chars()).ok()
            } else {
                None
            }
        })
        .collect()
}

pub fn normalized_hamming_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<Option<f64>> {
    candidates
        .iter()
        .map(|c| {
            let score = rapid_hamming::normalized_similarity(reference.chars(), c.chars()).ok()?;
            match min_similarity {
                Some(cutoff) if score < cutoff => None,
                _ => Some(score),
            }
        })
        .collect()
}

// ─── Jaro ────────────────────────────────────────────────────────────────────

pub fn jaro(a: &str, b: &str) -> f64 {
    rapid_jaro::similarity(a.chars(), b.chars())
}

pub fn jaro_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, |a, b| rapid_jaro::similarity(a.chars(), b.chars()))
}

pub fn jaro_many(reference: &str, candidates: &[String], min_similarity: Option<f64>) -> Vec<f64> {
    let scorer = rapid_jaro::BatchComparator::new(reference.chars());
    match min_similarity {
        Some(cutoff) => {
            let args = rapid_jaro::Args::default().score_cutoff(cutoff);
            candidates
                .iter()
                .map(|c| scorer.similarity_with_args(c.chars(), &args).unwrap_or(0.0))
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.similarity(c.chars()))
            .collect(),
    }
}

// ─── Jaro-Winkler ────────────────────────────────────────────────────────────

pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    rapid_jw::similarity(a.chars(), b.chars())
}

pub fn jaro_winkler_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, |a, b| rapid_jw::similarity(a.chars(), b.chars()))
}

pub fn jaro_winkler_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let scorer = rapid_jw::BatchComparator::new(reference.chars());
    match min_similarity {
        Some(cutoff) => {
            let args = rapid_jw::Args::default().score_cutoff(cutoff);
            candidates
                .iter()
                .map(|c| scorer.similarity_with_args(c.chars(), &args).unwrap_or(0.0))
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.similarity(c.chars()))
            .collect(),
    }
}

// ─── Sorensen-Dice ───────────────────────────────────────────────────────────

pub fn sorensen_dice(a: &str, b: &str) -> f64 {
    strsim::sorensen_dice(a, b)
}

pub fn sorensen_dice_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, strsim::sorensen_dice)
}

pub fn sorensen_dice_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let ref_chars: Vec<char> = reference.chars().collect();
    let ref_len = ref_chars.len();
    let ref_count = ref_len.saturating_sub(1);

    let mut ref_bigrams: HashMap<(char, char), usize> = HashMap::new();
    for i in 0..ref_count {
        *ref_bigrams
            .entry((ref_chars[i], ref_chars[i + 1]))
            .or_insert(0) += 1;
    }

    candidates
        .iter()
        .map(|c| {
            let c_chars: Vec<char> = c.chars().collect();
            let c_len = c_chars.len();
            let c_count = c_len.saturating_sub(1);

            let score = if ref_len + c_len == 0 {
                1.0
            } else if ref_count + c_count == 0 {
                if ref_len == c_len { 1.0 } else { 0.0 }
            } else {
                let mut c_bigrams: HashMap<(char, char), usize> = HashMap::new();
                for i in 0..c_count {
                    *c_bigrams.entry((c_chars[i], c_chars[i + 1])).or_insert(0) += 1;
                }
                let mut intersection = 0_usize;
                for (bigram, count) in &ref_bigrams {
                    intersection += count.min(c_bigrams.get(bigram).unwrap_or(&0));
                }
                (2 * intersection) as f64 / (ref_count + c_count) as f64
            };

            match min_similarity {
                Some(cutoff) if score < cutoff => 0.0,
                _ => score,
            }
        })
        .collect()
}

// ─── Normalized Levenshtein ──────────────────────────────────────────────────

pub fn normalized_levenshtein(a: &str, b: &str) -> f64 {
    rapid_lev::normalized_similarity(a.chars(), b.chars())
}

pub fn normalized_levenshtein_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, |a, b| {
        rapid_lev::normalized_similarity(a.chars(), b.chars())
    })
}

pub fn normalized_levenshtein_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let scorer = rapid_lev::BatchComparator::new(reference.chars());
    match min_similarity {
        Some(cutoff) => {
            let args = rapid_lev::Args::default().score_cutoff(cutoff);
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

// ─── Indel ───────────────────────────────────────────────────────────────────

pub fn indel(a: &str, b: &str) -> u32 {
    rapid_indel::distance(a.chars(), b.chars()) as u32
}

pub fn indel_batch(pairs: &[Vec<String>]) -> Vec<u32> {
    batch_apply(pairs, |a, b| {
        rapid_indel::distance(a.chars(), b.chars()) as u32
    })
}

pub fn indel_many(reference: &str, candidates: &[String], max_distance: Option<u32>) -> Vec<u32> {
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

pub fn normalized_indel(a: &str, b: &str) -> f64 {
    rapid_indel::normalized_similarity(a.chars(), b.chars())
}

pub fn normalized_indel_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, |a, b| {
        rapid_indel::normalized_similarity(a.chars(), b.chars())
    })
}

pub fn normalized_indel_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let scorer = rapid_indel::BatchComparator::new(reference.chars());
    match min_similarity {
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

// ─── Internal helpers for token-based algorithms ─────────────────────────────

pub fn normalize_str(s: &str) -> String {
    let mut result = String::new();
    for word in s.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&word.to_lowercase());
    }
    result
}

pub fn sorted_tokens(s: &str) -> Vec<String> {
    let mut tokens: Vec<String> = s.split_whitespace().map(|w| w.to_lowercase()).collect();
    tokens.sort();
    tokens
}

pub fn token_sort_ratio_impl(a: &str, b: &str) -> f64 {
    let sorted_a = sorted_tokens(a).join(" ");
    let sorted_b = sorted_tokens(b).join(" ");
    if sorted_a.is_empty() && sorted_b.is_empty() {
        return 1.0;
    }
    rapid_lev::normalized_similarity(sorted_a.chars(), sorted_b.chars())
}

pub fn token_set_ratio_impl(a: &str, b: &str) -> f64 {
    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);
    token_set_ratio_from_normalized(&norm_a, &norm_b)
}

pub fn token_set_ratio_from_normalized(norm_a: &str, norm_b: &str) -> f64 {
    if norm_a.is_empty() || norm_b.is_empty() {
        return if norm_a.is_empty() && norm_b.is_empty() {
            1.0
        } else {
            0.0
        };
    }

    let tokens_a: BTreeSet<&str> = norm_a.split_whitespace().collect();
    let tokens_b: BTreeSet<&str> = norm_b.split_whitespace().collect();

    let diff_a: Vec<&str> = tokens_a.difference(&tokens_b).copied().collect();
    let diff_b: Vec<&str> = tokens_b.difference(&tokens_a).copied().collect();

    if diff_a.is_empty() && diff_b.is_empty() {
        return 1.0;
    }

    let sect_str: String = tokens_a
        .intersection(&tokens_b)
        .copied()
        .collect::<Vec<_>>()
        .join(" ");

    let combined_a = if diff_a.is_empty() {
        std::borrow::Cow::Borrowed(sect_str.as_str())
    } else {
        std::borrow::Cow::Owned(format!("{} {}", sect_str, diff_a.join(" ")))
    };
    let combined_b = if diff_b.is_empty() {
        std::borrow::Cow::Borrowed(sect_str.as_str())
    } else {
        std::borrow::Cow::Owned(format!("{} {}", sect_str, diff_b.join(" ")))
    };

    let score_ab = rapid_lev::normalized_similarity(combined_a.chars(), combined_b.chars());
    let score_a = if diff_a.is_empty() {
        1.0
    } else {
        rapid_lev::normalized_similarity(sect_str.chars(), combined_a.chars())
    };
    let score_b = if diff_b.is_empty() {
        1.0
    } else {
        rapid_lev::normalized_similarity(sect_str.chars(), combined_b.chars())
    };

    f64::max(score_ab, f64::max(score_a, score_b))
}

pub fn partial_ratio_impl(a: &str, b: &str) -> f64 {
    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);
    partial_ratio_from_normalized(&norm_a, &norm_b)
}

pub fn partial_ratio_from_normalized(norm_a: &str, norm_b: &str) -> f64 {
    if norm_a.is_empty() && norm_b.is_empty() {
        return 1.0;
    }
    if norm_a.is_empty() || norm_b.is_empty() {
        return 0.0;
    }

    let (shorter, longer) = if norm_a.chars().count() <= norm_b.chars().count() {
        (norm_a, norm_b)
    } else {
        (norm_b, norm_a)
    };

    let short_len = shorter.chars().count();
    let long_len = longer.chars().count();

    if short_len == long_len {
        return rapid_lev::normalized_similarity(shorter.chars(), longer.chars());
    }

    let long_chars: Vec<char> = longer.chars().collect();
    let scorer = rapid_lev::BatchComparator::new(shorter.chars());
    let mut best = 0.0_f64;

    for start in 0..=(long_len - short_len) {
        let window = long_chars[start..start + short_len].iter().copied();
        let args = rapid_lev::Args::default().score_cutoff(best);
        if let Some(score) = scorer.normalized_similarity_with_args(window, &args) {
            best = score;
            if best == 1.0 {
                break;
            }
        }
    }

    best
}

pub fn weighted_ratio_impl(a: &str, b: &str) -> f64 {
    let raw = rapid_lev::normalized_similarity(a.chars(), b.chars());
    if raw == 1.0 {
        return 1.0;
    }
    let sort = token_sort_ratio_impl(a, b);

    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);
    let set = token_set_ratio_from_normalized(&norm_a, &norm_b);
    let partial = partial_ratio_from_normalized(&norm_a, &norm_b);

    [raw, sort, set, partial]
        .into_iter()
        .fold(0.0_f64, f64::max)
}

// ─── Token Sort Ratio ────────────────────────────────────────────────────────

pub fn token_sort_ratio(a: &str, b: &str) -> f64 {
    token_sort_ratio_impl(a, b)
}

pub fn token_sort_ratio_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, token_sort_ratio_impl)
}

pub fn token_sort_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let sorted_ref = sorted_tokens(reference).join(" ");
    candidates
        .iter()
        .map(|c| {
            let sorted_c = sorted_tokens(c).join(" ");
            let score = if sorted_ref.is_empty() && sorted_c.is_empty() {
                1.0
            } else {
                rapid_lev::normalized_similarity(sorted_ref.chars(), sorted_c.chars())
            };
            match min_similarity {
                Some(cutoff) if score < cutoff => 0.0,
                _ => score,
            }
        })
        .collect()
}

// ─── Token Set Ratio ─────────────────────────────────────────────────────────

pub fn token_set_ratio(a: &str, b: &str) -> f64 {
    token_set_ratio_impl(a, b)
}

pub fn token_set_ratio_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, token_set_ratio_impl)
}

pub fn token_set_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let norm_ref = normalize_str(reference);
    let tokens_ref: BTreeSet<&str> = norm_ref.split_whitespace().collect();

    candidates
        .iter()
        .map(|c| {
            let norm_c = normalize_str(c);

            if norm_ref.is_empty() || norm_c.is_empty() {
                let score = if norm_ref.is_empty() && norm_c.is_empty() {
                    1.0
                } else {
                    0.0
                };
                return match min_similarity {
                    Some(cutoff) if score < cutoff => 0.0,
                    _ => score,
                };
            }

            let tokens_c: BTreeSet<&str> = norm_c.split_whitespace().collect();

            let diff_ref: Vec<&str> = tokens_ref.difference(&tokens_c).copied().collect();
            let diff_c: Vec<&str> = tokens_c.difference(&tokens_ref).copied().collect();

            if diff_ref.is_empty() && diff_c.is_empty() {
                return 1.0;
            }

            let sect_str: String = tokens_ref
                .intersection(&tokens_c)
                .copied()
                .collect::<Vec<_>>()
                .join(" ");

            let combined_ref = if diff_ref.is_empty() {
                std::borrow::Cow::Borrowed(sect_str.as_str())
            } else {
                std::borrow::Cow::Owned(format!("{} {}", sect_str, diff_ref.join(" ")))
            };
            let combined_c = if diff_c.is_empty() {
                std::borrow::Cow::Borrowed(sect_str.as_str())
            } else {
                std::borrow::Cow::Owned(format!("{} {}", sect_str, diff_c.join(" ")))
            };

            let score_ab =
                rapid_lev::normalized_similarity(combined_ref.chars(), combined_c.chars());
            let score_a = if diff_ref.is_empty() {
                1.0
            } else {
                rapid_lev::normalized_similarity(sect_str.chars(), combined_ref.chars())
            };
            let score_b = if diff_c.is_empty() {
                1.0
            } else {
                rapid_lev::normalized_similarity(sect_str.chars(), combined_c.chars())
            };

            let score = f64::max(score_ab, f64::max(score_a, score_b));
            match min_similarity {
                Some(cutoff) if score < cutoff => 0.0,
                _ => score,
            }
        })
        .collect()
}

// ─── Partial Ratio ───────────────────────────────────────────────────────────

pub fn partial_ratio(a: &str, b: &str) -> f64 {
    partial_ratio_impl(a, b)
}

pub fn partial_ratio_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, partial_ratio_impl)
}

pub fn partial_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let norm_ref = normalize_str(reference);

    candidates
        .iter()
        .map(|c| {
            let norm_c = normalize_str(c);

            if norm_ref.is_empty() && norm_c.is_empty() {
                return 1.0;
            }
            if norm_ref.is_empty() || norm_c.is_empty() {
                return 0.0;
            }

            let (shorter, longer) = if norm_ref.chars().count() <= norm_c.chars().count() {
                (norm_ref.as_str(), norm_c.as_str())
            } else {
                (norm_c.as_str(), norm_ref.as_str())
            };

            let short_len = shorter.chars().count();
            let long_len = longer.chars().count();

            if short_len == long_len {
                let score = rapid_lev::normalized_similarity(shorter.chars(), longer.chars());
                return match min_similarity {
                    Some(cutoff) if score < cutoff => 0.0,
                    _ => score,
                };
            }

            let long_chars: Vec<char> = longer.chars().collect();
            let scorer = rapid_lev::BatchComparator::new(shorter.chars());
            let mut best = 0.0_f64;

            for start in 0..=(long_len - short_len) {
                let window = long_chars[start..start + short_len].iter().copied();
                let cutoff = min_similarity.map_or(best, |t| best.max(t));
                let args = rapid_lev::Args::default().score_cutoff(cutoff);
                if let Some(score) = scorer.normalized_similarity_with_args(window, &args) {
                    best = score;
                    if best == 1.0 {
                        break;
                    }
                }
            }

            match min_similarity {
                Some(cutoff) if best < cutoff => 0.0,
                _ => best,
            }
        })
        .collect()
}

// ─── Weighted Ratio ──────────────────────────────────────────────────────────

pub fn weighted_ratio(a: &str, b: &str) -> f64 {
    weighted_ratio_impl(a, b)
}

pub fn weighted_ratio_batch(pairs: &[Vec<String>]) -> Vec<f64> {
    batch_apply(pairs, weighted_ratio_impl)
}

pub fn weighted_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Vec<f64> {
    let norm_ref = normalize_str(reference);
    let sorted_ref = sorted_tokens(reference).join(" ");
    let tokens_ref: BTreeSet<String> = norm_ref.split_whitespace().map(String::from).collect();
    let ref_scorer = rapid_lev::BatchComparator::new(norm_ref.chars());

    candidates
        .iter()
        .map(|c| {
            let norm_c = normalize_str(c);
            let raw = ref_scorer.normalized_similarity(norm_c.chars());
            if raw == 1.0 {
                return 1.0;
            }

            let sorted_c = sorted_tokens(c).join(" ");
            let sort = if sorted_ref.is_empty() && sorted_c.is_empty() {
                1.0
            } else {
                rapid_lev::normalized_similarity(sorted_ref.chars(), sorted_c.chars())
            };

            let set = {
                if norm_ref.is_empty() || norm_c.is_empty() {
                    if norm_ref.is_empty() && norm_c.is_empty() {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    let tokens_c: BTreeSet<&str> = norm_c.split_whitespace().collect();
                    let tokens_ref_borrowed: BTreeSet<&str> =
                        tokens_ref.iter().map(|s| s.as_str()).collect();

                    let diff_ref: Vec<&str> =
                        tokens_ref_borrowed.difference(&tokens_c).copied().collect();
                    let diff_c: Vec<&str> =
                        tokens_c.difference(&tokens_ref_borrowed).copied().collect();

                    if diff_ref.is_empty() && diff_c.is_empty() {
                        1.0
                    } else {
                        let sect_str: String = tokens_ref_borrowed
                            .intersection(&tokens_c)
                            .copied()
                            .collect::<Vec<_>>()
                            .join(" ");

                        let combined_ref = if diff_ref.is_empty() {
                            std::borrow::Cow::Borrowed(sect_str.as_str())
                        } else {
                            std::borrow::Cow::Owned(format!("{} {}", sect_str, diff_ref.join(" ")))
                        };
                        let combined_c = if diff_c.is_empty() {
                            std::borrow::Cow::Borrowed(sect_str.as_str())
                        } else {
                            std::borrow::Cow::Owned(format!("{} {}", sect_str, diff_c.join(" ")))
                        };

                        let score_ab = rapid_lev::normalized_similarity(
                            combined_ref.chars(),
                            combined_c.chars(),
                        );
                        let score_a = if diff_ref.is_empty() {
                            1.0
                        } else {
                            rapid_lev::normalized_similarity(sect_str.chars(), combined_ref.chars())
                        };
                        let score_b = if diff_c.is_empty() {
                            1.0
                        } else {
                            rapid_lev::normalized_similarity(sect_str.chars(), combined_c.chars())
                        };

                        f64::max(score_ab, f64::max(score_a, score_b))
                    }
                }
            };

            let partial = {
                if norm_ref.is_empty() && norm_c.is_empty() {
                    1.0
                } else if norm_ref.is_empty() || norm_c.is_empty() {
                    0.0
                } else {
                    let (shorter, longer) = if norm_ref.chars().count() <= norm_c.chars().count() {
                        (norm_ref.as_str(), norm_c.as_str())
                    } else {
                        (norm_c.as_str(), norm_ref.as_str())
                    };

                    let short_len = shorter.chars().count();
                    let long_len = longer.chars().count();

                    if short_len == long_len {
                        rapid_lev::normalized_similarity(shorter.chars(), longer.chars())
                    } else {
                        let long_chars: Vec<char> = longer.chars().collect();
                        let scorer = rapid_lev::BatchComparator::new(shorter.chars());
                        let mut best = 0.0_f64;

                        for start in 0..=(long_len - short_len) {
                            let window = long_chars[start..start + short_len].iter().copied();
                            let args = rapid_lev::Args::default().score_cutoff(best);
                            if let Some(score) =
                                scorer.normalized_similarity_with_args(window, &args)
                            {
                                best = score;
                                if best == 1.0 {
                                    break;
                                }
                            }
                        }

                        best
                    }
                }
            };

            let score = [raw, sort, set, partial]
                .into_iter()
                .fold(0.0_f64, f64::max);
            match min_similarity {
                Some(cutoff) if score < cutoff => 0.0,
                _ => score,
            }
        })
        .collect()
}
