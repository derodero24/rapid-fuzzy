# Migrating from leven / fastest-levenshtein to rapid-fuzzy

[leven](https://www.npmjs.com/package/leven) and [fastest-levenshtein](https://www.npmjs.com/package/fastest-levenshtein) are single-purpose Levenshtein distance libraries. rapid-fuzzy provides the same Levenshtein distance (for text in the Basic Multilingual Plane, see [Emoji and other non-BMP characters](#emoji-and-other-non-bmp-characters)) plus other distance metrics (Damerau-Levenshtein, Hamming, Indel, Jaro, Jaro-Winkler, Sorensen-Dice), token-based ratios, batch APIs, and fuzzy search — all from a single package.

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
| `closest(s, targets)` | `levenshteinMany(s, targets)` + pick the minimum (see below) |

```typescript
// Before (fastest-levenshtein)
import { distance, closest } from 'fastest-levenshtein';
distance('kitten', 'sitting');                    // 3
closest('kitten', ['sitting', 'mitten', 'kitchen']); // 'mitten'

// After (rapid-fuzzy)
import { levenshtein, levenshteinMany } from 'rapid-fuzzy';
levenshtein('kitten', 'sitting'); // 3

const targets = ['sitting', 'mitten', 'kitchen'];
const distances = levenshteinMany('kitten', targets); // [3, 1, 2]
const best = targets[distances.indexOf(Math.min(...distances))]; // 'mitten'
```

> **rapid-fuzzy's `closest()` is not an edit-distance search.** It returns the best *fuzzy search* match: an item that contains the query's characters in order, ranked by the nucleo matcher. `closest('kitten', ['sitting', 'mitten', 'kitchen'])` returns `null`, because no target contains `k`, `i`, `t`, `t`, `e`, `n` in that order. Use `levenshteinMany` as above to keep fastest-levenshtein's behaviour.

### Emoji and other non-BMP characters

rapid-fuzzy counts Unicode code points, while leven and fastest-levenshtein count UTF-16 code units. The two are the same for text in the Basic Multilingual Plane (Latin, Cyrillic, Greek, CJK, combining marks and so on), and the distances agreed on every pair in our differential tests of such text. An emoji or another character outside the BMP is one code point but two UTF-16 code units, so distances involving one can differ, and so can the best match:

```typescript
levenshtein('😀', 'a');        // 1 (leven and fastest-levenshtein's distance(): 2)
levenshtein('café 😀', 'cafe'); // 3 (leven and distance(): 4)

// fastest-levenshtein's closest('😀', ['ab', 'a']) returns 'ab' (distance 2 to both)
levenshteinMany('😀', ['ab', 'a']); // [2, 1]: the recipe above picks 'a'
```

## What You Gain

### Multiple algorithms

```typescript
import {
  levenshtein,           // Same as leven / fastest-levenshtein (for BMP text)
  normalizedLevenshtein, // 0.0-1.0 similarity (length-independent)
  damerauLevenshtein,    // Handles transpositions (ab → ba = 1 edit)
  hamming,               // Positional differences (equal-length strings)
  indel,                 // Insertions and deletions only
  jaroWinkler,           // Prefix-weighted, great for names
  sorensenDice,          // Bigram-based text similarity
  jaro,                  // Base Jaro similarity
} from 'rapid-fuzzy';
```

### Batch APIs

Process many pairs in a single call:

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
// [2, 2, 4]

// Stop early above a maximum distance (exceeding candidates get maxDistance + 1)
levenshteinMany('hello', ['help', 'held', 'world'], 2);
// [2, 2, 3]
```

### Fuzzy search

```typescript
import { search, closest } from 'rapid-fuzzy';

// Find the best fuzzy match
const best = closest('typscript', ['TypeScript', 'JavaScript', 'Python']);
// 'TypeScript'

// Search with ranked results
const results = search('type', ['TypeScript', 'JavaScript', 'Python']);
// [{ item: 'TypeScript', score: 1, index: 0, positions: [] }]
```

## Performance Considerations

From the [README benchmarks](../../README.md#benchmarks) (Node.js 22, Linux x64, Intel Xeon @ 2.10GHz; 6 string pairs per operation; ops/s, higher is better):

| Operation | rapid-fuzzy | fastest-levenshtein | leven |
|---|---:|---:|---:|
| Single pairs (`levenshtein` / `distance` / `leven`) | 333,490 ops/s | **400,349 ops/s** | 131,383 ops/s |
| 1 vs 1,000 short candidates | 3,752 ops/s (`levenshteinMany`) | **10,458 ops/s** (loop over `distance`) | — |

fastest-levenshtein is a highly optimized pure-JS implementation, and every rapid-fuzzy call has to cross the JS/Rust boundary and convert its strings: for short strings fastest-levenshtein was about 1.2x faster per pair, and about 2.8x faster than `levenshteinMany` over 1,000 short candidates. rapid-fuzzy was about 2.5x faster than leven. On long strings the native bit-parallel implementation is fast (two 100,000-character strings take about 0.5 s).

**When to choose rapid-fuzzy**:
- You need more than Levenshtein (normalized similarities, Damerau-Levenshtein, Jaro-Winkler, token ratios)
- You need fuzzy search, object search or a persistent search index in the same package
- You want a single dependency for string distance and search

**When to keep fastest-levenshtein**:
- You only need Levenshtein distance on short strings
- Maximum per-pair throughput, or the smallest bundle (no native addon or WebAssembly), matters most
