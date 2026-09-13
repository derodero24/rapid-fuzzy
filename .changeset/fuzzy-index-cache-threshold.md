---
'rapid-fuzzy': patch
---

Fix `FuzzyIndex` returning too few results after a thresholded search. The incremental cache reused the candidates left over from a previous `minScore` search or `closest()` call, so a longer query typed afterwards was limited to the items that had passed the earlier threshold. The cache now only stores unfiltered matches, and it is not reused across a change of case mode (`isCaseSensitive`).
