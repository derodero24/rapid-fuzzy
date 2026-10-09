//! Benchmarks of rapid-fuzzy's own code (`rapid-fuzzy-core`), so CodSpeed
//! reports regressions in the shipped algorithms. The `distance` and `search`
//! benches measure the third-party crates the core builds on.

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::distance;
use rapid_fuzzy_core::search::{
    FuzzyIndexCore, KeyMatchMode, KeyScoreMode, KeyedFuzzyIndexCore, SearchKeysOptions,
    search_impl, search_keys_impl,
};

const WORDS: [&str; 20] = [
    "async",
    "await",
    "function",
    "class",
    "interface",
    "type",
    "export",
    "import",
    "const",
    "let",
    "return",
    "promise",
    "observable",
    "subscriber",
    "handler",
    "middleware",
    "controller",
    "service",
    "repository",
    "factory",
];

const DESCRIPTIONS: [&str; 6] = [
    "Handles incoming requests",
    "Café menu repository",
    "Naïve retry middleware",
    "Observable event stream",
    "Factory for service instances",
    "Exports the public interface",
];

/// Index sizes: below and above the 5,000-item threshold where the index
/// switches on its bigram prefilter.
const INDEX_SIZES: [usize; 2] = [1_000, 10_000];

/// Queries run against every index and the one-shot search:
/// - `substring`: a contiguous piece of the `controller_*` items;
/// - `fuzzy`: `handler` with gaps, the typical fuzzy-finder input;
/// - `miss`: matches nothing, exercising the character-mask rejection path.
const QUERIES: [(&str, &str); 3] = [
    ("substring", "ntroll"),
    ("fuzzy", "hndlr"),
    ("miss", "zqxjv"),
];
/// Queries whose terms are in different keys, for cross-key matching:
/// - `terms`: a `handler_*` item and a "Handles incoming requests"
///   description (one item in 60 has both);
/// - `terms_miss`: the second term matches nothing.
const TERM_QUERIES: [(&str, &str); 2] = [
    ("terms", "handler requests"),
    ("terms_miss", "handler zqxjv"),
];

/// Keystrokes of a user typing `handler`; each step may reuse the previous
/// step's matches, and the next round starts over because "h" is shorter.
const TYPING: [&str; 5] = ["h", "ha", "han", "hand", "handl"];

fn items(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| format!("{}_{}", WORDS[i % WORDS.len()], i))
        .collect()
}

fn descriptions(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| format!("{} #{}", DESCRIPTIONS[i % DESCRIPTIONS.len()], i))
        .collect()
}

