---
"rapid-fuzzy": minor
---

Add optional `minSimilarity` threshold parameter to `sorensenDiceMany`, `tokenSortRatioMany`, `tokenSetRatioMany`, `partialRatioMany`, and `weightedRatioMany`. Candidates scoring below the threshold return `0.0`.

Also optimizes `sorensenDiceMany` by pre-computing reference bigrams once and reusing them across all candidates, matching the pattern of other `*Many` functions.
