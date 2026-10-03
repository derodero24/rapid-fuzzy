//! Differential tests for `rapid_fuzzy_core::distance`.
//!
//! Every algorithm has a single-pair, a batch and a one-to-many entry point;
//! they must return bit-identical results for the same pair, and every
//! `*_many` threshold must follow the documented rule. The inputs are random
//! but seeded (ASCII, accents in both precomposed and combining form, CJK,
//! emoji, ZWJ sequences, whitespace, empty strings), so failures reproduce.

use rapid_fuzzy_core::distance::{self as d, DistanceError};

// ─── Deterministic input generation ──────────────────────────────────────────

/// SplitMix64: tiny, seedable and good enough for test inputs.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.next_u64() % 100 < percent
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

const ALPHABETS: &[&[&str]] = &[
    &["a", "b", "c"],
    &["a", "b", "c", "d", " "],
    &[
        "a", "é", "e\u{301}", "😀", "😃", "中", "日", "本", " ", "A", "ß", "\t",
    ],
    &["x", "y", "👍🏽", "\u{200d}", "🇯", "🇵", "\u{301}", " "],
    &["New", "new", "York", "york", "Mets", " ", "  ", "a", "b"],
    &[
        "the ", "quick ", "brown ", "fox ", "jumps ", "over ", "lazy ", "dog ",
    ],
];

fn random_string(rng: &mut Rng, alphabet: &[&str], max_units: usize) -> String {
    let units = rng.below(max_units + 1);
    (0..units).map(|_| rng.pick(alphabet)).collect()
}

/// `s` with a few character edits, so candidates land close to the reference.
fn mutate(rng: &mut Rng, s: &str, alphabet: &[&str]) -> String {
    let mut chars: Vec<String> = s.chars().map(String::from).collect();
    for _ in 0..=rng.below(3) {
        let at = rng.below(chars.len() + 1);
        match rng.below(4) {
            0 if at + 1 < chars.len() => chars.swap(at, at + 1),
            1 if at < chars.len() => {
                chars.remove(at);
            }
            2 => chars.insert(at, rng.pick(alphabet).to_string()),
            _ if at < chars.len() => chars[at] = rng.pick(alphabet).to_string(),
            _ => chars.push(rng.pick(alphabet).to_string()),
        }
    }
    chars.concat()
}

struct Case {
    reference: String,
    candidates: Vec<String>,
}

fn cases(seed: u64, count: usize) -> Vec<Case> {
    let mut rng = Rng(seed);
    (0..count)
        .map(|_| {
            let alphabet = ALPHABETS[rng.below(ALPHABETS.len())];
            // Some long strings exercise rapidfuzz's multi-word (> 64 chars) paths.
            let max_units = if rng.chance(90) { 10 } else { 70 };
            let reference = random_string(&mut rng, alphabet, max_units);
            let other = ALPHABETS[rng.below(ALPHABETS.len())];
            let twice = mutate(&mut rng, &reference, alphabet);
            let candidates = vec![
                reference.clone(),
                mutate(&mut rng, &reference, alphabet),
                mutate(&mut rng, &reference, alphabet),
                mutate(&mut rng, &twice, alphabet),
                format!("  {}  ", reference.to_uppercase()),
                random_string(&mut rng, alphabet, max_units),
                random_string(&mut rng, other, max_units),
                String::new(),
                " \t ".to_string(),
            ];
            Case {
                reference,
                candidates,
            }
        })
        .collect()
}

fn pairs_of(case: &Case) -> Vec<Vec<String>> {
    case.candidates
        .iter()
        .map(|c| vec![case.reference.clone(), c.clone()])
        .collect()
}

// ─── Families ────────────────────────────────────────────────────────────────

type Single = fn(&str, &str) -> f64;
type Batch = fn(&[Vec<String>]) -> Result<Vec<f64>, DistanceError>;
type Many = fn(&str, &[String], Option<f64>) -> Result<Vec<f64>, DistanceError>;

