import { bench, describe } from 'vitest';
import { damerauLevenshtein, hamming, levenshtein, levenshteinBatch } from '../index.js';
import { equalLengthPairs, pairs } from './bench-fixtures.js';

describe('Levenshtein Distance', () => {
  bench('rapid-fuzzy', () => {
    for (const [a, b] of pairs) {
      levenshtein(a, b);
    }
  });

  bench('rapid-fuzzy (batch)', () => {
    levenshteinBatch(pairs);
  });
});

describe('Damerau-Levenshtein', () => {
  bench('rapid-fuzzy', () => {
    for (const [a, b] of pairs) {
      damerauLevenshtein(a, b);
    }
  });
});

describe('Hamming Distance', () => {
  bench('rapid-fuzzy', () => {
    for (const [a, b] of equalLengthPairs) {
      hamming(a, b);
    }
  });
});
