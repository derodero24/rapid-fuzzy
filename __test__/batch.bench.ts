import { bench, describe } from 'vitest';
import {
  levenshtein,
  levenshteinMany,
  normalizedLevenshtein,
  normalizedLevenshteinMany,
} from '../index.js';
import { manyCandidates } from './bench-fixtures.js';

describe('Levenshtein Distance — Many (1K candidates)', () => {
  bench('rapid-fuzzy (many)', () => {
    levenshteinMany('kitten', manyCandidates);
  });

  bench('rapid-fuzzy (loop)', () => {
    for (const c of manyCandidates) {
      levenshtein('kitten', c);
    }
  });
});

describe('Normalized Levenshtein — Many (1K candidates)', () => {
  bench('rapid-fuzzy (many)', () => {
    normalizedLevenshteinMany('kitten', manyCandidates);
  });

  bench('rapid-fuzzy (loop)', () => {
    for (const c of manyCandidates) {
      normalizedLevenshtein('kitten', c);
    }
  });
});
