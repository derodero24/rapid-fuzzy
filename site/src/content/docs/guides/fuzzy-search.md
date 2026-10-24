---
title: Fuzzy search
description: Search a list of strings with search() and closest(), filter by score, limit results, get match positions and use the query syntax.
---

`search()` ranks a list of strings against a query, and `closest()` returns the single best match. Both take the items as a plain array; to search the same items repeatedly, build a [persistent index](/rapid-fuzzy/guides/indexes/) instead.

## `search()`

```ts
import { search } from 'rapid-fuzzy';

const items = ['TypeScript', 'JavaScript', 'Python', 'TypeSpec'];

search('typscript', items);
// → [{ item: 'TypeScript', score: 0.86, index: 0, positions: [] }]
```

Each result has the matched `item`, its `index` in the array and a `score` between 0 and 1. Results are sorted by score, best first. Ties go to the shorter item, then to the lower index.

### Options

```ts
// Keep at most 5 results scoring at least 0.3
search('app', items, { maxResults: 5, minScore: 0.3 });

// A number in place of the options is maxResults
search('app', items, 5);

// Matched character positions, for highlighting
search('hlo', ['hello world'], { includePositions: true });
// → [{ item: 'hello world', score: 0.77, index: 0, positions: [0, 3, 4], matchType: 'Fuzzy' }]

// Make every query case-sensitive (the default is smart case, see below)
search('type', items, { isCaseSensitive: true });

// Return every item for an empty query, for filter-as-you-type lists
search('', items, { returnAllOnEmpty: true });
```

| Option | Default | Meaning |
| --- | --- | --- |
| `maxResults` | no limit | The most results to return: a non-negative integer, or `Infinity`. |
| `minScore` | `0` | Drop results scoring below this. |
| `includePositions` | `false` | Fill in `positions` and `matchType` (`'Exact'`, `'Prefix'`, `'Contains'` or `'Fuzzy'`). |
| `isCaseSensitive` | `false` | Match case exactly. When `false`, matching uses smart case: a lower-case query ignores case, and a query with a capital letter is case-sensitive. |
| `returnAllOnEmpty` | `false` | For a query without a search term, return every item (score 1, in index order) instead of nothing. |

Pass `positions` to [`highlight()`](/rapid-fuzzy/guides/highlighting/) to mark the matched characters. Positions count graphemes, not UTF-16 code units, so do not use them as string offsets yourself.

## `closest()`

`closest()` returns the item that `search()` ranks first, or `null`:

```ts
import { closest } from 'rapid-fuzzy';

closest('tsc', ['TypeScript', 'JavaScript', 'Python']); // → 'TypeScript'

// The third argument is minScore: null when no match scores that high
closest('xyz', items, 0.5); // → null
```

`closest()` finds the best fuzzy match, not the string with the smallest edit distance. To find the nearest string by edit distance (for "did you mean?" suggestions with typos), use a [distance function](/rapid-fuzzy/guides/string-similarity/) such as `levenshteinMany`.

## Query syntax

Queries support the [nucleo](https://github.com/helix-editor/nucleo) pattern syntax:

| Pattern | Meaning | Example |
| --- | --- | --- |
| `foo bar` | Every term must match, in any order | `john smith` matches `Smith, John` |
| `!term` | Exclude items that contain the term | `apple !pie` excludes `apple pie` |
| `^term` | Starts with | `^app` matches `apple` but not `pineapple` |
| `term$` | Ends with | `pie$` matches `apple pie` |
| `'term` | Contains the exact substring | `'pie` matches `pie` literally |

- Terms are separated by any whitespace, including the ideographic space (U+3000) that Japanese and Chinese input methods type, so `東京　港区` searches for both terms.
- Escape a space with a backslash (`foo\ bar`) to match it literally.
- A query made only of syntax, such as `^` or `!`, has no search term and is treated like an empty query.
- A single term longer than 2,520 characters cannot be scored and matches nothing.
- Diacritics are folded: `cafe` matches `café`, `uber` matches `über` and `naive` matches `naïve`.

The syntax applies to every search function: `search()`, `closest()`, `FuzzyIndex`, `searchObjects()` and `FuzzyObjectIndex`. It does not apply to the distance functions.

## Typos

Search matches a term when its characters appear in the item in order. A query that leaves letters out still matches (`typscript` finds `TypeScript`), but one with swapped or wrong letters does not (`tpyescript` finds nothing). See [How matching works](/rapid-fuzzy/guides/matching-and-scoring/) for the details, and for how to handle typos with the distance functions.
