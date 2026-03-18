---
"rapid-fuzzy": patch
---

Fix `tokenSetRatio` returning 1.0 (perfect match) when comparing an empty string against a non-empty string. Now correctly returns 0.0.
