---
"rapid-fuzzy": patch
---

Document how strings with lone UTF-16 surrogates are handled (in the declarations of `SearchResult.item` and `closest()`, and in the README's Unicode section): strings are converted to UTF-8, so a lone surrogate, for example from cutting an emoji in half with `slice()`, becomes U+FFFD, and `search()`, `closest()` and `FuzzyIndex` return that converted copy, which is not `===` the original string (`result.index` identifies it). `SearchResult.item` was documented as "the original string". The README also notes that the WASI fallback build currently decodes lone surrogates differently, because of a bug in its emnapi runtime.
