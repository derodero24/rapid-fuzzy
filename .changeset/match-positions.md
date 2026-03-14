---
"rapid-fuzzy": minor
---

Add match highlight positions to search results.

SearchResult now includes a `positions` field containing indices of matched characters. Enable by setting `includePositions: true` in SearchOptions. Positions are computed via nucleo-matcher's indices API, sorted and deduplicated. When not requested, positions is an empty array with zero overhead.
