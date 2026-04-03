# rapid-fuzzy

[![CI](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml/badge.svg)](https://github.com/derodero24/rapid-fuzzy/actions/workflows/ci.yml)
[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/derodero24/rapid-fuzzy)
[![codecov](https://codecov.io/gh/derodero24/rapid-fuzzy/branch/develop/graph/badge.svg)](https://codecov.io/gh/derodero24/rapid-fuzzy)
[![npm version](https://img.shields.io/npm/v/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![npm downloads](https://img.shields.io/npm/dm/rapid-fuzzy)](https://www.npmjs.com/package/rapid-fuzzy)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Node.js](https://img.shields.io/badge/node-%3E%3D20.0.0-brightgreen)](https://nodejs.org/)

Blazing-fast fuzzy search for JavaScript — powered by Rust, works everywhere.

## Features

- **Fast**: Up to 15,000x faster than fuse.js in indexed mode on large datasets (Rust + napi-rs) — [see benchmarks](#benchmarks)
- **Universal**: Works in Node.js (native), browsers (WASM), Deno, and Bun
- **Zero JS dependencies**: Pure Rust core with napi-rs bindings
- **Type-safe**: Full TypeScript support with auto-generated type definitions
- **Drop-in**: API compatible with popular fuzzy search libraries

## Playground

Try rapid-fuzzy in the browser — no installation required: **[Open Playground](https://derodero24.github.io/rapid-fuzzy/)**

## Quick Start

```typescript
import { search } from 'rapid-fuzzy';

const results = search('typscript', ['TypeScript', 'JavaScript', 'Python']);
// → [{ item: 'TypeScript', score: 0.85, index: 0 }, ...]
```

For repeated searches, use `FuzzyIndex` for up to 297x faster lookups:

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
- **Bun**: Uses native napi-rs bindings. WASM fallback also works — see [Bun section](#bun) below.
- **Browser / CDN / Cloudflare Workers / Deno**: Falls back to the wasm-bindgen WASM build (~195 KB raw). No `SharedArrayBuffer` or COOP/COEP headers required.

### Framework Integration (SSR)

Native modules need to be externalized in SSR frameworks:

**Next.js**

```js
// next.config.js
const nextConfig = {
  serverExternalPackages: ['rapid-fuzzy'],
};
```

**Vite SSR**

```js
// vite.config.js
export default {
  ssr: {
    external: ['rapid-fuzzy'],
  },
};
```

On the client side, rapid-fuzzy automatically falls back to WASM — no additional configuration needed.

### Browser and Edge Runtime Usage

#### CDN (no bundler required)

Import directly from [esm.sh](https://esm.sh) in any `<script type="module">`:

```html
<script type="module">
  import { search, FuzzyIndex } from 'https://esm.sh/rapid-fuzzy';

  const results = search('typscript', ['TypeScript', 'JavaScript', 'Python']);
  console.log(results[0].item); // 'TypeScript'

  const index = new FuzzyIndex(['TypeScript', 'JavaScript', 'Python']);
  console.log(index.search('typscript')[0].item); // 'TypeScript'
  index.destroy();
</script>
```

See [`examples/cdn-usage/`](examples/cdn-usage/) for a complete HTML example.

#### Cloudflare Workers

Install the package and import it in your Worker script. Wrangler bundles the WASM binary automatically:

```bash
npm install rapid-fuzzy
```

```js
// worker.js
import { search, FuzzyIndex } from 'rapid-fuzzy';

// Create the index once at module scope (shared across requests)
const index = new FuzzyIndex(['hello', 'world', 'foo', 'bar']);

export default {
  fetch(request) {
    const { searchParams } = new URL(request.url);
    const query = searchParams.get('q') ?? '';
    const results = index.search(query);
    return Response.json(results);
  },
};
```

```toml
# wrangler.toml
name = "rapid-fuzzy-worker"
main = "worker.js"
compatibility_date = "2025-01-01"
```

See [`examples/cloudflare-workers/`](examples/cloudflare-workers/) for a complete example.

#### Deno

Use rapid-fuzzy via the `npm:` specifier (Deno 1.28+):

```ts
import { search } from 'npm:rapid-fuzzy';

const results = search('typscript', ['TypeScript', 'JavaScript', 'Python']);
console.log(results[0].item); // 'TypeScript'
```

Or import from a CDN directly:

```ts
import { search } from 'https://esm.sh/rapid-fuzzy';
```

#### Bun

Bun uses the native napi-rs bindings when available (fastest). You can also use the wasm-bindgen WASM files directly if needed — see [`e2e/wasm-bun.test.ts`](e2e/wasm-bun.test.ts) for the manual initialization pattern.

## API

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

// Get matched character positions for highlighting
const [match] = search('hlo', ['hello world'], { includePositions: true });
// → { item: 'hello world', score: 0.75, index: 0, positions: [0, 2, 4] }

// Case-sensitive matching (default: smart case)
search('Type', items, { isCaseSensitive: true });

// Return all items when query is empty (useful for filter-as-you-type UIs)
search('', items, { returnAllOnEmpty: true });

// Find the single best match
closest('tsc', ['TypeScript', 'JavaScript', 'Python']);
// → 'TypeScript'

// With minimum score threshold (returns null if no match is good enough)
closest('xyz', items, 0.5);
// → null
```

### String Distance

```typescript
import {
  levenshtein,
  jaro,
  jaroWinkler,
  sorensenDice,
  hamming,
  normalizedHamming,
  indel,
  normalizedIndel,
} from 'rapid-fuzzy';

levenshtein('kitten', 'sitting');     // 3
jaro('martha', 'marhta');             // 0.944 (similarity between 0–1)
jaroWinkler('MARTHA', 'MARHTA');      // 0.961 (jaro + prefix bonus)
sorensenDice('night', 'nacht');       // 0.25
hamming('karolin', 'kathrin');        // 3 (null if lengths differ)
normalizedHamming('karolin', 'kathrin'); // 0.571 (similarity 0–1, null if lengths differ)
indel('abc', 'ac');                   // 1 (insertions + deletions only, no substitutions)
normalizedIndel('kitten', 'sitting'); // 0.615 (similarity 0–1)
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

// String search index — up to 297x faster than standalone search()
const index = new FuzzyIndex(['TypeScript', 'JavaScript', 'Python', ...]);

index.search('typscript', { maxResults: 5 });
index.closest('tsc');

// Tip: FuzzyIndex caches results internally — extending a previous query
// (e.g. typing "app" → "apple") reuses cached candidates for faster lookups.

// Index-only results (no string cloning — less GC pressure)
const hits = index.searchIndices('typscript', { maxResults: 5 });
// → [{ index: 0, score: 0.85, positions: [] }, ...]

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

#### Incremental Search (Autocomplete)

FuzzyIndex automatically caches matching candidates. When a new query extends the previous one, only cached candidates are re-scored:

```typescript
const index = new FuzzyIndex(items);
index.search('app');    // scores all items, caches matches
index.search('apple');  // only re-scores cached candidates — much faster
index.search('xyz');    // different query — full scan, new cache
```

This makes FuzzyIndex ideal for search-as-you-type UIs where each keystroke extends the query.

#### Index Serialization

Save and restore a `FuzzyIndex` to avoid rebuilding on startup:

```typescript
import { FuzzyIndex } from 'rapid-fuzzy';

const index = new FuzzyIndex(['apple', 'banana', 'cherry']);

// Serialize to Buffer
const data = index.serialize();

// Restore from serialized data
const restored = FuzzyIndex.deserialize(data);
restored.search('aple'); // works immediately
```

> **Note:** The serialization format is version-specific. Regenerate the index after updating rapid-fuzzy.

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

<details>
<summary><strong>Token-Based Matching</strong></summary>

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

</details>

<details>
<summary><strong>Batch Operations</strong></summary>

All distance functions have `Batch` and `Many` variants that amortize FFI overhead. `*Batch` functions compute metrics for multiple pairs at once, while `*Many` functions compare a single reference string against multiple candidates.

- **`*Batch`** — compute distances for an array of string pairs (many-to-many)
- **`*Many`** — compare one reference string against many candidates (one-to-many)

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

// With early-termination threshold (skip candidates that can't match)
levenshteinMany('kitten', candidates, 3);        // maxDistance → returns 4 for exceeding
jaroWinklerMany('MARTHA', candidates, 0.8);       // minSimilarity → returns 0.0 for below
```

> **Tip**: Prefer batch/many variants over calling single-pair functions in a loop — they are significantly faster for multiple comparisons.

</details>

<details>
<summary><strong>TypedArray Variants</strong></summary>

All `*Many` functions have TypedArray counterparts that return `Uint32Array` or `Float64Array` instead of `Array<number>`. These avoid boxing overhead and GC pressure when processing large candidate sets.

- **`*ManyU32`** — returns `Uint32Array` (for integer distances: `levenshteinManyU32`, `damerauLevenshteinManyU32`, `indelManyU32`)
- **`*ManyF64`** — returns `Float64Array` (for similarity scores: `jaroManyF64`, `jaroWinklerManyF64`, `normalizedLevenshteinManyF64`, `normalizedIndelManyF64`, `sorensenDiceManyF64`, `tokenSortRatioManyF64`, `tokenSetRatioManyF64`, `partialRatioManyF64`, `weightedRatioManyF64`)

```typescript
import { levenshteinManyU32, jaroWinklerManyF64 } from 'rapid-fuzzy';

const candidates = ['sitting', 'kittens', 'kitchen'];

// Returns Uint32Array instead of Array<number>
levenshteinManyU32('kitten', candidates);       // Uint32Array [3, 1, 2]

// Returns Float64Array instead of Array<number>
jaroWinklerManyF64('kitten', candidates);       // Float64Array [0.746, 0.976, 0.933]
```

> **When to use**: Prefer TypedArray variants when comparing against thousands of candidates. The returned typed arrays can also be passed directly to WebGL, WASM, or worker threads without copying.

</details>

## Framework Integration

`FuzzyIndex` and `FuzzyObjectIndex` are designed for repeated search on the same data — build the index once, search many times. The examples below show the recommended pattern for each major framework.

### React

```tsx
import { useEffect, useRef, useState } from 'react';
import { FuzzyObjectIndex } from 'rapid-fuzzy/objects';

const users = [
  { name: 'Alice', email: 'alice@example.com' },
  { name: 'Bob', email: 'bob@example.com' },
];

function UserSearch() {
  const indexRef = useRef<FuzzyObjectIndex<typeof users[number]> | null>(null);
  const [query, setQuery] = useState('');
  const [results, setResults] = useState(users);

  useEffect(() => {
    indexRef.current = new FuzzyObjectIndex(users, {
      keys: [{ name: 'name', weight: 2.0 }, 'email'],
    });
    return () => indexRef.current?.destroy();
  }, []); // rebuild only when data changes — pass `users` as dependency if dynamic

  useEffect(() => {
    if (!indexRef.current) return;
    if (!query) { setResults(users); return; }
    setResults(indexRef.current.search(query).map((r) => r.item));
  }, [query]);

  return (
    <>
      <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search…" />
      <ul>{results.map((u) => <li key={u.email}>{u.name}</li>)}</ul>
    </>
  );
}
```

### Vue

```vue
<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue';
import { FuzzyObjectIndex } from 'rapid-fuzzy/objects';

const users = [
  { name: 'Alice', email: 'alice@example.com' },
  { name: 'Bob', email: 'bob@example.com' },
];

const query = ref('');
const results = ref(users);

const index = new FuzzyObjectIndex(users, {
  keys: [{ name: 'name', weight: 2.0 }, 'email'],
});

watch(query, (q) => {
  results.value = q ? index.search(q).map((r) => r.item) : users;
});

onUnmounted(() => index.destroy());
</script>

<template>
  <input v-model="query" placeholder="Search…" />
  <ul><li v-for="u in results" :key="u.email">{{ u.name }}</li></ul>
</template>
```

### Svelte

```svelte
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { FuzzyObjectIndex } from 'rapid-fuzzy/objects';

  const users = [
    { name: 'Alice', email: 'alice@example.com' },
    { name: 'Bob', email: 'bob@example.com' },
  ];

  let query = '';

  const index = new FuzzyObjectIndex(users, {
    keys: [{ name: 'name', weight: 2.0 }, 'email'],
  });

  $: results = query ? index.search(query).map((r) => r.item) : users;

  onDestroy(() => index.destroy());
</script>

<input bind:value={query} placeholder="Search…" />
<ul>{#each results as u}<li>{u.name}</li>{/each}</ul>
```

> **Note**: Always call `index.destroy()` in your cleanup handler (`useEffect` return, `onUnmounted`, `onDestroy`) to free Rust-side memory.

## Choosing an Algorithm

| Use case | Recommended | Why |
|---|---|---|
| Typo detection / spell check | `levenshtein`, `damerauLevenshtein` | Counts edits; Damerau adds transposition support |
| Insertion/deletion only edits | `indel`, `normalizedIndel` | No substitutions — useful for diff-like or DNA alignment scenarios |
| Fixed-length comparison | `hamming`, `normalizedHamming` | Counts differing positions; only for equal-length strings |
| Name / address matching | `jaroWinkler`, `tokenSortRatio` | Prefix-weighted or order-independent matching |
| Character-level similarity | `jaro` | Good baseline similarity without prefix weighting |
| Document / text similarity | `sorensenDice` | Bigram-based; handles longer text well |
| Normalized comparison (0–1) | `normalizedLevenshtein` | Length-independent similarity score |
| Reordered words / messy data | `tokenSortRatio`, `tokenSetRatio` | Handles word order differences and extra tokens |
| Substring / abbreviation matching | `partialRatio` | Finds best partial match within longer strings |
| Best-effort similarity | `weightedRatio` | Picks the best score across all methods automatically |
| Interactive fuzzy search | `search`, `closest` | Nucleo algorithm (same as Helix editor) |
| Repeated search on same data | `FuzzyIndex`, `FuzzyObjectIndex` | Persistent Rust-side index with incremental cache, up to 297x faster |

**Return types:**

- `levenshtein`, `damerauLevenshtein`, `hamming`, `indel` → integer (edit/difference count; `hamming` returns `null` if lengths differ)
- `jaro`, `jaroWinkler`, `sorensenDice`, `normalizedLevenshtein`, `normalizedIndel` → float between 0.0 (no match) and 1.0 (identical)
- `normalizedHamming` → float between 0.0 and 1.0 (`null` if lengths differ)
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
| Levenshtein | 545,338 ops/s | **741,195 ops/s** | 225,457 ops/s | — |
| Normalized Levenshtein | **514,446 ops/s** | — | — | — |
| Sorensen-Dice | **142,180 ops/s** | — | — | 56,729 ops/s |
| Jaro-Winkler | **505,762 ops/s** | — | — | — |
| Damerau-Levenshtein | **116,186 ops/s** | — | — | — |
| Hamming | **883,614 ops/s** | — | — | — |

</details>

> **Note**: For single-pair Levenshtein, fastest-levenshtein is ~1.4x faster due to its optimized pure-JS implementation that avoids FFI overhead. rapid-fuzzy is **2.4x faster** than leven, and provides broader algorithm coverage plus batch / search scenarios.

### Search Performance

<img src=".github/assets/bench-search.svg" alt="Search performance chart — rapid-fuzzy vs fuse.js vs fuzzysort vs uFuzzy" width="680" />

> Both `rapid-fuzzy` columns below show the same library: standalone `search()` vs `FuzzyIndex` (indexed mode for repeated searches).

<details>
<summary>Raw numbers</summary>

| Dataset size | rapid-fuzzy | rapid-fuzzy (indexed) | fuse.js | fuzzysort | uFuzzy |
|---|---:|---:|---:|---:|---:|
| Small (20 items) | 279,509 ops/s | 395,932 ops/s | 118,443 ops/s | **1,661,273 ops/s** | 422,032 ops/s |
| Medium (1K items) | 6,274 ops/s | **77,271 ops/s** | 358 ops/s | 58,123 ops/s | 26,052 ops/s |
| Large (10K items) | 777 ops/s | **230,848 ops/s** | 15 ops/s | 25,315 ops/s | 4,663 ops/s |

</details>

### Closest Match (Levenshtein-based)

| Dataset size | rapid-fuzzy | rapid-fuzzy (indexed) | fastest-levenshtein |
|---|---:|---:|---:|
| Medium (1K items) | 7,690 ops/s | **906,274 ops/s** | 3,536 ops/s |
| Large (10K items) | 611 ops/s | **352,688 ops/s** | 620 ops/s |

> In indexed mode (`FuzzyIndex`), rapid-fuzzy is up to **569x faster** than fastest-levenshtein for closest-match lookups.

### Key takeaways

- **vs fuse.js**: `FuzzyIndex` is **216x–15,390x faster** depending on dataset size. Even standalone `search()` is 18–52x faster.
- **Indexed mode**: `FuzzyIndex` keeps data on the Rust side with incremental caching — **297x faster** than standalone `search()` on large datasets, delivering sub-millisecond autocomplete.
- **vs fuzzysort / uFuzzy**: `FuzzyIndex` outperforms both on 1K+ datasets (up to 9.1x vs fuzzysort, 50x vs uFuzzy).

## Why rapid-fuzzy?

| | rapid-fuzzy | fuse.js | fastest-levenshtein | fuzzysort | uFuzzy |
|---|:---:|:---:|:---:|:---:|:---:|
| **Algorithms** | 10 (Levenshtein, Hamming, Jaro, Dice, …) | Bitap | Levenshtein | Substring | Regex-based |
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
| **Browser** | ✅ WASM (~230 KB gzipped) | ✅ | ✅ | ✅ | ✅ |
| **TypeScript** | ✅ full | ✅ full | ✅ | ✅ | ✅ |

## Migration Guides

Switching from another library? These guides provide API mapping tables, code examples, and performance comparisons:

- [**From string-similarity**](docs/migration/from-string-similarity.md) — Same Dice coefficient algorithm, now maintained and faster
- [**From fuse.js**](docs/migration/from-fuse-js.md) — Up to 15,000x faster fuzzy search with FuzzyIndex
- [**From leven / fastest-levenshtein**](docs/migration/from-leven.md) — Multi-algorithm upgrade with batch APIs
- [**From fuzzysort**](docs/migration/from-fuzzysort.md) — Richer matching with query syntax and 10 distance algorithms
- [**From uFuzzy**](docs/migration/from-ufuzzy.md) — Weighted object search, batch APIs, and persistent indexes
- [**From fuzzball**](docs/migration/from-fuzzball.md) — Same ratio functions (token sort, token set, weighted), now 0.0–1.0 scale
- [**From FlexSearch**](docs/migration/from-flexsearch.md) — Typo-tolerant fuzzy search to replace exact-token full-text search
- [**From MiniSearch**](docs/migration/from-minisearch.md) — Faster fuzzy search with richer distance algorithms

## License

MIT