struct SimilarityFamily {
    name: &'static str,
    single: Single,
    batch: Batch,
    many: Many,
}

const SIMILARITY_FAMILIES: &[SimilarityFamily] = &[
    SimilarityFamily {
        name: "jaro",
        single: d::jaro,
        batch: d::jaro_batch,
        many: d::jaro_many,
    },
    SimilarityFamily {
        name: "jaro_winkler",
        single: d::jaro_winkler,
        batch: d::jaro_winkler_batch,
        many: d::jaro_winkler_many,
    },
    SimilarityFamily {
        name: "sorensen_dice",
        single: d::sorensen_dice,
        batch: d::sorensen_dice_batch,
        many: d::sorensen_dice_many,
    },
    SimilarityFamily {
        name: "normalized_levenshtein",
        single: d::normalized_levenshtein,
        batch: d::normalized_levenshtein_batch,
        many: d::normalized_levenshtein_many,
    },
    SimilarityFamily {
        name: "normalized_indel",
        single: d::normalized_indel,
        batch: d::normalized_indel_batch,
        many: d::normalized_indel_many,
    },
    SimilarityFamily {
        name: "token_sort_ratio",
        single: d::token_sort_ratio,
        batch: d::token_sort_ratio_batch,
        many: d::token_sort_ratio_many,
    },
    SimilarityFamily {
        name: "token_set_ratio",
        single: d::token_set_ratio,
        batch: d::token_set_ratio_batch,
        many: d::token_set_ratio_many,
    },
    SimilarityFamily {
        name: "partial_ratio",
        single: d::partial_ratio,
        batch: d::partial_ratio_batch,
        many: d::partial_ratio_many,
    },
    SimilarityFamily {
        name: "weighted_ratio",
        single: d::weighted_ratio,
        batch: d::weighted_ratio_batch,
        many: d::weighted_ratio_many,
    },
];

type DistSingle = fn(&str, &str) -> u32;
type DistBatch = fn(&[Vec<String>]) -> Result<Vec<u32>, DistanceError>;
type DistMany = fn(&str, &[String], Option<u32>) -> Vec<u32>;

struct DistanceFamily {
    name: &'static str,
    single: DistSingle,
    batch: DistBatch,
    many: DistMany,
}

const DISTANCE_FAMILIES: &[DistanceFamily] = &[
    DistanceFamily {
        name: "levenshtein",
        single: d::levenshtein,
        batch: d::levenshtein_batch,
        many: d::levenshtein_many,
    },
    DistanceFamily {
        name: "damerau_levenshtein",
        single: d::damerau_levenshtein,
        batch: d::damerau_levenshtein_batch,
        many: d::damerau_levenshtein_many,
    },
    DistanceFamily {
        name: "indel",
        single: d::indel,
        batch: d::indel_batch,
        many: d::indel_many,
    },
];

const FIXED_THRESHOLDS: &[f64] = &[
    0.0,
    -0.0,
    0.3,
    0.5,
    0.7,
    0.8,
    0.9,
    1.0,
    1.5,
    -0.5,
    f64::INFINITY,
    f64::NEG_INFINITY,
];

const FIXED_MAX_DISTANCES: &[u32] = &[0, 1, 2, 3, 5, 10, u32::MAX - 1, u32::MAX];

/// The documented `min_similarity` rule.
fn filter_similarity(score: f64, min_similarity: f64) -> f64 {
    if score >= min_similarity { score } else { 0.0 }
}

/// The documented `max_distance` rule.
fn filter_distance(distance: u32, max_distance: u32) -> u32 {
    if distance <= max_distance {
        distance
    } else {
        max_distance.saturating_add(1)
    }
}

/// The closest representable values around a positive score.
fn neighbours(score: f64) -> Vec<f64> {
    if score > 0.0 && score.is_finite() {
        vec![
            f64::from_bits(score.to_bits() - 1),
            f64::from_bits(score.to_bits() + 1),
        ]
    } else {
        Vec::new()
    }
}

