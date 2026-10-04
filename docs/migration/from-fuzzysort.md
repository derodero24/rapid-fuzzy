# Migrating from fuzzysort to rapid-fuzzy

[fuzzysort](https://github.com/farzher/fuzzysort) is a fast fuzzy search library written in pure JavaScript. Like rapid-fuzzy, it matches a query when its characters occur in the item in order, and ranks matches with its own scoring. rapid-fuzzy uses the nucleo matcher (the engine of the Helix editor) in a Rust core, and adds query syntax, weighted object search, distance functions and batch APIs. Scores and rankings differ between the two.

## Installation

```bash
# Remove fuzzysort
npm uninstall fuzzysort

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## Quick Start

```typescript
// Before (fuzzysort)
import fuzzysort from 'fuzzysort';
const results = fuzzysort.go('typscript', ['TypeScript', 'JavaScript', 'Python']);
console.log(results[0].target); // 'TypeScript'

// After (rapid-fuzzy)
import { search } from 'rapid-fuzzy';
const results = search('typscript', ['TypeScript', 'JavaScript', 'Python']);
console.log(results[0].item); // 'TypeScript'
```

## API Mapping

| fuzzysort | rapid-fuzzy | Notes |
|---|---|---|
| `fuzzysort.go(query, targets)` | `search(query, items)` | No constructor needed |
| `fuzzysort.go(query, targets, { key })` | `searchObjects(query, items, { keys })` | See [Object Search](#object-search) |
| `result.highlight('<b>', '</b>')` | `highlight(item, positions, '<b>', '</b>')` | Built-in highlight utility |
| `fuzzysort.prepare(target)` | `new FuzzyIndex(items)` | Persistent index, mutable |
| `result.target` | `result.item` | Different property name |
| `result.score` | `result.score` | Different scale (see below) |
| `result.indexes` | `result.positions` | Character indices |
| `options.threshold` | `{ minScore }` | Same direction, different scores (see below) |
| `options.limit` | `{ maxResults }` or pass a number | `search(q, items, 5)` |
| `options.key` / `options.keys` | `searchObjects(q, items, { keys })` | See [Object Search](#object-search) |

## Score Direction

**Important**: fuzzysort and rapid-fuzzy use different scoring systems. Both use 0–1 with higher = better, but the same match gets different scores (`tsc` → `TypeScript`: 0.74 in fuzzysort, 0.83 in rapid-fuzzy), so pick `minScore` from rapid-fuzzy's scores on your data:

| | fuzzysort | rapid-fuzzy |
|---|---|---|
| Perfect match | `1` | `1.0` |
| No match | not returned | not returned |
| Scale | 0–1 (higher = better) | 0.0–1.0 (higher = better) |
| Default threshold | `0.5` (fuzzysort 4; `0` in 3.x) | None (include all) |

```typescript
// fuzzysort 4: threshold is 0-1, higher = stricter, and defaults to 0.5
fuzzysort.go('query', items, { threshold: 0.3 });

// rapid-fuzzy: minScore is 0-1, higher = stricter, and defaults to 0 (include all)
search('query', items, { minScore: 0.3 });
```

## Common Patterns

### Basic search

```typescript
// fuzzysort
const results = fuzzysort.go('query', items);
results[0].target; // matched string
results[0].score;  // 0–1

// rapid-fuzzy
const results = search('query', items);
results[0].item;   // matched string
results[0].score;  // 0.0–1.0
```

### Limiting results

```typescript
// fuzzysort
const results = fuzzysort.go('query', items, { limit: 5 });

// rapid-fuzzy — either form works
const results = search('query', items, 5);
const results = search('query', items, { maxResults: 5 });
```

### Score threshold

```typescript
// fuzzysort — threshold is 0-1 (default 0.5 since fuzzysort 4)
fuzzysort.go('query', items, { threshold: 0.5 });

// rapid-fuzzy — minScore is 0-1 where higher = stricter
search('query', items, { minScore: 0.5 });
```

### Object search

```typescript
// fuzzysort — single key
fuzzysort.go('john', users, { key: 'name' });

// fuzzysort — multiple keys
fuzzysort.go('john', users, { keys: ['name', 'email'] });

// rapid-fuzzy
import { searchObjects } from 'rapid-fuzzy';
searchObjects('john', users, {
  keys: ['name', 'email'],
});

// rapid-fuzzy — weighted keys
searchObjects('john', users, {
  keys: [
    { name: 'name', weight: 2.0 },
    { name: 'email', weight: 1.0 },
  ],
});
```

### Match highlighting

```typescript
// fuzzysort — built-in HTML highlight
const [result] = fuzzysort.go('tsc', ['TypeScript']);
result.highlight('<b>', '</b>'); // '<b>T</b>ype<b>Sc</b>ript'

// rapid-fuzzy — highlight utility with positions
import { search, highlight } from 'rapid-fuzzy';
const results = search('query', items, { includePositions: true });
highlight(results[0].item, results[0].positions, '<b>', '</b>');

// Callback form (React, JSX)
highlight(results[0].item, results[0].positions, (matched) => `<mark>${matched}</mark>`);
```

### Prepared targets / persistent index

```typescript
// fuzzysort — prepare targets for faster repeated searches
const prepared = items.map(fuzzysort.prepare);
fuzzysort.go('query', prepared);

// rapid-fuzzy — persistent index with mutation support
import { FuzzyIndex } from 'rapid-fuzzy';
const index = new FuzzyIndex(items);
index.search('query');

// Mutate without rebuilding
index.add('new item');
index.remove(2); // swap-remove by index

// Free Rust-side memory when done
index.destroy();
```

## Performance

From the [README benchmarks](../../README.md#benchmarks) (Node.js 22, Linux x64, Intel Xeon @ 2.10GHz, 4 vCPUs; ops/s, higher is better; fuzzysort with prepared targets from 1K items on):

| Dataset size | rapid-fuzzy `search()` | `FuzzyIndex` | fuzzysort |
|---|---:|---:|---:|
| Small (20 items) | 157,995 ops/s | 208,250 ops/s | **1,271,223 ops/s** |
| Medium (1K items) | 3,502 ops/s | 50,318 ops/s | **54,110 ops/s** |
| Large (10K items) | 508 ops/s | **4,803 ops/s** | 2,977 ops/s |
| Large (10K items, rotating queries) | 451 ops/s | **4,644 ops/s** | 2,231 ops/s |
| Huge (100K items) | — | **1,125 ops/s** | 231 ops/s |

fuzzysort was about 6x faster on 20 items and about as fast on 1K items. `FuzzyIndex` was 1.6-2.1x faster on 10K items, and about 5x faster on 100K items, where it scored candidates on 4 threads (about 1.6x faster on one thread). Standalone `search()` was slower than fuzzysort's prepared search at every size, because it converts every item on each call.

## Why Choose rapid-fuzzy Over fuzzysort?

fuzzysort is an excellent choice for small and medium lists in the browser, with no native addon or WebAssembly module to load. rapid-fuzzy is worth it when you need:

- **Distance functions**: Levenshtein, Damerau-Levenshtein, Hamming, Indel, Jaro, Jaro-Winkler, Sorensen-Dice, and token-based ratios — useful beyond search
- **Query syntax**: Exclude (`!term`), prefix (`^term`), suffix (`term$`), exact (`'term`) operators
- **Batch APIs**: `levenshteinBatch`, `jaroWinklerMany`, etc. for bulk operations
- **Object search with weighted keys**: `searchObjects()` and `FuzzyObjectIndex` with per-key weights
- **Mutable persistent index**: `FuzzyIndex` supports `add()` / `remove()` without rebuilding, and `serialize()` / `deserialize()`
- **Large lists in Node.js**: native code that scales to 100K+ items

## Additional Capabilities

rapid-fuzzy includes distance/similarity functions that fuzzysort does not offer:

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
