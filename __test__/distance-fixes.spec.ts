import stringSimilarity from 'string-similarity';
import { describe, expect, it } from 'vitest';

import {
  normalizedLevenshtein,
  sorensenDice,
  sorensenDiceBatch,
  sorensenDiceMany,
  tokenSetRatio,
  tokenSetRatioBatch,
  tokenSetRatioMany,
  weightedRatio,
  weightedRatioMany,
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

/** Distinguishes 0 / -0 and treats NaN as equal to itself, like the Rust bitwise checks. */
function expectSame(actual: unknown, expected: unknown, context: string): void {
  if (!Object.is(actual, expected)) {
    expect.fail(`${context}: got ${String(actual)}, expected ${String(expected)}`);
  }
}

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
