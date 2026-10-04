mod fuzzy_index;
mod keyed_index;
mod keys;
pub mod serialization;

pub use fuzzy_index::FuzzyIndexCore;
pub use keyed_index::KeyedFuzzyIndexCore;
pub use keys::{SearchKeysOptions, search_keys_impl};

use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::Ordering;

use nucleo_matcher::pattern::{Atom, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str, Utf32String, chars};

/// Classification of how a query matched an item.
///
/// Derived from the matched character positions:
/// - **Exact**: all positions consecutive from index 0, covering every character in the item.
/// - **Prefix**: all positions consecutive from index 0, but the item is longer.
/// - **Contains**: all positions consecutive (a substring match), not starting at 0.
/// - **Fuzzy**: positions have gaps (character-level fuzzy match).
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

/// A single fuzzy search result with the matched item and its score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The original string that matched.
    pub item: String,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// The index of the item in the original input array.
    pub index: u32,
    /// Indices of matched characters in the item string.
    /// Empty unless `include_positions` is set to true.
    pub positions: Vec<u32>,
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `include_positions` is set to true.
    pub match_type: Option<MatchType>,
}

/// A lightweight search result containing only index and score (no item string).
///
/// Use this when you maintain your own data array and only need the index
/// to look up the original item. Avoids String cloning overhead.
#[derive(Debug, Clone)]
pub struct IndexSearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// Indices of matched characters in the item string.
    /// Empty unless `include_positions` is set to true.
    pub positions: Vec<u32>,
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `include_positions` is set to true.
    pub match_type: Option<MatchType>,
}

/// A single result from multi-key fuzzy search.
#[derive(Debug, Clone)]
pub struct KeySearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The combined weighted score normalized to 0.0-1.0 range.
    pub score: f64,
    /// Per-key scores in the same order as the input keys.
    /// A score of 0.0 means the item did not match on that key.
    pub key_scores: Vec<f64>,
}

/// Classify the match type from the matched character positions.
///
/// `item_char_count` is the length of the haystack the positions index into
/// (one entry per grapheme for non-ASCII text, see [`utf32_haystack`]).
pub fn classify_match(positions: &[u32], item_char_count: usize) -> MatchType {
    if positions.is_empty() {
        return MatchType::Fuzzy;
    }

    let is_consecutive = positions.len() == 1 || positions.windows(2).all(|w| w[1] == w[0] + 1);

    if is_consecutive {
        if positions[0] == 0 && positions.len() == item_char_count {
            MatchType::Exact
        } else if positions[0] == 0 {
            MatchType::Prefix
        } else {
            MatchType::Contains
        }
    } else {
        MatchType::Fuzzy
    }
}

/// Convert the `is_case_sensitive` flag into a `CaseMatching` variant.
pub fn resolve_case_matching(is_case_sensitive: Option<bool>) -> CaseMatching {
    match is_case_sensitive {
        Some(true) => CaseMatching::Respect,
        _ => CaseMatching::Smart,
    }
}

thread_local! {
    /// Reusable Matcher for standalone search/closest calls.
    /// Avoids allocating internal scoring matrices on every invocation.
    /// FuzzyIndex has its own Matcher, so this is only for the standalone path.
    static STANDALONE_MATCHER: RefCell<Matcher> = RefCell::new(Matcher::new(Config::DEFAULT));
}

// ─── Query parsing ──────────────────────────────────────────────────────────

/// Longest query term, in characters, that can be scored.
///
/// nucleo-matcher 0.3 scores with `u16`. A term of `m` characters scores at
/// most `16` per matched character plus a bonus of at most `10` per character
/// (`20` on the first one), i.e. `26 * m + 10`, which first exceeds
/// `u16::MAX` at 2521 characters: the score would wrap around in release
/// builds (and panic in debug builds). A query containing a longer term
/// therefore matches nothing.
pub const MAX_TERM_CHARS: usize = 2520;

