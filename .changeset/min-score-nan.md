---
"rapid-fuzzy": patch
---

A `NaN` `minScore` throws instead of silently filtering out every match, like a `NaN` `maxResults` or `minSimilarity` already does: in `search()`, `closest()`, `FuzzyIndex` (`search()`, `searchIndices()`, `closest()`), `searchKeys()`, `KeyedFuzzyIndex` (`search()`, `closest()` with an options object or the numeric shorthand), `searchObjects()` and `FuzzyObjectIndex`, in every build (an `InvalidArg` error in Node.js and a `TypeError` in the browser build, with the message `minScore must be a number, got NaN`). No score compares as at least `NaN`, so such a call always returned no results (or `null`); a threshold computed from bad input, such as `parseFloat('')`, now fails loudly. Every other number, including `±Infinity` and values above 1, is accepted as before.
