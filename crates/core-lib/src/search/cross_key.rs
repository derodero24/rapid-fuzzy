//! Cross-key matching ([`KeyMatchMode::CrossKey`](super::KeyMatchMode)): the
//! terms of the query are matched against the keys of an item separately, so
//! that they may match different keys. See the `keys` module documentation
//! for the semantics, which this module implements for both the standalone
//! search and `KeyedFuzzyIndexCore`.

use nucleo_matcher::pattern::{Atom, Pattern};
use nucleo_matcher::{Matcher, Utf32Str};

use super::keys::{ActiveKeys, KeyScoreMode, KeyedCorpus, Selection};
use super::{atom_char_mask, atom_max_score};

/// A term of the query.
struct Term<'p> {
    atom: &'p Atom,
    /// Character mask of the term's needle (see [`atom_char_mask`]): a text
    /// lacking one of its bits cannot match the needle.
    mask: u64,
    /// For a positive term, the raw score of a perfect match (`m(t)` in the
    /// module documentation, at least 1); the term's score on a key is
    /// capped at it. 0 for a negative term.
    cap: u32,
}

impl Term<'_> {
    /// Raw score of a positive term on `haystack`, at most `cap`, or 0 when
    /// the term does not match it.
    #[inline]
    fn score(&self, haystack: Utf32Str<'_>, matcher: &mut Matcher) -> u32 {
        self.atom
            .score(haystack, matcher)
            .map_or(0, |raw| u32::from(raw).min(self.cap))
    }

    /// Whether the needle of a negative term (`!term`) matches `haystack`.
    #[inline]
    fn excludes(&self, haystack: Utf32Str<'_>, matcher: &mut Matcher) -> bool {
        // nucleo scores a negative atom `None` exactly when its needle
        // matches the haystack.
        self.atom.score(haystack, matcher).is_none()
    }
}

/// The haystack of key `k` of item `i`, preparing `buf` for it unless
/// `prepared` says it already is (each key text is converted at most once
/// per item, however many terms are matched against it).
#[inline]
fn item_haystack<'b, C: KeyedCorpus>(
    corpus: &'b C,
    k: usize,
    i: usize,
    buf: &'b mut Vec<char>,
    prepared: &mut bool,
) -> Utf32Str<'b> {
    if !*prepared {
        corpus.prepare_haystack(k, i, buf);
        *prepared = true;
    }
    corpus.haystack(k, i, buf)
}

/// The first item from `start` on in which every mask of `term_masks` is
/// contained in the character mask of one of `active_keys` (each term may
/// match one of its keys), or `None`. Kept out of line for the reason given
/// at [`first_may_match`](super::keys::first_may_match).
#[inline(never)]
fn first_may_match_all<C: KeyedCorpus>(
    corpus: &C,
    active_keys: &[usize],
    term_masks: &[u64],
    start: usize,
) -> Option<usize> {
    let num_items = corpus.key_texts().first().map_or(0, Vec::len);
    (start..num_items).find(|&i| {
        term_masks
            .iter()
            .all(|&mask| active_keys.iter().any(|&k| corpus.may_match(k, i, mask)))
    })
}

/// How the scan rejects an item before all of its positive terms are
/// scored, once `min_score` is set: after each term, it checks whether even
/// perfect scores of the remaining terms on every key could reach
/// `min_score`.
///
/// The bounds are computed with the same floating-point operations as the
/// final score, from integer inputs at least as large. Correctly rounded
/// arithmetic is monotonic, so a bound is never below the score the item
/// ends up with, and no slack is needed.
trait Bound {
    /// Start the next item.
    fn start(&mut self);

    /// Account for the scores `row` of the next positive term on the active
    /// keys. True when the item cannot reach `min_score`, whatever the
    /// positive terms after it (whose caps sum to `remaining`) score.
    fn unreachable(&mut self, row: &[u32], remaining: u64) -> bool;
}

