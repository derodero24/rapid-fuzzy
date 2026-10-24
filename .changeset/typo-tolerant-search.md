---
"rapid-fuzzy": minor
---

New `maxTypos` search option for typo-tolerant search: `search('tpyescript', items, { maxTypos: 2 })` finds `TypeScript`. A query term that does not match as a subsequence is matched again with frizbee, a Smith-Waterman matcher that tolerates wrong, extra and swapped letters. A term tolerates at most one typo per four characters, and only plain fuzzy terms tolerate typos (not `'exact`, `^prefix`, `suffix$` or `!excluded` terms). Matches without typos keep exactly the scores and positions they have without the option, and typo matches score below a perfect match. The option works in `search()`, `FuzzyIndex`, `searchKeys()`, `KeyedFuzzyIndex`, `searchObjects()` and `FuzzyObjectIndex` (including their `closest()` options), in both the Node.js and the WebAssembly builds. The minimum supported Rust version for building from source is now 1.89.
