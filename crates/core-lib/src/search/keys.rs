//! Multi-key fuzzy search, shared by the standalone [`search_keys_impl`] and
//! [`KeyedFuzzyIndexCore`](super::KeyedFuzzyIndexCore).
//!
//! Both run [`keyed_search_core`] and therefore return identical results; the
//! index only adds pre-computed haystacks and character masks.
//!
//! # Semantics
//!
//! The query is parsed like `search()` parses it: Unicode whitespace separates
//! its terms, and `^`, `$`, `'`, `!` and `\` are query syntax. Only keys with a
//! positive weight `w` take part in a search. Keys whose weight is zero are
//! informational: the returned items get key scores for them, but they never
//! change which items are returned, their scores or their order. Only the
//! ratios between the weights matter: multiplying every weight exactly by the
//! same power of two changes no result, even when the weights are subnormal
//! (unless they span some 300 orders of magnitude, where the smallest ones
//! lose precision anyway); any other positive factor changes results by
//! rounding only.
//!
//! The [`KeyMatchMode`] decides how the query is matched against the keys,
//! which gives the key scores `s` (`key_scores`, one per key, the same in
//! every [`KeyScoreMode`]); the [`KeyScoreMode`] decides how the key scores
//! of the keys with a positive weight are combined into the item's score.
//!
//! ## Per-key matching ([`KeyMatchMode::PerKey`], the default)
//!
//! * Every key is scored against the whole query, like `search()` scores an
//!   item (same query parsing, haystack conversion and score normalization):
//!   a key scores 0 unless it matches every term and no `!term`.
//! * The combined score by [`KeyScoreMode`]:
//!   - [`Weighted`](KeyScoreMode::Weighted) (the default):
//!     `sum(w * s) / sum(w)`, over all keys;
//!   - [`Matched`](KeyScoreMode::Matched): `sum(w * s) / sum(w)` over the
//!     keys that match (`s > 0`) only;
//!   - [`Max`](KeyScoreMode::Max): `max(s)`; weights only decide which keys
//!     take part.
//!
//! ## Cross-key matching ([`KeyMatchMode::CrossKey`])
//!
//! The terms of the query are matched against each key separately, so that
//! they may match different keys: `john tokyo` matches an item whose name is
//! `John Smith` and whose city is `Tokyo`.
//!
//! * Every positive term must match at least one key (with a positive
//!   weight), and no `!term` may match any of these keys; other items are
//!   excluded. A query without a positive term matches nothing.
//! * Let `m(t)` be the raw score of a perfect match of the positive term `t`
//!   (its needle matched against itself), `M` the sum of `m(t)` over the
//!   positive terms (what `search()` divides raw scores by),
//!   and `r(t, k)` the raw score of `t` on key `k`, at most `m(t)`, or 0
//!   when `t` does not match `k`. The key score of `k` is the share of the
//!   query it matches, `s(k) = sum_t r(t, k) / M`, so a key matching every
//!   term perfectly scores 1 as in per-key mode. `!term`s exclude items but
//!   do not change key scores.
//! * The combined score by [`KeyScoreMode`]:
//!   - [`Weighted`](KeyScoreMode::Weighted): `sum(w * s) / sum(w)` over
//!     all keys, as in per-key mode;
//!   - [`Matched`](KeyScoreMode::Matched): `sum(w * s) / sum(w * c)`, where
//!     the coverage `c(k)` of key `k` is the share of the query its matching
//!     terms make up: `sum(m(t)) / M` over the terms `t` that match `k`.
//!     (Per-key mode is the special case where `c` is 1 for the keys that
//!     match and 0 for the others.)
//!   - [`Max`](KeyScoreMode::Max): every term counts with its best key,
//!     `sum_t max_k r(t, k) / M`; weights only decide which keys take part.
//!
//!   With `Max`, an item scores 1 when every term matches some key
//!   perfectly. With `Matched`, an item scores 1 only when each term
//!   matches perfectly every key it matches: a term that also matches
//!   another key partially adds its whole coverage there but only part of
//!   its score, so `john tokyo` scores 0.89 on an item whose name
//!   `John Smith` and city `Tokyo` match perfectly but whose email
//!   `jtokyo@example.com` matches `tokyo` partially (all weights 1).
//! * For a query of one term without `!term`s, both match modes return the
//!   same results (scores and key scores included).
//!
//! ## In both modes
//!
//! * Items whose combined score is 0 are excluded, as are items scoring
//!   below `min_score`; `max_results` keeps the first results of the order
//!   below. Both apply to the combined score of the mode.
//! * Results are sorted by combined score (descending), then by the UTF-8
//!   byte length of the best-matching key's text (shorter first, like
//!   `search()` prefers shorter items), then by index. The best-matching key
//!   is the one contributing the most to the combined score (`w * s`, or `s`
//!   in `Max` mode); on a tie, the first such key.
//! * With `return_all_on_empty`, a query without a search term returns every
//!   item with a score of 1 and key scores of 1, in every mode.

