# rapid-fuzzy

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
