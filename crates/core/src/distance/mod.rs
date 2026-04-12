use napi_derive::napi;
use rapid_fuzzy_core::distance as core_dist;

/// Compute the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string
/// into the other.
#[napi]
pub fn levenshtein(a: String, b: String) -> u32 {
    core_dist::levenshtein(&a, &b)
}

/// Compute the Levenshtein distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
/// Each pair must be an array of exactly two strings `[a, b]`.
#[napi]
pub fn levenshtein_batch(pairs: Vec<Vec<String>>) -> Vec<u32> {
    core_dist::levenshtein_batch(&pairs)
}

/// Compute the Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// If `max_distance` is provided, candidates with distance exceeding the threshold
/// will return `max_distance + 1` (enabling early termination for better performance).
#[napi]
pub fn levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::levenshtein_many(&reference, &candidates, max_distance)
}

/// Compute the Damerau-Levenshtein distance between two strings.
///
/// Like Levenshtein, but also considers transpositions of two adjacent
/// characters as a single edit.
#[napi]
pub fn damerau_levenshtein(a: String, b: String) -> u32 {
    core_dist::damerau_levenshtein(&a, &b)
}

/// Compute the Damerau-Levenshtein distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
#[napi]
pub fn damerau_levenshtein_batch(pairs: Vec<Vec<String>>) -> Vec<u32> {
    core_dist::damerau_levenshtein_batch(&pairs)
}

/// Compute the Damerau-Levenshtein distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// If `max_distance` is provided, candidates with distance exceeding the threshold
/// will return `max_distance + 1` (enabling early termination for better performance).
#[napi]
pub fn damerau_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::damerau_levenshtein_many(&reference, &candidates, max_distance)
}

/// Compute the Hamming distance between two strings.
///
/// The Hamming distance counts the number of positions at which the corresponding
/// characters differ. It is only defined for strings of equal length.
/// Returns `null` if the strings have different lengths.
#[napi]
pub fn hamming(a: String, b: String) -> Option<u32> {
    core_dist::hamming(&a, &b)
}

/// Compute the Hamming distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
/// Each pair must be an array of exactly two strings `[a, b]`.
/// Returns `null` for pairs with different lengths.
#[napi]
pub fn hamming_batch(pairs: Vec<Vec<String>>) -> Vec<Option<u32>> {
    core_dist::hamming_batch(&pairs)
}

/// Compute the Hamming distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// Returns `null` for candidates with a different length than the reference.
/// If `max_distance` is provided, candidates with distance exceeding the threshold
/// will also return `null` (enabling early termination for better performance).
#[napi]
pub fn hamming_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<Option<u32>> {
    core_dist::hamming_many(&reference, &candidates, max_distance)
}

/// Compute the normalized Hamming similarity between two strings.
///
/// Returns `null` if the strings have different lengths.
/// Returns a value between 0.0 (no matching characters) and 1.0 (identical).
#[napi]
pub fn normalized_hamming(a: String, b: String) -> Option<f64> {
    core_dist::normalized_hamming(&a, &b)
}

/// Compute the normalized Hamming similarity for multiple pairs of strings in a single call.
///
/// Returns an array of scores in the same order as the input pairs.
/// Returns `null` for pairs with different lengths.
#[napi]
pub fn normalized_hamming_batch(pairs: Vec<Vec<String>>) -> Vec<Option<f64>> {
    core_dist::normalized_hamming_batch(&pairs)
}

/// Compute the normalized Hamming similarity from one reference string to many candidates.
///
/// Returns an array of scores, one per candidate, in the same order as the input.
/// Returns `null` for candidates with a different length than the reference.
/// If `min_similarity` is provided, candidates with similarity below the threshold
/// will also return `null` (enabling early termination for better performance).
#[napi]
pub fn normalized_hamming_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<Option<f64>> {
    core_dist::normalized_hamming_many(&reference, &candidates, min_similarity)
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[napi]
pub fn jaro(a: String, b: String) -> f64 {
    core_dist::jaro(&a, &b)
}

/// Compute the Jaro similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn jaro_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::jaro_batch(&pairs)
}

/// Compute the Jaro similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[napi]
pub fn jaro_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::jaro_many(&reference, &candidates, min_similarity)
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// A modification of Jaro that gives more weight to common prefixes.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn jaro_winkler(a: String, b: String) -> f64 {
    core_dist::jaro_winkler(&a, &b)
}

