// Runtime tests for the wasm-bindgen build (`browser.mjs` re-exports it).
//
// The committed glue (`rapid-fuzzy-wasm-bindgen.mjs`) is loaded together with
// the compiled `rapid-fuzzy-wasm-bindgen_bg.wasm`, which is a build artifact
// (`pnpm run build:wasm-bindgen`). When the binary is missing the suite is
// skipped, except where RAPID_FUZZY_REQUIRE_WASM_BINDGEN is set (CI).
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import * as napi from '../index.js';
import type * as WasmBindgen from '../rapid-fuzzy-wasm-bindgen.mjs' with {
  'resolution-mode': 'import',
};

// The repo's tsconfig has no DOM lib and @types/node does not declare the
// WebAssembly namespace, so describe the small part of it used here.
interface WasmMemory {
  readonly buffer: ArrayBuffer;
}
type Glue = typeof WasmBindgen;
type InitSync = (input: { module: Uint8Array }) => { memory: WasmMemory };

const ROOT = join(__dirname, '..');
const WASM_PATH = join(ROOT, 'rapid-fuzzy-wasm-bindgen_bg.wasm');
const DTS_PATH = join(ROOT, 'rapid-fuzzy-wasm-bindgen.d.mts');
// A non-literal specifier keeps vitest from loading the glue when the suite is skipped.
const GLUE_SPECIFIER: string = '../rapid-fuzzy-wasm-bindgen.mjs';

const wasmAvailable = existsSync(WASM_PATH);
if (!wasmAvailable && process.env.RAPID_FUZZY_REQUIRE_WASM_BINDGEN) {
  throw new Error(`wasm-bindgen binary is required but missing: ${WASM_PATH}`);
}

let wasm: Glue;
let memory: WasmMemory;

beforeAll(async () => {
  if (!wasmAvailable) return;
  const glue = (await import(GLUE_SPECIFIER)) as Glue;
  const initSync = glue.initSync as unknown as InitSync;
  memory = initSync({ module: readFileSync(WASM_PATH) }).memory;
  wasm = glue;
});

/** Call an export with arguments its TypeScript signature rejects. */
function callUnchecked(fn: unknown, ...args: unknown[]): unknown {
  return (fn as (...a: unknown[]) => unknown)(...args);
}

/** Return the value thrown by `fn` (fails the test if nothing is thrown). */
function thrown(fn: () => unknown): unknown {
  try {
    fn();
  } catch (err) {
    return err;
  }
  throw new Error('expected the call to throw');
}

const FRUITS = ['apple', 'banana', 'grape', 'orange', 'pineapple', 'apricot'];

describe('wasm-bindgen TypeScript declarations', () => {
  // wasm-bindgen appends the initialization API (InitInput, InitOutput with the
  // raw WebAssembly exports, initSync, init); check the public API before it.
  const dts = readFileSync(DTS_PATH, 'utf8').split('export type InitInput')[0] ?? '';
  // Strip comments so prose like "returns any matches" cannot cause false positives.
  const code = dts.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');

  it('contains no `any`', () => {
    expect(code.match(/\bany\b/g) ?? []).toEqual([]);
  });

  it('declares every SearchOptions field as optional', () => {
    const body = /export interface SearchOptions \{([^}]*)\}/.exec(code)?.[1] ?? '';
    const fields = body
      .split(';')
      .map((f) => f.trim())
      .filter(Boolean);
    expect(fields.length).toBe(5);
    for (const field of fields) {
      expect(field).toMatch(/^\w+\?: /);
    }
  });

  it('uses camelCase parameter names', () => {
    expect(code.match(/\b[a-z]+(?:_[a-z]+)+\??:/g) ?? []).toEqual([]);
  });

  it('names the *Many threshold parameters like the Node.js binding', () => {
    expect(code).toMatch(/levenshteinMany\([^)]*maxDistance\?: number \| null\)/);
    expect(code).toMatch(/jaroWinklerMany\([^)]*minSimilarity\?: number \| null\)/);
    expect(code).not.toMatch(/score_cutoff|scoreCutoff/);
  });
});

