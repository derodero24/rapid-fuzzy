---
"rapid-fuzzy": minor
---

Add optional threshold parameters to `_many` distance functions for early termination. `levenshteinMany` and `damerauLevenshteinMany` accept `maxDistance`, while `jaroMany`, `jaroWinklerMany`, and `normalizedLevenshteinMany` accept `minSimilarity`.
