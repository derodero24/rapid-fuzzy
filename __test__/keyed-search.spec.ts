import { describe, expect, it } from 'vitest';

import {
  KeyedFuzzyIndex,
  type KeyScoreMode,
  type KeySearchOptions,
  type KeySearchResult,
  type SearchOptions,
  search,
  searchKeys,
} from '../index.js';
import { FuzzyObjectIndex, searchObjects } from '../objects.js';

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
  options?: KeySearchOptions,
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

// ─── scoreMode (#781) ───────────────────────────────────────────────────────

const MODES: readonly KeyScoreMode[] = ['weighted', 'matched', 'max'];

/** The combined score of an item's key scores, as documented for each mode. */
function combine(
  keyScores: readonly number[],
  weights: readonly number[],
  mode: KeyScoreMode,
): number {
  let weightedSum = 0;
  let totalWeight = 0;
  let matchedWeight = 0;
  let max = 0;
  keyScores.forEach((score, k) => {
    const weight = weights[k] ?? 0;
    totalWeight += weight;
    if (weight > 0) {
      weightedSum += score * weight;
      if (score > 0) matchedWeight += weight;
      max = Math.max(max, score);
    }
  });
  switch (mode) {
    case 'weighted':
      return weightedSum / totalWeight;
    case 'matched':
      return weightedSum > 0 ? weightedSum / matchedWeight : 0;
    case 'max':
      return max;
  }
}

/** The error thrown for an invalid scoreMode; `got` describes the value. */
function invalidScoreMode(got: string): unknown {
  return expect.objectContaining({
    code: 'InvalidArg',
    message: `scoreMode must be "weighted", "matched" or "max", got ${got}`,
  });
}

/** Results ordered by index, without their scores. */
function matches(results: readonly KeySearchResult[]): Array<[number, number[]]> {
  return results.map((r): [number, number[]] => [r.index, r.keyScores]).sort(([a], [b]) => a - b);
}

// name, email, bio: "John Smith" matches "smith" exactly in its name only;
// "S. Mitchell" matches it partially in both its name and its email.
const PEOPLE = [
  { name: 'John Smith', email: 'john@example.com', bio: 'Engineer' },
  { name: 'S. Mitchell', email: 'smitchell@example.com', bio: 'Designer' },
  { name: 'Jane Doe', email: 'jane@example.com', bio: 'Writer' },
];
const PEOPLE_KEYS = [
  PEOPLE.map((p) => p.name),
  PEOPLE.map((p) => p.email),
  PEOPLE.map((p) => p.bio),
];

/**
 * Check that `index` (built from `keyTexts` and `weights`) returns what
 * searchKeys() returns for `options`, also with `minScore` set to each
 * achievable score (the early exit's boundary), which keeps that result.
 * Returns the number of results.
 */
function expectIndexMatchesSearchKeys(
  index: KeyedFuzzyIndex,
  query: string,
  keyTexts: string[][],
  weights: number[],
  options: KeySearchOptions,
): number {
  const context = JSON.stringify({ query, keyTexts, weights, options });
  const expected = searchKeys(query, keyTexts, weights, options);
  expect(index.search(query, options), context).toEqual(expected);
  for (const r of expected) {
    const atScore = { ...options, minScore: r.score };
    const kept = searchKeys(query, keyTexts, weights, atScore);
    expect(kept, context).toContainEqual(r);
    expect(index.search(query, atScore), context).toEqual(kept);
  }
  return expected.length;
}

