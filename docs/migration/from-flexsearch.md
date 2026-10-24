# Migrating from FlexSearch to rapid-fuzzy

[FlexSearch](https://www.npmjs.com/package/flexsearch) is a full-text search engine that uses inverted indexes and tokenization. rapid-fuzzy is a fuzzy matching library: its search matches items that contain the query's characters in order (so `tsc` and `typscript` find `TypeScript`), and it provides edit-distance and similarity functions. They serve different primary use cases, but rapid-fuzzy can replace FlexSearch for autocomplete over short strings.

## When to switch

**Switch to rapid-fuzzy when**:
- You need matches for partial words, abbreviations and missing letters (e.g., "tsc" or "typscript" → "TypeScript")
- You want ranked results by string similarity
- You need match highlighting positions
- You search short strings (names, tags, file names, commands)

**Stay with FlexSearch when**:
- You search full-text documents (articles, blog posts, logs)
- You rely on tokenization, stemming, or language-specific analyzers
- You need field-level scoring with document weights

## Installation

```bash
# Remove FlexSearch
npm uninstall flexsearch

# Install rapid-fuzzy
npm install rapid-fuzzy
```

## API Mapping

| FlexSearch | rapid-fuzzy | Notes |
|---|---|---|
| `new Index()` + `index.add(id, text)` | `new FuzzyIndex(items)` | rapid-fuzzy indexes by array position |
| `index.search(query)` | `index.search(query)` | Returns `{ item, score, index }[]` |
| `index.add(id, text)` | `index.add(item)` | Appends to index |
| `index.remove(id)` | `index.remove(index)` | Removes by position; the last item moves into the freed position |

## Code Examples

### Basic search

```typescript
const items = ['TypeScript', 'JavaScript', 'Python'];

// Before (FlexSearch)
import { Index } from 'flexsearch';
const index = new Index();
items.forEach((item, i) => index.add(i, item));
index.search('typescript'); // [0] — returns IDs only
index.search('typscript');  // [] — not a token of any item

// After (rapid-fuzzy)
import { FuzzyIndex } from 'rapid-fuzzy';
const index = new FuzzyIndex(items);
index.search('typscript');
// [{ item: 'TypeScript', score: 0.86, index: 0, positions: [] }]
```

### Standalone search (no index)

```typescript
// Before (FlexSearch) — always requires index setup
import { Index } from 'flexsearch';
const index = new Index();
items.forEach((item, i) => index.add(i, item));
const ids = index.search(query);

// After (rapid-fuzzy) — one-liner for simple cases
import { search } from 'rapid-fuzzy';
const results = search(query, items);
```

### Options

```typescript
// Before (FlexSearch)
index.search(query, { limit: 10 });

// After (rapid-fuzzy)
index.search(query, { maxResults: 10, minScore: 0.3 });
```

## What You Gain

### Fuzzy matching

FlexSearch's default index matches whole tokens — "typscript" won't find "TypeScript". rapid-fuzzy's search finds items that contain the query's characters in order, so partial words, abbreviations and missing letters match. It does not tolerate substituted or swapped letters (`tpyescript`); for those, compare candidates with a distance function such as `jaroWinklerMany` or `damerauLevenshtein`.

### Similarity scores

Every result includes a normalized score (0.0–1.0), enabling threshold-based filtering and ranked display.

### Match positions

Get character-level match positions for highlighting:

```typescript
import { search } from 'rapid-fuzzy';
import { highlight } from 'rapid-fuzzy/highlight';

const results = search('type', items, { includePositions: true });
const html = highlight(results[0].item, results[0].positions, '<b>', '</b>');
```

### Additional algorithms

Access edit-distance and similarity functions for specialized use cases:

```typescript
import { levenshtein, jaroWinkler, sorensenDice } from 'rapid-fuzzy';
```
