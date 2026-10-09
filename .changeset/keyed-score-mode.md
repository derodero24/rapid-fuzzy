---
"rapid-fuzzy": minor
---

Add a `scoreMode` option to multi-key search (`searchObjects()`, `FuzzyObjectIndex`, `searchKeys()`, `KeyedFuzzyIndex`; Node.js and WebAssembly builds) that chooses how the per-key scores are combined (#781):

- `'weighted'` (the default, unchanged): the weighted average over all keys. A key that does not match counts as 0, so an exact match on one of several keys only scores that key's share of the total weight.
- `'matched'`: the weighted average over the keys that match only. An item whose only match is exact scores 1, and is no longer ranked below items that match several keys partially or dropped by `minScore`.
- `'max'`: the highest key score; weights only select the keys that take part.

```typescript
searchObjects('smith', people, { keys: ['name', 'email'], scoreMode: 'matched' });
index.closest('smith', 0.9, 'matched'); // closest(query, minScore?, scoreMode?)
```

`keyScores`, tie-breaking, zero-weight keys and `returnAllOnEmpty` work the same in every mode, and `minScore` and `maxResults` apply to the mode's score. The `KeyedFuzzyIndex` early exit stays exact in every mode: the index returns the same results as `searchKeys()`. A `scoreMode` other than the three modes throws (an `InvalidArg` error in Node.js, a `TypeError` in the WebAssembly build). TypeScript: the new `KeyScoreMode` type is `'weighted' | 'matched' | 'max'`; `searchKeys()` and `KeyedFuzzyIndex.search()` take the new `KeySearchOptions` (`SearchOptions` plus `scoreMode`), which accepts every existing `SearchOptions` value. Plain `search()` and `FuzzyIndex` options are unchanged.
