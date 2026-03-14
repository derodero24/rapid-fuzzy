# rapid-fuzzy

## 0.3.0

### Minor Changes

- Add score threshold filtering to search() and closest() functions.

  search() now accepts a SearchOptions object with maxResults and minScore fields, while maintaining backward compatibility with the existing number argument for maxResults. closest() accepts an optional minScore parameter to return null when the best match is below the threshold.

- Add token-based matching algorithms inspired by Python's RapidFuzz.

  Four new similarity functions: tokenSortRatio (order-independent via sorted tokens), tokenSetRatio (set intersection-based), partialRatio (best substring match via sliding window), and weightedRatio (maximum across all methods). Each includes batch and many variants for efficient bulk comparisons.

- Add match highlight positions to search results.

  SearchResult now includes a `positions` field containing indices of matched characters. Enable by setting `includePositions: true` in SearchOptions. Positions are computed via nucleo-matcher's indices API, sorted and deduplicated. When not requested, positions is an empty array with zero overhead.

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
