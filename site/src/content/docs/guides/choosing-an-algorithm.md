---
title: Choosing an algorithm
description: Which rapid-fuzzy function to use for autocomplete, typo correction, name matching, deduplication and document similarity.
---

| Use case | Use | Why |
| --- | --- | --- |
| Autocomplete, command palettes, file finders | [`FuzzyIndex`](/rapid-fuzzy/guides/indexes/) | Persistent index with an incremental type-ahead cache |
| One-off search in a list | [`search`, `closest`](/rapid-fuzzy/guides/fuzzy-search/) | No setup; subsequence matching like fzf and the Helix editor |
| Searching records by several fields | [`searchObjects`, `FuzzyObjectIndex`](/rapid-fuzzy/guides/object-search/) | Weighted keys, with `scoreMode` and `matchMode` |
| "Did you mean?" and spelling correction | `levenshteinMany`, `damerauLevenshteinMany` | Count edits, including substitutions; Damerau also counts transpositions as one edit |
| Names and short labels | `jaroWinkler`, `tokenSortRatio` | Jaro-Winkler rewards a common prefix; token sort ignores word order |
| Messy records and deduplication | `tokenSetRatio`, `weightedRatio` | Tolerate extra, missing and reordered words |
| Truncated or embedded strings | `partialRatio` | The best match of the shorter string inside the longer one |
| Longer text | `sorensenDice` | Bigram overlap; cheap on long strings |
| Equal-length codes (hashes, SKUs) | `hamming`, `normalizedHamming` | Count differing positions |
| Insertions and deletions only | `indel`, `normalizedIndel` | No substitutions, as in diffs |
| A length-independent edit score | `normalizedLevenshtein` | `1 - distance / max length` |

## Search or distance?

Search ([`search`](/rapid-fuzzy/guides/fuzzy-search/) and the indexes) answers "which of these items contain what I typed?". It is fast on large lists and good for interactive input, but it does not forgive swapped or wrong letters.

Distance functions answer "how different are these two strings?". They handle any kind of typo, but they compare whole strings: `levenshtein('script', 'TypeScript')` is large, even though the item contains the query.

Many applications use both: search first, and fall back to `levenshteinMany` or `jaroWinklerMany` over the candidates when search finds nothing. See [How matching works](/rapid-fuzzy/guides/matching-and-scoring/#what-matches) for an example.
