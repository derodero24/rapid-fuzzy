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
//!   zero (they are informational there).
//! * The combined score is `sum(key_score * weight) / sum(weights)`.
//! * Items whose combined score is 0 — no match on any key with a positive
//!   weight — are excluded, as are items scoring below `min_score`.
//! * Results are sorted by combined score (descending), then by the UTF-8
//!   byte length of the best-matching key's text (shorter first, like
//!   `search()` prefers shorter items), then by index. The best-matching key
//!   is the one contributing the most to the combined score (`key_score *
//!   weight`); on a tie, the first such key.

use std::cmp::Ordering;

use nucleo_matcher::pattern::{CaseMatching, Pattern};
use nucleo_matcher::{Matcher, Utf32Str, Utf32String};

use super::{
    KeySearchResult, QueryPlan, is_empty_query, resolve_case_matching, utf32_haystack, with_matcher,
};

/// Search options for the `search_keys_impl` function.
pub struct SearchKeysOptions {
    pub max_results: Option<u32>,
    pub min_score: Option<f64>,
    pub is_case_sensitive: Option<bool>,
    pub return_all_on_empty: Option<bool>,
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
}

/// Relative slack of the early-exit bound, in units of the total weight.
///
/// The bound and the final combined score are computed with different
/// floating-point summation orders, which may differ by a few ULPs of the
/// total weight; this slack is orders of magnitude larger, so the early exit
/// never rejects an item whose combined score reaches `min_score`.
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
    // NaN thresholds never prune (and the final check rejects every item).
    let early_exit = threshold > 0.0;
    let required_weighted_sum = threshold * total_weight - total_weight * EARLY_EXIT_SLACK;

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
        // (contribution, key) of the best-matching key so far.
        let mut best: Option<(f64, usize)> = None;
        for (&score, &k) in item_scores.iter().zip(&active_keys) {
            let contribution = score * weights[k];
            weighted_sum += contribution;
            if contribution > best.map_or(0.0, |(c, _)| c) {
                best = Some((contribution, k));
            }
        }
        let combined = weighted_sum / total_weight;
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
        let query_mask = plan.char_mask;
        'items: for i in 0..num_items {
            // Without min_score, reject items none of whose weighted keys
            // can match (by character mask) in one tight check. With
            // min_score the early exit below usually rejects them sooner.
            if !early_exit
                && !active_keys
                    .iter()
                    .any(|&k| corpus.may_match(k, i, query_mask))
            {
                continue;
            }
            let mut weighted_sum = 0.0;
            for (j, &k) in active_keys.iter().enumerate() {
                let score = corpus.key_score(&plan, matcher, &mut buf, k, i);
                item_scores[j] = score;
                weighted_sum += score * weights[k];
                // Early exit: even perfect scores on the remaining keys
                // cannot reach min_score.
                if early_exit && weighted_sum + remaining_weight[j + 1] < required_weighted_sum {
                    continue 'items;
                }
            }
            // A weighted sum of 0 means a combined score of 0: not selected.
            if weighted_sum > 0.0 {
                select(i, &item_scores);
            }
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
/// Returns results sorted by combined weighted score (best match first),
/// exactly like [`KeyedFuzzyIndexCore::search`](super::KeyedFuzzyIndexCore::search)
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

    let params = match options {
        Some(opts) => KeyedSearchParams {
            max_results: opts.max_results,
            min_score: opts.min_score,
            case_matching: resolve_case_matching(opts.is_case_sensitive),
            return_all_on_empty: opts.return_all_on_empty.unwrap_or(false),
        },
        None => KeyedSearchParams {
            max_results: None,
            min_score: None,
            case_matching: CaseMatching::Smart,
            return_all_on_empty: false,
        },
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
