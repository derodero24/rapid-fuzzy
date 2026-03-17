use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

const PAIRS: &[(&str, &str)] = &[
    ("kitten", "sitting"),
    ("saturday", "sunday"),
    ("rosettacode", "raisethysword"),
    (
        "pneumonoultramicroscopicsilicovolcanoconiosis",
        "ultramicroscopically",
    ),
    (
        "the quick brown fox jumps over the lazy dog",
        "the fast brown fox leaps over the lazy dog",
    ),
    ("abcdefghijklmnopqrstuvwxyz", "zyxwvutsrqponmlkjihgfedcba"),
];

fn bench_levenshtein(c: &mut Criterion) {
    c.bench_function("levenshtein", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::levenshtein(a, s));
            }
        });
    });
}

fn bench_damerau_levenshtein(c: &mut Criterion) {
    c.bench_function("damerau_levenshtein", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::damerau_levenshtein(a, s));
            }
        });
    });
}

fn bench_jaro(c: &mut Criterion) {
    let mut group = c.benchmark_group("jaro");
    group.bench_function("strsim", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::jaro(a, s));
            }
        });
    });
    group.bench_function("rapidfuzz", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(rapidfuzz::distance::jaro::similarity(a.chars(), s.chars()));
            }
        });
    });
    group.finish();
}

fn bench_jaro_winkler(c: &mut Criterion) {
    let mut group = c.benchmark_group("jaro_winkler");
    group.bench_function("strsim", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::jaro_winkler(a, s));
            }
        });
    });
    group.bench_function("rapidfuzz", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(rapidfuzz::distance::jaro_winkler::similarity(
                    a.chars(),
                    s.chars(),
                ));
            }
        });
    });
    group.finish();
}

fn bench_jaro_many(c: &mut Criterion) {
    // Generate 1000 candidate strings for 1:N comparison
    let candidates: Vec<String> = (0..1000)
        .map(|i| format!("candidate_string_number_{}", i))
        .collect();
    let reference = "candidate_string";

    let mut group = c.benchmark_group("jaro_many_1k");
    group.bench_function("strsim_loop", |b| {
        b.iter(|| {
            let results: Vec<f64> = candidates
                .iter()
                .map(|c| strsim::jaro(reference, c))
                .collect();
            black_box(results);
        });
    });
    group.bench_function("rapidfuzz_batch_comparator", |b| {
        b.iter(|| {
            let scorer = rapidfuzz::distance::jaro::BatchComparator::new(reference.chars());
            let results: Vec<f64> = candidates
                .iter()
                .map(|c| scorer.similarity(c.chars()))
                .collect();
            black_box(results);
        });
    });
    group.finish();
}

fn bench_jaro_winkler_many(c: &mut Criterion) {
    let candidates: Vec<String> = (0..1000)
        .map(|i| format!("candidate_string_number_{}", i))
        .collect();
    let reference = "candidate_string";

    let mut group = c.benchmark_group("jaro_winkler_many_1k");
    group.bench_function("strsim_loop", |b| {
        b.iter(|| {
            let results: Vec<f64> = candidates
                .iter()
                .map(|c| strsim::jaro_winkler(reference, c))
                .collect();
            black_box(results);
        });
    });
    group.bench_function("rapidfuzz_batch_comparator", |b| {
        b.iter(|| {
            let scorer = rapidfuzz::distance::jaro_winkler::BatchComparator::new(reference.chars());
            let results: Vec<f64> = candidates
                .iter()
                .map(|c| scorer.similarity(c.chars()))
                .collect();
            black_box(results);
        });
    });
    group.finish();
}

fn bench_sorensen_dice(c: &mut Criterion) {
    c.bench_function("sorensen_dice", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::sorensen_dice(a, s));
            }
        });
    });
}

fn bench_normalized_levenshtein(c: &mut Criterion) {
    c.bench_function("normalized_levenshtein", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::normalized_levenshtein(a, s));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_levenshtein,
    bench_damerau_levenshtein,
    bench_jaro,
    bench_jaro_winkler,
    bench_jaro_many,
    bench_jaro_winkler_many,
    bench_sorensen_dice,
    bench_normalized_levenshtein,
);
criterion_main!(benches);