/// Compute the Jaro-Winkler similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn jaro_winkler_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::jaro_winkler_batch(&pairs)
}

/// Compute the Jaro-Winkler similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[napi]
pub fn jaro_winkler_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::jaro_winkler_many(&reference, &candidates, min_similarity)
}

/// Compute the Sorensen-Dice coefficient between two strings.
///
/// Uses bigrams (pairs of consecutive characters) to measure similarity.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn sorensen_dice(a: String, b: String) -> f64 {
    core_dist::sorensen_dice(&a, &b)
}

/// Compute the Sorensen-Dice coefficient for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn sorensen_dice_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::sorensen_dice_batch(&pairs)
}

/// Compute the Sorensen-Dice coefficient from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates scoring below the threshold return `0.0`.
/// Reference bigrams are pre-computed once and reused for all candidates.
#[napi]
pub fn sorensen_dice_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::sorensen_dice_many(&reference, &candidates, min_similarity)
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[napi]
pub fn normalized_levenshtein(a: String, b: String) -> f64 {
    core_dist::normalized_levenshtein(&a, &b)
}

/// Compute the normalized Levenshtein similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn normalized_levenshtein_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::normalized_levenshtein_batch(&pairs)
}

/// Compute the normalized Levenshtein similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[napi]
pub fn normalized_levenshtein_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::normalized_levenshtein_many(&reference, &candidates, min_similarity)
}

/// Compute the Indel distance between two strings.
///
/// The Indel distance counts the minimum number of insertions and deletions
/// (no substitutions) required to transform one string into the other.
/// It equals `len(a) + len(b) - 2 * LCS_length(a, b)`.
///
/// Useful when substitutions are semantically two operations (one deletion +
/// one insertion), such as in DNA sequence alignment.
#[napi]
pub fn indel(a: String, b: String) -> u32 {
    core_dist::indel(&a, &b)
}

/// Compute the Indel distance for multiple pairs of strings in a single call.
///
/// Returns an array of distances in the same order as the input pairs.
/// Each pair must be an array of exactly two strings `[a, b]`.
#[napi]
pub fn indel_batch(pairs: Vec<Vec<String>>) -> Vec<u32> {
    core_dist::indel_batch(&pairs)
}

/// Compute the Indel distance from one reference string to many candidates.
///
/// Returns an array of distances, one per candidate, in the same order as the input.
/// If `max_distance` is provided, candidates with distance exceeding the threshold
/// will return `max_distance + 1` (enabling early termination for better performance).
#[napi]
pub fn indel_many(
    reference: String,
    candidates: Vec<String>,
    max_distance: Option<u32>,
) -> Vec<u32> {
    core_dist::indel_many(&reference, &candidates, max_distance)
}

/// Compute the normalized Indel similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[napi]
pub fn normalized_indel(a: String, b: String) -> f64 {
    core_dist::normalized_indel(&a, &b)
}

/// Compute the normalized Indel similarity for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn normalized_indel_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::normalized_indel_batch(&pairs)
}

/// Compute the normalized Indel similarity from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates with similarity below the threshold
/// will return `0.0` (enabling early termination for better performance).
#[napi]
pub fn normalized_indel_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::normalized_indel_many(&reference, &candidates, min_similarity)
}

/// Compute the token sort ratio between two strings.
///
/// Splits both strings into tokens, sorts them alphabetically, then computes
/// the normalized Levenshtein similarity. This makes the comparison
/// order-independent, ideal for matching names or addresses where word order varies.
/// Returns a value between 0.0 (completely different) and 1.0 (identical after sorting).
#[napi]
pub fn token_sort_ratio(a: String, b: String) -> f64 {
    core_dist::token_sort_ratio(&a, &b)
}

/// Compute the token sort ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn token_sort_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::token_sort_ratio_batch(&pairs)
}

/// Compute the token sort ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates scoring below the threshold return `0.0`.
#[napi]
pub fn token_sort_ratio_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::token_sort_ratio_many(&reference, &candidates, min_similarity)
}

