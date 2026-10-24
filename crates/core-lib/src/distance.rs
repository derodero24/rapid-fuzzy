//! String distance and similarity algorithms shared by the Node.js (napi) and
//! browser (wasm-bindgen) bindings.
//!
//! Every algorithm has three entry points that return identical scores for the
//! same pair of strings:
//!
//! - `name(a, b)` compares one pair,
//! - `name_batch(pairs)` compares many independent `[a, b]` pairs,
//! - `name_many(reference, candidates, threshold)` compares one reference with
//!   many candidates and reuses the work that only depends on the reference.
//!
//! The optional threshold of the `*_many` functions follows one rule:
//!
//! - `max_distance` (distances): a candidate whose distance exceeds it gets
//!   `max_distance + 1` instead (saturating at `u32::MAX`), or `None` for
//!   Hamming. The bindings read it from JavaScript with
//!   [`check_max_distance`].
//! - `min_similarity` (similarities): a candidate scoring below it gets `0.0`
//!   instead (`None` for normalized Hamming); a score equal to it is kept.
//!   `NaN` is rejected with [`DistanceError::NanThreshold`].

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;

use rapidfuzz::distance::damerau_levenshtein as rapid_damerau;
use rapidfuzz::distance::hamming as rapid_hamming;
use rapidfuzz::distance::indel as rapid_indel;
use rapidfuzz::distance::jaro as rapid_jaro;
use rapidfuzz::distance::jaro_winkler as rapid_jw;
use rapidfuzz::distance::levenshtein as rapid_lev;

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Invalid input to a `*_batch` or `*_many` function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceError {
    /// A `*_batch` pair that does not hold exactly two strings.
    InvalidPair {
        /// Position of the pair in the input.
        index: usize,
        /// Number of strings the pair holds.
        len: usize,
    },
    /// A `min_similarity` threshold that is `NaN`.
    NanThreshold,
    /// A `maxDistance` that is `NaN`, negative or fractional (see
    /// [`check_max_distance`]).
    InvalidMaxDistance(f64),
}

impl fmt::Display for DistanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPair { index, len } => write!(
                f,
                "pairs[{index}] must contain exactly 2 strings, got {len}"
            ),
            Self::NanThreshold => f.write_str("minSimilarity must be a number, got NaN"),
            Self::InvalidMaxDistance(value) => write!(
                f,
                "maxDistance must be a non-negative integer or Infinity, got {}",
                crate::js_number(*value)
            ),
        }
    }
}

impl std::error::Error for DistanceError {}

/// Validate the `maxDistance` threshold of a `*_many` distance function
/// coming from JavaScript (as a double), like `maxResults` is validated
/// (see `search::check_max_results`):
///
/// - `None` (not given) and `Infinity` mean no limit, and so does any value
///   of at least `u32::MAX`, which no distance reaches;
/// - NaN, negative values (including `-Infinity`) and fractions are rejected
///   with [`DistanceError::InvalidMaxDistance`];
/// - any other value is the threshold.
///
/// Reading the threshold as a `u32` instead wrapped it modulo 2^32:
/// `Infinity` and NaN became 0, so every candidate that was not identical to
/// the reference was reported as exceeding the threshold, and `-1` disabled
/// it.
pub fn check_max_distance(max_distance: Option<f64>) -> Result<Option<u32>, DistanceError> {
    let Some(value) = max_distance else {
        return Ok(None);
    };
    if value == f64::INFINITY {
        return Ok(None);
    }
    if value.is_nan() || value < 0.0 || value.fract() != 0.0 {
        return Err(DistanceError::InvalidMaxDistance(value));
    }
    Ok((value < f64::from(u32::MAX)).then_some(value as u32))
}

// ─── Shared helpers ──────────────────────────────────────────────────────────

