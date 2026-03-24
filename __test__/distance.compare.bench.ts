// Competitor comparison benchmarks (local use only, excluded from CodSpeed CI)
import { distance as fastestLevenshteinDistance } from 'fastest-levenshtein';
import * as fuzz from 'fuzzball';
import leven from 'leven';
import stringSimilarity from 'string-similarity';
import { bench, describe } from 'vitest';
import {
  levenshtein,
  levenshteinMany,
  normalizedLevenshtein,
  sorensenDice,
  tokenSetRatio,
  tokenSortRatio,
  weightedRatio,
} from '../index.js';

// Test data — realistic string pairs of varying length and similarity
const pairs: [string, string][] = [
  ['kitten', 'sitting'],
  ['saturday', 'sunday'],
  ['rosettacode', 'raisethysword'],
  ['pneumonoultramicroscopicsilicovolcanoconiosis', 'ultramicroscopically'],
  ['the quick brown fox jumps over the lazy dog', 'the fast brown fox leaps over the lazy dog'],
  ['abcdefghijklmnopqrstuvwxyz', 'zyxwvutsrqponmlkjihgfedcba'],
];

describe('Levenshtein Distance (vs competitors)', () => {
  bench('rapid-fuzzy', () => {
    for (const [a, b] of pairs) {
      levenshtein(a, b);
    }
  });

  bench('fastest-levenshtein', () => {
    for (const [a, b] of pairs) {
      fastestLevenshteinDistance(a, b);
    }
  });

  bench('leven', () => {
    for (const [a, b] of pairs) {
      leven(a, b);
    }
  });

  bench('fuzzball', () => {
    for (const [a, b] of pairs) {
      fuzz.distance(a, b);
    }
  });
});

describe('Normalized Similarity (vs competitors)', () => {
  bench('rapid-fuzzy (normalizedLevenshtein)', () => {
    for (const [a, b] of pairs) {
      normalizedLevenshtein(a, b);
    }
  });

  bench('string-similarity (compareTwoStrings / Dice)', () => {
    for (const [a, b] of pairs) {
      stringSimilarity.compareTwoStrings(a, b);
    }
  });

  bench('rapid-fuzzy (sorensenDice)', () => {
    for (const [a, b] of pairs) {
      sorensenDice(a, b);
    }
  });

  bench('fuzzball (ratio)', () => {
    for (const [a, b] of pairs) {
      fuzz.ratio(a, b);
    }
  });
});

describe('Token-Based Ratio (vs competitors)', () => {
  bench('rapid-fuzzy (tokenSetRatio)', () => {
    for (const [a, b] of pairs) {
      tokenSetRatio(a, b);
    }
  });

  bench('rapid-fuzzy (tokenSortRatio)', () => {
    for (const [a, b] of pairs) {
      tokenSortRatio(a, b);
    }
  });

  bench('rapid-fuzzy (weightedRatio)', () => {
    for (const [a, b] of pairs) {
      weightedRatio(a, b);
    }
  });

  bench('fuzzball (token_set_ratio)', () => {
    for (const [a, b] of pairs) {
      fuzz.token_set_ratio(a, b);
    }
  });

  bench('fuzzball (token_sort_ratio)', () => {
    for (const [a, b] of pairs) {
      fuzz.token_sort_ratio(a, b);
    }
  });

  bench('fuzzball (WRatio)', () => {
    for (const [a, b] of pairs) {
      fuzz.WRatio(a, b);
    }
  });
});

// --- Many candidates (1-to-N comparison) ---

const manyCandidates = Array.from({ length: 1_000 }, (_, i) => {
  const words = [
    'kitten',
    'sitting',
    'saturday',
    'sunday',
    'hello',
    'world',
    'fuzzy',
    'search',
    'match',
    'distance',
  ];
  return `${words[i % words.length]}${i}`;
});

describe('Levenshtein Distance — Many 1K (vs competitors)', () => {
  bench('rapid-fuzzy (many)', () => {
    levenshteinMany('kitten', manyCandidates);
  });

  bench('rapid-fuzzy (loop)', () => {
    for (const c of manyCandidates) {
      levenshtein('kitten', c);
    }
  });

  bench('fastest-levenshtein (loop)', () => {
    for (const c of manyCandidates) {
      fastestLevenshteinDistance('kitten', c);
    }
  });

  bench('fuzzball (extract)', () => {
    fuzz.extract('kitten', manyCandidates, { scorer: fuzz.ratio, limit: manyCandidates.length });
  });
});
