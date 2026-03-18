---
"rapid-fuzzy": minor
---

Add `searchIndices()` method to FuzzyIndex that returns only indices and scores without cloning item strings, reducing GC pressure for large datasets.