/// Apply `f` to every `[a, b]` pair, rejecting the whole batch (before any
/// work is done) if a pair does not hold exactly two strings.
pub fn batch_apply<T, F: Fn(&str, &str) -> T>(
    pairs: &[Vec<String>],
    f: F,
) -> Result<Vec<T>, DistanceError> {
    if let Some((index, pair)) = pairs.iter().enumerate().find(|(_, p)| p.len() != 2) {
        return Err(DistanceError::InvalidPair {
            index,
            len: pair.len(),
        });
    }
    Ok(pairs.iter().map(|pair| f(&pair[0], &pair[1])).collect())
}

/// The value a `*_many` distance function returns for a candidate whose
/// distance exceeds `max_distance`.
fn distance_sentinel(max_distance: u32) -> u32 {
    max_distance.saturating_add(1)
}

/// How far below the requested threshold the cutoff handed to rapidfuzz is
/// set. rapidfuzz prunes candidates with bounds that are computed differently
/// from the final score (Jaro-Winkler, for one, converts the cutoff into a
/// Jaro cutoff), so a score exactly at the threshold can be pruned by a
/// rounding error of a few ulps. Pruning with a slightly lower cutoff and
/// comparing the exact score afterwards keeps such candidates.
const CUTOFF_SLACK: f64 = 1e-9;

/// A validated `min_similarity` threshold of a `*_many` similarity function.
#[derive(Debug, Clone, Copy)]
struct MinSimilarity(Option<f64>);

impl MinSimilarity {
    fn new(min_similarity: Option<f64>) -> Result<Self, DistanceError> {
        match min_similarity {
            Some(t) if t.is_nan() => Err(DistanceError::NanThreshold),
            // Every score is >= 0, so such a threshold keeps every candidate.
            Some(t) if t <= 0.0 => Ok(Self(None)),
            t => Ok(Self(t)),
        }
    }

    /// Whether a candidate with this `score` is kept.
    fn keeps(self, score: f64) -> bool {
        self.0.is_none_or(|t| score >= t)
    }

    /// `score` if the candidate is kept, `0.0` otherwise.
    fn apply(self, score: f64) -> f64 {
        if self.keeps(score) { score } else { 0.0 }
    }

    /// Whether no score can be kept: every similarity is at most `1.0`.
    fn rejects_all(self) -> bool {
        self.0.is_some_and(|t| t > 1.0)
    }

    /// The (slightly lowered) cutoff to prune with before the exact check.
    fn pruning_cutoff(self) -> Option<f64> {
        self.0.map(|t| t - CUTOFF_SLACK)
    }
}

/// Shared shape of the `*_many` functions backed by a rapidfuzz scorer:
/// `exact` computes the score, `pruned` computes it with a cutoff and may
/// return `None` for candidates below it.
fn similarity_many<E, P>(
    candidates: &[String],
    min_similarity: Option<f64>,
    exact: E,
    pruned: P,
) -> Result<Vec<f64>, DistanceError>
where
    E: Fn(&str) -> f64,
    P: Fn(&str, f64) -> Option<f64>,
{
    let threshold = MinSimilarity::new(min_similarity)?;
    if threshold.rejects_all() {
        return Ok(vec![0.0; candidates.len()]);
    }
    Ok(match threshold.pruning_cutoff() {
        None => candidates.iter().map(|c| exact(c)).collect(),
        Some(cutoff) => candidates
            .iter()
            .map(|c| pruned(c, cutoff).map_or(0.0, |score| threshold.apply(score)))
            .collect(),
    })
}

/// Shared shape of the `*_many` functions that compute the score themselves.
fn similarity_many_with<F>(
    candidates: &[String],
    min_similarity: Option<f64>,
    mut score: F,
) -> Result<Vec<f64>, DistanceError>
where
    F: FnMut(&str, MinSimilarity) -> f64,
{
    let threshold = MinSimilarity::new(min_similarity)?;
    if threshold.rejects_all() {
        return Ok(vec![0.0; candidates.len()]);
    }
    Ok(candidates
        .iter()
        .map(|c| threshold.apply(score(c, threshold)))
        .collect())
}

