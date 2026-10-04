import { describe, expect, test } from 'bun:test';

// Run with `bun test --conditions=browser` (pnpm run test:bun): the package then
// resolves to its WebAssembly build (browser.mjs) instead of the Node-API addon
// Bun loads by default. The package imports itself by name, so this exercises
// the published entry points and their export conditions.
import * as wasm from 'rapid-fuzzy';
import { highlight } from 'rapid-fuzzy/highlight';
import { FuzzyObjectIndex } from 'rapid-fuzzy/objects';

describe('WASM on Bun (wasm-bindgen)', () => {
  test('resolves the WebAssembly build', () => {
    expect(import.meta.resolve('rapid-fuzzy')).toEndWith('/browser.mjs');
    expect(import.meta.resolve('rapid-fuzzy/highlight')).toEndWith('/highlight.browser.mjs');
    expect(import.meta.resolve('rapid-fuzzy/objects')).toEndWith('/browser.mjs');
  });

  describe('distance functions', () => {
    test('levenshtein', () => {
      expect(wasm.levenshtein('hello', 'hello')).toBe(0);
      expect(wasm.levenshtein('kitten', 'sitting')).toBe(3);
    });

    test('normalizedLevenshtein', () => {
      expect(wasm.normalizedLevenshtein('hello', 'hello')).toBe(1.0);
    });

    test('damerauLevenshtein', () => {
      expect(wasm.damerauLevenshtein('hello', 'ehllo')).toBe(1);
    });

    test('jaro', () => {
      expect(wasm.jaro('hello', 'hello')).toBe(1.0);
    });

    test('jaroWinkler', () => {
      expect(wasm.jaroWinkler('hello', 'hello')).toBe(1.0);
    });

    test('sorensenDice', () => {
      expect(wasm.sorensenDice('hello', 'hello')).toBe(1.0);
    });

    test('hamming', () => {
      expect(wasm.hamming('hello', 'hello')).toBe(0);
      expect(wasm.hamming('karolin', 'kathrin')).toBe(3);
      expect(wasm.hamming('hello', 'hi')).toBeNull();
    });

    test('indel', () => {
      expect(wasm.indel('hello', 'hello')).toBe(0);
      expect(wasm.indel('abc', 'ac')).toBe(1);
    });

    test('normalizedIndel', () => {
      expect(wasm.normalizedIndel('hello', 'hello')).toBe(1.0);
    });

    test('normalizedHamming', () => {
      expect(wasm.normalizedHamming('hello', 'hello')).toBe(1.0);
      expect(wasm.normalizedHamming('hello', 'hi')).toBeNull();
    });
  });

  describe('batch functions', () => {
    test('levenshteinBatch', () => {
      expect(
        wasm.levenshteinBatch([
          ['hello', 'hello'],
          ['hello', 'world'],
        ]),
      ).toEqual(new Uint32Array([0, 4]));
    });

    test('indelBatch', () => {
      const result = wasm.indelBatch([
        ['hello', 'hello'],
        ['abc', 'ac'],
      ]);
      expect(Array.from(result)).toEqual([0, 1]);
    });

    test('normalizedHammingBatch', () => {
      const result = wasm.normalizedHammingBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });
  });

  describe('many functions', () => {
    test('levenshteinMany', () => {
      const result = wasm.levenshteinMany('hello', ['hello', 'world', 'help']);
      expect(result).toHaveLength(3);
      expect(result[0]).toBe(0);
    });

    test('indelMany', () => {
      const result = wasm.indelMany('abc', ['abc', 'ac', '']);
      expect(result).toHaveLength(3);
      expect(result[0]).toBe(0);
      expect(result[1]).toBe(1);
    });

    test('normalizedHammingMany', () => {
      const result = wasm.normalizedHammingMany('hello', ['hello', 'world', 'hi']);
      expect(result).toHaveLength(3);
      expect(result[0]).toBe(1.0);
      // Length mismatches are null, as in the Node.js binding
      expect(result[2]).toBeNull();
    });
  });

  describe('token-based functions', () => {
    test('tokenSortRatio', () => {
      expect(wasm.tokenSortRatio('New York Mets', 'Mets New York')).toBe(1.0);
    });

    test('partialRatio', () => {
      expect(wasm.partialRatio('hello', 'hello world')).toBe(1.0);
    });

    test('weightedRatio', () => {
      expect(wasm.weightedRatio('hello', 'hello')).toBe(1.0);
    });
  });

  describe('search', () => {
    test('search returns results', () => {
      const results = wasm.search('type', ['TypeScript', 'JavaScript', 'Python']);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].item).toBe('TypeScript');
    });

    test('search returns empty for empty query', () => {
      expect(wasm.search('', ['hello'])).toEqual([]);
    });
  });

  describe('closest', () => {
    test('closest returns best match', () => {
      const result = wasm.closest('apple', ['application', 'banana', 'apple pie']);
      expect(result).not.toBeNull();
    });

    test('closest returns null for empty items', () => {
      expect(wasm.closest('hello', [])).toBeNull();
    });
  });

  describe('FuzzyIndex', () => {
    test('constructor, search, and lifecycle', () => {
      const index = new wasm.FuzzyIndex(['apple', 'banana', 'grape', 'orange']);
      expect(index.size).toBe(4);

      const results = index.search('aple');
      expect(results.length).toBeGreaterThan(0);
      expect(results.some((r: { item: string }) => r.item === 'apple')).toBe(true);

      expect(index.closest('aple')).toBe('apple');

      index.add('mango');
      expect(index.size).toBe(5);

      index.destroy();
      expect(index.size).toBe(0);
    });
  });

  describe('subpath exports', () => {
    test('highlight', () => {
      const [hit] = wasm.search('fzy', ['fuzzy'], { includePositions: true });
      expect(hit && highlight(hit.item, hit.positions, '[', ']')).toBe('[f]uz[zy]');
    });

    test('FuzzyObjectIndex', () => {
      const index = new FuzzyObjectIndex([{ name: 'Jane' }, { name: 'John' }], {
        keys: ['name'],
      });
      expect(index.search('jane')[0]?.item).toEqual({ name: 'Jane' });
      expect(index.closest('jon')).toEqual({ name: 'John' });
    });
  });
});
