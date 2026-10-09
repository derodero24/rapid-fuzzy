// Competitor comparison benchmarks (local use only, not run in CI)
import { distance as fastestLevenshteinDistance } from 'fastest-levenshtein';
import * as fuzz from 'fuzzball';
import leven from 'leven';
import stringSimilarity from 'string-similarity';
import { test } from 'vitest';
import {
  levenshtein,
  levenshteinMany,
  normalizedLevenshtein,
  sorensenDice,
  tokenSetRatio,
  tokenSortRatio,
  weightedRatio,
} from '../index.js';
import * as fixtures from './bench-fixtures.js';

// Vitest's module runner turns imported bindings into getters: copy the
// fixtures into local constants so the measured functions don't call a getter
// on every iteration.
const { manyCandidates, pairs } = fixtures;

test('Levenshtein Distance (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      for (const [a, b] of pairs) {
        levenshtein(a, b);
      }
    }),
    bench('fastest-levenshtein', () => {
      for (const [a, b] of pairs) {
        fastestLevenshteinDistance(a, b);
      }
    }),
    bench('leven', () => {
      for (const [a, b] of pairs) {
        leven(a, b);
      }
    }),
    bench('fuzzball', () => {
      for (const [a, b] of pairs) {
        fuzz.distance(a, b);
      }
    }),
  );
});

test('Normalized Similarity (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (normalizedLevenshtein)', () => {
      for (const [a, b] of pairs) {
        normalizedLevenshtein(a, b);
      }
    }),
    bench('string-similarity (compareTwoStrings / Dice)', () => {
      for (const [a, b] of pairs) {
        stringSimilarity.compareTwoStrings(a, b);
      }
    }),
    bench('rapid-fuzzy (sorensenDice)', () => {
      for (const [a, b] of pairs) {
        sorensenDice(a, b);
      }
    }),
    bench('fuzzball (ratio)', () => {
      for (const [a, b] of pairs) {
        fuzz.ratio(a, b);
      }
    }),
  );
});

test('Token-Based Ratio (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (tokenSetRatio)', () => {
      for (const [a, b] of pairs) {
        tokenSetRatio(a, b);
      }
    }),
    bench('rapid-fuzzy (tokenSortRatio)', () => {
      for (const [a, b] of pairs) {
        tokenSortRatio(a, b);
      }
    }),
    bench('rapid-fuzzy (weightedRatio)', () => {
      for (const [a, b] of pairs) {
        weightedRatio(a, b);
      }
    }),
    bench('fuzzball (token_set_ratio)', () => {
      for (const [a, b] of pairs) {
        fuzz.token_set_ratio(a, b);
      }
    }),
    bench('fuzzball (token_sort_ratio)', () => {
      for (const [a, b] of pairs) {
        fuzz.token_sort_ratio(a, b);
      }
    }),
    bench('fuzzball (WRatio)', () => {
      for (const [a, b] of pairs) {
        fuzz.WRatio(a, b);
      }
    }),
  );
});

// --- Many candidates (1-to-N comparison) ---

test('Levenshtein Distance — Many 1K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (many)', () => {
      levenshteinMany('kitten', manyCandidates);
    }),
    bench('rapid-fuzzy (loop)', () => {
      for (const c of manyCandidates) {
        levenshtein('kitten', c);
      }
    }),
    bench('fastest-levenshtein (loop)', () => {
      for (const c of manyCandidates) {
        fastestLevenshteinDistance('kitten', c);
      }
    }),
    bench('fuzzball (extract)', () => {
      fuzz.extract('kitten', manyCandidates, { scorer: fuzz.ratio, limit: manyCandidates.length });
    }),
  );
});