// ─── Levenshtein ────────────────────────────────────────────────────────────

pub fn levenshtein(a: &str, b: &str) -> u32 {
    rapid_lev::distance(a.chars(), b.chars()) as u32
}

pub fn levenshtein_batch(pairs: &[Vec<String>]) -> Result<Vec<u32>, DistanceError> {
    batch_apply(pairs, levenshtein)
}

pub fn levenshtein_many(
    reference: &str,
    candidates: &[String],
    max_distance: Option<u32>,
) -> Vec<u32> {
    let scorer = rapid_lev::BatchComparator::new(reference.chars());
    match max_distance {
        Some(cutoff) => {
            let args = rapid_lev::Args::default().score_cutoff(cutoff as usize);
            let sentinel = distance_sentinel(cutoff);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .map_or(sentinel, |d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()) as u32)
            .collect(),
    }
}

// ─── Damerau-Levenshtein ────────────────────────────────────────────────────

pub fn damerau_levenshtein(a: &str, b: &str) -> u32 {
    rapid_damerau::distance(a.chars(), b.chars()) as u32
}

pub fn damerau_levenshtein_batch(pairs: &[Vec<String>]) -> Result<Vec<u32>, DistanceError> {
    batch_apply(pairs, damerau_levenshtein)
}

pub fn damerau_levenshtein_many(
    reference: &str,
    candidates: &[String],
    max_distance: Option<u32>,
) -> Vec<u32> {
    let scorer = rapid_damerau::BatchComparator::new(reference.chars());
    match max_distance {
        Some(cutoff) => {
            let args = rapid_damerau::Args::default().score_cutoff(cutoff as usize);
            let sentinel = distance_sentinel(cutoff);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .map_or(sentinel, |d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()) as u32)
            .collect(),
    }
}

// ─── Hamming ─────────────────────────────────────────────────────────────────

pub fn hamming(a: &str, b: &str) -> Option<u32> {
    rapid_hamming::distance(a.chars(), b.chars())
        .ok()
        .map(|d| d as u32)
}

pub fn hamming_batch(pairs: &[Vec<String>]) -> Result<Vec<Option<u32>>, DistanceError> {
    batch_apply(pairs, hamming)
}

pub fn hamming_many(
    reference: &str,
    candidates: &[String],
    max_distance: Option<u32>,
) -> Vec<Option<u32>> {
    let scorer = rapid_hamming::BatchComparator::new(reference.chars());
    match max_distance {
        Some(cutoff) => {
            let args = rapid_hamming::Args::default().score_cutoff(cutoff as usize);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .ok()
                        .flatten()
                        .map(|d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()).ok().map(|d| d as u32))
            .collect(),
    }
}

// ─── Normalized Hamming ──────────────────────────────────────────────────────

pub fn normalized_hamming(a: &str, b: &str) -> Option<f64> {
    rapid_hamming::normalized_similarity(a.chars(), b.chars()).ok()
}

pub fn normalized_hamming_batch(pairs: &[Vec<String>]) -> Result<Vec<Option<f64>>, DistanceError> {
    batch_apply(pairs, normalized_hamming)
}

pub fn normalized_hamming_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<Option<f64>>, DistanceError> {
    let threshold = MinSimilarity::new(min_similarity)?;
    if threshold.rejects_all() {
        return Ok(vec![None; candidates.len()]);
    }
    Ok(candidates
        .iter()
        .map(|c| normalized_hamming(reference, c).filter(|&score| threshold.keeps(score)))
        .collect())
}

// ─── Jaro ────────────────────────────────────────────────────────────────────

pub fn jaro(a: &str, b: &str) -> f64 {
    rapid_jaro::similarity(a.chars(), b.chars())
}

pub fn jaro_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, jaro)
}

