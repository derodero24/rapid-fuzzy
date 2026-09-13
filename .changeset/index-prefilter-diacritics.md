---
'rapid-fuzzy': patch
---

Fix `FuzzyIndex` and `KeyedFuzzyIndex` dropping matches on accented text. The standalone `search()` folds diacritics (`cafe` matches `café`), but the index prefilters compared raw characters and rejected those items before scoring, so `new FuzzyIndex(['café']).search('cafe')` returned nothing and indexes over 5000 items lost most folded matches. The prefilters now normalize characters the same way the matcher does.
