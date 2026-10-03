import stringSimilarity from 'string-similarity';
import { describe, expect, it } from 'vitest';

import {
  damerauLevenshtein,
  damerauLevenshteinBatch,
  damerauLevenshteinMany,
  damerauLevenshteinManyU32,
  hamming,
  hammingBatch,
  hammingMany,
  hammingManyU32,
  indel,
  indelBatch,
  indelMany,
  indelManyU32,
  jaro,
  jaroBatch,
  jaroMany,
  jaroManyF64,
  jaroWinkler,
  jaroWinklerBatch,
  jaroWinklerMany,
  jaroWinklerManyF64,
  levenshtein,
  levenshteinBatch,
  levenshteinMany,
  levenshteinManyU32,
  normalizedHamming,
  normalizedHammingBatch,
  normalizedHammingMany,
  normalizedHammingManyF64,
  normalizedIndel,
  normalizedIndelBatch,
  normalizedIndelMany,
  normalizedIndelManyF64,
  normalizedLevenshtein,
  normalizedLevenshteinBatch,
  normalizedLevenshteinMany,
  normalizedLevenshteinManyF64,
  partialRatio,
  partialRatioBatch,
  partialRatioMany,
  partialRatioManyF64,
  sorensenDice,
  sorensenDiceBatch,
  sorensenDiceMany,
  sorensenDiceManyF64,
  tokenSetRatio,
  tokenSetRatioBatch,
  tokenSetRatioMany,
  tokenSetRatioManyF64,
  tokenSortRatio,
  tokenSortRatioBatch,
  tokenSortRatioMany,
  tokenSortRatioManyF64,
  weightedRatio,
  weightedRatioBatch,
  weightedRatioMany,
  weightedRatioManyF64,
} from '../index.js';

// ─── Shared fixtures ─────────────────────────────────────────────────────────

