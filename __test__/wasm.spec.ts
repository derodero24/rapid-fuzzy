import { describe, expect, it } from 'vitest';

// Check if WASM module is available and functional.
// Handles both missing binary (not built) and stale binary (outdated build).
let wasmAvailable = false;
// biome-ignore lint/suspicious/noExplicitAny: WASM module has dynamic exports
let wasm: Record<string, any> = {};
try {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const mod = require('../rapid-fuzzy.wasi.cjs');
  // Check for a recently-added export to detect stale binaries
  wasmAvailable = typeof mod.levenshtein === 'function' && typeof mod.hamming === 'function';
  if (wasmAvailable) wasm = mod;
} catch {
  wasmAvailable = false;
}

// Skip all WASM tests if the module is not available or outdated
// Build with: pnpm run build:wasm
describe.skipIf(!wasmAvailable)('wasm', () => {
  describe('exports', () => {
    it('should export all expected functions', () => {
      const expectedExports = [
        'closest',
        'damerauLevenshtein',
        'damerauLevenshteinBatch',
        'damerauLevenshteinMany',
        'hamming',
        'hammingBatch',
        'hammingMany',
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
        'partialRatio',
        'partialRatioBatch',
        'partialRatioMany',
        'search',
        'searchKeys',
        'sorensenDice',
        'sorensenDiceBatch',
        'sorensenDiceMany',
        'tokenSetRatio',
        'tokenSetRatioBatch',
        'tokenSetRatioMany',
        'tokenSortRatio',
        'tokenSortRatioBatch',
        'tokenSortRatioMany',
        'weightedRatio',
        'weightedRatioBatch',
        'weightedRatioMany',
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

    it('hamming', () => {
      expect(wasm.hamming('hello', 'hello')).toBe(0);
      expect(wasm.hamming('karolin', 'kathrin')).toBe(3);
      expect(wasm.hamming('hello', 'world')).toBe(4);
      expect(wasm.hamming('hello', 'hi')).toBeNull();
      expect(wasm.hamming('', '')).toBe(0);
    });

    it('tokenSortRatio', () => {
      expect(wasm.tokenSortRatio('hello world', 'hello world')).toBe(1.0);
      expect(wasm.tokenSortRatio('world hello', 'hello world')).toBe(1.0);
      const score = wasm.tokenSortRatio('hello', 'world');
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });

    it('tokenSetRatio', () => {
      expect(wasm.tokenSetRatio('hello world', 'hello world')).toBe(1.0);
      expect(wasm.tokenSetRatio('hello', 'hello world')).toBe(1.0);
      const score = wasm.tokenSetRatio('hello', 'world');
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });

    it('partialRatio', () => {
      expect(wasm.partialRatio('hello', 'hello')).toBe(1.0);
      expect(wasm.partialRatio('hello', 'hello world')).toBe(1.0);
      const score = wasm.partialRatio('hello', 'world');
      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });

    it('weightedRatio', () => {
      expect(wasm.weightedRatio('hello', 'hello')).toBe(1.0);
      const score = wasm.weightedRatio('hello', 'world');
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

    it('hammingBatch', () => {
      const result = wasm.hammingBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
        ['hello', 'hi'],
      ]);
      expect(result).toHaveLength(3);
      expect(result[0]).toBe(0);
      expect(result[1]).toBe(4);
      expect(result[2]).toBeNull();
    });

    it('tokenSortRatioBatch', () => {
      const result = wasm.tokenSortRatioBatch([
        ['hello world', 'hello world'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('tokenSetRatioBatch', () => {
      const result = wasm.tokenSetRatioBatch([
        ['hello world', 'hello world'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('partialRatioBatch', () => {
      const result = wasm.partialRatioBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('weightedRatioBatch', () => {
      const result = wasm.weightedRatioBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
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

    it('hammingMany', () => {
      const result = wasm.hammingMany('hello', ['hello', 'world', 'hi']);
      expect(result).toHaveLength(3);
      expect(result[0]).toBe(0);
      expect(result[1]).toBe(4);
      expect(result[2]).toBeNull();
    });

    it('tokenSortRatioMany', () => {
      const result = wasm.tokenSortRatioMany('hello world', ['hello world', 'world hello']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBe(1.0);
    });

    it('tokenSetRatioMany', () => {
      const result = wasm.tokenSetRatioMany('hello', ['hello world', 'world']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('partialRatioMany', () => {
      const result = wasm.partialRatioMany('hello', ['hello world', 'world']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
    });

    it('weightedRatioMany', () => {
      const result = wasm.weightedRatioMany('hello', ['hello', 'world']);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(1.0);
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

    it('hamming results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.hamming(a, b)).toBe(native.hamming(a, b));
      }
    });

    it('tokenSortRatio results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.tokenSortRatio(a, b)).toBe(native.tokenSortRatio(a, b));
      }
    });

    it('tokenSetRatio results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.tokenSetRatio(a, b)).toBe(native.tokenSetRatio(a, b));
      }
    });

    it('partialRatio results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.partialRatio(a, b)).toBe(native.partialRatio(a, b));
      }
    });

    it('weightedRatio results should match native', () => {
      for (const [a, b] of testPairs) {
        expect(wasm.weightedRatio(a, b)).toBe(native.weightedRatio(a, b));
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
