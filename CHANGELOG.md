# rapid-fuzzy

## 2.1.1

### Patch Changes

- ec2c8e1: Fix `FuzzyIndex` returning too few results after a thresholded search. The incremental cache reused the candidates left over from a previous `minScore` search or `closest()` call, so a longer query typed afterwards was limited to the items that had passed the earlier threshold. The cache now only stores unfiltered matches, and it is not reused across a change of case mode (`isCaseSensitive`).
- b4d0546: Fix `FuzzyIndex` and `KeyedFuzzyIndex` dropping matches on accented text. The standalone `search()` folds diacritics (`cafe` matches `café`), but the index prefilters compared raw characters and rejected those items before scoring, so `new FuzzyIndex(['café']).search('cafe')` returned nothing and indexes over 5000 items lost most folded matches. The prefilters now normalize characters the same way the matcher does.
- eb1b247: Update the napi-rs toolchain (`napi` 3.12, `@napi-rs/cli` 3.9, emnapi 2.0) and align the `rapid-fuzzy-wasm32-wasi` fallback package with the current napi-rs layout:
  
  - It now declares exact `@emnapi/core` / `@emnapi/runtime` dependencies matching the runtime the WASM binary was built against, instead of relying on peer resolution.
  - It ships its own type definitions (`rapid-fuzzy.wasi.d.cts`).
  - It no longer carries a `cpu: ["wasm32"]` restriction, so it can be installed on any platform. It is not an optional dependency of `rapid-fuzzy`: on platforms without a prebuilt native binary, install `rapid-fuzzy-wasm32-wasi` explicitly and the loader falls back to it.
  - Its `engines.node` range follows the WASI API requirements (`>=22.13.0 <23.0.0-0 || >=23.5.0`).
  
  The platform package manifests also pick up the current package description, keywords, and bug tracker URL.
- 0b55a12: `sorensenDiceMany` now returns the same scores as `sorensenDice` and `sorensenDiceBatch`. The many variant used its own bigram implementation, so it disagreed with the single-pair function on whitespace (`'a b'` vs `'ab'`), inputs shorter than two characters, and multi-byte characters.
- e63f755: `weightedRatio`, `weightedRatioBatch` and `weightedRatioMany` now return identical scores. The plain-ratio component was computed on the original strings by the single-pair function but on the normalized (lower-cased, whitespace-collapsed) strings by the many variant; all three now take the better of the two.
- e6642b3: Treat whitespace-only queries as empty in `search`, `closest`, `searchKeys`, `FuzzyIndex` and `KeyedFuzzyIndex`. A query such as `' '` used to return every item with a score of 1.0 (a pattern with no atoms scored 0 and normalized to NaN); it now returns no results unless `returnAllOnEmpty` is set, matching the behavior of the empty string.

## 2.1.0

### Minor Changes

- 49a4ca7: Add `hammingManyU32` and `normalizedHammingManyF64` TypedArray variants, completing the `*Many` TypedArray family. Because `hammingMany`/`normalizedHammingMany` return `null` for length mismatches or candidates filtered by the threshold, the typed arrays use a documented sentinel for those slots: `0xffffffff` for `hammingManyU32` and `NaN` for `normalizedHammingManyF64`.

  Also enrich type documentation: field-level JSDoc on `KeyConfig` and `ObjectSearchResult` (and a clearer note on why `includePositions` has no effect for multi-key search), and document the `number` (maxResults shorthand) argument on `FuzzyIndex.search()` and `searchIndices()`.

## 2.0.0

### Major Changes

- cf53aa4: Drop Node.js 20 support. The minimum required Node.js version is now 22.0.0.

  Node.js 20 reached End-of-Life on 2026-04-30. Consumers running on Node 20
  must upgrade to Node 22+ before pulling in this release.

## 1.2.0

### Minor Changes

- 560bf4d: Add `indel`, `indelBatch`, `indelMany`, `normalizedIndel`, `normalizedIndelBatch`, `normalizedIndelMany` (insertion-deletion distance) and `normalizedHamming`, `normalizedHammingBatch`, `normalizedHammingMany` (normalized Hamming similarity) functions.
- d6a5a57: Add `FuzzyIndex.fromAsync(items)` static factory that constructs the index on the libuv thread pool, returning `Promise<FuzzyIndex>`. For large datasets this keeps the JavaScript event loop unblocked during index construction — useful in Next.js API routes, Nuxt server handlers, and other environments where blocking is a concern.
- 1ec91c9: Add `closest()`, `serialize()`, and `deserialize()` to `KeyedFuzzyIndex`, bringing it to API parity with `FuzzyIndex`.
- 0af7da3: Add optional `minSimilarity` threshold parameter to `sorensenDiceMany`, `tokenSortRatioMany`, `tokenSetRatioMany`, `partialRatioMany`, and `weightedRatioMany`. Candidates scoring below the threshold return `0.0`.

  Also optimizes `sorensenDiceMany` by pre-computing reference bigrams once and reusing them across all candidates, matching the pattern of other `*Many` functions.

