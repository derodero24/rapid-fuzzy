# Migrating from string-similarity to rapid-fuzzy

[string-similarity](https://www.npmjs.com/package/string-similarity) is no longer maintained. rapid-fuzzy provides the same Dice coefficient algorithm (`sorensenDice`) plus five additional algorithms, all powered by a high-performance Rust core.

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
| `compareTwoStrings(a, b)` | `sorensenDice(a, b)` | Same Dice coefficient algorithm |
| `findBestMatch(s, targets).bestMatch.target` | `closest(s, targets)` | Returns the best matching string |
| `findBestMatch(s, targets).ratings` | `sorensenDiceMany(s, targets)` | Returns all similarity scores |

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
import { closest } from 'rapid-fuzzy';
const best = closest('healed', ['sealed', 'healthy', 'help']);
console.log(best); // 'sealed'
```

### Getting all similarity scores

```typescript
// Before (string-similarity)
import stringSimilarity from 'string-similarity';
const result = stringSimilarity.findBestMatch('healed', ['sealed', 'healthy', 'help']);
const ratings = result.ratings.map(r => ({ target: r.target, rating: r.rating }));

// After (rapid-fuzzy)
import { sorensenDiceMany } from 'rapid-fuzzy';
const scores = sorensenDiceMany('healed', ['sealed', 'healthy', 'help']);
// [0.8, 0.4444, 0.4] — scores in the same order as the input array
```

### Batch comparisons

```typescript
// Before (string-similarity) — no batch API, loop required
import stringSimilarity from 'string-similarity';
const pairs = [['healed', 'sealed'], ['hello', 'world']];
const scores = pairs.map(([a, b]) => stringSimilarity.compareTwoStrings(a, b));

// After (rapid-fuzzy) — native batch API
import { sorensenDiceBatch } from 'rapid-fuzzy';
const scores = sorensenDiceBatch([['healed', 'sealed'], ['hello', 'world']]);
// [0.8, 0.0]
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

rapid-fuzzy's `sorensenDice` is **1.8x faster** than string-similarity's `compareTwoStrings` for single-pair comparisons. For bulk operations, the batch API (`sorensenDiceBatch`) provides even greater speedups by amortizing FFI overhead.

| Operation | rapid-fuzzy | string-similarity |
|---|---:|---:|
| Sorensen-Dice (single pair) | **152,317 ops/s** | 86,399 ops/s |

## Key Differences

- **Return values**: Both libraries return 0.0–1.0 similarity scores for Dice coefficient.
- **`findBestMatch` vs `closest`**: `closest()` returns just the string, not a detailed ratings object. Use `sorensenDiceMany()` if you need all scores.
- **TypeScript**: rapid-fuzzy ships with built-in TypeScript declarations. No `@types/` package needed.
- **ESM/CJS**: rapid-fuzzy supports both ESM (`import`) and CommonJS (`require`).
