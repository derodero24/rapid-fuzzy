---
"rapid-fuzzy": minor
---

Add token-based matching algorithms inspired by Python's RapidFuzz.

Four new similarity functions: tokenSortRatio (order-independent via sorted tokens), tokenSetRatio (set intersection-based), partialRatio (best substring match via sliding window), and weightedRatio (maximum across all methods). Each includes batch and many variants for efficient bulk comparisons.
