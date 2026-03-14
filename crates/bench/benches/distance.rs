use std::hint::black_box;

use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};

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
    c.bench_function("jaro", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::jaro(a, s));
            }
        });
    });
}

fn bench_jaro_winkler(c: &mut Criterion) {
    c.bench_function("jaro_winkler", |b| {
        b.iter(|| {
            for (a, s) in PAIRS {
                black_box(strsim::jaro_winkler(a, s));
            }
        });
    });
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
    bench_sorensen_dice,
    bench_normalized_levenshtein,
);
criterion_main!(benches);