use std::cmp::Ordering;

use nucleo_matcher::pattern::{CaseMatching, Pattern};
use nucleo_matcher::{Matcher, Utf32Str, chars};

use super::cross_key::CrossKeyPlan;
use super::{
    KeySearchResult, QueryPlan, is_empty_query, resolve_case_matching, utf32_haystack, with_matcher,
};

/// How a multi-key search combines the per-key scores of an item into its
/// score (see the module documentation).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyScoreMode {
    /// The weighted mean over all keys: `sum(w * s) / sum(w)`. A key that
    /// does not match counts as 0.
    #[default]
    Weighted,
    /// The weighted mean over the keys that match (`s > 0`): an item whose
    /// only matching key matches exactly scores 1.
    Matched,
    /// The best score of any key with a positive weight.
    Max,
}

impl KeyScoreMode {
    /// The mode's name in the JavaScript API (`scoreMode`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Weighted => "weighted",
            Self::Matched => "matched",
            Self::Max => "max",
        }
    }

    /// Parse a `scoreMode` value: `"weighted"`, `"matched"` or `"max"`
    /// (case-sensitive). Anything else is rejected with the message both
    /// bindings report.
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name {
            "weighted" => Ok(Self::Weighted),
            "matched" => Ok(Self::Matched),
            "max" => Ok(Self::Max),
            _ => Err(invalid_score_mode(&format!("{name:?}"))),
        }
    }
}

/// The error message for an invalid `scoreMode`; `got` describes the value
/// (a quoted string, or the type of a value that is not a string).
pub fn invalid_score_mode(got: &str) -> String {
    format!("scoreMode must be \"weighted\", \"matched\" or \"max\", got {got}")
}

/// How a multi-key search matches the query against the keys of an item
/// (see the module documentation).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyMatchMode {
    /// Every key is matched against the whole query: an item matches when
    /// one of its keys contains every term.
    #[default]
    PerKey,
    /// Every term is matched against the keys on its own: an item matches
    /// when each term matches one of its keys, and no `!term` matches any.
    CrossKey,
}

impl KeyMatchMode {
    /// The mode's name in the JavaScript API (`matchMode`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::PerKey => "perKey",
            Self::CrossKey => "crossKey",
        }
    }

    /// Parse a `matchMode` value: `"perKey"` or `"crossKey"`
    /// (case-sensitive). Anything else is rejected with the message both
    /// bindings report.
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name {
            "perKey" => Ok(Self::PerKey),
            "crossKey" => Ok(Self::CrossKey),
            _ => Err(invalid_match_mode(&format!("{name:?}"))),
        }
    }
}

/// The error message for an invalid `matchMode`; `got` describes the value
/// (a quoted string, or the type of a value that is not a string).
pub fn invalid_match_mode(got: &str) -> String {
    format!("matchMode must be \"perKey\" or \"crossKey\", got {got}")
}

/// Search options for the `search_keys_impl` function and
/// [`KeyedFuzzyIndexCore::search_with_options`](super::KeyedFuzzyIndexCore::search_with_options).
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchKeysOptions {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub is_case_sensitive: Option<bool>,
    pub return_all_on_empty: Option<bool>,
    /// How per-key scores are combined; `None` means
    /// [`KeyScoreMode::Weighted`].
    pub score_mode: Option<KeyScoreMode>,
    /// How the query is matched against the keys; `None` means
    /// [`KeyMatchMode::PerKey`].
    pub match_mode: Option<KeyMatchMode>,
}

/// Validate multi-key search input and return the total weight.
///
/// `key_texts[k]` holds the text of key `k` for every item, so all columns
/// must have the same length; there must be one weight per key, every weight
/// must be a finite non-negative number, and their sum must be positive and
/// finite. `KeyedFuzzyIndexCore::new` and `search_keys_impl` report the same
/// errors.
pub fn validate_keyed_input(key_texts: &[Vec<String>], weights: &[f64]) -> Result<f64, String> {
    if let Some(num_items) = key_texts.first().map(Vec::len) {
        for (k, col) in key_texts.iter().enumerate().skip(1) {
            if col.len() != num_items {
                return Err(format!(
                    "All key_texts columns must have the same length; key 0 has {}, key {} has {}",
                    num_items,
                    k,
                    col.len()
                ));
            }
        }
    }

    if weights.len() != key_texts.len() {
        return Err(format!(
            "Expected {} weights, got {}",
            key_texts.len(),
            weights.len()
        ));
    }

    if weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
        return Err("Weights must be finite non-negative numbers".to_string());
    }

    let total_weight: f64 = weights.iter().sum();
    if total_weight <= 0.0 {
        return Err("Total weight must be greater than zero".to_string());
    }
    if !total_weight.is_finite() {
        return Err("Total weight must be finite; the weights sum to Infinity".to_string());
    }
    Ok(total_weight)
}

