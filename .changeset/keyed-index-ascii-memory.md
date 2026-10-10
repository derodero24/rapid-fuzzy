---
"rapid-fuzzy": patch
---

`KeyedFuzzyIndex` and `FuzzyObjectIndex` keep ASCII key text only once, like `FuzzyIndex`: they used to store every key text a second time as the matcher's haystack. A one-key index over 1,000,000 ASCII items of about 33 characters now holds 81 MB instead of 122 MB (a third less; about 20% less for a corpus with 25% non-ASCII text), the same as a `FuzzyIndex` over those items, and every additional key saves the same. Results are identical.
