---
"rapid-fuzzy": minor
---

Normalize search scores to 0.0-1.0 range

The `score` field in `SearchResult` is now a normalized float between 0.0 (weakest match) and 1.0 (perfect/exact match), instead of a raw integer from the underlying matcher. This makes scores intuitive, self-documenting, and consistent with the distance functions (`normalizedLevenshtein`, `sorensenDice`, etc.) that already return 0.0-1.0 values.

**Breaking change**: `SearchResult.score` changed from integer to float. Since the project is pre-v1.0, this is a minor version bump.
