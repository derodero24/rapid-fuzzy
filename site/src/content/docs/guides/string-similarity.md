---
title: String similarity
description: Levenshtein, Damerau-Levenshtein, Jaro-Winkler, Sørensen-Dice, Hamming, Indel and RapidFuzz-style token ratios, with batch and typed-array variants.
---

rapid-fuzzy has seven edit-distance and similarity metrics and four token-based ratios. Each comes in three forms: a single comparison, a `*Batch` function for many pairs, and a `*Many` function for one string against many.

## Distance and similarity metrics

```ts
import {
  levenshtein,
  normalizedLevenshtein,
  damerauLevenshtein,
  jaro,
  jaroWinkler,
  sorensenDice,
  hamming,
  normalizedHamming,
  indel,
  normalizedIndel,
} from 'rapid-fuzzy';

levenshtein('kitten', 'sitting'); // 3
normalizedLevenshtein('kitten', 'sitting'); // 0.571 (1 - 3 / 7)
damerauLevenshtein('ab', 'ba'); // 1 (an adjacent transposition is one edit)
jaro('martha', 'marhta'); // 0.944
jaroWinkler('MARTHA', 'MARHTA'); // 0.961 (Jaro plus a bonus for a common prefix)
sorensenDice('night', 'nacht'); // 0.25
hamming('karolin', 'kathrin'); // 3 (null if the lengths differ)
normalizedHamming('karolin', 'kathrin'); // 0.571 (null if the lengths differ)
indel('abc', 'ac'); // 1 (insertions and deletions only)
normalizedIndel('kitten', 'sitting'); // 0.615
```

| Function | Returns |
| --- | --- |
| `levenshtein`, `damerauLevenshtein`, `indel` | The number of edits (an integer) |
| `hamming` | The number of differing positions, or `null` if the lengths differ |
| `normalizedLevenshtein`, `normalizedIndel`, `jaro`, `jaroWinkler`, `sorensenDice` | A similarity from 0 (nothing in common) to 1 (identical) |
| `normalizedHamming` | A similarity from 0 to 1, or `null` if the lengths differ |

These functions compare the Unicode code points of the strings exactly as given. They do not ignore case or whitespace, and they do not normalize Unicode: `levenshtein('Cafe', 'cafe')` is 1, and a precomposed `é` differs from `e` plus a combining accent. Lower-case or [normalize](/rapid-fuzzy/guides/unicode/) the input first if that matters to you. `sorensenDice` removes whitespace and compares character bigrams.

## Token-based ratios

The token-based ratios come from Python's [RapidFuzz](https://github.com/rapidfuzz/RapidFuzz). They lower-case their input and split it on whitespace, so they ignore case and word order:

```ts
import { tokenSortRatio, tokenSetRatio, partialRatio, weightedRatio } from 'rapid-fuzzy';

// Sorts the words before comparing
tokenSortRatio('New York Mets', 'Mets New York'); // 1

// Compares shared and extra words separately
tokenSetRatio('Great Gatsby', 'The Great Gatsby by Fitzgerald'); // 1

// The shorter string against every same-length window of the longer one
partialRatio('hello', 'hello world'); // 1

// The highest of normalizedLevenshtein, tokenSortRatio, tokenSetRatio and partialRatio
weightedRatio('John Smith', 'Smith John'); // 1
```

Scores are on a 0–1 scale. They are not always equal to RapidFuzz's or fuzzball's: `weightedRatio` does not scale by length difference the way `WRatio` does. See the [fuzzball migration guide](/rapid-fuzzy/migration/from-fuzzball/). `partialRatio` and `weightedRatio` are expensive on long strings; see [Performance](/rapid-fuzzy/guides/performance/).

## Many strings at once

Every metric has two bulk variants, which make one call across the JavaScript–Rust boundary instead of one call per pair:

- **`*Many(reference, candidates)`** compares one string against many. It reuses the work that depends only on the reference, which makes it the fastest way to rank candidates.
- **`*Batch(pairs)`** computes the metric for an array of `[a, b]` pairs.

```ts
import { levenshteinBatch, levenshteinMany, jaroWinklerMany } from 'rapid-fuzzy';

levenshteinBatch([
  ['kitten', 'sitting'],
  ['hello', 'help'],
]); // → [3, 2]

levenshteinMany('kitten', ['sitting', 'kittens', 'kitchen']); // → [3, 1, 2]

// A threshold skips candidates that cannot qualify
levenshteinMany('kitten', candidates, 3); // a candidate more than 3 edits away gets 4
jaroWinklerMany('MARTHA', candidates, 0.8); // a candidate below 0.8 gets 0
```

`levenshteinMany` over 1,000 short candidates was about 1.5x faster than calling `levenshtein` in a loop. For a handful of short pairs, `*Batch` is not faster than single calls.

A `*Batch` pair that is not exactly two strings throws, and so does a `NaN` threshold.

### Typed-array results

Each `*Many` function has a variant that returns a typed array instead of `number[]`, which avoids boxing every number and transfers to a worker without copying (`postMessage(result, [result.buffer])`):

- **`*ManyU32`** returns a `Uint32Array` of distances: `levenshteinManyU32`, `damerauLevenshteinManyU32`, `indelManyU32`, `hammingManyU32`.
- **`*ManyF64`** returns a `Float64Array` of similarities: `jaroManyF64`, `jaroWinklerManyF64`, `normalizedLevenshteinManyF64`, `normalizedIndelManyF64`, `normalizedHammingManyF64`, `sorensenDiceManyF64` and the four token ratios.

A typed array cannot hold `null`. `hammingManyU32` therefore returns `0xffffffff`, and `normalizedHammingManyF64` returns `NaN`, for a candidate of a different length or one filtered out by the threshold.

```ts
import { levenshteinManyU32, jaroWinklerManyF64 } from 'rapid-fuzzy';

levenshteinManyU32('kitten', ['sitting', 'kittens', 'kitchen']); // Uint32Array [3, 1, 2]
jaroWinklerManyF64('kitten', ['sitting', 'kittens', 'kitchen']); // Float64Array [0.746, 0.971, 0.894]
```