/// The items a multi-key search runs over, indexed `[key][item]`.
///
/// Implemented by the standalone key texts and by `KeyedFuzzyIndexCore`'s
/// pre-computed data; [`keyed_search_core`] is generic over it so that each
/// gets its own specialized loop.
pub(crate) trait KeyedCorpus {
    /// Whether [`may_match`](Self::may_match) can reject items cheaply.
    ///
    /// Such corpora are scored item by item, skipping items rejected by
    /// every weighted key and stopping early when `min_score` can no longer
    /// be reached. Others are scored key by key (one key over all items,
    /// then the next), which keeps nucleo's branches predictable.
    const HAS_CHAR_MASKS: bool;

    /// The key texts, `[key][item]`.
    fn key_texts(&self) -> &[Vec<String>];

    /// Byte length of key `k` of item `i`.
    #[inline]
    fn text_len(&self, k: usize, i: usize) -> usize {
        self.key_texts()[k][i].len()
    }

    /// Whether key `k` of item `i` can match a query whose character mask is
    /// `query_mask`. Must never return false for an item nucleo would match.
    fn may_match(&self, k: usize, i: usize, query_mask: u64) -> bool;

    /// Raw nucleo score of key `k` of item `i`.
    fn raw_score(
        &self,
        plan: &QueryPlan,
        matcher: &mut Matcher,
        buf: &mut Vec<char>,
        k: usize,
        i: usize,
    ) -> Option<u32>;

    /// Normalized score of key `k` of item `i`, 0.0 when it does not match.
    #[inline(always)]
    fn key_score(
        &self,
        plan: &QueryPlan,
        matcher: &mut Matcher,
        buf: &mut Vec<char>,
        k: usize,
        i: usize,
    ) -> f64 {
        if !self.may_match(k, i, plan.char_mask) {
            return 0.0;
        }
        self.raw_score(plan, matcher, buf, k, i)
            .map_or(0.0, |raw| plan.normalize(raw))
    }

    /// Prepare `buf` for [`haystack`](Self::haystack) of key `k` of item
    /// `i`: corpora converting texts on the fly write the conversion there.
    fn prepare_haystack(&self, k: usize, i: usize, buf: &mut Vec<char>);

    /// The haystack nucleo matches key `k` of item `i` as (see
    /// [`utf32_haystack`]), `buf` having been prepared for it by
    /// [`prepare_haystack`](Self::prepare_haystack).
    fn haystack<'b>(&'b self, k: usize, i: usize, buf: &'b [char]) -> Utf32Str<'b>;
}

/// What `Pattern::score` computes — the sum of the atoms' scores, or `None`
/// when one of them does not match — written out so that it is inlined into
/// the scoring loops (an out-of-line call per key cost several percent).
/// The sum saturates instead of overflowing.
#[inline(always)]
fn pattern_score(pattern: &Pattern, haystack: Utf32Str<'_>, matcher: &mut Matcher) -> Option<u32> {
    let mut score: u32 = 0;
    for atom in &pattern.atoms {
        score = score.saturating_add(u32::from(atom.score(haystack, matcher)?));
    }
    Some(score)
}

/// Plain key texts (standalone `search_keys_impl`), converted to haystacks
/// on the fly.
///
/// No character mask is computed for them: building one takes a pass over
/// the text, which costs more than nucleo needs to reject a haystack itself.
pub(crate) struct KeyTexts<'a>(pub &'a [Vec<String>]);

impl KeyedCorpus for KeyTexts<'_> {
    const HAS_CHAR_MASKS: bool = false;

    #[inline]
    fn key_texts(&self) -> &[Vec<String>] {
        self.0
    }

    #[inline]
    fn may_match(&self, _k: usize, _i: usize, _query_mask: u64) -> bool {
        true
    }

    #[inline]
    fn raw_score(
        &self,
        plan: &QueryPlan,
        matcher: &mut Matcher,
        buf: &mut Vec<char>,
        k: usize,
        i: usize,
    ) -> Option<u32> {
        pattern_score(&plan.pattern, utf32_haystack(&self.0[k][i], buf), matcher)
    }

    #[inline]
    fn prepare_haystack(&self, k: usize, i: usize, buf: &mut Vec<char>) {
        // `buf` stays empty for ASCII text, which is matched as its bytes
        // (non-ASCII text has at least one grapheme).
        let text = &self.0[k][i];
        buf.clear();
        if !text.is_ascii() {
            buf.extend(chars::graphemes(text));
        }
    }

    #[inline]
    fn haystack<'b>(&'b self, k: usize, i: usize, buf: &'b [char]) -> Utf32Str<'b> {
        let text = &self.0[k][i];
        debug_assert_eq!(buf.is_empty(), text.is_ascii(), "prepared for {text:?}");
        if buf.is_empty() {
            Utf32Str::Ascii(text.as_bytes())
        } else {
            Utf32Str::Unicode(buf)
        }
    }
}

