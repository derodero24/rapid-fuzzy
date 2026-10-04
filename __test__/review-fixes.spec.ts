// Regression tests for the review of the 2.2 release candidate: Node.js and
// WebAssembly builds must accept the same arguments and agree on the results.
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import * as napi from '../index.js';
import { FuzzyObjectIndex as NodeFuzzyObjectIndex, searchObjects } from '../objects.js';

const ROOT = join(__dirname, '..');
const wasmAvailable = existsSync(join(ROOT, 'rapid-fuzzy-wasm-bindgen_bg.wasm'));
if (!wasmAvailable && process.env.RAPID_FUZZY_REQUIRE_WASM_BINDGEN) {
  throw new Error('wasm-bindgen binary is required but missing');
}

type BrowserModule = typeof import('../browser.mjs', { with: { 'resolution-mode': 'import' }});
const loadBrowser = async (): Promise<BrowserModule> =>
  (await import('../browser.mjs')) as BrowserModule;

/** Call a function with arguments its TypeScript signature rejects. */
function callUnchecked(fn: unknown, thisArg: unknown, ...args: unknown[]): unknown {
  return (fn as (...a: unknown[]) => unknown).apply(thisArg, args);
}

const ITEMS = ['apple', 'apricot', 'grape', 'pineapple', 'banana'];
const USERS = [
  { name: 'apple pie', tag: 'dessert' },
  { name: 'apricot jam', tag: 'spread' },
  { name: 'grape juice', tag: 'drink' },
  { name: 'pineapple', tag: 'fruit' },
];
const KEY_TEXTS = [USERS.map((u) => u.name), USERS.map((u) => u.tag)];

