use std::collections::BTreeSet;

use napi_derive::napi;

/// Apply a distance/similarity function to each pair in a batch.
fn batch_apply<T: Default, F: Fn(&str, &str) -> T>(pairs: &[Vec<String>], f: F) -> Vec<T> {
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

/// Apply a distance/similarity function from one reference string to many candidates.
fn many_apply<T, F: Fn(&str, &str) -> T>(reference: &str, candidates: &[String], f: F) -> Vec<T> {
    candidates.iter().map(|c| f(reference, c)).collect()
}

/// Compute the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string
/// into the other.
#[napi]
pub fn levenshtein(a: String, b: String) -> u32 {
    strsim::levenshtein(&a, &b) as u32
}

/// Compute the Levenshtein distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
/// Each pair must be an array of exactly two strings `[a, b]`.
#[napi]
pub fn levenshtein_batch(pairs: Vec<Vec<String>>) -> Vec<u32> {
    batch_apply(&pairs, |a, b| strsim::levenshtein(a, b) as u32)
}

/// Compute the Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
#[napi]
pub fn levenshtein_many(reference: String, candidates: Vec<String>) -> Vec<u32> {
    many_apply(&reference, &candidates, |a, b| {
        strsim::levenshtein(a, b) as u32
    })
}

/// Compute the Damerau-Levenshtein distance between two strings.
///
/// Like Levenshtein, but also considers transpositions of two adjacent
/// characters as a single edit.
#[napi]
pub fn damerau_levenshtein(a: String, b: String) -> u32 {
    strsim::damerau_levenshtein(&a, &b) as u32
}

/// Compute the Damerau-Levenshtein distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
#[napi]
pub fn damerau_levenshtein_batch(pairs: Vec<Vec<String>>) -> Vec<u32> {
    batch_apply(&pairs, |a, b| strsim::damerau_levenshtein(a, b) as u32)
}

/// Compute the Damerau-Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
#[napi]
pub fn damerau_levenshtein_many(reference: String, candidates: Vec<String>) -> Vec<u32> {
    many_apply(&reference, &candidates, |a, b| {
        strsim::damerau_levenshtein(a, b) as u32
    })
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[napi]
pub fn jaro(a: String, b: String) -> f64 {
    strsim::jaro(&a, &b)
}

/// Compute the Jaro similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn jaro_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, strsim::jaro)
}

/// Compute the Jaro similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn jaro_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, strsim::jaro)
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// A modification of Jaro that gives more weight to common prefixes.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn jaro_winkler(a: String, b: String) -> f64 {
    strsim::jaro_winkler(&a, &b)
}

/// Compute the Jaro-Winkler similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn jaro_winkler_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, strsim::jaro_winkler)
}

/// Compute the Jaro-Winkler similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn jaro_winkler_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, strsim::jaro_winkler)
}

/// Compute the Sorensen-Dice coefficient between two strings.
///
/// Uses bigrams (pairs of consecutive characters) to measure similarity.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn sorensen_dice(a: String, b: String) -> f64 {
    strsim::sorensen_dice(&a, &b)
}

/// Compute the Sorensen-Dice coefficient for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn sorensen_dice_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, strsim::sorensen_dice)
}

/// Compute the Sorensen-Dice coefficient from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn sorensen_dice_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, strsim::sorensen_dice)
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[napi]
pub fn normalized_levenshtein(a: String, b: String) -> f64 {
    strsim::normalized_levenshtein(&a, &b)
}

/// Compute the normalized Levenshtein similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn normalized_levenshtein_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, strsim::normalized_levenshtein)
}

/// Compute the normalized Levenshtein similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn normalized_levenshtein_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, strsim::normalized_levenshtein)
}

// --- Internal helpers for token-based algorithms ---

/// Normalize a string: lowercase, trim, and collapse whitespace.
fn normalize_str(s: &str) -> String {
    let mut result = String::new();
    for word in s.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&word.to_lowercase());
    }
    result
}

