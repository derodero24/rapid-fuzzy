//! Differential test: `FuzzyIndexCore` must return exactly what the standalone
//! `search_impl` returns (items, indices, scores, order, positions, match
//! types) for any items, query and options — at every corpus size (including
//! above the size where the index used to switch to bigram pre-filtering),
//! for Unicode input and nucleo query syntax, with a warm or cold incremental
//! cache, and after `add` / `add_many` / `remove`.
//!
//! The inputs come from a seeded generator so failures are reproducible: the
//! assertion message names the seed, the corpus and the query.

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::{
    FuzzyIndexCore, IndexSearchResult, MatchType, SearchResult, search_impl,
};

/// SplitMix64: tiny, deterministic, good enough for test-case generation.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
}

/// Building blocks for items: ASCII of both cases, separators, nucleo syntax
/// characters, letters whose case folding or normalization is non-trivial
/// (Latin Extended-A, Cyrillic, Greek final sigma, Kelvin sign, long s, dotted
/// capital I, titlecase digraphs, full-width Latin), NFD sequences, CJK,
/// Unicode whitespace and multi-codepoint graphemes.
const ITEM_PIECES: &[&str] = &[
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "h",
    "l",
    "n",
    "o",
    "r",
    "s",
    "x",
    "z",
    "A",
    "B",
    "D",
    "F",
    "H",
    "O",
    "R",
    "S",
    "_",
    "/",
    ".",
    "-",
    " ",
    " ",
    "$",
    "\\",
    "!",
    "^",
    "'",
    "0",
    "1",
    "4",
    "é",
    "É",
    "e\u{301}",
    "E\u{301}",
    "ł",
    "Ł",
    "ó",
    "Ó",
    "ź",
    "Ź",
    "д",
    "Д",
    "м",
    "М",
    "о",
    "О",
    "с",
    "С",
    "к",
    "К",
    "\u{212A}",
    "ſ",
    "ß",
    "ẞ",
    "İ",
    "ı",
    "Σ",
    "σ",
    "ς",
    "ǅ",
    "Ǆ",
    "ǆ",
    "ﬁ",
    "Ａ",
    "ａ",
    "東",
    "京",
    "港",
    "区",
    "\u{3000}",
    "\u{a0}",
    "\t",
    "👍",
    "👍🏽",
    "foo",
    "bar",
    "handler",
    "src/index.ts",
    "Łódź",
    "москва",
    "Москва",
];

/// Extra query pieces: syntax, escapes and every kind of whitespace.
const QUERY_PIECES: &[&str] = &[
    " ", " ", "\u{3000}", "\u{a0}", "\t", "\n", "^", "$", "!", "'", "\\", "\\ ", "\\$",
];

/// Queries that exercised the bugs fixed alongside this test.
const FIXED_QUERIES: &[&str] = &[
    "hndlr",
    "src/index.ts",
    "src index",
    "москва",
    "МОСКВА",
    "łódź",
    "ŁÓDŹ",
    "lodz",
    "ecole",
    "école",
    "k",
    "K",
    "foo\\ bar",
    "foo\\$",
    "foo\\",
    "bar$",
    "fob !zzz",
    "^f ob",
    "^",
    "'",
    "$",
    "!",
    "^$",
    "!^",
    "東京\u{3000}港区",
    "foo\u{a0}bar",
    "foo\tbar",
    "foo\nbar",
    "fo$",
    "fo$x",
];

fn gen_item(rng: &mut Rng) -> String {
    let pieces = 1 + rng.below(8);
    (0..pieces).map(|_| *rng.pick(ITEM_PIECES)).collect()
}

/// A query that is likely to match `source`: a random subsequence of its
/// characters, optionally re-cased, split into terms and decorated with syntax.
fn derived_query(rng: &mut Rng, source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut q = String::new();
    if chars.is_empty() {
        return q;
    }
    let start = rng.below(chars.len());
    for &c in &chars[start..] {
        if rng.chance(70) {
            q.push(c);
        } else if rng.chance(10) {
            q.push(' ');
        }
        if q.chars().count() >= 6 {
            break;
        }
    }
    if rng.chance(15) {
        q = q.to_uppercase();
    }
    if rng.chance(15) {
        q = q.to_lowercase();
    }
    match rng.below(10) {
        0 => q.insert(0, '^'),
        1 => q.push('$'),
        2 => q.insert(0, '\''),
        3 => q.push_str(" !zz"),
        4 => q.push_str(rng.pick::<&str>(QUERY_PIECES)),
        _ => {}
    }
    q
}

