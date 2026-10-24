---
title: Persistent indexes
description: Keep items on the Rust side with FuzzyIndex and FuzzyObjectIndex for fast repeated search, incremental type-ahead, mutation and serialization.
---

`search()` converts every item on every call. For autocomplete, live search, command palettes and file finders, build an index once and query it many times. A `FuzzyIndex` costs about 2 ms to build for 10,000 short items, and each query is then about 10x faster than `search()` on 1,000 to 10,000 items (see [Benchmarks](/rapid-fuzzy/benchmarks/)).

## `FuzzyIndex`

```ts
import { FuzzyIndex } from 'rapid-fuzzy';

const index = new FuzzyIndex(['TypeScript', 'JavaScript', 'Python']);
index.size; // 3

index.search('typscript', { maxResults: 5 }); // or index.search('typscript', 5)
index.closest('tsc'); // → 'TypeScript'

// Indices and scores only: no strings are copied back to JavaScript
index.searchIndices('typscript', { maxResults: 5 });
// → [{ index: 0, score: 0.86, positions: [] }]

// Change the items without rebuilding
index.add('Rust');
index.addMany(['Go', 'Zig']);
index.remove(2); // swap-remove: the last item moves to position 2; false if out of range
```

When you no longer need it, `index.destroy()` frees its Rust-side memory right away (see [Memory](#memory)).

`index.search(query, options)` always returns the same results as `search(query, items, options)` over the same items. Prefer `searchIndices()` in hot paths: it avoids copying item strings and reduces garbage-collection pressure.

### Building large indexes off the main thread

In Node.js, `FuzzyIndex.fromAsync(items)` builds the index on the libuv thread pool and resolves with it. In the WebAssembly build it builds synchronously.

```ts
const big = await FuzzyIndex.fromAsync(manyStrings);
```

### Incremental type-ahead

A `FuzzyIndex` remembers which items matched the previous query. When the next query only adds characters to it (typing `appl`, then `apple`), only those items are scored again:

```ts
index.search('appl'); // scores all items, remembers the matches
index.search('apple'); // scores only the items that matched 'appl'
index.search('xyz'); // not an extension: a full scan
```

- Matches are remembered only when fewer than half of the items matched, because a short first keystroke that matches most items is not worth narrowing to.
- A search with a positive `minScore` does not update them.
- They are used only for plain extensions: not once the query contains `!`, `^`, `$`, `'` or `\`, and not when it contains a character that matching normalizes, such as an accented letter.

Results are the same with or without this cache. In the type-ahead benchmark, 12 keystrokes over 10,000 items took about 1.6 ms in total.

## `FuzzyObjectIndex`

`FuzzyObjectIndex` is the indexed form of [`searchObjects()`](/rapid-fuzzy/guides/object-search/). It keeps your objects on the JavaScript side and their key texts on the Rust side:

```ts
import { FuzzyObjectIndex } from 'rapid-fuzzy';

const userIndex = new FuzzyObjectIndex(users, {
  keys: [{ name: 'name', weight: 2 }, 'email'],
});

userIndex.search('john', { maxResults: 10 });
userIndex.closest('john'); // the best-matching object, or null
userIndex.closest('john', { minScore: 0.9, scoreMode: 'matched' }); // or closest('john', 0.9)
userIndex.destroy();
```

It has the same members as `FuzzyIndex`: `size`, `search()`, `closest()`, `add()`, `addMany()`, `remove()` and `destroy()`. In Node.js it also has `serialize()` and `FuzzyObjectIndex.deserialize()`, for items that are JSON-serializable. `addMany()` is atomic: if one item is invalid, nothing is added, and the error names the item.

## Serialization

Save a `FuzzyIndex` (or a `KeyedFuzzyIndex`) and restore it later:

```ts
const data = index.serialize(); // a Buffer in Node.js, a Uint8Array in the browser build
const restored = FuzzyIndex.deserialize(data);
restored.search('aple');
```

The data holds the item strings in UTF-8, with 4 bytes of length each and a 12-byte header. It does not hold precomputed search data, so `deserialize()` takes about as long as `new FuzzyIndex(items)`. Serialization is useful for persisting an index you built up with `add()` and `remove()`, without keeping the source array.

:::caution
The data starts with a format version. `deserialize()` rejects data from another format version and throws on corrupt data. Keep the source data, or be ready to rebuild the index, after upgrading rapid-fuzzy.
:::

## Memory

An index stores the item strings and precomputed search data on the Rust side: a character mask per item and, for non-ASCII items, their UTF-32 text. ASCII text is stored once.

- Call `destroy()` when you no longer need an index, to free its memory immediately.
- Indexes you drop without `destroy()` are reclaimed by the garbage collector. This happens only once the event loop gets a turn to run their finalizers, so code that creates many indexes in a tight synchronous loop should call `destroy()` on each.
- After `destroy()`, an index stays usable. It behaves as an empty index: searches return nothing, `size` is 0, and `add()` and `addMany()` work as before.
