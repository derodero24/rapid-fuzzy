---
'rapid-fuzzy': patch
---

`sorensenDiceMany` now returns the same scores as `sorensenDice` and `sorensenDiceBatch`. The many variant used its own bigram implementation, so it disagreed with the single-pair function on whitespace (`'a b'` vs `'ab'`), inputs shorter than two characters, and multi-byte characters.
