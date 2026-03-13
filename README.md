# rapid-fuzzy

Rust-powered fuzzy search and string distance for JavaScript/TypeScript.

> **Status**: Work in progress. Not yet published to npm.

## Features

- **Fast**: 10-50x faster than fuse.js/leven for large datasets (Rust + SIMD)
- **Universal**: Works in Node.js (native), browsers (WASM), Deno, and Bun
- **Zero JS dependencies**: Pure Rust core with napi-rs bindings
- **Type-safe**: Full TypeScript support with auto-generated type definitions
- **Drop-in**: API compatible with popular fuzzy search libraries

## Installation

```bash
npm install rapid-fuzzy
# or
pnpm add rapid-fuzzy
```

## Usage

### String Distance

```typescript
import { levenshtein, jaroWinkler, sorensenDice } from 'rapid-fuzzy';

levenshtein('kitten', 'sitting');     // 3
jaroWinkler('MARTHA', 'MARHTA');      // 0.961
sorensenDice('night', 'nacht');       // 0.25
```

### Fuzzy Search

```typescript
import { search, closest } from 'rapid-fuzzy';

// Find matches sorted by relevance
const results = search('typscript', [
  'TypeScript',
  'JavaScript',
  'Python',
  'TypeSpec',
]);
// → [{ item: 'TypeScript', score: 89, index: 0 }, ...]

// Find the single best match
closest('tsc', ['TypeScript', 'JavaScript', 'Python']);
// → 'TypeScript'
```

## Benchmarks

Coming soon.

## License

MIT