/// Replace every Unicode whitespace character in a query with an ASCII space.
///
/// nucleo only splits terms on ASCII spaces, so ideographic spaces (U+3000,
/// typed by Japanese/Chinese input methods), no-break spaces, tabs and
/// newlines (from pasted text) would otherwise become part of a term and the
/// query would match nothing. A backslash keeps escaping the space it now
/// precedes: `foo\<U+3000>bar` becomes the single term `foo bar`, exactly like
/// `foo\ bar`.
pub fn normalize_query_whitespace(query: &str) -> Cow<'_, str> {
    if query.chars().all(|c| c == ' ' || !c.is_whitespace()) {
        return Cow::Borrowed(query);
    }
    Cow::Owned(
        query
            .chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect(),
    )
}

/// Parse a query exactly as every search function does: Unicode whitespace
/// is normalized to ASCII spaces (see [`normalize_query_whitespace`]), then
/// nucleo's extended syntax (`^`, `$`, `'`, `!` and `\` escapes) is applied.
pub fn parse_query(query: &str, case_matching: CaseMatching) -> Pattern {
    Pattern::parse(
        &normalize_query_whitespace(query),
        case_matching,
        Normalization::Smart,
    )
}

/// Whether a query contains no search term at all.
///
/// True for empty and whitespace-only queries, and for queries made only of
/// syntax characters (`^`, `'`, `$`, `!`, `^$`, ...): nucleo drops terms that
/// are empty once their syntax is removed. Such queries return no results
/// (or every item, with `returnAllOnEmpty`).
pub fn is_empty_query(query: &str) -> bool {
    parse_query(query, CaseMatching::Smart).atoms.is_empty()
}

/// A query parsed and prepared for scoring.
pub struct QueryPlan {
    /// The parsed nucleo pattern.
    pub pattern: Pattern,
    /// Raw score of a perfect match, used to normalize scores to 0.0-1.0.
    pub max_score: f64,
    /// Character mask every matching item must contain (see [`compute_char_mask`]).
    pub char_mask: u64,
}

impl QueryPlan {
    /// Parse `query` with [`parse_query`].
    ///
    /// Returns `None` when the query cannot match anything: when it has no
    /// search term ([`is_empty_query`]) or when one of its terms is longer
    /// than [`MAX_TERM_CHARS`].
    pub fn new(query: &str, case_matching: CaseMatching, matcher: &mut Matcher) -> Option<Self> {
        let pattern = parse_query(query, case_matching);
        if pattern.atoms.is_empty()
            || pattern
                .atoms
                .iter()
                .any(|atom| atom.needle_text().len() > MAX_TERM_CHARS)
        {
            return None;
        }
        let max_score = pattern_max_score(&pattern, matcher);
        let char_mask = pattern_char_mask(&pattern);
        Some(Self {
            pattern,
            max_score,
            char_mask,
        })
    }

    /// Normalize a raw nucleo score to the 0.0-1.0 range.
    pub fn normalize(&self, raw_score: u32) -> f64 {
        (raw_score as f64 / self.max_score).min(1.0)
    }
}

/// Raw score of the best possible match for a pattern: the sum, over its
/// positive terms, of each term matched against its own text.
///
/// Every term is matched against a haystack that satisfies its own anchors
/// (the term's text is its own prefix, suffix and exact match), so `bar$`,
/// `^foo` or `foo\ bar` get the same maximum as `bar`, `foo` or `foo bar`.
/// Negative terms (`!term`) never contribute to a score and are ignored.
/// The result is at least 1 so normalization never divides by zero.
pub fn pattern_max_score(pattern: &Pattern, matcher: &mut Matcher) -> f64 {
    let total: u32 = pattern
        .atoms
        .iter()
        .filter(|atom| !atom.negative)
        .map(|atom| atom_max_score(atom, matcher))
        .sum();
    total.max(1) as f64
}

