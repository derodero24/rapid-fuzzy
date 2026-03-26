---
"rapid-fuzzy": minor
---

Add `serialize()` and `static deserialize()` to `FuzzyObjectIndex`, enabling SSR/SSG pre-building patterns where the index is constructed at build time and shipped as a binary blob for fast client-side initialization.