/// Pre-converted haystacks and character masks (`KeyedFuzzyIndexCore`).
pub(crate) struct IndexedKeys<'a> {
    pub key_texts: &'a [Vec<String>],
    /// [`index_haystack`](super::index_haystack) of every key text: `None`
    /// for ASCII text, which is matched as its own bytes.
    pub haystacks: &'a [Vec<Option<Box<[char]>>>],
    /// [`compute_char_mask`](super::compute_char_mask) of every key text.
    pub char_masks: &'a [Vec<u64>],
}

impl IndexedKeys<'_> {
    /// The haystack of key `k` of item `i`: exactly what `Utf32String::from`
    /// (and [`utf32_haystack`]) make of its text, like `FuzzyIndexCore`'s
    /// stored haystacks.
    #[inline]
    fn indexed_haystack(&self, k: usize, i: usize) -> Utf32Str<'_> {
        match &self.haystacks[k][i] {
            Some(chars) => Utf32Str::Unicode(chars),
            None => {
                let text = &self.key_texts[k][i];
                debug_assert!(text.is_ascii());
                Utf32Str::Ascii(text.as_bytes())
            }
        }
    }
}

impl KeyedCorpus for IndexedKeys<'_> {
    const HAS_CHAR_MASKS: bool = true;

    #[inline]
    fn key_texts(&self) -> &[Vec<String>] {
        self.key_texts
    }

    #[inline]
    fn may_match(&self, k: usize, i: usize, query_mask: u64) -> bool {
        // The mask is a superset of the characters nucleo compares, so a
        // text lacking a bit of the query's mask cannot match.
        self.char_masks[k][i] & query_mask == query_mask
    }

    #[inline]
    fn raw_score(
        &self,
        plan: &QueryPlan,
        matcher: &mut Matcher,
        _buf: &mut Vec<char>,
        k: usize,
        i: usize,
    ) -> Option<u32> {
        pattern_score(&plan.pattern, self.indexed_haystack(k, i), matcher)
    }

    #[inline]
    fn prepare_haystack(&self, _k: usize, _i: usize, _buf: &mut Vec<char>) {}

    #[inline]
    fn haystack<'b>(&'b self, k: usize, i: usize, _buf: &'b [char]) -> Utf32Str<'b> {
        self.indexed_haystack(k, i)
    }
}

/// Result-shaping options of a multi-key search.
#[derive(Clone, Copy, Debug)]
pub(crate) struct KeyedSearchParams {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub case_matching: CaseMatching,
    pub return_all_on_empty: bool,
    pub score_mode: KeyScoreMode,
    pub match_mode: KeyMatchMode,
}

impl From<SearchKeysOptions> for KeyedSearchParams {
    /// Resolve the defaults of unset options.
    fn from(options: SearchKeysOptions) -> Self {
        Self {
            max_results: options.max_results,
            min_score: options.min_score,
            case_matching: resolve_case_matching(options.is_case_sensitive),
            return_all_on_empty: options.return_all_on_empty.unwrap_or(false),
            score_mode: options.score_mode.unwrap_or_default(),
            match_mode: options.match_mode.unwrap_or_default(),
        }
    }
}

/// Relative slack of the early-exit bound, in units of the weight the bound
/// divides by (the total weight, or the weight of the matched and remaining
/// keys in `Matched` mode).
///
/// The bound and the final combined score are computed with different
/// floating-point summation orders, which may differ by a few ULPs of that
/// weight; this slack is orders of magnitude larger, so the early exit never
/// rejects an item whose combined score reaches `min_score`. When that weight
/// is so small (below ~2.5e-315) that the slack underflows to 0, which the
/// scaling of [`scale_small_weights`] leaves to weights more than ~1e300
/// times smaller than the largest one, every weight involved is subnormal: a
/// product `w * s` rounds to at most `w` and sums are exact, so the bound
/// still never rejects a qualifying item (`tests/keyed_search.rs` checks
/// such weights).
const EARLY_EXIT_SLACK: f64 = 1e-9;

/// A matching item during pass 1.
///
/// Kept to 16 bytes for the top-k selection; per-candidate data needed less
/// often lives in side tables indexed by `slot` (the candidate's position in
/// discovery order).
#[derive(Clone, Copy)]
struct Candidate {
    score: f64,
    index: u32,
    slot: u32,
}

/// How the item-by-item scan rejects an item before all of its keys are
/// scored, once `min_score` is set: after each active key, it checks whether
/// even perfect scores on the remaining keys could reach `min_score`.
trait EarlyExit {
    /// Whether to first skip items none of whose active keys can match (by
    /// character mask), in one tight check.
    const MASK_FIRST: bool;

    /// Start the next item.
    fn start(&mut self);

