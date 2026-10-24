---
"rapid-fuzzy": patch
---

Browser and edge (WebAssembly) build: an array argument that is not an array of strings no longer breaks the module. `new FuzzyIndex(items)`, `FuzzyIndex.addMany()`, `search()`, `closest()` and every `*Many` function now check their `string[]` argument before doing any work and throw a `TypeError` (`Expected an array of strings`, or `Expected a string at index N`), like `FuzzyIndex.fromAsync()` already did. Before, the conversion failed inside the WebAssembly call: after one `addMany(['x', 42])` (for example from `rows.map((r) => r.title)` with a missing title) every method of that index, including `free()`, threw `recursive use of an object detected`; every rejected call leaked the converted strings and part of the WebAssembly stack, so about 16,000 rejected calls made every export fail with `memory access out of bounds`; and a string or a number was silently read as an array (`search('a', 'abc')` searched the characters `a`, `b` and `c`).
