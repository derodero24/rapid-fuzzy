//! Typo-tolerant matching (the `maxTypos` search option).
//!
//! nucleo only tolerates characters *missing* from a query: `tpyescript`
//! (two letters swapped) or `typozcript` (a wrong letter) match nothing in
//! `TypeScript`. With `maxTypos` set, a fuzzy term that nucleo does not match
//! is matched again with [frizbee](https://docs.rs/frizbee), a Smith-Waterman
//! matcher that allows a number of typos (substituted, extra or swapped
//! characters).
//!
//! nucleo stays the matcher of everything it matches, so a match without
//! typos keeps exactly the score it has without the option; the option only
//! adds matches. A term matched with typos scores frizbee's score relative to
//! a perfect frizbee match, scaled to the term's perfect nucleo score and
//! kept below it, so that a match with typos never ties an exact one.
//!
//! Only plain fuzzy terms tolerate typos: `'exact`, `^prefix`, `suffix$` and
//! `!negated` terms match exactly as before. Short terms tolerate fewer typos
//! (see [`term_typos`]), since a typo in a two-letter term matches almost
//! anything.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use frizbee::{
    CaseMatching as TypoCase, Config as TypoConfig, Matcher as TypoMatcher, UnicodeMatching,
};
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching};
use nucleo_matcher::{Utf32Str, chars};

/// Validate a `maxTypos` value coming from JavaScript (as a double).
///
/// Non-negative integers are accepted (values beyond `u16::MAX` are more
/// typos than any term can have and mean "as many as the term allows"),
/// `Infinity` means the same, and NaN, negative or fractional values are
/// rejected with the message both bindings report.
pub fn check_max_typos(value: f64) -> Result<u16, String> {
    if value == f64::INFINITY {
        return Ok(u16::MAX);
    }
    if value.is_nan() || value < 0.0 || value.fract() != 0.0 {
        return Err(format!(
            "maxTypos must be a non-negative integer or Infinity, got {}",
            crate::js_number(value)
        ));
    }
    Ok(if value >= f64::from(u16::MAX) {
        u16::MAX
    } else {
        value as u16
    })
}

/// Number of typos a fuzzy term of `term_chars` characters tolerates when the
/// search allows `max_typos`: one per four characters, so terms of up to
/// three characters match without typos, terms of four to seven characters
/// with one, terms of eight to eleven with two, and so on.
pub fn term_typos(max_typos: u16, term_chars: usize) -> u16 {
    max_typos.min(u16::try_from(term_chars / 4).unwrap_or(u16::MAX))
}

/// A fuzzy term matched with typos when nucleo does not match it.
struct TypoTerm {
    /// The term's needle, as nucleo parsed it (escapes resolved).
    needle: String,
    typos: u16,
    /// Whether nucleo matches the term case-insensitively (smart case).
    ignore_case: bool,
    /// Whether nucleo folds diacritics for the term (`e` matches `é`).
    normalize: bool,
    /// The term's perfect nucleo score: typo scores are scaled to it.
    cap: u32,
    /// frizbee's scores of a perfect match, for ASCII and non-ASCII
    /// haystacks (matched by different matchers, see [`term_matchers`]).
    perfect: [u16; 2],
}

