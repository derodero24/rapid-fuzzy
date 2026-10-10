import { describe, expect, it } from 'vitest';

import {
  closest,
  FuzzyIndex,
  type IndexSearchResult,
  KeyedFuzzyIndex,
  type SearchOptions,
  type SearchResult,
  search,
  searchKeys,
} from '../index.js';
import { FuzzyObjectIndex, searchObjects } from '../objects.js';

// FuzzyIndex must return exactly what search() returns over the same items:
// same items, indices, scores, order, positions and match types, at any size,
// for Unicode input and query syntax, with a warm or cold incremental cache,
// and after add/addMany/remove.

// Invisible or confusable characters, spelled out so they stay readable.
const IDEOGRAPHIC_SPACE = String.fromCodePoint(0x3000);
const NO_BREAK_SPACE = String.fromCodePoint(0xa0);
const COMBINING_ACUTE = String.fromCodePoint(0x301);
const KELVIN_SIGN = String.fromCodePoint(0x212a);
const NFD_ECOLE = `e${COMBINING_ACUTE}cole`;

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

function pick<T>(random: Random, values: readonly T[]): T {
  const value = values[Math.floor(random() * values.length)];
  if (value === undefined) throw new Error('pick() from an empty list');
  return value;
}

function repeat(random: Random, maxTimes: number, piece: () => string): string {
  let out = '';
  const times = 1 + Math.floor(random() * maxTimes);
  for (let i = 0; i < times; i++) out += piece();
  return out;
}

/**
 * Item building blocks: ASCII of both cases, separators, query syntax
 * characters, letters with non-trivial case folding or normalization, NFD
 * sequences, CJK, Unicode whitespace and multi-codepoint graphemes.
 */
const ITEM_PIECES: readonly string[] = [
  ..."abcdefhlnorsxABFOR_/.- $\\!^'01",
  ...'éÉłŁóÓźŹмМоОсСкКſßİıΣσςǅＡ東京港区',
  `e${COMBINING_ACUTE}`,
  KELVIN_SIGN,
  IDEOGRAPHIC_SPACE,
  NO_BREAK_SPACE,
  '\t',
  '👍',
  '👍🏽',
  'foo',
  'bar',
  'handler',
  'src/index.ts',
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
  '\\',
  '\\ ',
];

const FIXED_QUERIES: readonly string[] = [
  'hndlr',
  'src/index.ts',
  'москва',
  'łódź',
  'ecole',
  'k',
  'foo\\ bar',
  'foo\\$',
  'bar$',
  'fob !zzz',
  '^f ob',
  '^',
  '$',
  '!',
  "'",
  '^$',
  `東京${IDEOGRAPHIC_SPACE}港区`,
  `foo${NO_BREAK_SPACE}bar`,
  'foo\tbar',
  'fo$x',
];

function genItem(random: Random): string {
  return repeat(random, 6, () => pick(random, ITEM_PIECES));
}

/** A subsequence of an item's characters, sometimes split into terms. */
function derivedQuery(random: Random, item: string): string {
  const chars = Array.from(item);
  let query = '';
  for (const char of chars.slice(Math.floor(random() * chars.length))) {
    if (random() < 0.7) query += char;
    else if (random() < 0.1) query += ' ';
    if (query.length >= 6) break;
  }
  return random() < 0.2 ? query + pick(random, QUERY_PIECES) : query;
}

function genQuery(random: Random, items: readonly string[]): string {
  const roll = random();
  if (roll < 0.4 && items.length > 0) return derivedQuery(random, pick(random, items));
  if (roll < 0.7) return pick(random, FIXED_QUERIES);
  return repeat(random, 4, () => pick(random, random() < 0.3 ? QUERY_PIECES : ITEM_PIECES));
}

function genOptions(random: Random): SearchOptions {
  const options: SearchOptions = {
    includePositions: random() < 0.5,
    isCaseSensitive: random() < 0.2,
  };
  // -1 leaves the option unset.
  const maxResults = pick(random, [-1, -1, 0, 1, 3, 10]);
  if (maxResults >= 0) options.maxResults = maxResults;
  const minScore = pick(random, [-1, -1, 0, 0.3, 0.8]);
  if (minScore >= 0) options.minScore = minScore;
  return options;
}

