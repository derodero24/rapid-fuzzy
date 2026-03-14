---
"rapid-fuzzy": minor
---

Add FuzzyIndex class for persistent indexed search.

A Rust-backed class that holds items in memory, eliminating repeated FFI overhead for applications searching the same dataset multiple times. Supports search with all existing options (maxResults, minScore, includePositions), closest match, and incremental updates via add/addMany/remove methods.
