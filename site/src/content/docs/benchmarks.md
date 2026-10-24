---
title: Benchmarks
description: rapid-fuzzy compared with fuse.js, fuzzysort, uFuzzy, fastest-levenshtein, leven, string-similarity and fuzzball on the same data and queries.
---

These benchmarks compare rapid-fuzzy with popular JavaScript libraries on the same data and queries. Run them on your own hardware with `pnpm bench:readme` in a clone of the repository; your data and queries matter more than these numbers. To try your own data in the browser, open the [Playground](/rapid-fuzzy/playground/), which times rapid-fuzzy, fuse.js and fuzzysort side by side.

<!-- bench:env:start -->
Measured on 2026-10-04 with Node.js v22.22.0 on linux x64 (Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical CPUs, a shared cloud VM), using the release build of the native addon and [Vitest bench](https://vitest.dev/guide/benchmarking). Numbers vary by up to about ±10% between runs; treat them as relative, not absolute.
<!-- bench:env:end -->

How these numbers were produced (sources: [`__test__/search.compare.bench.ts`](https://github.com/derodero24/rapid-fuzzy/blob/develop/__test__/search.compare.bench.ts), [`__test__/distance.compare.bench.ts`](https://github.com/derodero24/rapid-fuzzy/blob/develop/__test__/distance.compare.bench.ts) and the datasets in [`__test__/bench-fixtures.ts`](https://github.com/derodero24/rapid-fuzzy/blob/develop/__test__/bench-fixtures.ts); `pnpm bench:readme` re-runs them and rewrites these tables and the ones in the README):

- **Datasets**: 20 fruit names; 1,000 file paths such as `utils/config4.tsx`; 10,000 to 100,000 generated identifiers such as `handler_repository_14`.
- **Queries**: `aple` (20 items), `utils config` (1K), `handler repo` (10K-100K, matches 5% of the items). Every query matches items: a query that matches nothing measures an early exit, not a search. *Rotating* cycles through 8 different queries (`ctrl`, `prms`, `service fac`, …) so no search can reuse the previous one; *type-ahead* is one operation of 12 searches, one per keystroke of `handler repo`.
- **Settings**: each library returns its top 5 (20 items) or 10 results; fuse.js uses `threshold: 0.4`; fuzzysort uses `threshold: 0` and, from 1K items on, prepared targets; uFuzzy uses its default options.
- **The libraries do not find the same matches**, so the work they do differs: uFuzzy finds nothing for `aple`, and fuse.js and uFuzzy find nothing for abbreviations such as `ctrl` in the rotating set. FlexSearch and MiniSearch are token-based (FlexSearch finds nothing for most of these queries) and are left out.
- **Threads**: on large inputs, the native addon spreads scoring over a thread pool (one thread per CPU, or `RAYON_NUM_THREADS`); the JavaScript libraries use one thread. With `RAYON_NUM_THREADS=1`, `FuzzyIndex` measured 1,004 ops/s on 50K items and 484 ops/s on 100K. The WASM build is single-threaded.

## Search Performance

<img src="/rapid-fuzzy/bench-search.svg" alt="Search performance chart — rapid-fuzzy vs fuse.js vs fuzzysort vs uFuzzy" width="680" />

> Both `rapid-fuzzy` columns show the same library: standalone `search()` (converts the items on every call) and a prebuilt `FuzzyIndex`.

<!-- bench:search:start -->
| Dataset | rapid-fuzzy | rapid-fuzzy (indexed) | fuse.js | fuzzysort | uFuzzy |
|---|---:|---:|---:|---:|---:|
| Small (20 items) | 157,995 ops/s | 208,250 ops/s | 73,144 ops/s | **1,271,223 ops/s** | 383,890 ops/s |
| Medium (1K items) | 3,502 ops/s | 50,318 ops/s | 249 ops/s | **54,110 ops/s** | 16,858 ops/s |
| Large (10K items) | 508 ops/s | **4,803 ops/s** | 27 ops/s | 2,977 ops/s | 1,404 ops/s |
| Large (10K items, rotating queries) | 451 ops/s | **4,644 ops/s** | 32 ops/s | 2,231 ops/s | 1,037 ops/s |
| Large (10K items, type-ahead, 12 searches) | 42 ops/s | **555 ops/s** | 4 ops/s | 271 ops/s | 62 ops/s |
| Extra large (50K items) | — | **2,120 ops/s** | — | 631 ops/s | 755 ops/s |
| Huge (100K items) | — | **1,125 ops/s** | — | 231 ops/s | 374 ops/s |
<!-- bench:search:end -->

## Closest Match

`closest()` returns the best fuzzy-search match, while fastest-levenshtein's `closest()` returns the item with the smallest edit distance, so the two can return different items (rapid-fuzzy returns `null` when no item contains the query's characters in order). The queries below (`utils/index42` and `handler_repositry_514`, one letter missing) make both return the same item. Both queries contain digits, which few items share, so the index's character prefilter skips most items; less selective queries run at about the speed of the search table above.

<!-- bench:closest:start -->
| Dataset | rapid-fuzzy | rapid-fuzzy (indexed) | fastest-levenshtein |
|---|---:|---:|---:|
| Medium (1K items) | 4,773 ops/s | **583,626 ops/s** | 4,229 ops/s |
| Large (10K items) | 478 ops/s | **55,603 ops/s** | 419 ops/s |
<!-- bench:closest:end -->

## Distance Functions

Each operation computes 6 string pairs of varying length and similarity (6 to 45 characters).

<img src="/rapid-fuzzy/bench-distance.svg" alt="Distance function performance chart" width="680" />

<!-- bench:distance:start -->
| Function | rapid-fuzzy | fastest-levenshtein | leven | string-similarity |
|---|---:|---:|---:|---:|
| Levenshtein | 333,490 ops/s | **400,349 ops/s** | 131,383 ops/s | — |
| Normalized Levenshtein | 325,462 ops/s | — | — | — |
| Sorensen-Dice | **178,705 ops/s** | — | — | 56,320 ops/s |
| Jaro-Winkler | 299,960 ops/s | — | — | — |
| Damerau-Levenshtein | 87,724 ops/s | — | — | — |
| Hamming | 477,800 ops/s | — | — | — |
<!-- bench:distance:end -->

<!-- bench:ratio:start -->
| Function | rapid-fuzzy | fuzzball |
|---|---:|---:|
| Token sort ratio | **209,369 ops/s** | 51,588 ops/s |
| Token set ratio | **115,354 ops/s** | 36,665 ops/s |
| Weighted ratio (`weightedRatio` / `WRatio`) | **50,289 ops/s** | 11,668 ops/s |
| Levenshtein, 1 vs 1,000 candidates | 3,752 ops/s (`levenshteinMany`) | — |
| ↳ same, one call per candidate | 2,423 ops/s (`levenshtein`) | fastest-levenshtein: 10,458 ops/s |
<!-- bench:ratio:end -->

The token-based ratios do not return the same scores as fuzzball's (see the [fuzzball migration guide](/rapid-fuzzy/migration/from-fuzzball/)).

## Key takeaways

- **Indexed search**: a `FuzzyIndex` was 10-14x faster per query than standalone `search()` on 1K-10K items, because `search()` converts every item on every call.
- **vs fuse.js**: `FuzzyIndex` was about 150-200x faster on 1K-10K items, and standalone `search()` about 14-19x.
- **vs fuzzysort**: fuzzysort was faster on 20 items (about 6x) and about as fast on 1K items. On 10K items `FuzzyIndex` was 1.6-2.1x faster, and on 50K-100K items 3-5x faster using 4 threads (1.6-2.1x on one thread).
- **vs uFuzzy**: uFuzzy was faster on 20 items; `FuzzyIndex` was about 3-4.5x faster on 1K-10K items.
- **Distance functions**: for single pairs, fastest-levenshtein (pure JS) was about 1.2x faster than `levenshtein`, and about 2.8x faster than `levenshteinMany` over 1,000 short candidates, since each call crosses the JS/Rust boundary. rapid-fuzzy was about 2.5x faster than leven, 3x faster than string-similarity and 3-4x faster than fuzzball's ratios.