/** Apply a random add / addMany / swap-remove to both the index and its mirror. */
function mutate(random: Random, index: FuzzyIndex, items: string[]): void {
  const action = random();
  if (action < 0.3) {
    const item = genItem(random);
    index.add(item);
    items.push(item);
  } else if (action < 0.5) {
    const batch = [genItem(random), genItem(random)];
    index.addMany(batch);
    items.push(...batch);
  } else if (action < 0.8 && items.length > 0) {
    const at = Math.floor(random() * items.length);
    expect(index.remove(at)).toBe(true);
    const last = items.pop();
    if (last !== undefined && at < items.length) items[at] = last;
  }
}

function asIndexResults(results: readonly SearchResult[]): IndexSearchResult[] {
  return results.map(({ index, score, positions, matchType }) =>
    matchType === undefined ? { index, score, positions } : { index, score, positions, matchType },
  );
}

function expectParity(
  index: FuzzyIndex,
  items: readonly string[],
  query: string,
  options: SearchOptions,
  context: string,
): void {
  const expected = search(query, [...items], options);
  const message = `${context} query=${JSON.stringify(query)} options=${JSON.stringify(options)}`;
  expect(index.search(query, options), message).toEqual(expected);
  expect(index.searchIndices(query, options), message).toEqual(asIndexResults(expected));
}

function expectClosestParity(index: FuzzyIndex, items: readonly string[], query: string): void {
  expect(index.closest(query)).toBe(closest(query, [...items]));
  expect(index.closest(query, 0.5)).toBe(closest(query, [...items], 0.5));
}

/** Type a query one character at a time, as a filter-as-you-type UI does. */
function expectTypingParity(
  index: FuzzyIndex,
  items: readonly string[],
  query: string,
  options: SearchOptions,
  context: string,
): void {
  let typed = '';
  for (const char of query) {
    typed += char;
    expectParity(index, items, typed, options, context);
  }
}

const found = (results: readonly SearchResult[]): string[] => results.map((r) => r.item);

describe('FuzzyIndex matches search() exactly', () => {
  it('on small random corpora with cold and warm caches', () => {
    for (let seed = 1; seed <= 150; seed++) {
      const random = rng(seed);
      const items = Array.from({ length: Math.floor(random() * 30) }, () => genItem(random));
      const context = `seed=${seed} items=${JSON.stringify(items)}`;
      for (let i = 0; i < 3; i++) {
        const query = genQuery(random, items);
        const cold = new FuzzyIndex(items);
        expectParity(cold, items, query, genOptions(random), context);
        expectClosestParity(cold, items, query);
      }
      const warm = new FuzzyIndex(items);
      for (let i = 0; i < 3; i++) {
        expectTypingParity(warm, items, genQuery(random, items), genOptions(random), context);
      }
    }
  });

  it('after add, addMany and remove', () => {
    for (let seed = 500; seed < 560; seed++) {
      const random = rng(seed);
      const items = Array.from({ length: 1 + Math.floor(random() * 20) }, () => genItem(random));
      const index = new FuzzyIndex(items);
      for (let step = 0; step < 8; step++) {
        mutate(random, index, items);
        const context = `seed=${seed} step=${step} items=${JSON.stringify(items)}`;
        expectTypingParity(index, items, genQuery(random, items), genOptions(random), context);
      }
    }
  });

  it('above 5000 items (where a bigram pre-filter used to drop matches, #746)', () => {
    const items = Array.from({ length: 6000 }, (_, i) => `filler_entry_${i}`);
    items.push('handler', 'my_handler_x', 'Москва', NFD_ECOLE, 'foo bar', 'foo$', 'Łódź');
    items.push('src/components/index.ts');
    const index = new FuzzyIndex(items);
    for (const query of FIXED_QUERIES) {
      expectParity(index, items, query, { includePositions: true }, 'large');
    }
    expect(found(index.search('hndlr'))).toEqual(['handler', 'my_handler_x']);
    expect(found(index.search('москва'))).toEqual(['Москва']);
    expect(found(index.search('ecole'))).toEqual([NFD_ECOLE]);
    expect(found(index.search('foo\\ bar'))).toEqual(['foo bar']);
    expect(found(index.search('src/index.ts'))).toEqual(['src/components/index.ts']);
    for (const word of ['handler', 'src/index.ts', 'fo$x', 'foo\\ bar']) {
      expectTypingParity(index, items, word, { maxResults: 10 }, 'large typing');
    }
  });
});

