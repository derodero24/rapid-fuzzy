---
'rapid-fuzzy': patch
---

`weightedRatio`, `weightedRatioBatch` and `weightedRatioMany` now return identical scores. The plain-ratio component was computed on the original strings by the single-pair function but on the normalized (lower-cased, whitespace-collapsed) strings by the many variant; all three now take the better of the two.