    /// Account for an active key of weight `weight` scoring `score`. True
    /// when the item cannot reach `min_score`, whatever the active keys after
    /// it (of total weight `remaining`) score.
    fn unreachable(&mut self, score: f64, weight: f64, remaining: f64) -> bool;
}

/// Without `min_score`, and in `Max` mode (where any remaining key may still
/// score 1): every item is scored in full.
struct NoEarlyExit;

impl EarlyExit for NoEarlyExit {
    const MASK_FIRST: bool = true;

    #[inline(always)]
    fn start(&mut self) {}

    #[inline(always)]
    fn unreachable(&mut self, _score: f64, _weight: f64, _remaining: f64) -> bool {
        false
    }
}

/// `Weighted` mode: the final weighted sum is at most the sum so far plus
/// the remaining weight, and must reach `min_score * total_weight`.
struct WeightedExit {
    /// `min_score * total_weight`, minus the slack.
    required: f64,
    weighted_sum: f64,
}

impl EarlyExit for WeightedExit {
    // Items none of whose keys can match are usually rejected after their
    // first key, sooner than by checking all of their masks first.
    const MASK_FIRST: bool = false;

    #[inline(always)]
    fn start(&mut self) {
        self.weighted_sum = 0.0;
    }

    #[inline(always)]
    fn unreachable(&mut self, score: f64, weight: f64, remaining: f64) -> bool {
        self.weighted_sum += score * weight;
        self.weighted_sum + remaining < self.required
    }
}

/// `Matched` mode. The final score is
/// `(weighted_sum + a) / (matched_weight + b)`, where `b` is the weight of
/// the remaining keys that match and `a <= b` what they add. As
/// `weighted_sum <= matched_weight`, it is at most
/// `(weighted_sum + R) / (matched_weight + R)`, `R` being the weight of all
/// remaining keys: adding a perfect match never lowers a mean of scores of at
/// most 1. While no key has matched, that bound is 1, so items are only
/// rejected once they match partially.
struct MatchedExit {
    threshold: f64,
    weighted_sum: f64,
    matched_weight: f64,
}

impl EarlyExit for MatchedExit {
    // The bound of an item that has matched no key yet stays at 1.
    const MASK_FIRST: bool = true;

    #[inline(always)]
    fn start(&mut self) {
        self.weighted_sum = 0.0;
        self.matched_weight = 0.0;
    }

    #[inline(always)]
    fn unreachable(&mut self, score: f64, weight: f64, remaining: f64) -> bool {
        self.weighted_sum += score * weight;
        if score > 0.0 {
            self.matched_weight += weight;
        }
        let bound_weight = self.matched_weight + remaining;
        self.weighted_sum + remaining
            < self.threshold * bound_weight - bound_weight * EARLY_EXIT_SLACK
    }
}

/// The item-by-item scan of corpora with character masks.
struct MaskedScan<'a, C> {
    corpus: &'a C,
    plan: &'a QueryPlan,
    num_items: usize,
    /// The keys with a positive weight.
    active_keys: &'a [usize],
    weights: &'a [f64],
    /// `remaining_weight[j]`: the total weight of `active_keys[j..]`.
    remaining_weight: &'a [f64],
}

impl<C: KeyedCorpus> MaskedScan<'_, C> {
    /// Score the items one by one, stopping as soon as `exit` rejects one,
    /// and pass every item that matches an active key to `select`, with its
    /// active-key scores. Monomorphized per [`EarlyExit`], so that each mode
    /// gets a loop of its own.
    fn run<E: EarlyExit>(
        &self,
        mut exit: E,
        matcher: &mut Matcher,
        buf: &mut Vec<char>,
        item_scores: &mut [f64],
        select: &mut impl FnMut(usize, &[f64]),
    ) {
        let mut next = 0;
        'items: while next < self.num_items {
            let i = if E::MASK_FIRST {
                match first_may_match(self.corpus, self.active_keys, self.plan.char_mask, next) {
                    Some(i) => i,
                    None => break,
                }
            } else {
                next
            };
            next = i + 1;
            exit.start();
            let mut matched = false;
            for (j, &k) in self.active_keys.iter().enumerate() {
                let score = self.corpus.key_score(self.plan, matcher, buf, k, i);
                item_scores[j] = score;
                matched |= score > 0.0;
                if exit.unreachable(score, self.weights[k], self.remaining_weight[j + 1]) {
                    continue 'items;
                }
            }
            // Without a matching active key the combined score is 0.
            if matched {
                select(i, item_scores);
            }
        }
    }
}

