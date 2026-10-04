import { describe, expect, it } from 'vitest';

import {
  KeyedFuzzyIndex,
  type KeySearchResult,
  type SearchOptions,
  search,
  searchKeys,
} from '../index.js';
import { searchObjects } from '../objects.js';

// new KeyedFuzzyIndex(keyTexts, weights).search(q, opts) must return exactly
// what searchKeys(q, keyTexts, weights, opts) returns: same indices, scores,
// keyScores and order, for any input.

const IDEOGRAPHIC_SPACE = String.fromCodePoint(0x3000);
const NO_BREAK_SPACE = String.fromCodePoint(0xa0);
const KELVIN_SIGN = String.fromCodePoint(0x212a);

type Random = () => number;

/** mulberry32: small deterministic PRNG so failures are reproducible. */
function rng(seed: number): Random {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function below(random: Random, n: number): number {
  return Math.floor(random() * n);
}

function pick<T>(random: Random, values: readonly T[]): T {
  const value = values[below(random, values.length)];
  if (value === undefined) throw new Error('pick() from an empty list');
  return value;
}

const ITEM_PIECES: readonly string[] = [
  ..."abcefors xABFOS_/.-$^!'\\01",
  ...'éÉłŁóÓźŹдДмМſßİıΣσςＡ東京',
  KELVIN_SIGN,
  IDEOGRAPHIC_SPACE,
  NO_BREAK_SPACE,
  '\t',
  '👍🏽',
  'foo',
  'bar',
  'foobar',
  'Łódź',
  'Москва',
];

const QUERY_PIECES: readonly string[] = [
  ' ',
  IDEOGRAPHIC_SPACE,
  NO_BREAK_SPACE,
  '\t',
  '\n',
  '^',
  '$',
  '!',
  "'",
  '\\ ',
];

function randomItem(random: Random): string {
  let out = '';
  const pieces = below(random, 6);
  for (let i = 0; i < pieces; i++) out += pick(random, ITEM_PIECES);
  return out;
}

function randomWeight(random: Random): number {
  return pick(random, [
    0,
    1,
    0.1,
    0.2,
    0.3,
    0.494,
    0.953,
    0.137,
    3,
    1e-9,
    below(random, 1000) / 997,
  ]);
}

function randomQuery(random: Random, texts: readonly string[]): string {
  let query = '';
  if (texts.length > 0 && random() < 0.75) {
    const chars = [...pick(random, texts)];
    const start = below(random, chars.length + 1);
    const end = start + 1 + below(random, chars.length - start + 1);
    query = chars.slice(start, end).join('');
  } else {
    const pieces = below(random, 4);
    for (let i = 0; i < pieces; i++) query += pick(random, ITEM_PIECES);
  }
  if (random() < 0.25) {
    const at = below(random, query.length + 1);
    query = query.slice(0, at) + pick(random, QUERY_PIECES) + query.slice(at);
  }
  return random() < 0.1 ? query.toUpperCase() : query;
}

function randomOptions(random: Random, numItems: number): SearchOptions {
  const options: SearchOptions = {};
  if (random() < 0.4) options.maxResults = below(random, numItems + 2);
  if (random() < 0.3) options.minScore = pick(random, [0, 0.1, 0.25, 0.5, 0.75, 1]);
  if (random() < 0.2) options.isCaseSensitive = true;
  if (random() < 0.2) options.returnAllOnEmpty = true;
  return options;
}

function bothPaths(
  query: string,
  keyTexts: string[][],
  weights: number[],
  options?: SearchOptions,
): { index: KeySearchResult[]; keys: KeySearchResult[] } {
  return {
    index: new KeyedFuzzyIndex(keyTexts, weights).search(query, options),
    keys: searchKeys(query, keyTexts, weights, options),
  };
}

function indices(results: readonly KeySearchResult[]): number[] {
  return results.map((r) => r.index);
}

/** The error thrown by the KeyedFuzzyIndex constructor for this input. */
function constructorError(keyTexts: string[][], weights: number[]): Error {
  try {
    new KeyedFuzzyIndex(keyTexts, weights);
  } catch (error) {
    if (error instanceof Error) return error;
    throw new Error(`constructor threw a non-Error: ${String(error)}`);
  }
  throw new Error(`constructor accepted ${JSON.stringify({ keyTexts, weights })}`);
}

describe('KeyedFuzzyIndex and searchKeys parity', () => {
  it('return identical results for random input', () => {
    let nonEmpty = 0;
    for (let seed = 0; seed < 600; seed++) {
      const random = rng(seed);
      const numKeys = 1 + below(random, 4);
      const numItems = below(random, 12);
      const keyTexts = Array.from({ length: numKeys }, () =>
        Array.from({ length: numItems }, () => randomItem(random)),
      );
      const weights = Array.from({ length: numKeys }, () => randomWeight(random));
      if (weights.reduce((a, b) => a + b, 0) <= 0) continue;
      const index = new KeyedFuzzyIndex(keyTexts, weights);
      const texts = keyTexts.flat();
      for (let q = 0; q < 4; q++) {
        const query = randomQuery(random, texts);
        const options = randomOptions(random, numItems);
        const expected = searchKeys(query, keyTexts, weights, options);
        if (expected.length > 0) nonEmpty++;
        expect(index.search(query, options), JSON.stringify({ seed, query, options })).toEqual(
          expected,
        );
      }
    }
    expect(nonEmpty).toBeGreaterThan(300);
  });

  it('keeps items whose combined score equals minScore exactly', () => {
    const keyTexts = Array.from({ length: 5 }, () => ['foo', 'bar']);
    const weights = [0.494, 0.953, 0.137, 0.447, 0.673];
    const { index, keys } = bothPaths('foo', keyTexts, weights, { minScore: 1 });
    expect(keys).toEqual([{ index: 0, score: 1, keyScores: [1, 1, 1, 1, 1] }]);
    expect(index).toEqual(keys);
  });

  it('scores zero-weight keys without letting them select items', () => {
    const keyTexts = [
      ['apple', 'zzz', 'apple'],
      ['zzz', 'apple', 'apple'],
    ];
    const { index, keys } = bothPaths('apple', keyTexts, [1, 0]);
    expect(indices(keys)).toEqual([0, 2]);
    expect(keys.map((r) => r.keyScores)).toEqual([
      [1, 0],
      [1, 1],
    ]);
    expect(index).toEqual(keys);
  });

  it('matches case-folded characters in both paths', () => {
    const keyTexts = [['Łódź', `${KELVIN_SIGN}elvin`, 'ŠKODA', 'other']];
    for (const query of ['łódź', 'kelvin', 'škoda']) {
      const expected = search(query, keyTexts[0] ?? []).map((r) => r.index);
      expect(expected.length).toBeGreaterThan(0);
      const { index, keys } = bothPaths(query, keyTexts, [1]);
      expect(indices(index)).toEqual(expected);
      expect(indices(keys)).toEqual(expected);
    }
  });

  it('breaks score ties by the shorter best-matching key, like search()', () => {
    const items = ['foobar', 'foo', 'foo bar baz', 'xfoo'];
    const expected = search('foo', items).map((r) => ({ index: r.index, score: r.score }));
    expect(expected[0]?.index).toBe(1);
    const { index, keys } = bothPaths('foo', [items], [1]);
    expect(keys.map((r) => ({ index: r.index, score: r.score }))).toEqual(expected);
    expect(index).toEqual(keys);
  });

  it('treats Unicode whitespace in the query as a term separator', () => {
    const keyTexts = [['foo bar', 'foo', 'bar']];
    const plain = searchKeys('foo bar', keyTexts, [1]);
    expect(indices(plain)).toEqual([0]);
    for (const separator of [IDEOGRAPHIC_SPACE, NO_BREAK_SPACE, '\t', '\n']) {
      const { index, keys } = bothPaths(`foo${separator}bar`, keyTexts, [1]);
      expect(keys).toEqual(plain);
      expect(index).toEqual(plain);
    }
  });

  it('treats syntax-only queries as empty', () => {
    const keyTexts = [['a^b', 'c']];
    for (const query of ['^', '!', '$', "'", '^$']) {
      const { index, keys } = bothPaths(query, keyTexts, [1]);
      expect(keys).toEqual([]);
      expect(index).toEqual([]);
      const all = bothPaths(query, keyTexts, [1], { returnAllOnEmpty: true });
      expect(indices(all.keys)).toEqual([0, 1]);
      expect(all.index).toEqual(all.keys);
    }
  });

  it('returns nothing for a term too long to score instead of a wrapped score', () => {
    const long = 'a'.repeat(3000);
    const { index, keys } = bothPaths(long, [[long]], [1]);
    expect(keys).toEqual([]);
    expect(index).toEqual([]);
  });
});

describe('searchKeys input validation', () => {
  const invalid: ReadonlyArray<[string, string[][], number[]]> = [
    ['ragged key texts', [['a', 'b'], ['c']], [1, 1]],
    ['too many weights', [['a']], [1, 2]],
    ['too few weights', [['a'], ['b']], [1]],
    ['a negative weight', [['a']], [-1]],
    ['a NaN weight', [['a']], [Number.NaN]],
    ['an infinite weight', [['a']], [Number.POSITIVE_INFINITY]],
    ['weights summing to 0', [['a'], ['b']], [0, 0]],
    ['weights summing to Infinity', [['a'], ['b']], [Number.MAX_VALUE, Number.MAX_VALUE]],
  ];

  for (const [name, keyTexts, weights] of invalid) {
    it(`throws like the KeyedFuzzyIndex constructor for ${name}`, () => {
      const { message } = constructorError(keyTexts, weights);
      const error = expect.objectContaining({ code: 'InvalidArg', message });
      expect(() => searchKeys('a', keyTexts, weights)).toThrow(error);
      expect(() => searchKeys('', keyTexts, weights)).toThrow(error);
    });
  }

  it('rejects invalid weights in searchObjects too', () => {
    expect(() =>
      searchObjects('a', [{ name: 'a' }], { keys: [{ name: 'name', weight: -1 }] }),
    ).toThrow('Weights must be finite non-negative numbers');
  });

  it('still returns nothing for valid input without items', () => {
    expect(searchKeys('a', [[], []], [1, 1])).toEqual([]);
  });
});

describe('searchKeys maxResults shorthand', () => {
  const items = ['a', 'ab', 'abc', 'abcd'];

  it('accepts a number like search() does', () => {
    expect(searchKeys('a', [items], [1], 2)).toEqual(
      searchKeys('a', [items], [1], { maxResults: 2 }),
    );
    expect(searchKeys('a', [items], [1], 0)).toEqual([]);
    expect(searchKeys('a', [items], [1], Number.POSITIVE_INFINITY)).toHaveLength(4);
    expect(searchKeys('a', [items], [1], null)).toHaveLength(4);
    expect(searchKeys('a', [items], [1], undefined)).toHaveLength(4);
  });

  it('validates the number instead of wrapping it', () => {
    const error = expect.objectContaining({
      code: 'InvalidArg',
      message: expect.stringContaining('maxResults must be a non-negative integer or Infinity'),
    });
    for (const maxResults of [Number.NaN, -1, 0.5, Number.NEGATIVE_INFINITY]) {
      expect(() => searchKeys('a', [items], [1], maxResults)).toThrow(error);
    }
  });
});
