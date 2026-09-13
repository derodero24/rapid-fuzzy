---
'rapid-fuzzy': patch
---

Treat whitespace-only queries as empty in `search`, `closest`, `searchKeys`, `FuzzyIndex` and `KeyedFuzzyIndex`. A query such as `' '` used to return every item with a score of 1.0 (a pattern with no atoms scored 0 and normalized to NaN); it now returns no results unless `returnAllOnEmpty` is set, matching the behavior of the empty string.
