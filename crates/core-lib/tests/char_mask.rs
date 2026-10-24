//! The character-mask pre-filter must never reject an item nucleo matches.
//!
//! nucleo compares a needle character with a haystack character after
//! optionally normalizing (`é` -> `e`) and case folding the haystack one, so
//! the mask of an item has to cover every form each of its characters can be
//! compared as. This checks it exhaustively: for every Unicode scalar value
//! as a one-character item, and every needle character nucleo could compare
//! it to, whenever nucleo reports a match the item's mask must contain the
//! query's mask.

use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String, chars};
use rapid_fuzzy_core::search::{
    compute_char_mask, compute_query_mask, haystack_char_mask, parse_query, pattern_char_mask,
};

fn single_char_pattern(needle: char, case_matching: CaseMatching) -> Pattern {
    let mut pattern = Pattern::default();
    pattern.atoms.push(Atom::new(
        &needle.to_string(),
        case_matching,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    ));
    pattern
}

/// Every character nucleo might compare `c` as, plus the standard library's
/// case mappings for good measure.
fn comparison_candidates(c: char) -> Vec<char> {
    let normalized = chars::normalize(c);
    let mut candidates = vec![
        c,
        normalized,
        chars::to_lower_case(c),
        chars::to_lower_case(normalized),
        chars::normalize(chars::to_lower_case(c)),
    ];
    candidates.extend(c.to_lowercase());
    candidates.extend(c.to_uppercase());
    candidates.extend(normalized.to_uppercase());
    candidates.sort_unstable();
    candidates.dedup();
    candidates
}

#[test]
fn item_mask_covers_every_char_nucleo_matches() {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut matches_checked = 0usize;
    for code in 0..=char::MAX as u32 {
        let Some(item_char) = char::from_u32(code) else {
            continue; // surrogates
        };
        let item = item_char.to_string();
        let haystack = Utf32String::from(item.as_str());
        let item_mask = haystack_char_mask(haystack.slice(..));
        assert_eq!(
            compute_char_mask(&item),
            item_mask,
            "compute_char_mask and haystack_char_mask disagree for {item_char:?} (U+{code:04X})"
        );

        for needle in comparison_candidates(item_char) {
            for case_matching in [CaseMatching::Smart, CaseMatching::Respect] {
                let pattern = single_char_pattern(needle, case_matching);
                if pattern.score(haystack.slice(..), &mut matcher).is_none() {
                    continue;
                }
                let query_mask = pattern_char_mask(&pattern);
                assert_eq!(
                    item_mask & query_mask,
                    query_mask,
                    "nucleo matches needle {needle:?} (U+{:04X}, {case_matching:?}) against item \
                     {item_char:?} (U+{code:04X}) but the item mask lacks its bit",
                    needle as u32,
                );
                matches_checked += 1;
            }
        }
    }
    // Every character matches at least itself in both case modes.
    assert!(
        matches_checked > 2 * 1_100_000,
        "only {matches_checked} matches checked"
    );
}

#[test]
fn masks_fold_case_and_diacritics_like_nucleo() {
    // Lowercase query characters match uppercase and accented item characters.
    for (item, query) in [
        ("Łódź", "łódź"),
        ("Łódź", "lodz"),
        ("ŠŽĆĐŌ", "šžćđō"),
        ("Москва", "москва"),
        ("ΣΟΦΙΑ", "σοφια"),
        ("\u{212A}elvin", "kelvin"),
        ("café", "cafe"),
        ("Ärger", "arger"),
        ("e\u{301}cole", "ecole"),
    ] {
        let item_mask = compute_char_mask(item);
        let query_mask = pattern_char_mask(&parse_query(query, CaseMatching::Smart));
        assert_eq!(item_mask & query_mask, query_mask, "{query:?} vs {item:?}");
    }
}

#[test]
fn query_mask_ignores_syntax_and_negative_terms() {
    assert_eq!(compute_query_mask("^foo$"), compute_query_mask("foo"));
    assert_eq!(compute_query_mask("'foo"), compute_query_mask("foo"));
    assert_eq!(compute_query_mask("foo !bar"), compute_query_mask("foo"));
    assert_eq!(compute_query_mask("!bar"), 0);
    assert_eq!(compute_query_mask("^ $ ! '"), 0);
    // Escaped syntax characters are part of the needle.
    assert_eq!(compute_query_mask("foo\\$"), compute_query_mask("$foo"));
    assert_ne!(compute_query_mask("foo\\$"), compute_query_mask("foo"));
}