describe.skipIf(!wasmAvailable)('wasm-bindgen runtime', () => {
  describe('search options', () => {
    it('accepts a number as maxResults shorthand (search)', () => {
      expect(wasm.search('a', FRUITS, 2)).toHaveLength(2);
      expect(wasm.search('a', FRUITS, 2)).toEqual(wasm.search('a', FRUITS, { maxResults: 2 }));
    });

    it('accepts a number as maxResults shorthand (FuzzyIndex)', () => {
      const index = new wasm.FuzzyIndex(FRUITS);
      expect(index.search('a', 2)).toHaveLength(2);
      expect(index.searchIndices('a', 2)).toHaveLength(2);
      index.free();
    });

    it('treats null, undefined and {} as default options', () => {
      const all = wasm.search('a', FRUITS);
      expect(wasm.search('a', FRUITS, null)).toEqual(all);
      expect(wasm.search('a', FRUITS, undefined)).toEqual(all);
      expect(wasm.search('a', FRUITS, {})).toEqual(all);
    });

    it('rejects options of the wrong type with a TypeError', () => {
      for (const bad of ['3', true, { maxResults: 'x' }, { maxResults: -1 }, { minScore: 'x' }]) {
        const err = thrown(() => callUnchecked(wasm.search, 'a', FRUITS, bad));
        expect(err).toBeInstanceOf(TypeError);
      }
      const index = new wasm.FuzzyIndex(FRUITS);
      expect(thrown(() => callUnchecked(index.search.bind(index), 'a', 'x'))).toBeInstanceOf(
        TypeError,
      );
      const keyed = new wasm.KeyedFuzzyIndex([FRUITS], [1]);
      expect(thrown(() => callUnchecked(keyed.search.bind(keyed), 'a', 'x'))).toBeInstanceOf(
        TypeError,
      );
      index.free();
      keyed.free();
    });

    it('does not leak wasm memory when options are rejected', () => {
      const items = Array.from({ length: 2000 }, (_, i) => `item number ${i} with padding`);
      const run = (n: number): void => {
        for (let i = 0; i < n; i++) {
          thrown(() => callUnchecked(wasm.search, 'item', items, { maxResults: 'x' }));
        }
      };
      run(20);
      const before = memory.buffer.byteLength;
      run(300);
      // Before the fix every rejected call leaked the converted `items` (~100 KB).
      expect(memory.buffer.byteLength - before).toBeLessThan(1024 * 1024);
    });
  });

  describe('null results (parity with the Node.js binding)', () => {
    it('closest returns null when nothing matches', () => {
      expect(wasm.closest('zzz', FRUITS)).toBeNull();
      expect(wasm.closest('a', [])).toBeNull();
      expect(wasm.closest('aple', FRUITS)).toBe('apple');
      const index = new wasm.FuzzyIndex(FRUITS);
      expect(index.closest('zzz')).toBeNull();
      expect(index.closest('aple')).toBe('apple');
      index.free();
      const keyed = new wasm.KeyedFuzzyIndex([FRUITS], [1]);
      expect(keyed.closest('zzz')).toBeNull();
      expect(keyed.closest('aple')).toBe(0);
      keyed.free();
    });

    it('hamming variants use null for undefined distances', () => {
      expect(wasm.hamming('abc', 'ab')).toBeNull();
      expect(wasm.normalizedHamming('abc', 'ab')).toBeNull();
      const pairs = [
        ['karolin', 'kathrin'],
        ['abc', 'ab'],
      ];
      expect(wasm.hammingBatch(pairs)).toEqual([3, null]);
      expect(wasm.hammingBatch(pairs)).toEqual(napi.hammingBatch(pairs));
      expect(wasm.normalizedHammingBatch(pairs)).toEqual(napi.normalizedHammingBatch(pairs));
      // A pair without exactly two strings throws instead of giving null.
      expect(thrown(() => wasm.hammingBatch([['x']]))).toBeInstanceOf(Error);
      expect(() => napi.hammingBatch([['x']])).toThrow();
      const candidates = ['abd', 'xyz', 'abcd'];
      expect(wasm.hammingMany('abc', candidates)).toEqual([1, 3, null]);
      expect(wasm.hammingMany('abc', candidates, 1)).toEqual(
        napi.hammingMany('abc', candidates, 1),
      );
      expect(wasm.normalizedHammingMany('abc', candidates, 0.5)).toEqual(
        napi.normalizedHammingMany('abc', candidates, 0.5),
      );
    });
  });

  describe('errors are Error objects', () => {
    it('KeyedFuzzyIndex validation errors', () => {
      const mismatch = thrown(() => new wasm.KeyedFuzzyIndex([['a', 'b'], ['c']], [1, 1]));
      expect(mismatch).toBeInstanceOf(Error);
      expect((mismatch as Error).message).toMatch(/same length/);

      const keyed = new wasm.KeyedFuzzyIndex([['a'], ['b']], [1, 1]);
      expect(thrown(() => keyed.add(['only one']))).toBeInstanceOf(Error);
      expect(thrown(() => keyed.addMany([['c', 'd'], ['e']]))).toBeInstanceOf(Error);
      expect(thrown(() => callUnchecked(keyed.add.bind(keyed), 'not an array'))).toBeInstanceOf(
        TypeError,
      );
      keyed.free();

      for (const keyTexts of ['x', null, [[1]], {}]) {
        const err = thrown(() => Reflect.construct(wasm.KeyedFuzzyIndex, [keyTexts, [1]]));
        expect(err).toBeInstanceOf(TypeError);
        expect((err as TypeError).message).toMatch(/^Invalid keyTexts: /);
      }
      expect(thrown(() => new wasm.KeyedFuzzyIndex([['a']], [-1]))).toBeInstanceOf(Error);
    });

    it('deserialize rejects invalid data with an Error', () => {
      const bad = new Uint8Array([1, 2, 3]);
      const err = thrown(() => wasm.FuzzyIndex.deserialize(bad));
      expect(err).toBeInstanceOf(Error);
      expect((err as Error).message).toMatch(/Invalid data/);
      expect(typeof (err as Error).stack).toBe('string');
      expect(thrown(() => wasm.KeyedFuzzyIndex.deserialize(bad))).toBeInstanceOf(Error);
    });

    it('rejects malformed array arguments with a TypeError', () => {
      expect(thrown(() => callUnchecked(wasm.levenshteinBatch, 'x'))).toBeInstanceOf(TypeError);
      expect(thrown(() => callUnchecked(wasm.hammingBatch, [[1, 2]]))).toBeInstanceOf(TypeError);
      expect(thrown(() => callUnchecked(wasm.searchKeys, 'a', 'x', [1]))).toBeInstanceOf(TypeError);
    });
  });

  describe('typed inputs', () => {
    it('accepts weights as number[] or Float64Array', () => {
      const keyTexts = [FRUITS, FRUITS.map((f) => f.toUpperCase())];
      const fromArray = wasm.searchKeys('ap', keyTexts, [2, 1]);
      expect(fromArray.length).toBeGreaterThan(0);
      expect(wasm.searchKeys('ap', keyTexts, new Float64Array([2, 1]))).toEqual(fromArray);
      const index = new wasm.KeyedFuzzyIndex(keyTexts, [2, 1]);
      expect(index.search('ap')).toEqual(fromArray);
      index.free();
    });
  });

  describe('parity with the Node.js binding', () => {
    const items = [
      'TypeScript',
      'JavaScript',
      'Python',
      'TypeSpec',
      'type theory',
      'typewriter',
      'Rust',
      'protobuf',
    ];

    it('search results match', () => {
      for (const opts of [
        undefined,
        3,
        { includePositions: true },
        { minScore: 0.3, maxResults: 4 },
        { isCaseSensitive: true },
      ]) {
        expect(wasm.search('type', items, opts)).toEqual(napi.search('type', items, opts));
      }
      const all = { returnAllOnEmpty: true, maxResults: 3 };
      expect(wasm.search('', items, all)).toEqual(napi.search('', items, all));
    });

    it('omits matchType unless positions are requested', () => {
      const [plain] = wasm.search('type', items);
      expect(plain).toBeDefined();
      expect(Object.keys(plain ?? {})).toEqual(['item', 'score', 'index', 'positions']);
      const [withPositions] = wasm.search('type', items, { includePositions: true });
      expect(withPositions?.matchType).toBe('Prefix');
    });

    it('FuzzyIndex results match', () => {
      const w = new wasm.FuzzyIndex(items);
      const n = new napi.FuzzyIndex(items);
      expect(w.size).toBe(n.size);
      expect(w.search('type', { includePositions: true })).toEqual(
        n.search('type', { includePositions: true }),
      );
      expect(w.searchIndices('type', 2)).toEqual(n.searchIndices('type', 2));
      expect(w.closest('pyhton')).toBe(n.closest('pyhton'));
      w.free();
    });

    it('KeyedFuzzyIndex and searchKeys results match', () => {
      const keyTexts = [items, items.map((s) => s.toLowerCase())];
      const opts = { maxResults: 5 };
      expect(wasm.searchKeys('type', keyTexts, [1, 2], opts)).toEqual(
        napi.searchKeys('type', keyTexts, [1, 2], opts),
      );
      const w = new wasm.KeyedFuzzyIndex(keyTexts, [1, 2]);
      const n = new napi.KeyedFuzzyIndex(keyTexts, [1, 2]);
      expect(w.search('type', opts)).toEqual(n.search('type', opts));
      expect(w.closest('pyhton')).toBe(n.closest('pyhton'));
      w.free();
    });
  });

  describe('API parity additions', () => {
    it('FuzzyIndex.fromAsync resolves to a FuzzyIndex', async () => {
      const promise = wasm.FuzzyIndex.fromAsync(FRUITS);
      expect(promise).toBeInstanceOf(Promise);
      const index = await promise;
      expect(index).toBeInstanceOf(wasm.FuzzyIndex);
      expect(index.size).toBe(FRUITS.length);
      expect(index.closest('aple')).toBe('apple');
      index.free();
    });

    it('size is a getter on both index classes', () => {
      for (const cls of [wasm.FuzzyIndex, wasm.KeyedFuzzyIndex]) {
        const descriptor = Object.getOwnPropertyDescriptor(cls.prototype, 'size');
        expect(typeof descriptor?.get).toBe('function');
      }
    });
  });
});
