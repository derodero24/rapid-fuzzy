# Migrating from leven / fastest-levenshtein to rapid-fuzzy

[leven](https://www.npmjs.com/package/leven) and [fastest-levenshtein](https://www.npmjs.com/package/fastest-levenshtein) are single-purpose Levenshtein distance libraries. rapid-fuzzy provides the same Levenshtein distance plus five additional algorithms, batch APIs, and fuzzy search — all from a single package.

## Installation

```bash
# Remove the old library
npm uninstall leven
# or
npm uninstall fastest-levenshtein

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## API Mapping

### From leven

| leven | rapid-fuzzy |
|---|---|
| `leven(a, b)` | `levenshtein(a, b)` |

```typescript
// Before (leven)
import leven from 'leven';
leven('kitten', 'sitting'); // 3

// After (rapid-fuzzy)
import { levenshtein } from 'rapid-fuzzy';
levenshtein('kitten', 'sitting'); // 3
```

### From fastest-levenshtein

| fastest-levenshtein | rapid-fuzzy |
|---|---|
| `distance(a, b)` | `levenshtein(a, b)` |
| `closest(s, targets)` | `closest(s, targets)` |

```typescript
// Before (fastest-levenshtein)
import { distance, closest } from 'fastest-levenshtein';
distance('kitten', 'sitting');                  // 3
closest('fast', ['slow', 'faster', 'fastest']); // 'faster'

// After (rapid-fuzzy)
import { levenshtein, closest } from 'rapid-fuzzy';
levenshtein('kitten', 'sitting');                  // 3
closest('fast', ['slow', 'faster', 'fastest']);     // 'faster'
```

## What You Gain

By switching to rapid-fuzzy, you get access to a broader set of tools without adding extra dependencies:

### Multiple algorithms

```typescript
import {
  levenshtein,           // Same as leven / fastest-levenshtein
  normalizedLevenshtein, // 0.0-1.0 similarity (length-independent)
  damerauLevenshtein,    // Handles transpositions (ab → ba = 1 edit)
  jaroWinkler,           // Prefix-weighted, great for names
  sorensenDice,          // Bigram-based text similarity
  jaro,                  // Base Jaro similarity
} from 'rapid-fuzzy';
```

### Batch APIs

Process multiple pairs in a single FFI call for better performance:

```typescript
import { levenshteinBatch, levenshteinMany } from 'rapid-fuzzy';

// Compare multiple pairs at once
const distances = levenshteinBatch([
  ['kitten', 'sitting'],
  ['hello', 'world'],
  ['fast', 'faster'],
]);
// [3, 4, 2]

// Compare one string against many
const scores = levenshteinMany('hello', ['help', 'held', 'world']);
// [1, 2, 4]
```

### Fuzzy search

```typescript
import { search, closest } from 'rapid-fuzzy';

// Find the best match
const best = closest('typscript', ['TypeScript', 'JavaScript', 'Python']);
// 'TypeScript'

// Search with ranked results
const results = search('type', ['TypeScript', 'JavaScript', 'Python']);
// [{ item: 'TypeScript', score: 1.0, index: 0 }, ...]
```

## Performance Considerations

For single-pair Levenshtein distance, fastest-levenshtein is faster due to its highly optimized pure-JS implementation that avoids FFI overhead:

| Operation | rapid-fuzzy | fastest-levenshtein | leven |
|---|---:|---:|---:|
| Single pair | 67,346 ops/s | **243,026 ops/s** | 51,789 ops/s |

However, rapid-fuzzy excels in closest-match scenarios where batch FFI overhead is amortized:

| Closest match | rapid-fuzzy | fastest-levenshtein | Speedup |
|---|---:|---:|---:|
| 1,000 items | **5,912 ops/s** | 3,974 ops/s | 1.5x |
| 10,000 items | **387 ops/s** | 126 ops/s | 3x |

**When to choose rapid-fuzzy over fastest-levenshtein**:
- You need more than just Levenshtein (similarity scores, other algorithms)
- You process batches of string pairs
- You need fuzzy search / closest match on medium-to-large datasets
- You want a single dependency for all string distance needs

**When to keep fastest-levenshtein**:
- You only need Levenshtein distance for individual pairs
- Maximum single-pair throughput is critical
