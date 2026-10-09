---
"rapid-fuzzy": minor
---

Add a `matchMode` option to multi-key search (`searchObjects()`, `FuzzyObjectIndex`, `searchKeys()`, `KeyedFuzzyIndex`; Node.js and WebAssembly builds) so that the terms of a query can match different keys (#782):

- `'perKey'` (the default, unchanged): every key is matched against the whole query, so `john tokyo` only finds items with a key containing both terms, and a `!term` only zeroes the key that contains it.
- `'crossKey'`: every term is matched against the keys on its own. Each term must match at least one key, and a `!term` matching any key excludes the item. A key's score (`keyScores`) is the share of the query it matches, and `scoreMode` combines these scores. With `'max'`, an item whose every term matches some key perfectly scores 1; with `'matched'`, only an item whose every term matches perfectly each key it matches does (a term that also matches another key partially lowers the score).

```typescript
searchObjects('john tokyo', [{ name: 'John Smith', city: 'Tokyo' }], {
  keys: ['name', 'city'],
  matchMode: 'crossKey',
}); // → John Smith (previously no result)
index.closest('john tokyo', { scoreMode: 'max', matchMode: 'crossKey' });
```

A query of a single term without `!term`s gives the same results in both modes, keys with weight 0 never select or exclude items, and `KeyedFuzzyIndex` returns exactly what `searchKeys()` returns. A `matchMode` other than the two modes throws (an `InvalidArg` error in Node.js, a `TypeError` in the WebAssembly build). TypeScript: the new `KeyMatchMode` type is `'perKey' | 'crossKey'`, and `KeySearchOptions`, `KeyClosestOptions` and `ObjectIndexSearchOptions` gain `matchMode`.