pub fn jaro_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let scorer = rapid_jaro::BatchComparator::new(reference.chars());
    similarity_many(
        candidates,
        min_similarity,
        |c| scorer.similarity(c.chars()),
        |c, cutoff| {
            let args = rapid_jaro::Args::default().score_cutoff(cutoff);
            scorer.similarity_with_args(c.chars(), &args)
        },
    )
}

// ─── Jaro-Winkler ────────────────────────────────────────────────────────────

pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    rapid_jw::similarity(a.chars(), b.chars())
}

pub fn jaro_winkler_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, jaro_winkler)
}

pub fn jaro_winkler_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let scorer = rapid_jw::BatchComparator::new(reference.chars());
    similarity_many(
        candidates,
        min_similarity,
        |c| scorer.similarity(c.chars()),
        |c, cutoff| {
            let args = rapid_jw::Args::default().score_cutoff(cutoff);
            scorer.similarity_with_args(c.chars(), &args)
        },
    )
}

// ─── Sorensen-Dice ───────────────────────────────────────────────────────────

/// A string prepared for the Sorensen-Dice coefficient: its characters with
/// whitespace removed, and its character bigrams packed into `u64`s and sorted
/// so two profiles intersect with a single merge pass.
///
/// The buffers are reused when a profile is reloaded, so comparing one
/// reference with many candidates allocates nothing per candidate.
#[derive(Debug, Default)]
struct DiceProfile {
    chars: Vec<char>,
    bigrams: Vec<u64>,
}

impl DiceProfile {
    fn new(s: &str) -> Self {
        let mut profile = Self::default();
        profile.load(s);
        profile
    }

    fn load(&mut self, s: &str) {
        self.chars.clear();
        self.chars.extend(s.chars().filter(|c| !c.is_whitespace()));
        self.bigrams.clear();
        self.bigrams.extend(
            self.chars
                .windows(2)
                .map(|w| (u64::from(w[0]) << 32) | u64::from(w[1])),
        );
        self.bigrams.sort_unstable();
    }

    /// `2 * |shared bigrams| / (|bigrams(a)| + |bigrams(b)|)`, counted in
    /// characters. Identical strings score `1.0`; otherwise a string with
    /// fewer than two characters has no bigrams and scores `0.0`. This is
    /// `strsim::sorensen_dice` with character instead of UTF-8 byte lengths,
    /// so ASCII input scores exactly the same.
    fn similarity(&self, other: &Self) -> f64 {
        if self.chars == other.chars {
            return 1.0;
        }
        if self.chars.len() < 2 || other.chars.len() < 2 {
            return 0.0;
        }
        let shared = sorted_intersection_len(&self.bigrams, &other.bigrams);
        (2 * shared) as f64 / (self.chars.len() + other.chars.len() - 2) as f64
    }
}

/// Size of the multiset intersection of two sorted slices.
fn sorted_intersection_len(a: &[u64], b: &[u64]) -> usize {
    let (mut i, mut j, mut shared) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                shared += 1;
                i += 1;
                j += 1;
            }
        }
    }
    shared
}

pub fn sorensen_dice(a: &str, b: &str) -> f64 {
    DiceProfile::new(a).similarity(&DiceProfile::new(b))
}

pub fn sorensen_dice_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, sorensen_dice)
}

pub fn sorensen_dice_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let reference = DiceProfile::new(reference);
    let mut candidate = DiceProfile::default();
    similarity_many_with(candidates, min_similarity, |c, _| {
        candidate.load(c);
        reference.similarity(&candidate)
    })
}

// ─── Normalized Levenshtein ──────────────────────────────────────────────────

pub fn normalized_levenshtein(a: &str, b: &str) -> f64 {
    rapid_lev::normalized_similarity(a.chars(), b.chars())
}

pub fn normalized_levenshtein_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, normalized_levenshtein)
}

