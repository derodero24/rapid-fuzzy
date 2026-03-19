---
'rapid-fuzzy': patch
---

Add bigram inverted index pre-filtering to FuzzyIndex for improved search performance on large datasets (5K+ items). Reduces the number of candidates passed to the scoring function by filtering items that lack query character adjacency patterns.
