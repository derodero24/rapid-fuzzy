---
title: Object search
description: Search arrays of objects across weighted keys with searchObjects(), and control how key scores combine with scoreMode and matchMode.
---

`searchObjects()` searches an array of objects across one or more keys, like fuse.js's `keys` option. To search the same objects repeatedly, use [`FuzzyObjectIndex`](/rapid-fuzzy/guides/indexes/#fuzzyobjectindex).

```ts
import { searchObjects } from 'rapid-fuzzy';

const users = [
  { name: 'John Smith', email: 'john@example.com' },
  { name: 'Jane Doe', email: 'jane@example.com' },
  { name: 'Bob Johnson', email: 'bob@test.com' },
];

searchObjects('john', users, { keys: ['name', 'email'] });
// → [
//   { item: { name: 'John Smith', … }, index: 0, score: 1, keyScores: [1, 1] },
//   { item: { name: 'Bob Johnson', … }, index: 2, score: 0.5, keyScores: [1, 0] },
// ]
```

Each result has the `item`, its `index`, the combined `score` and `keyScores`: one score per key, each computed the way `search()` scores a string. The `maxResults`, `minScore` and `returnAllOnEmpty` options work as in [`search()`](/rapid-fuzzy/guides/fuzzy-search/#options) and apply to the combined score.

## Keys and weights

```ts
searchObjects('john', users, {
  keys: [
    { name: 'name', weight: 2 },
    { name: 'email', weight: 1 },
  ],
});
// → John Smith scores 1; Bob Johnson (2 × 1 + 1 × 0) / 3 = 0.667

// Nested properties and array elements
searchObjects('new york', places, { keys: ['address.city', 'tags.0'] });
```

- A key path reads own properties, separated by dots. Array elements use their index, as in `'tags.0'`.
- Strings are indexed as they are. Numbers, booleans, bigints and objects with their own `toString()` (such as `Date`) go through `String()`. Arrays are joined with spaces. Missing values, `null` and plain objects count as an empty string.
- In TypeScript, key names written as literals are checked against the item type.
- A key with weight `0` still gets a score in `keyScores`, but never selects an item on its own.
- Ties go to the item whose best-matching key text is shorter in UTF-8 bytes, then to the lower index.
- `includePositions` has no effect on object search.

## Combining key scores: `scoreMode`

By default (`'weighted'`), a key that does not match counts as 0. An item that matches one key exactly then scores only that key's share of the total weight: it can rank below an item that matches several keys partially, or fall below `minScore`. `scoreMode` chooses how key scores are combined:

| `scoreMode` | Score | Use it when |
| --- | --- | --- |
| `'weighted'` (default) | `sum(weight × keyScore) / sum(weight)` over all keys | A good item matches on most keys |
| `'matched'` | The same weighted average, over the matching keys (`keyScore > 0`) only. An item whose only match is exact scores 1. | A match on any one key should count fully |
| `'max'` | The highest key score. Weights only decide which keys take part (`weight > 0`). | Only the best-matching key matters |

```ts
const people = [
  { name: 'John Smith', email: 'john@example.com' },
  { name: 'S. Mitchell', email: 'smitchell@example.com' },
];
const keys = ['name', 'email'];

searchObjects('smith', people, { keys });
// → S. Mitchell 0.89 (keyScores [0.88, 0.91]), John Smith 0.5 (keyScores [1, 0])
searchObjects('smith', people, { keys, scoreMode: 'matched' });
// → John Smith 1, S. Mitchell 0.89
searchObjects('smith', people, { keys, scoreMode: 'max' });
// → John Smith 1, S. Mitchell 0.91

searchObjects('smith', people, { keys, minScore: 0.95 }); // → []
searchObjects('smith', people, { keys, minScore: 0.95, scoreMode: 'matched' }); // → [John Smith]
```

In every mode the `keyScores` are the same, keys with weight `0` never count, and `returnAllOnEmpty` gives every item a score of 1. For ties, the best-matching key is the one that contributes most to the score: the highest `weight × keyScore`, or the highest `keyScore` in `'max'` mode.

## Terms across keys: `matchMode`

By default (`'perKey'`), every key is matched against the whole query, the way `search()` matches a string. An item matches only when one of its keys contains every term, and a `!term` only zeroes the key that contains it. With `matchMode: 'crossKey'`, each term is matched against the keys on its own, so the terms of a query may match different keys:

```ts
const people = [
  { name: 'John Smith', city: 'Tokyo' },
  { name: 'John Doe', city: 'Osaka' },
  { name: 'Jane Doe', city: 'Tokyo' },
];
const keys = ['name', 'city'];

searchObjects('john tokyo', people, { keys }); // → [] (no key contains both terms)
searchObjects('john tokyo', people, { keys, matchMode: 'crossKey' });
// → John Smith 0.5 (keyScores [0.45, 0.55])
searchObjects('john tokyo', people, { keys, matchMode: 'crossKey', scoreMode: 'matched' });
// → John Smith 1

searchObjects('john !tokyo', people, { keys }); // → John Doe, John Smith
searchObjects('john !tokyo', people, { keys, matchMode: 'crossKey' }); // → John Doe
```

In `'crossKey'` mode:

- Every term must match at least one key, and an item is excluded when a `!term` matches any of its keys. Only keys with a positive weight count toward selecting items.
- A key's score is the share of the query it matches: the scores of the terms it matches, each term counting in proportion to its best possible score, which grows with its length. A key that matches every term perfectly scores 1. `!term`s do not change key scores.
- `scoreMode` still combines the key scores.
  - `'weighted'` is `sum(weight × keyScore) / sum(weight)`.
  - `'matched'` divides by `sum(weight × coverage)` instead. A key's coverage is the share of the query made up by the terms it matches.
  - `'max'` adds up each term's score on the key it matches best, so an item whose every term matches some key perfectly scores 1.
- A query of a single term without `!term`s returns exactly what `'perKey'` mode returns.

## Lower-level APIs: `searchKeys()` and `KeyedFuzzyIndex`

`searchObjects()` and `FuzzyObjectIndex` are built on two APIs that take column-oriented key texts and return item indices. Use them directly when your data is already in columns, or to avoid holding the objects twice:

```ts
import { searchKeys, KeyedFuzzyIndex } from 'rapid-fuzzy';

// keyTexts[k][i] is the text of key k for item i; one weight per key
const keyTexts = [
  ['John Smith', 'Jane Doe'],
  ['john@example.com', 'jane@example.com'],
];
searchKeys('jane', keyTexts, [2, 1]);
// → [{ index: 1, score: 1, keyScores: [1, 1] }]

const keyed = new KeyedFuzzyIndex(keyTexts, [2, 1]);
keyed.search('jane', { maxResults: 10, scoreMode: 'matched' });
keyed.closest('jane'); // → 1 (an index, or null)
keyed.closest('jane example', { minScore: 0.9, scoreMode: 'max', matchMode: 'crossKey' }); // → 1
```

`searchObjects()` and `FuzzyObjectIndex` are also exported from `rapid-fuzzy/objects`.
