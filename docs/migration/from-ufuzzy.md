# Migrating from uFuzzy to rapid-fuzzy

[uFuzzy](https://github.com/leeoniya/uFuzzy) is a lightweight, fast fuzzy search library written in pure JavaScript using regex-based matching. rapid-fuzzy uses the nucleo fuzzy matching engine (same as the Helix editor) with Rust-powered native bindings, and provides a simpler API with ready-to-use sorted results.

## Installation

```bash
# Remove uFuzzy
npm uninstall @leeoniya/ufuzzy

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## Quick Start

```typescript
// Before (uFuzzy)
import uFuzzy from '@leeoniya/ufuzzy';
const uf = new uFuzzy();
const haystack = ['TypeScript', 'JavaScript', 'Python'];
const [idxs, info, order] = uf.search(haystack, 'typscript');
const results = order.map(i => haystack[idxs[i]]);
console.log(results[0]); // 'TypeScript'

// After (rapid-fuzzy)
import { search } from 'rapid-fuzzy';
const results = search('typscript', ['TypeScript', 'JavaScript', 'Python']);
console.log(results[0].item); // 'TypeScript'
```

## API Mapping

| uFuzzy | rapid-fuzzy | Notes |
|---|---|---|
| `new uFuzzy(opts)` + `uf.search(haystack, needle)` | `search(query, items)` | No constructor; returns sorted results directly |
| `idxs[order[i]]` → index into haystack | `results[i].item` | No manual mapping needed |
| `info.ranges` | `results[i].positions` | Use `{ includePositions: true }` |
| `uFuzzy.highlight()` / `mark()` | `highlight(item, positions, open, close)` | Built-in utility |
| `opts.intraMode` / `opts.interMode` | Query syntax (`^`, `$`, `'`, `!`) | Different approach to matching control |
| Manual `order.slice(0, n)` | `{ maxResults: n }` or pass a number | `search(q, items, 5)` |
| Manual sort + slice + map | Automatic | Results are sorted by relevance |

## Result Structure

uFuzzy returns three separate arrays that require manual assembly. rapid-fuzzy returns ready-to-use sorted results:

```typescript
// uFuzzy — 3 return values, manual result construction
const uf = new uFuzzy();
const [idxs, info, order] = uf.search(haystack, 'query');

// Check for no results
if (idxs == null) {
  // no matches
}

// Assemble results manually
const results = [];
for (let i = 0; i < order.length; i++) {
  const idx = cycleOrder[i];
  results.push({
    item: haystack[idxs[idx]],
    ranges: info.ranges[idx],
  });
}

// rapid-fuzzy — sorted results returned directly
const results = search('query', items);
// → [{ item: 'match', score: 0.85, index: 0, positions: [] }, ...]
```

## Common Patterns

### Basic search

```typescript
// uFuzzy
const uf = new uFuzzy();
const [idxs, info, order] = uf.search(haystack, 'query');
const top = haystack[idxs[order[0]]];

// rapid-fuzzy
const results = search('query', items);
const top = results[0]?.item;
```

### Limiting results

```typescript
// uFuzzy — manual slicing
const [idxs, info, order] = uf.search(haystack, 'query');
const top5 = order.slice(0, 5).map(i => haystack[idxs[i]]);

// rapid-fuzzy — either form works
const results = search('query', items, 5);
const results = search('query', items, { maxResults: 5 });
```

### Finding the best match

```typescript
// uFuzzy
const [idxs, info, order] = uf.search(haystack, 'query');
const best = order.length > 0 ? haystack[idxs[order[0]]] : null;

// rapid-fuzzy
import { closest } from 'rapid-fuzzy';
const best = closest('query', items);
```

### Match highlighting

```typescript
// uFuzzy — mark function
const uf = new uFuzzy();
const [idxs, info, order] = uf.search(haystack, 'query');
const highlighted = uFuzzy.highlight(
  haystack[idxs[order[0]]],
  info.ranges[order[0]],
);

// rapid-fuzzy — highlight utility with positions
import { search, highlight, highlightRanges } from 'rapid-fuzzy';
const results = search('query', items, { includePositions: true });
highlight(results[0].item, results[0].positions, '<b>', '</b>');

// Callback form (React, JSX)
highlight(results[0].item, results[0].positions, (matched) => `<mark>${matched}</mark>`);

// Raw ranges for custom rendering
highlightRanges(results[0].item, results[0].positions);
// → [{ start: 0, end: 1, matched: true }, ...]
```

### Object search

uFuzzy does not have built-in object search — you must extract string values, search them, and map results back manually. rapid-fuzzy provides dedicated object search:

```typescript
// uFuzzy — manual key extraction
const names = users.map(u => u.name);
const [idxs, info, order] = uf.search(names, 'john');
const matchedUsers = order.map(i => users[idxs[i]]);

// rapid-fuzzy — built-in object search with weighted keys
import { searchObjects } from 'rapid-fuzzy';
const results = searchObjects('john', users, {
  keys: [
    { name: 'name', weight: 2.0 },
    { name: 'email', weight: 1.0 },
  ],
});
console.log(results[0].item); // { name: 'John Smith', email: '...' }
```

### Persistent index

```typescript
// uFuzzy — no persistent index, re-searches the full array each time

// rapid-fuzzy — persistent index with mutation support
import { FuzzyIndex, FuzzyObjectIndex } from 'rapid-fuzzy';

const index = new FuzzyIndex(items);
index.search('query', { maxResults: 10 });

// Mutate without rebuilding
index.add('new item');
index.remove(2); // swap-remove by index

// Object index with weighted keys
const userIndex = new FuzzyObjectIndex(users, {
  keys: [
    { name: 'name', weight: 2.0 },
    { name: 'email', weight: 1.0 },
  ],
});

// Free Rust-side memory when done
index.destroy();
userIndex.destroy();
```

## Performance

uFuzzy is a highly optimized pure JavaScript library that uses regex-based matching. It is slightly faster than rapid-fuzzy for raw search speed and uses less memory since it avoids FFI overhead:

| Dataset size | rapid-fuzzy | FuzzyIndex | uFuzzy |
|---|---:|---:|---:|
| Small (20 items) | 220,621 ops/s | 403,559 ops/s | **933,599 ops/s** |
| Medium (1K items) | 6,654 ops/s | 21,658 ops/s | **26,986 ops/s** |
| Large (10K items) | 812 ops/s | 4,638 ops/s | **6,234 ops/s** |
| XL (50K items) | — | 864 ops/s | **1,271 ops/s** |

Measured on Apple M-series with Node.js v22.

uFuzzy is faster because it uses optimized regex-based matching in pure JavaScript, avoiding FFI overhead entirely. If raw search speed and minimal memory usage are your primary concerns and you don't need the features below, uFuzzy is an excellent choice.

## Why Choose rapid-fuzzy Over uFuzzy?

While uFuzzy wins on raw speed and memory efficiency, rapid-fuzzy offers capabilities that uFuzzy does not:

- **9 distance algorithms**: Levenshtein, Damerau-Levenshtein, Jaro, Jaro-Winkler, Sorensen-Dice, and more — useful beyond search
- **Simpler API**: Returns ready-to-use sorted results instead of three arrays requiring manual assembly
- **Batch APIs**: `levenshteinBatch`, `jaroWinklerMany`, etc. for bulk distance computations
- **Weighted object search**: `searchObjects()` and `FuzzyObjectIndex` with per-key weights — no manual key extraction needed
- **Persistent mutable index**: `FuzzyIndex` supports `add()` / `remove()` without rebuilding
- **Full TypeScript types**: Auto-generated type definitions with full coverage
- **Extended query syntax**: Exclude (`!term`), prefix (`^term`), suffix (`term$`), exact (`'term`) operators
- **WASM fallback**: Works in browsers, Deno, and Bun via automatic WASM fallback

## Additional Capabilities

rapid-fuzzy includes distance/similarity functions that uFuzzy does not offer:

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