fn atom_max_score(atom: &Atom, matcher: &mut Matcher) -> u32 {
    let needle = atom.needle_text();
    if needle.len() > MAX_TERM_CHARS {
        // Scoring would overflow nucleo's u16 score; use the theoretical bound.
        return u32::try_from(needle.len())
            .unwrap_or(u32::MAX)
            .saturating_mul(26)
            .saturating_add(10);
    }
    atom.score(needle, matcher).map_or(0, u32::from)
}

/// Compute the maximum possible score for a pattern.
///
/// Kept for existing callers; the `query` argument is not used. Prefer
/// [`pattern_max_score`].
pub fn compute_max_score(_query: &str, pattern: &Pattern, matcher: &mut Matcher) -> f64 {
    pattern_max_score(pattern, matcher)
}

// ─── Character masks ────────────────────────────────────────────────────────
//
// A 64-bit character-presence mask lets the indexes skip items that cannot
// match before running nucleo. It must never reject an item nucleo would
// match, so it is derived from the characters nucleo actually compares:
//
// * a query contributes the characters of its positive terms' needles
//   (after nucleo's parsing, grapheme segmentation and escapes);
// * an item contributes every form nucleo may compare each of its
//   characters as: the character itself, its simple case folding, its
//   normalization (`é` -> `e`) and the case folding of that normalization.
//
// Both sides map characters to bits through the same folding, so a needle
// character equal to any of those forms always finds its bit in the item.
// The exhaustive test in `tests/char_mask.rs` checks this for every Unicode
// scalar value against nucleo itself.

/// Map a folded character to one of 64 bits: `a`-`z` -> 0-25, `0`-`9` ->
/// 26-35, everything else hashed into 36-63.
#[inline]
fn mask_bit(c: char) -> u64 {
    let bit = match c {
        'a'..='z' => c as u32 - 'a' as u32,
        '0'..='9' => 26 + (c as u32 - '0' as u32),
        _ => 36 + c as u32 % 28,
    };
    1 << bit
}

/// Mask bit of a query (needle) character.
#[inline]
fn needle_char_bit(c: char) -> u64 {
    if c.is_ascii() {
        mask_bit(c.to_ascii_lowercase())
    } else {
        mask_bit(chars::to_lower_case(chars::normalize(c)))
    }
}

/// Mask bits of an item (haystack) character: one bit for every form nucleo
/// may compare it as.
#[inline]
fn haystack_char_bits(c: char) -> u64 {
    if c.is_ascii() {
        // nucleo only ever lowercases ASCII characters.
        return mask_bit(c.to_ascii_lowercase());
    }
    let normalized = chars::normalize(c);
    needle_char_bit(c)
        | needle_char_bit(chars::to_lower_case(c))
        | needle_char_bit(normalized)
        | needle_char_bit(chars::to_lower_case(normalized))
}

/// Character mask of an item string.
///
/// Considers every codepoint of `s` (a superset of the grapheme-leading
/// codepoints nucleo matches against), so it never rejects a match.
pub fn compute_char_mask(s: &str) -> u64 {
    if s.is_ascii() {
        return s
            .bytes()
            .fold(0, |mask, b| mask | mask_bit(b.to_ascii_lowercase() as char));
    }
    s.chars().fold(0, |mask, c| mask | haystack_char_bits(c))
}

/// Character mask of a haystack, as matched by nucleo (see [`utf32_haystack`]).
pub fn haystack_char_mask(haystack: Utf32Str<'_>) -> u64 {
    match haystack {
        Utf32Str::Ascii(bytes) => bytes.iter().fold(0, |mask, &b| {
            mask | mask_bit(b.to_ascii_lowercase() as char)
        }),
        Utf32Str::Unicode(chars) => chars
            .iter()
            .fold(0, |mask, &c| mask | haystack_char_bits(c)),
    }
}

