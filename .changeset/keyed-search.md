---
"rapid-fuzzy": patch
---

Make `KeyedFuzzyIndex` (and `FuzzyObjectIndex`) return exactly the same results as `searchKeys()` (and `searchObjects()`), and fix several multi-key search bugs. Both now run the same search routine.

- `KeyedFuzzyIndex` no longer drops items whose combined score equals `minScore` exactly. Its early exit compared against a running float subtraction, so for example 5 keys weighted `[0.494, 0.953, 0.137, 0.447, 0.673]` that all match `foo` perfectly returned nothing with `minScore: 1`.
- Keys with a weight of `0` are handled the same way everywhere: they are scored and reported in `keyScores` (the index used to report 0 for them), but they never select an item on their own. `searchKeys()` used to return items that matched only a zero-weight key, with a score of 0; it no longer does. In general, items whose combined score is 0 are not returned.
- Equal scores are ordered like `search()` orders them: the item whose best-matching key text is shorter comes first, then the lower index. Previously both multi-key paths ordered ties by index only, so `foobar` could rank above an exact `foo`.
- Queries are handled like in `search()`: ideographic spaces (U+3000), no-break spaces, tabs and newlines separate terms; queries made only of syntax characters (`^`, `!`, `$`, `'`) are treated as empty (no results, or every item with `returnAllOnEmpty`); and a query term longer than 2,520 characters matches nothing instead of producing wrapped-around scores.
- `searchKeys()` now throws an `InvalidArg` error, with the same message as the `KeyedFuzzyIndex` constructor, when the key text arrays have different lengths, the number of weights differs from the number of keys, a weight is negative, `NaN` or infinite, or the weights sum to 0. It used to silently return `[]`. `searchObjects()` throws for invalid weights accordingly. In the browser (wasm) build, `searchKeys()` throws a real `Error` for these cases and for key texts that are not arrays of strings.
- Weights whose sum overflows to `Infinity` (e.g. two weights of `Number.MAX_VALUE`) are rejected by `searchKeys()`, `searchObjects()`, `KeyedFuzzyIndex`, `FuzzyObjectIndex` and their `deserialize()`; previously every score became 0.
- `searchKeys()` accepts a number as its fourth argument as a shorthand for `{ maxResults }`, like `search()`, in both the Node.js and the browser builds; invalid numbers (`NaN`, negative, fractional) throw.
- Performance: `KeyedFuzzyIndex.search()` no longer allocates and zeroes a keys × items score matrix on every search and skips items whose keys all fail the character prefilter, making searches that match few items about 3x faster on 100k items × 3 keys. `searchKeys()` reuses its matcher between calls (about 2x faster for small inputs).