describe('scoreMode', () => {
  it('keeps KeyedFuzzyIndex and searchKeys identical in every mode on random input', () => {
    let nonEmpty = 0;
    for (let seed = 0; seed < 300; seed++) {
      const random = rng(seed);
      const numKeys = 1 + below(random, 4);
      const numItems = below(random, 12);
      const keyTexts = Array.from({ length: numKeys }, () =>
        Array.from({ length: numItems }, () => randomItem(random)),
      );
      const weights = Array.from({ length: numKeys }, () => randomWeight(random));
      if (weights.reduce((a, b) => a + b, 0) <= 0) continue;
      const index = new KeyedFuzzyIndex(keyTexts, weights);
      for (let q = 0; q < 3; q++) {
        const query = randomQuery(random, keyTexts.flat());
        const base = randomOptions(random, numItems);
        for (const scoreMode of MODES) {
          const options = { ...base, scoreMode };
          const found = expectIndexMatchesSearchKeys(index, query, keyTexts, weights, options);
          nonEmpty += Math.min(found, 1);
        }
      }
    }
    expect(nonEmpty).toBeGreaterThan(300);
  });

  it('combines the key scores as documented', () => {
    for (let seed = 0; seed < 200; seed++) {
      const random = rng(1000 + seed);
      const numKeys = 1 + below(random, 4);
      const numItems = 1 + below(random, 10);
      const keyTexts = Array.from({ length: numKeys }, () =>
        Array.from({ length: numItems }, () => randomItem(random)),
      );
      const weights = Array.from({ length: numKeys }, () => randomWeight(random));
      if (weights.reduce((a, b) => a + b, 0) <= 0) continue;
      const query = randomQuery(random, keyTexts.flat());
      const weighted = searchKeys(query, keyTexts, weights);
      for (const scoreMode of MODES) {
        const results = searchKeys(query, keyTexts, weights, { scoreMode });
        // The same items match in every mode, with the same key scores.
        expect(matches(results)).toEqual(matches(weighted));
        for (const [i, r] of results.entries()) {
          expect(r.score).toBe(combine(r.keyScores, weights, scoreMode));
          expect(r.score).toBeLessThanOrEqual(results[i - 1]?.score ?? 1);
        }
      }
    }
  });

  it("defaults to 'weighted'", () => {
    const weights = [2, 1, 1];
    const weighted = searchKeys('smith', PEOPLE_KEYS, weights, { scoreMode: 'weighted' });
    expect(weighted.length).toBeGreaterThan(0);
    const index = new KeyedFuzzyIndex(PEOPLE_KEYS, weights);
    for (const options of [undefined, null, {}, { scoreMode: undefined }, Infinity]) {
      expect(searchKeys('smith', PEOPLE_KEYS, weights, options)).toEqual(weighted);
      expect(index.search('smith', options)).toEqual(weighted);
    }
  });

  it('ranks an exact single-key match first in matched and max modes', () => {
    const weights = [2, 1, 1];
    const byIndex = (results: readonly KeySearchResult[]) =>
      new Map(results.map((r) => [r.index, r]));

    const weighted = searchKeys('smith', PEOPLE_KEYS, weights);
    const exact = byIndex(weighted).get(0);
    const partial = byIndex(weighted).get(1);
    expect(exact?.keyScores).toEqual([1, 0, 0]);
    expect(exact?.score).toBe(0.5);
    const [name = 0, email = 0] = partial?.keyScores ?? [];
    expect(name).toBeGreaterThan(0);
    expect(email).toBeGreaterThan(0);
    expect(partial?.score).toBe((2 * name + email) / 4);

    const matched = searchKeys('smith', PEOPLE_KEYS, weights, { scoreMode: 'matched' });
    expect(matched[0]).toEqual({ index: 0, score: 1, keyScores: [1, 0, 0] });
    expect(byIndex(matched).get(1)?.score).toBe((2 * name + email) / 3);

    const max = searchKeys('smith', PEOPLE_KEYS, weights, { scoreMode: 'max' });
    expect(max[0]).toEqual({ index: 0, score: 1, keyScores: [1, 0, 0] });
    expect(byIndex(max).get(1)?.score).toBe(Math.max(name, email));
  });

  it('applies minScore to the combined score of the mode', () => {
    const weights = [1, 1, 1];
    const index = new KeyedFuzzyIndex(PEOPLE_KEYS, weights);
    // By default the exact surname match scores 1/3 and is dropped.
    expect(indices(searchKeys('smith', PEOPLE_KEYS, weights, { minScore: 0.9 }))).toEqual([]);
    for (const scoreMode of ['matched', 'max'] as const) {
      for (const minScore of [0.95, 1]) {
        const options = { minScore, scoreMode };
        expect(indices(searchKeys('smith', PEOPLE_KEYS, weights, options))).toEqual([0]);
        expect(indices(index.search('smith', options))).toEqual([0]);
      }
    }
  });

  it('ignores keys with weight 0 in every mode', () => {
    const keyTexts = [
      ['apple', 'zzz', 'pineapple'],
      ['zzz', 'apple', 'apple'],
    ];
    const partial = searchKeys('apple', [keyTexts[0] ?? []], [1]).find((r) => r.index === 2);
    const score = partial?.score ?? Number.NaN;
    expect(score).toBeGreaterThan(0);
    expect(score).toBeLessThan(1);
    for (const scoreMode of MODES) {
      const { index, keys } = bothPaths('apple', keyTexts, [1, 0], { scoreMode });
      expect(keys).toEqual([
        { index: 0, score: 1, keyScores: [1, 0] },
        { index: 2, score, keyScores: [score, 1] },
      ]);
      expect(index).toEqual(keys);
    }
  });

  it('returns every item with score 1 for an empty query with returnAllOnEmpty', () => {
    for (const scoreMode of MODES) {
      const { index, keys } = bothPaths('', PEOPLE_KEYS, [1, 0, 2], {
        returnAllOnEmpty: true,
        scoreMode,
      });
      expect(keys).toEqual(PEOPLE.map((_, i) => ({ index: i, score: 1, keyScores: [1, 1, 1] })));
      expect(index).toEqual(keys);
    }
  });

  it('closest() returns the first result of search() in the given mode', () => {
    const index = new KeyedFuzzyIndex(PEOPLE_KEYS, [1, 1, 1]);
    for (const scoreMode of [...MODES, undefined, null]) {
      for (const minScore of [undefined, null, 0, 0.3, 0.6, 0.9, 1]) {
        for (const query of ['smith', 'mitchell', 'jane', 'zzz']) {
          const [best] = index.search(query, {
            maxResults: 1,
            minScore: minScore ?? undefined,
            scoreMode: scoreMode ?? undefined,
          });
          expect(index.closest(query, minScore, scoreMode)).toBe(best?.index ?? null);
        }
      }
    }
    expect(index.closest('smith', 0.9)).toBeNull();
    expect(index.closest('smith', 0.9, 'matched')).toBe(0);
    expect(index.closest('smith', 0.9, 'max')).toBe(0);
  });

  it('is passed through by searchObjects() and FuzzyObjectIndex', () => {
    const keys = [{ name: 'name', weight: 2 }, 'email', 'bio'] as const;
    const objectIndex = new FuzzyObjectIndex(PEOPLE, { keys });
    for (const scoreMode of MODES) {
      for (const query of ['smith', 'mitchell', 'example']) {
        const expected = searchKeys(query, PEOPLE_KEYS, [2, 1, 1], { scoreMode }).map((r) => ({
          item: PEOPLE[r.index],
          ...r,
        }));
        expect(searchObjects(query, PEOPLE, { keys, scoreMode })).toEqual(expected);
        expect(objectIndex.search(query, { scoreMode })).toEqual(expected);
        const [best] = searchObjects(query, PEOPLE, { keys, scoreMode, minScore: 0.6 });
        expect(objectIndex.closest(query, 0.6, scoreMode)).toBe(best?.item ?? null);
      }
    }
    expect(objectIndex.closest('smith', 0.9)).toBeNull();
    expect(objectIndex.closest('smith', 0.9, 'matched')).toBe(PEOPLE[0]);
  });

  it('rejects anything but the three modes with an InvalidArg error', () => {
    const index = new KeyedFuzzyIndex(PEOPLE_KEYS, [1, 1, 1]);
    const objectIndex = new FuzzyObjectIndex(PEOPLE, { keys: ['name', 'email', 'bio'] });
    const invalid: ReadonlyArray<[unknown, string]> = [
      ['mean', '"mean"'],
      ['Weighted', '"Weighted"'],
      ['MAX', '"MAX"'],
      ['', '""'],
      [' max', '" max"'],
      ['matched\n', '"matched\\n"'],
      [1, 'number'],
      [true, 'boolean'],
      [{}, 'object'],
      [['max'], 'object'],
    ];
    for (const [value, got] of invalid) {
      const scoreMode = value as KeyScoreMode;
      const error = invalidScoreMode(got);
      expect(() => searchKeys('smith', PEOPLE_KEYS, [1, 1, 1], { scoreMode })).toThrow(error);
      expect(() => index.search('smith', { scoreMode })).toThrow(error);
      expect(() => index.closest('smith', undefined, scoreMode)).toThrow(error);
      expect(() => searchObjects('smith', PEOPLE, { keys: ['name'], scoreMode })).toThrow(error);
      expect(() => objectIndex.search('smith', { scoreMode })).toThrow(error);
      expect(() => objectIndex.closest('smith', undefined, scoreMode)).toThrow(error);
      // Also when there is nothing to search.
      expect(() => searchKeys('', [[]], [1], { scoreMode })).toThrow(error);
    }
    // In an options object, null is rejected like in the other fields.
    const nullMode = { scoreMode: null } as unknown as KeySearchOptions;
    expect(() => searchKeys('smith', PEOPLE_KEYS, [1, 1, 1], nullMode)).toThrow(
      invalidScoreMode('null'),
    );
  });

  it('is not an option of search()', () => {
    const items = PEOPLE.map((p) => p.name);
    // Unknown SearchOptions fields are ignored, as before.
    const options = { scoreMode: 'max' } as SearchOptions;
    expect(search('smith', items, options)).toEqual(search('smith', items));
  });
});
