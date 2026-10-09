---
"rapid-fuzzy": patch
---

`FuzzyIndex.remove()` and `KeyedFuzzyIndex.remove()` validate their index like `FuzzyObjectIndex.remove()` does, in the Node.js, WASI and browser builds. The index used to be converted to a 32-bit unsigned integer, wrapping it modulo 2^32, so `remove(NaN)`, `remove(Infinity)`, `remove(0.9)` and `remove(2 ** 32)` silently removed item 0 and returned `true`, and `remove(1.5)` removed item 1; in the browser build `remove(undefined)` (for example from a missed `Map` lookup), `remove(null)` and `remove('1')` were accepted too. Now an index that is not a number throws a `TypeError`, one that is not an integer (`NaN`, `±Infinity` or a fraction) throws a `RangeError`, and an integer out of range (negative, not less than `size`, including values of 2^32 and above) returns `false` without removing anything. Only these invalid calls change behaviour.
