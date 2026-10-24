---
"rapid-fuzzy": patch
---

Documentation: equal scores are ordered by length in UTF-8 bytes (shorter first), which the declarations of `searchKeys()` and `KeySearchOptions.scoreMode` now say for the best-matching key's text, as the README does for `search()` and object search. The README also describes multi-term scores correctly: the terms' scores are averaged weighted by each term's best possible score, which grows with its length, not averaged equally.