/// Character mask every item matching `pattern` contains: the characters of
/// its positive terms. Negative terms (`!term`) are ignored.
pub fn pattern_char_mask(pattern: &Pattern) -> u64 {
    pattern
        .atoms
        .iter()
        .filter(|atom| !atom.negative)
        .flat_map(|atom| atom.needle_text().chars())
        .fold(0, |mask, c| mask | needle_char_bit(c))
}

/// Character mask of a query parsed with `Pattern::parse(query, ..)`.
///
/// The query is parsed as-is (without [`normalize_query_whitespace`]) to
/// match callers that parse the raw query themselves. Prefer
/// [`pattern_char_mask`] on the pattern actually used for matching.
pub fn compute_query_mask(query: &str) -> u64 {
    pattern_char_mask(&Pattern::parse(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
    ))
}

/// Convert an item to the haystack nucleo matches against.
///
/// Produces the same representation as `Utf32String::from` (which
/// `FuzzyIndex` stores): ASCII text as bytes, anything else as the first
/// codepoint of each grapheme. `Utf32Str::new` differs for non-ASCII text
/// whose graphemes all start with an ASCII character (e.g. NFD `école`): it
/// returns the raw UTF-8 bytes, so positions and scores would count the
/// bytes of combining marks.
pub fn utf32_haystack<'a>(s: &'a str, buf: &'a mut Vec<char>) -> Utf32Str<'a> {
    if s.is_ascii() {
        Utf32Str::Ascii(s.as_bytes())
    } else {
        buf.clear();
        buf.extend(chars::graphemes(s));
        Utf32Str::Unicode(buf)
    }
}

// ─── Search ─────────────────────────────────────────────────────────────────

/// The items a search runs over.
#[derive(Clone, Copy)]
pub(crate) enum Corpus<'a> {
    /// Plain strings (standalone `search`), converted to haystacks on the fly.
    Strings(&'a [String]),
    /// Pre-converted haystacks and character masks (`FuzzyIndex`); items
    /// whose mask lacks a bit of the query's mask are never scored.
    Indexed {
        items: &'a [String],
        haystacks: &'a [Utf32String],
        char_masks: &'a [u64],
    },
}

impl<'a> Corpus<'a> {
    fn items(self) -> &'a [String] {
        match self {
            Corpus::Strings(items) | Corpus::Indexed { items, .. } => items,
        }
    }

    fn haystack<'b>(self, index: usize, buf: &'b mut Vec<char>) -> Utf32Str<'b>
    where
        'a: 'b,
    {
        match self {
            Corpus::Strings(items) => utf32_haystack(&items[index], buf),
            Corpus::Indexed { haystacks, .. } => haystacks[index].slice(..),
        }
    }
}

/// A ranked match, before it is converted to a public result type.
pub(crate) struct RankedMatch {
    pub index: u32,
    pub score: f64,
    pub positions: Vec<u32>,
    pub match_type: Option<MatchType>,
}

/// Matches of a search plus, optionally, every matching index.
pub(crate) struct SearchOutcome {
    /// Top-k matches, best first.
    pub matches: Vec<RankedMatch>,
    /// Ascending indices of every item scoring at or above `min_score`
    /// (before `max_results` truncation), when requested and when fewer than
    /// half of the items matched; `None` otherwise.
    pub all_matching: Option<Vec<u32>>,
}

/// Result-shaping options of a search.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SearchParams {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub include_positions: bool,
}

