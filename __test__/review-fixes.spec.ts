// Regression tests for the review of the 2.2 release candidate: Node.js and
// WebAssembly builds must accept the same arguments and agree on the results.
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { highlight, highlightRanges } from '../highlight.js';
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

describe('FuzzyObjectIndex.deserialize() checks the keys against the native index', () => {
  const items = [
    { name: 'apple', color: 'red' },
    { name: 'banana', color: 'yellow' },
  ];

  /** Split serialized data into its metadata and its native index payload. */
  function split(bytes: Buffer): {
    meta: { keys: Array<{ name: string; weight: number }> };
    native: Buffer;
  } {
    const metaLen = bytes.readUInt32LE(0);
    return {
      meta: JSON.parse(bytes.subarray(4, 4 + metaLen).toString('utf8')),
      native: bytes.subarray(4 + metaLen),
    };
  }

  function join(meta: unknown, native: Buffer): Buffer {
    const json = Buffer.from(JSON.stringify(meta), 'utf8');
    const header = Buffer.alloc(4);
    header.writeUInt32LE(json.length, 0);
    return Buffer.concat([header, json, native]);
  }

  it('rejects metadata with more or fewer keys than the native index', () => {
    const { meta, native } = split(new NodeFuzzyObjectIndex(items, { keys: ['name'] }).serialize());
    const more = { ...meta, keys: [...meta.keys, { name: 'color', weight: 1 }] };
    expect(() => NodeFuzzyObjectIndex.deserialize(join(more, native))).toThrow(
      /^Invalid FuzzyObjectIndex data: key count mismatch \(2 keys, 1 indexed\)/,
    );
    const two = split(new NodeFuzzyObjectIndex(items, { keys: ['name', 'color'] }).serialize());
    const fewer = { ...two.meta, keys: two.meta.keys.slice(0, 1) };
    expect(() => NodeFuzzyObjectIndex.deserialize(join(fewer, two.native))).toThrow(
      /key count mismatch \(1 keys, 2 indexed\)/,
    );
  });

  it('rejects metadata whose weights differ from the native index', () => {
    const { meta, native } = split(
      new NodeFuzzyObjectIndex(items, { keys: [{ name: 'name', weight: 2 }, 'color'] }).serialize(),
    );
    const reweighted = { ...meta, keys: meta.keys.map((k) => ({ ...k, weight: 1 })) };
    expect(() => NodeFuzzyObjectIndex.deserialize(join(reweighted, native))).toThrow(
      /^Invalid FuzzyObjectIndex data: weight of key 0 \(name\) is 1, the native index has 2/,
    );
  });

  it('still loads consistent data, which then accepts new items', () => {
    const original = new NodeFuzzyObjectIndex(items, {
      keys: [{ name: 'name', weight: 2 }, 'color'],
    });
    const restored = NodeFuzzyObjectIndex.deserialize(original.serialize());
    expect(restored.search('apple')).toEqual(original.search('apple'));
    restored.add({ name: 'cherry', color: 'red' });
    expect(restored.search('cherry')[0]?.item).toEqual({ name: 'cherry', color: 'red' });
  });

  it('turns a payload destroyed by 2.1 (no native keys) into a usable empty index', () => {
    // rapid-fuzzy 2.1 cleared the native key configuration on destroy(), so
    // its serialized payload kept the keys only in the metadata.
    const legacyNative = Buffer.alloc(16);
    legacyNative.write('RFKI', 0, 'latin1');
    legacyNative.writeUInt32LE(1, 4); // version
    const meta = {
      items: [],
      keys: [
        { name: 'name', weight: 2 },
        { name: 'color', weight: 1 },
      ],
    };
    const restored = NodeFuzzyObjectIndex.deserialize(join(meta, legacyNative));
    expect(restored.size).toBe(0);
    restored.add({ name: 'cherry', color: 'red' });
    expect(restored.search('red')[0]?.keyScores).toHaveLength(2);
  });
});

