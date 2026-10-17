mod cross_key;
mod fuzzy_index;
mod keyed_index;
mod keys;
pub mod serialization;

pub use fuzzy_index::FuzzyIndexCore;
pub use keyed_index::KeyedFuzzyIndexCore;
pub use keys::{
    KeyMatchMode, KeyScoreMode, SearchKeysOptions, invalid_match_mode, invalid_score_mode,
    search_keys_impl,
};

use std::borrow::Cow;
use std::cell::RefCell;

use nucleo_matcher::pattern::{Atom, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str, chars};

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
    /// The matched item, as converted to UTF-8 by the bindings: a lone UTF-16
    /// surrogate of the caller's string is U+FFFD here (see the bindings'
    /// `SearchResult.item`).
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
    /// The combined score (0.0-1.0) of the key scores, as set by the
    /// search's [`KeyScoreMode`].
    pub score: f64,
    /// Per-key scores in the same order as the input keys, as set by the
    /// search's [`KeyMatchMode`]. A score of 0.0 means the item did not
    /// match on that key.
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

/// Validate a `maxResults` value coming from JavaScript (as a double).
///
/// Non-negative integers are accepted (values beyond `u32::MAX` exceed any
/// array length and mean "no limit"), `Infinity` means no limit, and NaN,
/// negative or fractional values are rejected with the message both
/// bindings report.
pub fn check_max_results(value: f64) -> Result<Option<u32>, String> {
    if value == f64::INFINITY {
        return Ok(None);
    }
    if value.is_nan() || value < 0.0 || value.fract() != 0.0 {
        return Err(format!(
            "maxResults must be a non-negative integer or Infinity, got {}",
            crate::js_number(value)
        ));
    }
    Ok(Some(if value >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        value as u32
    }))
}

/// Validate the `index` argument of `FuzzyIndex.remove()` and
/// `KeyedFuzzyIndex.remove()` coming from JavaScript (as a double), exactly
/// like `FuzzyObjectIndex.remove()` validates its own:
///
/// - NaN, `±Infinity` and fractional values are rejected with the message
///   both bindings report (as a `RangeError`): reading them as a `u32`
///   wrapped them modulo 2^32, so `remove(NaN)` removed item 0;
/// - negative values and values beyond `u32::MAX` are out of range, like any
///   index not below the size of the index: `Ok(None)`, for which `remove()`
///   returns false without removing anything;
/// - any other value is the index of the item to remove.
///
/// A value that is not a number at all is rejected with
/// [`invalid_index_type`] (as a `TypeError`) before this check.
pub fn check_remove_index(value: f64) -> Result<Option<u32>, String> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(format!(
            "index must be an integer, got {}",
            crate::js_number(value)
        ));
    }
    // `-0.0 >= 0.0`, so -0 is index 0, as in JavaScript.
    Ok((value >= 0.0 && value <= f64::from(u32::MAX)).then_some(value as u32))
}

/// The message of the `TypeError` both bindings throw for an `index`
/// argument of `remove()` that is not a number, given its `typeof`.
pub fn invalid_index_type(type_of: &str) -> String {
    format!("index must be a number, got {type_of}")
}

/// Validate a `minScore` option coming from JavaScript: NaN is rejected with
/// the message both bindings report, and any other value (including
/// `±Infinity` and values above 1) is accepted.
///
/// No score compares as at least NaN, so a NaN threshold used to filter out
/// every match silently, while a NaN `maxResults` or `minSimilarity` throws.
pub fn check_min_score(min_score: Option<f64>) -> Result<Option<f64>, String> {
    match min_score {
        Some(score) if score.is_nan() => Err("minScore must be a number, got NaN".to_string()),
        other => Ok(other),
    }
}

