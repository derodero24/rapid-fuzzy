import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// Bun 1.x does not implement the TC39 WebAssembly ESM integration that
// wasm-bindgen's bundler target relies on (Bun treats .wasm imports as URL
// strings rather than instantiated module exports). We instantiate the WASM
// binary manually via the Node.js-compatible WebAssembly API instead.
import * as wasm from '../rapid-fuzzy-wasm-bindgen_bg.js';

const wasmPath = join(
  dirname(fileURLToPath(import.meta.url)),
  '../rapid-fuzzy-wasm-bindgen_bg.wasm',
);
const wasmModule = new WebAssembly.Module(readFileSync(wasmPath));
const wasmInstance = new WebAssembly.Instance(wasmModule, {
  './rapid-fuzzy-wasm-bindgen_bg.js': wasm,
});
wasm.__wbg_set_wasm(wasmInstance.exports);

describe('WASM on Bun (wasm-bindgen)', () => {
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
      ).toEqual(new Uint32Array([0, 4]));
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

    test('closest returns undefined for empty items', () => {
      expect(wasm.closest('hello', [])).toBeUndefined();
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