fn gen_query(rng: &mut Rng, items: &[String]) -> String {
    match rng.below(10) {
        0..=4 if !items.is_empty() => {
            let source = &items[rng.below(items.len())];
            derived_query(rng, source)
        }
        5 | 6 => (*rng.pick(FIXED_QUERIES)).to_string(),
        _ => {
            let pieces = 1 + rng.below(4);
            (0..pieces)
                .map(|_| {
                    if rng.chance(30) {
                        *rng.pick(QUERY_PIECES)
                    } else {
                        *rng.pick(ITEM_PIECES)
                    }
                })
                .collect()
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Opts {
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
}

fn gen_opts(rng: &mut Rng) -> Opts {
    Opts {
        max_results: *rng.pick(&[None, None, Some(0), Some(1), Some(3), Some(10)]),
        min_score: *rng.pick(&[None, None, Some(0.0), Some(0.3), Some(0.8)]),
        include_positions: rng.chance(50),
        case_matching: *rng.pick(&[
            CaseMatching::Smart,
            CaseMatching::Smart,
            CaseMatching::Respect,
        ]),
    }
}

type Row = (u32, u64, Vec<u32>, Option<MatchType>);

fn rows(results: &[SearchResult], items: &[String]) -> Vec<Row> {
    results
        .iter()
        .map(|r| {
            assert_eq!(
                r.item, items[r.index as usize],
                "item does not match its index"
            );
            (
                r.index,
                r.score.to_bits(),
                r.positions.clone(),
                r.match_type.clone(),
            )
        })
        .collect()
}

fn index_rows(results: &[IndexSearchResult]) -> Vec<Row> {
    results
        .iter()
        .map(|r| {
            (
                r.index,
                r.score.to_bits(),
                r.positions.clone(),
                r.match_type.clone(),
            )
        })
        .collect()
}

/// Compare `search`, `search_indices` and `closest`-style top-1 queries.
fn assert_parity(index: &FuzzyIndexCore, items: &[String], query: &str, opts: Opts, ctx: &str) {
    let expected = search_impl(
        query.to_owned(),
        items.to_vec(),
        opts.max_results,
        opts.min_score,
        opts.include_positions,
        opts.case_matching,
    );
    let expected_rows = rows(&expected, items);

    let got = index.search_impl(
        query,
        opts.max_results,
        opts.min_score,
        opts.include_positions,
        opts.case_matching,
    );
    assert_eq!(
        rows(&got, items),
        expected_rows,
        "FuzzyIndex.search differs from search(): {ctx} query={query:?} opts={opts:?}"
    );

    let got_indices = index.search_indices_impl(
        query,
        opts.max_results,
        opts.min_score,
        opts.include_positions,
        opts.case_matching,
    );
    assert_eq!(
        index_rows(&got_indices),
        expected_rows,
        "FuzzyIndex.searchIndices differs from search(): {ctx} query={query:?} opts={opts:?}"
    );
}

fn assert_closest_parity(index: &FuzzyIndexCore, items: &[String], query: &str, ctx: &str) {
    for min_score in [None, Some(0.5)] {
        let expected = search_impl(
            query.to_owned(),
            items.to_vec(),
            Some(1),
            min_score,
            false,
            CaseMatching::Smart,
        );
        let got = index.search_impl(query, Some(1), min_score, false, CaseMatching::Smart);
        assert_eq!(
            got.first().map(|r| r.item.as_str()),
            expected.first().map(|r| r.item.as_str()),
            "FuzzyIndex.closest differs from closest(): {ctx} query={query:?} min_score={min_score:?}"
        );
    }
}

/// Type a query one character at a time against the same index, as a
/// filter-as-you-type UI does, so every step runs with a warm cache.
fn assert_typing_parity(
    index: &FuzzyIndexCore,
    items: &[String],
    query: &str,
    opts: Opts,
    ctx: &str,
) {
    let mut typed = String::new();
    for c in query.chars() {
        typed.push(c);
        assert_parity(index, items, &typed, opts, ctx);
    }
}

fn filler(n: usize) -> Vec<String> {
    const WORDS: [&str; 8] = [
        "async",
        "handler",
        "service",
        "repository",
        "factory",
        "observable",
        "promise",
        "await",
    ];
    (0..n)
        .map(|i| {
            format!(
                "{}_{}_{i}",
                WORDS[i % WORDS.len()],
                WORDS[(i * 3 + 1) % WORDS.len()]
            )
        })
        .collect()
}

#[test]
fn small_corpora_match_standalone_search() {
    for seed in 0..400u64 {
        let mut rng = Rng(seed);
        let n = rng.below(40);
        let items: Vec<String> = (0..n).map(|_| gen_item(&mut rng)).collect();
        let ctx = format!("seed={seed} items={items:?}");

        // Cold cache: a fresh index for every query.
        for _ in 0..4 {
            let query = gen_query(&mut rng, &items);
            let opts = gen_opts(&mut rng);
            let index = FuzzyIndexCore::new(items.clone());
            assert_parity(&index, &items, &query, opts, &ctx);
            assert_closest_parity(&index, &items, &query, &ctx);
        }

        // Warm cache: one index, many queries, including typed prefixes.
        let index = FuzzyIndexCore::new(items.clone());
        for _ in 0..4 {
            let query = gen_query(&mut rng, &items);
            let opts = gen_opts(&mut rng);
            assert_typing_parity(&index, &items, &query, opts, &ctx);
        }
    }
}

#[test]
fn mutated_index_matches_standalone_search() {
    for seed in 1000..1150u64 {
        let mut rng = Rng(seed);
        let n = 1 + rng.below(30);
        let mut items: Vec<String> = (0..n).map(|_| gen_item(&mut rng)).collect();
        let mut index = FuzzyIndexCore::new(items.clone());
        for step in 0..12 {
            match rng.below(4) {
                0 => {
                    let item = gen_item(&mut rng);
                    index.add(item.clone());
                    items.push(item);
                }
                1 => {
                    let batch: Vec<String> =
                        (0..rng.below(4)).map(|_| gen_item(&mut rng)).collect();
                    index.add_many(batch.clone());
                    items.extend(batch);
                }
                2 if !items.is_empty() => {
                    let at = rng.below(items.len());
                    assert!(index.remove(at as u32));
                    items.swap_remove(at);
                }
                _ => {}
            }
            let ctx = format!("seed={seed} step={step} items={items:?}");
            let opts = gen_opts(&mut rng);
            let query = gen_query(&mut rng, &items);
            assert_typing_parity(&index, &items, &query, opts, &ctx);
        }
    }
}

/// Corpora above 5000 items: the size at which the index used to pre-filter
/// with an (unsound) bigram index.
#[test]
fn large_corpora_match_standalone_search() {
    for seed in 2000..2004u64 {
        let mut rng = Rng(seed);
        let mut items = filler(5000 + rng.below(1500));
        for _ in 0..300 {
            let at = rng.below(items.len());
            items[at] = gen_item(&mut rng);
        }
        items.extend(
            [
                "handler",
                "my_handler_x",
                "Москва",
                "e\u{301}cole",
                "foo bar",
                "fo$x",
                "Łódź",
            ]
            .map(String::from),
        );
        let ctx = format!("seed={seed} large corpus of {} items", items.len());
        let index = FuzzyIndexCore::new(items.clone());
        for query in FIXED_QUERIES {
            let opts = Opts {
                max_results: None,
                min_score: None,
                include_positions: true,
                case_matching: CaseMatching::Smart,
            };
            assert_parity(&index, &items, query, opts, &ctx);
        }
        for _ in 0..12 {
            let query = gen_query(&mut rng, &items);
            let opts = gen_opts(&mut rng);
            assert_parity(&index, &items, &query, opts, &ctx);
            assert_closest_parity(&index, &items, &query, &ctx);
        }
        for word in ["handler", "src/index.ts", "fo$x", "foo\\ bar"] {
            let opts = Opts {
                max_results: Some(10),
                min_score: None,
                include_positions: false,
                case_matching: CaseMatching::Smart,
            };
            assert_typing_parity(&index, &items, word, opts, &ctx);
        }
    }
}
