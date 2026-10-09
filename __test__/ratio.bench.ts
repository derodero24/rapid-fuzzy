import { test } from 'vitest';
import { tokenSetRatio, tokenSortRatio, weightedRatio } from '../index.js';
import * as fixtures from './bench-fixtures.js';

// Vitest's module runner turns imported bindings into getters: copy the
// fixtures into local constants so the measured functions don't call a getter
// on every iteration.
const { pairs } = fixtures;

test('Token-Based Ratio', async ({ bench }) => {
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
  );
});
