# Migrating from fuzzysort to rapid-fuzzy

[fuzzysort](https://github.com/farzher/fuzzysort) is a fast fuzzy search library that uses substring matching and a custom scoring algorithm. rapid-fuzzy takes a different approach — it uses the nucleo fuzzy matching engine (same as the Helix editor) and adds 9 distance algorithms, query syntax, and batch APIs for broader use cases.

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
| `fuzzysort.highlight(result)` | `highlight(item, positions, '<b>', '</b>')` | Built-in highlight utility |
| `fuzzysort.prepare(target)` | `new FuzzyIndex(items)` | Persistent index, mutable |
| `result.target` | `result.item` | Different property name |
| `result.score` | `result.score` | Different scale (see below) |
| `result.indexes` | `result.positions` | Character indices |
| `options.threshold` | `{ minScore }` | Different scale (see below) |
| `options.limit` | `{ maxResults }` or pass a number | `search(q, items, 5)` |
| `options.key` / `options.keys` | `searchObjects(q, items, { keys })` | See [Object Search](#object-search) |

## Score Direction

**Important**: fuzzysort and rapid-fuzzy use very different scoring systems:

| | fuzzysort | rapid-fuzzy |
|---|---|---|
| Perfect match | `0` | `1.0` |
| No match | `-Infinity` | `0.0` |
| Scale | Negative integers (closer to 0 = better) | 0.0–1.0 (higher = better) |
| Default threshold | `-Infinity` (include all) | None (include all) |

```typescript
// fuzzysort: threshold is negative, closer to 0 = stricter
fuzzysort.go('query', items, { threshold: -1000 });

// rapid-fuzzy: minScore is 0-1, higher = stricter
search('query', items, { minScore: 0.3 });
```

## Common Patterns

### Basic search

```typescript
// fuzzysort
const results = fuzzysort.go('query', items);
results[0].target; // matched string
results[0].score;  // negative integer

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
// fuzzysort — threshold is a negative integer
fuzzysort.go('query', items, { threshold: -500 });

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
fuzzysort.highlight(result); // '<b>T</b>ypeScr<b>i</b>pt'
fuzzysort.highlight(result, '<mark>', '</mark>');

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

fuzzysort uses a substring-based matching algorithm that is extremely fast for raw search speed. rapid-fuzzy uses the nucleo fuzzy matching engine, which provides different (and often more flexible) matching behavior at a different performance profile:

| Dataset size | rapid-fuzzy | FuzzyIndex | fuzzysort |
|---|---:|---:|---:|
| Small (20 items) | 287,682 ops/s | 404,271 ops/s | **2,655,421 ops/s** |
| Medium (1K items) | 6,827 ops/s | **79,616 ops/s** | 63,831 ops/s |
| Large (10K items) | 827 ops/s | **136,294 ops/s** | 27,897 ops/s |

Measured on Apple M-series with Node.js v22.

fuzzysort is faster on small datasets because it uses optimized substring matching — a fundamentally simpler operation than fuzzy matching with out-of-order support. However, with `FuzzyIndex`, rapid-fuzzy outperforms fuzzysort on medium-and-above datasets (1.2x at 1K, 4.9x at 10K) thanks to Rust-side indexing with incremental caching.

## Why Choose rapid-fuzzy Over fuzzysort?

rapid-fuzzy with `FuzzyIndex` matches or exceeds fuzzysort's speed on real-world dataset sizes, while offering capabilities that fuzzysort does not:

- **9 distance algorithms**: Levenshtein, Damerau-Levenshtein, Jaro, Jaro-Winkler, Sorensen-Dice, and more — useful beyond search
- **Query syntax**: Exclude (`!term`), prefix (`^term`), suffix (`term$`), exact (`'term`) operators
- **Diacritics handling**: `cafe` matches `café`, `uber` matches `über` automatically
- **Batch APIs**: `levenshteinBatch`, `jaroWinklerMany`, etc. for bulk operations
- **Object search with weighted keys**: `searchObjects()` and `FuzzyObjectIndex` with per-key weights
- **Out-of-order matching**: `john smith` matches "Smith, John" automatically
- **Mutable persistent index**: `FuzzyIndex` supports `add()` / `remove()` without rebuilding
- **WASM fallback**: Works in browsers, Deno, and Bun via automatic WASM fallback

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