fn bench_fuzzy_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("core_fuzzy_index");
    for size in INDEX_SIZES {
        let data = items(size);
        group.bench_with_input(BenchmarkId::new("build", size), &data, |b, data| {
            b.iter_batched(|| data.clone(), FuzzyIndexCore::new, BatchSize::LargeInput);
        });

        let index = FuzzyIndexCore::new(data.clone());
        for (name, query) in QUERIES {
            group.bench_function(BenchmarkId::new(format!("search_{name}"), size), |b| {
                b.iter(|| {
                    index.search_impl(black_box(query), Some(10), None, false, CaseMatching::Smart)
                });
            });
        }
        group.bench_function(BenchmarkId::new("search_typing", size), |b| {
            b.iter(|| {
                for query in TYPING {
                    black_box(index.search_impl(
                        black_box(query),
                        Some(10),
                        None,
                        false,
                        CaseMatching::Smart,
                    ));
                }
            });
        });
    }
    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("core_search");
    let data = items(1_000);
    for (name, query) in QUERIES {
        group.bench_function(name, |b| {
            b.iter_batched(
                || data.clone(),
                |data| {
                    search_impl(
                        black_box(query).to_owned(),
                        data,
                        Some(10),
                        None,
                        false,
                        CaseMatching::Smart,
                    )
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

/// Search options returning the 10 best matches with `match_mode`.
fn top_ten(match_mode: KeyMatchMode) -> SearchKeysOptions {
    SearchKeysOptions {
        max_results: Some(10),
        match_mode: Some(match_mode),
        ..SearchKeysOptions::default()
    }
}

fn bench_keyed_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("core_keyed_index");
    for size in INDEX_SIZES {
        let index = KeyedFuzzyIndexCore::new(vec![items(size), descriptions(size)], vec![2.0, 1.0])
            .expect("equal-length key columns and valid weights");
        for (name, query) in QUERIES.iter().chain(&TERM_QUERIES) {
            group.bench_function(
                BenchmarkId::new(format!("search_cross_key_{name}"), size),
                |b| {
                    b.iter(|| {
                        index.search_with_options(black_box(query), top_ten(KeyMatchMode::CrossKey))
                    });
                },
            );
        }
        for (name, query) in QUERIES {
            group.bench_function(BenchmarkId::new(format!("search_{name}"), size), |b| {
                b.iter(|| {
                    index.search(
                        black_box(query),
                        Some(10),
                        None,
                        CaseMatching::Smart,
                        false,
                        KeyScoreMode::Weighted,
                    )
                });
            });
        }
    }
    group.finish();
}

fn bench_distance_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("core_distance_many");
    let candidates = descriptions(1_000);
    let reference = "Factory for service instance";

    group.bench_function("levenshtein_many", |b| {
        b.iter(|| distance::levenshtein_many(black_box(reference), &candidates, None));
    });
    group.bench_function("damerau_levenshtein_many", |b| {
        b.iter(|| distance::damerau_levenshtein_many(black_box(reference), &candidates, None));
    });
    group.bench_function("normalized_levenshtein_many", |b| {
        b.iter(|| distance::normalized_levenshtein_many(black_box(reference), &candidates, None));
    });
    group.bench_function("indel_many", |b| {
        b.iter(|| distance::indel_many(black_box(reference), &candidates, None));
    });
    group.bench_function("jaro_winkler_many", |b| {
        b.iter(|| distance::jaro_winkler_many(black_box(reference), &candidates, None));
    });
    group.bench_function("sorensen_dice_many", |b| {
        b.iter(|| distance::sorensen_dice_many(black_box(reference), &candidates, None));
    });
    group.bench_function("token_sort_ratio_many", |b| {
        b.iter(|| distance::token_sort_ratio_many(black_box(reference), &candidates, None));
    });
    group.bench_function("token_set_ratio_many", |b| {
        b.iter(|| distance::token_set_ratio_many(black_box(reference), &candidates, None));
    });
    group.bench_function("partial_ratio_many", |b| {
        b.iter(|| distance::partial_ratio_many(black_box(reference), &candidates, None));
    });
    group.bench_function("weighted_ratio_many", |b| {
        b.iter(|| distance::weighted_ratio_many(black_box(reference), &candidates, None));
    });
    group.finish();
}

/// The standalone multi-key search (`searchKeys()`), which converts the key
/// texts on every search.
fn bench_search_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("core_search_keys");
    let size = 10_000;
    let key_texts = vec![items(size), descriptions(size)];
    let weights = [2.0, 1.0];
    for (name, query) in QUERIES {
        group.bench_function(format!("search_{name}"), |b| {
            b.iter(|| {
                search_keys_impl(
                    black_box(query),
                    &key_texts,
                    &weights,
                    Some(top_ten(KeyMatchMode::PerKey)),
                )
            });
        });
    }
    for (name, query) in QUERIES.iter().chain(&TERM_QUERIES) {
        group.bench_function(format!("search_cross_key_{name}"), |b| {
            b.iter(|| {
                search_keys_impl(
                    black_box(query),
                    &key_texts,
                    &weights,
                    Some(top_ten(KeyMatchMode::CrossKey)),
                )
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_fuzzy_index,
    bench_search,
    bench_keyed_index,
    bench_search_keys,
    bench_distance_many
);
criterion_main!(benches);
