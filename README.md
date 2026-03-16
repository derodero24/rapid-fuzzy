# rapid-fuzzy

[![CI](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml/badge.svg)](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml)
[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/derodero24/rapid-fuzzy)
[![codecov](https://codecov.io/gh/derodero24/rapid-fuzzy/branch/develop/graph/badge.svg)](https://codecov.io/gh/derodero24/rapid-fuzzy)
[![npm version](https://img.shields.io/npm/v/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![npm downloads](https://img.shields.io/npm/dm/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Node.js](https://img.shields.io/badge/node-%3E%3D20.0.0-brightgreen)](https://nodejs.org/)

Blazing-fast fuzzy search for JavaScript — powered by Rust, works everywhere.

<img src=".github/assets/demo.svg" alt="rapid-fuzzy demo — fuzzy search, query syntax, FuzzyIndex, and string distance" width="580" />

## Features

- **Fast**: Up to 7,000x faster than fuse.js with FuzzyIndex (Rust + napi-rs)
- **Universal**: Works in Node.js (native), browsers (WASM), Deno, and Bun
- **Zero JS dependencies**: Pure Rust core with napi-rs bindings
- **Type-safe**: Full TypeScript support with auto-generated type definitions
- **Drop-in**: API compatible with popular fuzzy search libraries

## Playground

An interactive demo featuring fuzzy search with match highlighting and string distance calculations.

<img src=".github/assets/playground.png" alt="rapid-fuzzy Playground — fuzzy search and string distance demo" width="580" />

```bash
git clone https://github.com/derodero24/rapid-fuzzy.git
cd rapid-fuzzy
pnpm run build:wasm
cd playground && npm install && npm run dev
# Open http://localhost:5173
```

> **Note**: The playground uses WASM and requires a local dev server with specific CORS headers (configured automatically by Vite).

## Quick Start

```typescript
import { search } from 'rapid-fuzzy';

const results = search('typscript', ['TypeScript', 'JavaScript', 'Python']);
// → [{ item: 'TypeScript', score: 0.85, index: 0 }, ...]
```

For repeated searches, use `FuzzyIndex` for up to 182x faster lookups:

```typescript
import { FuzzyIndex } from 'rapid-fuzzy';

const index = new FuzzyIndex(['TypeScript', 'JavaScript', 'Python', ...]);
index.search('typscript'); // sub-millisecond with incremental cache
```

## Installation

```bash
npm install rapid-fuzzy
# or
pnpm add rapid-fuzzy
```

### Runtime-specific notes

- **Node.js** (>=20): Uses native bindings via napi-rs for best performance.
- **Browser / Deno / Bun**: Falls back to a WASM build automatically. The WASM binary is ~607 KB raw (~200 KB gzipped).

> **Browser WASM requirement**: The WASM build uses `SharedArrayBuffer` for threading, which requires the following HTTP headers on your page:
> ```
> Cross-Origin-Opener-Policy: same-origin
> Cross-Origin-Embedder-Policy: require-corp
> ```
> Without these headers, you will see `SharedArrayBuffer is not defined`. See [MDN: SharedArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#security_requirements) for details.

> **Note**: rapid-fuzzy is pre-1.0 — the API is stable but minor versions may include additions.

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

### String Distance

```typescript
import { levenshtein, jaroWinkler, sorensenDice } from 'rapid-fuzzy';

levenshtein('kitten', 'sitting');     // 3
jaroWinkler('MARTHA', 'MARHTA');      // 0.961
sorensenDice('night', 'nacht');       // 0.25
```

### Query Syntax

Queries support extended syntax powered by the [nucleo](https://github.com/helix-editor/nucleo) pattern parser:

| Pattern | Match type | Example |
|---|---|---|
| `foo bar` | AND (order-independent) | `john smith` matches "Smith, John" |
| `!term` | Exclude | `apple !pie` excludes "apple pie" |
| `^term` | Starts with | `^app` matches "apple" but not "pineapple" |
| `term$` | Ends with | `pie$` matches "apple pie" |
| `'term` | Exact substring | `'pie` matches "pie" literally |

Diacritics are handled automatically — `cafe` matches `café`, `uber` matches `über`, and `naive` matches `naïve` with no configuration needed.

> **Note**: These patterns apply to all search functions: `search()`, `closest()`, `FuzzyIndex.search()`, `FuzzyObjectIndex.search()`, and `searchObjects()`. They do **not** apply to distance functions (`levenshtein`, `jaro`, etc.).

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

### Persistent Index

For applications that search the same dataset repeatedly (autocomplete, file finders, etc.), use `FuzzyIndex` or `FuzzyObjectIndex` to keep data on the Rust side and eliminate per-search FFI overhead:

```typescript
import { FuzzyIndex, FuzzyObjectIndex } from 'rapid-fuzzy';

// String search index — up to 182x faster than standalone search()
const index = new FuzzyIndex(['TypeScript', 'JavaScript', 'Python', ...]);

index.search('typscript', { maxResults: 5 });
index.closest('tsc');

// Mutate the index without rebuilding
index.add('Rust');
index.remove(2); // swap-remove by index

// Object search index — keeps objects on the JS side, keys on the Rust side
const userIndex = new FuzzyObjectIndex(users, {
  keys: [
    { name: 'name', weight: 2.0 },
    { name: 'email', weight: 1.0 },
  ],
});

userIndex.search('john', { maxResults: 10 });

// Free Rust-side memory when done
index.destroy();
userIndex.destroy();
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
| Repeated search on same data | `FuzzyIndex`, `FuzzyObjectIndex` | Persistent Rust-side index with incremental cache, up to 182x faster |

**Return types:**

- `levenshtein`, `damerauLevenshtein` → integer (edit count)
- `jaro`, `jaroWinkler`, `sorensenDice`, `normalizedLevenshtein` → float between 0.0 (no match) and 1.0 (identical)
- `tokenSortRatio`, `tokenSetRatio`, `partialRatio`, `weightedRatio` → float between 0.0 and 1.0
- `search` → array of `{ item, score, index, positions }` sorted by relevance (score: 0.0–1.0)

## Benchmarks

Measured on Apple M-series with Node.js v22 using [Vitest bench](https://vitest.dev/guide/features.html#benchmarking). Each benchmark processes 6 realistic string pairs of varying length and similarity.

### Distance Functions

<img src=".github/assets/bench-distance.svg" alt="Distance function performance chart" width="680" />

<details>
<summary>Raw numbers</summary>

| Function | rapid-fuzzy | fastest-levenshtein | leven | string-similarity |
|---|---:|---:|---:|---:|
| Levenshtein | 562,063 ops/s | **794,298 ops/s** | 228,688 ops/s | — |
| Normalized Levenshtein | **546,107 ops/s** | — | — | — |
| Sorensen-Dice | **147,850 ops/s** | — | — | 84,308 ops/s |
| Jaro-Winkler | **293,403 ops/s** | — | — | — |
| Damerau-Levenshtein | **116,153 ops/s** | — | — | — |

</details>

> **Note**: For single-pair Levenshtein, fastest-levenshtein is ~1.4x faster due to its optimized pure-JS implementation that avoids FFI overhead. rapid-fuzzy is **2.5x faster** than leven, and provides broader algorithm coverage plus batch / search scenarios.

### Search Performance

<img src=".github/assets/bench-search.svg" alt="Search performance chart — rapid-fuzzy vs fuse.js vs fuzzysort vs uFuzzy" width="680" />

> Both `rapid-fuzzy` columns below show the same library: standalone `search()` vs `FuzzyIndex` (indexed mode for repeated searches).

<details>
<summary>Raw numbers</summary>

| Dataset size | rapid-fuzzy | rapid-fuzzy (indexed) | fuse.js | fuzzysort | uFuzzy |
|---|---:|---:|---:|---:|---:|
| Small (20 items) | 303,982 ops/s | 405,604 ops/s | 105,568 ops/s | **2,606,394 ops/s** | 923,069 ops/s |
| Medium (1K items) | 6,787 ops/s | **80,579 ops/s** | 367 ops/s | 64,372 ops/s | 28,953 ops/s |
| Large (10K items) | 751 ops/s | **136,528 ops/s** | 19 ops/s | 26,112 ops/s | 6,393 ops/s |
| XL (50K items) | — | **31,903 ops/s** | — | 5,916 ops/s | 1,292 ops/s |

</details>

### Closest Match (Levenshtein-based)

| Dataset size | rapid-fuzzy | rapid-fuzzy (indexed) | fastest-levenshtein |
|---|---:|---:|---:|
| Medium (1K items) | 8,611 ops/s | **989,095 ops/s** | 6,797 ops/s |
| Large (10K items) | 924 ops/s | **156,014 ops/s** | 658 ops/s |

> In indexed mode (`FuzzyIndex`), rapid-fuzzy is up to **237x faster** than fastest-levenshtein for closest-match lookups.

### Why these numbers matter

- **vs fuse.js**: `FuzzyIndex` is **219x faster** on medium datasets and **6,869x faster** on large datasets. Even standalone `search()` is 18x / 40x faster.
- **Indexed mode**: `FuzzyIndex` keeps data on the Rust side with an incremental search cache, delivering sub-millisecond autocomplete. On large datasets this is **182x faster** than standalone `search()`.
- **vs fuzzysort**: `FuzzyIndex` now **outperforms fuzzysort** on medium-and-above datasets — 1.25x faster at 1K, 5.2x at 10K, and 5.4x at 50K.
- **vs uFuzzy**: `FuzzyIndex` is **2.8x faster** at medium and **21x faster** at large datasets.
- **vs fastest-levenshtein**: With `FuzzyIndex`, closest-match is **145x faster** at 1K and **237x faster** at 10K.

Run benchmarks yourself:

```bash
pnpm run bench        # JavaScript benchmarks
cargo bench           # Rust internal benchmarks
```

## Why rapid-fuzzy?

| | rapid-fuzzy | fuse.js | fastest-levenshtein | fuzzysort | uFuzzy |
|---|:---:|:---:|:---:|:---:|:---:|
| **Algorithms** | 9 (Levenshtein, Jaro, Dice, …) | Bitap | Levenshtein | Substring | Regex-based |
| **Runtime** | Rust native + WASM | Pure JS | Pure JS | Pure JS | Pure JS |
| **Object search** | ✅ weighted keys | ✅ | — | ✅ | — |
| **Persistent index** | ✅ FuzzyIndex / FuzzyObjectIndex | — | — | ✅ prepared targets | — |
| **Query syntax** | ✅ exclude, prefix, suffix, exact | ✅ extended search | — | — | partial (`-` only) |
| **Out-of-order matching** | ✅ automatic | — | — | — | ✅ with option |
| **Diacritics** | ✅ automatic | ✅ option | — | ✅ auto | ✅ `latinize()` |
| **Score threshold** | ✅ | ✅ | — | ✅ | — |
| **Match positions** | ✅ | ✅ | — | ✅ | ✅ |
| **Highlight utility** | ✅ | — | — | ✅ | ✅ |
| **Batch API** | ✅ | — | — | — | — |
| **Node.js native** | ✅ napi-rs | — | — | — | — |
| **Browser** | ✅ WASM (~200 KB gzipped) | ✅ | ✅ | ✅ | ✅ |
| **TypeScript** | ✅ full | ✅ full | ✅ | ✅ | ✅ |

## Migration Guides

Switching from another library? These guides provide API mapping tables, code examples, and performance comparisons:

- [**From string-similarity**](docs/migration/from-string-similarity.md) — Same Dice coefficient algorithm, now maintained and faster
- [**From fuse.js**](docs/migration/from-fuse-js.md) — Up to 7,000x faster fuzzy search with FuzzyIndex
- [**From leven / fastest-levenshtein**](docs/migration/from-leven.md) — Multi-algorithm upgrade with batch APIs
- [**From fuzzysort**](docs/migration/from-fuzzysort.md) — Richer matching with query syntax and 9 distance algorithms
- [**From uFuzzy**](docs/migration/from-ufuzzy.md) — Weighted object search, batch APIs, and persistent indexes

## License

MIT
