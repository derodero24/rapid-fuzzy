//! Differential tests for `rapid_fuzzy_core::distance`.
//!
//! The single-pair, batch and one-to-many entry points must return
//! bit-identical results for the same pair. The inputs are random but seeded
//! (ASCII, accents in both precomposed and combining form, CJK, emoji, ZWJ
//! sequences, whitespace, empty strings), so failures reproduce.

use rapid_fuzzy_core::distance as d;

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

fn assert_bits(actual: f64, expected: f64, context: impl Fn() -> String) {
    assert!(
        actual.to_bits() == expected.to_bits(),
        "{}: got {actual:?}, expected {expected:?}",
        context()
    );
}

// ─── Differential tests ──────────────────────────────────────────────────────

#[test]
fn sorensen_dice_entry_points_agree_bit_for_bit() {
    for case in cases(0x5eed_0001, 600) {
        let reference = case.reference.as_str();
        let batch = d::sorensen_dice_batch(&pairs_of(&case));
        let many = d::sorensen_dice_many(reference, &case.candidates, None);
        for (i, candidate) in case.candidates.iter().enumerate() {
            let score = d::sorensen_dice(reference, candidate);
            let ctx = || format!("sorensen_dice({reference:?}, {candidate:?})");
            assert_bits(batch[i], score, || format!("{} batch", ctx()));
            assert_bits(many[i], score, || format!("{} many", ctx()));
        }
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
            d::sorensen_dice_many(&a, std::slice::from_ref(&b), None)[0],
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
