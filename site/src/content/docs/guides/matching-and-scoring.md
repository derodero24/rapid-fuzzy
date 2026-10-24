---
title: How matching works
description: What rapid-fuzzy's search matches, how scores are computed, how case and ties are handled, and how to deal with typos.
---

Search uses the [nucleo](https://github.com/helix-editor/nucleo) matcher from the Helix editor, which scores matches the way fzf does. Knowing how it matches helps you choose a `minScore`, or decide whether you need a [distance function](/rapid-fuzzy/guides/string-similarity/) instead.

## What matches

A query term matches an item when all of its characters occur in the item **in order**. This is a subsequence match:

- `tsc` matches `TypeScript`.
- `typscript` matches `TypeScript`: the query only leaves out a letter.
- `tpyescript` (swapped letters) and `typozcript` (a wrong letter) find nothing in `['TypeScript']`.

To tolerate swapped or substituted letters, compare the strings with a distance function, for example `jaroWinklerMany(query, candidates)` or `levenshteinMany(query, candidates, maxDistance)`. A common pattern is to run `search()` first, and fall back to a distance function when it finds nothing:

```ts
import { search, levenshteinMany } from 'rapid-fuzzy';

function suggest(query: string, words: string[]) {
  const hits = search(query, words, 5);
  if (hits.length > 0) return hits.map((hit) => hit.item);
  // "did you mean?": the words at most 2 edits away
  const distances = levenshteinMany(query, words, 2);
  return words.filter((_, i) => distances[i]! <= 2);
}
```

## Scores

Scores are nucleo's scores normalized to 0–1, where 1 is the best score the query can get. That usually means the query appears verbatim at the start of the item or of a space-separated word.

- Matches score higher at the start of the item, after a space or another delimiter (`-`, `_`, `.`, `/`), at camelCase humps, and in consecutive runs. So `york` scores 1.0 against `new york city` but 0.67 against `newyorkcity`.
- A query with several terms is scored by the sum of its terms' raw scores, divided by the sum of their best possible scores. This is an average weighted by each term's best possible score, which grows with the term's length. A short term therefore moves the score less than a long one. For example, `k newyorkcity` scores 0.94 against `xxxxkxxxx newyorkcity`, where `k` alone scores 0.44 and `newyorkcity` scores 1.0.
- Excluded terms (`!term`) do not count toward the score.

Scores are meant for ranking. They are not a percentage of similarity, and you cannot compare them with fuse.js or fuzzysort scores. Choose `minScore` by looking at the scores on your own data, and keep it low (0.4 or less) for Chinese, Japanese and other scripts written without spaces, which get fewer word-boundary bonuses (see [Unicode](/rapid-fuzzy/guides/unicode/)).

## Case

Matching uses smart case by default. A query in lower case matches any case, and a query that contains a capital letter is matched case-sensitively. With `isCaseSensitive: true`, every query is case-sensitive. No option forces case-insensitive matching for a query with capitals: lower-case the query instead.

## Ties

Equal scores are ordered by the item's length in UTF-8 bytes, shorter first, then by index. An ASCII character takes 1 byte, most CJK characters 3, and an emoji 4.

The exception is a query without a search term with `returnAllOnEmpty`. Every item then scores 1, and the items come back in index order.

## Long items and queries

When an item is longer than a few thousand characters, nucleo switches to a faster greedy match. The limit is about 8,300 characters for a 6-character query; it is lower for longer queries and for non-ASCII text. The same happens when a single query term is longer than about 180 characters.

The item still matches, but the score reflects the first possible character positions rather than the best ones, so a verbatim occurrence can score as low as a scattered one. Split long documents into shorter fields or chunks before indexing them.