/// Tokenize into sorted, lowercased tokens (skipping the intermediate joined string).
fn sorted_tokens(s: &str) -> Vec<String> {
    let mut tokens: Vec<String> = s.split_whitespace().map(|w| w.to_lowercase()).collect();
    tokens.sort();
    tokens
}

/// Internal implementation of token_sort_ratio.
fn token_sort_ratio_impl(a: &str, b: &str) -> f64 {
    let sorted_a = sorted_tokens(a).join(" ");
    let sorted_b = sorted_tokens(b).join(" ");
    if sorted_a.is_empty() && sorted_b.is_empty() {
        return 1.0;
    }
    strsim::normalized_levenshtein(&sorted_a, &sorted_b)
}

/// Internal implementation of token_set_ratio.
fn token_set_ratio_impl(a: &str, b: &str) -> f64 {
    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);

    if norm_a.is_empty() && norm_b.is_empty() {
        return 1.0;
    }

    let tokens_a: BTreeSet<&str> = norm_a.split_whitespace().collect();
    let tokens_b: BTreeSet<&str> = norm_b.split_whitespace().collect();

    let diff_a: Vec<&str> = tokens_a.difference(&tokens_b).copied().collect();
    let diff_b: Vec<&str> = tokens_b.difference(&tokens_a).copied().collect();

    // Identical token sets — no further computation needed.
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

    let score_ab = strsim::normalized_levenshtein(&combined_a, &combined_b);
    let score_a = if diff_a.is_empty() {
        1.0
    } else {
        strsim::normalized_levenshtein(&sect_str, &combined_a)
    };
    let score_b = if diff_b.is_empty() {
        1.0
    } else {
        strsim::normalized_levenshtein(&sect_str, &combined_b)
    };

    f64::max(score_ab, f64::max(score_a, score_b))
}

/// Internal implementation of partial_ratio.
fn partial_ratio_impl(a: &str, b: &str) -> f64 {
    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);

    if norm_a.is_empty() && norm_b.is_empty() {
        return 1.0;
    }
    if norm_a.is_empty() || norm_b.is_empty() {
        return 0.0;
    }

    let (shorter, longer) = if norm_a.chars().count() <= norm_b.chars().count() {
        (&norm_a, &norm_b)
    } else {
        (&norm_b, &norm_a)
    };

    let short_len = shorter.chars().count();
    let long_len = longer.chars().count();

    if short_len == long_len {
        return strsim::normalized_levenshtein(shorter, longer);
    }

    let long_chars: Vec<char> = longer.chars().collect();
    let mut best = 0.0_f64;

    for start in 0..=(long_len - short_len) {
        let window: String = long_chars[start..start + short_len].iter().collect();
        let score = strsim::normalized_levenshtein(shorter, &window);
        best = f64::max(best, score);
        if best == 1.0 {
            break;
        }
    }

    best
}

/// Internal implementation of weighted_ratio.
fn weighted_ratio_impl(a: &str, b: &str) -> f64 {
    let raw = strsim::normalized_levenshtein(a, b);
    let sort = token_sort_ratio_impl(a, b);
    let set = token_set_ratio_impl(a, b);
    let partial = partial_ratio_impl(a, b);

    // Return the best score across all methods
    [raw, sort, set, partial]
        .into_iter()
        .fold(0.0_f64, f64::max)
}

// --- Token Sort Ratio ---

/// Compute the token sort ratio between two strings.
///
/// Splits both strings into tokens, sorts them alphabetically, then computes
/// the normalized Levenshtein similarity. This makes the comparison
/// order-independent, ideal for matching names or addresses where word order varies.
/// Returns a value between 0.0 (completely different) and 1.0 (identical after sorting).
#[napi]
pub fn token_sort_ratio(a: String, b: String) -> f64 {
    token_sort_ratio_impl(&a, &b)
}

/// Compute the token sort ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn token_sort_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, token_sort_ratio_impl)
}

/// Compute the token sort ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn token_sort_ratio_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, token_sort_ratio_impl)
}

// --- Token Set Ratio ---

