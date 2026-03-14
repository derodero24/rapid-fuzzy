---
"rapid-fuzzy": minor
---

Add batch distance computation API

Add `*Batch` and `*Many` variants for all six distance functions to reduce FFI overhead when processing multiple string pairs in a single call:

- `levenshteinBatch` / `levenshteinMany`
- `damerauLevenshteinBatch` / `damerauLevenshteinMany`
- `jaroBatch` / `jaroMany`
- `jaroWinklerBatch` / `jaroWinklerMany`
- `sorensenDiceBatch` / `sorensenDiceMany`
- `normalizedLevenshteinBatch` / `normalizedLevenshteinMany`
