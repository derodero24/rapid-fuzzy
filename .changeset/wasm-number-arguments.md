---
"rapid-fuzzy": patch
---

Browser and edge (WebAssembly) build: the `maxDistance` of `levenshteinMany`, `damerauLevenshteinMany`, `indelMany` and `hammingMany` (and their `*ManyU32` variants), the `minSimilarity` of the `*Many` similarity functions, and the `minScore` of `closest()` and `FuzzyIndex.closest()` throw a `TypeError` (`maxDistance must be a number, got string`) when given something other than a number, `undefined` or `null`, as the Node.js binding throws. They used to be converted with `Number()`: `''`, `false` and `[]` became a `maxDistance` of 0, which capped every distance at 1 (`levenshteinMany('sitten', ['kitten', 'sitting'], '')` returned `[1, 1]`), `true` became 1 and `'2'` became 2, so `closest('ap', ['apricot', 'ap'], true)` and `jaroMany('abc', ['abd'], '0.9')` silently applied a threshold the types do not allow. Numbers, `undefined` and `null` are read as before.
