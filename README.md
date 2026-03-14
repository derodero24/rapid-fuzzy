# rapid-fuzzy

[![CI](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml/badge.svg)](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml)
[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/derodero24/rapid-fuzzy)
[![codecov](https://codecov.io/gh/derodero24/rapid-fuzzy/branch/develop/graph/badge.svg)](https://codecov.io/gh/derodero24/rapid-fuzzy)
[![npm version](https://img.shields.io/npm/v/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![npm downloads](https://img.shields.io/npm/dm/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Node.js](https://img.shields.io/badge/node-%3E%3D20.0.0-brightgreen)](https://nodejs.org/)

Rust-powered fuzzy search and string distance for JavaScript/TypeScript.

> **Status**: Early release (v0.x). API may change between minor versions.

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

// Find matches sorted by relevance (scores normalized to 0.0-1.0)
const results = search('typscript', [
  'TypeScript',
  'JavaScript',
  'Python',
  'TypeSpec',
]);
// → [{ item: 'TypeScript', score: 0.85, index: 0, positions: [] }, ...]

// With options: filter by minimum score and limit results
search('app', items, { maxResults: 5, minScore: 0.3 });

// Backward compatible: pass a number for maxResults
search('app', items, 5);

// Get matched character positions for highlighting
const [match] = search('hlo', ['hello world'], { includePositions: true });
// → { item: 'hello world', score: 0.75, index: 0, positions: [0, 2, 4] }

// Case-sensitive matching (default: smart case)
search('Type', items, { isCaseSensitive: true });

// Find the single best match
closest('tsc', ['TypeScript', 'JavaScript', 'Python']);
// → 'TypeScript'

// With minimum score threshold (returns null if no match is good enough)
closest('xyz', items, 0.5);
// → null
```

### Object Search

Search across object properties with weighted keys — a drop-in replacement for fuse.js's `keys` option:

```typescript
import { searchObjects } from 'rapid-fuzzy';

const users = [
  { name: 'John Smith', email: 'john@example.com' },
  { name: 'Jane Doe', email: 'jane@example.com' },
  { name: 'Bob Johnson', email: 'bob@test.com' },
];

// Search across multiple keys
const results = searchObjects('john', users, {
  keys: ['name', 'email'],
});
// → [{ item: { name: 'John Smith', ... }, score: 0.95, keyScores: [0.98, 0.85], index: 0 }]

// Weighted keys — prioritize name matches over email
searchObjects('john', users, {
  keys: [
    { name: 'name', weight: 2.0 },
    { name: 'email', weight: 1.0 },
  ],
});

// Nested key paths
searchObjects('new york', items, { keys: ['address.city'] });
```

### Match Highlighting

Convert matched positions into highlighted markup for UI rendering:

```typescript
import { search, highlight, highlightRanges } from 'rapid-fuzzy';

const results = search('fzy', ['fuzzy'], { includePositions: true });
const { item, positions } = results[0];

// String markers
highlight(item, positions, '<b>', '</b>');
// → '<b>f</b>u<b>zy</b>'

// Callback (React, JSX, custom DOM)
highlight(item, positions, (matched) => `<mark>${matched}</mark>`);

// Raw ranges for custom rendering
highlightRanges(item, positions);
// → [{ start: 0, end: 1, matched: true }, { start: 1, end: 2, matched: false }, ...]
```

### Token-Based Matching

Order-independent and partial string matching, inspired by Python's [RapidFuzz](https://github.com/rapidfuzz/RapidFuzz):

```typescript
import {
  tokenSortRatio,
  tokenSetRatio,
  partialRatio,
  weightedRatio,
} from 'rapid-fuzzy';

// Token Sort: order-independent comparison
tokenSortRatio('New York Mets', 'Mets New York'); // 1.0

// Token Set: handles extra/missing tokens
tokenSetRatio('Great Gatsby', 'The Great Gatsby by Fitzgerald'); // ~0.85

// Partial: best substring match
partialRatio('hello', 'hello world'); // 1.0

// Weighted: best score across all methods
weightedRatio('John Smith', 'Smith, John'); // 1.0
```

All token-based functions include `Batch` and `Many` variants (e.g., `tokenSortRatioBatch`, `tokenSortRatioMany`).

### Batch Operations

All distance functions have `Batch` and `Many` variants that amortize FFI overhead:

```typescript
import { levenshteinBatch, levenshteinMany } from 'rapid-fuzzy';

// Compute distances for multiple pairs at once
levenshteinBatch([
  ['kitten', 'sitting'],
  ['hello', 'help'],
  ['foo', 'bar'],
]);
// → [3, 2, 3]

// Compare one string against many candidates
levenshteinMany('kitten', ['sitting', 'kittens', 'kitchen']);
// → [3, 1, 2]
```

> **Tip**: Prefer batch/many variants over calling single-pair functions in a loop — they are significantly faster for multiple comparisons.

## Benchmarks

Measured on Apple M-series with Node.js v22 using [Vitest bench](https://vitest.dev/guide/features.html#benchmarking). Each benchmark processes 6 realistic string pairs of varying length and similarity.

### Distance Functions

| Function | rapid-fuzzy | fastest-levenshtein | leven | string-similarity |
|---|---:|---:|---:|---:|
| Levenshtein | 193,593 ops/s | **774,820 ops/s** | 204,047 ops/s | — |
| Normalized Levenshtein | **136,854 ops/s** | — | — | — |
| Sorensen-Dice | **144,698 ops/s** | — | — | 84,108 ops/s |
| Jaro-Winkler | **291,673 ops/s** | — | — | — |
| Damerau-Levenshtein | **72,238 ops/s** | — | — | — |

> **Note**: For single-pair Levenshtein distance, fastest-levenshtein is faster due to its highly optimized pure-JS implementation that avoids FFI overhead. rapid-fuzzy provides broader algorithm coverage and excels in batch / search scenarios.

### Search Performance

| Dataset size | rapid-fuzzy | fuse.js | fuzzysort |
|---|---:|---:|---:|
| Small (20 items) | 179,222 ops/s | 109,059 ops/s | **2,501,773 ops/s** |
| Medium (1K items) | 6,614 ops/s | 381 ops/s | **63,032 ops/s** |
| Large (10K items) | 794 ops/s | 20 ops/s | **28,616 ops/s** |

### Closest Match (Levenshtein-based)

| Dataset size | rapid-fuzzy | fastest-levenshtein |
|---|---:|---:|
| Medium (1K items) | 8,416 ops/s | **8,762 ops/s** |
| Large (10K items) | **905 ops/s** | 662 ops/s |

> rapid-fuzzy is up to **1.4x faster** than fastest-levenshtein for closest-match lookups on large datasets.

### Why these numbers matter

- **vs fuse.js**: rapid-fuzzy is **17x faster** on medium datasets and **40x faster** on large datasets for fuzzy search.
- **vs fastest-levenshtein**: rapid-fuzzy wins on closest-match at scale where batch FFI overhead is amortized.
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
| Name / address matching | `jaroWinkler`, `tokenSortRatio` | Prefix-weighted or order-independent matching |
| Document / text similarity | `sorensenDice` | Bigram-based; handles longer text well |
| Normalized comparison (0–1) | `normalizedLevenshtein` | Length-independent similarity score |
| Reordered words / messy data | `tokenSortRatio`, `tokenSetRatio` | Handles word order differences and extra tokens |
| Substring / abbreviation matching | `partialRatio` | Finds best partial match within longer strings |
| Best-effort similarity | `weightedRatio` | Picks the best score across all methods automatically |
| Interactive fuzzy search | `search`, `closest` | Nucleo algorithm (same as Helix editor) |

**Return types:**

- `levenshtein`, `damerauLevenshtein` → integer (edit count)
- `jaro`, `jaroWinkler`, `sorensenDice`, `normalizedLevenshtein` → float between 0.0 (no match) and 1.0 (identical)
- `tokenSortRatio`, `tokenSetRatio`, `partialRatio`, `weightedRatio` → float between 0.0 and 1.0
- `search` → array of `{ item, score, index, positions }` sorted by relevance (score: 0.0–1.0)

## Why rapid-fuzzy?

| | rapid-fuzzy | fuse.js | fastest-levenshtein | fuzzysort |
|---|---|---|---|---|
| **Algorithms** | Levenshtein, Jaro-Winkler, Sorensen-Dice, Damerau-Levenshtein, token sort/set, partial ratio, fuzzy search | Bitap-based fuzzy | Levenshtein only | Substring fuzzy |
| **Runtime** | Rust (native + WASM) | Pure JS | Pure JS | Pure JS |
| **Object search** | Yes (searchObjects with weighted keys) | Yes (keys option) | No | Yes (keys) |
| **Score threshold** | Yes (minScore) | Yes (threshold) | No | Yes (threshold) |
| **Match positions** | Yes (includePositions) | Yes | No | Yes |
| **Highlight utility** | Yes (highlight, highlightRanges) | No (manual) | No | Yes (highlight) |
| **Batch API** | Yes | No | No | No |
| **Node.js native** | Yes (napi-rs) | No | No | No |
| **Browser support** | Yes (WASM) | Yes | Yes | Yes |
| **TypeScript** | Full (auto-generated) | Full | Yes | Yes |

## Migration Guides

Switching from another library? These guides provide API mapping tables, code examples, and performance comparisons:

- [**From string-similarity**](docs/migration/from-string-similarity.md) — Same Dice coefficient algorithm, now maintained and faster
- [**From fuse.js**](docs/migration/from-fuse-js.md) — 17–40x faster fuzzy search with a simpler API
- [**From leven / fastest-levenshtein**](docs/migration/from-leven.md) — Multi-algorithm upgrade with batch APIs

## License

MIT