/// The first item from `start` on that one of `active_keys` may match
/// according to the character masks of `corpus` (all of `query_mask`'s
/// bits), or `None`.
///
/// Most items of a typical search fail this check. It is kept out of line so
/// that its tight loop gets registers of its own instead of competing with
/// the scoring loop of its caller (inlined, it kept reloading the mask table
/// and the item index from the stack).
#[inline(never)]
pub(super) fn first_may_match<C: KeyedCorpus>(
    corpus: &C,
    active_keys: &[usize],
    query_mask: u64,
    start: usize,
) -> Option<usize> {
    let num_items = corpus.key_texts().first().map_or(0, Vec::len);
    (start..num_items).find(|&i| {
        active_keys
            .iter()
            .any(|&k| corpus.may_match(k, i, query_mask))
    })
}

/// The items pass 1 selected, in the order it found them, with what pass 2
/// needs to rank them and build their results.
pub(super) struct Selection {
    candidates: Vec<Candidate>,
    /// Byte length of each candidate's best-matching key text, by slot.
    tie_lens: Vec<usize>,
    /// Active-key scores of the candidates, `num_active` per slot.
    active_scores: Vec<f64>,
}

impl Selection {
    pub(super) fn new() -> Self {
        Self {
            candidates: Vec::new(),
            tie_lens: Vec::new(),
            active_scores: Vec::new(),
        }
    }

    /// Select item `index` with its combined `score`, the byte length of its
    /// best-matching key text and its active-key scores.
    #[inline]
    pub(super) fn push(&mut self, index: usize, score: f64, tie_len: usize, active_scores: &[f64]) {
        self.candidates.push(Candidate {
            score,
            index: index as u32,
            slot: self.tie_lens.len() as u32,
        });
        self.tie_lens.push(tie_len);
        self.active_scores.extend_from_slice(active_scores);
    }

    /// Rank the selected items and build the results of the first
    /// `max_results`: score descending, then shorter best-matching key text,
    /// then index. The key scores of the keys whose weight is zero, which
    /// pass 1 does not compute, come from `inactive_key_score(k, i)`.
    pub(super) fn into_results(
        self,
        max_results: Option<u32>,
        weights: &[f64],
        mut inactive_key_score: impl FnMut(usize, usize) -> f64,
    ) -> Vec<KeySearchResult> {
        let Self {
            mut candidates,
            tie_lens,
            active_scores,
        } = self;
        let num_keys = weights.len();
        let num_active = weights.iter().filter(|&&w| w > 0.0).count();

        let cmp = |a: &Candidate, b: &Candidate| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| tie_lens[a.slot as usize].cmp(&tie_lens[b.slot as usize]))
                .then_with(|| a.index.cmp(&b.index))
        };

        // Top-k selection: quickselect O(n) + sort O(k log k) instead of a
        // full O(n log n) sort when maxResults is set.
        if let Some(max) = max_results {
            let k = max as usize;
            if candidates.len() > k {
                candidates.select_nth_unstable_by(k, cmp);
                candidates.truncate(k);
            }
        }
        candidates.sort_unstable_by(cmp);

        // Pass 2: key scores of the returned items only. Active keys were
        // scored in pass 1; zero-weight keys are scored now.
        candidates
            .into_iter()
            .map(|c| {
                let i = c.index as usize;
                let mut key_scores = vec![0.0; num_keys];
                let at = c.slot as usize * num_active;
                let mut active = active_scores[at..at + num_active].iter();
                for (k, key_score) in key_scores.iter_mut().enumerate() {
                    *key_score = if weights[k] > 0.0 {
                        active.next().copied().unwrap_or(0.0)
                    } else {
                        inactive_key_score(k, i)
                    };
                }
                KeySearchResult {
                    index: c.index,
                    score: c.score,
                    key_scores,
                }
            })
            .collect()
    }
}

/// The multi-key search algorithm shared by `search_keys_impl` and
/// `KeyedFuzzyIndexCore` (see the module documentation for its semantics).
///
/// `weights` and `total_weight` must have been checked with
/// [`validate_keyed_input`].
pub(crate) fn keyed_search_core<C: KeyedCorpus>(
    query: &str,
    corpus: &C,
    weights: &[f64],
    total_weight: f64,
    params: KeyedSearchParams,
    matcher: &mut Matcher,
) -> Vec<KeySearchResult> {
    let KeyedSearchParams {
        max_results,
        min_score,
        case_matching,
        return_all_on_empty,
        score_mode,
        match_mode,
    } = params;
    let key_texts = corpus.key_texts();
    let num_keys = key_texts.len();
    let num_items = key_texts.first().map_or(0, Vec::len);
    if num_keys == 0 || num_items == 0 {
        return Vec::new();
    }

    if return_all_on_empty && is_empty_query(query) {
        let limit = max_results.map_or(usize::MAX, |max| max as usize);
        return (0..num_items)
            .take(limit)
            .map(|i| KeySearchResult {
                index: i as u32,
                score: 1.0,
                key_scores: vec![1.0; num_keys],
            })
            .collect();
    }

    let Some(plan) = QueryPlan::new(query, case_matching, matcher) else {
        return Vec::new();
    };

    let scaled = scale_small_weights(weights);
    let (weights, total_weight) = match &scaled {
        Some((scaled, total)) => (scaled.as_slice(), *total),
        None => (weights, total_weight),
    };

    // Only keys with a positive weight can select an item; zero-weight keys
    // are scored for the returned items only.
    let active_keys: Vec<usize> = (0..num_keys).filter(|&k| weights[k] > 0.0).collect();
    let keys = ActiveKeys {
        num_items,
        active_keys: &active_keys,
        weights,
        total_weight,
    };
    let threshold = min_score.unwrap_or(0.0);
    let mut buf = Vec::new();

    match match_mode {
        KeyMatchMode::PerKey => {
            let selection = per_key_pass(corpus, &plan, &keys, score_mode, threshold, matcher);
            selection.into_results(max_results, weights, |k, i| {
                corpus.key_score(&plan, matcher, &mut buf, k, i)
            })
        }
        KeyMatchMode::CrossKey => {
            let cross = CrossKeyPlan::new(&plan.pattern, matcher);
            let selection = cross.pass(corpus, &keys, score_mode, threshold, matcher);
            selection.into_results(max_results, weights, |k, i| {
                cross.key_score(corpus, matcher, &mut buf, k, i)
            })
        }
    }
}