/// Compute the token set ratio between two strings.
///
/// Compares the intersection and differences of token sets from both strings.
/// Returns the maximum similarity among comparisons of the intersection with
/// each remainder. Highly effective for strings with shared tokens but
/// different lengths. Returns a value between 0.0 and 1.0.
#[napi]
pub fn token_set_ratio(a: String, b: String) -> f64 {
    token_set_ratio_impl(&a, &b)
}

/// Compute the token set ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn token_set_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, token_set_ratio_impl)
}

/// Compute the token set ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn token_set_ratio_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, token_set_ratio_impl)
}

// --- Partial Ratio ---

/// Compute the partial ratio between two strings.
///
/// Finds the best matching substring of the shorter string within the longer string
/// using a sliding window approach. Returns the highest normalized Levenshtein
/// similarity across all windows. Useful for matching when one string is a
/// substring or abbreviation of the other. Returns a value between 0.0 and 1.0.
#[napi]
pub fn partial_ratio(a: String, b: String) -> f64 {
    partial_ratio_impl(&a, &b)
}

/// Compute the partial ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn partial_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, partial_ratio_impl)
}

/// Compute the partial ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn partial_ratio_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, partial_ratio_impl)
}

// --- Weighted Ratio ---

/// Compute the weighted ratio between two strings.
///
/// Returns the maximum score across normalized Levenshtein, token sort ratio,
/// token set ratio, and partial ratio. This provides a single "best effort"
/// similarity score that automatically selects the most appropriate algorithm.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn weighted_ratio(a: String, b: String) -> f64 {
    weighted_ratio_impl(&a, &b)
}

/// Compute the weighted ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn weighted_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    batch_apply(&pairs, weighted_ratio_impl)
}

