# rapid-fuzzy

## 2.2.0

### Minor Changes

- 1503b7f: Fix the browser and Cloudflare Workers builds, which previously could not be loaded from bundlers, CDNs or edge runtimes, make the WebAssembly build available to Deno, and add the object search API to them (#730). The Node.js entry points (`import` / `require`) are unchanged.
  
  - **Bundlers**: the `browser` export condition is now listed before `import` / `require`, so Vite, webpack and Next.js resolve the WebAssembly build for browser code instead of the Node.js loader (which failed at build time or at runtime with `createRequire is not a function`). `require()` still resolves to the Node.js entry under the `browser` condition, so test setups such as Jest with `jest-environment-jsdom` keep working. The `module` field, which pointed tools that ignore `exports` at the Node.js loader, is removed.
  - **No initialization step**: the wasm-bindgen glue is now generated with `--target web` and the browser entry (`browser.mjs`) instantiates the WebAssembly module with top-level `await`, so the API is ready, and synchronous, once the import resolves. It no longer relies on the WebAssembly ESM integration that Vite, esbuild, Bun, Deno and Workers do not support. Works without configuration in Vite 8 (build and dev server), webpack 5 and Next.js 16 (Turbopack and webpack); with esbuild and `bun build`, copy `rapid-fuzzy-wasm-bindgen_bg.wasm` next to the bundle.
  - **`sideEffects`** now lists the modules that instantiate WebAssembly. With `"sideEffects": false`, webpack and Rollup dropped the initialization and every call failed.
  - The browser files are `.mjs` ES modules: webpack and Next.js rejected the previous ES module syntax in `.js` files of this CommonJS package.
  - **CDN**: the browser build is plain ES modules and loads directly from jsDelivr or unpkg, e.g. `import { search } from 'https://cdn.jsdelivr.net/npm/rapid-fuzzy@2/browser.mjs'`.
  - **Cloudflare Workers**: a new `workerd` export condition selects `workerd.mjs`, which imports the `.wasm` as a precompiled `WebAssembly.Module` (Workers cannot compile WebAssembly at runtime; the previous build failed at startup).
  - **Deno**: `npm:rapid-fuzzy` still loads the Node.js entry and its native addon by default (run with `--allow-ffi`), so existing Deno code keeps the full Node.js API. `deno run --conditions=browser --allow-read` selects the WebAssembly build instead, which needs no FFI permission.
  - **`rapid-fuzzy/highlight` and `rapid-fuzzy/objects`** now resolve to browser / edge builds, and `searchObjects` / `FuzzyObjectIndex` are available outside Node.js (#730). `FuzzyObjectIndex` has no `serialize()` / `deserialize()` there, since they exchange Node.js Buffers.
  - New declarations (`browser.d.mts`, used through the `browser` and `workerd` conditions, e.g. with TypeScript's `customConditions`) describe exactly what these entries export.
  - In Node.js and Bun, `--conditions=browser` loads the WebAssembly build, e.g. to test the code path of a browser bundle.
  - **Removed files**: `browser.js` and the `--target bundler` glue (`rapid-fuzzy-wasm-bindgen.js`, `rapid-fuzzy-wasm-bindgen_bg.js` and their `.d.ts`) are no longer published; they are replaced by `browser.mjs` and `rapid-fuzzy-wasm-bindgen.mjs`. The package `exports` never exposed them, and loading them required WebAssembly ESM integration, so only direct CDN file URLs referenced them: point those at `browser.mjs` instead.
- 1503b7f: Harden the object-search wrappers, `KeyedFuzzyIndex` and the TypeScript declarations.
  
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
- b403c8a: Add a `matchMode` option to multi-key search (`searchObjects()`, `FuzzyObjectIndex`, `searchKeys()`, `KeyedFuzzyIndex`; Node.js and WebAssembly builds) so that the terms of a query can match different keys (#782):
  
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
- b403c8a: Add a `scoreMode` option to multi-key search (`searchObjects()`, `FuzzyObjectIndex`, `searchKeys()`, `KeyedFuzzyIndex`; Node.js and WebAssembly builds) that chooses how the per-key scores are combined (#781):
  
  - `'weighted'` (the default, unchanged): the weighted average over all keys. A key that does not match counts as 0, so an exact match on one of several keys only scores that key's share of the total weight.
  - `'matched'`: the weighted average over the keys that match only. An item whose only match is exact scores 1, and is no longer ranked below items that match several keys partially or dropped by `minScore`.
  - `'max'`: the highest key score; weights only select the keys that take part.
  
  ```typescript
  searchObjects('smith', people, { keys: ['name', 'email'], scoreMode: 'matched' });
  index.closest('smith', { minScore: 0.9, scoreMode: 'matched' });
  ```
  
  `closest()` of `KeyedFuzzyIndex` and `FuzzyObjectIndex` now takes an options object, `closest(query, { minScore, scoreMode, matchMode })`, and returns the item (or, for `KeyedFuzzyIndex`, the index) that `search()` ranks first with these options. A number in place of the object is the `minScore`, so `closest(query, 0.9)` works as before.
  
  `keyScores`, tie-breaking, zero-weight keys and `returnAllOnEmpty` work the same in every mode, and `minScore` and `maxResults` apply to the mode's score. The `KeyedFuzzyIndex` early exit stays exact in every mode: the index returns the same results as `searchKeys()`. A `scoreMode` other than the three modes throws (an `InvalidArg` error in Node.js, a `TypeError` in the WebAssembly build). TypeScript: the new `KeyScoreMode` type is `'weighted' | 'matched' | 'max'`; `searchKeys()` and `KeyedFuzzyIndex.search()` take the new `KeySearchOptions` (`SearchOptions` plus `scoreMode`), which accepts every existing `SearchOptions` value; the `closest()` options are the new `KeyClosestOptions` (`minScore`, `scoreMode` and `matchMode`). Plain `search()` and `FuzzyIndex` options are unchanged.

### Patch Changes

- 1503b7f: Packaging fixes:
  
  - The Windows on ARM64 addon (`rapid-fuzzy-win32-arm64-msvc`) now links the MSVC C runtime statically, as the x64 addon already did. It no longer needs `VCRUNTIME140.dll` (the Visual C++ Redistributable) and loads on a clean Windows on ARM machine.
  - Every package now ships license information. The nine platform packages (`rapid-fuzzy-<platform>`, `rapid-fuzzy-wasm32-wasi`) now include the MIT `LICENSE` (previously missing), and all ten packages include a `THIRD_PARTY_NOTICES` file with the licenses of the Rust crates and toolchain components compiled into the binaries. This includes `nucleo-matcher` (MPL-2.0), which is used unmodified and whose source is available at https://crates.io/crates/nucleo-matcher.
- 1503b7f: Fix several distance / similarity bugs and make every entry point (single, `*Batch`, `*Many`, `*ManyU32` / `*ManyF64`) return identical results, in both the Node.js and the browser (wasm-bindgen) build.
  
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
- 1503b7f: Documentation and benchmark corrections (no runtime behaviour changes):
  
  - The package description and README no longer claim "up to 15,000x faster than fuse.js". That figure, and the "297x" (`FuzzyIndex` vs `search()`) and "569x" (closest match) figures, came from benchmark queries that matched no items or for which `closest()` returned `null`. The benchmarks now use queries that match, add rotating-query and type-ahead cases, and the README reports the re-measured numbers together with the machine, Node.js version and methodology. Measured that way, `FuzzyIndex` is roughly 150-200x faster than fuse.js and 1.6-2x faster than fuzzysort on 10,000 items, while fuzzysort and uFuzzy are faster on a 20-item list.
  - README examples now show the actual results (`searchObjects`, `highlightRanges`, `jaroWinklerManyF64`, search scores), the actual `destroy()` behaviour (the index stays usable and empty instead of throwing), and what `serialize()` stores (the item strings; `deserialize()` recomputes the search data).
  - New README sections: how matching and scoring work (subsequence matching tolerates missing letters, not substituted or swapped ones; `isCaseSensitive: false` means smart case), performance limits for long and untrusted input with measured timings, and Unicode notes (no NFC/NFKC normalization, one code point per grapheme, realistic `minScore` values for Chinese and Japanese text, the greedy fallback for very long items and query terms). The low-level `searchKeys()` / `KeyedFuzzyIndex`, `FuzzyIndex.fromAsync()`, `addMany()`, `size`, `matchType` and `highlight`'s `escapeHtml` option are now documented.
  - Migration guides corrected: `fuzz.ratio` corresponds to `normalizedIndel` (not `normalizedLevenshtein`), and fuzzball's token, partial and weighted ratios score differently from rapid-fuzzy's; `closest()` is a fuzzy search, not a replacement for fastest-levenshtein's `closest()` or string-similarity's `findBestMatch()`; fuse.js `threshold` values do not translate to `minScore`; several "Before" snippets called APIs that do not exist or printed results the libraries do not return.
  - JSDoc: parameters are referred to by their TypeScript names (`maxDistance`, `minSimilarity`, …), the distance functions describe how they treat case, whitespace and Unicode, and the `weightedRatio` and `partialRatio` descriptions match what they compute.
- 1503b7f: Make `KeyedFuzzyIndex` (and `FuzzyObjectIndex`) return exactly the same results as `searchKeys()` (and `searchObjects()`), and fix several multi-key search bugs. Both now run the same search routine.
  
  - `KeyedFuzzyIndex` no longer drops items whose combined score equals `minScore` exactly. Its early exit compared against a running float subtraction, so for example 5 keys weighted `[0.494, 0.953, 0.137, 0.447, 0.673]` that all match `foo` perfectly returned nothing with `minScore: 1`.
  - Keys with a weight of `0` are handled the same way everywhere: they are scored and reported in `keyScores` (the index used to report 0 for them), but they never select an item on their own. `searchKeys()` used to return items that matched only a zero-weight key, with a score of 0; it no longer does. In general, items whose combined score is 0 are not returned.
  - Equal scores are ordered like `search()` orders them: the item whose best-matching key text is shorter comes first, then the lower index. Previously both multi-key paths ordered ties by index only, so `foobar` could rank above an exact `foo`.
  - Queries are handled like in `search()`: ideographic spaces (U+3000), no-break spaces, tabs and newlines separate terms; queries made only of syntax characters (`^`, `!`, `$`, `'`) are treated as empty (no results, or every item with `returnAllOnEmpty`); and a query term longer than 2,520 characters matches nothing instead of producing wrapped-around scores.
  - `searchKeys()` now throws an `InvalidArg` error, with the same message as the `KeyedFuzzyIndex` constructor, when the key text arrays have different lengths, the number of weights differs from the number of keys, a weight is negative, `NaN` or infinite, or the weights sum to 0. It used to silently return `[]`. `searchObjects()` throws for invalid weights accordingly. In the browser (wasm) build, `searchKeys()` throws a real `Error` for these cases and for key texts that are not arrays of strings.
  - Weights whose sum overflows to `Infinity` (e.g. two weights of `Number.MAX_VALUE`) are rejected by `searchKeys()`, `searchObjects()`, `KeyedFuzzyIndex`, `FuzzyObjectIndex` and their `deserialize()`; previously every score became 0.
  - `searchKeys()` accepts a number as its fourth argument as a shorthand for `{ maxResults }`, like `search()`, in both the Node.js and the browser builds; invalid numbers (`NaN`, negative, fractional) throw.
  - Performance: `KeyedFuzzyIndex.search()` no longer allocates and zeroes a keys × items score matrix on every search and skips items whose keys all fail the character prefilter, making searches that match few items about 3x faster on 100k items × 3 keys. `searchKeys()` reuses its matcher between calls (about 2x faster for small inputs).
- b403c8a: Multi-key search (`searchObjects()`, `FuzzyObjectIndex`, `searchKeys()`, `KeyedFuzzyIndex`) now depends only on the ratios between the weights. With very small weights, such as `[1e-323, 5e-324]`, every `weight × keyScore` product was rounded to a multiple of the smallest double, so a partial match could score as high as an exact one and pass a `minScore` it should not; such weights now give exactly the results of `[2, 1]`. Results for weights of ordinary size are unchanged, and `weights` still returns the weights as given.
- 1503b7f: Fixes found while reviewing the release candidate:
  
  - WebAssembly build (browsers, Deno, Workers): `maxResults` is read like in the Node.js build. `Infinity` means no limit (it used to throw `TypeError: ... expected u32`), values beyond 2^32 are capped instead of throwing, and NaN, negative and fractional values throw a `TypeError` saying that `maxResults must be a non-negative integer or Infinity`. This applies to `search`, `searchKeys`, `searchObjects` and every index's `search` / `searchIndices`, in both the options object and the numeric shorthand.
  - WebAssembly build: `KeyedFuzzyIndex.search(query, n)` and therefore `FuzzyObjectIndex.search(query, n)` accept the numeric `maxResults` shorthand, as declared and as in the Node.js build; they used to throw `Invalid SearchOptions`.
  - WebAssembly build: `FuzzyIndex.fromAsync()` with invalid items (such as an array containing a non-string) returns a Promise rejected with a `TypeError` instead of throwing synchronously.
  - WebAssembly build: array parameters are declared as readonly (`ReadonlyArray<string>`, `ReadonlyArray<ReadonlyArray<string>>`), like in the Node.js declarations. Readonly arrays passed to `levenshteinBatch`, `levenshteinMany`, `search`, `FuzzyIndex` and the rest used to type-check only against the Node.js entry point.
  - TypeScript: literal key names passed to `searchObjects` / `new FuzzyObjectIndex` inside a generic function compile again when the item type parameter is constrained to have them (`function f<T extends { name: string }>(items: T[]) { searchObjects(q, items, { keys: ['name'] }) }`), including nested paths and `{ name, weight }` entries. Names the constraint lacks are still rejected. An item type parameter without such a constraint cannot be checked, so there literal key names are rejected; add a constraint or pass keys typed as `string` (for example `const keys: string[] = ['name']`).
  - TypeScript: the browser / edge `FuzzyObjectIndex` constructor checks key names against the item type like the Node.js one (a typo such as `keys: ['nmae']` was accepted).
  - `FuzzyObjectIndex.deserialize()` checks the keys stored in the metadata against the native index: a different key count or different weights throw `Invalid FuzzyObjectIndex data: ...` instead of loading an index whose next `add()` fails with an unrelated error. Data serialized by 2.1 after `destroy()` (which kept the keys only in the metadata) loads as an empty index that accepts new items again.
  - `highlight()` and `highlightRanges()` read positions in the unit search results use: by character for ASCII items and by grapheme cluster otherwise. They used to treat positions as UTF-16 offsets, so any emoji, other non-BMP character or combining mark before the match shifted the highlight (`'🍎 apple pie'` came out as `'🍎<b> appl</b>e pie'`) and could split a surrogate pair. Each position now covers its whole grapheme cluster, and `highlightRanges()` returns UTF-16 offsets for `item.slice(start, end)`. Behaviour change: positions computed by hand as UTF-16 offsets for non-ASCII items are now read as grapheme indices. `SearchResult.positions` documents its unit.
  - `THIRD_PARTY_NOTICES` now reproduces the license texts and copyright notices of wasi-libc and the code it includes (cloudlibc, musl, musl-fts, emmalloc, dlmalloc), which the WASI build links in, and of the Rust standard library, instead of pointing to their repositories.
- 1503b7f: Make `FuzzyIndex` return exactly the same results as `search()` and fix several search correctness bugs.
  
  - `FuzzyIndex` with more than 5,000 items no longer drops valid matches (#746). Its bigram pre-filter treated "contains every adjacent pair of query characters" as a requirement, so abbreviations (`hndlr`), paths (`src/index.ts`), Cyrillic or Greek text in a different case (`москва` vs `Москва`), decomposed (NFD) text and escaped spaces (`foo\ bar`) were missed. The bigram index is removed: searches are now always exact, the index builds about 10x faster and uses less memory. Queries that previously returned truncated or empty results now return every match and can take longer on large indexes.
  - `FuzzyIndex` and `KeyedFuzzyIndex` (`FuzzyObjectIndex`) no longer miss uppercase non-ASCII items such as `Łódź`, `ŠKODA` or the Kelvin sign for lowercase queries, at any index size.
  - `search()` reports positions and scores per grapheme for decomposed text (NFD `école`), like `FuzzyIndex` already did, instead of counting the bytes of combining marks. `matchType` counts graphemes too.
  - Scores of queries using syntax are normalized against their own terms: `bar$`, `^foo`, `foo\ bar` and queries with `!` exclusions no longer give every match a score of 1.0, so `minScore` filters them again (this also applies to `searchKeys()` and keyed indexes).
  - In `search()`, `closest()` and `FuzzyIndex`, queries made only of syntax characters (`^`, `$`, `!`, `'`, `^$`) are treated like empty queries: they return no results (or every item with `returnAllOnEmpty`) instead of every item with a score of 0.
  - The incremental `FuzzyIndex` cache is only reused when the new query provably narrows the previous one, fixing wrong results when typing `fo$` → `fo$x` or `foo\` → `foo\ bar`.
  - Unicode whitespace separates query terms: ideographic spaces (U+3000) typed by Japanese and Chinese input methods, no-break spaces, tabs and newlines now work like a regular space in `search()`, `closest()` and `FuzzyIndex`.
  - In `search()`, `closest()` and `FuzzyIndex`, a query term longer than 2,520 characters now matches nothing instead of producing wrapped-around, meaningless scores.
  - `maxResults` (as an option or as the numeric shorthand) must be a non-negative integer or `Infinity` (no limit). `NaN`, negative and fractional values now throw an `InvalidArg` error instead of wrapping modulo 2^32 (`Infinity` used to return no results and `-1` every result). This applies to every function taking `SearchOptions`.
- 1503b7f: Faster large searches and much lower memory use for `FuzzyIndex`, `KeyedFuzzyIndex` and `FuzzyObjectIndex`. Search results are unchanged.
  
  - Large searches use several CPU cores in the native (Node.js, Bun, Deno) builds. A search is split across threads only when it is estimated to take more than about a quarter of a millisecond; smaller searches run on the calling thread as before. On 4 cores, `FuzzyIndex` searches over 10k-1M file paths are about 2-3.5x faster, including with `includePositions`; `search()` over large arrays gains 1.2-1.8x, as converting its input strings remains the main cost. Results are identical to a single-threaded search. Set `RAYON_NUM_THREADS=1` to keep searches on one thread. The WebAssembly builds are unchanged.
  - Ranking large result sets is about 3x faster.
  - `FuzzyIndex`, `KeyedFuzzyIndex` and `FuzzyObjectIndex` no longer allocate their own ~100-130 KB matcher: all indexes, `search()` and `searchKeys()` share one per thread. 10,000 small `FuzzyIndex` objects used ~1 GB, now ~13 MB; 3,000 small `KeyedFuzzyIndex` objects used ~290 MB, now ~6 MB.
  - `destroy()` now frees all of the index's item data immediately (a `KeyedFuzzyIndex` keeps only its key configuration).
  - `FuzzyIndex` and `KeyedFuzzyIndex` (and so `FuzzyObjectIndex`) report their native memory to V8, so the garbage collector reclaims indexes that are dropped without `destroy()`. The memory is freed once the event loop runs the indexes' finalizers: in an `async` loop that awaits between iterations, 300 unreachable 50k-item `FuzzyIndex` objects used to grow memory by ~1.45 GB and now by ~105 MB, and 200 unreachable 20k-item `KeyedFuzzyIndex` objects by ~920 MB and now by ~110 MB. A tight synchronous loop gives the finalizers no chance to run (memory still grows by ~1.2 GB in the first case), so call `destroy()` on indexes created in such a loop. As with allocating a large `Buffer`, building an index of more than ~64 MB can trigger a garbage collection.
  - ASCII items are no longer stored twice: `FuzzyIndex` uses 30-45% less native memory and builds faster.
- 1503b7f: Harden `FuzzyIndex`, `KeyedFuzzyIndex` and `FuzzyObjectIndex` deserialization against corrupt or untrusted buffers.
  
  - `KeyedFuzzyIndex.deserialize()` and `FuzzyObjectIndex.deserialize()` no longer abort the whole Node.js process with `memory allocation of N bytes failed` when a buffer's header declares more items or keys than it contains. Every count is now checked against the bytes actually present before anything is allocated, and a regular `Error` is thrown instead.
  - In the WebAssembly builds, oversized lengths no longer wrap the 32-bit bounds checks and trap the module; they throw an `Error` too.
  - Serialized `FuzzyIndex` buffers are now interchangeable between the Node.js and browser builds: both write the same header (`RFZI`) and both read buffers written by either build, including those written by the browser build of rapid-fuzzy 2.1.1 and earlier (`RFUZ`). Buffers serialized by this release's browser build cannot be read by the browser build of 2.1.1 or earlier.
  - A destroyed `KeyedFuzzyIndex` or `FuzzyObjectIndex` can now be serialized and deserialized. Previously its own `deserialize()` rejected the buffer with `Total weight must be greater than zero`.
  - Deserialization errors now say what is wrong and where, for example `Invalid data: 3 trailing bytes after the last item (at byte 17)`, `Invalid data: item 1 at byte 18 is not valid UTF-8: …`, `Invalid data: bad magic bytes: expected "RFKI", got "RFZI" (this is a serialized FuzzyIndex)` or `Invalid data: weight of key 1 is -1; weights must be finite non-negative numbers`. Messages still start with `Invalid data:` (or `Unsupported format version:`), but code that matched the old messages exactly needs updating.
- 1503b7f: Bring the browser / WebAssembly build (the wasm-bindgen package behind the `browser` export condition) in line with the Node.js binding and the declarations in `index.d.ts`:
  
  - `search`, `FuzzyIndex.search` and `FuzzyIndex.searchIndices` accept a number as `maxResults` shorthand (`search(query, items, 5)`), which previously threw.
  - `closest`, `FuzzyIndex.closest` and `KeyedFuzzyIndex.closest` return `null` instead of `undefined` when nothing matches.
  - `hammingBatch`, `hammingMany`, `normalizedHammingBatch` and `normalizedHammingMany` use `null` instead of `undefined` for undefined distances.
  - Errors are real `Error` objects (`TypeError` for arguments of the wrong type or shape) instead of plain strings, so `instanceof Error` and stack traces work. `searchKeys`, `KeyedFuzzyIndex` and the `*Batch` functions now throw a `TypeError` for malformed array input instead of silently returning an empty result.
  - Invalid options no longer leak WebAssembly memory: each rejected call used to leak its already-converted arguments.
  - `matchType` is omitted from results (instead of being present as `undefined`) when positions are not requested, as in the Node.js binding.
  - Added `FuzzyIndex.fromAsync`. The WebAssembly build has no worker thread, so it builds the index synchronously and returns an already-resolved Promise.
  - The browser entry now also exports `MatchType` and the `*ManyU32` / `*ManyF64` typed-array variants declared in `index.d.ts`.
  - `rapid-fuzzy-wasm-bindgen.d.ts` no longer contains `any`: every `SearchOptions` field is optional, results are typed (`SearchResult`, `IndexSearchResult`, `KeySearchResult`, `MatchType`), parameter names match the Node.js binding (`minScore`, `maxDistance`, `minSimilarity`, `keyTexts`), and `weights` accepts `number[]` as well as `Float64Array`.
- b403c8a: The TypeScript declarations of the WebAssembly build (`browser.d.mts` and the `rapid-fuzzy-wasm-bindgen.d.mts` glue behind it) let every field of the options objects (`SearchOptions`, `KeySearchOptions` and `KeyClosestOptions`) be set to `undefined` explicitly, like the Node.js declarations: `maxResults?: number | undefined` instead of `maxResults?: number`. Under `exactOptionalPropertyTypes`, options such as `{ maxResults: limit }` with an optional `limit` now type-check in browser code too, and the option types of the two builds are the same types. At runtime both builds already treated `undefined` as "not set". The result types are unchanged: a `matchType` that is not set is omitted, never `undefined`.

## 2.1.1

### Patch Changes

- ec2c8e1: Fix `FuzzyIndex` returning too few results after a thresholded search. The incremental cache reused the candidates left over from a previous `minScore` search or `closest()` call, so a longer query typed afterwards was limited to the items that had passed the earlier threshold. The cache now only stores unfiltered matches, and it is not reused across a change of case mode (`isCaseSensitive`).
- b4d0546: Fix `FuzzyIndex` and `KeyedFuzzyIndex` dropping matches on accented text. The standalone `search()` folds diacritics (`cafe` matches `café`), but the index prefilters compared raw characters and rejected those items before scoring, so `new FuzzyIndex(['café']).search('cafe')` returned nothing and indexes over 5000 items lost most folded matches. The prefilters now normalize characters the same way the matcher does.
- eb1b247: Update the napi-rs toolchain (`napi` 3.12, `@napi-rs/cli` 3.9, emnapi 2.0) and align the `rapid-fuzzy-wasm32-wasi` fallback package with the current napi-rs layout:
  
  - It now declares exact `@emnapi/core` / `@emnapi/runtime` dependencies matching the runtime the WASM binary was built against, instead of relying on peer resolution.
  - It ships its own type definitions (`rapid-fuzzy.wasi.d.cts`).
  - It no longer carries a `cpu: ["wasm32"]` restriction, so it can be installed on any platform. It is not an optional dependency of `rapid-fuzzy`: on platforms without a prebuilt native binary, install `rapid-fuzzy-wasm32-wasi` explicitly and the loader falls back to it.
  - Its `engines.node` range follows the WASI API requirements (`>=22.13.0 <23.0.0-0 || >=23.5.0`).
  
  The platform package manifests also pick up the current package description, keywords, and bug tracker URL.
- 0b55a12: `sorensenDiceMany` now returns the same scores as `sorensenDice` and `sorensenDiceBatch`. The many variant used its own bigram implementation, so it disagreed with the single-pair function on whitespace (`'a b'` vs `'ab'`), inputs shorter than two characters, and multi-byte characters.
- e63f755: `weightedRatio`, `weightedRatioBatch` and `weightedRatioMany` now return identical scores. The plain-ratio component was computed on the original strings by the single-pair function but on the normalized (lower-cased, whitespace-collapsed) strings by the many variant; all three now take the better of the two.
- e6642b3: Treat whitespace-only queries as empty in `search`, `closest`, `searchKeys`, `FuzzyIndex` and `KeyedFuzzyIndex`. A query such as `' '` used to return every item with a score of 1.0 (a pattern with no atoms scored 0 and normalized to NaN); it now returns no results unless `returnAllOnEmpty` is set, matching the behavior of the empty string.

## 2.1.0

### Minor Changes

- 49a4ca7: Add `hammingManyU32` and `normalizedHammingManyF64` TypedArray variants, completing the `*Many` TypedArray family. Because `hammingMany`/`normalizedHammingMany` return `null` for length mismatches or candidates filtered by the threshold, the typed arrays use a documented sentinel for those slots: `0xffffffff` for `hammingManyU32` and `NaN` for `normalizedHammingManyF64`.

  Also enrich type documentation: field-level JSDoc on `KeyConfig` and `ObjectSearchResult` (and a clearer note on why `includePositions` has no effect for multi-key search), and document the `number` (maxResults shorthand) argument on `FuzzyIndex.search()` and `searchIndices()`.

## 2.0.0

### Major Changes

- cf53aa4: Drop Node.js 20 support. The minimum required Node.js version is now 22.0.0.

  Node.js 20 reached End-of-Life on 2026-04-30. Consumers running on Node 20
  must upgrade to Node 22+ before pulling in this release.

## 1.2.0

### Minor Changes

- 560bf4d: Add `indel`, `indelBatch`, `indelMany`, `normalizedIndel`, `normalizedIndelBatch`, `normalizedIndelMany` (insertion-deletion distance) and `normalizedHamming`, `normalizedHammingBatch`, `normalizedHammingMany` (normalized Hamming similarity) functions.
- d6a5a57: Add `FuzzyIndex.fromAsync(items)` static factory that constructs the index on the libuv thread pool, returning `Promise<FuzzyIndex>`. For large datasets this keeps the JavaScript event loop unblocked during index construction — useful in Next.js API routes, Nuxt server handlers, and other environments where blocking is a concern.
- 1ec91c9: Add `closest()`, `serialize()`, and `deserialize()` to `KeyedFuzzyIndex`, bringing it to API parity with `FuzzyIndex`.
- 0af7da3: Add optional `minSimilarity` threshold parameter to `sorensenDiceMany`, `tokenSortRatioMany`, `tokenSetRatioMany`, `partialRatioMany`, and `weightedRatioMany`. Candidates scoring below the threshold return `0.0`.

  Also optimizes `sorensenDiceMany` by pre-computing reference bigrams once and reusing them across all candidates, matching the pattern of other `*Many` functions.

- 6487992: Add `serialize()` and `static deserialize()` to `FuzzyObjectIndex`, enabling SSR/SSG pre-building patterns where the index is constructed at build time and shipped as a binary blob for fast client-side initialization.
- a4d79e6: Add `*ManyU32` and `*ManyF64` typed-array variants for all `*Many` distance functions. These return `Uint32Array` or `Float64Array` instead of `Array<number>`, reducing GC pressure for large candidate sets (1000+ items).

  **New `Uint32Array` variants:** `levenshteinManyU32`, `damerauLevenshteinManyU32`, `indelManyU32`

  **New `Float64Array` variants:** `jaroManyF64`, `jaroWinklerManyF64`, `sorensenDiceManyF64`, `normalizedLevenshteinManyF64`, `normalizedIndelManyF64`, `tokenSortRatioManyF64`, `tokenSetRatioManyF64`, `partialRatioManyF64`, `weightedRatioManyF64`

  All variants accept the same parameters as their `*Many` counterparts.

### Patch Changes

- 7ae4ff2: Fix mutation ordering in `objects.js` where the JS items array was mutated before Rust operations completed, which could cause state divergence if the Rust call threw an error.

## 1.1.1

### Patch Changes

- 1895bc6: Fix TypeScript `node16`/`nodenext` ESM resolution for `rapid-fuzzy/highlight` and `rapid-fuzzy/objects` subpath exports by adding proper `.d.mts` type declarations and fixing import specifiers in `index.d.mts`.

## 1.1.0

### Minor Changes

- 077c83a: Add Hamming distance functions (`hamming`, `hammingBatch`, `hammingMany`) for comparing equal-length strings.
- 0ea6ae9: Add optional threshold parameters to `_many` distance functions for early termination. `levenshteinMany` and `damerauLevenshteinMany` accept `maxDistance`, while `jaroMany`, `jaroWinklerMany`, and `normalizedLevenshteinMany` accept `minSimilarity`.
- bfeb33b: Add `searchIndices()` method to FuzzyIndex that returns only indices and scores without cloning item strings, reducing GC pressure for large datasets.

### Patch Changes

- 29a340b: Add bigram inverted index pre-filtering to FuzzyIndex for improved search performance on large datasets (5K+ items). Reduces the number of candidates passed to the scoring function by filtering items that lack query character adjacency patterns.
- c10a29e: Fix `tokenSetRatio` returning 1.0 (perfect match) when comparing an empty string against a non-empty string. Now correctly returns 0.0.
- d1bf5f5: Optimize KeyedFuzzyIndex search with zero-weight key skipping, early exit on partial score upper bound, and per-key character-presence pre-filtering.
- dc578d4: Reduce redundant string normalization in weighted_ratio by sharing pre-normalized strings across sub-algorithms.
- 90d0cc2: Replace unwrap() with proper error propagation in FuzzyIndex deserialization
- 530c0d3: Optimize token-based \_many distance functions by pre-computing reference string normalization and tokenization once instead of per-candidate.

## 1.0.0

### Major Changes

- 649e1f6: First stable release. The public API is now considered stable and will follow semver strictly.

  Key highlights since 0.x:

  - 9 string distance algorithms with batch and many variants
  - Fuzzy search powered by nucleo with query syntax (exclude, prefix, suffix, exact)
  - FuzzyIndex and FuzzyObjectIndex for persistent Rust-side indexing with incremental cache
  - Object search with weighted keys (searchObjects, FuzzyObjectIndex)
  - Match highlighting utilities (highlight, highlightRanges)
  - Subpath exports for tree-shaking (rapid-fuzzy/highlight, rapid-fuzzy/objects)
  - Node.js native (napi-rs) + WASM (browser/Deno/Bun) dual distribution
  - Full TypeScript support with auto-generated type definitions

### Patch Changes

- 94bbcab: Migrate Jaro and Jaro-Winkler from strsim to rapidfuzz bit-parallel implementation for ~2.7x faster single-pair and ~1.9x faster batch comparisons
- ce08337: Add weight and key length validation to KeyedFuzzyIndex constructor to prevent panics during search

## 0.6.0

### Minor Changes

- 541c6a6: Add `serialize()` and `FuzzyIndex.deserialize()` for prebuilt index persistence

### Patch Changes

- b48e501: Add character-presence pre-filter to FuzzyIndex to skip non-matching items before scoring, reducing search time by ~2x
- ac0fcd8: Add incremental search cache to FuzzyIndex for faster keystroke-by-keystroke autocomplete
- d17701f: Validate key count in KeyedFuzzyIndex.add() and addMany() to prevent index corruption and Node.js crashes
- 9376569: Add uFuzzy competitor and 50K dataset to search benchmarks
- 38dd449: Reuse Matcher via thread-local storage in standalone search and closest to avoid per-call allocation overhead
- 86d6bcd: Use quickselect for top-k selection and add length-based tiebreaker for better ranking differentiation

## 0.5.0

### Minor Changes

- 3de4f98: Add `FuzzyObjectIndex` class for persistent indexed search over object collections with weighted keys
- db3d8ea: Adopt bit-parallel algorithm (Myers' method) for Levenshtein and Damerau-Levenshtein distance functions via the `rapidfuzz` crate, significantly improving performance for string distance computations

### Patch Changes

- 5695bb9: Add `positions` field to `ObjectSearchResult` type for match position tracking
- 7b11d57: Use score-based early termination in partial_ratio sliding window to skip windows that cannot beat the current best score
- e5a6a27: Optimize search result construction with two-pass scoring to reduce heap allocations
- 20d1aba: Replace duplicated highlight.mjs implementation with ESM re-export from highlight.js to eliminate manual sync requirement

## 0.4.0

### Minor Changes

- 0744dd7: Add `isCaseSensitive` option to `SearchOptions` for explicit control over case-sensitive matching
- 5bd6b95: Add `searchKeys` function for multi-key weighted fuzzy search
- b120905: Add `searchObjects` function for ergonomic object array search with weighted keys

## 0.3.0

### Minor Changes

- Add score threshold filtering to search() and closest() functions.

  search() now accepts a SearchOptions object with maxResults and minScore fields, while maintaining backward compatibility with the existing number argument for maxResults. closest() accepts an optional minScore parameter to return null when the best match is below the threshold.

- Add token-based matching algorithms inspired by Python's RapidFuzz.

  Four new similarity functions: tokenSortRatio (order-independent via sorted tokens), tokenSetRatio (set intersection-based), partialRatio (best substring match via sliding window), and weightedRatio (maximum across all methods). Each includes batch and many variants for efficient bulk comparisons.

- Add match highlight positions to search results.

  SearchResult now includes a `positions` field containing indices of matched characters. Enable by setting `includePositions: true` in SearchOptions. Positions are computed via nucleo-matcher's indices API, sorted and deduplicated. When not requested, positions is an empty array with zero overhead.

- Add FuzzyIndex class for persistent indexed search.

  A Rust-backed class that holds items in memory, eliminating repeated FFI overhead for applications searching the same dataset multiple times. Supports search with all existing options (maxResults, minScore, includePositions), closest match, and incremental updates via add/addMany/remove methods.

### Patch Changes

- Optimize search performance by reusing UTF-32 conversion buffers across items and switching to unstable sort. Reduces allocations in the hot scoring loop, yielding ~30% improvement on medium-sized datasets (1K items).

## 0.2.0

### Minor Changes

- d279d4b: Normalize search scores to 0.0-1.0 range

  The `score` field in `SearchResult` is now a normalized float between 0.0 (weakest match) and 1.0 (perfect/exact match), instead of a raw integer from the underlying matcher. This makes scores intuitive, self-documenting, and consistent with the distance functions (`normalizedLevenshtein`, `sorensenDice`, etc.) that already return 0.0-1.0 values.

  **Breaking change**: `SearchResult.score` changed from integer to float. Since the project is pre-v1.0, this is a minor version bump.

## 0.1.1

### Patch Changes

- f603232: Fix npm package missing index.js, index.d.ts, and browser.js

  These napi-rs loader files were incorrectly gitignored, causing them to be excluded from the published npm package. CJS `require('rapid-fuzzy')` now works correctly.

## 0.1.0

### Minor Changes

- cc14981: Add batch distance computation API

  Add `*Batch` and `*Many` variants for all six distance functions to reduce FFI overhead when processing multiple string pairs in a single call:

  - `levenshteinBatch` / `levenshteinMany`
  - `damerauLevenshteinBatch` / `damerauLevenshteinMany`
  - `jaroBatch` / `jaroMany`
  - `jaroWinklerBatch` / `jaroWinklerMany`
  - `sorensenDiceBatch` / `sorensenDiceMany`
  - `normalizedLevenshteinBatch` / `normalizedLevenshteinMany`

- d60a11c: Add ESM module format support

  Add dual CJS/ESM package exports via `index.mjs` wrapper and conditional `exports` field in `package.json`. Both `import { search } from 'rapid-fuzzy'` and `const { search } = require('rapid-fuzzy')` now work correctly.

- Initial release: Rust-powered fuzzy search and string distance for JavaScript/TypeScript

  Core distance functions powered by strsim:

  - `levenshtein` / `normalizedLevenshtein` / `damerauLevenshtein`
  - `jaro` / `jaroWinkler`
  - `sorensenDice`

  Fuzzy search powered by nucleo-matcher:

  - `search` — ranked fuzzy search with scores and indices
  - `closest` — find the closest match from a list

  Platform support:

  - Node.js native bindings via napi-rs (macOS, Linux, Windows)
  - WASM fallback for browsers, Deno, and Bun

### Patch Changes

- c0bc486: Add property-based testing with proptest for distance and search functions

  Verify mathematical invariants (symmetry, identity, bounded range, triangle inequality) across thousands of random inputs for all distance algorithms and search functions.
