# Migrating from fuse.js to rapid-fuzzy

[fuse.js](https://www.fusejs.io/) is a popular fuzzy search library written in pure JavaScript, based on the Bitap algorithm. rapid-fuzzy uses the nucleo matcher (the engine of the Helix editor) in a Rust core, which is much faster on larger lists, especially with a persistent `FuzzyIndex`. The two libraries match differently, so read [Matching differences](#matching-differences) before switching.

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
| `new Fuse(list)` reused for many queries | `new FuzzyIndex(list)` | Build once, search many times |
| `results[0].item` | `results[0].item` | Same property name |
| `results[0].score` | `results[0].score` | fuse.js: 0 = perfect; rapid-fuzzy: 1.0 = perfect. Not comparable otherwise |
| `results[0].refIndex` | `results[0].index` | Index in original array |
| `options.threshold` | `{ minScore }` | Different scale and meaning (see below) |
| `fuse.search(query, { limit })` | `{ maxResults }` or pass a number | `search(q, items, 5)` |
| `options.isCaseSensitive` | `{ isCaseSensitive }` | `true` is case-sensitive in both. `false` is fully case-insensitive in fuse.js, but **smart case** in rapid-fuzzy (see below) |
| `options.keys` | `searchObjects(q, items, { keys })` | See [Object search](#object-search) |
| `options.includeMatches` | `{ includePositions: true }` | Returns character indices |

## Matching differences

- **Typos**: fuse.js's Bitap algorithm tolerates substituted and swapped letters (`tpyescript` finds `TypeScript`). rapid-fuzzy's `search()` only tolerates letters *missing* from the query (`typscript`, `tscript`, `tsc` all find `TypeScript`) and finds nothing for `tpyescript`. If you need substitution and transposition tolerance, score candidates with a distance function such as `jaroWinklerMany` or `damerauLevenshtein`.
- **Case**: rapid-fuzzy is smart case by default — a lower-case query matches any case, a query containing an upper-case letter is matched case-sensitively — and `isCaseSensitive: false` does not change that. Lower-case the query if you want case-insensitive matching for every query.
- **Location**: fuse.js prefers matches near the start of the string (`location`, `distance`, `ignoreLocation`). rapid-fuzzy has no such options; nucleo gives bonuses to matches at the start of the item and of words.

## Score Direction

fuse.js and rapid-fuzzy use opposite score scales:

| | fuse.js | rapid-fuzzy |
|---|---|---|
| Perfect match | `0.0` | `1.0` |
| No match | not returned | not returned |
| Threshold meaning | "exclude scores above X" | "exclude scores below X" |

The scores come from different algorithms, so there is no formula that converts a fuse.js `threshold` into a rapid-fuzzy `minScore`. For example, for the query `appl` over `['apple', 'application', 'apply', 'maple', 'pineapple', 'grape']`, fuse.js with `threshold: 0.4` returns `maple` but rapid-fuzzy does not (the letters of `appl` do not occur in order in `maple`), and rapid-fuzzy scores `pineapple` 0.67 where fuse.js scores it 0.04. Start without a `minScore`, look at the scores on your own data, and pick a threshold from them.

```typescript
// fuse.js: threshold 0.4 means "include matches with score ≤ 0.4"
new Fuse(items, { threshold: 0.4 });

// rapid-fuzzy: minScore drops results scoring below it; choose it from your data
search(query, items, { minScore: 0.5 });
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
// fuse.js — limit is a search option
const fuse = new Fuse(items);
const results = fuse.search('query', { limit: 5 });

// rapid-fuzzy — either form works
const results = search('query', items, 5);
const results = search('query', items, { maxResults: 5 });
```

### Finding the best match

```typescript
// fuse.js
const fuse = new Fuse(items);
const results = fuse.search('query');
const best = results[0]?.item;

// rapid-fuzzy — returns the first result of search(), or null
const best = closest('query', items);
```

### Object search

```typescript
const users = [
  { name: 'John Smith', email: 'john@example.com' },
  { name: 'Jane Doe', email: 'jane@example.com' },
  { name: 'Bob Johnson', email: 'bob@test.com' },
];

// fuse.js
const fuse = new Fuse(users, {
  keys: ['name', { name: 'email', weight: 0.5 }],
});
fuse.search('john').map((r) => r.item.name);
// ['John Smith', 'Bob Johnson', 'Jane Doe']

// rapid-fuzzy
import { searchObjects } from 'rapid-fuzzy';
searchObjects('john', users, {
  keys: ['name', { name: 'email', weight: 0.5 }],
}).map((r) => r.item.name);
// ['John Smith', 'Bob Johnson'] — scores 1 and 0.67
```

By default rapid-fuzzy's combined score is the weighted average of the per-key scores (returned as `keyScores`) over all keys, so a key that does not match lowers the score (Bob Johnson's email). fuse.js only combines the keys that match; the closest rapid-fuzzy equivalent is `scoreMode: 'matched'`, the weighted average over the matching keys only (`'max'` takes the best key score instead):

```typescript
searchObjects('john', users, {
  keys: ['name', { name: 'email', weight: 0.5 }],
  scoreMode: 'matched',
}).map((r) => [r.item.name, r.score]);
// [['John Smith', 1], ['Bob Johnson', 1]]
```

In every mode an item must match at least one key with a positive weight. See [Combining key scores](../../README.md#combining-key-scores-scoremode).

By default every term of a multi-term query must match the same key: like fuse.js's extended search (`useExtendedSearch`), `john tokyo` finds nothing in `{ name: 'John Smith', city: 'Tokyo' }`. Where fuse.js needs a logical `$and` query that names the key of each term, rapid-fuzzy has `matchMode: 'crossKey'`, which matches every term against every key on its own (a `!term` then excludes items in which it matches any key):

```typescript
const people = [{ name: 'John Smith', city: 'Tokyo' }];
searchObjects('john tokyo', people, { keys: ['name', 'city'], matchMode: 'crossKey' });
// [{ item: { name: 'John Smith', city: 'Tokyo' }, score: 0.5, ... }]
```

See [Matching terms across keys](../../README.md#matching-terms-across-keys-matchmode).

### Match highlighting

```typescript
// fuse.js — returns match index ranges, highlighting is manual
const fuse = new Fuse(['apple'], { includeMatches: true });
fuse.search('appl')[0].matches[0].indices; // [[0, 3]]

// rapid-fuzzy — returns matched character positions + a highlight utility
import { search, highlight } from 'rapid-fuzzy';
const [result] = search('appl', ['apple'], { includePositions: true });
result.positions; // [0, 1, 2, 3]
highlight(result.item, result.positions, '<b>', '</b>'); // '<b>appl</b>e'
```

## Performance

From the [README benchmarks](../../README.md#benchmarks) (Node.js 22, Linux x64, Intel Xeon @ 2.10GHz; ops/s, higher is better):

| Dataset size | rapid-fuzzy `search()` | `FuzzyIndex` | fuse.js |
|---|---:|---:|---:|
| 1,000 items | 3,502 ops/s | 50,318 ops/s | 249 ops/s |
| 10,000 items | 508 ops/s | 4,803 ops/s | 27 ops/s |
| 10,000 items, rotating queries | 451 ops/s | 4,644 ops/s | 32 ops/s |

The two libraries do not return the same matches (see above), so these numbers compare the cost of a search, not identical work. On these datasets `FuzzyIndex` was roughly 150-200x faster than fuse.js and standalone `search()` roughly 14-19x.

For repeated searches against the same dataset, use `FuzzyIndex` (for string arrays) or `FuzzyObjectIndex` (for object arrays with keys). They work like a fuse.js instance — build once, search many times:

```typescript
import { FuzzyIndex, FuzzyObjectIndex } from 'rapid-fuzzy';

// String search — replaces new Fuse(strings).search(query)
const index = new FuzzyIndex(strings);
const results = index.search('query');

// Object search — replaces new Fuse(objects, { keys }).search(query)
const objIndex = new FuzzyObjectIndex(users, { keys: ['name', 'email'] });
const userResults = objIndex.search('john');
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
