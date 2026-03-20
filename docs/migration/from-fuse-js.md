# Migrating from fuse.js to rapid-fuzzy

[fuse.js](https://www.fusejs.io/) is a popular fuzzy search library written in pure JavaScript. However, fuse.js has been effectively unmaintained since March 2025 — the maintainer does periodic releases but does not engage with issues or community PRs. rapid-fuzzy provides significantly faster fuzzy search powered by the nucleo algorithm (same engine used by the Helix editor), while offering a simpler API for common use cases and active maintenance.

## Installation

```bash
# Remove fuse.js
npm uninstall fuse.js

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## Quick Start

```typescript
// Before (fuse.js)
import Fuse from 'fuse.js';
const fuse = new Fuse(['TypeScript', 'JavaScript', 'Python'], {
  threshold: 0.4,
});
const results = fuse.search('typscript');
console.log(results[0].item); // 'TypeScript'

// After (rapid-fuzzy)
import { search } from 'rapid-fuzzy';
const results = search('typscript', ['TypeScript', 'JavaScript', 'Python']);
console.log(results[0].item); // 'TypeScript'
```

## API Mapping

| fuse.js | rapid-fuzzy | Notes |
|---|---|---|
| `new Fuse(list).search(query)` | `search(query, list)` | No constructor needed |
| `results[0].item` | `results[0].item` | Same property name |
| `results[0].score` | `results[0].score` | fuse.js: 0=perfect; rapid-fuzzy: 1.0=perfect |
| `results[0].refIndex` | `results[0].index` | Index in original array |
| `options.threshold` | `{ minScore }` | Note: inverted scale (see below) |
| `options.limit` | `{ maxResults }` or pass a number | `search(q, items, 5)` |
| `options.isCaseSensitive` | `{ isCaseSensitive }` | Same name; default is smart case |
| `options.keys` | `searchObjects(q, items, { keys })` | See [Object Search](#object-search) |
| `options.includeMatches` | `{ includePositions: true }` | Returns character indices |

## Score Direction

**Important**: fuse.js and rapid-fuzzy use opposite score scales:

| | fuse.js | rapid-fuzzy |
|---|---|---|
| Perfect match | `0.0` | `1.0` |
| No match | `1.0` | `0.0` |
| Threshold meaning | "exclude scores above X" | "exclude scores below X" |

```typescript
// fuse.js: threshold 0.4 means "include matches with score ≤ 0.4"
new Fuse(items, { threshold: 0.4 });

// rapid-fuzzy: minScore 0.6 achieves similar filtering
search(query, items, { minScore: 0.6 });
```

## Common Patterns

### Basic search

```typescript
// fuse.js
const fuse = new Fuse(items);
const results = fuse.search('query');

// rapid-fuzzy
const results = search('query', items);
```

### Limiting results

```typescript
// fuse.js
const fuse = new Fuse(items, { limit: 5 });
const results = fuse.search('query');

// rapid-fuzzy — either form works
const results = search('query', items, 5);
const results = search('query', items, { maxResults: 5 });
```

### Score threshold

```typescript
// fuse.js — threshold is 0-1 where lower = stricter
const fuse = new Fuse(items, { threshold: 0.3 });

// rapid-fuzzy — minScore is 0-1 where higher = stricter
const results = search('query', items, { minScore: 0.7 });
```

### Finding the best match

```typescript
// fuse.js
const fuse = new Fuse(items);
const results = fuse.search('query');
const best = results[0]?.item;

// rapid-fuzzy
const best = closest('query', items);
```

### Object search

```typescript
// fuse.js
const fuse = new Fuse(users, {
  keys: ['name', { name: 'email', weight: 0.5 }],
});
const results = fuse.search('john');
console.log(results[0].item); // { name: 'John Smith', email: '...' }

// rapid-fuzzy
import { searchObjects } from 'rapid-fuzzy';
const results = searchObjects('john', users, {
  keys: ['name', { name: 'email', weight: 0.5 }],
});
console.log(results[0].item); // { name: 'John Smith', email: '...' }
```

### Match highlighting

```typescript
// fuse.js — returns match indices, highlighting is manual
const fuse = new Fuse(items, { includeMatches: true });
const results = fuse.search('query');
// results[0].matches[0].indices → [[0, 2], [5, 5]]

// rapid-fuzzy — returns positions + built-in highlight utility
import { search, highlight } from 'rapid-fuzzy';
const results = search('query', items, { includePositions: true });
highlight(results[0].item, results[0].positions, '<b>', '</b>');
```

## Performance

rapid-fuzzy is significantly faster than fuse.js for fuzzy search:

| Dataset size | rapid-fuzzy | FuzzyIndex | fuse.js | Speedup |
|---|---:|---:|---:|---:|
| 1,000 items | 6,274 ops/s | 77,271 ops/s | 358 ops/s | **18x / 216x** |
| 10,000 items | 777 ops/s | 230,848 ops/s | 15 ops/s | **52x / 15,390x** |

The performance advantage grows with dataset size because rapid-fuzzy's Rust-based nucleo engine scales better than fuse.js's pure JavaScript implementation.

For repeated searches against the same dataset, use `FuzzyIndex` (for string arrays) or `FuzzyObjectIndex` (for object arrays with keys) to get even greater speedups. These work similarly to fuse.js's constructor — build once, search many times:

```typescript
import { FuzzyIndex, FuzzyObjectIndex } from 'rapid-fuzzy';

// String search — replaces new Fuse(strings).search(query)
const index = new FuzzyIndex(strings);
const results = index.search('query');

// Object search — replaces new Fuse(objects, { keys }).search(query)
const objIndex = new FuzzyObjectIndex(users, { keys: ['name', 'email'] });
const results = objIndex.search('john');
```

## Additional Capabilities

rapid-fuzzy includes distance/similarity functions that fuse.js does not offer:

```typescript
import {
  levenshtein,           // edit distance
  normalizedLevenshtein, // 0-1 similarity
  jaroWinkler,           // name matching
  sorensenDice,          // text similarity
  damerauLevenshtein,    // transposition-aware
} from 'rapid-fuzzy';

// Batch APIs for bulk operations
import { levenshteinBatch, jaroWinklerMany } from 'rapid-fuzzy';
```
