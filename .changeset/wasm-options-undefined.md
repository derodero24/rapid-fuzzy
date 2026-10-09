---
"rapid-fuzzy": patch
---

The TypeScript declarations of the WebAssembly build (`browser.d.mts` and the `rapid-fuzzy-wasm-bindgen.d.mts` glue behind it) let every field of the options objects (`SearchOptions`, `KeySearchOptions` and `KeyClosestOptions`) be set to `undefined` explicitly, like the Node.js declarations: `maxResults?: number | undefined` instead of `maxResults?: number`. Under `exactOptionalPropertyTypes`, options such as `{ maxResults: limit }` with an optional `limit` now type-check in browser code too, and the option types of the two builds are the same types. At runtime both builds already treated `undefined` as "not set". The result types are unchanged: a `matchType` that is not set is omitted, never `undefined`.