pub fn normalized_levenshtein_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let scorer = rapid_lev::BatchComparator::new(reference.chars());
    similarity_many(
        candidates,
        min_similarity,
        |c| scorer.normalized_similarity(c.chars()),
        |c, cutoff| lev_similarity_at_least(&scorer, c.chars(), cutoff),
    )
}

/// Normalized Levenshtein similarity of `scorer`'s string and `other`, or
/// `None` if it is below `cutoff` (which lets rapidfuzz stop early).
fn lev_similarity_at_least<I>(
    scorer: &rapid_lev::BatchComparator<char>,
    other: I,
    cutoff: f64,
) -> Option<f64>
where
    I: IntoIterator<Item = char>,
    I::IntoIter: DoubleEndedIterator + Clone,
{
    let args = rapid_lev::Args::default().score_cutoff(cutoff);
    scorer.normalized_similarity_with_args(other, &args)
}

// ─── Indel ───────────────────────────────────────────────────────────────────

pub fn indel(a: &str, b: &str) -> u32 {
    rapid_indel::distance(a.chars(), b.chars()) as u32
}

pub fn indel_batch(pairs: &[Vec<String>]) -> Result<Vec<u32>, DistanceError> {
    batch_apply(pairs, indel)
}

pub fn indel_many(reference: &str, candidates: &[String], max_distance: Option<u32>) -> Vec<u32> {
    let scorer = rapid_indel::BatchComparator::new(reference.chars());
    match max_distance {
        Some(cutoff) => {
            let args = rapid_indel::Args::default().score_cutoff(cutoff as usize);
            let sentinel = distance_sentinel(cutoff);
            candidates
                .iter()
                .map(|c| {
                    scorer
                        .distance_with_args(c.chars(), &args)
                        .map_or(sentinel, |d| d as u32)
                })
                .collect()
        }
        None => candidates
            .iter()
            .map(|c| scorer.distance(c.chars()) as u32)
            .collect(),
    }
}

// ─── Normalized Indel ────────────────────────────────────────────────────────

pub fn normalized_indel(a: &str, b: &str) -> f64 {
    rapid_indel::normalized_similarity(a.chars(), b.chars())
}

pub fn normalized_indel_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, normalized_indel)
}

pub fn normalized_indel_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let scorer = rapid_indel::BatchComparator::new(reference.chars());
    similarity_many(
        candidates,
        min_similarity,
        |c| scorer.normalized_similarity(c.chars()),
        |c, cutoff| {
            let args = rapid_indel::Args::default().score_cutoff(cutoff);
            scorer.normalized_similarity_with_args(c.chars(), &args)
        },
    )
}

// ─── Internal helpers for token-based algorithms ─────────────────────────────

/// `word.to_lowercase()`, borrowing `word` when it is already lower-case ASCII
/// (the common case, which then needs no allocation).
fn lowercase_word(word: &str) -> Cow<'_, str> {
    if !word.is_ascii() {
        // Full Unicode lower-casing (e.g. a word-final `Σ` becomes `ς`).
        Cow::Owned(word.to_lowercase())
    } else if word.bytes().any(|b| b.is_ascii_uppercase()) {
        Cow::Owned(word.to_ascii_lowercase())
    } else {
        Cow::Borrowed(word)
    }
}

/// Lower-case `s` and collapse every whitespace run into a single space,
/// dropping leading and trailing whitespace.
pub fn normalize_str(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for word in s.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&lowercase_word(word));
    }
    result
}

/// The lower-cased whitespace-separated tokens of `s`, sorted and joined by
/// single spaces: the string token sort ratio compares.
fn sorted_token_string(s: &str) -> String {
    let mut tokens: Vec<Cow<'_, str>> = s.split_whitespace().map(lowercase_word).collect();
    tokens.sort_unstable();
    tokens.join(" ")
}

