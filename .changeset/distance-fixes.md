---
"rapid-fuzzy": patch
---

Fix several distance / similarity bugs and make every entry point (single, `*Batch`, `*Many`, `*ManyU32` / `*ManyF64`) return identical results, in both the Node.js and the browser (wasm-bindgen) build.

Behavior changes:

- `sorensenDice`, `sorensenDiceBatch` and `sorensenDiceMany` count bigrams in characters instead of UTF-8 bytes, so non-ASCII input scores correctly: `sorensenDice('日本', '日本人')` is now `0.667` (was `0.154`), and `'😀😃'` vs `'😀😃😄'` is `0.667` (was `0.111`). Results for ASCII input are unchanged, and whitespace is still ignored. Scores now match `string-similarity`'s `compareTwoStrings` on typical text such as accented Latin or CJK.
- A `*Many` similarity function keeps a candidate whose score is exactly `minSimilarity`. `jaroWinklerMany` dropped about 1% of such candidates to `0` because of a rounding error in its early-termination cutoff (e.g. `jaroWinklerMany('aaac dc ', ['aa'], 0.8)` returned `[0]` although `jaroWinkler` is `0.8`).
- `tokenSetRatio` (and so `weightedRatio`, and their `*Batch` / `*Many` variants) no longer gives strings without a shared token a score boosted by a stray leading space; it now compares the sorted tokens directly. `tokenSetRatio('cat', 'dog')` is now `0` (was `0.25`).
- `*Batch` functions throw an `InvalidArg` error (`Error` in the browser build) when a pair does not hold exactly two strings. They used to return `0` (an "identical" distance), `0.0` or `null` for short pairs and silently ignore extra strings. The browser build also throws for a non-array argument instead of returning `[]`.
- `*Many` similarity functions throw an `InvalidArg` error (`Error` in the browser build) for a `NaN` `minSimilarity`, which used to filter everything in some functions and nothing in others.
- `*Many` similarity functions apply `minSimilarity` the same way everywhere: a threshold above `1` filters every candidate, including exact matches that `tokenSetRatioMany`, `partialRatioMany` and `weightedRatioMany` used to return as `1`.
- `*Many` distance functions no longer overflow the "exceeds `maxDistance`" sentinel for `maxDistance = 4294967295` (negative values wrap to it); the sentinel is capped at `4294967295`.

Performance (same results):

- `*ManyU32` / `*ManyF64` build their typed arrays natively instead of copying a JS array, making them roughly 1.5–2x faster than before (`levenshteinManyU32` with 100,000 candidates: 32 ms → 21 ms) and faster than the plain `*Many` functions, with far fewer garbage collections. The returned arrays still own ordinary, transferable buffers.
- `weightedRatio` / `weightedRatioMany` skip or prune the expensive partial-ratio scan when another component already reaches the best score (`weightedRatio` on two English texts of 2,000 and 4,000 characters: ~290 ms → ~2 ms).
- `sorensenDiceMany` no longer allocates per candidate (~2x faster); `tokenSortRatioMany`, `partialRatioMany` and `weightedRatioMany` reuse the reference's precomputed comparator and lower-case without allocating for ASCII words (~1.25–1.5x faster).