/// The search algorithm shared by standalone search and `FuzzyIndex`.
///
/// When `candidates` is set, only those items (ascending indices) are scored.
pub(crate) fn search_core(
    plan: &QueryPlan,
    matcher: &mut Matcher,
    corpus: Corpus<'_>,
    candidates: Option<&[u32]>,
    params: SearchParams,
    collect_matching: bool,
) -> SearchOutcome {
    let SearchParams {
        max_results,
        min_score,
        include_positions,
    } = params;
    let items = corpus.items();
    let threshold = min_score.unwrap_or(0.0);
    let mut buf = Vec::new();

    // Pass 1: score items, keeping only (index, score) — no String cloning.
    let mut scored: Vec<(u32, f64)> = Vec::new();
    let mut score_item = |index: u32| {
        let haystack = corpus.haystack(index as usize, &mut buf);
        if let Some(raw_score) = plan.pattern.score(haystack, matcher) {
            let score = plan.normalize(raw_score);
            if score >= threshold {
                scored.push((index, score));
            }
        }
    };
    let query_mask = plan.char_mask;
    match (corpus, candidates) {
        (Corpus::Indexed { char_masks, .. }, None) => {
            // A tight loop over the masks: they usually reject most items.
            for (index, &mask) in char_masks.iter().enumerate() {
                if mask & query_mask == query_mask {
                    score_item(index as u32);
                }
            }
        }
        (Corpus::Indexed { char_masks, .. }, Some(candidates)) => {
            for &index in candidates {
                if char_masks[index as usize] & query_mask == query_mask {
                    score_item(index);
                }
            }
        }
        (Corpus::Strings(_), Some(candidates)) => candidates.iter().for_each(|&i| score_item(i)),
        (Corpus::Strings(items), None) => (0..items.len() as u32).for_each(score_item),
    }

    // Matching indices for the incremental cache, only when the match set is
    // meaningfully smaller than the dataset: when most items match (e.g.
    // short queries) narrowing the next search would not pay off.
    let all_matching = (collect_matching && scored.len() < items.len() / 2)
        .then(|| scored.iter().map(|&(index, _)| index).collect());

    // Sort by score descending, with shorter items first as tiebreaker,
    // then by original index for fully deterministic ordering.
    let cmp = |a: &(u32, f64), b: &(u32, f64)| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| items[a.0 as usize].len().cmp(&items[b.0 as usize].len()))
            .then_with(|| a.0.cmp(&b.0))
    };

    // Top-k selection: quickselect O(n) + sort O(k log k) instead of a full
    // O(n log n) sort when maxResults is set.
    if let Some(max) = max_results {
        let k = max as usize;
        if scored.len() > k {
            scored.select_nth_unstable_by(k, cmp);
            scored.truncate(k);
        }
    }
    scored.sort_unstable_by(cmp);

    // Pass 2: positions only for the final top-k items.
    let matches = scored
        .into_iter()
        .map(|(index, score)| {
            let (positions, match_type) = if include_positions {
                let haystack = corpus.haystack(index as usize, &mut buf);
                let mut positions = Vec::new();
                plan.pattern.indices(haystack, matcher, &mut positions);
                positions.sort_unstable();
                positions.dedup();
                let match_type = classify_match(&positions, haystack.len());
                (positions, Some(match_type))
            } else {
                (Vec::new(), None)
            };
            RankedMatch {
                index,
                score,
                positions,
                match_type,
            }
        })
        .collect();

    SearchOutcome {
        matches,
        all_matching,
    }
}

/// Fuzzy search over a borrowed slice of items (the standalone `search`).
///
/// `FuzzyIndex` runs the same algorithm over pre-computed data and returns
/// identical results.
pub fn search_over_items(
    query: &str,
    items: &[String],
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    if items.is_empty() {
        return Vec::new();
    }
    STANDALONE_MATCHER.with(|cell| {
        let mut matcher = cell.borrow_mut();
        let Some(plan) = QueryPlan::new(query, case_matching, &mut matcher) else {
            return Vec::new();
        };
        search_core(
            &plan,
            &mut matcher,
            Corpus::Strings(items),
            None,
            SearchParams {
                max_results,
                min_score,
                include_positions,
            },
            false,
        )
        .matches
        .into_iter()
        .map(|m| SearchResult {
            item: items[m.index as usize].clone(),
            score: m.score,
            index: m.index,
            positions: m.positions,
            match_type: m.match_type,
        })
        .collect()
    })
}

/// Internal search implementation used by both the napi export and tests.
pub fn search_impl(
    query: String,
    items: Vec<String>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    search_over_items(
        &query,
        &items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    )
}