pub fn token_sort_ratio_impl(a: &str, b: &str) -> f64 {
    let sorted_a = sorted_token_string(a);
    let sorted_b = sorted_token_string(b);
    rapid_lev::normalized_similarity(sorted_a.chars(), sorted_b.chars())
}

pub fn token_set_ratio_impl(a: &str, b: &str) -> f64 {
    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);
    token_set_ratio_from_normalized(&norm_a, &norm_b)
}

/// Token set ratio of two strings already passed through [`normalize_str`].
pub fn token_set_ratio_from_normalized(norm_a: &str, norm_b: &str) -> f64 {
    if norm_a.is_empty() || norm_b.is_empty() {
        return if norm_a.is_empty() && norm_b.is_empty() {
            1.0
        } else {
            0.0
        };
    }
    let tokens_a: BTreeSet<&str> = norm_a.split_whitespace().collect();
    let tokens_b: BTreeSet<&str> = norm_b.split_whitespace().collect();
    token_set_ratio_from_tokens(&tokens_a, &tokens_b)
}

/// Token set ratio of two non-empty token sets.
///
/// With `sect` the sorted shared tokens and `diff_a` / `diff_b` the sorted
/// tokens only one side has, this is the best plain ratio among
/// `sect + diff_a` vs `sect + diff_b`, `sect` vs `sect + diff_a`, and `sect`
/// vs `sect + diff_b`. Without shared tokens it is the ratio of `diff_a` and
/// `diff_b`.
fn token_set_ratio_from_tokens(tokens_a: &BTreeSet<&str>, tokens_b: &BTreeSet<&str>) -> f64 {
    let diff_a: Vec<&str> = tokens_a.difference(tokens_b).copied().collect();
    let diff_b: Vec<&str> = tokens_b.difference(tokens_a).copied().collect();
    if diff_a.is_empty() && diff_b.is_empty() {
        return 1.0;
    }

    let sect = tokens_a
        .intersection(tokens_b)
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    // `sect` followed by `diff`, joined by a space only when both are non-empty.
    let combine = |diff: &[&str]| -> String {
        let diff = diff.join(" ");
        match (sect.is_empty(), diff.is_empty()) {
            (_, true) => sect.clone(),
            (true, false) => diff,
            (false, false) => format!("{sect} {diff}"),
        }
    };
    let combined_a = combine(&diff_a);
    let combined_b = combine(&diff_b);

    let score_ab = rapid_lev::normalized_similarity(combined_a.chars(), combined_b.chars());
    let score_a = if diff_a.is_empty() {
        1.0
    } else {
        rapid_lev::normalized_similarity(sect.chars(), combined_a.chars())
    };
    let score_b = if diff_b.is_empty() {
        1.0
    } else {
        rapid_lev::normalized_similarity(sect.chars(), combined_b.chars())
    };

    f64::max(score_ab, f64::max(score_a, score_b))
}

pub fn partial_ratio_impl(a: &str, b: &str) -> f64 {
    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);
    partial_ratio_from_normalized(&norm_a, &norm_b)
}

/// Partial ratio of two strings already passed through [`normalize_str`].
pub fn partial_ratio_from_normalized(norm_a: &str, norm_b: &str) -> f64 {
    // Every score is >= 0, so a floor of 0 always yields a score.
    partial_ratio_at_least(norm_a, norm_b, 0.0, None).unwrap_or(0.0)
}

