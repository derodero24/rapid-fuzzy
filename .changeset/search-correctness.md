---
"rapid-fuzzy": patch
---

Make `FuzzyIndex` return exactly the same results as `search()` and fix several search correctness bugs.

- `FuzzyIndex` with more than 5,000 items no longer drops valid matches (#746). Its bigram pre-filter treated "contains every adjacent pair of query characters" as a requirement, so abbreviations (`hndlr`), paths (`src/index.ts`), Cyrillic or Greek text in a different case (`москва` vs `Москва`), decomposed (NFD) text and escaped spaces (`foo\ bar`) were missed. The bigram index is removed: searches are now always exact, the index builds about 10x faster and uses less memory. Queries that previously returned truncated or empty results now return every match and can take longer on large indexes.
- `FuzzyIndex` and `KeyedFuzzyIndex` (`FuzzyObjectIndex`) no longer miss uppercase non-ASCII items such as `Łódź`, `ŠKODA` or the Kelvin sign for lowercase queries, at any index size.
- `search()` reports positions and scores per grapheme for decomposed text (NFD `école`), like `FuzzyIndex` already did, instead of counting the bytes of combining marks. `matchType` counts graphemes too.
- Scores of queries using syntax are normalized against their own terms: `bar$`, `^foo`, `foo\ bar` and queries with `!` exclusions no longer give every match a score of 1.0, so `minScore` filters them again (this also applies to `searchKeys()` and keyed indexes).
- In `search()`, `closest()` and `FuzzyIndex`, queries made only of syntax characters (`^`, `$`, `!`, `'`, `^$`) are treated like empty queries: they return no results (or every item with `returnAllOnEmpty`) instead of every item with a score of 0.
- The incremental `FuzzyIndex` cache is only reused when the new query provably narrows the previous one, fixing wrong results when typing `fo$` → `fo$x` or `foo\` → `foo\ bar`.
- Unicode whitespace separates query terms: ideographic spaces (U+3000) typed by Japanese and Chinese input methods, no-break spaces, tabs and newlines now work like a regular space in `search()`, `closest()` and `FuzzyIndex`.
- In `search()`, `closest()` and `FuzzyIndex`, a query term longer than 2,520 characters now matches nothing instead of producing wrapped-around, meaningless scores.
- `maxResults` (as an option or as the numeric shorthand) must be a non-negative integer or `Infinity` (no limit). `NaN`, negative and fractional values now throw an `InvalidArg` error instead of wrapping modulo 2^32 (`Infinity` used to return no results and `-1` every result). This applies to every function taking `SearchOptions`.