thread_local! {
    /// The nucleo `Matcher` of this thread, shared by every search running on
    /// it: standalone `search`/`closest`/`searchKeys` and every `FuzzyIndex`
    /// and `KeyedFuzzyIndex`.
    ///
    /// A matcher owns a ~130 KB scratch slab. Sharing one per thread instead
    /// of allocating one per index keeps small indexes small, and freeing an
    /// index frees all of its memory. Scores never depend on a matcher's
    /// previous use: nucleo sets its configuration before every match.
    static MATCHER: RefCell<Matcher> = RefCell::new(Matcher::new(Config::DEFAULT));
}

/// Run `f` with this thread's shared nucleo `Matcher`.
///
/// A re-entrant call, made while the shared matcher is in use, gets a
/// temporary matcher instead of panicking.
pub fn with_matcher<R>(f: impl FnOnce(&mut Matcher) -> R) -> R {
    MATCHER.with(|cell| match cell.try_borrow_mut() {
        Ok(mut matcher) => f(&mut matcher),
        Err(_) => f(&mut Matcher::new(Config::DEFAULT)),
    })
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

/// Raw score of the best possible match for one term (`atom`): its needle
/// matched against itself, or the theoretical bound for terms longer than
/// [`MAX_TERM_CHARS`]. 0 for negative terms, which never contribute to a
/// score.
pub(crate) fn atom_max_score(atom: &Atom, matcher: &mut Matcher) -> u32 {
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
        .fold(0, |mask, atom| mask | atom_char_mask(atom))
}

/// Character mask every text matched by one term (`atom`) contains: the
/// characters of its needle, whether the term is positive or negative (a
/// `!term` excludes only the texts that its needle matches).
pub(crate) fn atom_char_mask(atom: &Atom) -> u64 {
    atom.needle_text()
        .chars()
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
/// Produces the same representation as `Utf32String::from` (and as the
/// haystacks `FuzzyIndex` stores): ASCII text as bytes, anything else as the
/// first codepoint of each grapheme. `Utf32Str::new` differs for non-ASCII text
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

/// Haystack of an item stored in an index: `None` for ASCII items, which
/// nucleo matches as their own bytes, otherwise the first codepoint of each
/// grapheme (exactly what [`utf32_haystack`] and `Utf32String::from`
/// produce).
///
/// Indexes keep their items as `String`s anyway, so storing only the
/// non-ASCII haystacks avoids keeping a second copy of every ASCII item.
#[inline(always)]
pub(crate) fn index_haystack(item: &str) -> Option<Box<[char]>> {
    (!item.is_ascii()).then(|| chars::graphemes(item).collect())
}

/// The items a search runs over.
#[derive(Clone, Copy)]
pub(crate) enum Corpus<'a> {
    /// Plain strings (standalone `search`), converted to haystacks on the fly.
    Strings(&'a [String]),
    /// An index (`FuzzyIndex`): items with their pre-converted haystacks (see
    /// [`index_haystack`]) and character masks; items whose mask lacks a bit
    /// of the query's mask are never scored.
    Indexed {
        items: &'a [String],
        haystacks: &'a [Option<Box<[char]>>],
        char_masks: &'a [u64],
    },
}

impl<'a> Corpus<'a> {
    fn items(self) -> &'a [String] {
        match self {
            Corpus::Strings(items) | Corpus::Indexed { items, .. } => items,
        }
    }

    #[inline]
    fn haystack<'b>(self, index: usize, buf: &'b mut Vec<char>) -> Utf32Str<'b>
    where
        'a: 'b,
    {
        match self {
            Corpus::Strings(items) => utf32_haystack(&items[index], buf),
            Corpus::Indexed {
                items, haystacks, ..
            } => match &haystacks[index] {
                Some(chars) => Utf32Str::Unicode(chars),
                None => {
                    debug_assert!(items[index].is_ascii());
                    Utf32Str::Ascii(items[index].as_bytes())
                }
            },
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

/// Number of items a search pass times to estimate its cost (native builds
/// only).
#[cfg(not(target_family = "wasm"))]
const PARALLEL_SAMPLE: usize = 64;

/// Smallest input a search pass considers splitting across threads (native
/// builds only).
#[cfg(not(target_family = "wasm"))]
const PARALLEL_MIN_LEN: usize = 1024;

/// Estimated work, in nanoseconds, above which a search pass is split across
/// threads (native builds only; WebAssembly builds always search on the
/// calling thread).
///
/// The cost of scoring an item varies by two orders of magnitude with the
/// item's length and content and with the query, so neither the number of
/// items nor their total length predicts whether parallelism pays off. Each
/// pass over at least [`PARALLEL_MIN_LEN`] items therefore first times
/// [`PARALLEL_SAMPLE`] items spread evenly over its input, and only hands the
/// input to rayon's thread pool when the whole pass would take longer than
/// this on the calling thread.
///
/// Parallel and sequential passes return identical results: chunk outputs are
/// concatenated in input order, and matches are ranked with the same total
/// order either way. The `RAYON_NUM_THREADS` environment variable caps the
/// number of threads (`RAYON_NUM_THREADS=1` disables parallel searching).
#[cfg(not(target_family = "wasm"))]
const PARALLEL_MIN_NANOS: u128 = 250_000;

/// Run `work` over `input`, returning the concatenation of its outputs.
///
/// `work(chunk, out)` must append the outputs for `chunk` to `out` in order.
/// On native targets, inputs that are expensive enough are split across
/// threads (see [`PARALLEL_MIN_NANOS`]); the result is the same either way.
fn split_work<T: Copy + Sync, R: Send>(
    input: &[T],
    work: impl Fn(&[T], &mut Vec<R>) + Sync,
) -> Vec<R> {
    let mut out = Vec::new();
    #[cfg(not(target_family = "wasm"))]
    if input.len() >= PARALLEL_MIN_LEN && parallel::available() {
        // Time an evenly spread sample: inputs are often far from uniform
        // (sorted lists, short names first, ...). Its outputs are discarded.
        let stride = input.len() / PARALLEL_SAMPLE;
        let sample: Vec<T> = input.iter().step_by(stride).copied().collect();
        let start = std::time::Instant::now();
        work(&sample, &mut out);
        let estimate = start.elapsed().as_nanos() * input.len() as u128 / sample.len() as u128;
        out.clear();
        if estimate >= PARALLEL_MIN_NANOS {
            parallel::extend(input, &work, &mut out);
            return out;
        }
    }
    work(input, &mut out);
    out
}

/// Score the items at `indices`, appending those scoring at or above
/// `threshold` to `out` in the order of `indices`.
fn score_items(
    plan: &QueryPlan,
    corpus: Corpus<'_>,
    threshold: f64,
    indices: &[u32],
    out: &mut Vec<(u32, f64)>,
) {
    with_matcher(|matcher| {
        let mut buf = Vec::new();
        for &index in indices {
            let haystack = corpus.haystack(index as usize, &mut buf);
            if let Some(raw_score) = plan.pattern.score(haystack, matcher) {
                let score = plan.normalize(raw_score);
                if score >= threshold {
                    out.push((index, score));
                }
            }
        }
    });
}

/// Ascending indices of the items whose character mask contains every bit of
/// `query_mask`.
fn mask_survivors(char_masks: &[u64], query_mask: u64) -> Vec<u32> {
    char_masks
        .iter()
        .enumerate()
        .filter(|&(_, &mask)| mask & query_mask == query_mask)
        .map(|(index, _)| index as u32)
        .collect()
}

/// Pass 1 of a search: `(index, score)` of every item scoring at or above
/// `threshold`, in ascending index order.
///
/// When `candidates` is set, only those items (ascending indices) are scored.
fn score_corpus(
    plan: &QueryPlan,
    corpus: Corpus<'_>,
    candidates: Option<&[u32]>,
    threshold: f64,
) -> Vec<(u32, f64)> {
    let query_mask = plan.char_mask;
    // The items to score: those whose character mask contains the query's.
    let to_score: Cow<'_, [u32]> = match (corpus, candidates) {
        (Corpus::Indexed { char_masks, .. }, None) => {
            Cow::Owned(mask_survivors(char_masks, query_mask))
        }
        (Corpus::Indexed { char_masks, .. }, Some(candidates)) => Cow::Owned(
            candidates
                .iter()
                .copied()
                .filter(|&index| char_masks[index as usize] & query_mask == query_mask)
                .collect(),
        ),
        (Corpus::Strings(_), Some(candidates)) => Cow::Borrowed(candidates),
        (Corpus::Strings(items), None) => Cow::Owned((0..items.len() as u32).collect()),
    };
    split_work(&to_score, |indices, out| {
        score_items(plan, corpus, threshold, indices, out);
    })
}

/// Pass 2 of a search: turn ranked `(index, score)` pairs into matches,
/// computing positions and match types when requested.
fn rank_matches(
    plan: &QueryPlan,
    corpus: Corpus<'_>,
    scored: &[(u32, f64)],
    include_positions: bool,
) -> Vec<RankedMatch> {
    if !include_positions {
        return scored
            .iter()
            .map(|&(index, score)| RankedMatch {
                index,
                score,
                positions: Vec::new(),
                match_type: None,
            })
            .collect();
    }
    split_work(scored, |scored, out| {
        matches_with_positions(plan, corpus, scored, out);
    })
}

/// Append the matches of `scored` with their positions and match types to
/// `out`, in the order of `scored`.
fn matches_with_positions(
    plan: &QueryPlan,
    corpus: Corpus<'_>,
    scored: &[(u32, f64)],
    out: &mut Vec<RankedMatch>,
) {
    with_matcher(|matcher| {
        let mut buf = Vec::new();
        out.extend(scored.iter().map(|&(index, score)| {
            let haystack = corpus.haystack(index as usize, &mut buf);
            let mut positions = Vec::new();
            plan.pattern.indices(haystack, matcher, &mut positions);
            positions.sort_unstable();
            positions.dedup();
            let match_type = classify_match(&positions, haystack.len());
            RankedMatch {
                index,
                score,
                positions,
                match_type: Some(match_type),
            }
        }));
    });
}

/// Rayon-based execution of the parallel search passes (native builds only).
#[cfg(not(target_family = "wasm"))]
mod parallel {
    use std::sync::OnceLock;

    use rayon::prelude::*;
    use rayon::{ThreadPool, ThreadPoolBuilder};

    /// The thread pool parallel searches run on, created on first use.
    ///
    /// It is separate from rayon's global pool so that the host application's
    /// own use of rayon is unaffected, and a failure to start threads only
    /// disables parallel searching. Its size follows rayon's defaults: the
    /// `RAYON_NUM_THREADS` environment variable, else the number of CPUs.
    fn pool() -> Option<&'static ThreadPool> {
        static POOL: OnceLock<Option<ThreadPool>> = OnceLock::new();
        POOL.get_or_init(|| {
            ThreadPoolBuilder::new()
                .thread_name(|i| format!("rapid-fuzzy-search-{i}"))
                .build()
                .ok()
                .filter(|pool| pool.current_num_threads() > 1)
        })
        .as_ref()
    }

    /// Whether more than one thread is available for parallel work.
    pub(super) fn available() -> bool {
        pool().is_some()
    }

    /// Run `work` over chunks of `input` in parallel, appending the outputs
    /// to `out` in input order.
    ///
    /// The input is split into several chunks per thread so that work
    /// stealing evens out items of very different cost.
    pub(super) fn extend<T: Sync, R: Send>(
        input: &[T],
        work: &(impl Fn(&[T], &mut Vec<R>) + Sync),
        out: &mut Vec<R>,
    ) {
        let Some(pool) = pool() else {
            work(input, out);
            return;
        };
        let chunk_len = input
            .len()
            .div_ceil(pool.current_num_threads() * 8)
            .max(super::PARALLEL_MIN_LEN / 4);
        let chunks: Vec<Vec<R>> = pool.install(|| {
            input
                .par_chunks(chunk_len)
                .map(|chunk| {
                    let mut chunk_out = Vec::new();
                    work(chunk, &mut chunk_out);
                    chunk_out
                })
                .collect()
        });
        out.reserve(chunks.iter().map(Vec::len).sum());
        for chunk_out in chunks {
            out.extend(chunk_out);
        }
    }

    /// Sort `keys` in parallel.
    pub(super) fn sort(keys: &mut [u128]) {
        match pool() {
            Some(pool) => pool.install(|| keys.par_sort_unstable()),
            None => keys.sort_unstable(),
        }
    }
}

/// Sort key of a match: score descending, then item length ascending, then
/// index ascending.
///
/// Scores are finite and non-negative, so the order of their bit patterns is
/// their numeric order; inverting the bits makes higher scores sort first.
/// Comparing keys is much cheaper than comparing scores and looking up item
/// lengths, which dominated the ranking of large result sets. (Lengths
/// saturate at 4 GiB, beyond the longest string JavaScript can create.)
#[inline]
fn rank_key(index: u32, score: f64, item_len: usize) -> u128 {
    debug_assert!(score.is_finite() && score.is_sign_positive());
    let len = u32::try_from(item_len).unwrap_or(u32::MAX);
    (u128::from(!score.to_bits()) << 64) | (u128::from(len) << 32) | u128::from(index)
}

/// The `(index, score)` a [`rank_key`] was built from.
#[inline]
fn decode_rank_key(key: u128) -> (u32, f64) {
    (key as u32, f64::from_bits(!((key >> 64) as u64)))
}

/// Sort rank keys ascending (best match first).
fn sort_keys(keys: &mut [u128]) {
    #[cfg(not(target_family = "wasm"))]
    if keys.len() >= PARALLEL_MIN_SORT {
        parallel::sort(keys);
        return;
    }
    keys.sort_unstable();
}

/// Number of matches above which they are ranked in parallel (native builds
/// only). Keys are distinct, so the result does not depend on the algorithm.
#[cfg(not(target_family = "wasm"))]
const PARALLEL_MIN_SORT: usize = 32_768;

/// The search algorithm shared by standalone search and `FuzzyIndex`.
///
/// When `candidates` is set, only those items (ascending indices) are scored.
pub(crate) fn search_core(
    plan: &QueryPlan,
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

    // Pass 1: score items, keeping only (index, score) — no String cloning.
    let scored = score_corpus(plan, corpus, candidates, min_score.unwrap_or(0.0));

    // Matching indices for the incremental cache, only when the match set is
    // meaningfully smaller than the dataset: when most items match (e.g.
    // short queries) narrowing the next search would not pay off.
    let all_matching = (collect_matching && scored.len() < items.len() / 2)
        .then(|| scored.iter().map(|&(index, _)| index).collect());

    // Rank by score descending, with shorter items first as tiebreaker,
    // then by original index for fully deterministic ordering. Every match
    // gets a distinct sort key, so the order is total and independent of the
    // sorting algorithm.
    let mut keys: Vec<u128> = scored
        .iter()
        .map(|&(index, score)| rank_key(index, score, items[index as usize].len()))
        .collect();
    drop(scored);

    // Top-k selection: quickselect O(n) + sort O(k log k) instead of a full
    // O(n log n) sort when maxResults is set.
    if let Some(max) = max_results {
        let k = max as usize;
        if keys.len() > k {
            keys.select_nth_unstable(k);
            keys.truncate(k);
        }
    }
    sort_keys(&mut keys);
    let scored: Vec<(u32, f64)> = keys.into_iter().map(decode_rank_key).collect();

    // Pass 2: positions only for the final top-k items.
    let matches = rank_matches(plan, corpus, &scored, include_positions);

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
    let Some(plan) = with_matcher(|matcher| QueryPlan::new(query, case_matching, matcher)) else {
        return Vec::new();
    };
    search_core(
        &plan,
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
