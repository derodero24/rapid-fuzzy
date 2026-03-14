use napi_derive::napi;

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
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                strsim::levenshtein(&pair[0], &pair[1]) as u32
            } else {
                0
            }
        })
        .collect()
}

/// Compute the Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
#[napi]
pub fn levenshtein_many(reference: String, candidates: Vec<String>) -> Vec<u32> {
    candidates
        .iter()
        .map(|c| strsim::levenshtein(&reference, c) as u32)
        .collect()
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
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                strsim::damerau_levenshtein(&pair[0], &pair[1]) as u32
            } else {
                0
            }
        })
        .collect()
}

/// Compute the Damerau-Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
#[napi]
pub fn damerau_levenshtein_many(reference: String, candidates: Vec<String>) -> Vec<u32> {
    candidates
        .iter()
        .map(|c| strsim::damerau_levenshtein(&reference, c) as u32)
        .collect()
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
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                strsim::jaro(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

/// Compute the Jaro similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn jaro_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    candidates
        .iter()
        .map(|c| strsim::jaro(&reference, c))
        .collect()
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
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                strsim::jaro_winkler(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

/// Compute the Jaro-Winkler similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn jaro_winkler_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    candidates
        .iter()
        .map(|c| strsim::jaro_winkler(&reference, c))
        .collect()
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
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                strsim::sorensen_dice(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

/// Compute the Sorensen-Dice coefficient from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn sorensen_dice_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    candidates
        .iter()
        .map(|c| strsim::sorensen_dice(&reference, c))
        .collect()
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
    pairs
        .iter()
        .map(|pair| {
            if pair.len() >= 2 {
                strsim::normalized_levenshtein(&pair[0], &pair[1])
            } else {
                0.0
            }
        })
        .collect()
}

/// Compute the normalized Levenshtein similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
#[napi]
pub fn normalized_levenshtein_many(reference: String, candidates: Vec<String>) -> Vec<f64> {
    candidates
        .iter()
        .map(|c| strsim::normalized_levenshtein(&reference, c))
        .collect()
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
        }
    }
}