/// The typo-tolerant part of a [`QueryPlan`](super::QueryPlan).
pub(crate) struct Typos {
    /// Identifies the plan in the per-thread matcher cache.
    id: u64,
    /// One entry per atom of the plan's pattern, in order: the atoms
    /// matched with typos, `None` for the others.
    terms: Vec<Option<TypoTerm>>,
    /// Characters of the query's mask an item may lack: every typo can make
    /// one character of a term absent from the item.
    pub(crate) allowed_missing: u32,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// The frizbee matchers of `term`: one for ASCII haystacks and one, always
/// using Unicode matching, for the folded non-ASCII haystacks (see
/// [`fold_haystack`]).
fn term_matchers(term: &TypoTerm) -> [TypoMatcher; 2] {
    let casing = if term.ignore_case {
        TypoCase::Ignore
    } else {
        TypoCase::Respect
    };
    let config = TypoConfig::default()
        .max_typos(Some(term.typos))
        .casing(casing);
    [
        TypoMatcher::new(
            term.needle.as_str(),
            &config.clone().unicode(UnicodeMatching::Smart),
        ),
        TypoMatcher::new(
            term.needle.as_str(),
            &config.unicode(UnicodeMatching::Always),
        ),
    ]
}

impl Typos {
    /// The typo-tolerant terms of `atoms` (with `caps[a]` the perfect nucleo
    /// score of atom `a`), or `None` when no term tolerates a typo.
    pub(crate) fn new(
        atoms: &[Atom],
        caps: &[u32],
        case_matching: CaseMatching,
        max_typos: u16,
    ) -> Option<Self> {
        if max_typos == 0 {
            return None;
        }
        let mut allowed_missing = 0u32;
        let terms: Vec<Option<TypoTerm>> = atoms
            .iter()
            .zip(caps)
            .map(|(atom, &cap)| {
                if atom.negative || atom.kind != AtomKind::Fuzzy {
                    return None;
                }
                let needle: String = atom.needle_text().chars().collect();
                let typos = term_typos(max_typos, needle.chars().count());
                if typos == 0 {
                    return None;
                }
                allowed_missing += u32::from(typos);
                let mut term = TypoTerm {
                    ignore_case: case_matching != CaseMatching::Respect
                        && !needle.chars().any(chars::is_upper_case),
                    normalize: needle.chars().all(|c| chars::normalize(c) == c),
                    needle,
                    typos,
                    cap: cap.max(1),
                    perfect: [1; 2],
                };
                let [mut ascii, mut unicode] = term_matchers(&term);
                let perfect = |m: &mut TypoMatcher| {
                    m.match_one(term.needle.as_str(), 0)
                        .map_or(1, |m| m.score.max(1))
                };
                term.perfect = [perfect(&mut ascii), perfect(&mut unicode)];
                Some(term)
            })
            .collect();
        terms.iter().any(Option::is_some).then(|| Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            terms,
            allowed_missing,
        })
    }

    /// Number of typos atom `a` tolerates.
    pub(crate) fn allowed_typos(&self, a: usize) -> u32 {
        self.terms
            .get(a)
            .and_then(Option::as_ref)
            .map_or(0, |term| u32::from(term.typos))
    }

    /// Raw score, on nucleo's scale, of atom `a` matched with typos against
    /// `haystack`, or `None` when the atom does not tolerate typos or does
    /// not match even with them. At least 1 and below the atom's perfect
    /// score.
    pub(crate) fn score(&self, a: usize, haystack: Utf32Str<'_>) -> Option<u32> {
        let term = self.terms.get(a)?.as_ref()?;
        self.with_matcher(a, term, haystack, |matcher, text| {
            let score = matcher.match_one(text, 0)?.score;
            Some(scale(term, score, haystack))
        })
    }

    /// Append the positions (indices into `haystack`) atom `a` matches with
    /// typos to `out`, returning whether it matched.
    pub(crate) fn indices(&self, a: usize, haystack: Utf32Str<'_>, out: &mut Vec<u32>) -> bool {
        let Some(term) = self.terms.get(a).and_then(Option::as_ref) else {
            return false;
        };
        self.with_matcher(a, term, haystack, |matcher, text| {
            let found = matcher.match_one_indices(text, 0)?;
            match haystack {
                Utf32Str::Ascii(_) => out.extend(found.indices),
                Utf32Str::Unicode(_) => {
                    // frizbee reports byte offsets into the folded text,
                    // whose characters are the haystack's one for one.
                    out.extend(found.indices.iter().filter_map(|&byte| {
                        let prefix = text.get(..byte as usize)?;
                        u32::try_from(prefix.chars().count()).ok()
                    }));
                }
            }
            Some(())
        })
        .is_some()
    }

    /// Run `f` with this thread's frizbee matcher for atom `a` and the text
    /// it matches `haystack` as.
    fn with_matcher<R>(
        &self,
        a: usize,
        term: &TypoTerm,
        haystack: Utf32Str<'_>,
        f: impl FnOnce(&mut TypoMatcher, &str) -> Option<R>,
    ) -> Option<R> {
        MATCHERS.with(|cell| match cell.try_borrow_mut() {
            Ok(mut cache) => cache.run(self, a, term, haystack, f),
            // Re-entrant use (not expected): use temporary matchers.
            Err(_) => MatcherCache::default().run(self, a, term, haystack, f),
        })
    }
}