/// Without `min_score`, and in `Matched` mode: only the final check rejects
/// an item. (In `Matched` mode, a perfect match of a remaining term on a new
/// key adds as much to the denominator as to the numerator and raises the
/// score of an item matching partially, so no simple exact bound applies.)
struct NoBound;

impl Bound for NoBound {
    #[inline(always)]
    fn start(&mut self) {}

    #[inline(always)]
    fn unreachable(&mut self, _row: &[u32], _remaining: u64) -> bool {
        false
    }
}

/// `Max` mode: the score is `sum_t max_k r(t, k) / M`; each remaining term
/// adds at most its cap to the sum.
struct MaxBound {
    best_sum: u64,
    max_score: f64,
    threshold: f64,
}

impl Bound for MaxBound {
    #[inline(always)]
    fn start(&mut self) {
        self.best_sum = 0;
    }

    #[inline(always)]
    fn unreachable(&mut self, row: &[u32], remaining: u64) -> bool {
        self.best_sum += u64::from(row.iter().copied().max().unwrap_or(0));
        ((self.best_sum + remaining) as f64 / self.max_score) < self.threshold
    }
}

/// `Weighted` mode: the score is `sum_k w * s(k) / sum(w)`; each remaining
/// term adds at most its cap to the raw sum of every key.
struct WeightedBound<'a> {
    /// Raw sums of the terms scored so far, by active key.
    sums: Vec<u64>,
    /// The weights of the active keys, in order.
    active_weights: &'a [f64],
    max_score: f64,
    total_weight: f64,
    threshold: f64,
}

impl Bound for WeightedBound<'_> {
    #[inline(always)]
    fn start(&mut self) {
        self.sums.fill(0);
    }

    #[inline(always)]
    fn unreachable(&mut self, row: &[u32], remaining: u64) -> bool {
        let mut weighted_sum = 0.0;
        for ((sum, &score), &weight) in self.sums.iter_mut().zip(row).zip(self.active_weights) {
            *sum += u64::from(score);
            // As in `CrossKeyPlan::select`.
            let key_score = (*sum + remaining) as f64 / self.max_score;
            weighted_sum += key_score * weight;
        }
        weighted_sum / self.total_weight < self.threshold
    }
}

/// A query prepared for cross-key matching.
pub(super) struct CrossKeyPlan<'p> {
    /// The positive terms, in query order.
    positive: Vec<Term<'p>>,
    /// The negative terms (`!term`), in query order.
    negative: Vec<Term<'p>>,
    /// `remaining_caps[j]`: the sum of the caps of `positive[j..]`.
    remaining_caps: Vec<u64>,
    /// `M`, the sum of the caps of the positive terms (at least 1): key
    /// scores are raw sums divided by it.
    max_score: f64,
}

