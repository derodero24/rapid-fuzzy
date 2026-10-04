//! Large searches split their work across threads on native targets. This
//! checks that they return exactly what a plain sequential search returns:
//! every match, in the same order, with bit-identical scores, positions and
//! match types — for `FuzzyIndexCore` and for the standalone search.
//!
//! The reference below scores every item one after another on the calling
//! thread with nucleo directly, then ranks the matches with the documented
//! order (score descending, then shorter item, then lower index). The
//! corpora are large and their items long, so the searches under test are
//! well above the cost at which they go parallel (even more so in debug
//! builds).

use std::cmp::Ordering;

use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher};
use rapid_fuzzy_core::search::{
    FuzzyIndexCore, MatchType, QueryPlan, classify_match, normalize_query_whitespace, search_impl,
    utf32_haystack,
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
}

const WORDS: &[&str] = &[
    "src",
    "index",
    "handler",
    "Repository",
    "service",
    "controller",
    "lib",
    "python3",
    "usr",
    "ctrl",
    "abc",
    "test",
    "node_modules",
    "components",
    "Москва",
    "café",
    "東京",
    "ÉCOLE",
    "naïve",
];

const SEPARATORS: &[&str] = &["/", "_", "-", ".", " "];

/// `n` path-like items of 2 to 25 words. Lengths vary a lot, and short items
/// come first, like in sorted file lists.
fn corpus(seed: u64, n: usize) -> Vec<String> {
    let mut rng = Rng(seed);
    let mut items: Vec<String> = (0..n)
        .map(|_| {
            let words = 2 + rng.below(24);
            let mut item = String::new();
            for i in 0..words {
                if i > 0 {
                    item.push_str(SEPARATORS[rng.below(SEPARATORS.len())]);
                }
                item.push_str(WORDS[rng.below(WORDS.len())]);
                if rng.below(4) == 0 {
                    item.push_str(&rng.below(100).to_string());
                }
            }
            item
        })
        .collect();
    items.sort_by_key(String::len);
    items
}

const QUERIES: &[&str] = &[
    "a",
    "hndlr",
    "src index",
    "usr ctrl",
    "srv ctl lib",
    "^src",
    "test$",
    "'handler !python",
    "москва",
    "Cafe",
    "東京 idx",
    "repository",
];

#[derive(Debug, PartialEq)]
struct Row {
    index: u32,
    /// Bit pattern of the score, so that scores compare exactly.
    score: u64,
    positions: Vec<u32>,
    match_type: Option<MatchType>,
}

/// Sequential reference search.
fn reference(
    items: &[String],
    query: &str,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
) -> Vec<Row> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let query = normalize_query_whitespace(query);
    let Some(plan) = QueryPlan::new(&query, CaseMatching::Smart, &mut matcher) else {
        return Vec::new();
    };
    let threshold = min_score.unwrap_or(0.0);
    let mut buf = Vec::new();
    let mut scored: Vec<(u32, f64)> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let haystack = utf32_haystack(item, &mut buf);
        if let Some(raw) = plan.pattern.score(haystack, &mut matcher) {
            let score = plan.normalize(raw);
            if score >= threshold {
                scored.push((index as u32, score));
            }
        }
    }
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| items[a.0 as usize].len().cmp(&items[b.0 as usize].len()))
            .then_with(|| a.0.cmp(&b.0))
    });
    if let Some(max) = max_results {
        scored.truncate(max as usize);
    }
    scored
        .into_iter()
        .map(|(index, score)| {
            let (positions, match_type) = if include_positions {
                let haystack = utf32_haystack(&items[index as usize], &mut buf);
                let mut positions = Vec::new();
                plan.pattern.indices(haystack, &mut matcher, &mut positions);
                positions.sort_unstable();
                positions.dedup();
                let match_type = classify_match(&positions, haystack.len());
                (positions, Some(match_type))
            } else {
                (Vec::new(), None)
            };
            Row {
                index,
                score: score.to_bits(),
                positions,
                match_type,
            }
        })
        .collect()
}

type Options = (Option<u32>, Option<f64>, bool);

const OPTIONS: &[Options] = &[
    (None, None, false),
    (Some(10), None, false),
    (None, None, true),
    (Some(3000), Some(0.3), true),
];

