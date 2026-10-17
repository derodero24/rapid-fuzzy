---
"rapid-fuzzy": patch
---

`FuzzyIndex` answers type-ahead queries after a query that matched nothing without rescanning the index: its incremental cache now also keeps an empty match set, and a query that only adds characters to one without matches cannot match anything either. Before, every keystroke after a typo rescanned every item (about 0.3-0.8 ms per keystroke per 100,000 items); results are unchanged.