impl<'p> CrossKeyPlan<'p> {
    /// Prepare the terms of `pattern`, parsed like every search parses its
    /// query.
    pub(super) fn new(pattern: &'p Pattern, matcher: &mut Matcher) -> Self {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        for atom in &pattern.atoms {
            if atom.negative {
                negative.push(Term {
                    atom,
                    mask: atom_char_mask(atom),
                    cap: 0,
                });
            } else {
                positive.push(Term {
                    atom,
                    mask: atom_char_mask(atom),
                    // At least 1, so that a match never makes the term
                    // count as unmatched; for a term of one character or
                    // more it is at least 16 anyway.
                    cap: atom_max_score(atom, matcher).max(1),
                });
            }
        }
        let mut remaining_caps = vec![0; positive.len() + 1];
        for j in (0..positive.len()).rev() {
            remaining_caps[j] = remaining_caps[j + 1] + u64::from(positive[j].cap);
        }
        let max_score = remaining_caps[0].max(1) as f64;
        Self {
            positive,
            negative,
            remaining_caps,
            max_score,
        }
    }

    /// Pass 1: the items that qualify, with their active-key scores. Kept
    /// out of line (see `per_key_pass`).
    #[inline(never)]
    pub(super) fn pass<C: KeyedCorpus>(
        &self,
        corpus: &C,
        keys: &ActiveKeys<'_>,
        score_mode: KeyScoreMode,
        threshold: f64,
        matcher: &mut Matcher,
    ) -> Selection {
        let mut selection = Selection::new();
        if self.positive.is_empty() {
            // Every item would score 0.
            return selection;
        }
        let active_weights: Vec<f64> = keys.active_keys.iter().map(|&k| keys.weights[k]).collect();
        let scan = Scan {
            plan: self,
            corpus,
            keys,
            active_weights: &active_weights,
            score_mode,
            threshold,
        };
        // NaN thresholds never prune (and the final check rejects every item).
        let prune = threshold > 0.0;
        match score_mode {
            _ if !prune => scan.run(NoBound, matcher, &mut selection),
            KeyScoreMode::Max => {
                let bound = MaxBound {
                    best_sum: 0,
                    max_score: self.max_score,
                    threshold,
                };
                scan.run(bound, matcher, &mut selection);
            }
            KeyScoreMode::Weighted => {
                let bound = WeightedBound {
                    sums: vec![0; active_weights.len()],
                    active_weights: &active_weights,
                    max_score: self.max_score,
                    total_weight: keys.total_weight,
                    threshold,
                };
                scan.run(bound, matcher, &mut selection);
            }
            KeyScoreMode::Matched => scan.run(NoBound, matcher, &mut selection),
        }
        selection
    }

    /// The key score of key `k` of item `i` (any key, including keys whose
    /// weight is zero): the share of the query it matches.
    pub(super) fn key_score<C: KeyedCorpus>(
        &self,
        corpus: &C,
        matcher: &mut Matcher,
        buf: &mut Vec<char>,
        k: usize,
        i: usize,
    ) -> f64 {
        let mut prepared = false;
        let mut sum = 0u64;
        for term in &self.positive {
            if corpus.may_match(k, i, term.mask) {
                let haystack = item_haystack(corpus, k, i, buf, &mut prepared);
                sum += u64::from(term.score(haystack, matcher));
            }
        }
        sum as f64 / self.max_score
    }
}

/// The item-by-item scan of pass 1.
struct Scan<'s, 'p, C> {
    plan: &'s CrossKeyPlan<'p>,
    corpus: &'s C,
    keys: &'s ActiveKeys<'s>,
    /// The weights of the active keys, in order.
    active_weights: &'s [f64],
    score_mode: KeyScoreMode,
    threshold: f64,
}