fn check_corpus(items: &[String], queries: &[&str]) {
    let index = FuzzyIndexCore::new(items.to_vec());
    for &query in queries {
        for &(max_results, min_score, include_positions) in OPTIONS {
            let expected = reference(items, query, max_results, min_score, include_positions);
            let ctx = format!(
                "query {query:?}, maxResults {max_results:?}, minScore {min_score:?}, \
                 positions {include_positions}"
            );

            let got: Vec<Row> = index
                .search_impl(
                    query,
                    max_results,
                    min_score,
                    include_positions,
                    CaseMatching::Smart,
                )
                .into_iter()
                .map(|r| {
                    assert_eq!(r.item, items[r.index as usize], "{ctx}");
                    Row {
                        index: r.index,
                        score: r.score.to_bits(),
                        positions: r.positions,
                        match_type: r.match_type,
                    }
                })
                .collect();
            assert_eq!(got, expected, "FuzzyIndex, {ctx}");

            let standalone: Vec<Row> = search_impl(
                query.to_string(),
                items.to_vec(),
                max_results,
                min_score,
                include_positions,
                CaseMatching::Smart,
            )
            .into_iter()
            .map(|r| Row {
                index: r.index,
                score: r.score.to_bits(),
                positions: r.positions,
                match_type: r.match_type,
            })
            .collect();
            assert_eq!(standalone, expected, "standalone search, {ctx}");
        }
    }
}

#[test]
fn large_searches_match_sequential_reference() {
    check_corpus(&corpus(0x5EED, 3_000), QUERIES);
}

#[test]
fn searches_near_the_parallel_threshold_match_sequential_reference() {
    // Around the smallest input that is considered for splitting.
    for n in [1_023, 1_024, 1_025] {
        check_corpus(&corpus(n as u64, n), &QUERIES[..4]);
    }
}

#[test]
fn incremental_type_ahead_matches_sequential_reference() {
    let items = corpus(7, 8_000);
    let index = FuzzyIndexCore::new(items.clone());
    for query in ["h", "ha", "han", "hand", "handl", "handle", "handler"] {
        let got: Vec<(u32, u64)> = index
            .search_indices_impl(query, None, None, false, CaseMatching::Smart)
            .into_iter()
            .map(|r| (r.index, r.score.to_bits()))
            .collect();
        let expected: Vec<(u32, u64)> = reference(&items, query, None, None, false)
            .into_iter()
            .map(|row| (row.index, row.score))
            .collect();
        assert_eq!(got, expected, "query {query:?}");
    }
}

#[test]
fn concurrent_searches_match_sequential_reference() {
    // Searches running on several threads at once share the thread pool.
    let items = corpus(11, 6_000);
    std::thread::scope(|scope| {
        for &query in &QUERIES[..4] {
            let items = &items;
            scope.spawn(move || {
                let expected = reference(items, query, None, None, true);
                let got: Vec<Row> = search_impl(
                    query.to_string(),
                    items.clone(),
                    None,
                    None,
                    true,
                    CaseMatching::Smart,
                )
                .into_iter()
                .map(|r| Row {
                    index: r.index,
                    score: r.score.to_bits(),
                    positions: r.positions,
                    match_type: r.match_type,
                })
                .collect();
                assert_eq!(got, expected, "query {query:?}");
            });
        }
    });
}

#[test]
fn ranking_large_result_sets_matches_sequential_reference() {
    // More matches than the size above which they are sorted in parallel,
    // with many equal scores so that the length and index tie-breaks matter.
    let items: Vec<String> = (0..40_000)
        .map(|i| format!("{}item_{}", ["", "x", "ab_"][i % 3], (i * 7919) % 1000))
        .collect();
    let index = FuzzyIndexCore::new(items.clone());
    for query in ["item", "i 9", "^item"] {
        for max_results in [None, Some(35_000), Some(100)] {
            let expected: Vec<(u32, u64)> = reference(&items, query, max_results, None, false)
                .into_iter()
                .map(|row| (row.index, row.score))
                .collect();
            let got: Vec<(u32, u64)> = index
                .search_indices_impl(query, max_results, None, false, CaseMatching::Smart)
                .into_iter()
                .map(|r| (r.index, r.score.to_bits()))
                .collect();
            assert_eq!(got, expected, "query {query:?}, maxResults {max_results:?}");
        }
    }
}