describe('search correctness', () => {
  it('matches uppercase non-ASCII items with lowercase queries', () => {
    const kelvin = `${KELVIN_SIGN}elvin`;
    const items = ['Łódź', 'ŠKODA', 'Ōsaka', kelvin];
    const index = new FuzzyIndex(items);
    for (const [query, expected] of [
      ['łódź', 'Łódź'],
      ['škoda', 'ŠKODA'],
      ['ōsaka', 'Ōsaka'],
      ['kelvin', kelvin],
    ] as const) {
      expect(found(search(query, items))).toEqual([expected]);
      expect(found(index.search(query))).toEqual([expected]);
    }
  });

  it('reports grapheme positions for decomposed (NFD) items', () => {
    const [match] = search('ecole', [NFD_ECOLE], { includePositions: true });
    expect(match?.positions).toEqual([0, 1, 2, 3, 4]);
    expect(match?.matchType).toBe('Exact');
    expect(match?.score).toBe(1);
  });

  it('normalizes scores of anchored and negated queries against their own terms', () => {
    const results = search('bar$', ['foobar', 'xbar', 'bar']);
    expect(found(results)).toEqual(['bar', 'xbar', 'foobar']);
    expect(results.map((r) => r.score < 1)).toEqual([false, true, true]);
    expect(search('fob !zzz', ['fob', 'foobar', 'f_o_b'], { minScore: 0.99 })).toHaveLength(1);
  });

  it('treats syntax-only queries as empty', () => {
    const items = ['a', 'b'];
    const index = new FuzzyIndex(items);
    for (const query of ['^', "'", '$', '!', '^$', '!^', IDEOGRAPHIC_SPACE]) {
      expect(search(query, items)).toEqual([]);
      expect(index.search(query)).toEqual([]);
      expect(index.searchIndices(query)).toEqual([]);
      expect(closest(query, items)).toBeNull();
      expect(search(query, items, { returnAllOnEmpty: true })).toHaveLength(2);
      expect(index.search(query, { returnAllOnEmpty: true })).toHaveLength(2);
      expect(index.searchIndices(query, { returnAllOnEmpty: true })).toHaveLength(2);
    }
  });

  it('does not reuse the incremental cache when syntax changes the meaning', () => {
    const items = ['fo$x', 'xfo', 'foo bar', 'foo\\x', 'foo$', 'a', 'b', 'c', 'd'];
    for (const [first, second] of [
      ['fo$', 'fo$x'],
      ['foo\\', 'foo\\ bar'],
      ['foo\\', 'foo\\$'],
    ] as const) {
      const index = new FuzzyIndex(items);
      index.search(first);
      expect(index.search(second)).toEqual(search(second, items));
      expect(index.search(second)).not.toEqual([]);
    }
  });

  it('splits terms on Unicode whitespace (IME and pasted queries)', () => {
    const items = ['東京都港区', '大阪府', 'foo bar'];
    const index = new FuzzyIndex(items);
    for (const [query, expected] of [
      [`東京${IDEOGRAPHIC_SPACE}港区`, '東京都港区'],
      [`港区${IDEOGRAPHIC_SPACE}東京`, '東京都港区'],
      [`foo${NO_BREAK_SPACE}bar`, 'foo bar'],
      ['bar\tfoo', 'foo bar'],
      ['bar\nfoo', 'foo bar'],
    ] as const) {
      expect(found(search(query, items))).toEqual([expected]);
      expect(found(index.search(query))).toEqual([expected]);
      expect(closest(query, items)).toBe(expected);
    }
  });

  it('scores the longest supported term and rejects longer ones without wrapping', () => {
    const term = 'a'.repeat(2520);
    const [match] = search(term, [term, 'aaa']);
    expect(match?.item).toBe(term);
    expect(match?.score).toBe(1);
    for (const length of [2521, 10_000]) {
      const long = 'a'.repeat(length);
      expect(search(long, [long, 'a'])).toEqual([]);
      expect(new FuzzyIndex([long, 'a']).search(long)).toEqual([]);
      expect(closest(long, [long])).toBeNull();
    }
  });
});

