# rapid-fuzzy

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
