---
"rapid-fuzzy": patch
---

The Node.js declarations match what the binding accepts and returns. `FuzzyIndex.deserialize()` and `KeyedFuzzyIndex.deserialize()` are declared to take any `Uint8Array` or a `Buffer` (`Uint8Array | NodeBuffer`: with older `@types/node` releases, TypeScript 5.7+ does not treat a `Buffer` as a `Uint8Array`), so data read from `fetch()`, IndexedDB or the browser build no longer needs a cast or a copy into a `Buffer` under strict TypeScript; the binding always accepted it. `hammingBatch`, `hammingMany`, `normalizedHammingBatch` and `normalizedHammingMany` return `Array<number | null>` instead of `Array<number | undefined | null>`: they never return `undefined`, and the browser declarations already said `(number | null)[]`, so shared Node.js and browser code type-checks the same way.