describe('maxResults validation', () => {
  const items = ['a', 'ab', 'abc', 'abcd'];
  const index = new FuzzyIndex(items);
  const keyed = new KeyedFuzzyIndex([items], [1]);

  it('accepts non-negative integers and Infinity', () => {
    for (const [maxResults, expected] of [
      [0, 0],
      [2, 2],
      [Number.POSITIVE_INFINITY, 4],
      [2 ** 32, 4],
      [2 ** 32 + 2, 4],
      [Number.MAX_SAFE_INTEGER, 4],
    ] as const) {
      expect(search('a', items, maxResults)).toHaveLength(expected);
      expect(search('a', items, { maxResults })).toHaveLength(expected);
      expect(index.search('a', maxResults)).toHaveLength(expected);
      expect(index.search('a', { maxResults })).toHaveLength(expected);
      expect(index.searchIndices('a', maxResults)).toHaveLength(expected);
      expect(index.searchIndices('a', { maxResults })).toHaveLength(expected);
      expect(keyed.search('a', maxResults)).toHaveLength(expected);
      expect(keyed.search('a', { maxResults })).toHaveLength(expected);
      expect(searchKeys('a', [items], [1], { maxResults })).toHaveLength(expected);
      expect(search('', items, { maxResults, returnAllOnEmpty: true })).toHaveLength(expected);
    }
  });

  it('throws on NaN, negative and fractional values instead of wrapping', () => {
    const error = expect.objectContaining({
      code: 'InvalidArg',
      message: expect.stringContaining('maxResults must be a non-negative integer or Infinity'),
    });
    for (const maxResults of [Number.NaN, -1, 0.5, -0.5, Number.NEGATIVE_INFINITY]) {
      expect(() => search('a', items, maxResults)).toThrow(error);
      expect(() => search('a', items, { maxResults })).toThrow(error);
      expect(() => index.search('a', maxResults)).toThrow(error);
      expect(() => index.search('a', { maxResults })).toThrow(error);
      expect(() => index.searchIndices('a', maxResults)).toThrow(error);
      expect(() => index.searchIndices('a', { maxResults })).toThrow(error);
      expect(() => keyed.search('a', maxResults)).toThrow(error);
      expect(() => keyed.search('a', { maxResults })).toThrow(error);
      expect(() => searchKeys('a', [items], [1], { maxResults })).toThrow(error);
    }
  });

  it('still accepts omitted, undefined and null options', () => {
    expect(search('a', items)).toHaveLength(4);
    expect(search('a', items, undefined)).toHaveLength(4);
    expect(search('a', items, null)).toHaveLength(4);
    expect(search('a', items, {})).toHaveLength(4);
    expect(index.search('a', null)).toHaveLength(4);
    expect(index.searchIndices('a', undefined)).toHaveLength(4);
  });
});

// No score compares as at least NaN, so a NaN minScore used to filter out
// every match silently, while a NaN maxResults or minSimilarity throws.
describe('minScore validation', () => {
  const items = ['a', 'ab', 'abc', 'abcd'];
  const index = new FuzzyIndex(items);
  const keyed = new KeyedFuzzyIndex([items], [1]);
  const objects = items.map((name) => ({ name }));
  const objectIndex = new FuzzyObjectIndex(objects, { keys: ['name'] });
  const NAN = Number.NaN;

  /** Every API taking a minScore, called with `minScore`. */
  const calls = (minScore: number): Array<[string, () => unknown]> => [
    ['search', () => search('a', items, { minScore })],
    ['closest', () => closest('a', items, minScore)],
    ['FuzzyIndex.search', () => index.search('a', { minScore })],
    ['FuzzyIndex.searchIndices', () => index.searchIndices('a', { minScore })],
    ['FuzzyIndex.closest', () => index.closest('a', minScore)],
    ['searchKeys', () => searchKeys('a', [items], [1], { minScore })],
    ['KeyedFuzzyIndex.search', () => keyed.search('a', { minScore })],
    ['KeyedFuzzyIndex.closest(number)', () => keyed.closest('a', minScore)],
    ['KeyedFuzzyIndex.closest(options)', () => keyed.closest('a', { minScore })],
    ['searchObjects', () => searchObjects('a', objects, { keys: ['name'], minScore })],
    ['FuzzyObjectIndex.search', () => objectIndex.search('a', { minScore })],
    ['FuzzyObjectIndex.closest(number)', () => objectIndex.closest('a', minScore)],
    ['FuzzyObjectIndex.closest(options)', () => objectIndex.closest('a', { minScore })],
  ];

  it('throws an InvalidArg error for NaN in every API', () => {
    const error = expect.objectContaining({
      code: 'InvalidArg',
      message: expect.stringContaining('minScore must be a number, got NaN'),
    });
    for (const [name, call] of calls(NAN)) {
      expect(call, name).toThrow(error);
    }
  });

  it('still accepts -Infinity, values above 1 and Infinity', () => {
    for (const minScore of [Number.NEGATIVE_INFINITY, -1, 0, 2, Number.POSITIVE_INFINITY]) {
      for (const [name, call] of calls(minScore)) {
        expect(call, `${name} minScore=${minScore}`).not.toThrow();
      }
    }
    expect(search('a', items, { minScore: Number.NEGATIVE_INFINITY })).toHaveLength(4);
    expect(search('a', items, { minScore: 2 })).toEqual([]);
    expect(closest('a', items, 2)).toBeNull();
  });
});