/// Compute the token set ratio between two strings.
///
/// Compares the intersection and differences of token sets from both strings.
/// Returns the maximum similarity among comparisons of the intersection with
/// each remainder. Highly effective for strings with shared tokens but
/// different lengths. Returns a value between 0.0 and 1.0.
#[napi]
pub fn token_set_ratio(a: String, b: String) -> f64 {
    core_dist::token_set_ratio(&a, &b)
}

/// Compute the token set ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn token_set_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::token_set_ratio_batch(&pairs)
}

/// Compute the token set ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates scoring below the threshold return `0.0`.
#[napi]
pub fn token_set_ratio_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::token_set_ratio_many(&reference, &candidates, min_similarity)
}

/// Compute the partial ratio between two strings.
///
/// Finds the best matching substring of the shorter string within the longer string
/// using a sliding window approach. Returns the highest normalized Levenshtein
/// similarity across all windows. Useful for matching when one string is a
/// substring or abbreviation of the other. Returns a value between 0.0 and 1.0.
#[napi]
pub fn partial_ratio(a: String, b: String) -> f64 {
    core_dist::partial_ratio(&a, &b)
}

/// Compute the partial ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn partial_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::partial_ratio_batch(&pairs)
}

/// Compute the partial ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates scoring below the threshold return `0.0`.
#[napi]
pub fn partial_ratio_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::partial_ratio_many(&reference, &candidates, min_similarity)
}

/// Compute the weighted ratio between two strings.
///
/// Returns the maximum score across normalized Levenshtein, token sort ratio,
/// token set ratio, and partial ratio. This provides a single "best effort"
/// similarity score that automatically selects the most appropriate algorithm.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn weighted_ratio(a: String, b: String) -> f64 {
    core_dist::weighted_ratio(&a, &b)
}

/// Compute the weighted ratio for multiple pairs of strings in a single call.
///
/// Returns an array of similarity scores in the same order as the input pairs.
#[napi]
pub fn weighted_ratio_batch(pairs: Vec<Vec<String>>) -> Vec<f64> {
    core_dist::weighted_ratio_batch(&pairs)
}