impl<C: KeyedCorpus> Scan<'_, '_, C> {
    /// Score every item, rejecting it as soon as a positive term matches none
    /// of its active keys or `bound` rules it out, and select those whose
    /// every positive term matches and that no negative term excludes.
    /// Monomorphized per [`Bound`], so that each mode gets a loop of its own.
    fn run<B: Bound>(&self, mut bound: B, matcher: &mut Matcher, selection: &mut Selection) {
        let Self { plan, corpus, .. } = *self;
        let ActiveKeys {
            num_items,
            active_keys,
            ..
        } = *self.keys;
        let num_active = active_keys.len();
        // scores[j * num_active + a]: score of positive term j on active key a.
        let mut scores = vec![0u32; plan.positive.len() * num_active];
        let mut key_scores = vec![0.0; num_active];
        // The haystacks of the current item's active keys, prepared on
        // first use.
        let mut bufs = vec![Vec::new(); num_active];
        let mut prepared = vec![false; num_active];
        let term_masks: Vec<u64> = plan.positive.iter().map(|term| term.mask).collect();

        let mut next = 0;
        'items: while next < num_items {
            // Every positive term must be able to match an active key: a
            // cheap check of the character masks first.
            let i = if C::HAS_CHAR_MASKS {
                match first_may_match_all(corpus, active_keys, &term_masks, next) {
                    Some(i) => i,
                    None => break,
                }
            } else {
                next
            };
            next = i + 1;
            prepared.fill(false);
            bound.start();
            for (j, term) in plan.positive.iter().enumerate() {
                let row = &mut scores[j * num_active..(j + 1) * num_active];
                let mut matched = false;
                for (a, &k) in active_keys.iter().enumerate() {
                    row[a] = if corpus.may_match(k, i, term.mask) {
                        let haystack = item_haystack(corpus, k, i, &mut bufs[a], &mut prepared[a]);
                        term.score(haystack, matcher)
                    } else {
                        0
                    };
                    matched |= row[a] > 0;
                }
                if !matched || bound.unreachable(row, plan.remaining_caps[j + 1]) {
                    continue 'items;
                }
            }
            for term in &plan.negative {
                for (a, &k) in active_keys.iter().enumerate() {
                    if corpus.may_match(k, i, term.mask) {
                        let haystack = item_haystack(corpus, k, i, &mut bufs[a], &mut prepared[a]);
                        if term.excludes(haystack, matcher) {
                            continue 'items;
                        }
                    }
                }
            }
            self.select(i, &scores, &mut key_scores, selection);
        }
    }

    /// Combine the scores of item `i`, whose every positive term matches an
    /// active key and which no negative term excludes, and select it if it
    /// qualifies.
    #[inline]
    fn select(&self, i: usize, scores: &[u32], key_scores: &mut [f64], selection: &mut Selection) {
        let Self {
            plan,
            corpus,
            active_weights,
            score_mode,
            threshold,
            ..
        } = *self;
        let ActiveKeys {
            active_keys,
            total_weight,
            ..
        } = *self.keys;
        let num_active = active_keys.len();

        let mut weighted_sum = 0.0;
        // `sum(w * c)`, the weight of the keys scaled by their coverage
        // (`Matched` mode).
        let mut covered_weight = 0.0;
        // (contribution, key) of the best-matching key so far.
        let mut best: Option<(f64, usize)> = None;
        for (a, (&k, &weight)) in active_keys.iter().zip(active_weights).enumerate() {
            let mut sum = 0u64;
            let mut covered = 0u64;
            for (j, term) in plan.positive.iter().enumerate() {
                let score = scores[j * num_active + a];
                sum += u64::from(score);
                if score > 0 {
                    covered += u64::from(term.cap);
                }
            }
            let key_score = sum as f64 / plan.max_score;
            key_scores[a] = key_score;
            let weighted = key_score * weight;
            weighted_sum += weighted;
            if score_mode == KeyScoreMode::Matched {
                covered_weight += (covered as f64 / plan.max_score) * weight;
            }
            let contribution = match score_mode {
                KeyScoreMode::Max => key_score,
                KeyScoreMode::Weighted | KeyScoreMode::Matched => weighted,
            };
            if contribution > best.map_or(0.0, |(c, _)| c) {
                best = Some((contribution, k));
            }
        }
        let combined = match score_mode {
            KeyScoreMode::Weighted => weighted_sum / total_weight,
            // covered_weight >= weighted_sum > 0 (a key's coverage is at
            // least its score), so the mean is at most 1.
            KeyScoreMode::Matched if weighted_sum > 0.0 => weighted_sum / covered_weight,
            KeyScoreMode::Matched => 0.0,
            KeyScoreMode::Max => {
                let best_sum: u64 = scores
                    .chunks_exact(num_active)
                    .map(|row| u64::from(row.iter().copied().max().unwrap_or(0)))
                    .sum();
                best_sum as f64 / plan.max_score
            }
        };
        match best {
            Some((_, best_key)) if combined > 0.0 && combined >= threshold => {
                selection.push(i, combined, corpus.text_len(best_key, i), key_scores);
            }
            _ => {}
        }
    }
}