fn assert_bits(actual: f64, expected: f64, context: impl Fn() -> String) {
    assert!(
        actual.to_bits() == expected.to_bits(),
        "{}: got {actual:?}, expected {expected:?}",
        context()
    );
}

// ─── Differential tests ──────────────────────────────────────────────────────

#[test]
fn similarity_entry_points_agree_bit_for_bit() {
    let cases = cases(0x5eed_0001, 600);
    for family in SIMILARITY_FAMILIES {
        for case in &cases {
            let reference = case.reference.as_str();
            let ctx =
                |i: usize| format!("{}({reference:?}, {:?})", family.name, case.candidates[i]);
            let scores: Vec<f64> = case
                .candidates
                .iter()
                .map(|c| (family.single)(reference, c))
                .collect();
            let batch = (family.batch)(&pairs_of(case)).unwrap();
            let many = (family.many)(reference, &case.candidates, None).unwrap();
            for (i, &score) in scores.iter().enumerate() {
                assert!((0.0..=1.0).contains(&score), "{} out of [0, 1]", ctx(i));
                assert_bits(batch[i], score, || format!("{} batch", ctx(i)));
                assert_bits(many[i], score, || format!("{} many", ctx(i)));
            }

            for &threshold in FIXED_THRESHOLDS {
                let filtered = (family.many)(reference, &case.candidates, Some(threshold)).unwrap();
                for (i, &score) in scores.iter().enumerate() {
                    assert_bits(filtered[i], filter_similarity(score, threshold), || {
                        format!("{} min_similarity={threshold}", ctx(i))
                    });
                }
            }

            // The boundary: a threshold equal to the score keeps it, one ulp
            // above drops it.
            for (i, &score) in scores.iter().enumerate() {
                let candidate = std::slice::from_ref(&case.candidates[i]);
                for threshold in std::iter::once(score).chain(neighbours(score)) {
                    let got = (family.many)(reference, candidate, Some(threshold)).unwrap()[0];
                    assert_bits(got, filter_similarity(score, threshold), || {
                        format!("{} min_similarity={threshold:?}", ctx(i))
                    });
                }
            }
        }
    }
}

#[test]
fn distance_entry_points_agree() {
    let cases = cases(0x5eed_0002, 600);
    for family in DISTANCE_FAMILIES {
        for case in &cases {
            let reference = case.reference.as_str();
            let ctx =
                |i: usize| format!("{}({reference:?}, {:?})", family.name, case.candidates[i]);
            let distances: Vec<u32> = case
                .candidates
                .iter()
                .map(|c| (family.single)(reference, c))
                .collect();
            let batch = (family.batch)(&pairs_of(case)).unwrap();
            let many = (family.many)(reference, &case.candidates, None);
            assert_eq!(batch, distances, "{} batch", ctx(0));
            assert_eq!(many, distances, "{} many", ctx(0));

            let thresholds = FIXED_MAX_DISTANCES.iter().copied().chain(
                distances
                    .iter()
                    .flat_map(|&d| [d.saturating_sub(1), d, d.saturating_add(1)]),
            );
            for max_distance in thresholds {
                let filtered = (family.many)(reference, &case.candidates, Some(max_distance));
                let expected: Vec<u32> = distances
                    .iter()
                    .map(|&d| filter_distance(d, max_distance))
                    .collect();
                assert_eq!(filtered, expected, "{} max_distance={max_distance}", ctx(0));
            }
        }
    }
}