/** Deterministic PRNG (mulberry32) so failures are reproducible. */
function createRng(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Units to build strings from: ASCII, accents (precomposed and combining), CJK, emoji, ZWJ. */
const ALPHABETS: readonly (readonly string[])[] = [
  ['a', 'b', 'c'],
  ['a', 'b', 'c', 'd', ' '],
  ['a', 'é', 'é', '😀', '😃', '中', '日', '本', ' ', 'A', 'ß', '\t'],
  ['x', 'y', '👍🏽', '‍', '🇯', '🇵', '́', ' '],
  ['New', 'new', 'York', 'york', 'Mets', ' ', '  ', 'a', 'b'],
];

function pick<T>(rng: () => number, items: readonly T[]): T {
  const item = items[Math.floor(rng() * items.length)];
  if (item === undefined) throw new Error('pick() from an empty list');
  return item;
}

function randomString(rng: () => number, alphabet: readonly string[], maxUnits: number): string {
  const units = Math.floor(rng() * (maxUnits + 1));
  let s = '';
  for (let i = 0; i < units; i++) s += pick(rng, alphabet);
  return s;
}

/** A copy of `s` with a few code-point edits, so candidates land near thresholds. */
function mutate(rng: () => number, s: string, alphabet: readonly string[]): string {
  const chars = Array.from(s);
  const edits = 1 + Math.floor(rng() * 3);
  for (let i = 0; i < edits; i++) {
    const at = Math.floor(rng() * (chars.length + 1));
    const op = rng();
    if (op < 0.3 && at + 1 < chars.length) {
      const a = chars[at];
      const b = chars[at + 1];
      if (a !== undefined && b !== undefined) {
        chars[at] = b;
        chars[at + 1] = a;
      }
    } else if (op < 0.55) chars.splice(at, 1);
    else if (op < 0.8) chars.splice(at, 0, pick(rng, alphabet));
    else chars[Math.min(at, chars.length - 1)] = pick(rng, alphabet);
  }
  return chars.join('');
}

interface Case {
  readonly reference: string;
  readonly candidates: readonly string[];
}

function generateCases(seed: number, count: number): Case[] {
  const rng = createRng(seed);
  const cases: Case[] = [];
  for (let i = 0; i < count; i++) {
    const alphabet = pick(rng, ALPHABETS);
    const maxUnits = rng() < 0.85 ? 10 : 40;
    const reference = randomString(rng, alphabet, maxUnits);
    const candidates = [
      reference,
      mutate(rng, reference, alphabet),
      mutate(rng, reference, alphabet),
      mutate(rng, mutate(rng, reference, alphabet), alphabet),
      randomString(rng, alphabet, maxUnits),
      randomString(rng, pick(rng, ALPHABETS), maxUnits),
      '',
      ' \t ',
    ];
    cases.push({ reference, candidates });
  }
  return cases;
}

const CASES = generateCases(0x5eed, 200);

/** Distinguishes 0 / -0 and treats NaN as equal to itself, like the Rust bitwise checks. */
function expectSame(actual: unknown, expected: unknown, context: string): void {
  if (!Object.is(actual, expected)) {
    expect.fail(`${context}: got ${String(actual)}, expected ${String(expected)}`);
  }
}

function expectInvalidArg(fn: () => unknown): void {
  let caught: unknown;
  try {
    fn();
  } catch (error) {
    caught = error;
  }
  expect(caught).toBeInstanceOf(Error);
  expect(caught).toMatchObject({ code: 'InvalidArg' });
}

type Pairs = string[][];
type MaybeThreshold = number | undefined | null;

interface SimilarityFamily {
  readonly name: string;
  readonly single: (a: string, b: string) => number;
  readonly batch: (pairs: Pairs) => number[];
  readonly many: (
    reference: string,
    candidates: string[],
    minSimilarity?: MaybeThreshold,
  ) => number[];
  readonly manyF64: (
    reference: string,
    candidates: string[],
    minSimilarity?: MaybeThreshold,
  ) => Float64Array;
}

interface DistanceFamily {
  readonly name: string;
  readonly single: (a: string, b: string) => number;
  readonly batch: (pairs: Pairs) => number[];
  readonly many: (
    reference: string,
    candidates: string[],
    maxDistance?: MaybeThreshold,
  ) => number[];
  readonly manyU32: (
    reference: string,
    candidates: string[],
    maxDistance?: MaybeThreshold,
  ) => Uint32Array;
}

const SIMILARITY_FAMILIES: readonly SimilarityFamily[] = [
  { name: 'jaro', single: jaro, batch: jaroBatch, many: jaroMany, manyF64: jaroManyF64 },
  {
    name: 'jaroWinkler',
    single: jaroWinkler,
    batch: jaroWinklerBatch,
    many: jaroWinklerMany,
    manyF64: jaroWinklerManyF64,
  },
  {
    name: 'sorensenDice',
    single: sorensenDice,
    batch: sorensenDiceBatch,
    many: sorensenDiceMany,
    manyF64: sorensenDiceManyF64,
  },
  {
    name: 'normalizedLevenshtein',
    single: normalizedLevenshtein,
    batch: normalizedLevenshteinBatch,
    many: normalizedLevenshteinMany,
    manyF64: normalizedLevenshteinManyF64,
  },
  {
    name: 'normalizedIndel',
    single: normalizedIndel,
    batch: normalizedIndelBatch,
    many: normalizedIndelMany,
    manyF64: normalizedIndelManyF64,
  },
  {
    name: 'tokenSortRatio',
    single: tokenSortRatio,
    batch: tokenSortRatioBatch,
    many: tokenSortRatioMany,
    manyF64: tokenSortRatioManyF64,
  },
  {
    name: 'tokenSetRatio',
    single: tokenSetRatio,
    batch: tokenSetRatioBatch,
    many: tokenSetRatioMany,
    manyF64: tokenSetRatioManyF64,
  },
  {
    name: 'partialRatio',
    single: partialRatio,
    batch: partialRatioBatch,
    many: partialRatioMany,
    manyF64: partialRatioManyF64,
  },
  {
    name: 'weightedRatio',
    single: weightedRatio,
    batch: weightedRatioBatch,
    many: weightedRatioMany,
    manyF64: weightedRatioManyF64,
  },
];

const DISTANCE_FAMILIES: readonly DistanceFamily[] = [
  {
    name: 'levenshtein',
    single: levenshtein,
    batch: levenshteinBatch,
    many: levenshteinMany,
    manyU32: levenshteinManyU32,
  },
  {
    name: 'damerauLevenshtein',
    single: damerauLevenshtein,
    batch: damerauLevenshteinBatch,
    many: damerauLevenshteinMany,
    manyU32: damerauLevenshteinManyU32,
  },
  { name: 'indel', single: indel, batch: indelBatch, many: indelMany, manyU32: indelManyU32 },
];

const FIXED_SIMILARITY_THRESHOLDS: readonly number[] = [
  0,
  0.3,
  0.5,
  0.7,
  0.8,
  0.9,
  1,
  1.5,
  2,
  -0.5,
  Number.POSITIVE_INFINITY,
  Number.NEGATIVE_INFINITY,
];

const U32_MAX = 4294967295;
const FIXED_MAX_DISTANCES: readonly number[] = [0, 1, 2, 3, 5, 10, U32_MAX - 1, U32_MAX];

/** The rule every `*Many` similarity function follows: keep scores >= the threshold. */
function filterSimilarity(score: number, minSimilarity: number): number {
  return score >= minSimilarity ? score : 0;
}

/** The rule every `*Many` distance function follows, with the sentinel capped at u32::MAX. */
function filterDistance(distance: number, maxDistance: number): number {
  return distance <= maxDistance ? distance : Math.min(maxDistance + 1, U32_MAX);
}

// ─── Differential: single vs Batch vs Many vs typed-array variants ──────────

/** Asserts that every named result list equals `expected` element-wise (see `expectSame`). */
function expectLists(
  context: string,
  expected: readonly unknown[],
  actual: Readonly<Record<string, ArrayLike<unknown>>>,
): void {
  for (const [label, values] of Object.entries(actual)) {
    expect(values.length, `${context} ${label} length`).toBe(expected.length);
    for (const [i, value] of expected.entries()) {
      expectSame(values[i], value, `${context} ${label}[${i}]`);
    }
  }
}

function describeCase(name: string, reference: string, candidates: readonly string[]): string {
  return `${name}(${JSON.stringify(reference)}, ${JSON.stringify(candidates)})`;
}

describe('distance differential (single = Batch = Many = typed array)', () => {
  for (const family of SIMILARITY_FAMILIES) {
    it(`${family.name}: every entry point returns the identical score`, () => {
      for (const { reference, candidates } of CASES) {
        const cands = [...candidates];
        const ctx = describeCase(family.name, reference, cands);
        const scores = cands.map((c) => family.single(reference, c));
        expect(
          scores.every((s) => s >= 0 && s <= 1),
          `${ctx} in [0, 1]`,
        ).toBe(true);
        expectLists(ctx, scores, {
          Batch: family.batch(cands.map((c) => [reference, c])),
          Many: family.many(reference, cands),
          ManyF64: family.manyF64(reference, cands),
        });
        // Fixed thresholds plus each candidate's exact score (the boundary must pass).
        for (const threshold of [...FIXED_SIMILARITY_THRESHOLDS, ...scores]) {
          expectLists(
            `${ctx} minSimilarity=${threshold}`,
            scores.map((s) => filterSimilarity(s, threshold)),
            {
              Many: family.many(reference, cands, threshold),
              ManyF64: family.manyF64(reference, cands, threshold),
            },
          );
        }
      }
    });
  }

  for (const family of DISTANCE_FAMILIES) {
    it(`${family.name}: every entry point returns the identical distance`, () => {
      for (const { reference, candidates } of CASES) {
        const cands = [...candidates];
        const ctx = describeCase(family.name, reference, cands);
        const distances = cands.map((c) => family.single(reference, c));
        expectLists(ctx, distances, {
          Batch: family.batch(cands.map((c) => [reference, c])),
          Many: family.many(reference, cands),
          ManyU32: family.manyU32(reference, cands),
        });
        for (const maxDistance of [...FIXED_MAX_DISTANCES, ...distances]) {
          expectLists(
            `${ctx} maxDistance=${maxDistance}`,
            distances.map((d) => filterDistance(d, maxDistance)),
            {
              Many: family.many(reference, cands, maxDistance),
              ManyU32: family.manyU32(reference, cands, maxDistance),
            },
          );
        }
      }
    });
  }

  it('hamming: every entry point agrees, with null / sentinel for unequal lengths', () => {
    for (const { reference, candidates } of CASES) {
      const cands = [...candidates];
      const ctx = describeCase('hamming', reference, cands);
      const distances = cands.map((c) => hamming(reference, c));
      expectLists(ctx, distances, {
        Batch: hammingBatch(cands.map((c) => [reference, c])),
        Many: hammingMany(reference, cands),
      });
      expectLists(
        ctx,
        distances.map((d) => d ?? U32_MAX),
        { ManyU32: hammingManyU32(reference, cands) },
      );
      for (const maxDistance of FIXED_MAX_DISTANCES) {
        const expected = distances.map((d) => (d != null && d <= maxDistance ? d : null));
        expectLists(`${ctx} maxDistance=${maxDistance}`, expected, {
          Many: hammingMany(reference, cands, maxDistance),
        });
        expectLists(
          `${ctx} maxDistance=${maxDistance}`,
          expected.map((d) => d ?? U32_MAX),
          { ManyU32: hammingManyU32(reference, cands, maxDistance) },
        );
      }
    }
  });

  it('normalizedHamming: every entry point agrees, with null / NaN for unequal lengths', () => {
    for (const { reference, candidates } of CASES) {
      const cands = [...candidates];
      const ctx = describeCase('normalizedHamming', reference, cands);
      const scores = cands.map((c) => normalizedHamming(reference, c));
      expectLists(ctx, scores, {
        Batch: normalizedHammingBatch(cands.map((c) => [reference, c])),
        Many: normalizedHammingMany(reference, cands),
      });
      expectLists(
        ctx,
        scores.map((s) => s ?? Number.NaN),
        { ManyF64: normalizedHammingManyF64(reference, cands) },
      );
      const exact = scores.filter((s): s is number => s != null);
      for (const threshold of [...FIXED_SIMILARITY_THRESHOLDS, ...exact]) {
        const expected = scores.map((s) => (s != null && s >= threshold ? s : null));
        expectLists(`${ctx} minSimilarity=${threshold}`, expected, {
          Many: normalizedHammingMany(reference, cands, threshold),
        });
        expectLists(
          `${ctx} minSimilarity=${threshold}`,
          expected.map((s) => s ?? Number.NaN),
          { ManyF64: normalizedHammingManyF64(reference, cands, threshold) },
        );
      }
    }
  });

  it('the typed-array variants return Uint32Array / Float64Array', () => {
    for (const family of SIMILARITY_FAMILIES) {
      expect(family.manyF64('kitten', ['sitting'])).toBeInstanceOf(Float64Array);
    }
    for (const family of DISTANCE_FAMILIES) {
      expect(family.manyU32('kitten', ['sitting'])).toBeInstanceOf(Uint32Array);
    }
    expect(hammingManyU32('abc', ['abd'])).toBeInstanceOf(Uint32Array);
    expect(normalizedHammingManyF64('abc', ['abd'])).toBeInstanceOf(Float64Array);
  });
});

// ─── sorensenDice counts characters, not UTF-8 bytes ────────────────────────

describe('sorensenDice on non-ASCII input', () => {
  // 2 * |shared bigrams| / (|bigrams(a)| + |bigrams(b)|), counted in characters.
  const cases: readonly (readonly [string, string, number])[] = [
    ['日本', '日本人', 2 / 3],
    ['😀😃', '😀😃😄', 2 / 3],
    ['日本語', '日本人', 0.5],
    ['café', 'cafe', 2 / 3],
    ['naïve', 'naïve', 1],
    ['é', 'é', 1],
    ['é', 'è', 0],
    // Whitespace is still removed before the bigrams are built.
    ['東 京', '東京', 1],
  ];

  for (const [a, b, expected] of cases) {
    it(`sorensenDice(${JSON.stringify(a)}, ${JSON.stringify(b)}) = ${expected}`, () => {
      expect(sorensenDice(a, b)).toBeCloseTo(expected, 12);
      expect(sorensenDice(b, a)).toBeCloseTo(expected, 12);
      expectSame(sorensenDiceBatch([[a, b]])[0], sorensenDice(a, b), 'Batch');
      expectSame(sorensenDiceMany(a, [b])[0], sorensenDice(a, b), 'Many');
    });
  }

  it('keeps ASCII results unchanged', () => {
    expect(sorensenDice('night', 'nacht')).toBe(0.25);
    expect(sorensenDice('a', 'b')).toBe(0);
    expect(sorensenDice('a b', 'ab')).toBe(1);
    expect(sorensenDice('', '')).toBe(1);
  });

  it('matches string-similarity (documented as the same algorithm) on BMP text', () => {
    // string-similarity counts UTF-16 code units, which are characters here.
    const rng = createRng(0xd1ce);
    const alphabet = ['a', 'b', 'c', 'é', 'ü', '日', '本', '語', 'ア', ' ', 'Z'];
    for (let i = 0; i < 2000; i++) {
      const a = randomString(rng, alphabet, 12);
      const b = rng() < 0.5 ? mutate(rng, a, alphabet) : randomString(rng, alphabet, 12);
      const expected = stringSimilarity.compareTwoStrings(a, b);
      expectSame(
        sorensenDice(a, b),
        expected,
        `sorensenDice(${JSON.stringify(a)}, ${JSON.stringify(b)})`,
      );
    }
  });
});

// ─── *Many cutoffs keep scores exactly equal to the threshold ───────────────

describe('similarity *Many keeps a score equal to minSimilarity', () => {
  it('jaroWinklerMany keeps a candidate whose score equals the threshold', () => {
    const score = jaroWinkler('aaac dc ', 'aa');
    expect(score).toBe(0.8);
    expect(jaroWinklerMany('aaac dc ', ['aa'], score)).toEqual([score]);
    expect(Array.from(jaroWinklerManyF64('aaac dc ', ['aa'], score))).toEqual([score]);
    expect(jaroWinklerMany('\txAB', ['\tXAB'], jaroWinkler('\txAB', '\tXAB'))[0]).toBe(
      jaroWinkler('\txAB', '\tXAB'),
    );
  });
});

// ─── tokenSetRatio with no shared tokens ─────────────────────────────────────

describe('tokenSetRatio without shared tokens', () => {
  it('scores strings with no shared tokens and no shared characters as 0', () => {
    expect(tokenSetRatio('cat', 'dog')).toBe(0);
    expect(tokenSetRatio('Jan', 'Feb')).toBe(0);
    expect(tokenSetRatioMany('cat', ['dog', 'b'])).toEqual([0, 0]);
    expect(tokenSetRatioBatch([['abc', 'xyz uvw']])).toEqual([0]);
  });

  it('compares the sorted token remainders when no token is shared', () => {
    // No shared tokens: the score is the plain ratio of the sorted remainders.
    expect(tokenSetRatio('ab cd', 'ab_ cd_')).toBe(normalizedLevenshtein('ab cd', 'ab_ cd_'));
    expect(tokenSetRatio('b a', 'xa')).toBe(normalizedLevenshtein('a b', 'xa'));
    expect(tokenSetRatio('red blue', 'cat dog')).toBe(normalizedLevenshtein('blue red', 'cat dog'));
  });

  it('feeds the corrected score into weightedRatio', () => {
    expect(weightedRatio('cat', 'dog')).toBe(0);
    expect(weightedRatioMany('cat', ['dog'])).toEqual([0]);
  });
});

// ─── *Batch rejects malformed pairs ──────────────────────────────────────────

describe('*Batch rejects pairs that are not exactly two strings', () => {
  const batchFunctions: readonly (readonly [string, (pairs: Pairs) => unknown])[] = [
    ['levenshteinBatch', levenshteinBatch],
    ['damerauLevenshteinBatch', damerauLevenshteinBatch],
    ['hammingBatch', hammingBatch],
    ['normalizedHammingBatch', normalizedHammingBatch],
    ['jaroBatch', jaroBatch],
    ['jaroWinklerBatch', jaroWinklerBatch],
    ['sorensenDiceBatch', sorensenDiceBatch],
    ['normalizedLevenshteinBatch', normalizedLevenshteinBatch],
    ['indelBatch', indelBatch],
    ['normalizedIndelBatch', normalizedIndelBatch],
    ['tokenSortRatioBatch', tokenSortRatioBatch],
    ['tokenSetRatioBatch', tokenSetRatioBatch],
    ['partialRatioBatch', partialRatioBatch],
    ['weightedRatioBatch', weightedRatioBatch],
  ];

  for (const [name, fn] of batchFunctions) {
    it(`${name} throws InvalidArg for a short, empty or long pair`, () => {
      expectInvalidArg(() => fn([['abc'], ['abc', 'abd']]));
      expectInvalidArg(() => fn([['abc', 'abd'], []]));
      expectInvalidArg(() => fn([['abc', 'abd', 'zzz']]));
      expect(() => fn([])).not.toThrow();
      expect(() => fn([['abc', 'abd']])).not.toThrow();
    });
  }

  it('names the offending pair in the message', () => {
    expect(() => levenshteinBatch([['a', 'b'], ['abc']])).toThrow(/pairs\[1\].*exactly 2.*got 1/);
  });
});

// ─── *Many threshold handling is uniform ─────────────────────────────────────

describe('*Many threshold handling', () => {
  const similarityManyFunctions: readonly (readonly [
    string,
    (reference: string, candidates: string[], minSimilarity?: MaybeThreshold) => ArrayLike<unknown>,
  ])[] = [
    ...SIMILARITY_FAMILIES.flatMap(
      (f) =>
        [
          [`${f.name}Many`, f.many],
          [`${f.name}ManyF64`, f.manyF64],
        ] as const,
    ),
    ['normalizedHammingMany', normalizedHammingMany],
    ['normalizedHammingManyF64', normalizedHammingManyF64],
  ];

  for (const [name, fn] of similarityManyFunctions) {
    it(`${name} rejects a NaN minSimilarity`, () => {
      expectInvalidArg(() => fn('hello', ['hello', 'help'], Number.NaN));
      expect(() => fn('hello', ['hello'], undefined)).not.toThrow();
      expect(() => fn('hello', ['hello'], null)).not.toThrow();
    });
  }

  for (const family of SIMILARITY_FAMILIES) {
    it(`${family.name}Many filters even an exact match when minSimilarity > 1`, () => {
      expect(family.many('hello', ['hello', ''], 2)).toEqual([0, 0]);
      expect(family.many('', [''], 1.5)).toEqual([0]);
      expect(family.many('hello', ['hello'], Number.POSITIVE_INFINITY)).toEqual([0]);
      expect(family.many('hello', ['hello'], 1)).toEqual([1]);
    });
  }

  it('normalizedHammingMany filters an exact match when minSimilarity > 1', () => {
    expect(normalizedHammingMany('abc', ['abc'], 2)).toEqual([null]);
  });

  describe('maxDistance at the top of the u32 range (#732)', () => {
    for (const family of DISTANCE_FAMILIES) {
      it(`${family.name}Many does not overflow for maxDistance = 4294967295`, () => {
        const cands = ['kitten', 'sitting', ''];
        const plain = family.many('kitten', cands);
        expect(family.many('kitten', cands, U32_MAX)).toEqual(plain);
        expect(Array.from(family.manyU32('kitten', cands, U32_MAX))).toEqual(plain);
        expect(family.many('kitten', cands, U32_MAX - 1)).toEqual(plain);
        // Negative values wrap to the top of the u32 range, which disables filtering.
        expect(family.many('kitten', cands, -1)).toEqual(plain);
      });
    }

    it('hammingMany does not overflow for maxDistance = 4294967295', () => {
      expect(hammingMany('abc', ['abd', 'ab'], U32_MAX)).toEqual([1, null]);
      expect(hammingMany('abc', ['abd', 'ab'], -1)).toEqual([1, null]);
    });
  });
});
