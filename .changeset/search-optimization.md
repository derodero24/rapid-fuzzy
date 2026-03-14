---
"rapid-fuzzy": patch
---

Optimize search performance by reusing UTF-32 conversion buffers across items and switching to unstable sort. Reduces allocations in the hot scoring loop, yielding ~30% improvement on medium-sized datasets (1K items).