#[test]
fn hamming_entry_points_agree() {
    for case in cases(0x5eed_0003, 600) {
        let reference = case.reference.as_str();
        let distances: Vec<Option<u32>> = case
            .candidates
            .iter()
            .map(|c| d::hamming(reference, c))
            .collect();
        assert_eq!(d::hamming_batch(&pairs_of(&case)).unwrap(), distances);
        assert_eq!(
            d::hamming_many(reference, &case.candidates, None),
            distances
        );
        for &max_distance in FIXED_MAX_DISTANCES {
            let expected: Vec<Option<u32>> = distances
                .iter()
                .map(|d| d.filter(|&d| d <= max_distance))
                .collect();
            assert_eq!(
                d::hamming_many(reference, &case.candidates, Some(max_distance)),
                expected,
                "hamming_many({reference:?}, .., {max_distance})"
            );
        }

        let scores: Vec<Option<f64>> = case
            .candidates
            .iter()
            .map(|c| d::normalized_hamming(reference, c))
            .collect();
        assert_eq!(
            d::normalized_hamming_batch(&pairs_of(&case)).unwrap(),
            scores
        );
        assert_eq!(
            d::normalized_hamming_many(reference, &case.candidates, None).unwrap(),
            scores
        );
        let exact = scores.iter().flatten().copied();
        for threshold in FIXED_THRESHOLDS.iter().copied().chain(exact) {
            let expected: Vec<Option<f64>> = scores
                .iter()
                .map(|s| s.filter(|&s| s >= threshold))
                .collect();
            assert_eq!(
                d::normalized_hamming_many(reference, &case.candidates, Some(threshold)).unwrap(),
                expected,
                "normalized_hamming_many({reference:?}, .., {threshold})"
            );
        }
    }
}

/// `weighted_ratio` is the best of the plain ratio (of the original and of the
/// normalized strings), token sort, token set and partial ratio; the partial
/// ratio is pruned with the best of the others, which must not change it.
#[test]
fn weighted_ratio_is_the_best_component() {
    for case in cases(0x5eed_0004, 600) {
        let a = case.reference.as_str();
        for b in &case.candidates {
            let raw = d::normalized_levenshtein(a, b).max(d::normalized_levenshtein(
                &d::normalize_str(a),
                &d::normalize_str(b),
            ));
            let expected = raw
                .max(d::token_sort_ratio(a, b))
                .max(d::token_set_ratio(a, b))
                .max(d::partial_ratio(a, b));
            assert_bits(d::weighted_ratio(a, b), expected, || {
                format!("weighted_ratio({a:?}, {b:?})")
            });
        }
    }
}

/// The token-based ratios lower-case word by word (ASCII words without an
/// allocation); that must equal plain `str::to_lowercase` per word, including
/// context-sensitive cases such as a word-final Greek sigma.
#[test]
fn tokenization_matches_the_plain_definition() {
    let alphabet: &[&str] = &[
        "a", "B", "Σ", "σ", "ς", "ΣΑΣ", "İ", "ǅ", "ẞ", "É", "Ab", "x", " ", "\t", "\u{3000}",
    ];
    let lowercase_words =
        |s: &str| -> Vec<String> { s.split_whitespace().map(str::to_lowercase).collect() };
    let mut rng = Rng(0x5eed_0006);
    for _ in 0..5_000 {
        let a = random_string(&mut rng, alphabet, 8);
        let b = random_string(&mut rng, alphabet, 8);
        assert_eq!(d::normalize_str(&a), lowercase_words(&a).join(" "), "{a:?}");

        let mut sorted_a = lowercase_words(&a);
        sorted_a.sort();
        let mut sorted_b = lowercase_words(&b);
        sorted_b.sort();
        let expected = d::normalized_levenshtein(&sorted_a.join(" "), &sorted_b.join(" "));
        assert_bits(d::token_sort_ratio(&a, &b), expected, || {
            format!("token_sort_ratio({a:?}, {b:?})")
        });
    }
}

// ─── Sorensen-Dice ───────────────────────────────────────────────────────────

