import { test } from 'vitest';
import { damerauLevenshtein, hamming, levenshtein, levenshteinBatch } from '../index.js';
import * as fixtures from './bench-fixtures.js';

// Vitest's module runner turns imported bindings into getters: copy the
// fixtures into local constants so the measured functions don't call a getter
// on every iteration.
const { equalLengthPairs, pairs } = fixtures;

test('Levenshtein Distance', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      for (const [a, b] of pairs) {
        levenshtein(a, b);
      }
    }),
    bench('rapid-fuzzy (batch)', () => {
      levenshteinBatch(pairs);
    }),
  );
});

test('Damerau-Levenshtein', async ({ bench }) => {
  await bench('rapid-fuzzy', () => {
    for (const [a, b] of pairs) {
      damerauLevenshtein(a, b);
    }
  }).run();
});

test('Hamming Distance', async ({ bench }) => {
  await bench('rapid-fuzzy', () => {
    for (const [a, b] of equalLengthPairs) {
      hamming(a, b);
    }
  }).run();
});