/// The weights a search computes with when the largest of `weights` is below
/// 1: `weights` multiplied by the power of two that brings the largest into
/// `[1, 2)`, with their sum. `None` when the largest weight is at least 1, in
/// which case `weights` are used as they are.
///
/// Only the ratios between the weights matter, but a product `score * weight`
/// loses precision once it is subnormal: with subnormal weights every product
/// rounds to a multiple of 5e-324, so that a partial match could score 1 in
/// `Matched` mode, like an exact one. Multiplying by a power of two is exact
/// here (the scaled weights stay below 2) and changes neither the products
/// that were normal (each is multiplied by the same power of two) nor any
/// ratio or comparison of them, so the results of weights whose products were
/// normal stay bit-identical.
fn scale_small_weights(weights: &[f64]) -> Option<(Vec<f64>, f64)> {
    let largest = weights.iter().copied().fold(0.0, f64::max);
    // At least 1, or no positive weight (rejected by `validate_keyed_input`).
    if !(largest > 0.0 && largest < 1.0) {
        return None;
    }
    // largest = m * 2^-shift with 1 <= m < 2 and 1 <= shift <= 1074. The
    // sign bit is clear, so the exponent field is `bits >> 52`.
    let bits = largest.to_bits();
    let shift = match bits >> 52 {
        // Subnormal: largest = bits * 2^-1074, whose highest set bit is bit
        // `63 - leading_zeros`.
        0 => 1074 - (63 - bits.leading_zeros()),
        biased => 1023 - biased as u32,
    };
    // 2^shift, in two exact factors (2^1074 is not a finite f64). Every
    // weight is at most `largest`, so no product overflows.
    let first = shift.min(1023);
    let factors = [pow2(first), pow2(shift - first)];
    let scaled: Vec<f64> = weights
        .iter()
        .map(|&w| w * factors[0] * factors[1])
        .collect();
    let total = scaled.iter().sum();
    Some((scaled, total))
}

/// `2^exp` for `exp <= 1023`, exactly.
fn pow2(exp: u32) -> f64 {
    debug_assert!(exp <= 1023, "2^{exp}");
    f64::from_bits(u64::from(exp + 1023) << 52)
}

/// The keys of a multi-key search, as pass 1 sees them.
pub(super) struct ActiveKeys<'a> {
    pub num_items: usize,
    /// The keys with a positive weight, in key order.
    pub active_keys: &'a [usize],
    /// The weight of every key.
    pub weights: &'a [f64],
    /// The sum of `weights`, checked by [`validate_keyed_input`].
    pub total_weight: f64,
}