/// For ASCII input (one byte per character) the in-crate implementation must
/// return exactly what `strsim::sorensen_dice` returned before.
#[test]
fn sorensen_dice_matches_strsim_bit_for_bit_on_ascii() {
    let mut rng = Rng(0x5eed_0005);
    let alphabets: &[&[&str]] = &[
        &["a", "b"],
        &["a", "b", "c", " "],
        &["n", "i", "g", "h", "t", "a", "c", " ", "\t", "\n"],
        &[
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z", "A", "Z", "0", "9", " ", "-", ".",
        ],
    ];
    for _ in 0..20_000 {
        let alphabet = alphabets[rng.below(alphabets.len())];
        let a = random_string(&mut rng, alphabet, 16);
        let b = if rng.chance(40) {
            mutate(&mut rng, &a, alphabet)
        } else {
            random_string(&mut rng, alphabet, 16)
        };
        let expected = strsim::sorensen_dice(&a, &b);
        assert_bits(d::sorensen_dice(&a, &b), expected, || {
            format!("sorensen_dice({a:?}, {b:?})")
        });
        assert_bits(
            d::sorensen_dice_many(&a, std::slice::from_ref(&b), None).unwrap()[0],
            expected,
            || format!("sorensen_dice_many({a:?}, [{b:?}])"),
        );
    }
}

#[test]
fn sorensen_dice_counts_characters_not_bytes() {
    let cases: &[(&str, &str, f64)] = &[
        ("日本", "日本人", 2.0 / 3.0),
        ("😀😃", "😀😃😄", 2.0 / 3.0),
        ("日本語", "日本人", 0.5),
        ("café", "cafe", 2.0 / 3.0),
        ("naïve", "naïve", 1.0),
        ("é", "é", 1.0),
        ("é", "è", 0.0),
        ("東 京", "東京", 1.0),
        // Repeated bigrams count as a multiset.
        ("ああああ", "ああ", 2.0 * 1.0 / 4.0),
    ];
    for &(a, b, expected) in cases {
        for (x, y) in [(a, b), (b, a)] {
            let score = d::sorensen_dice(x, y);
            assert!(
                (score - expected).abs() < 1e-12,
                "sorensen_dice({x:?}, {y:?}) = {score}, expected {expected}"
            );
        }
    }
}

// ─── Jaro-Winkler cutoff ─────────────────────────────────────────────────────

#[test]
fn jaro_winkler_many_keeps_a_score_equal_to_the_threshold() {
    for (reference, candidate) in [("aaac dc ", "aa"), ("\txAB", "\tXAB")] {
        let score = d::jaro_winkler(reference, candidate);
        let many = d::jaro_winkler_many(reference, &[candidate.to_string()], Some(score)).unwrap();
        assert_bits(many[0], score, || {
            format!("jaro_winkler_many({reference:?}, [{candidate:?}], {score})")
        });
    }
    assert_eq!(d::jaro_winkler("aaac dc ", "aa"), 0.8);
}

// ─── Token set ratio ─────────────────────────────────────────────────────────

#[test]
fn token_set_ratio_without_shared_tokens_compares_the_remainders() {
    assert_eq!(d::token_set_ratio("cat", "dog"), 0.0);
    assert_eq!(d::token_set_ratio("Jan", "Feb"), 0.0);
    assert_eq!(d::weighted_ratio("cat", "dog"), 0.0);
    assert_eq!(
        d::token_set_ratio("ab cd", "ab_ cd_"),
        d::normalized_levenshtein("ab cd", "ab_ cd_")
    );
    assert_eq!(
        d::token_set_ratio("b a", "xa"),
        d::normalized_levenshtein("a b", "xa")
    );
    // Shared tokens still compare `sect + rest` (unchanged behaviour).
    assert_eq!(
        d::token_set_ratio("great gatsby", "the great gatsby"),
        1.0,
        "a token subset scores 1"
    );
    assert_eq!(
        d::token_set_ratio("new york mets", "new york yankees"),
        d::normalized_levenshtein("new york mets", "new york yankees")
            .max(d::normalized_levenshtein("new york", "new york mets"))
            .max(d::normalized_levenshtein("new york", "new york yankees"))
    );
}

