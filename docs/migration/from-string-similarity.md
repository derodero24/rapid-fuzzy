# Migrating from string-similarity to rapid-fuzzy

[string-similarity](https://www.npmjs.com/package/string-similarity) is deprecated on npm and no longer maintained. rapid-fuzzy's `sorensenDice` computes the same Dice coefficient, alongside other distance metrics and fuzzy search, all powered by a Rust core.

## Installation

```bash
# Remove string-similarity
npm uninstall string-similarity

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## API Mapping

| string-similarity | rapid-fuzzy | Notes |
|---|---|---|
| `compareTwoStrings(a, b)` | `sorensenDice(a, b)` | Same Dice coefficient (whitespace removed, case-sensitive) |
| `findBestMatch(s, targets).ratings` | `sorensenDiceMany(s, targets)` | All scores, in the order of `targets` |
| `findBestMatch(s, targets).bestMatch` | `sorensenDiceMany` + pick the maximum | See [Finding the best match](#finding-the-best-match) |

`sorensenDice` returned the same scores as `compareTwoStrings` in our differential tests on text without emoji or other characters outside the Basic Multilingual Plane. For those, the results can differ, because rapid-fuzzy counts Unicode code points and string-similarity counts UTF-16 code units (`'a😀b'` vs `'a😀c'`: 0.5 vs 0.67).

## Code Examples

### Comparing two strings

```typescript
// Before (string-similarity)
import stringSimilarity from 'string-similarity';
const score = stringSimilarity.compareTwoStrings('healed', 'sealed');
// 0.8

// After (rapid-fuzzy)
import { sorensenDice } from 'rapid-fuzzy';
const score = sorensenDice('healed', 'sealed');
// 0.8
```

### Finding the best match

```typescript
// Before (string-similarity)
import stringSimilarity from 'string-similarity';
const result = stringSimilarity.findBestMatch('healed', ['sealed', 'healthy', 'help']);
console.log(result.bestMatch.target); // 'sealed'
console.log(result.bestMatch.rating); // 0.8

// After (rapid-fuzzy)
import { sorensenDiceMany } from 'rapid-fuzzy';
const targets = ['sealed', 'healthy', 'help'];
const ratings = sorensenDiceMany('healed', targets); // [0.8, 0.5455, 0.25]
const bestIndex = ratings.indexOf(Math.max(...ratings));
console.log(targets[bestIndex]); // 'sealed'
console.log(ratings[bestIndex]); // 0.8
```

> **Do not replace `findBestMatch` with `closest()`.** rapid-fuzzy's `closest()` is a fuzzy *search*: it returns an item that contains the query's characters in order, and `null` if there is none. `closest('healed', ['sealed', 'healthy', 'help'])` returns `null`.

### Getting all similarity scores

```typescript
// Before (string-similarity)
import stringSimilarity from 'string-similarity';
const result = stringSimilarity.findBestMatch('healed', ['sealed', 'healthy', 'help']);
const ratings = result.ratings.map((r) => r.rating);

// After (rapid-fuzzy)
import { sorensenDiceMany } from 'rapid-fuzzy';
const scores = sorensenDiceMany('healed', ['sealed', 'healthy', 'help']);
// [0.8, 0.5455, 0.25] — scores in the same order as the input array
```

### Batch comparisons

```typescript
// Before (string-similarity) — no batch API, loop required
import stringSimilarity from 'string-similarity';
const pairs = [['healed', 'sealed'], ['hello', 'world']];
const scores = pairs.map(([a, b]) => stringSimilarity.compareTwoStrings(a, b));

// After (rapid-fuzzy) — batch API
import { sorensenDiceBatch } from 'rapid-fuzzy';
const scores = sorensenDiceBatch([['healed', 'sealed'], ['hello', 'world']]);
// [0.8, 0]
```

## Additional Algorithms

rapid-fuzzy provides algorithms that string-similarity does not:

| Algorithm | Function | Best for |
|---|---|---|
| Levenshtein distance | `levenshtein(a, b)` | Typo detection, spell checking |
| Normalized Levenshtein | `normalizedLevenshtein(a, b)` | Length-independent comparison |
| Jaro-Winkler | `jaroWinkler(a, b)` | Name / address matching |
| Damerau-Levenshtein | `damerauLevenshtein(a, b)` | Transposition-aware edit distance |
| Fuzzy search | `search(query, items)` | Interactive search / autocomplete |

## Performance

From the [README benchmarks](../../README.md#benchmarks) (Node.js 22, Linux x64, Intel Xeon @ 2.10GHz; 6 string pairs per operation):

| Operation | rapid-fuzzy | string-similarity |
|---|---:|---:|
| Sorensen-Dice (single pairs) | **178,705 ops/s** | 56,320 ops/s |

`sorensenDice` was about 3x faster than `compareTwoStrings`. `sorensenDiceMany` prepares the reference string once for all candidates.

## Key Differences

- **Return values**: Both libraries return 0.0–1.0 similarity scores for the Dice coefficient.
- **`findBestMatch`**: rapid-fuzzy has no single equivalent; use `sorensenDiceMany()` and pick the maximum, as shown above.
- **TypeScript**: rapid-fuzzy ships with built-in TypeScript declarations. No `@types/` package needed.
- **ESM/CJS**: rapid-fuzzy supports both ESM (`import`) and CommonJS (`require`).
