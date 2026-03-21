import { bench, describe } from 'vitest';
import { tokenSetRatio, tokenSortRatio, weightedRatio } from '../index.js';

// Test data — realistic string pairs of varying length and similarity
const pairs: [string, string][] = [
  ['kitten', 'sitting'],
  ['saturday', 'sunday'],
  ['rosettacode', 'raisethysword'],
  ['pneumonoultramicroscopicsilicovolcanoconiosis', 'ultramicroscopically'],
  ['the quick brown fox jumps over the lazy dog', 'the fast brown fox leaps over the lazy dog'],
  ['abcdefghijklmnopqrstuvwxyz', 'zyxwvutsrqponmlkjihgfedcba'],
];

describe('Token-Based Ratio', () => {
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
});