describe.skipIf(!wasmAvailable)('maxResults in the WebAssembly build', () => {
  it('accepts Infinity as "no limit" everywhere, like the Node.js build', async () => {
    const b = await loadBrowser();
    const all = { maxResults: Number.POSITIVE_INFINITY };
    expect(b.search('ap', ITEMS, all)).toEqual(napi.search('ap', ITEMS, all));
    expect(b.search('ap', ITEMS, Number.POSITIVE_INFINITY)).toEqual(
      napi.search('ap', ITEMS, Number.POSITIVE_INFINITY),
    );
    expect(b.search('', ITEMS, { ...all, returnAllOnEmpty: true })).toHaveLength(ITEMS.length);

    const index = new b.FuzzyIndex(ITEMS);
    const nIndex = new napi.FuzzyIndex(ITEMS);
    expect(index.search('ap', all)).toEqual(nIndex.search('ap', all));
    expect(index.search('ap', Number.POSITIVE_INFINITY)).toEqual(
      nIndex.search('ap', Number.POSITIVE_INFINITY),
    );
    expect(index.searchIndices('ap', all)).toEqual(nIndex.searchIndices('ap', all));

    const keyed = new b.KeyedFuzzyIndex(KEY_TEXTS, [1, 1]);
    const nKeyed = new napi.KeyedFuzzyIndex(KEY_TEXTS, [1, 1]);
    expect(keyed.search('ap', all)).toEqual(nKeyed.search('ap', all));
    expect(b.searchKeys('ap', KEY_TEXTS, [1, 1], all)).toEqual(
      napi.searchKeys('ap', KEY_TEXTS, [1, 1], all),
    );
    expect(b.searchKeys('ap', KEY_TEXTS, [1, 1], Number.POSITIVE_INFINITY)).toHaveLength(
      napi.searchKeys('ap', KEY_TEXTS, [1, 1]).length,
    );

    const keys = ['name', 'tag'];
    expect(b.searchObjects('ap', USERS, { keys, ...all })).toEqual(
      searchObjects('ap', USERS, { keys, ...all }),
    );
    expect(new b.FuzzyObjectIndex(USERS, { keys }).search('ap', all)).toEqual(
      new NodeFuzzyObjectIndex(USERS, { keys }).search('ap', all),
    );
  });

  it('caps values beyond u32 and accepts -0, like the Node.js build', async () => {
    const b = await loadBrowser();
    for (const maxResults of [-0, 0, 2, 2 ** 32, 1e300]) {
      expect(b.search('ap', ITEMS, { maxResults })).toEqual(
        napi.search('ap', ITEMS, { maxResults }),
      );
    }
  });

  it('rejects NaN, negative and fractional values with a TypeError', async () => {
    const b = await loadBrowser();
    const index = new b.FuzzyIndex(ITEMS);
    const keyed = new b.KeyedFuzzyIndex(KEY_TEXTS, [1, 1]);
    for (const bad of [Number.NaN, -1, 1.5, Number.NEGATIVE_INFINITY]) {
      expect(() => napi.search('ap', ITEMS, { maxResults: bad })).toThrow(/maxResults/);
      expect(() => b.search('ap', ITEMS, { maxResults: bad })).toThrow(TypeError);
      expect(() => b.search('ap', ITEMS, { maxResults: bad })).toThrow(
        /maxResults must be a non-negative integer or Infinity/,
      );
      expect(() => b.search('ap', ITEMS, bad)).toThrow(TypeError);
      expect(() => index.search('ap', bad)).toThrow(TypeError);
      expect(() => keyed.search('ap', bad)).toThrow(TypeError);
      expect(() => keyed.search('ap', { maxResults: bad })).toThrow(TypeError);
      expect(() => b.searchKeys('ap', KEY_TEXTS, [1, 1], { maxResults: bad })).toThrow(TypeError);
    }
  });

  it('accepts the numeric shorthand in KeyedFuzzyIndex and FuzzyObjectIndex', async () => {
    const b = await loadBrowser();
    const keyed = new b.KeyedFuzzyIndex(KEY_TEXTS, [1, 1]);
    const nKeyed = new napi.KeyedFuzzyIndex(KEY_TEXTS, [1, 1]);
    expect(keyed.search('ap', 2)).toEqual(nKeyed.search('ap', 2));
    expect(keyed.search('ap', 2)).toHaveLength(2);
    expect(keyed.search('ap', null)).toEqual(nKeyed.search('ap', null));

    const keys = ['name', 'tag'];
    const objects = new b.FuzzyObjectIndex(USERS, { keys });
    const nObjects = new NodeFuzzyObjectIndex(USERS, { keys });
    expect(objects.search('ap', 2)).toEqual(nObjects.search('ap', 2));
    expect(objects.search('ap', 2)).toHaveLength(2);
  });

  it('rejects a non-object, non-number options argument with a TypeError', async () => {
    const b = await loadBrowser();
    const keyed = new b.KeyedFuzzyIndex(KEY_TEXTS, [1, 1]);
    expect(() => callUnchecked(keyed.search, keyed, 'ap', 'many')).toThrow(TypeError);
  });
});

describe.skipIf(!wasmAvailable)('FuzzyIndex.fromAsync in the WebAssembly build', () => {
  it('returns a rejected Promise (a TypeError) for invalid items, like the Node.js build', async () => {
    const b = await loadBrowser();
    const bad: unknown = ['a', 5];
    const nodePromise = callUnchecked(napi.FuzzyIndex.fromAsync, napi.FuzzyIndex, bad);
    expect(nodePromise).toBeInstanceOf(Promise);
    await expect(nodePromise).rejects.toThrow();

    let wasmPromise: unknown;
    expect(() => {
      wasmPromise = callUnchecked(b.FuzzyIndex.fromAsync, b.FuzzyIndex, bad);
    }).not.toThrow();
    expect(wasmPromise).toBeInstanceOf(Promise);
    await expect(wasmPromise).rejects.toThrow(TypeError);
    await expect(callUnchecked(b.FuzzyIndex.fromAsync, b.FuzzyIndex, 'abc')).rejects.toThrow(
      TypeError,
    );
  });

  it('still resolves to a working index', async () => {
    const b = await loadBrowser();
    const index = await b.FuzzyIndex.fromAsync(ITEMS);
    expect(index.search('ap')).toEqual(new napi.FuzzyIndex(ITEMS).search('ap'));
  });
});
