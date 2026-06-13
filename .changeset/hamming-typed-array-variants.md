---
'rapid-fuzzy': minor
---

Add `hammingManyU32` and `normalizedHammingManyF64` TypedArray variants, completing the `*Many` TypedArray family. Because `hammingMany`/`normalizedHammingMany` return `null` for length mismatches or candidates filtered by the threshold, the typed arrays use a documented sentinel for those slots: `0xffffffff` for `hammingManyU32` and `NaN` for `normalizedHammingManyF64`.

Also enrich type documentation: field-level JSDoc on `KeyConfig` and `ObjectSearchResult` (and a clearer note on why `includePositions` has no effect for multi-key search), and document the `number` (maxResults shorthand) argument on `FuzzyIndex.search()` and `searchIndices()`.
