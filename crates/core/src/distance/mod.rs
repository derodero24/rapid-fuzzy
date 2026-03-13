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

/// Compute the Damerau-Levenshtein distance between two strings.
///
/// Like Levenshtein, but also considers transpositions of two adjacent
/// characters as a single edit.
#[napi]
pub fn damerau_levenshtein(a: String, b: String) -> u32 {
    strsim::damerau_levenshtein(&a, &b) as u32
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[napi]
pub fn jaro(a: String, b: String) -> f64 {
    strsim::jaro(&a, &b)
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// A modification of Jaro that gives more weight to common prefixes.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn jaro_winkler(a: String, b: String) -> f64 {
    strsim::jaro_winkler(&a, &b)
}

/// Compute the Sorensen-Dice coefficient between two strings.
///
/// Uses bigrams (pairs of consecutive characters) to measure similarity.
/// Returns a value between 0.0 and 1.0.
#[napi]
pub fn sorensen_dice(a: String, b: String) -> f64 {
    strsim::sorensen_dice(&a, &b)
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[napi]
pub fn normalized_levenshtein(a: String, b: String) -> f64 {
    strsim::normalized_levenshtein(&a, &b)
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
    fn test_jaro_winkler() {
        let score = jaro_winkler("MARTHA".into(), "MARHTA".into());
        assert!(score > 0.96);
    }

    #[test]
    fn test_sorensen_dice() {
        let score = sorensen_dice("night".into(), "nacht".into());
        assert!(score > 0.0 && score < 1.0);
    }
}