/// Compute the weighted ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn weighted_ratio_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    many_apply(&reference, &candidates, weighted_ratio_impl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("kitten".into(), "sitting".into()), 3);
        assert_eq!(levenshtein("".into(), "".into()), 0);
        assert_eq!(levenshtein("abc".into(), "abc".into()), 0);
    }

    #[test]
    fn test_levenshtein_batch() {
        let pairs = vec![
            vec!["kitten".to_string(), "sitting".to_string()],
            vec!["".to_string(), "".to_string()],
            vec!["abc".to_string(), "abc".to_string()],
        ];
        assert_eq!(levenshtein_batch(pairs), vec![3, 0, 0]);
    }

    #[test]
    fn test_levenshtein_many() {
        let candidates = vec!["sitting".to_string(), "".to_string(), "kitten".to_string()];
        let result = levenshtein_many("kitten".to_string(), candidates);
        assert_eq!(result, vec![3, 6, 0]);
    }

    #[test]
    fn test_jaro_winkler() {
        let score = jaro_winkler("MARTHA".into(), "MARHTA".into());
        assert!(score > 0.96);
    }

    #[test]
    fn test_jaro_winkler_batch() {
        let pairs = vec![
            vec!["MARTHA".to_string(), "MARHTA".to_string()],
            vec!["hello".to_string(), "hello".to_string()],
        ];
        let result = jaro_winkler_batch(pairs);
        assert!(result[0] > 0.96);
        assert_eq!(result[1], 1.0);
    }

    #[test]
    fn test_jaro_winkler_many() {
        let candidates = vec!["MARHTA".to_string(), "MARTHA".to_string()];
        let result = jaro_winkler_many("MARTHA".to_string(), candidates);
        assert!(result[0] > 0.96);
        assert_eq!(result[1], 1.0);
    }

    #[test]
    fn test_sorensen_dice() {
        let score = sorensen_dice("night".into(), "nacht".into());
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn test_sorensen_dice_batch() {
        let pairs = vec![
            vec!["night".to_string(), "nacht".to_string()],
            vec!["abc".to_string(), "abc".to_string()],
        ];
        let result = sorensen_dice_batch(pairs);
        assert!(result[0] > 0.0 && result[0] < 1.0);
        assert_eq!(result[1], 1.0);
    }

    #[test]
    fn test_sorensen_dice_many() {
        let candidates = vec!["nacht".to_string(), "night".to_string()];
        let result = sorensen_dice_many("night".to_string(), candidates);
        assert!(result[0] > 0.0 && result[0] < 1.0);
        assert_eq!(result[1], 1.0);
    }

    mod proptest_distance {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            // --- Levenshtein properties ---

            #[test]
            fn levenshtein_identity(a in ".*") {
                prop_assert_eq!(levenshtein(a.clone(), a), 0);
            }

            #[test]
            fn levenshtein_symmetry(a in ".*", b in ".*") {
                prop_assert_eq!(
                    levenshtein(a.clone(), b.clone()),
                    levenshtein(b, a)
                );
            }

            #[test]
            fn levenshtein_triangle_inequality(a in ".{0,20}", b in ".{0,20}", c in ".{0,20}") {
                let ab = levenshtein(a.clone(), b.clone());
                let bc = levenshtein(b, c.clone());
                let ac = levenshtein(a, c);
                prop_assert!(ac <= ab + bc, "triangle inequality violated: d(a,c)={} > d(a,b)={} + d(b,c)={}", ac, ab, bc);
            }

            // --- Damerau-Levenshtein properties ---

            #[test]
            fn damerau_levenshtein_identity(a in ".*") {
                prop_assert_eq!(damerau_levenshtein(a.clone(), a), 0);
            }

            #[test]
            fn damerau_levenshtein_symmetry(a in ".*", b in ".*") {
                prop_assert_eq!(
                    damerau_levenshtein(a.clone(), b.clone()),
                    damerau_levenshtein(b, a)
                );
            }

            // --- Normalized Levenshtein properties ---

            #[test]
            fn normalized_levenshtein_bounded(a in ".*", b in ".*") {
                let score = normalized_levenshtein(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn normalized_levenshtein_identity(a in ".+") {
                let score = normalized_levenshtein(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            #[test]
            fn normalized_levenshtein_symmetry(a in ".*", b in ".*") {
                let ab = normalized_levenshtein(a.clone(), b.clone());
                let ba = normalized_levenshtein(b, a);
                prop_assert!((ab - ba).abs() < f64::EPSILON, "symmetry violated: {} != {}", ab, ba);
            }

            // --- Jaro properties ---

            #[test]
            fn jaro_bounded(a in ".*", b in ".*") {
                let score = jaro(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn jaro_identity(a in ".+") {
                let score = jaro(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            #[test]
            fn jaro_symmetry(a in ".*", b in ".*") {
                let ab = jaro(a.clone(), b.clone());
                let ba = jaro(b, a);
                prop_assert!((ab - ba).abs() < f64::EPSILON, "symmetry violated: {} != {}", ab, ba);
            }

            // --- Jaro-Winkler properties ---

            #[test]
            fn jaro_winkler_bounded(a in ".*", b in ".*") {
                let score = jaro_winkler(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn jaro_winkler_identity(a in ".+") {
                let score = jaro_winkler(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            #[test]
            fn jaro_winkler_ge_jaro(a in ".*", b in ".*") {
                let jw = jaro_winkler(a.clone(), b.clone());
                let j = jaro(a, b);
                prop_assert!(jw >= j - f64::EPSILON, "jaro_winkler({}) < jaro({})", jw, j);
            }

            // --- Sorensen-Dice properties ---

            #[test]
            fn sorensen_dice_bounded(a in ".*", b in ".*") {
                let score = sorensen_dice(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn sorensen_dice_identity(a in ".{2,}") {
                // Sorensen-Dice uses bigrams, so needs at least 2 chars for meaningful identity
                let score = sorensen_dice(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            #[test]
            fn sorensen_dice_symmetry(a in ".*", b in ".*") {
                let ab = sorensen_dice(a.clone(), b.clone());
                let ba = sorensen_dice(b, a);
                prop_assert!((ab - ba).abs() < f64::EPSILON, "symmetry violated: {} != {}", ab, ba);
            }

            // --- Token Sort Ratio properties ---

            #[test]
            fn token_sort_ratio_bounded(a in ".*", b in ".*") {
                let score = token_sort_ratio(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn token_sort_ratio_identity(a in ".+") {
                let score = token_sort_ratio(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            #[test]
            fn token_sort_ratio_symmetry(a in ".*", b in ".*") {
                let ab = token_sort_ratio(a.clone(), b.clone());
                let ba = token_sort_ratio(b, a);
                prop_assert!((ab - ba).abs() < f64::EPSILON, "symmetry violated: {} != {}", ab, ba);
            }

            // --- Token Set Ratio properties ---

            #[test]
            fn token_set_ratio_bounded(a in ".*", b in ".*") {
                let score = token_set_ratio(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn token_set_ratio_identity(a in ".+") {
                let score = token_set_ratio(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            // --- Partial Ratio properties ---

            #[test]
            fn partial_ratio_bounded(a in ".*", b in ".*") {
                let score = partial_ratio(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn partial_ratio_identity(a in ".+") {
                let score = partial_ratio(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            // --- Weighted Ratio properties ---

            #[test]
            fn weighted_ratio_bounded(a in ".*", b in ".*") {
                let score = weighted_ratio(a, b);
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn weighted_ratio_identity(a in ".+") {
                let score = weighted_ratio(a.clone(), a);
                prop_assert_eq!(score, 1.0);
            }

            #[test]
            fn weighted_ratio_ge_normalized_levenshtein(a in ".{1,20}", b in ".{1,20}") {
                let w = weighted_ratio(a.clone(), b.clone());
                let base = normalized_levenshtein(a, b);
                prop_assert!(w >= base - f64::EPSILON, "weighted {} < base {}", w, base);
            }
        }
    }

    mod token_sort_ratio_tests {
        use super::*;

        #[test]
        fn test_identical_reordered() {
            let score = token_sort_ratio("New York Mets".into(), "Mets New York".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_case_insensitive() {
            let score = token_sort_ratio("john smith".into(), "SMITH JOHN".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_similar_reordered() {
            let score = token_sort_ratio("John A. Smith".into(), "Smith, John A".into());
            assert!(score > 0.8, "expected > 0.8, got {}", score);
        }

        #[test]
        fn test_completely_different() {
            let score = token_sort_ratio("abc".into(), "xyz".into());
            assert!(score < 0.5, "expected < 0.5, got {}", score);
        }

        #[test]
        fn test_empty_strings() {
            assert_eq!(token_sort_ratio("".into(), "".into()), 1.0);
        }

        #[test]
        fn test_one_empty() {
            let score = token_sort_ratio("hello".into(), "".into());
            assert_eq!(score, 0.0);
        }

        #[test]
        fn test_batch() {
            let pairs = vec![
                vec!["New York Mets".to_string(), "Mets New York".to_string()],
                vec!["abc".to_string(), "abc".to_string()],
            ];
            let result = token_sort_ratio_batch(pairs);
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 1.0);
        }

        #[test]
        fn test_many() {
            let candidates = vec![
                "Mets New York".to_string(),
                "completely different".to_string(),
            ];
            let result = token_sort_ratio_many("New York Mets".to_string(), candidates);
            assert_eq!(result[0], 1.0);
            assert!(result[1] < 0.5);
        }
    }

    mod token_set_ratio_tests {
        use super::*;

        #[test]
        fn test_reordered_with_shared_tokens() {
            let score = token_set_ratio("Mariners vs Yankees".into(), "Yankees vs Mariners".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_subset_tokens() {
            let score = token_set_ratio(
                "Great Gatsby".into(),
                "The Great Gatsby by Fitzgerald".into(),
            );
            assert!(score > 0.7, "expected > 0.7, got {}", score);
        }

        #[test]
        fn test_no_shared_tokens() {
            let score = token_set_ratio("abc def".into(), "xyz uvw".into());
            assert!(score < 0.5, "expected < 0.5, got {}", score);
        }

        #[test]
        fn test_empty_strings() {
            assert_eq!(token_set_ratio("".into(), "".into()), 1.0);
        }

        #[test]
        fn test_batch() {
            let pairs = vec![
                vec![
                    "Mariners vs Yankees".to_string(),
                    "Yankees vs Mariners".to_string(),
                ],
                vec!["abc".to_string(), "xyz".to_string()],
            ];
            let result = token_set_ratio_batch(pairs);
            assert_eq!(result[0], 1.0);
            assert!(result[1] < 0.5);
        }

        #[test]
        fn test_many() {
            let candidates = vec![
                "Yankees vs Mariners".to_string(),
                "completely different".to_string(),
            ];
            let result = token_set_ratio_many("Mariners vs Yankees".to_string(), candidates);
            assert_eq!(result[0], 1.0);
            assert!(result[1] < 0.5);
        }
    }

    mod partial_ratio_tests {
        use super::*;

        #[test]
        fn test_substring_match() {
            let score = partial_ratio("hello".into(), "hello world".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_identical() {
            let score = partial_ratio("hello".into(), "hello".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_partial_overlap() {
            let score = partial_ratio("cat".into(), "scattered".into());
            assert!(score > 0.5, "expected > 0.5, got {}", score);
        }

        #[test]
        fn test_no_match() {
            let score = partial_ratio("abc".into(), "xyz".into());
            assert!(score < 0.5, "expected < 0.5, got {}", score);
        }

        #[test]
        fn test_empty_strings() {
            assert_eq!(partial_ratio("".into(), "".into()), 1.0);
        }

        #[test]
        fn test_one_empty() {
            assert_eq!(partial_ratio("hello".into(), "".into()), 0.0);
        }

        #[test]
        fn test_batch() {
            let pairs = vec![
                vec!["hello".to_string(), "hello world".to_string()],
                vec!["abc".to_string(), "xyz".to_string()],
            ];
            let result = partial_ratio_batch(pairs);
            assert_eq!(result[0], 1.0);
            assert!(result[1] < 0.5);
        }

        #[test]
        fn test_many() {
            let candidates = vec!["hello world".to_string(), "xyz".to_string()];
            let result = partial_ratio_many("hello".to_string(), candidates);
            assert_eq!(result[0], 1.0);
            assert!(result[1] < 0.5);
        }
    }

    mod weighted_ratio_tests {
        use super::*;

        #[test]
        fn test_identical() {
            assert_eq!(weighted_ratio("hello".into(), "hello".into()), 1.0);
        }

        #[test]
        fn test_reordered_tokens() {
            let score = weighted_ratio("New York Mets".into(), "Mets New York".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_substring() {
            let score = weighted_ratio("hello".into(), "hello world".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_different() {
            let score = weighted_ratio("abc".into(), "xyz".into());
            assert!(score < 0.5, "expected < 0.5, got {}", score);
        }

        #[test]
        fn test_at_least_base_ratio() {
            let a = "test string".to_string();
            let b = "testing strings".to_string();
            let w = weighted_ratio(a.clone(), b.clone());
            let base = normalized_levenshtein(a, b);
            assert!(w >= base, "weighted {} < base {}", w, base);
        }

        #[test]
        fn test_batch() {
            let pairs = vec![
                vec!["hello".to_string(), "hello".to_string()],
                vec!["abc".to_string(), "xyz".to_string()],
            ];
            let result = weighted_ratio_batch(pairs);
            assert_eq!(result[0], 1.0);
            assert!(result[1] < 0.5);
        }

        #[test]
        fn test_many() {
            let candidates = vec!["hello".to_string(), "xyz".to_string()];
            let result = weighted_ratio_many("hello".to_string(), candidates);
            assert_eq!(result[0], 1.0);
            assert!(result[1] < 0.5);
        }
    }

    mod unicode_tests {
        use super::*;

        #[test]
        fn test_levenshtein_cjk() {
            // Each CJK character is one Unicode scalar value (char)
            assert_eq!(levenshtein("東京".into(), "京都".into()), 2);
            assert_eq!(levenshtein("日本語".into(), "日本語".into()), 0);
            assert_eq!(levenshtein("日本語".into(), "日本人".into()), 1);
        }

        #[test]
        fn test_levenshtein_emoji() {
            // Each emoji (non-ZWJ) is one Unicode scalar value
            assert_eq!(levenshtein("👋🌍".into(), "👋🌎".into()), 1);
            assert_eq!(levenshtein("🎉🎊".into(), "🎉🎊".into()), 0);
        }

        #[test]
        fn test_levenshtein_diacritics() {
            // 'é' (U+00E9) is one char, 'e' (U+0065) is one char -> distance 1
            assert_eq!(levenshtein("café".into(), "cafe".into()), 1);
            assert_eq!(levenshtein("naïve".into(), "naïve".into()), 0);
        }

        #[test]
        fn test_levenshtein_mixed_scripts() {
            assert_eq!(levenshtein("hello世界".into(), "hello世間".into()), 1);
        }

        #[test]
        fn test_jaro_winkler_accented() {
            let score = jaro_winkler("café".into(), "cafe".into());
            assert!(score > 0.8 && score < 1.0);
        }

        #[test]
        fn test_sorensen_dice_cjk() {
            let score = sorensen_dice("日本語".into(), "日本人".into());
            assert!(score > 0.0 && score < 1.0);
        }

        #[test]
        fn test_normalized_levenshtein_unicode_identity() {
            assert_eq!(normalized_levenshtein("naïve".into(), "naïve".into()), 1.0);
            assert_eq!(normalized_levenshtein("東京".into(), "東京".into()), 1.0);
            assert_eq!(normalized_levenshtein("🎉🎊".into(), "🎉🎊".into()), 1.0);
        }

        #[test]
        fn test_token_sort_ratio_cjk() {
            let score = token_sort_ratio("東京 日本".into(), "日本 東京".into());
            assert_eq!(score, 1.0);
        }

        #[test]
        fn test_token_set_ratio_cjk() {
            let score = token_set_ratio("東京 観光".into(), "東京 観光 案内".into());
            assert!(score > 0.7);
        }

        #[test]
        fn test_partial_ratio_cjk() {
            let score = partial_ratio("東京".into(), "東京タワー".into());
            assert!(score > 0.5);
        }

        #[test]
        fn test_token_sort_ratio_emoji() {
            let score = token_sort_ratio("🎉 🎊".into(), "🎊 🎉".into());
            assert_eq!(score, 1.0);
        }
    }

    mod proptest_unicode {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            // Use any::<String>() to generate arbitrary Unicode strings

            #[test]
            fn levenshtein_unicode_identity(ref s in any::<String>()) {
                prop_assert_eq!(levenshtein(s.clone(), s.clone()), 0);
            }

            #[test]
            fn levenshtein_unicode_symmetry(ref a in any::<String>(), ref b in any::<String>()) {
                prop_assert_eq!(
                    levenshtein(a.clone(), b.clone()),
                    levenshtein(b.clone(), a.clone())
                );
            }

            #[test]
            fn normalized_levenshtein_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = normalized_levenshtein(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn jaro_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = jaro(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn jaro_winkler_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = jaro_winkler(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn sorensen_dice_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = sorensen_dice(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn sorensen_dice_unicode_symmetry(ref a in any::<String>(), ref b in any::<String>()) {
                let ab = sorensen_dice(a.clone(), b.clone());
                let ba = sorensen_dice(b.clone(), a.clone());
                prop_assert!((ab - ba).abs() < f64::EPSILON, "symmetry violated: {} != {}", ab, ba);
            }

            // --- Token Sort Ratio Unicode properties ---

            #[test]
            fn token_sort_ratio_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = token_sort_ratio(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            #[test]
            fn token_sort_ratio_unicode_symmetry(ref a in any::<String>(), ref b in any::<String>()) {
                let ab = token_sort_ratio(a.clone(), b.clone());
                let ba = token_sort_ratio(b.clone(), a.clone());
                prop_assert!((ab - ba).abs() < f64::EPSILON, "symmetry violated: {} != {}", ab, ba);
            }

            // --- Token Set Ratio Unicode properties ---

            #[test]
            fn token_set_ratio_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = token_set_ratio(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            // --- Partial Ratio Unicode properties ---

            #[test]
            fn partial_ratio_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = partial_ratio(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }

            // --- Weighted Ratio Unicode properties ---

            #[test]
            fn weighted_ratio_unicode_bounded(ref a in any::<String>(), ref b in any::<String>()) {
                let score = weighted_ratio(a.clone(), b.clone());
                prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of [0, 1]", score);
            }
        }
    }
}
