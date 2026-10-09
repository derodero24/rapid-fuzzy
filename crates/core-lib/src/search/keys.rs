//! Multi-key fuzzy search, shared by the standalone [`search_keys_impl`] and
//! [`KeyedFuzzyIndexCore`](super::KeyedFuzzyIndexCore).
//!
//! Both run [`keyed_search_core`] and therefore return identical results; the
//! index only adds pre-computed haystacks and character masks.
//!
//! # Semantics
//!
//! * Every key is scored like `search()` scores an item (same query parsing,
//!   haystack conversion and score normalization). The per-key scores are
//!   returned as `key_scores` for every key, including keys whose weight is
//!   zero (they are informational there), whatever the [`KeyScoreMode`].
//! * The combined score is computed from the scores `s` of the keys with a
//!   positive weight `w` by the [`KeyScoreMode`]:
//!   - [`Weighted`](KeyScoreMode::Weighted) (the default):
//!     `sum(w * s) / sum(w)`, over all keys;
//!   - [`Matched`](KeyScoreMode::Matched): `sum(w * s) / sum(w)` over the
//!     keys that match (`s > 0`) only;
//!   - [`Max`](KeyScoreMode::Max): `max(s)`; weights only decide which keys
//!     take part.
//!
//!   Keys whose weight is zero never contribute, in any mode.
//! * Items whose combined score is 0 — no match on any key with a positive
//!   weight — are excluded, as are items scoring below `min_score`;
//!   `max_results` keeps the first results of the order below. Both apply
//!   to the combined score of the mode.
//! * Results are sorted by combined score (descending), then by the UTF-8
//!   byte length of the best-matching key's text (shorter first, like
//!   `search()` prefers shorter items), then by index. The best-matching key
//!   is the one contributing the most to the combined score (`w * s`, or `s`
//!   in `Max` mode); on a tie, the first such key.
//! * With `return_all_on_empty`, a query without a search term returns every
//!   item with a score of 1 and key scores of 1, in every mode.

use std::cmp::Ordering;

use nucleo_matcher::pattern::{CaseMatching, Pattern};
use nucleo_matcher::{Matcher, Utf32Str, Utf32String};

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

/// Search options for the `search_keys_impl` function.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchKeysOptions {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub is_case_sensitive: Option<bool>,
    pub return_all_on_empty: Option<bool>,
    /// How per-key scores are combined; `None` means
    /// [`KeyScoreMode::Weighted`].
    pub score_mode: Option<KeyScoreMode>,
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
    #[inline]
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
}

/// Pre-converted haystacks and character masks (`KeyedFuzzyIndexCore`).
pub(crate) struct IndexedKeys<'a> {
    pub key_texts: &'a [Vec<String>],
    /// `Utf32String::from` of every key text.
    pub haystacks: &'a [Vec<Utf32String>],
    /// [`compute_char_mask`](super::compute_char_mask) of every key text.
    pub char_masks: &'a [Vec<u64>],
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
        pattern_score(&plan.pattern, self.haystacks[k][i].slice(..), matcher)
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
}

/// Relative slack of the early-exit bound, in units of the weight the bound
/// divides by (the total weight, or the weight of the matched and remaining
/// keys in `Matched` mode).
///
/// The bound and the final combined score are computed with different
/// floating-point summation orders, which may differ by a few ULPs of that
/// weight; this slack is orders of magnitude larger, so the early exit never
/// rejects an item whose combined score reaches `min_score`. When that weight
/// is so small (below ~2.5e-315) that the slack underflows to 0, every weight
/// involved is subnormal: a product `w * s` rounds to at most `w` and
/// sums are exact, so the bound still never rejects a qualifying item
/// (`tests/keyed_search.rs` checks such weights).
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
        let query_mask = self.plan.char_mask;
        'items: for i in 0..self.num_items {
            if E::MASK_FIRST
                && !self
                    .active_keys
                    .iter()
                    .any(|&k| self.corpus.may_match(k, i, query_mask))
            {
                continue;
            }
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

    // Only keys with a positive weight can select an item; zero-weight keys
    // are scored for the returned items only.
    let active_keys: Vec<usize> = (0..num_keys).filter(|&k| weights[k] > 0.0).collect();
    // remaining_weight[j]: sum of the weights of active_keys[j..], an upper
    // bound of what those keys can still add to the weighted sum.
    let mut remaining_weight = vec![0.0; active_keys.len() + 1];
    for j in (0..active_keys.len()).rev() {
        remaining_weight[j] = remaining_weight[j + 1] + weights[active_keys[j]];
    }

    let threshold = min_score.unwrap_or(0.0);

    let num_active = active_keys.len();
    let mut buf = Vec::new();
    let mut candidates: Vec<Candidate> = Vec::new();
    // Byte length of each candidate's best-matching key text, by slot.
    let mut tie_lens: Vec<usize> = Vec::new();
    // Active-key scores of the candidates, `num_active` per slot.
    let mut active_scores: Vec<f64> = Vec::new();
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
        for (&score, &k) in item_scores.iter().zip(&active_keys) {
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
                candidates.push(Candidate {
                    score: combined,
                    index: i as u32,
                    slot: tie_lens.len() as u32,
                });
                tie_lens.push(corpus.text_len(best_key, i));
                active_scores.extend_from_slice(item_scores);
            }
            _ => {}
        }
    };

    if C::HAS_CHAR_MASKS {
        // Item by item, so that items can be rejected early.
        let scan = MaskedScan {
            corpus,
            plan: &plan,
            num_items,
            active_keys: &active_keys,
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
        for &k in &active_keys {
            columns
                .extend((0..num_items).map(|i| corpus.key_score(&plan, matcher, &mut buf, k, i)));
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

    // Score descending, then shorter best-matching key text, then index.
    let cmp = |a: &Candidate, b: &Candidate| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| tie_lens[a.slot as usize].cmp(&tie_lens[b.slot as usize]))
            .then_with(|| a.index.cmp(&b.index))
    };

    // Top-k selection: quickselect O(n) + sort O(k log k) instead of a full
    // O(n log n) sort when maxResults is set.
    if let Some(max) = max_results {
        let k = max as usize;
        if candidates.len() > k {
            candidates.select_nth_unstable_by(k, cmp);
            candidates.truncate(k);
        }
    }
    candidates.sort_unstable_by(cmp);

    // Pass 2: key scores of the returned items only. Active keys were scored
    // in pass 1; zero-weight keys are scored now.
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
                    corpus.key_score(&plan, matcher, &mut buf, k, i)
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

/// Perform fuzzy search across multiple text keys with weights.
///
/// `key_texts[k]` is an array of strings for key `k`, one per item.
/// All inner arrays must have the same length (the number of items).
/// `weights` specifies the relative importance of each key.
///
/// Returns results sorted by combined score (best match first; see the
/// module documentation and [`KeyScoreMode`]), exactly like
/// [`KeyedFuzzyIndexCore::search`](super::KeyedFuzzyIndexCore::search)
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

    let opts = options.unwrap_or_default();
    let params = KeyedSearchParams {
        max_results: opts.max_results,
        min_score: opts.min_score,
        case_matching: resolve_case_matching(opts.is_case_sensitive),
        return_all_on_empty: opts.return_all_on_empty.unwrap_or(false),
        score_mode: opts.score_mode.unwrap_or_default(),
    };

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
