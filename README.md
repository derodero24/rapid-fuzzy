# rapid-fuzzy

[![CI](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml/badge.svg)](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/derodero24/rapid-fuzzy/branch/develop/graph/badge.svg)](https://codecov.io/gh/derodero24/rapid-fuzzy)
[![npm version](https://img.shields.io/npm/v/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![npm downloads](https://img.shields.io/npm/dm/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Node.js](https://img.shields.io/badge/node-%3E%3D20.0.0-brightgreen)](https://nodejs.org/)

Rust-powered fuzzy search and string distance for JavaScript/TypeScript.

> **Status**: Work in progress. Not yet published to npm.

## Features

- **Fast**: Up to 40x faster than fuse.js for large datasets (Rust + napi-rs)
- **Universal**: Works in Node.js (native), browsers (WASM), Deno, and Bun
- **Zero JS dependencies**: Pure Rust core with napi-rs bindings
- **Type-safe**: Full TypeScript support with auto-generated type definitions
- **Drop-in**: API compatible with popular fuzzy search libraries

## Installation

```bash
npm install rapid-fuzzy
# or
pnpm add rapid-fuzzy
```

### Runtime-specific notes

- **Node.js** (>=20): Uses native bindings via napi-rs for best performance.
- **Browser / Deno / Bun**: Falls back to a WASM build automatically.

## Usage

### String Distance

```typescript
import { levenshtein, jaroWinkler, sorensenDice } from 'rapid-fuzzy';

levenshtein('kitten', 'sitting');     // 3
jaroWinkler('MARTHA', 'MARHTA');      // 0.961
sorensenDice('night', 'nacht');       // 0.25
```

### Fuzzy Search

```typescript
import { search, closest } from 'rapid-fuzzy';

// Find matches sorted by relevance
const results = search('typscript', [
  'TypeScript',
  'JavaScript',
  'Python',
  'TypeSpec',
]);
// → [{ item: 'TypeScript', score: 89, index: 0 }, ...]

// Find the single best match
closest('tsc', ['TypeScript', 'JavaScript', 'Python']);
// → 'TypeScript'
```

## Benchmarks

Measured on Apple M-series with Node.js v22 using [Vitest bench](https://vitest.dev/guide/features.html#benchmarking). Each benchmark processes 6 realistic string pairs of varying length and similarity.

### Distance Functions

| Function | rapid-fuzzy | fastest-levenshtein | leven | string-similarity |
|---|---:|---:|---:|---:|
| Levenshtein | 67,346 ops/s | **243,026 ops/s** | 51,789 ops/s | — |
| Normalized Levenshtein | **64,592 ops/s** | — | — | — |
| Sorensen-Dice | **61,050 ops/s** | — | — | 40,241 ops/s |
| Jaro-Winkler | **198,140 ops/s** | — | — | — |
| Damerau-Levenshtein | **58,888 ops/s** | — | — | — |

> **Note**: For single-pair Levenshtein distance, fastest-levenshtein is faster due to its highly optimized pure-JS implementation that avoids FFI overhead. rapid-fuzzy provides broader algorithm coverage and excels in batch / search scenarios.

### Search Performance

| Dataset size | rapid-fuzzy | fuse.js | fuzzysort |
|---|---:|---:|---:|
| 20 items | 171,967 ops/s | 121,978 ops/s | **2,537,323 ops/s** |
| 1,000 items | 4,941 ops/s | 376 ops/s | **55,388 ops/s** |
| 10,000 items | 588 ops/s | 14 ops/s | **15,005 ops/s** |

### Closest Match (Levenshtein-based)

| Dataset size | rapid-fuzzy | fastest-levenshtein |
|---|---:|---:|
| 1,000 items | **5,912 ops/s** | 3,974 ops/s |
| 10,000 items | **387 ops/s** | 126 ops/s |

> rapid-fuzzy is up to **3x faster** than fastest-levenshtein for closest-match lookups on large datasets.

### Why these numbers matter

- **vs fuse.js**: rapid-fuzzy is **13x faster** on medium datasets and **41x faster** on large datasets for fuzzy search.
- **vs fastest-levenshtein**: rapid-fuzzy wins on closest-match (1.5–3x faster) where batch FFI overhead is amortized.
- **fuzzysort** uses a different (substring-based) matching algorithm that is extremely fast but produces different ranking results. Choose based on your matching needs.

Run benchmarks yourself:

```bash
pnpm run bench        # JavaScript benchmarks
cargo bench           # Rust internal benchmarks
```

## Choosing an Algorithm

| Use case | Recommended | Why |
|---|---|---|
| Typo detection / spell check | `levenshtein`, `damerauLevenshtein` | Counts edits; Damerau adds transposition support |
| Name / address matching | `jaroWinkler` | Prefix-weighted similarity for short strings |
| Document / text similarity | `sorensenDice` | Bigram-based; handles longer text well |
| Normalized comparison (0–1) | `normalizedLevenshtein` | Length-independent similarity score |
| Interactive fuzzy search | `search`, `closest` | Nucleo algorithm (same as Helix editor) |

**Return types:**

- `levenshtein`, `damerauLevenshtein` → integer (edit count)
- `jaro`, `jaroWinkler`, `sorensenDice`, `normalizedLevenshtein` → float between 0.0 (no match) and 1.0 (identical)
- `search` → array of `{ item, score, index }` sorted by relevance

## Why rapid-fuzzy?

| | rapid-fuzzy | fuse.js | fastest-levenshtein | fuzzysort |
|---|---|---|---|---|
| **Algorithms** | Levenshtein, Jaro-Winkler, Sorensen-Dice, Damerau-Levenshtein, fuzzy search | Bitap-based fuzzy | Levenshtein only | Substring fuzzy |
| **Runtime** | Rust (native + WASM) | Pure JS | Pure JS | Pure JS |
| **Batch API** | Yes | No | No | No |
| **Node.js native** | Yes (napi-rs) | No | No | No |
| **Browser support** | Yes (WASM) | Yes | Yes | Yes |
| **TypeScript** | Full (auto-generated) | Full | Yes | Yes |

## License

MIT