describe('highlight() uses the position unit of search results', () => {
  /** Highlight the best match of `query` in `item`, via search() and FuzzyIndex. */
  function marked(query: string, item: string): string {
    const [hit] = napi.search(query, [item], { includePositions: true });
    const [indexed] = new napi.FuzzyIndex([item]).search(query, { includePositions: true });
    expect(hit?.positions).toEqual(indexed?.positions);
    return highlight(item, hit?.positions ?? [], '[', ']');
  }

  it('skips emoji and other characters outside the BMP before the match', () => {
    expect(marked('apple', '🍎 apple pie')).toBe('🍎 [apple] pie');
    expect(marked('abc', '😀abc')).toBe('😀[abc]');
    expect(marked('pie', '🍎🍐 pie')).toBe('🍎🍐 [pie]');
  });

  it('never splits a surrogate pair or a combining sequence', () => {
    const nfd = 'e\u0301abc'; // é as e + combining acute accent
    expect(marked('abc', nfd)).toBe('e\u0301[abc]');
    expect(marked('eab', nfd)).toBe('[e\u0301ab]c');
    for (const [item, positions] of [
      ['😀abc', [0, 1]],
      ['a👍🏽b', [1]],
      ['🇯🇵x', [0]],
    ] as const) {
      for (const range of highlightRanges(item, positions)) {
        const text = item.slice(range.start, range.end);
        const loneSurrogate =
          /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/;
        expect(loneSurrogate.test(text), `${item} ${JSON.stringify(range)}`).toBe(false);
      }
    }
    expect(highlight('a👍🏽b', [1], '[', ']')).toBe('a[👍🏽]b');
    expect(highlight('🇯🇵x', [0], '[', ']')).toBe('[🇯🇵]x');
  });

  it('returns UTF-16 offsets from highlightRanges()', () => {
    const item = '🍎 apple pie';
    const [hit] = napi.search('apple', [item], { includePositions: true });
    const matched = highlightRanges(item, hit?.positions ?? []).filter((r) => r.matched);
    expect(matched.map((r) => item.slice(r.start, r.end))).toEqual(['apple']);
    expect(highlightRanges('😀ab', [1])).toEqual([
      { start: 0, end: 2, matched: false },
      { start: 2, end: 3, matched: true },
      { start: 3, end: 4, matched: false },
    ]);
  });

  it('counts ASCII items by character, like the search', () => {
    // An ASCII item is matched byte by byte: \r\n counts as two positions.
    expect(marked('b', 'a\r\nb')).toBe('a\r\n[b]');
    expect(highlightRanges('fuzzy', [0, 3, 4])).toEqual([
      { start: 0, end: 1, matched: true },
      { start: 1, end: 3, matched: false },
      { start: 3, end: 5, matched: true },
    ]);
  });

  it.skipIf(!wasmAvailable)('gives the same result with the WebAssembly build', async () => {
    const b = await loadBrowser();
    for (const [query, item] of [
      ['apple', '🍎 apple pie'],
      ['abc', 'e\u0301abc'],
      ['tokyo', '東京 tokyo 🗼'],
    ] as const) {
      const [hit] = b.search(query, [item], { includePositions: true });
      expect(hit?.positions).toEqual(
        napi.search(query, [item], { includePositions: true })[0]?.positions,
      );
      expect(b.highlight(item, hit?.positions ?? [], '[', ']')).toBe(
        highlight(item, hit?.positions ?? [], '[', ']'),
      );
    }
  });
});

describe('THIRD_PARTY_NOTICES', () => {
  const notices = readFileSync(join(ROOT, 'THIRD_PARTY_NOTICES'), 'utf8');

  it('reproduces the notices of the code wasi-libc links into the WASI build', () => {
    for (const notice of [
      'Copyright (c) 2015-2017 Nuxi (https://nuxi.nl/) and contributors.', // cloudlibc, BSD-2-Clause
      'Copyright © 2005-2020 Rich Felker, et al.', // musl, MIT
      'The Regents of the University of California.  All rights reserved.', // musl-fts, BSD-3-Clause
      'Copyright (c) 2010-2014 Emscripten authors, see AUTHORS file.', // emmalloc, MIT
      'Doug Lea and released to the public domain', // dlmalloc, CC0
      'wasi-libc as a whole is multi-licensed',
    ]) {
      expect(notices).toContain(notice);
    }
  });

  it('reproduces the notice of the Rust standard library', () => {
    expect(notices).toContain('It is Copyright (c) The Rust Project Contributors.');
    expect(notices).toContain('UNICODE LICENSE V3');
  });
});
