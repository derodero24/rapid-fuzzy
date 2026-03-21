import { bench, describe } from 'vitest';
import {
  levenshtein,
  levenshteinMany,
  normalizedLevenshtein,
  normalizedLevenshteinMany,
} from '../index.js';

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
