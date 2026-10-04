---
"rapid-fuzzy": minor
---

Harden the object-search wrappers, `KeyedFuzzyIndex` and the TypeScript declarations.

**New**

- `require('rapid-fuzzy')` now exposes `searchObjects` and `FuzzyObjectIndex`, like the ESM entry already did (they are loaded on first access).
- `KeyedFuzzyIndex.search()` and `FuzzyObjectIndex.search()` accept a number as a shorthand for `maxResults`, like `FuzzyIndex.search()`. Other non-object values passed as options to `FuzzyObjectIndex.search()` throw a `TypeError` instead of being ignored.
- `highlight()` takes an optional last argument `{ escapeHtml: true }` that HTML-escapes the item text (matched and unmatched parts) before adding the markers. `highlight()` still does not escape anything by default.
- `FuzzyObjectIndex.deserialize()` accepts any `Uint8Array` / `ArrayBufferView` or an `ArrayBuffer`, not only a Node.js `Buffer`.

**Fixes**

- `FuzzyObjectIndex.remove()` throws a `RangeError` for a non-integer index (`NaN`, `1.5`, `Infinity`) and a `TypeError` for a non-number. Previously `remove(NaN)` or `remove(1.5)` removed a different item natively than in the JavaScript item list, so later searches returned the wrong objects.
- `FuzzyObjectIndex.addMany()`, `KeyedFuzzyIndex.addMany()` (Node.js and WebAssembly) are atomic: when one row is invalid (or reading a key of one item throws), nothing is added. The error names the offending row (`item 3`).
- `destroy()` on `FuzzyObjectIndex` and `KeyedFuzzyIndex` now leaves a usable, empty index, as with `FuzzyIndex`: `add()` / `addMany()` work afterwards (they used to throw `Expected 0 key values`), and a destroyed index serializes to data that deserializes again (it used to fail with `Total weight must be greater than zero`).
- Key paths of `searchObjects()` / `FuzzyObjectIndex` resolve own properties (and getters defined by the object's class) only: inherited members such as `constructor` or `toString` are no longer indexed. Plain objects are no longer indexed as `"[object Object]"`, functions and symbols are skipped, null-prototype objects no longer throw, and arrays are indexed as their elements joined with spaces (`"red green"` instead of `"red,green"`). Numeric path segments index into arrays (`'tags.0'`), while a path that continues past a string or number (such as `'name.length'`) indexes as an empty string. Sparse input arrays no longer throw.
- `searchObjects()` and `FuzzyObjectIndex` validate their arguments: a non-array `items` or a malformed key throws a `TypeError` with a clear message.
- `FuzzyObjectIndex.deserialize()` throws an `Error` whose message starts with `Invalid FuzzyObjectIndex data` for malformed input (truncated data, bad metadata, a corrupt native part, or an item count that does not match) instead of a `SyntaxError` or an out-of-bounds read, and a `TypeError` for non-binary input.
- `FuzzyIndex.fromAsync()` returns a rejected Promise for invalid input (such as an array containing a non-string) instead of throwing synchronously.

**Types**

- `MatchType` is declared as a regular `enum` instead of a `const enum`, so `MatchType.Exact` compiles under `isolatedModules` (TS2748). The runtime object was already exported.
- Array parameters accept readonly arrays (`ReadonlyArray<string>`) across `index.d.ts`, `objects.d.ts` and `highlight.d.ts`, and the optional fields of `SearchOptions`, `ObjectIndexSearchOptions`, `KeyConfig` and `HighlightOptions` accept an explicit `undefined` under `exactOptionalPropertyTypes`.
- Key names written as literals in `searchObjects()` / `new FuzzyObjectIndex()` are checked against the item type, segment by segment (dotted paths such as `'address.city'`); a typo reports the valid alternatives. Keys typed as plain `string` are still accepted, and wide item types (`any`, `unknown`, index signatures) accept any key. The new `KeyPath<T>` type lists the paths of `T`, and `KeyConfig` / `ObjectSearchOptions` / `ObjectIndexOptions` take an optional item type parameter to check key lists declared ahead of time.
- `ObjectSearchResult.index` is documented as the item's current position, which changes after `remove()`.
