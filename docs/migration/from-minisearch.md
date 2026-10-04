# Migrating from MiniSearch to rapid-fuzzy

[MiniSearch](https://www.npmjs.com/package/minisearch) is a lightweight full-text search library with prefix search and fuzzy matching. rapid-fuzzy provides fuzzy matching of short strings (names, titles, file paths, commands) plus distance functions. The two tolerate different kinds of typos: MiniSearch's `fuzzy` option allows a number of edits per term, while rapid-fuzzy's search matches items containing the query's characters in order — missing letters and abbreviations match, substituted or swapped letters do not.

## When to switch

**Switch to rapid-fuzzy when**:
- You need faster fuzzy search on 1K+ item datasets
- You want multiple distance algorithms (Levenshtein, Jaro-Winkler, Dice, etc.)
- You need batch distance operations
- You want match positions for highlighting
- You want a persistent index with `serialize()` / `deserialize()`

**Stay with MiniSearch when**:
- You search structured documents with multiple fields and need field-level boosting
- You rely on prefix matching for autocomplete on exact prefixes
- You need custom tokenizers or term processing pipelines
- You need search results that tolerate substituted or swapped letters

## Installation

```bash
# Remove MiniSearch
npm uninstall minisearch

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## API Mapping

| MiniSearch | rapid-fuzzy | Notes |
|---|---|---|
| `new MiniSearch({ fields })` + `ms.addAll(docs)` | `new FuzzyIndex(items)` | rapid-fuzzy works on string arrays |
| `ms.search(query)` | `index.search(query)` | Returns `{ item, score, index }[]` |
| `ms.search(query, { fuzzy: 0.2 })` | `index.search(query)` | No equivalent option; see [Key Differences](#key-differences) |
| `ms.add(doc)` | `index.add(item)` | Append to index |
| `ms.remove(doc)` | `index.remove(index)` | Remove by position |
| Multi-field search | `searchObjects(query, items, { keys })` | Weighted multi-key search |

## Code Examples

### Basic search

```typescript
// Before (MiniSearch)
import MiniSearch from 'minisearch';
const ms = new MiniSearch({ fields: ['name'], storeFields: ['name'] });
ms.addAll(items.map((name, id) => ({ id, name })));
const results = ms.search('typscript', { fuzzy: 0.2 });

// After (rapid-fuzzy)
import { FuzzyIndex } from 'rapid-fuzzy';
const index = new FuzzyIndex(items);
const results = index.search('typscript');
// [{ item: 'TypeScript', score: 0.86, index: 0, positions: [] }] for ['TypeScript', 'JavaScript', 'Python']
```

### Multi-field object search

```typescript
// Before (MiniSearch)
import MiniSearch from 'minisearch';
const ms = new MiniSearch({
  fields: ['name', 'email'],
  storeFields: ['name', 'email'],
  searchOptions: { boost: { name: 2 } },
});
ms.addAll(users.map((user, id) => ({ id, ...user }))); // MiniSearch needs an id field

// After (rapid-fuzzy)
import { searchObjects } from 'rapid-fuzzy/objects';
const results = searchObjects('john', users, {
  keys: [{ name: 'name', weight: 2 }, 'email'],
});
// [{ item: { name: 'John Smith', email: '...' }, index: 0, score: 1, keyScores: [1, 1] }, ...]
```

### Standalone search (no index)

```typescript
// After (rapid-fuzzy) — no index required for simple cases
import { search } from 'rapid-fuzzy';
const results = search('typscript', items);
```

## What You Gain

### Performance

`FuzzyIndex` keeps the items on the Rust side, ready to search: in the [README benchmarks](../../README.md#benchmarks) (Node.js 22, Linux x64, Intel Xeon @ 2.10GHz) it answered a query over 10,000 items in about 0.2 ms (4,803 ops/s, vs 780 ops/s for MiniSearch with `{ fuzzy: 0.2, prefix: true }`). The two libraries return different matches for the same query, so this compares the cost of a search, not identical work.

### Index serialization

Save and restore indexes without rebuilding:

```typescript
const buffer = index.serialize();
// ... later
const restored = FuzzyIndex.deserialize(buffer);
```

### Distance algorithms

Use specialized algorithms beyond fuzzy search:

```typescript
import { levenshtein, jaroWinkler, sorensenDice, closest } from 'rapid-fuzzy';

levenshtein('kitten', 'sitting');     // 3 (edit distance)
jaroWinkler('Martha', 'Marhta');      // 0.961 (name matching)
closest('fast', ['slow', 'faster']);   // 'faster'
```

## Key Differences

- **Scoring**: MiniSearch uses TF-IDF-based scoring; rapid-fuzzy uses character-level similarity (0.0–1.0).
- **Tokenization**: MiniSearch tokenizes text into terms; rapid-fuzzy matches on the full string.
- **Fuzzy threshold**: MiniSearch's `fuzzy: 0.2` allows edits up to 20% of each term's length; rapid-fuzzy's `minScore` filters by match score (0.0–1.0), which rewards consecutive matches and matches at word starts.
