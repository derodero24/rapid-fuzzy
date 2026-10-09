---
"rapid-fuzzy": patch
---

Bring the browser / WebAssembly build (the wasm-bindgen package behind the `browser` export condition) in line with the Node.js binding and the declarations in `index.d.ts`:

- `search`, `FuzzyIndex.search` and `FuzzyIndex.searchIndices` accept a number as `maxResults` shorthand (`search(query, items, 5)`), which previously threw.
- `closest`, `FuzzyIndex.closest` and `KeyedFuzzyIndex.closest` return `null` instead of `undefined` when nothing matches.
- `hammingBatch`, `hammingMany`, `normalizedHammingBatch` and `normalizedHammingMany` use `null` instead of `undefined` for undefined distances.
- Errors are real `Error` objects (`TypeError` for arguments of the wrong type or shape) instead of plain strings, so `instanceof Error` and stack traces work. `searchKeys`, `KeyedFuzzyIndex` and the `*Batch` functions now throw a `TypeError` for malformed array input instead of silently returning an empty result.
- Invalid options no longer leak WebAssembly memory: each rejected call used to leak its already-converted arguments.
- `matchType` is omitted from results (instead of being present as `undefined`) when positions are not requested, as in the Node.js binding.
- Added `FuzzyIndex.fromAsync`. The WebAssembly build has no worker thread, so it builds the index synchronously and returns an already-resolved Promise.
- The browser entry now also exports `MatchType` and the `*ManyU32` / `*ManyF64` typed-array variants declared in `index.d.ts`.
- `rapid-fuzzy-wasm-bindgen.d.mts` uses no `any` type: every `SearchOptions` field is optional, results are typed (`SearchResult`, `IndexSearchResult`, `KeySearchResult`, `MatchType`), parameter names match the Node.js binding (`minScore`, `maxDistance`, `minSimilarity`, `keyTexts`), and `weights` accepts `number[]` as well as `Float64Array`.
