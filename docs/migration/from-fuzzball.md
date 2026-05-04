# Migrating from fuzzball to rapid-fuzzy

[fuzzball](https://www.npmjs.com/package/fuzzball) is a JavaScript port of Python's fuzzywuzzy library, providing ratio-based string matching with token sort/set algorithms. rapid-fuzzy provides the same ratio functions with better performance and additional distance algorithms — all powered by a Rust core.

## Installation

```bash
# Remove fuzzball
npm uninstall fuzzball

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## API Mapping

| fuzzball | rapid-fuzzy | Notes |
|---|---|---|
| `fuzz.ratio(a, b)` | `normalizedLevenshtein(a, b) * 100` | fuzzball returns 0–100, rapid-fuzzy returns 0.0–1.0 |
| `fuzz.partial_ratio(a, b)` | `partialRatio(a, b)` | Returns 0.0–1.0 |
| `fuzz.token_sort_ratio(a, b)` | `tokenSortRatio(a, b)` | Returns 0.0–1.0 |
| `fuzz.token_set_ratio(a, b)` | `tokenSetRatio(a, b)` | Returns 0.0–1.0 |
| `fuzz.WRatio(a, b)` | `weightedRatio(a, b)` | Returns 0.0–1.0 |
| `fuzz.extract(query, choices)` | `search(query, choices)` | Returns `{ item, score, index }[]` |

> **Score scale difference**: fuzzball returns integers 0–100, rapid-fuzzy returns floats 0.0–1.0. Multiply by 100 if you need the old scale.

## Code Examples

### Basic ratio

```typescript
// Before (fuzzball)
import * as fuzz from 'fuzzball';
fuzz.ratio('hello', 'hello');  // 100
fuzz.ratio('hello', 'world');  // 20

// After (rapid-fuzzy)
import { normalizedLevenshtein } from 'rapid-fuzzy';
normalizedLevenshtein('hello', 'hello');  // 1.0
normalizedLevenshtein('hello', 'world');  // 0.2
```

### Token-based matching

```typescript
// Before (fuzzball)
import * as fuzz from 'fuzzball';
fuzz.token_sort_ratio('New York Mets', 'Mets New York');   // 100
fuzz.token_set_ratio('hello', 'hello world');               // 100

// After (rapid-fuzzy)
import { tokenSortRatio, tokenSetRatio } from 'rapid-fuzzy';
tokenSortRatio('New York Mets', 'Mets New York');  // 1.0
tokenSetRatio('hello', 'hello world');               // 1.0
```

### Weighted ratio

```typescript
// Before (fuzzball)
import * as fuzz from 'fuzzball';
fuzz.WRatio('hello world', 'world hello');  // 100

// After (rapid-fuzzy)
import { weightedRatio } from 'rapid-fuzzy';
weightedRatio('hello world', 'world hello');  // 1.0
```

### Extract best matches

```typescript
// Before (fuzzball)
import * as fuzz from 'fuzzball';
const results = fuzz.extract('python', ['Python', 'JavaScript', 'TypeScript']);
// [[string, score, index], ...]

// After (rapid-fuzzy)
import { search } from 'rapid-fuzzy';
const results = search('python', ['Python', 'JavaScript', 'TypeScript']);
// [{ item: 'Python', score: 1.0, index: 0, positions: [] }, ...]
```

## What You Gain

### Batch APIs

Process multiple comparisons in a single call for better throughput:

```typescript
import { tokenSortRatioBatch, tokenSortRatioMany } from 'rapid-fuzzy';

// Compare multiple pairs at once
tokenSortRatioBatch([
  ['hello world', 'world hello'],
  ['foo bar', 'baz qux'],
]);

// Compare one string against many
tokenSortRatioMany('hello world', candidates);
```

### Additional algorithms

Beyond ratio-based functions, access edit distance and similarity algorithms:

```typescript
import {
  levenshtein,        // Edit distance (integer)
  damerauLevenshtein, // Handles transpositions
  jaroWinkler,        // Prefix-weighted, great for names
  sorensenDice,       // Bigram-based similarity
  hamming,            // Fixed-length positional comparison
} from 'rapid-fuzzy';
```

### Persistent indexed search

For repeated searches, `FuzzyIndex` is dramatically faster than calling `search()` in a loop:

```typescript
import { FuzzyIndex } from 'rapid-fuzzy';
const index = new FuzzyIndex(items);
index.search('python');   // Sub-millisecond on large datasets
index.closest('python');  // Best single match
```

### TypeScript support

rapid-fuzzy ships with built-in TypeScript declarations. No `@types/` package needed.

## Key Differences

- **Score scale**: fuzzball returns 0–100 (integer); rapid-fuzzy returns 0.0–1.0 (float). Adjust thresholds accordingly.
- **Partial ratio**: Both implement the same algorithm (best substring alignment via normalized Levenshtein).
- **`extract` vs `search`**: `search()` returns richer result objects with `item`, `score`, `index`, and `positions`.
- **`process` module**: fuzzball's `process.extract` / `process.extractBests` map to `search()` with `maxResults` and `minScore` options.