/// Partial ratio of two normalized strings if it is at least `floor`,
/// `None` otherwise.
///
/// The shorter string is compared with every window of the same length in the
/// longer one and the best window wins. Each window is computed with the best
/// score so far (or `floor`) as the cutoff, so a high `floor` lets rapidfuzz
/// abandon windows early. `cached_a`, if given, must be the comparator of
/// `norm_a` and is reused when `norm_a` is the shorter string.
fn partial_ratio_at_least(
    norm_a: &str,
    norm_b: &str,
    floor: f64,
    cached_a: Option<&rapid_lev::BatchComparator<char>>,
) -> Option<f64> {
    if norm_a.is_empty() || norm_b.is_empty() {
        let score = if norm_a.is_empty() && norm_b.is_empty() {
            1.0
        } else {
            0.0
        };
        return (score >= floor).then_some(score);
    }

    let len_a = norm_a.chars().count();
    let len_b = norm_b.chars().count();
    if len_a == len_b {
        let args = rapid_lev::Args::default().score_cutoff(floor);
        return rapid_lev::normalized_similarity_with_args(norm_a.chars(), norm_b.chars(), &args);
    }

    let built;
    let (scorer, longer, short_len) = if len_a < len_b {
        let scorer = match cached_a {
            Some(scorer) => scorer,
            None => {
                built = rapid_lev::BatchComparator::new(norm_a.chars());
                &built
            }
        };
        (scorer, norm_b, len_a)
    } else {
        built = rapid_lev::BatchComparator::new(norm_b.chars());
        (&built, norm_a, len_b)
    };

    let long_chars: Vec<char> = longer.chars().collect();
    let mut best: Option<f64> = None;
    for window in long_chars.windows(short_len) {
        let cutoff = best.unwrap_or(floor);
        if let Some(score) = lev_similarity_at_least(scorer, window.iter().copied(), cutoff) {
            best = Some(score);
            if score == 1.0 {
                break;
            }
        }
    }
    best
}

pub fn weighted_ratio_impl(a: &str, b: &str) -> f64 {
    // The plain ratio is taken on both the original and the normalized
    // (lower-cased, whitespace collapsed) strings; `weighted_ratio_many`
    // computes the same two so the variants always agree.
    let raw_original = rapid_lev::normalized_similarity(a.chars(), b.chars());
    if raw_original == 1.0 {
        return 1.0;
    }
    let norm_a = normalize_str(a);
    let norm_b = normalize_str(b);
    // Normalization changed nothing: the second ratio would be identical.
    let raw = if norm_a == a && norm_b == b {
        raw_original
    } else {
        raw_original.max(rapid_lev::normalized_similarity(
            norm_a.chars(),
            norm_b.chars(),
        ))
    };
    if raw == 1.0 {
        return 1.0;
    }
    let sort = token_sort_ratio_impl(a, b);
    let set = token_set_ratio_from_normalized(&norm_a, &norm_b);
    best_with_partial(raw.max(sort).max(set), &norm_a, &norm_b, 0.0, None)
}

/// `max(best, partial_ratio(norm_a, norm_b))`, skipping the (expensive)
/// partial ratio when it cannot change the result: it is only computed down to
/// `max(best, floor)`, and not at all once `best` is already `1.0`.
fn best_with_partial(
    best: f64,
    norm_a: &str,
    norm_b: &str,
    floor: f64,
    cached_a: Option<&rapid_lev::BatchComparator<char>>,
) -> f64 {
    if best == 1.0 {
        return best;
    }
    partial_ratio_at_least(norm_a, norm_b, best.max(floor), cached_a)
        .map_or(best, |partial| best.max(partial))
}

// ─── Token Sort Ratio ────────────────────────────────────────────────────────

pub fn token_sort_ratio(a: &str, b: &str) -> f64 {
    token_sort_ratio_impl(a, b)
}

pub fn token_sort_ratio_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, token_sort_ratio_impl)
}

pub fn token_sort_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let sorted_ref = sorted_token_string(reference);
    let scorer = rapid_lev::BatchComparator::new(sorted_ref.chars());
    similarity_many(
        candidates,
        min_similarity,
        |c| scorer.normalized_similarity(sorted_token_string(c).chars()),
        |c, cutoff| lev_similarity_at_least(&scorer, sorted_token_string(c).chars(), cutoff),
    )
}

// ─── Token Set Ratio ─────────────────────────────────────────────────────────

pub fn token_set_ratio(a: &str, b: &str) -> f64 {
    token_set_ratio_impl(a, b)
}