// ─── Errors and threshold edges ──────────────────────────────────────────────

fn pairs(raw: &[&[&str]]) -> Vec<Vec<String>> {
    raw.iter()
        .map(|p| p.iter().map(|s| s.to_string()).collect())
        .collect()
}

#[test]
fn batch_rejects_pairs_without_exactly_two_strings() {
    let short = pairs(&[&["abc", "abd"], &["abc"]]);
    let empty = pairs(&[&[]]);
    let long = pairs(&[&["abc", "abd", "zzz"]]);
    let cases = [
        (&short, DistanceError::InvalidPair { index: 1, len: 1 }),
        (&empty, DistanceError::InvalidPair { index: 0, len: 0 }),
        (&long, DistanceError::InvalidPair { index: 0, len: 3 }),
    ];
    for (input, expected) in cases {
        for family in SIMILARITY_FAMILIES {
            assert_eq!((family.batch)(input), Err(expected), "{}", family.name);
        }
        for family in DISTANCE_FAMILIES {
            assert_eq!((family.batch)(input), Err(expected), "{}", family.name);
        }
        assert_eq!(d::hamming_batch(input), Err(expected));
        assert_eq!(d::normalized_hamming_batch(input), Err(expected));
    }
    assert_eq!(
        DistanceError::InvalidPair { index: 1, len: 1 }.to_string(),
        "pairs[1] must contain exactly 2 strings, got 1"
    );
    assert_eq!(d::levenshtein_batch(&[]), Ok(vec![]));
}

#[test]
fn many_rejects_a_nan_threshold() {
    let candidates = vec!["hello".to_string(), "help".to_string()];
    for family in SIMILARITY_FAMILIES {
        assert_eq!(
            (family.many)("hello", &candidates, Some(f64::NAN)),
            Err(DistanceError::NanThreshold),
            "{}",
            family.name
        );
    }
    assert_eq!(
        d::normalized_hamming_many("hello", &candidates, Some(f64::NAN)),
        Err(DistanceError::NanThreshold)
    );
}

#[test]
fn many_filters_everything_above_one() {
    let candidates = vec!["hello".to_string(), String::new()];
    for family in SIMILARITY_FAMILIES {
        for threshold in [1.0 + f64::EPSILON, 2.0, f64::INFINITY] {
            assert_eq!(
                (family.many)("hello", &candidates, Some(threshold)).unwrap(),
                vec![0.0, 0.0],
                "{} min_similarity={threshold}",
                family.name
            );
            assert_eq!(
                (family.many)("", &[String::new()], Some(threshold)).unwrap(),
                vec![0.0],
                "{} empty strings, min_similarity={threshold}",
                family.name
            );
        }
        assert_eq!(
            (family.many)("hello", &candidates[..1], Some(1.0)).unwrap(),
            vec![1.0],
            "{} keeps an exact match at 1.0",
            family.name
        );
    }
    assert_eq!(
        d::normalized_hamming_many("abc", &["abc".to_string()], Some(2.0)).unwrap(),
        vec![None]
    );
}

#[test]
fn many_sentinel_saturates_at_u32_max() {
    let candidates = vec!["kitten".to_string(), "sitting".to_string(), String::new()];
    for family in DISTANCE_FAMILIES {
        let plain = (family.many)("kitten", &candidates, None);
        assert_eq!(
            (family.many)("kitten", &candidates, Some(u32::MAX)),
            plain,
            "{}",
            family.name
        );
        assert_eq!(
            (family.many)("kitten", &candidates, Some(u32::MAX - 1)),
            plain,
            "{}",
            family.name
        );
    }
    assert_eq!(
        d::levenshtein_many("kitten", &candidates, Some(0)),
        vec![0, 1, 1]
    );
    assert_eq!(
        d::hamming_many(
            "abc",
            &["abd".to_string(), "ab".to_string()],
            Some(u32::MAX)
        ),
        vec![Some(1), None]
    );
}
