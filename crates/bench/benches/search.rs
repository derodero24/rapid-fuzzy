use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

fn generate_items(n: usize) -> Vec<String> {
    let words = [
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
    (0..n)
        .map(|i| format!("{}_{}", words[i % words.len()], i))
        .collect()
}

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuzzy_search");

    for size in [100, 1_000, 10_000] {
        let items = generate_items(size);

        group.bench_with_input(BenchmarkId::new("nucleo", size), &items, |b, items| {
            b.iter(|| {
                let mut matcher = Matcher::new(Config::DEFAULT);
                let pattern = Pattern::parse(
                    "handler middleware",
                    CaseMatching::Smart,
                    Normalization::Smart,
                );

                let mut results: Vec<(u32, usize)> = items
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, item)| {
                        let mut buf = Vec::new();
                        let atoms = Utf32Str::new(item, &mut buf);
                        pattern.score(atoms, &mut matcher).map(|s| (s, idx))
                    })
                    .collect();

                results.sort_by(|a, b| b.0.cmp(&a.0));
                results.truncate(10);
                black_box(results);
            });
        });
    }

    group.finish();
}

fn bench_closest(c: &mut Criterion) {
    let mut group = c.benchmark_group("closest_match");

    for size in [100, 1_000, 10_000] {
        let items = generate_items(size);

        group.bench_with_input(BenchmarkId::new("nucleo", size), &items, |b, items| {
            b.iter(|| {
                let mut matcher = Matcher::new(Config::DEFAULT);
                let pattern =
                    Pattern::parse("handler_500", CaseMatching::Smart, Normalization::Smart);

                let best = items
                    .iter()
                    .filter_map(|item| {
                        let mut buf = Vec::new();
                        let atoms = Utf32Str::new(item, &mut buf);
                        pattern.score(atoms, &mut matcher).map(|s| (s, item))
                    })
                    .max_by_key(|(s, _)| *s);

                black_box(best);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_search, bench_closest);
criterion_main!(benches);
