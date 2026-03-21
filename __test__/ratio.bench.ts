import { bench, describe } from 'vitest';
import { tokenSetRatio, tokenSortRatio, weightedRatio } from '../index.js';
import { pairs } from './bench-fixtures.js';

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
