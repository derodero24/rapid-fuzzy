---
"rapid-fuzzy": patch
---

Harden `FuzzyIndex`, `KeyedFuzzyIndex` and `FuzzyObjectIndex` deserialization against corrupt or untrusted buffers.

- `KeyedFuzzyIndex.deserialize()` and `FuzzyObjectIndex.deserialize()` no longer abort the whole Node.js process with `memory allocation of N bytes failed` when a buffer's header declares more items or keys than it contains. Every count is now checked against the bytes actually present before anything is allocated, and a regular `Error` is thrown instead.
- In the WebAssembly builds, oversized lengths no longer wrap the 32-bit bounds checks and trap the module; they throw an `Error` too.
- Serialized `FuzzyIndex` buffers are now interchangeable between the Node.js and browser builds: both write the same header (`RFZI`) and both read buffers written by either build, including those written by the browser build of rapid-fuzzy 2.1.1 and earlier (`RFUZ`). Buffers serialized by this release's browser build cannot be read by the browser build of 2.1.1 or earlier.
- A destroyed `KeyedFuzzyIndex` or `FuzzyObjectIndex` can now be serialized and deserialized. Previously its own `deserialize()` rejected the buffer with `Total weight must be greater than zero`.
- Deserialization errors now say what is wrong and where, for example `Invalid data: 3 trailing bytes after the last item (at byte 17)`, `Invalid data: item 1 at byte 18 is not valid UTF-8: …`, `Invalid data: bad magic bytes: expected "RFKI", got "RFZI" (this is a serialized FuzzyIndex)` or `Invalid data: weight of key 1 is -1; weights must be finite non-negative numbers`. Messages still start with `Invalid data:` (or `Unsupported format version:`), but code that matched the old messages exactly needs updating.
