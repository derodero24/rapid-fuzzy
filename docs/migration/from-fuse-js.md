# Migrating from fuse.js to rapid-fuzzy

[fuse.js](https://www.fusejs.io/) is a popular fuzzy search library written in pure JavaScript. rapid-fuzzy provides significantly faster fuzzy search powered by the nucleo algorithm (same engine used by the Helix editor), while offering a simpler API for common use cases.

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

## What rapid-fuzzy Does Not Support (Yet)

fuse.js has features that rapid-fuzzy does not currently implement:

- **Object search** (`keys` option): fuse.js can search across object properties. rapid-fuzzy operates on string arrays. Extract the field to search before calling.
- **Weighted search**: fuse.js supports per-key weights for object search.
- **Extended search syntax**: fuse.js supports operators like `'word` (prefix), `!term` (negation).
- **Match indices**: fuse.js can return character positions of matches. (Planned: #34)

```typescript
// Workaround for object search
interface Item { name: string; description: string }
const items: Item[] = [/* ... */];

// Extract searchable strings, then map back
const names = items.map(item => item.name);
const results = search('query', names);
const matchedItems = results.map(r => items[r.index]);
```

## Performance

rapid-fuzzy is significantly faster than fuse.js for fuzzy search:

| Dataset size | rapid-fuzzy | fuse.js | Speedup |
|---|---:|---:|---:|
| 1,000 items | 4,941 ops/s | 376 ops/s | **13x** |
| 10,000 items | 588 ops/s | 14 ops/s | **41x** |

The performance advantage grows with dataset size because rapid-fuzzy's Rust-based nucleo engine scales better than fuse.js's pure JavaScript implementation.

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
