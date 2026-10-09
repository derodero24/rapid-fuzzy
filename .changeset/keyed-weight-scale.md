---
"rapid-fuzzy": patch
---

Multi-key search (`searchObjects()`, `FuzzyObjectIndex`, `searchKeys()`, `KeyedFuzzyIndex`) now depends only on the ratios between the weights. With very small weights, such as `[1e-323, 5e-324]`, every `weight × keyScore` product was rounded to a multiple of the smallest double, so a partial match could score as high as an exact one and pass a `minScore` it should not; such weights now give exactly the results of `[2, 1]`. Results for weights of ordinary size are unchanged, and `weights` still returns the weights as given.
