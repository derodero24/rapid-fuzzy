import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const wasmBinaryPath = resolve(__dirname, '../rapid-fuzzy.wasm32-wasi.wasm');
const wasmExists = existsSync(wasmBinaryPath);

// Skip all WASM tests if the binary is not built
// Build with: pnpm run build --target wasm32-wasip1-threads
describe.skipIf(!wasmExists)('wasm', () => {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const wasm = wasmExists ? require('../rapid-fuzzy.wasi.cjs') : {};

  describe('exports', () => {
    it('should export all expected functions', () => {
      const expectedExports = [
        'closest',
        'damerauLevenshtein',
        'damerauLevenshteinBatch',
        'damerauLevenshteinMany',
        'jaro',
        'jaroBatch',
        'jaroMany',
        'jaroWinkler',
        'jaroWinklerBatch',
        'jaroWinklerMany',
        'levenshtein',
        'levenshteinBatch',
        'levenshteinMany',
        'normalizedLevenshtein',
        'normalizedLevenshteinBatch',
        'normalizedLevenshteinMany',
        'search',
        'sorensenDice',
        'sorensenDiceBatch',
        'sorensenDiceMany',
      ];

      for (const name of expectedExports) {
        expect(typeof wasm[name]).toBe('function');
      }
    });
  });

  describe('distance functions', () => {
    it('levenshtein', () => {
      expect(wasm.levenshtein('hello', 'hello')).toBe(0);
      expect(wasm.levenshtein('hello', 'world')).toBe(4);
      expect(wasm.levenshtein('kitten', 'sitting')).toBe(3);
      expect(wasm.levenshtein('', '')).toBe(0);
    });

    it('normalizedLevenshtein', () => {
      expect(wasm.normalizedLevenshtein('hello', 'hello')).toBe(1.0);
      expect(wasm.normalizedLevenshtein('', '')).toBe(1.0);
      const score = wasm.normalizedLevenshtein('hello', 'world');
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });

    it('damerauLevenshtein', () => {
      expect(wasm.damerauLevenshtein('hello', 'hello')).toBe(0);
      expect(wasm.damerauLevenshtein('hello', 'ehllo')).toBe(1);
    });

    it('jaro', () => {
      expect(wasm.jaro('hello', 'hello')).toBe(1.0);
      const score = wasm.jaro('hello', 'world');
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });

    it('jaroWinkler', () => {
      expect(wasm.jaroWinkler('hello', 'hello')).toBe(1.0);
      const score = wasm.jaroWinkler('hello', 'world');
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });

    it('sorensenDice', () => {
      expect(wasm.sorensenDice('hello', 'hello')).toBe(1.0);
      const score = wasm.sorensenDice('hello', 'world');
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });
  });

  describe('batch functions', () => {
    it('levenshteinBatch', () => {
      const result = wasm.levenshteinBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toEqual([0, 4]);
    });

    it('jaroBatch', () => {
      const result = wasm.jaroBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeGreaterThanOrEqual(0);
    });

    it('jaroWinklerBatch', () => {
      const result = wasm.jaroWinklerBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('sorensenDiceBatch', () => {
      const result = wasm.sorensenDiceBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('normalizedLevenshteinBatch', () => {
      const result = wasm.normalizedLevenshteinBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('damerauLevenshteinBatch', () => {
      const result = wasm.damerauLevenshteinBatch([
        ['hello', 'hello'],
        ['hello', 'ehllo'],
      ]);
      expect(result).toEqual([0, 1]);
    });
  });

  describe('many functions', () => {
    it('levenshteinMany', () => {
      const result = wasm.levenshteinMany('hello', ['hello', 'world', 'help']);
      expect(result).toHaveLength(3);
      expect(result[0]).toBe(0);
    });

    it('jaroMany', () => {
      const result = wasm.jaroMany('hello', ['hello', 'world']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('jaroWinklerMany', () => {
      const result = wasm.jaroWinklerMany('hello', ['hello', 'world']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('sorensenDiceMany', () => {
      const result = wasm.sorensenDiceMany('hello', ['hello', 'world']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('normalizedLevenshteinMany', () => {
      const result = wasm.normalizedLevenshteinMany('hello', ['hello', 'world']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('damerauLevenshteinMany', () => {
      const result = wasm.damerauLevenshteinMany('hello', ['hello', 'ehllo']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(0);
    });
  });

  describe('search functions', () => {
    it('search should return sorted results', () => {
      const results = wasm.search('type', ['TypeScript', 'JavaScript', 'Python', 'TypeSpec']);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].item).toBe('TypeScript');
      expect(results[0]).toHaveProperty('score');
      expect(results[0]).toHaveProperty('index');
    });

    it('search should respect maxResults', () => {
      const results = wasm.search('a', ['apple', 'avocado', 'apricot', 'banana'], 2);
      expect(results.length).toBeLessThanOrEqual(2);
    });

    it('search should return empty for empty query', () => {
      const results = wasm.search('', ['hello', 'world']);
      expect(results).toEqual([]);
    });

    it('closest should return best match', () => {
      const result = wasm.closest('apple', ['application', 'banana', 'apple pie']);
      expect(result).not.toBeNull();
    });

    it('closest should return null for empty items', () => {
      const result = wasm.closest('hello', []);
      expect(result).toBeNull();
    });
  });

  describe('native parity', () => {
    // Import native module for comparison
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const native = require('../index.js');

    const testPairs: [string, string][] = [
      ['hello', 'hello'],
      ['hello', 'world'],
      ['kitten', 'sitting'],
      ['', ''],
      ['abc', ''],
      ['', 'abc'],
      ['café', 'cafe'],
      ['東京', '京都'],
    ];

    it('levenshtein results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.levenshtein(a, b)).toBe(native.levenshtein(a, b));
      }
    });

    it('normalizedLevenshtein results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.normalizedLevenshtein(a, b)).toBe(native.normalizedLevenshtein(a, b));
      }
    });

    it('jaro results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.jaro(a, b)).toBe(native.jaro(a, b));
      }
    });

    it('jaroWinkler results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.jaroWinkler(a, b)).toBe(native.jaroWinkler(a, b));
      }
    });

    it('sorensenDice results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.sorensenDice(a, b)).toBe(native.sorensenDice(a, b));
      }
    });

    it('damerauLevenshtein results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.damerauLevenshtein(a, b)).toBe(native.damerauLevenshtein(a, b));
      }
    });

    it('batch results should match native', () => {
      const pairs: [string, string][] = [
        ['hello', 'world'],
        ['kitten', 'sitting'],
      ];
      expect(wasm.levenshteinBatch(pairs)).toEqual(native.levenshteinBatch(pairs));
    });

    it('searchKeys results should match native', () => {
      const names = ['John Smith', 'Jane Doe', 'Bob Johnson'];
      const emails = ['john@example.com', 'jane@example.com', 'bob@test.com'];
      const weights = [1, 1];

      const wasmResults = wasm.searchKeys('john', [names, emails], weights);
      const nativeResults = native.searchKeys('john', [names, emails], weights);

      expect(wasmResults.length).toBe(nativeResults.length);
      for (let i = 0; i < wasmResults.length; i++) {
        expect(wasmResults[i]?.index).toBe(nativeResults[i]?.index);
        expect(wasmResults[i]?.score).toBeCloseTo(nativeResults[i]?.score ?? 0);
        expect(wasmResults[i]?.keyScores).toHaveLength(nativeResults[i]?.keyScores.length ?? 0);
      }
    });
  });
});
