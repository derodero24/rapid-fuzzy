---
"rapid-fuzzy": patch
---

Fixes found while reviewing the release candidate:

- WebAssembly build (browsers, Deno, Workers): `maxResults` is read like in the Node.js build. `Infinity` means no limit (it used to throw `TypeError: ... expected u32`), values beyond 2^32 are capped instead of throwing, and NaN, negative and fractional values throw a `TypeError` saying that `maxResults must be a non-negative integer or Infinity`. This applies to `search`, `searchKeys`, `searchObjects` and every index's `search` / `searchIndices`, in both the options object and the numeric shorthand.
- WebAssembly build: `KeyedFuzzyIndex.search(query, n)` and therefore `FuzzyObjectIndex.search(query, n)` accept the numeric `maxResults` shorthand, as declared and as in the Node.js build; they used to throw `Invalid SearchOptions`.
- WebAssembly build: `FuzzyIndex.fromAsync()` with invalid items (such as an array containing a non-string) returns a Promise rejected with a `TypeError` instead of throwing synchronously.
- WebAssembly build: array parameters are declared as readonly (`ReadonlyArray<string>`, `ReadonlyArray<ReadonlyArray<string>>`), like in the Node.js declarations. Readonly arrays passed to `levenshteinBatch`, `levenshteinMany`, `search`, `FuzzyIndex` and the rest used to type-check only against the Node.js entry point.
- TypeScript: literal key names passed to `searchObjects` / `new FuzzyObjectIndex` inside a generic function compile again when the item type parameter is constrained to have them (`function f<T extends { name: string }>(items: T[]) { searchObjects(q, items, { keys: ['name'] }) }`), including nested paths and `{ name, weight }` entries. Names the constraint lacks are still rejected. An item type parameter without such a constraint cannot be checked, so there literal key names are rejected; add a constraint or pass keys typed as `string` (for example `const keys: string[] = ['name']`).
- TypeScript: the browser / edge `FuzzyObjectIndex` constructor checks key names against the item type like the Node.js one (a typo such as `keys: ['nmae']` was accepted).
- `FuzzyObjectIndex.deserialize()` checks the keys stored in the metadata against the native index: a different key count or different weights throw `Invalid FuzzyObjectIndex data: ...` instead of loading an index whose next `add()` fails with an unrelated error. Data serialized by 2.1 after `destroy()` (which kept the keys only in the metadata) loads as an empty index that accepts new items again.
