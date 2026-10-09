---
"rapid-fuzzy": patch
---

Browser and edge (WebAssembly) build: passing something other than a string where a string is expected (`levenshtein(1, 'a')`, `search(42, items)`, `index.add(null)`, ...) throws a `TypeError` (`Expected a string, got number`) instead of trapping with `RuntimeError: memory access out of bounds`, which came from the allocator being handed an undefined length. The Node.js binding already threw an error for these calls.
