---
"rapid-fuzzy": patch
---

Fixes found while reviewing the release candidate:

- WebAssembly build (browsers, Deno, Workers): `maxResults` is read like in the Node.js build. `Infinity` means no limit (it used to throw `TypeError: ... expected u32`), values beyond 2^32 are capped instead of throwing, and NaN, negative and fractional values throw a `TypeError` saying that `maxResults must be a non-negative integer or Infinity`. This applies to `search`, `searchKeys`, `searchObjects` and every index's `search` / `searchIndices`, in both the options object and the numeric shorthand.
- WebAssembly build: `KeyedFuzzyIndex.search(query, n)` and therefore `FuzzyObjectIndex.search(query, n)` accept the numeric `maxResults` shorthand, as declared and as in the Node.js build; they used to throw `Invalid SearchOptions`.
- WebAssembly build: `FuzzyIndex.fromAsync()` with invalid items (such as an array containing a non-string) returns a Promise rejected with a `TypeError` instead of throwing synchronously.