- 6487992: Add `serialize()` and `static deserialize()` to `FuzzyObjectIndex`, enabling SSR/SSG pre-building patterns where the index is constructed at build time and shipped as a binary blob for fast client-side initialization.
- a4d79e6: Add `*ManyU32` and `*ManyF64` typed-array variants for all `*Many` distance functions. These return `Uint32Array` or `Float64Array` instead of `Array<number>`, reducing GC pressure for large candidate sets (1000+ items).

  **New `Uint32Array` variants:** `levenshteinManyU32`, `damerauLevenshteinManyU32`, `indelManyU32`

  **New `Float64Array` variants:** `jaroManyF64`, `jaroWinklerManyF64`, `sorensenDiceManyF64`, `normalizedLevenshteinManyF64`, `normalizedIndelManyF64`, `tokenSortRatioManyF64`, `tokenSetRatioManyF64`, `partialRatioManyF64`, `weightedRatioManyF64`

  All variants accept the same parameters as their `*Many` counterparts.

### Patch Changes

- 7ae4ff2: Fix mutation ordering in `objects.js` where the JS items array was mutated before Rust operations completed, which could cause state divergence if the Rust call threw an error.

## 1.1.1

### Patch Changes

- 1895bc6: Fix TypeScript `node16`/`nodenext` ESM resolution for `rapid-fuzzy/highlight` and `rapid-fuzzy/objects` subpath exports by adding proper `.d.mts` type declarations and fixing import specifiers in `index.d.mts`.

## 1.1.0

### Minor Changes

- 077c83a: Add Hamming distance functions (`hamming`, `hammingBatch`, `hammingMany`) for comparing equal-length strings.
- 0ea6ae9: Add optional threshold parameters to `_many` distance functions for early termination. `levenshteinMany` and `damerauLevenshteinMany` accept `maxDistance`, while `jaroMany`, `jaroWinklerMany`, and `normalizedLevenshteinMany` accept `minSimilarity`.
- bfeb33b: Add `searchIndices()` method to FuzzyIndex that returns only indices and scores without cloning item strings, reducing GC pressure for large datasets.

### Patch Changes

- 29a340b: Add bigram inverted index pre-filtering to FuzzyIndex for improved search performance on large datasets (5K+ items). Reduces the number of candidates passed to the scoring function by filtering items that lack query character adjacency patterns.
- c10a29e: Fix `tokenSetRatio` returning 1.0 (perfect match) when comparing an empty string against a non-empty string. Now correctly returns 0.0.
- d1bf5f5: Optimize KeyedFuzzyIndex search with zero-weight key skipping, early exit on partial score upper bound, and per-key character-presence pre-filtering.
- dc578d4: Reduce redundant string normalization in weighted_ratio by sharing pre-normalized strings across sub-algorithms.
- 90d0cc2: Replace unwrap() with proper error propagation in FuzzyIndex deserialization
- 530c0d3: Optimize token-based \_many distance functions by pre-computing reference string normalization and tokenization once instead of per-candidate.

## 1.0.0

### Major Changes

- 649e1f6: First stable release. The public API is now considered stable and will follow semver strictly.

  Key highlights since 0.x:

  - 9 string distance algorithms with batch and many variants
  - Fuzzy search powered by nucleo with query syntax (exclude, prefix, suffix, exact)
  - FuzzyIndex and FuzzyObjectIndex for persistent Rust-side indexing with incremental cache
  - Object search with weighted keys (searchObjects, FuzzyObjectIndex)
  - Match highlighting utilities (highlight, highlightRanges)
  - Subpath exports for tree-shaking (rapid-fuzzy/highlight, rapid-fuzzy/objects)
  - Node.js native (napi-rs) + WASM (browser/Deno/Bun) dual distribution
  - Full TypeScript support with auto-generated type definitions

### Patch Changes

- 94bbcab: Migrate Jaro and Jaro-Winkler from strsim to rapidfuzz bit-parallel implementation for ~2.7x faster single-pair and ~1.9x faster batch comparisons
- ce08337: Add weight and key length validation to KeyedFuzzyIndex constructor to prevent panics during search

## 0.6.0

### Minor Changes

- 541c6a6: Add `serialize()` and `FuzzyIndex.deserialize()` for prebuilt index persistence

### Patch Changes

- b48e501: Add character-presence pre-filter to FuzzyIndex to skip non-matching items before scoring, reducing search time by ~2x
- ac0fcd8: Add incremental search cache to FuzzyIndex for faster keystroke-by-keystroke autocomplete
- d17701f: Validate key count in KeyedFuzzyIndex.add() and addMany() to prevent index corruption and Node.js crashes
- 9376569: Add uFuzzy competitor and 50K dataset to search benchmarks
- 38dd449: Reuse Matcher via thread-local storage in standalone search and closest to avoid per-call allocation overhead
- 86d6bcd: Use quickselect for top-k selection and add length-based tiebreaker for better ranking differentiation

## 0.5.0