pub fn token_set_ratio_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, token_set_ratio_impl)
}

pub fn token_set_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let norm_ref = normalize_str(reference);
    let tokens_ref: BTreeSet<&str> = norm_ref.split_whitespace().collect();
    similarity_many_with(candidates, min_similarity, |c, _| {
        let norm_c = normalize_str(c);
        if norm_ref.is_empty() || norm_c.is_empty() {
            return token_set_ratio_from_normalized(&norm_ref, &norm_c);
        }
        let tokens_c: BTreeSet<&str> = norm_c.split_whitespace().collect();
        token_set_ratio_from_tokens(&tokens_ref, &tokens_c)
    })
}

// ─── Partial Ratio ───────────────────────────────────────────────────────────

pub fn partial_ratio(a: &str, b: &str) -> f64 {
    partial_ratio_impl(a, b)
}

pub fn partial_ratio_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, partial_ratio_impl)
}

pub fn partial_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let norm_ref = normalize_str(reference);
    let ref_scorer = rapid_lev::BatchComparator::new(norm_ref.chars());
    similarity_many_with(candidates, min_similarity, |c, threshold| {
        let norm_c = normalize_str(c);
        let floor = threshold.pruning_cutoff().unwrap_or(0.0);
        partial_ratio_at_least(&norm_ref, &norm_c, floor, Some(&ref_scorer)).unwrap_or(0.0)
    })
}

// ─── Weighted Ratio ──────────────────────────────────────────────────────────

pub fn weighted_ratio(a: &str, b: &str) -> f64 {
    weighted_ratio_impl(a, b)
}

pub fn weighted_ratio_batch(pairs: &[Vec<String>]) -> Result<Vec<f64>, DistanceError> {
    batch_apply(pairs, weighted_ratio_impl)
}

pub fn weighted_ratio_many(
    reference: &str,
    candidates: &[String],
    min_similarity: Option<f64>,
) -> Result<Vec<f64>, DistanceError> {
    let norm_ref = normalize_str(reference);
    let tokens_ref: BTreeSet<&str> = norm_ref.split_whitespace().collect();
    let ref_scorer = rapid_lev::BatchComparator::new(norm_ref.chars());
    let ref_is_normalized = norm_ref == reference;
    // Only needed when the original reference differs from the normalized one.
    let raw_ref_scorer =
        (!ref_is_normalized).then(|| rapid_lev::BatchComparator::new(reference.chars()));
    let sorted_ref = sorted_token_string(reference);
    let sort_scorer = rapid_lev::BatchComparator::new(sorted_ref.chars());

    similarity_many_with(candidates, min_similarity, |c, threshold| {
        let norm_c = normalize_str(c);
        let raw_normalized = ref_scorer.normalized_similarity(norm_c.chars());
        if raw_normalized == 1.0 {
            return 1.0;
        }
        // Skip the original-string pass when normalization changed nothing.
        let raw = if ref_is_normalized && norm_c == *c {
            raw_normalized
        } else {
            let scorer = raw_ref_scorer.as_ref().unwrap_or(&ref_scorer);
            raw_normalized.max(scorer.normalized_similarity(c.chars()))
        };
        if raw == 1.0 {
            return 1.0;
        }

        let sort = sort_scorer.normalized_similarity(sorted_token_string(c).chars());
        let set = if norm_ref.is_empty() || norm_c.is_empty() {
            token_set_ratio_from_normalized(&norm_ref, &norm_c)
        } else {
            let tokens_c: BTreeSet<&str> = norm_c.split_whitespace().collect();
            token_set_ratio_from_tokens(&tokens_ref, &tokens_c)
        };
        let floor = threshold.pruning_cutoff().unwrap_or(0.0);
        best_with_partial(
            raw.max(sort).max(set),
            &norm_ref,
            &norm_c,
            floor,
            Some(&ref_scorer),
        )
    })
}
