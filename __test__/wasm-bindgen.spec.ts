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

/**
 * Values that are neither a `scoreMode` nor a `matchMode`: every build
 * rejects them in an options object, and reports a value that is not a
 * string by its `typeof` (`null` as null).
 */
const BAD_MODES: readonly unknown[] = [
  'mean',
  'Max',
  'cross',
  'CrossKey',
  '',
  1,
  true,
  {},
  null,
  1n,
  Symbol('x'),
  () => {},
  new String('max'),
];

/** A keyed-search mode option set to a value that is not a mode. */
interface BadModeCase {
  field: 'scoreMode' | 'matchMode';
  bad: unknown;
  /** What the message of the Node.js binding must match. */
  nodeMessage: RegExp;
  keyTexts: string[][];
  weights: number[];
  /** `closest()` of a wasm `KeyedFuzzyIndex`. */
  closest: (options: WasmBindgen.KeyClosestOptions) => unknown;
  /** `search()` of the same index. */
  search: (options: WasmBindgen.KeySearchOptions) => unknown;
}

/**
 * Expect `{ [field]: bad }` to be rejected by wasm `searchKeys()`,
 * `KeyedFuzzyIndex.search()` and `KeyedFuzzyIndex.closest()` with a
 * `TypeError` carrying the message of the Node.js binding, which rejects it
 * in `closest()` too.
 */
function expectModeRejected(c: BadModeCase): void {
  const options = { [c.field]: c.bad } as WasmBindgen.KeySearchOptions &
    WasmBindgen.KeyClosestOptions;
  const nodeError = thrown(() =>
    callUnchecked(napi.searchKeys, 'type', c.keyTexts, c.weights, options),
  );
  const nodeMessage = nodeError instanceof Error ? nodeError.message : '';
  expect(nodeMessage).toMatch(c.nodeMessage);
  const nodeIndex = new napi.KeyedFuzzyIndex(c.keyTexts, c.weights);
  expect(thrown(() => nodeIndex.closest('type', options))).toHaveProperty('message', nodeMessage);
  const errors = [
    thrown(() => wasm.searchKeys('type', c.keyTexts, c.weights, options)),
    thrown(() => c.search(options)),
    thrown(() => c.closest(options)),
  ];
  for (const err of errors) {
    expect(err).toBeInstanceOf(TypeError);
    expect(err).toHaveProperty('message', expect.stringContaining(nodeMessage));
  }
}

/**
 * The fields of `export interface name` in the wasm-bindgen declarations, as
 * `[field, type]` pairs (`field` keeps its `?`).
 */
function declaredFields(name: string): Array<[field: string, type: string]> {
  const code = readFileSync(DTS_PATH, 'utf8')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/\/\/.*$/gm, '');
  const body = new RegExp(`export interface ${name} \\{([^}]*)\\}`).exec(code)?.[1] ?? '';
  return body
    .split(';')
    .map((field) => field.trim())
    .filter(Boolean)
    .map((field): [string, string] => {
      const colon = field.indexOf(':');
      return [field.slice(0, colon), field.slice(colon + 1).trim()];
    });
}