### Minor Changes

- 3de4f98: Add `FuzzyObjectIndex` class for persistent indexed search over object collections with weighted keys
- db3d8ea: Adopt bit-parallel algorithm (Myers' method) for Levenshtein and Damerau-Levenshtein distance functions via the `rapidfuzz` crate, significantly improving performance for string distance computations

### Patch Changes

- 5695bb9: Add `positions` field to `ObjectSearchResult` type for match position tracking
- 7b11d57: Use score-based early termination in partial_ratio sliding window to skip windows that cannot beat the current best score
- e5a6a27: Optimize search result construction with two-pass scoring to reduce heap allocations
- 20d1aba: Replace duplicated highlight.mjs implementation with ESM re-export from highlight.js to eliminate manual sync requirement

## 0.4.0

### Minor Changes

- 0744dd7: Add `isCaseSensitive` option to `SearchOptions` for explicit control over case-sensitive matching
- 5bd6b95: Add `searchKeys` function for multi-key weighted fuzzy search
- b120905: Add `searchObjects` function for ergonomic object array search with weighted keys

## 0.3.0

### Minor Changes

- Add score threshold filtering to search() and closest() functions.

  search() now accepts a SearchOptions object with maxResults and minScore fields, while maintaining backward compatibility with the existing number argument for maxResults. closest() accepts an optional minScore parameter to return null when the best match is below the threshold.

- Add token-based matching algorithms inspired by Python's RapidFuzz.

  Four new similarity functions: tokenSortRatio (order-independent via sorted tokens), tokenSetRatio (set intersection-based), partialRatio (best substring match via sliding window), and weightedRatio (maximum across all methods). Each includes batch and many variants for efficient bulk comparisons.

- Add match highlight positions to search results.

  SearchResult now includes a `positions` field containing indices of matched characters. Enable by setting `includePositions: true` in SearchOptions. Positions are computed via nucleo-matcher's indices API, sorted and deduplicated. When not requested, positions is an empty array with zero overhead.

- Add FuzzyIndex class for persistent indexed search.

  A Rust-backed class that holds items in memory, eliminating repeated FFI overhead for applications searching the same dataset multiple times. Supports search with all existing options (maxResults, minScore, includePositions), closest match, and incremental updates via add/addMany/remove methods.

### Patch Changes

- Optimize search performance by reusing UTF-32 conversion buffers across items and switching to unstable sort. Reduces allocations in the hot scoring loop, yielding ~30% improvement on medium-sized datasets (1K items).

## 0.2.0

### Minor Changes

- d279d4b: Normalize search scores to 0.0-1.0 range

  The `score` field in `SearchResult` is now a normalized float between 0.0 (weakest match) and 1.0 (perfect/exact match), instead of a raw integer from the underlying matcher. This makes scores intuitive, self-documenting, and consistent with the distance functions (`normalizedLevenshtein`, `sorensenDice`, etc.) that already return 0.0-1.0 values.

  **Breaking change**: `SearchResult.score` changed from integer to float. Since the project is pre-v1.0, this is a minor version bump.

## 0.1.1

### Patch Changes

- f603232: Fix npm package missing index.js, index.d.ts, and browser.js

  These napi-rs loader files were incorrectly gitignored, causing them to be excluded from the published npm package. CJS `require('rapid-fuzzy')` now works correctly.

## 0.1.0

### Minor Changes

- cc14981: Add batch distance computation API

  Add `*Batch` and `*Many` variants for all six distance functions to reduce FFI overhead when processing multiple string pairs in a single call:

  - `levenshteinBatch` / `levenshteinMany`
  - `damerauLevenshteinBatch` / `damerauLevenshteinMany`
  - `jaroBatch` / `jaroMany`
  - `jaroWinklerBatch` / `jaroWinklerMany`
  - `sorensenDiceBatch` / `sorensenDiceMany`
  - `normalizedLevenshteinBatch` / `normalizedLevenshteinMany`

- d60a11c: Add ESM module format support

  Add dual CJS/ESM package exports via `index.mjs` wrapper and conditional `exports` field in `package.json`. Both `import { search } from 'rapid-fuzzy'` and `const { search } = require('rapid-fuzzy')` now work correctly.

- Initial release: Rust-powered fuzzy search and string distance for JavaScript/TypeScript

  Core distance functions powered by strsim:

  - `levenshtein` / `normalizedLevenshtein` / `damerauLevenshtein`
  - `jaro` / `jaroWinkler`
  - `sorensenDice`

  Fuzzy search powered by nucleo-matcher:

  - `search` — ranked fuzzy search with scores and indices
  - `closest` — find the closest match from a list

  Platform support:

  - Node.js native bindings via napi-rs (macOS, Linux, Windows)
  - WASM fallback for browsers, Deno, and Bun

### Patch Changes

- c0bc486: Add property-based testing with proptest for distance and search functions

  Verify mathematical invariants (symmetry, identity, bounded range, triangle inequality) across thousands of random inputs for all distance algorithms and search functions.
