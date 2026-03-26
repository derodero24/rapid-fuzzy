---
"rapid-fuzzy": minor
---

Add `*ManyU32` and `*ManyF64` typed-array variants for all `*Many` distance functions. These return `Uint32Array` or `Float64Array` instead of `Array<number>`, reducing GC pressure for large candidate sets (1000+ items).

**New `Uint32Array` variants:** `levenshteinManyU32`, `damerauLevenshteinManyU32`, `indelManyU32`

**New `Float64Array` variants:** `jaroManyF64`, `jaroWinklerManyF64`, `sorensenDiceManyF64`, `normalizedLevenshteinManyF64`, `normalizedIndelManyF64`, `tokenSortRatioManyF64`, `tokenSetRatioManyF64`, `partialRatioManyF64`, `weightedRatioManyF64`

All variants accept the same parameters as their `*Many` counterparts.