/// Pass 1 of per-key matching: the items that qualify, with their
/// active-key scores.
///
/// Kept out of line, like [`CrossKeyPlan::pass`]: merged into one function,
/// the two scans compete for registers and the per-key loops got slower.
#[inline(never)]
fn per_key_pass<C: KeyedCorpus>(
    corpus: &C,
    plan: &QueryPlan,
    keys: &ActiveKeys<'_>,
    score_mode: KeyScoreMode,
    threshold: f64,
    matcher: &mut Matcher,
) -> Selection {
    let &ActiveKeys {
        num_items,
        active_keys,
        weights,
        total_weight,
    } = keys;
    // remaining_weight[j]: sum of the weights of active_keys[j..], an upper
    // bound of what those keys can still add to the weighted sum.
    let mut remaining_weight = vec![0.0; active_keys.len() + 1];
    for j in (0..active_keys.len()).rev() {
        remaining_weight[j] = remaining_weight[j + 1] + weights[active_keys[j]];
    }

    let num_active = active_keys.len();
    let mut buf = Vec::new();
    let mut selection = Selection::new();
    // Active-key scores of the current item.
    let mut item_scores = vec![0.0; num_active];

    // Combine an item's active-key scores and keep it if it qualifies. Every
    // item that may qualify goes through here, whichever way it was scored,
    // so both strategies below select exactly the same items.
    let mut select = |i: usize, item_scores: &[f64]| {
        let mut weighted_sum = 0.0;
        // Weight of the keys that match (`Matched` mode).
        let mut matched_weight = 0.0;
        // (contribution, key) of the best-matching key so far; in `Max` mode
        // its contribution is the combined score.
        let mut best: Option<(f64, usize)> = None;
        for (&score, &k) in item_scores.iter().zip(active_keys) {
            let weighted = score * weights[k];
            weighted_sum += weighted;
            if score > 0.0 {
                matched_weight += weights[k];
            }
            let contribution = match score_mode {
                KeyScoreMode::Max => score,
                KeyScoreMode::Weighted | KeyScoreMode::Matched => weighted,
            };
            if contribution > best.map_or(0.0, |(c, _)| c) {
                best = Some((contribution, k));
            }
        }
        let combined = match score_mode {
            KeyScoreMode::Weighted => weighted_sum / total_weight,
            // matched_weight > 0 whenever weighted_sum > 0; otherwise the
            // item is not selected (best is None).
            KeyScoreMode::Matched if weighted_sum > 0.0 => weighted_sum / matched_weight,
            KeyScoreMode::Matched => 0.0,
            KeyScoreMode::Max => best.map_or(0.0, |(c, _)| c),
        };
        match best {
            Some((_, best_key)) if combined > 0.0 && combined >= threshold => {
                selection.push(i, combined, corpus.text_len(best_key, i), item_scores);
            }
            _ => {}
        }
    };

    if C::HAS_CHAR_MASKS {
        // Item by item, so that items can be rejected early.
        let scan = MaskedScan {
            corpus,
            plan,
            num_items,
            active_keys,
            weights,
            remaining_weight: &remaining_weight,
        };
        let (buf, item_scores, select) = (&mut buf, &mut item_scores, &mut select);
        // NaN thresholds never prune (and the final check rejects every item).
        let prune = threshold > 0.0;
        match score_mode {
            _ if !prune => scan.run(NoEarlyExit, matcher, buf, item_scores, select),
            KeyScoreMode::Weighted => {
                let exit = WeightedExit {
                    required: threshold * total_weight - total_weight * EARLY_EXIT_SLACK,
                    weighted_sum: 0.0,
                };
                scan.run(exit, matcher, buf, item_scores, select);
            }
            KeyScoreMode::Matched => {
                let exit = MatchedExit {
                    threshold,
                    weighted_sum: 0.0,
                    matched_weight: 0.0,
                };
                scan.run(exit, matcher, buf, item_scores, select);
            }
            // Any remaining key may still score 1: only the final check can
            // reject an item.
            KeyScoreMode::Max => scan.run(NoEarlyExit, matcher, buf, item_scores, select),
        }
    } else {
        // Key by key: one key over all items, then the next.
        // columns[j * num_items + i]: score of active key j for item i.
        let mut columns = Vec::with_capacity(num_active * num_items);
        for &k in active_keys {
            columns.extend((0..num_items).map(|i| corpus.key_score(plan, matcher, &mut buf, k, i)));
        }
        for i in 0..num_items {
            let mut matched = false;
            for (j, score) in item_scores.iter_mut().enumerate() {
                *score = columns[j * num_items + i];
                matched |= *score > 0.0;
            }
            // Without a matching weighted key the combined score is 0.
            if matched {
                select(i, &item_scores);
            }
        }
    }
    selection
}

/// Perform fuzzy search across multiple text keys with weights.
///
/// `key_texts[k]` is an array of strings for key `k`, one per item.
/// All inner arrays must have the same length (the number of items).
/// `weights` specifies the relative importance of each key.
///
/// Returns results sorted by combined score (best match first; see the
/// module documentation, [`KeyScoreMode`] and [`KeyMatchMode`]), exactly
/// like [`KeyedFuzzyIndexCore::search_with_options`](super::KeyedFuzzyIndexCore::search_with_options)
/// on the same input. Returns an error, with the same message as
/// `KeyedFuzzyIndexCore::new`, when the input is invalid (see
/// [`validate_keyed_input`]).
pub fn search_keys_impl(
    query: &str,
    key_texts: &[Vec<String>],
    weights: &[f64],
    options: Option<SearchKeysOptions>,
) -> Result<Vec<KeySearchResult>, String> {
    let total_weight = validate_keyed_input(key_texts, weights)?;
    let params = KeyedSearchParams::from(options.unwrap_or_default());

    Ok(with_matcher(|matcher| {
        keyed_search_core(
            query,
            &KeyTexts(key_texts),
            weights,
            total_weight,
            params,
            matcher,
        )
    }))
}
