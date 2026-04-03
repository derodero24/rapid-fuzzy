# Migrating from FlexSearch to rapid-fuzzy

[FlexSearch](https://www.npmjs.com/package/flexsearch) is a full-text search engine that uses inverted indexes and tokenization. rapid-fuzzy is a fuzzy matching library that finds approximate string matches using edit distance and similarity algorithms. They serve different primary use cases, but rapid-fuzzy can replace FlexSearch for autocomplete and typo-tolerant search scenarios.

## When to switch

**Switch to rapid-fuzzy when**:
- You need typo tolerance (e.g., "typscript" → "TypeScript")
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
| `index.remove(id)` | `index.remove(index)` | Removes by position |

## Code Examples

### Basic search

```typescript
// Before (FlexSearch)
import { Index } from 'flexsearch';
const index = new Index();
items.forEach((item, i) => index.add(i, item));
const ids = index.search('typscript'); // [0] — returns IDs only

// After (rapid-fuzzy)
import { FuzzyIndex } from 'rapid-fuzzy';
const index = new FuzzyIndex(items);
const results = index.search('typscript');
// [{ item: 'TypeScript', score: 0.85, index: 0, positions: [] }]
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

### Typo tolerance

FlexSearch matches exact tokens — "typscript" won't find "TypeScript". rapid-fuzzy uses fuzzy algorithms that handle typos, transpositions, and partial matches automatically.

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

Access 10 distance algorithms for specialized use cases:

```typescript
import { levenshtein, jaroWinkler, sorensenDice } from 'rapid-fuzzy';
```
