---
title: Why rapid-fuzzy?
description: How rapid-fuzzy compares with fuse.js, fuzzysort, uFuzzy, match-sorter, MiniSearch, fastest-levenshtein and fuzzball, and when to choose another library.
---

rapid-fuzzy brings two things together that JavaScript usually gets from separate packages: fzf-style fuzzy search over lists, and edit-distance and similarity metrics from the RapidFuzz family. Both run in a Rust core, natively in Node.js and as WebAssembly everywhere else.

## At a glance

| | rapid-fuzzy | fuse.js | fuzzysort | uFuzzy | match-sorter |
| --- | :---: | :---: | :---: | :---: | :---: |
| Matching | Subsequence (nucleo, like fzf) | Bitap: tolerates wrong and swapped letters | Subsequence | Regex-based | Ranking tiers (exact, prefix, contains, acronym, …) |
| Engine | Rust: native addon, WebAssembly | JavaScript | JavaScript | JavaScript | JavaScript |
| Persistent index | `FuzzyIndex`, `FuzzyObjectIndex`, with incremental type-ahead | `Fuse.createIndex` | Prepared targets | — | — |
| Weighted object keys | ✅ with `scoreMode`, `matchMode` | ✅ | ✅ keys | — | ✅ keys |
| Query syntax | `^` `$` `'` `!`, terms in any order | Extended search, token search | — | `-exclude`, `"exact"` | — |
| Multi-threaded | Native addon, automatically | `FuseWorker` (Web Workers) | — | — | — |
| Highlighting helper | ✅ | — (match indices) | ✅ | ✅ | — |
| String distance metrics | 7, plus 4 token ratios, with batch APIs | — | — | — | — |
| Browser download (gzip) | ~195 KB (WebAssembly + glue) | ~9 KB | ~8 KB | ~4 KB | ~3 KB |
| Weekly npm downloads (Oct 2026) | new | ~8.7M | ~8.8M | ~0.3M | ~2.1M |

Bundle sizes are from [Bundlephobia](https://bundlephobia.com/) for the JavaScript libraries, and from the published package for rapid-fuzzy.

## Speed

On lists of 1,000 items or more, a `FuzzyIndex` answered queries 150–200x faster than fuse.js, 3–4.5x faster than uFuzzy and 1.6–5x faster than fuzzysort with prepared targets in our [benchmarks](/rapid-fuzzy/benchmarks/). fuzzysort and uFuzzy are faster on small lists of a few dozen items, where crossing into Rust costs more than the search itself. For single Levenshtein calls on short strings, the pure-JavaScript fastest-levenshtein is about 1.2x faster; rapid-fuzzy's `*Many` functions and its other metrics are faster than the JavaScript alternatives.

## When to choose another library

rapid-fuzzy is not the best tool for every job:

- **The smallest possible bundle** for a short list in the browser: fuzzysort, uFuzzy, match-sorter or `@nozbe/microfuzz` are a few kilobytes, while rapid-fuzzy's WebAssembly build is about 195 KB gzipped.
- **Typo-tolerant search**, where `tpyescript` should find `TypeScript`: fuse.js's Bitap matching and its token search tolerate wrong and swapped letters. rapid-fuzzy's search only tolerates *missing* letters; you can combine it with a distance function (see [How matching works](/rapid-fuzzy/guides/matching-and-scoring/#what-matches)).
- **Full-text search over documents**, with stemming, BM25 relevance and prefix search: use a search engine such as MiniSearch, FlexSearch or Orama. rapid-fuzzy matches characters in short strings, such as names, titles, commands and file paths.
- **Simple, predictable ranking tiers** for UI lists (exact match, then starts-with, then contains): match-sorter.

## The landscape in 2026

These are the main packages people use for fuzzy matching in JavaScript today, and how they relate to rapid-fuzzy:

- **fuse.js** (7.5) is the most widely known fuzzy search library. Version 7.5 added token search, which fuzzy-matches each word and ranks with BM25-style weights, and `FuseWorker`, which spreads a search over Web Workers. It is slow on large lists, about 27 queries per second on 10,000 items in our benchmark.
- **fuzzysort** (4.0) is a small, fast subsequence matcher with good ranking. It is the closest to rapid-fuzzy's search in behavior, and the fastest choice on small lists.
- **uFuzzy** builds regular expressions from the query. It is tiny and fast, but it finds nothing for some queries that subsequence matchers handle, such as `aple` for `apple`.
- **match-sorter** sorts by ranking tiers rather than fuzzy scores, and is popular in React tables.
- **fzf** (fzf-for-js) ports fzf's algorithm to JavaScript; it has not been released since 2023.
- **MiniSearch, FlexSearch and Orama** are full-text search engines with tokenization and inverted indexes, for documents rather than short strings.
- **Levenshtein packages**: fast-levenshtein, leven, fastest-levenshtein, js-levenshtein and damerau-levenshtein together have well over 100 million weekly downloads, but each implements a single metric. string-similarity (Dice coefficient) is no longer maintained.
- **fuzzball** ports Python's fuzzywuzzy and RapidFuzz ratios to JavaScript; rapid-fuzzy's token ratios are 3–4x faster.
- **Native and WebAssembly matchers** are appearing: `frizbee` (a SIMD Smith-Waterman matcher with typo tolerance, as WebAssembly), `@ff-labs/fff-node` (a file finder for agents) and `@stll/fuzzy-search` (approximate substring matching). Each focuses on one task.

## Coming from another library?

The migration guides map each library's API to rapid-fuzzy, and call out where results differ:

[fuse.js](/rapid-fuzzy/migration/from-fuse-js/) · [fuzzysort](/rapid-fuzzy/migration/from-fuzzysort/) · [uFuzzy](/rapid-fuzzy/migration/from-ufuzzy/) · [fuzzball](/rapid-fuzzy/migration/from-fuzzball/) · [leven and fastest-levenshtein](/rapid-fuzzy/migration/from-leven/) · [string-similarity](/rapid-fuzzy/migration/from-string-similarity/) · [FlexSearch](/rapid-fuzzy/migration/from-flexsearch/) · [MiniSearch](/rapid-fuzzy/migration/from-minisearch/)