/// Scale frizbee's `score` of `term` to nucleo's scale (see the module
/// documentation).
fn scale(term: &TypoTerm, score: u16, haystack: Utf32Str<'_>) -> u32 {
    let perfect = term.perfect[usize::from(matches!(haystack, Utf32Str::Unicode(_)))];
    let ratio = (f64::from(score) / f64::from(perfect)).min(1.0);
    // At most `cap - 1`, and at least 1 so that the term counts as matched.
    ((ratio * f64::from(term.cap)) as u32).clamp(1, term.cap.saturating_sub(1).max(1))
}

/// Write the text frizbee matches a non-ASCII haystack as to `out`: one
/// character per haystack character (the first codepoint of each grapheme,
/// see [`utf32_haystack`](super::utf32_haystack)), folded as nucleo compares
/// it with `term`. frizbee only folds the case of ASCII letters itself.
fn fold_haystack(term: &TypoTerm, haystack: &[char], out: &mut String) {
    out.clear();
    out.extend(haystack.iter().map(|&c| {
        let c = if term.normalize {
            chars::normalize(c)
        } else {
            c
        };
        if term.ignore_case && !c.is_ascii() {
            chars::to_lower_case(c)
        } else {
            c
        }
    }));
}

/// The frizbee matchers of the plan a thread last matched with typos.
#[derive(Default)]
struct MatcherCache {
    /// The [`Typos::id`] the matchers were built for.
    id: Option<u64>,
    matchers: Vec<Option<[TypoMatcher; 2]>>,
    /// Folded text of the current non-ASCII haystack.
    text: String,
}

impl MatcherCache {
    fn run<R>(
        &mut self,
        typos: &Typos,
        a: usize,
        term: &TypoTerm,
        haystack: Utf32Str<'_>,
        f: impl FnOnce(&mut TypoMatcher, &str) -> Option<R>,
    ) -> Option<R> {
        if self.id != Some(typos.id) {
            self.matchers = typos
                .terms
                .iter()
                .map(|term| term.as_ref().map(term_matchers))
                .collect();
            self.id = Some(typos.id);
        }
        let [ascii, unicode] = self.matchers.get_mut(a)?.as_mut()?;
        match haystack {
            Utf32Str::Ascii(bytes) => f(ascii, std::str::from_utf8(bytes).ok()?),
            Utf32Str::Unicode(chars) => {
                fold_haystack(term, chars, &mut self.text);
                f(unicode, &self.text)
            }
        }
    }
}

thread_local! {
    /// The frizbee matchers of this thread (see [`MatcherCache`]).
    static MATCHERS: RefCell<MatcherCache> = RefCell::new(MatcherCache::default());
}

#[cfg(test)]
mod tests {
    use super::{check_max_typos, term_typos};

    #[test]
    fn term_typos_grow_with_term_length() {
        assert_eq!(term_typos(2, 3), 0);
        assert_eq!(term_typos(2, 4), 1);
        assert_eq!(term_typos(2, 7), 1);
        assert_eq!(term_typos(2, 8), 2);
        assert_eq!(term_typos(2, 40), 2);
        assert_eq!(term_typos(0, 40), 0);
        assert_eq!(term_typos(u16::MAX, 12), 3);
    }

    #[test]
    fn check_max_typos_accepts_non_negative_integers() {
        assert_eq!(check_max_typos(0.0), Ok(0));
        assert_eq!(check_max_typos(-0.0), Ok(0));
        assert_eq!(check_max_typos(2.0), Ok(2));
        assert_eq!(check_max_typos(1e9), Ok(u16::MAX));
        assert_eq!(check_max_typos(f64::INFINITY), Ok(u16::MAX));
        for bad in [f64::NAN, -1.0, 0.5, f64::NEG_INFINITY] {
            assert!(check_max_typos(bad).is_err(), "{bad}");
        }
    }
}