/// Compute the weighted ratio from one reference string to many candidates.
///
/// Returns an array of similarity scores, one per candidate, in the same order as the input.
/// If `min_similarity` is provided, candidates scoring below the threshold return `0.0`.
#[napi]
pub fn weighted_ratio_many(
    reference: String,
    candidates: Vec<String>,
    min_similarity: Option<f64>,
) -> Vec<f64> {
    core_dist::weighted_ratio_many(&reference, &candidates, min_similarity)
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
        let result = levenshtein_many("kitten".to_string(), candidates, None);
        assert_eq!(result, vec![3, 6, 0]);
    }

    #[test]
    fn test_hamming() {
        assert_eq!(hamming("karolin".into(), "kathrin".into()), Some(3));
        assert_eq!(hamming("".into(), "".into()), Some(0));
        assert_eq!(hamming("abc".into(), "abc".into()), Some(0));
        // Different lengths return None
        assert_eq!(hamming("abc".into(), "ab".into()), None);
        assert_eq!(hamming("".into(), "a".into()), None);
    }

    #[test]
    fn test_hamming_batch() {
        let pairs = vec![
            vec!["karolin".to_string(), "kathrin".to_string()],
            vec!["abc".to_string(), "abc".to_string()],
            vec!["abc".to_string(), "ab".to_string()],
        ];
        assert_eq!(hamming_batch(pairs), vec![Some(3), Some(0), None]);
    }

    #[test]
    fn test_hamming_many() {
        let candidates = vec![
            "kathrin".to_string(),
            "karolin".to_string(),
            "abc".to_string(),
        ];
        let result = hamming_many("karolin".to_string(), candidates, None);
        assert_eq!(result, vec![Some(3), Some(0), None]);
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
        let result = jaro_winkler_many("MARTHA".to_string(), candidates, None);
        assert!(result[0] > 0.96);
        assert_eq!(result[1], 1.0);
    }

    #[test]
    fn test_jaro_regression_vectors() {
        let vectors: Vec<(&str, &str, f64)> = vec![
            ("MARTHA", "MARHTA", 0.9444444444444445),
            ("DWAYNE", "DUANE", 0.8222222222222223),
            ("DIXON", "DICKSONX", 0.7666666666666666),
            ("a jke", "jane a k", 0.6),
            ("", "", 1.0),
            ("a", "", 0.0),
            ("a", "a", 1.0),
        ];
        for (a, b, expected) in vectors {
            let score = jaro(a.into(), b.into());
            assert!(
                (score - expected).abs() <= f64::EPSILON,
                "jaro({a:?}, {b:?}) = {score}, expected {expected}",
            );
        }
    }

    #[test]
    fn test_jaro_winkler_regression_vectors() {
        let vectors: Vec<(&str, &str, f64)> = vec![
            ("MARTHA", "MARHTA", 0.9611111111111111),
            ("DWAYNE", "DUANE", 0.84),
            ("DIXON", "DICKSONX", 0.8133333333333332),
            ("", "", 1.0),
            ("a", "", 0.0),
            ("a", "a", 1.0),
        ];
        for (a, b, expected) in vectors {
            let score = jaro_winkler(a.into(), b.into());
            assert!(
                (score - expected).abs() <= f64::EPSILON,
                "jaro_winkler({a:?}, {b:?}) = {score}, expected {expected}",
            );
        }
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
        let result = sorensen_dice_many("night".to_string(), candidates, None);
        assert!(result[0] > 0.0 && result[0] < 1.0);
        assert_eq!(result[1], 1.0);
    }

    #[test]
    fn test_normalized_hamming() {
        // Equal-length strings return Some
        assert_eq!(normalized_hamming("abc".into(), "abc".into()), Some(1.0));
        assert_eq!(normalized_hamming("".into(), "".into()), Some(1.0));
        let score = normalized_hamming("karolin".into(), "kathrin".into());
        assert!(score.is_some());
        let s = score.unwrap();
        assert!(s >= 0.0 && s <= 1.0);
        // Different lengths return None
        assert_eq!(normalized_hamming("abc".into(), "ab".into()), None);
        assert_eq!(normalized_hamming("".into(), "a".into()), None);
    }

    #[test]
    fn test_indel() {
        // Identical strings have distance 0
        assert_eq!(indel("abc".into(), "abc".into()), 0);
        assert_eq!(indel("".into(), "".into()), 0);
        // Indel("abc", "ac") = len("abc") + len("ac") - 2 * LCS_len = 3 + 2 - 2*2 = 1
        assert_eq!(indel("abc".into(), "ac".into()), 1);
    }

    #[test]
    fn test_normalized_indel() {
        // Identical strings have similarity 1.0
        assert_eq!(normalized_indel("abc".into(), "abc".into()), 1.0);
        assert_eq!(normalized_indel("".into(), "".into()), 1.0);
        // Similarity is in [0, 1]
        let score = normalized_indel("abc".into(), "xyz".into());
        assert!(score >= 0.0 && score <= 1.0);
        // Partially similar strings
        let score2 = normalized_indel("kitten".into(), "sitting".into());
        assert!(score2 > 0.0 && score2 < 1.0);
    }

    mod score_cutoff_tests {
        use super::*;

        #[test]
        fn test_levenshtein_many_with_cutoff() {
            let candidates = vec![
                "sitting".to_string(),
                "kitten".to_string(),
                "abcdef".to_string(),
            ];
            // max_distance = 2: "sitting" (dist=3) exceeds, "kitten" (dist=0) passes, "abcdef" (dist=5) exceeds
            let result = levenshtein_many("kitten".to_string(), candidates, Some(2));
            assert_eq!(result, vec![3, 0, 3]); // sentinel = max_distance + 1 = 3
        }

        #[test]
        fn test_levenshtein_many_without_cutoff() {
            let candidates = vec!["sitting".to_string(), "kitten".to_string()];
            let result = levenshtein_many("kitten".to_string(), candidates, None);
            assert_eq!(result, vec![3, 0]);
        }

        #[test]
        fn test_damerau_levenshtein_many_with_cutoff() {
            let candidates = vec![
                "sitting".to_string(),
                "kitten".to_string(),
                "abcdef".to_string(),
            ];
            let result = damerau_levenshtein_many("kitten".to_string(), candidates, Some(2));
            // "sitting" has DL distance 3 (exceeds cutoff 2 -> sentinel 3)
            // "kitten" has DL distance 0 (within cutoff)
            // "abcdef" has DL distance > 2 (exceeds cutoff -> sentinel 3)
            assert_eq!(result[1], 0);
            assert_eq!(result[0], 3); // sentinel
            assert_eq!(result[2], 3); // sentinel
        }

        #[test]
        fn test_damerau_levenshtein_many_without_cutoff() {
            let candidates = vec!["sitting".to_string(), "kitten".to_string()];
            let result = damerau_levenshtein_many("kitten".to_string(), candidates, None);
            assert_eq!(result[1], 0);
            assert!(result[0] > 0);
        }

        #[test]
        fn test_hamming_many_with_cutoff() {
            let candidates = vec![
                "kathrin".to_string(), // dist=3, exceeds cutoff 2 -> None
                "karolin".to_string(), // dist=0, within cutoff
                "abc".to_string(),     // different length -> None
            ];
            let result = hamming_many("karolin".to_string(), candidates, Some(2));
            assert_eq!(result, vec![None, Some(0), None]);
        }

        #[test]
        fn test_hamming_many_without_cutoff() {
            let candidates = vec![
                "kathrin".to_string(),
                "karolin".to_string(),
                "abc".to_string(),
            ];
            let result = hamming_many("karolin".to_string(), candidates, None);
            assert_eq!(result, vec![Some(3), Some(0), None]);
        }

        #[test]
        fn test_hamming_many_cutoff_zero() {
            // max_distance = 0 means only exact matches pass
            let candidates = vec!["karolin".to_string(), "kathrin".to_string()];
            let result = hamming_many("karolin".to_string(), candidates, Some(0));
            assert_eq!(result, vec![Some(0), None]);
        }

        #[test]
        fn test_jaro_many_with_cutoff() {
            let candidates = vec![
                "MARHTA".to_string(),
                "XXXXXX".to_string(),
                "MARTHA".to_string(),
            ];
            // min_similarity = 0.9: "MARHTA" (high sim ~0.94) passes, "XXXXXX" (low sim) -> 0.0
            let result = jaro_many("MARTHA".to_string(), candidates, Some(0.9));
            assert!(result[0] > 0.9);
            assert_eq!(result[1], 0.0);
            assert_eq!(result[2], 1.0);
        }

        #[test]
        fn test_jaro_many_without_cutoff() {
            let candidates = vec!["MARHTA".to_string(), "XXXXXX".to_string()];
            let result = jaro_many("MARTHA".to_string(), candidates, None);
            assert!(result[0] > 0.9);
            assert!(result[1] < 0.9); // without cutoff, actual (low) score is returned
        }

        #[test]
        fn test_jaro_winkler_many_with_cutoff() {
            let candidates = vec![
                "MARHTA".to_string(),
                "XXXXXX".to_string(),
                "MARTHA".to_string(),
            ];
            let result = jaro_winkler_many("MARTHA".to_string(), candidates, Some(0.9));
            assert!(result[0] > 0.9);
            assert_eq!(result[1], 0.0);
            assert_eq!(result[2], 1.0);
        }

        #[test]
        fn test_jaro_winkler_many_without_cutoff() {
            let candidates = vec!["MARHTA".to_string(), "XXXXXX".to_string()];
            let result = jaro_winkler_many("MARTHA".to_string(), candidates, None);
            assert!(result[0] > 0.9);
            assert!(result[1] >= 0.0);
        }

        #[test]
        fn test_normalized_levenshtein_many_with_cutoff() {
            let candidates = vec![
                "kitten".to_string(),
                "abcdef".to_string(),
                "kittens".to_string(),
            ];
            // min_similarity = 0.8: "kitten" (identical, 1.0) passes, "abcdef" (low sim) -> 0.0
            let result = normalized_levenshtein_many("kitten".to_string(), candidates, Some(0.8));
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 0.0);
            assert!(result[2] > 0.8);
        }

        #[test]
        fn test_normalized_levenshtein_many_without_cutoff() {
            let candidates = vec!["kitten".to_string(), "abcdef".to_string()];
            let result = normalized_levenshtein_many("kitten".to_string(), candidates, None);
            assert_eq!(result[0], 1.0);
            assert!(result[1] >= 0.0);
        }

        #[test]
        fn test_levenshtein_many_cutoff_zero() {
            // max_distance = 0 means only exact matches pass
            let candidates = vec!["kitten".to_string(), "sitting".to_string()];
            let result = levenshtein_many("kitten".to_string(), candidates, Some(0));
            assert_eq!(result[0], 0); // exact match
            assert_eq!(result[1], 1); // sentinel = 0 + 1 = 1
        }

        #[test]
        fn test_similarity_cutoff_at_one() {
            // min_similarity = 1.0 means only identical strings pass
            let candidates = vec!["MARTHA".to_string(), "MARHTA".to_string()];
            let result = jaro_many("MARTHA".to_string(), candidates, Some(1.0));
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 0.0);
        }
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

            // --- Hamming properties ---

            #[test]
            fn hamming_identity(a in ".*") {
                prop_assert_eq!(hamming(a.clone(), a), Some(0));
            }

            #[test]
            fn hamming_symmetry(a in ".*", b in ".*") {
                prop_assert_eq!(
                    hamming(a.clone(), b.clone()),
                    hamming(b, a)
                );
            }

            #[test]
            fn hamming_none_for_different_lengths(a in ".{1,10}", b in ".{11,20}") {
                prop_assert_eq!(hamming(a, b), None);
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
            let result = token_sort_ratio_many("New York Mets".to_string(), candidates, None);
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
        fn test_one_empty_returns_zero() {
            assert_eq!(token_set_ratio("".into(), "foo".into()), 0.0);
            assert_eq!(token_set_ratio("foo".into(), "".into()), 0.0);
            assert_eq!(token_set_ratio("  ".into(), "foo".into()), 0.0);
            assert_eq!(token_set_ratio("foo".into(), "  ".into()), 0.0);
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
            let result = token_set_ratio_many("Mariners vs Yankees".to_string(), candidates, None);
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
            let result = partial_ratio_many("hello".to_string(), candidates, None);
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
            let result = weighted_ratio_many("hello".to_string(), candidates, None);
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

    mod many_equivalence_tests {
        use super::*;

        /// Candidates shared across all equivalence tests.
        fn test_candidates() -> Vec<String> {
            vec![
                "hello world".to_string(),
                "Mets New York".to_string(),
                "".to_string(),
                "completely different string".to_string(),
                "東京 タワー".to_string(),
                "hello".to_string(),
                "  extra   spaces  ".to_string(),
            ]
        }

        #[test]
        fn token_sort_ratio_many_matches_impl() {
            let reference = "New York Mets".to_string();
            let candidates = test_candidates();
            let many_results = token_sort_ratio_many(reference.clone(), candidates.clone(), None);
            let individual: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::token_sort_ratio_impl(&reference, c))
                .collect();
            assert_eq!(many_results.len(), individual.len());
            for (i, (m, s)) in many_results.iter().zip(individual.iter()).enumerate() {
                assert!(
                    (m - s).abs() < f64::EPSILON,
                    "token_sort_ratio mismatch at index {}: many={}, single={}",
                    i,
                    m,
                    s
                );
            }
        }

        #[test]
        fn token_set_ratio_many_matches_impl() {
            let reference = "Mariners vs Yankees".to_string();
            let candidates = test_candidates();
            let many_results = token_set_ratio_many(reference.clone(), candidates.clone(), None);
            let individual: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::token_set_ratio_impl(&reference, c))
                .collect();
            assert_eq!(many_results.len(), individual.len());
            for (i, (m, s)) in many_results.iter().zip(individual.iter()).enumerate() {
                assert!(
                    (m - s).abs() < f64::EPSILON,
                    "token_set_ratio mismatch at index {}: many={}, single={}",
                    i,
                    m,
                    s
                );
            }
        }

        #[test]
        fn partial_ratio_many_matches_impl() {
            let reference = "hello".to_string();
            let candidates = test_candidates();
            let many_results = partial_ratio_many(reference.clone(), candidates.clone(), None);
            let individual: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::partial_ratio_impl(&reference, c))
                .collect();
            assert_eq!(many_results.len(), individual.len());
            for (i, (m, s)) in many_results.iter().zip(individual.iter()).enumerate() {
                assert!(
                    (m - s).abs() < f64::EPSILON,
                    "partial_ratio mismatch at index {}: many={}, single={}",
                    i,
                    m,
                    s
                );
            }
        }

        #[test]
        fn weighted_ratio_many_matches_impl() {
            let reference = "New York Mets".to_string();
            let candidates = test_candidates();
            let many_results = weighted_ratio_many(reference.clone(), candidates.clone(), None);
            let individual: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::weighted_ratio_impl(&reference, c))
                .collect();
            assert_eq!(many_results.len(), individual.len());
            for (i, (m, s)) in many_results.iter().zip(individual.iter()).enumerate() {
                assert!(
                    (m - s).abs() < f64::EPSILON,
                    "weighted_ratio mismatch at index {}: many={}, single={}",
                    i,
                    m,
                    s
                );
            }
        }

        #[test]
        fn many_with_empty_reference() {
            let reference = "".to_string();
            let candidates = vec!["hello".to_string(), "".to_string(), "world".to_string()];

            let tsr = token_sort_ratio_many(reference.clone(), candidates.clone(), None);
            let tsr_expected: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::token_sort_ratio_impl(&reference, c))
                .collect();
            assert_eq!(tsr, tsr_expected);

            let tsetr = token_set_ratio_many(reference.clone(), candidates.clone(), None);
            let tsetr_expected: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::token_set_ratio_impl(&reference, c))
                .collect();
            assert_eq!(tsetr, tsetr_expected);

            let pr = partial_ratio_many(reference.clone(), candidates.clone(), None);
            let pr_expected: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::partial_ratio_impl(&reference, c))
                .collect();
            assert_eq!(pr, pr_expected);

            let wr = weighted_ratio_many(reference.clone(), candidates.clone(), None);
            let wr_expected: Vec<f64> = candidates
                .iter()
                .map(|c| core_dist::weighted_ratio_impl(&reference, c))
                .collect();
            assert_eq!(wr, wr_expected);
        }

        #[test]
        fn many_with_empty_candidates() {
            let reference = "hello world".to_string();
            let candidates: Vec<String> = vec![];

            assert!(token_sort_ratio_many(reference.clone(), candidates.clone(), None).is_empty());
            assert!(token_set_ratio_many(reference.clone(), candidates.clone(), None).is_empty());
            assert!(partial_ratio_many(reference.clone(), candidates.clone(), None).is_empty());
            assert!(weighted_ratio_many(reference, candidates, None).is_empty());
        }
    }

    mod threshold_many_tests {
        use super::*;

        #[test]
        fn sorensen_dice_many_threshold_filters_low_scores() {
            let candidates = vec!["night".to_string(), "nacht".to_string(), "xyz".to_string()];
            let result = sorensen_dice_many("night".to_string(), candidates, Some(0.9));
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 0.0); // below threshold
            assert_eq!(result[2], 0.0); // below threshold
        }

        #[test]
        fn sorensen_dice_many_bigram_matches_strsim() {
            let candidates = vec![
                "night".to_string(),
                "nacht".to_string(),
                "abc".to_string(),
                "".to_string(),
            ];
            let with_threshold = sorensen_dice_many("night".to_string(), candidates.clone(), None);
            let expected: Vec<f64> = candidates
                .iter()
                .map(|c| strsim::sorensen_dice("night", c))
                .collect();
            for (a, b) in with_threshold.iter().zip(expected.iter()) {
                assert!((a - b).abs() < 1e-10, "expected {b}, got {a}");
            }
        }

        #[test]
        fn token_sort_ratio_many_threshold_filters() {
            let candidates = vec!["New York Mets".to_string(), "xyz abc".to_string()];
            let result = token_sort_ratio_many("New York Mets".to_string(), candidates, Some(0.9));
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 0.0);
        }

        #[test]
        fn token_set_ratio_many_threshold_filters() {
            let candidates = vec!["Mariners vs Yankees".to_string(), "xyz".to_string()];
            let result =
                token_set_ratio_many("Mariners vs Yankees".to_string(), candidates, Some(0.9));
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 0.0);
        }

        #[test]
        fn partial_ratio_many_threshold_filters() {
            let candidates = vec!["hello world".to_string(), "xyz".to_string()];
            let result = partial_ratio_many("hello".to_string(), candidates, Some(0.9));
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 0.0);
        }

        #[test]
        fn weighted_ratio_many_threshold_filters() {
            let candidates = vec!["hello".to_string(), "xyz".to_string()];
            let result = weighted_ratio_many("hello".to_string(), candidates, Some(0.9));
            assert_eq!(result[0], 1.0);
            assert_eq!(result[1], 0.0);
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
