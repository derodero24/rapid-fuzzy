import { describe, expect, test } from 'bun:test';

// Bun does not support node:wasi (wasi.initialize is undefined),
// so we use the browser loader which works via @napi-rs/wasm-runtime.
const wasm = await import('../rapid-fuzzy.wasi-browser.js');

describe('WASM on Bun (browser loader)', () => {
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
  });

  describe('batch functions', () => {
    test('levenshteinBatch', () => {
      expect(
        wasm.levenshteinBatch([
          ['hello', 'hello'],
          ['hello', 'world'],
        ]),
      ).toEqual([0, 4]);
    });
  });

  describe('many functions', () => {
    test('levenshteinMany', () => {
      const result = wasm.levenshteinMany('hello', ['hello', 'world', 'help']);
      expect(result).toHaveLength(3);
      expect(result[0]).toBe(0);
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
});