/** The options object `name` with every declared field set to undefined. */
function unsetOptions(name: string): Record<string, undefined> {
  return Object.fromEntries(
    declaredFields(name).map(([field]) => [field.replace('?', ''), undefined]),
  );
}

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

  it('lets every field of the options interfaces be set to undefined, like index.d.ts', () => {
    for (const name of ['SearchOptions', 'KeySearchOptions', 'KeyClosestOptions']) {
      const fields = declaredFields(name);
      expect(fields.length, name).toBeGreaterThan(2);
      for (const [field, type] of fields) {
        expect(field, name).toMatch(/^\w+\?$/);
        expect(type, `${name}.${field}`).toMatch(/ \| undefined$/);
      }
    }
    // Result fields are omitted when unset, never undefined.
    for (const name of ['SearchResult', 'IndexSearchResult']) {
      expect(declaredFields(name), name).toContainEqual(['matchType?', 'MatchType']);
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

    it('treats an options field set to undefined as unset, as declared', () => {
      const searchUnset = unsetOptions('SearchOptions');
      const keyUnset = unsetOptions('KeySearchOptions');
      const closestUnset = unsetOptions('KeyClosestOptions');
      expect(Object.keys(searchUnset)).toHaveLength(5);
      expect(Object.keys(keyUnset)).toHaveLength(7);
      expect(Object.keys(closestUnset)).toEqual(['minScore', 'scoreMode', 'matchMode']);
      const keyTexts = [FRUITS, FRUITS.map((f) => f.toUpperCase())];
      const index = new wasm.FuzzyIndex(FRUITS);
      const keyed = new wasm.KeyedFuzzyIndex(keyTexts, [2, 1]);
      const n = new napi.KeyedFuzzyIndex(keyTexts, [2, 1]);
      for (const query of ['a', 'ap', 'zzz', '']) {
        expect(callUnchecked(wasm.search, query, FRUITS, searchUnset)).toEqual(
          wasm.search(query, FRUITS),
        );
        expect(callUnchecked(index.search.bind(index), query, searchUnset)).toEqual(
          index.search(query),
        );
        expect(callUnchecked(index.searchIndices.bind(index), query, searchUnset)).toEqual(
          index.searchIndices(query),
        );
        const keyedResults = wasm.searchKeys(query, keyTexts, [2, 1]);
        expect(callUnchecked(wasm.searchKeys, query, keyTexts, [2, 1], keyUnset)).toEqual(
          keyedResults,
        );
        expect(callUnchecked(keyed.search.bind(keyed), query, keyUnset)).toEqual(keyedResults);
        expect(callUnchecked(keyed.closest.bind(keyed), query, closestUnset)).toBe(
          keyed.closest(query),
        );
        // The Node.js binding reads the same objects the same way.
        expect(callUnchecked(napi.search, query, FRUITS, searchUnset)).toEqual(
          wasm.search(query, FRUITS),
        );
        expect(callUnchecked(n.search.bind(n), query, keyUnset)).toEqual(keyedResults);
        expect(callUnchecked(n.closest.bind(n), query, closestUnset)).toBe(keyed.closest(query));
      }
      index.free();
      keyed.free();
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
      // Syntax-only and Unicode-whitespace queries count as empty, like in napi.
      for (const query of ['', '^', "'", '!', '^$', '\u3000']) {
        expect(wasm.search(query, items, all)).toEqual(napi.search(query, items, all));
        const index = new wasm.FuzzyIndex(items);
        expect(index.search(query, all)).toEqual(new napi.FuzzyIndex(items).search(query, all));
        index.free();
      }
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

    describe('scoreMode', () => {
      const keyTexts = [items, items.map((s) => `${s.toLowerCase()} lang`), items.map(() => 'x')];
      const weights = [2, 1, 0.5];
      const modes = ['weighted', 'matched', 'max'] as const;

      it('gives the same results in every mode', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        const cases = [...modes, undefined].flatMap((scoreMode) =>
          [undefined, 0.5, 0.9].flatMap((minScore) =>
            ['type', 'lang', 'script', 'rust', 'zzz'].map((query) => ({
              query,
              minScore,
              scoreMode,
            })),
          ),
        );
        for (const { query, minScore, scoreMode } of cases) {
          const options: WasmBindgen.KeySearchOptions = { maxResults: 4, scoreMode, minScore };
          const expected = napi.searchKeys(query, keyTexts, weights, options);
          expect(wasm.searchKeys(query, keyTexts, weights, options)).toEqual(expected);
          expect(w.search(query, options)).toEqual(expected);
          expect(n.search(query, options)).toEqual(expected);
          const closestOptions: WasmBindgen.KeyClosestOptions = { scoreMode, minScore };
          expect(w.closest(query, closestOptions)).toBe(n.closest(query, closestOptions));
        }
        w.free();
      });

      it('rejects unknown modes with a TypeError carrying the Node.js message', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        for (const bad of BAD_MODES) {
          expectModeRejected({
            field: 'scoreMode',
            bad,
            nodeMessage: /^scoreMode must be "weighted", "matched" or "max", got /,
            keyTexts,
            weights,
            closest: (options) => w.closest('type', options),
            search: (options) => w.search('type', options),
          });
        }
        w.free();
      });

      it('rejects invalid KeySearchOptions with a TypeError', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const invalid: unknown[] = [
          '3',
          true,
          { maxResults: 'x' },
          { maxResults: -1 },
          { minScore: 'x' },
          { maxResults: 0.5, scoreMode: 'max' },
          { isCaseSensitive: 'yes', scoreMode: 'matched' },
        ];
        for (const options of invalid) {
          expect(
            thrown(() => callUnchecked(wasm.searchKeys, 'a', keyTexts, weights, options)),
          ).toBeInstanceOf(TypeError);
          expect(thrown(() => callUnchecked(w.search.bind(w), 'a', options))).toBeInstanceOf(
            TypeError,
          );
        }
        w.free();
      });

      it('reads valid KeySearchOptions like the Node.js binding', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const valid: unknown[] = [
          {},
          { maxResults: 2 },
          { maxResults: Number.POSITIVE_INFINITY, scoreMode: 'max' },
          { maxResults: undefined, minScore: undefined, scoreMode: undefined },
          { isCaseSensitive: true, scoreMode: 'matched' },
          { returnAllOnEmpty: true, maxResults: 3, scoreMode: 'matched' },
          { includePositions: true },
        ];
        const cases = valid.flatMap((options) =>
          ['type', 'T', ''].map((query) => ({ query, options })),
        );
        for (const { query, options } of cases) {
          const expected = callUnchecked(napi.searchKeys, query, keyTexts, weights, options);
          expect(callUnchecked(wasm.searchKeys, query, keyTexts, weights, options)).toEqual(
            expected,
          );
          expect(callUnchecked(w.search.bind(w), query, options)).toEqual(expected);
        }
        w.free();
      });

      it('declares exactly the accepted modes', () => {
        const declared = /export type KeyScoreMode = (.+);/.exec(readFileSync(DTS_PATH, 'utf8'));
        const literals = [...(declared?.[1] ?? '').matchAll(/"(\w+)"/g)].map((m) => m[1]);
        expect(literals).toEqual([...modes]);
      });
    });

    describe('matchMode', () => {
      const keyTexts = [items, items.map((s) => `${s.toLowerCase()} lang`), items.map(() => 'x')];
      const weights = [2, 1, 0.5];
      const matchModes = ['perKey', 'crossKey'] as const;
      const scoreModes = ['weighted', 'matched', 'max'] as const;

      it('gives the same results in every mode', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        const queries = ['type lang', 'script !java', 'rust x', 'py lang', 'lang !type', 'zzz'];
        const cases = [...matchModes, undefined].flatMap((matchMode) =>
          scoreModes.flatMap((scoreMode) =>
            [undefined, 0.5, 0.9].flatMap((minScore) =>
              queries.map((query) => ({ query, minScore, scoreMode, matchMode })),
            ),
          ),
        );
        let crossKeyMatches = 0;
        for (const { query, minScore, scoreMode, matchMode } of cases) {
          const closestOptions: WasmBindgen.KeyClosestOptions = { scoreMode, matchMode, minScore };
          const options: WasmBindgen.KeySearchOptions = { ...closestOptions, maxResults: 5 };
          const expected = napi.searchKeys(query, keyTexts, weights, options);
          if (matchMode === 'crossKey') crossKeyMatches += expected.length;
          expect(wasm.searchKeys(query, keyTexts, weights, options)).toEqual(expected);
          expect(w.search(query, options)).toEqual(expected);
          expect(n.search(query, options)).toEqual(expected);
          expect(w.closest(query, closestOptions)).toBe(n.closest(query, closestOptions));
        }
        expect(crossKeyMatches).toBeGreaterThan(20);
        // No key of Rust contains both terms of 'rust x'; two of them do.
        expect(napi.searchKeys('rust x', keyTexts, weights)).toEqual([]);
        const [rust] = wasm.searchKeys('rust x', keyTexts, weights, { matchMode: 'crossKey' });
        expect(rust?.index).toBe(items.indexOf('Rust'));
        w.free();
      });

      it('rejects unknown modes with a TypeError carrying the Node.js message', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        for (const bad of BAD_MODES) {
          expectModeRejected({
            field: 'matchMode',
            bad,
            nodeMessage: /^matchMode must be "perKey" or "crossKey", got /,
            keyTexts,
            weights,
            closest: (options) => w.closest('type', options),
            search: (options) => w.search('type', options),
          });
        }
        w.free();
      });

      it('declares exactly the accepted modes', () => {
        const declared = /export type KeyMatchMode = (.+);/.exec(readFileSync(DTS_PATH, 'utf8'));
        const literals = [...(declared?.[1] ?? '').matchAll(/"(\w+)"/g)].map((m) => m[1]);
        expect(literals).toEqual([...matchModes]);
      });
    });

    describe('KeySearchOptions objects', () => {
      const keyTexts = [items, items.map((s) => `${s.toLowerCase()} lang`), items.map(() => 'x')];
      const weights = [2, 1, 0.5];

      const queries = ['type', 'type lang', 'rust x', 'zzz', ''];

      /** `searchKeys` and `KeyedFuzzyIndex.search` of both builds on the same options. */
      function expectSameResults(options: unknown): void {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        for (const query of queries) {
          const expected = callUnchecked(napi.searchKeys, query, keyTexts, weights, options);
          expect(callUnchecked(n.search.bind(n), query, options)).toEqual(expected);
          expect(callUnchecked(wasm.searchKeys, query, keyTexts, weights, options)).toEqual(
            expected,
          );
          expect(callUnchecked(w.search.bind(w), query, options)).toEqual(expected);
        }
        w.free();
      }

      it('reads every field by name, like the Node.js binding', () => {
        class Getters {
          get maxResults(): number {
            return 1;
          }
          get scoreMode(): string {
            return 'max';
          }
          get matchMode(): string {
            return 'crossKey';
          }
        }
        const inherited: unknown[] = [
          new Getters(),
          Object.create({ maxResults: 1 }),
          Object.create({ minScore: 0.9 }),
          Object.create({ scoreMode: 'max', matchMode: 'crossKey' }),
          Object.create({ isCaseSensitive: true, returnAllOnEmpty: true }),
        ];
        for (const options of inherited) {
          // Fields read from a getter or the prototype chain take effect.
          const changed = queries.filter(
            (query) =>
              JSON.stringify(callUnchecked(napi.searchKeys, query, keyTexts, weights, options)) !==
              JSON.stringify(napi.searchKeys(query, keyTexts, weights)),
          );
          expect(changed).not.toEqual([]);
          expectSameResults(options);
        }
        // Entries of a Map are not properties.
        expectSameResults(new Map([['maxResults', 1]]));
      });

      it('rejects BigInt numbers like the Node.js binding', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        for (const options of [{ maxResults: 2n }, { minScore: 2n }, { minScore: 0n }]) {
          expect(() =>
            callUnchecked(napi.searchKeys, 'type', keyTexts, weights, options),
          ).toThrow();
          expect(
            thrown(() => callUnchecked(wasm.searchKeys, 'type', keyTexts, weights, options)),
          ).toBeInstanceOf(TypeError);
          expect(thrown(() => callUnchecked(w.search.bind(w), 'type', options))).toBeInstanceOf(
            TypeError,
          );
        }
        w.free();
      });
    });

    describe('KeyedFuzzyIndex.closest() options', () => {
      const keyTexts = [items, items.map((s) => `${s.toLowerCase()} lang`), items.map(() => 'x')];
      const weights = [2, 1, 0.5];
      const queries = ['type', 'type lang', 'rust x', 'script !java', 'T', 'zzz', ''];
      const scoreModes = ['weighted', 'matched', 'max', undefined] as const;
      const matchModes = ['perKey', 'crossKey', undefined] as const;

      /** Every combination of the modes and some minScore values. */
      function allOptions(): WasmBindgen.KeyClosestOptions[] {
        return scoreModes.flatMap((scoreMode) =>
          matchModes.flatMap((matchMode) =>
            [undefined, 0, 0.5, 0.9, 1].map((minScore) => ({ scoreMode, matchMode, minScore })),
          ),
        );
      }

      /** The smallest double greater than the non-negative number `x`. */
      function nextUp(x: number): number {
        const bits = new BigInt64Array(new Float64Array([x]).buffer);
        bits[0] = (bits[0] ?? 0n) + 1n;
        return new Float64Array(bits.buffer)[0] ?? Number.NaN;
      }

      /** `closest()` of both bindings on `options`, as given (unchecked). */
      function bothClosest(
        w: WasmBindgen.KeyedFuzzyIndex,
        n: napi.KeyedFuzzyIndex,
        query: string,
        options: unknown,
      ): { wasm: unknown; node: unknown } {
        return {
          wasm: callUnchecked(w.closest.bind(w), query, options),
          node: callUnchecked(n.closest.bind(n), query, options),
        };
      }

      it('returns the first result of search() for every combination of options', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        const cases = allOptions().flatMap((options) =>
          queries.map((query) => ({ options, query })),
        );
        let found = 0;
        for (const { options, query } of cases) {
          const [best] = w.search(query, { ...options, maxResults: 1 });
          const expected = best === undefined ? null : best.index;
          found += Number(expected !== null);
          expect(w.closest(query, options)).toBe(expected);
          expect(n.closest(query, options)).toBe(expected);
        }
        expect(found).toBeGreaterThan(100);
        w.free();
      });

      it('takes a number as a shorthand for minScore', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        for (const minScore of [0, 0.5, 0.9, 1, 2, -1, Number.POSITIVE_INFINITY]) {
          for (const query of queries) {
            const expected = n.closest(query, { minScore });
            expect(n.closest(query, minScore)).toBe(expected);
            expect(w.closest(query, minScore)).toBe(expected);
            expect(w.closest(query, { minScore })).toBe(expected);
          }
        }
        w.free();
      });

      it('uses the defaults for undefined, null, {} and fields set to undefined', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        const unset = { minScore: undefined, scoreMode: undefined, matchMode: undefined };
        for (const query of queries) {
          const expected = n.closest(query);
          expect(w.closest(query)).toBe(expected);
          for (const options of [undefined, null, {}, unset]) {
            expect(bothClosest(w, n, query, options)).toEqual({ wasm: expected, node: expected });
          }
        }
        w.free();
      });

      it('keeps the best match when it scores exactly minScore', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        const cases = scoreModes
          .flatMap((scoreMode) => matchModes.map((matchMode) => ({ scoreMode, matchMode })))
          .flatMap((modes) =>
            queries.flatMap((query) => {
              const [best] = n.search(query, { ...modes, maxResults: 1 });
              return best === undefined ? [] : [{ modes, query, best }];
            }),
          );
        expect(cases.length).toBeGreaterThan(20);
        for (const { modes, query, best } of cases) {
          // Kept at its own score; just above it, nothing qualifies.
          const boundaries = [
            [best.score, best.index],
            [nextUp(best.score), null],
          ] as const;
          for (const [minScore, expected] of boundaries) {
            const options = { ...modes, minScore };
            expect(w.closest(query, options)).toBe(expected);
            expect(n.closest(query, options)).toBe(expected);
          }
        }
        // The number shorthand with the default modes.
        const defaults = cases.filter(
          ({ modes }) => modes.scoreMode === undefined && modes.matchMode === undefined,
        );
        expect(defaults.length).toBeGreaterThan(0);
        for (const { query, best } of defaults) {
          expect(w.closest(query, best.score)).toBe(best.index);
          expect(n.closest(query, best.score)).toBe(best.index);
          expect(w.closest(query, nextUp(best.score))).toBeNull();
          expect(n.closest(query, nextUp(best.score))).toBeNull();
        }
        w.free();
      });

      it('rejects what the Node.js binding rejects, with a TypeError', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        const invalid: unknown[] = [
          '0.5',
          true,
          2n,
          Symbol('x'),
          () => 0.5,
          { minScore: '0.5' },
          { minScore: true },
          { minScore: 2n },
          { minScore: {} },
          { scoreMode: 'avg' },
          { scoreMode: null },
          { matchMode: 'cross' },
          { matchMode: null },
          { minScore: 0.5, scoreMode: 'MAX' },
        ];
        for (const options of invalid) {
          expect(() => callUnchecked(n.closest.bind(n), 'type', options)).toThrow();
          expect(thrown(() => callUnchecked(w.closest.bind(w), 'type', options))).toBeInstanceOf(
            TypeError,
          );
        }
        w.free();
      });

      it('reads minScore like KeySearchOptions.minScore in each binding', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        // null is outside the declared types: the Node.js binding rejects it
        // in both options objects and the WebAssembly build leaves it unset
        // in both.
        const options = { minScore: null, scoreMode: 'max' };
        for (const query of queries) {
          expect(() => callUnchecked(n.search.bind(n), query, options)).toThrow(/minScore/);
          expect(() => callUnchecked(n.closest.bind(n), query, options)).toThrow(/minScore/);
          const [best] = w.search(query, { maxResults: 1, scoreMode: 'max' });
          expect(callUnchecked(w.search.bind(w), query, { ...options, maxResults: 1 })).toEqual(
            best === undefined ? [] : [best],
          );
          expect(callUnchecked(w.closest.bind(w), query, options)).toBe(best?.index ?? null);
        }
        w.free();
      });

      it('ignores the other KeySearchOptions fields like the Node.js binding', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        const ignored = [
          { maxResults: 0 },
          { maxResults: -1 },
          { maxResults: 'x' },
          { includePositions: 'yes' },
          { isCaseSensitive: true },
          { returnAllOnEmpty: true },
          { unknown: 1 },
        ];
        for (const extra of ignored) {
          for (const query of queries) {
            const expected = n.closest(query, { scoreMode: 'max', matchMode: 'crossKey' });
            const options = { ...extra, scoreMode: 'max', matchMode: 'crossKey' };
            expect(bothClosest(w, n, query, options)).toEqual({ wasm: expected, node: expected });
          }
        }
        w.free();
      });

      it('reads every field by name, like the Node.js binding', () => {
        const w = new wasm.KeyedFuzzyIndex(keyTexts, weights);
        const n = new napi.KeyedFuzzyIndex(keyTexts, weights);
        class Getters {
          get minScore(): number {
            return 0.5;
          }
          get scoreMode(): string {
            return 'max';
          }
          get matchMode(): string {
            return 'crossKey';
          }
        }
        const inherited: unknown[] = [
          new Getters(),
          Object.create({ minScore: 2 }),
          Object.create({ scoreMode: 'max', matchMode: 'crossKey' }),
          new Map([['minScore', 2]]),
        ];
        let changed = 0;
        for (const options of inherited) {
          for (const query of queries) {
            const result = bothClosest(w, n, query, options);
            expect(result.wasm).toBe(result.node);
            if (result.node !== n.closest(query)) changed++;
          }
        }
        expect(changed).toBeGreaterThan(0);
        w.free();
      });
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

  // A string[] argument holding something other than a string used to fail
  // inside wasm-bindgen's Vec<String> conversion, which throws from inside
  // the wasm call: the index stayed borrowed for good ("recursive use of an
  // object"), and the converted strings and the call's shadow-stack space
  // leaked, until about 16,000 rejected calls broke the whole module.
  describe('string[] arguments are validated before any work', () => {
    it('leaves an index usable after a rejected addMany()', () => {
      const index = new wasm.FuzzyIndex(['apple', 'banana']);
      for (const bad of [
        ['cherry', 42],
        ['cherry', null],
        ['cherry', undefined],
      ]) {
        const err = thrown(() => callUnchecked(index.addMany.bind(index), bad));
        expect(err).toBeInstanceOf(TypeError);
        expect(err).toHaveProperty('message', 'Expected a string at index 1');
      }
      expect(index.size).toBe(2);
      expect(index.search('app').map((r) => r.item)).toEqual(['apple']);
      index.addMany(['cherry']);
      index.add('date');
      expect(index.size).toBe(4);
      index.free();
    });

    it('rejects anything but an array of strings with a TypeError', () => {
      const calls: Array<[string, (arg: unknown) => unknown]> = [
        ['new FuzzyIndex', (arg) => Reflect.construct(wasm.FuzzyIndex, [arg])],
        ['search', (arg) => callUnchecked(wasm.search, 'a', arg)],
        ['closest', (arg) => callUnchecked(wasm.closest, 'a', arg)],
        ['levenshteinMany', (arg) => callUnchecked(wasm.levenshteinMany, 'a', arg)],
        ['jaroWinklerMany', (arg) => callUnchecked(wasm.jaroWinklerMany, 'a', arg)],
        ['hammingMany', (arg) => callUnchecked(wasm.hammingMany, 'a', arg)],
        ['weightedRatioMany', (arg) => callUnchecked(wasm.weightedRatioMany, 'a', arg)],
      ];
      for (const [name, call] of calls) {
        // A string or a number used to be read as an array (of characters,
        // or empty).
        for (const bad of ['abc', 5, null, {}, ['a', 1]]) {
          expect(
            thrown(() => call(bad)),
            `${name}(${String(bad)})`,
          ).toBeInstanceOf(TypeError);
        }
      }
    });

    it('does not leak wasm memory when an array is rejected', () => {
      const items: unknown[] = Array.from({ length: 2000 }, (_, i) => `item number ${i} padding`);
      items.push(42);
      const calls: Array<() => unknown> = [
        () => callUnchecked(wasm.search, 'item', items),
        () => callUnchecked(wasm.closest, 'item', items),
        () => callUnchecked(wasm.levenshteinMany, 'item', items),
        () => Reflect.construct(wasm.FuzzyIndex, [items]),
      ];
      for (const call of calls) {
        const run = (n: number): void => {
          for (let i = 0; i < n; i++) thrown(call);
        };
        run(20);
        const before = memory.buffer.byteLength;
        run(300);
        // Before the fix every rejected call leaked the ~2,000 converted strings.
        expect(memory.buffer.byteLength - before).toBeLessThan(1024 * 1024);
      }
    });

    it('keeps the module working after many rejected calls (no shadow-stack leak)', () => {
      for (let i = 0; i < 20_000; i++) {
        thrown(() => callUnchecked(wasm.levenshteinMany, 'a', ['a', 1]));
      }
      expect(wasm.levenshtein('kitten', 'sitting')).toBe(3);
      expect(wasm.search('app', ['apple'])).toHaveLength(1);
    });
  });

  // A string parameter given anything else used to make the glue allocate
  // `undefined` bytes and trap with "RuntimeError: memory access out of
  // bounds" (null: "Cannot read properties of null").
  describe('string arguments', () => {
    it('throw a TypeError, not a WebAssembly trap, for a value that is not a string', () => {
      const index = new wasm.FuzzyIndex(['a']);
      const keyed = new wasm.KeyedFuzzyIndex([['a']], [1]);
      const calls: Array<[string, (arg: unknown) => unknown]> = [
        ['levenshtein', (arg) => callUnchecked(wasm.levenshtein, arg, 'a')],
        ['jaroWinkler (second argument)', (arg) => callUnchecked(wasm.jaroWinkler, 'a', arg)],
        ['search', (arg) => callUnchecked(wasm.search, arg, ['a'])],
        ['levenshteinMany (reference)', (arg) => callUnchecked(wasm.levenshteinMany, arg, ['a'])],
        ['FuzzyIndex#add', (arg) => callUnchecked(index.add.bind(index), arg)],
        ['FuzzyIndex#search', (arg) => callUnchecked(index.search.bind(index), arg)],
        ['FuzzyIndex#closest', (arg) => callUnchecked(index.closest.bind(index), arg)],
        ['KeyedFuzzyIndex#search', (arg) => callUnchecked(keyed.search.bind(keyed), arg)],
      ];
      const bad: Array<[unknown, string]> = [
        [1, 'number'],
        [true, 'boolean'],
        [{}, 'object'],
        [null, 'null'],
        [undefined, 'undefined'],
      ];
      for (const [name, call] of calls) {
        for (const [value, type] of bad) {
          const err = thrown(() => call(value));
          expect(err, `${name}(${String(value)})`).toBeInstanceOf(TypeError);
          expect(err).toHaveProperty('message', `Expected a string, got ${type}`);
        }
      }
      // Nothing was added, and the module keeps working.
      expect(index.size).toBe(1);
      expect(wasm.levenshtein('kitten', 'sitting')).toBe(3);
      // The Node.js binding rejects the same arguments.
      expect(() => callUnchecked(napi.levenshtein, 1, 'a')).toThrow();
      index.free();
      keyed.free();
    });
  });

  // A NaN minScore used to filter out every match silently.
  describe('minScore validation (like the Node.js binding)', () => {
    it('throws a TypeError for NaN in every API and accepts other numbers', () => {
      const items = ['a', 'ab', 'abc'];
      const index = new wasm.FuzzyIndex(items);
      const keyed = new wasm.KeyedFuzzyIndex([items], [1]);
      const calls = (minScore: number): Array<[string, () => unknown]> => [
        ['search', () => wasm.search('a', items, { minScore })],
        ['closest', () => wasm.closest('a', items, minScore)],
        ['FuzzyIndex.search', () => index.search('a', { minScore })],
        ['FuzzyIndex.searchIndices', () => index.searchIndices('a', { minScore })],
        ['FuzzyIndex.closest', () => index.closest('a', minScore)],
        ['searchKeys', () => wasm.searchKeys('a', [items], [1], { minScore })],
        ['KeyedFuzzyIndex.search', () => keyed.search('a', { minScore })],
        ['KeyedFuzzyIndex.closest(number)', () => keyed.closest('a', minScore)],
        ['KeyedFuzzyIndex.closest(options)', () => keyed.closest('a', { minScore })],
      ];
      for (const [name, call] of calls(Number.NaN)) {
        const err = thrown(call);
        expect(err, name).toBeInstanceOf(TypeError);
        expect((err as Error).message, name).toContain('minScore must be a number, got NaN');
      }
      for (const minScore of [Number.NEGATIVE_INFINITY, 0, 2, Number.POSITIVE_INFINITY]) {
        for (const [name, call] of calls(minScore)) {
          expect(call, `${name} minScore=${minScore}`).not.toThrow();
        }
      }
      index.free();
      keyed.free();
    });
  });

  // maxDistance used to be a u32 parameter, wrapping it modulo 2^32: Infinity,
  // NaN and 2 ** 32 became 0 and -1 disabled the limit.
  describe('maxDistance validation (like the Node.js binding)', () => {
    const cands = ['kitten', 'sitting', 'kitchen', 'a much longer candidate'];
    const fns = () =>
      [
        ['levenshteinMany', wasm.levenshteinMany, napi.levenshteinMany],
        ['damerauLevenshteinMany', wasm.damerauLevenshteinMany, napi.damerauLevenshteinMany],
        ['indelMany', wasm.indelMany, napi.indelMany],
        ['hammingMany', wasm.hammingMany, napi.hammingMany],
      ] as const;

    it('treats Infinity and values >= 2^32 as no limit', () => {
      for (const [name, fn, native] of fns()) {
        const plain = Array.from(fn('kitten', cands));
        expect(plain, name).toEqual(Array.from(native('kitten', cands)));
        for (const maxDistance of [Number.POSITIVE_INFINITY, 2 ** 32, 2 ** 32 + 2]) {
          expect(Array.from(fn('kitten', cands, maxDistance)), name).toEqual(plain);
        }
      }
    });

    it('throws an Error with the Node.js message for NaN, negative and fractional values', () => {
      for (const [name, fn, native] of fns()) {
        for (const maxDistance of [Number.NaN, -1, 2.9, Number.NEGATIVE_INFINITY, 'x', {}]) {
          const err = thrown(() => callUnchecked(fn, 'kitten', cands, maxDistance));
          expect(err, `${name}(${String(maxDistance)})`).toBeInstanceOf(Error);
          if (typeof maxDistance === 'number') {
            const nodeErr = thrown(() => native('kitten', cands, maxDistance));
            expect(err).toHaveProperty('message', (nodeErr as Error).message);
          }
        }
      }
    });
  });

  // The index used to be a u32 parameter, which wraps numbers modulo 2^32 and
  // converts anything else to a number: remove(NaN), remove(2 ** 32) and
  // remove(undefined) (a missed Map lookup) removed item 0 and returned true.
  describe('remove() argument validation (like FuzzyObjectIndex.remove)', () => {
    interface Removable {
      remove(index: number): boolean;
      readonly size: number;
      free(): void;
    }
    const indexes = (): Removable[] => [
      new wasm.FuzzyIndex(['a', 'b', 'c']),
      new wasm.KeyedFuzzyIndex([['a', 'b', 'c']], [1]),
    ];

    it.each([
      [Number.NaN, 'NaN'],
      [1.5, '1.5'],
      [-0.5, '-0.5'],
      [Number.POSITIVE_INFINITY, 'Infinity'],
      [Number.NEGATIVE_INFINITY, '-Infinity'],
    ])('throws a RangeError for %s and removes nothing', (value, shown) => {
      for (const index of indexes()) {
        const err = thrown(() => index.remove(value));
        expect(err).toBeInstanceOf(RangeError);
        expect(err).toHaveProperty('message', `index must be an integer, got ${shown}`);
        expect(index.size).toBe(3);
        index.free();
      }
    });

    it.each([
      ['1', 'string'],
      [null, 'object'],
      [undefined, 'undefined'],
      [true, 'boolean'],
      [{}, 'object'],
      [1n, 'bigint'],
    ])('throws a TypeError for %s and removes nothing', (value, type) => {
      for (const index of indexes()) {
        const err = thrown(() => callUnchecked(index.remove.bind(index), value));
        expect(err).toBeInstanceOf(TypeError);
        expect(err).toHaveProperty('message', `index must be a number, got ${type}`);
        expect(index.size).toBe(3);
        index.free();
      }
    });

    it('returns false for out-of-range integers, including those beyond 2^32', () => {
      for (const index of indexes()) {
        for (const value of [-1, 3, 2 ** 32, 2 ** 32 + 1, -(2 ** 32) + 1]) {
          expect(index.remove(value)).toBe(false);
        }
        expect(index.size).toBe(3);
        expect(index.remove(-0)).toBe(true);
        expect(index.size).toBe(2);
        index.free();
      }
    });

    it('throws the same errors as the Node.js binding', () => {
      const native = new napi.FuzzyIndex(['a', 'b', 'c']);
      for (const value of [Number.NaN, 1.5, undefined, '1']) {
        const wasmIndex = new wasm.FuzzyIndex(['a', 'b', 'c']);
        const wasmErr = thrown(() => callUnchecked(wasmIndex.remove.bind(wasmIndex), value));
        const nodeErr = thrown(() => callUnchecked(native.remove.bind(native), value));
        expect((wasmErr as Error).constructor).toBe((nodeErr as Error).constructor);
        expect(wasmErr).toHaveProperty('message', (nodeErr as Error).message);
        wasmIndex.free();
      }
    });
  });
});
