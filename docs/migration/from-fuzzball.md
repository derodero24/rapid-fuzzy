# Migrating from fuzzball to rapid-fuzzy

[fuzzball](https://www.npmjs.com/package/fuzzball) is a JavaScript port of Python's fuzzywuzzy / RapidFuzz, providing ratio-based string matching with token sort/set algorithms. rapid-fuzzy provides similar ratio functions on a 0.0–1.0 scale, plus edit distances and fuzzy search, powered by a Rust core. **Several functions are computed differently, so scores and thresholds do not carry over unchanged** — see [Score differences](#score-differences).

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
| `fuzz.ratio(a, b, { full_process: false })` | `normalizedIndel(a, b)` | Same measure, 0.0–1.0 instead of 0–100 (fuzzball rounds to an integer) |
| `fuzz.ratio(a, b)` | `normalizedIndel(fullProcess(a), fullProcess(b))` | fuzzball lower-cases and strips punctuation by default; rapid-fuzzy does not (see below) |
| `fuzz.token_sort_ratio(a, b)` | `tokenSortRatio(a, b)` | Built on normalized Levenshtein, not Indel: scores differ |
| `fuzz.token_set_ratio(a, b)` | `tokenSetRatio(a, b)` | Built on normalized Levenshtein, not Indel: scores differ |
| `fuzz.partial_ratio(a, b)` | `partialRatio(a, b)` | Different alignment: scores differ |
| `fuzz.WRatio(a, b)` | `weightedRatio(a, b)` | Plain maximum of the ratios, without WRatio's length-based scaling: scores differ, often higher |
| `fuzz.extract(query, choices)` | `normalizedIndelMany(query, choices)` (or another `*Many`) + sort | `search()` is a different kind of matching (see below) |

`normalizedLevenshtein` is **not** the same as `fuzz.ratio`: it divides the Levenshtein distance by the longer length, while `fuzz.ratio` uses the Indel distance (a substitution counts as two edits) over the sum of the lengths. For `kitten` / `sitting`, `fuzz.ratio` is 62, `normalizedIndel` 0.615 and `normalizedLevenshtein` 0.571.

> **Score scale difference**: fuzzball returns integers 0–100, rapid-fuzzy returns floats 0.0–1.0. Multiply by 100 (and round) for the old scale, but re-check thresholds for the functions whose scores differ.

### Preprocessing

By default fuzzball runs `full_process` on its input: it lower-cases it, replaces non-alphanumeric characters with spaces and trims it. rapid-fuzzy's token functions (`tokenSortRatio`, `tokenSetRatio`, `partialRatio`, `weightedRatio`) lower-case and split on whitespace but keep punctuation; the distance functions (`normalizedIndel`, `normalizedLevenshtein`, …) use the strings as given. To reproduce fuzzball's default processing, apply your own processor first:

```typescript
// Lower-case, replace non-alphanumerics (Unicode letters and digits kept) with spaces, trim
const fullProcess = (s: string) => s.toLowerCase().replace(/[^\p{L}\p{N}]+/gu, ' ').trim();
normalizedIndel(fullProcess('New York'), fullProcess('new york!')); // 1.0, like fuzz.ratio's 100
```

## Code Examples

### Basic ratio

```typescript
// Before (fuzzball)
import * as fuzz from 'fuzzball';
fuzz.ratio('hello', 'hello');    // 100
fuzz.ratio('kitten', 'sitting'); // 62

// After (rapid-fuzzy)
import { normalizedIndel } from 'rapid-fuzzy';
normalizedIndel('hello', 'hello');    // 1.0
normalizedIndel('kitten', 'sitting'); // 0.615
```

### Token-based matching

```typescript
// Before (fuzzball)
import * as fuzz from 'fuzzball';
fuzz.token_sort_ratio('New York Mets', 'Mets New York'); // 100
fuzz.token_set_ratio('hello', 'hello world');           // 100

// After (rapid-fuzzy)
import { tokenSortRatio, tokenSetRatio } from 'rapid-fuzzy';
tokenSortRatio('New York Mets', 'Mets New York'); // 1.0
tokenSetRatio('hello', 'hello world');            // 1.0
```

### Weighted ratio

```typescript
// Before (fuzzball)
import * as fuzz from 'fuzzball';
fuzz.WRatio('hello world', 'world hello'); // 95 (token ratios are scaled by 0.95)

// After (rapid-fuzzy)
import { weightedRatio } from 'rapid-fuzzy';
weightedRatio('hello world', 'world hello'); // 1.0 (no scaling)
```

### Scoring a list of choices

```typescript
// Before (fuzzball) — scores every choice with fuzz.ratio (after full_process)
import * as fuzz from 'fuzzball';
fuzz.extract('python', ['Python', 'JavaScript', 'TypeScript']);
// [['Python', 100, 0], ['JavaScript', 25, 1], ['TypeScript', 25, 2]]

// After (rapid-fuzzy) — score every choice, then sort
import { normalizedIndelMany } from 'rapid-fuzzy';
const choices = ['Python', 'JavaScript', 'TypeScript'];
const scores = normalizedIndelMany('python', choices.map((c) => c.toLowerCase()));
const ranked = choices
  .map((choice, index) => ({ choice, score: scores[index], index }))
  .sort((a, b) => b.score - a.score);
// [{ choice: 'Python', score: 1, index: 0 }, { choice: 'JavaScript', score: 0.25, … }, …]
```

rapid-fuzzy's `search()` is a different tool: an interactive fuzzy search that only returns items containing the query's characters in order (`search('python', choices)` returns only `Python`), ranked by the nucleo matcher. It is the better choice for search-as-you-type, while the `*Many` ratio functions are the equivalent of `extract` for scoring every choice.

## Score Differences

Scores for some example pairs (fuzzball 2.2 with its defaults vs rapid-fuzzy × 100):

| Pair | `token_sort_ratio` / `tokenSortRatio` | `token_set_ratio` / `tokenSetRatio` | `partial_ratio` / `partialRatio` | `WRatio` / `weightedRatio` |
|---|---:|---:|---:|---:|
| `kitten` / `sitting` | 62 / 57.1 | 62 / 57.1 | 67 / 66.7 | 62 / 66.7 |
| `fuzzy wuzzy was a bear` / `wuzzy fuzzy was a bear` | 100 / 100 | 100 / 100 | 91 / 90.9 | 95 / 100 |
| `New York Yankees` / `Yankees` | 61 / 43.8 | 100 / 100 | 100 / 100 | 90 / 100 |
| `Mariners vs Angels` / `Los Angeles Angels of Anaheim at Seattle Mariners` | 51 / 34.7 | 91 / 83.3 | 62 / 44.4 | 86 / 83.3 |
| `Microsoft Corporation` / `MSFT` | 32 / 19 | 32 / 19 | 75 / 50 | 68 / 50 |

The token functions agree when the token sets match exactly and differ otherwise, because rapid-fuzzy compares with normalized Levenshtein. `partialRatio` compares the shorter string with every window of the same length in the longer one, which suits substrings and truncations (`New York Yankees` / `Yankees`) but not abbreviations (`MSFT`). `weightedRatio` takes the plain maximum of its components, without WRatio's length-ratio rules and 0.9 / 0.95 scaling.

## What You Gain

### Batch APIs

Process multiple comparisons in a single call:

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

For search-as-you-type over the same items, build a `FuzzyIndex` once:

```typescript
import { FuzzyIndex } from 'rapid-fuzzy';
const index = new FuzzyIndex(items);
index.search('python');   // ranked fuzzy search results
index.closest('python');  // best single match, or null
```

### Performance

From the [README benchmarks](../../README.md#benchmarks) (Node.js 22, Linux x64, Intel Xeon @ 2.10GHz; 6 string pairs per operation): `tokenSortRatio` 209,369 ops/s vs `token_sort_ratio` 51,588; `tokenSetRatio` 115,354 vs `token_set_ratio` 36,665; `weightedRatio` 50,289 vs `WRatio` 11,668. As shown above, the functions do not compute identical scores.

### TypeScript support

rapid-fuzzy ships with built-in TypeScript declarations. No `@types/` package needed.

## Key Differences

- **Score scale**: fuzzball returns 0–100 (integer); rapid-fuzzy returns 0.0–1.0 (float).
- **Preprocessing**: fuzzball strips punctuation and lower-cases by default; rapid-fuzzy's token functions lower-case but keep punctuation, and its distance functions use the input as given.
- **Token, partial and weighted ratios** are computed differently and can score differently (see [Score Differences](#score-differences)).
- **`extract`**: score choices with a `*Many` function; `search()` is a fuzzy search with different matching rules. `search()` supports `maxResults` and `minScore` options.
