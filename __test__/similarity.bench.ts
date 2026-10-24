import { test } from 'vitest';
import { jaro, jaroWinkler, normalizedLevenshtein, sorensenDice } from '../index.js';
import * as fixtures from './bench-fixtures.js';

// Vitest's module runner turns imported bindings into getters: copy the
// fixtures into local constants so the measured functions don't call a getter
// on every iteration.
const { pairs } = fixtures;

test('Normalized Similarity', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (normalizedLevenshtein)', () => {
      for (const [a, b] of pairs) {
        normalizedLevenshtein(a, b);
      }
    }),
    bench('rapid-fuzzy (sorensenDice)', () => {
      for (const [a, b] of pairs) {
        sorensenDice(a, b);
      }
    }),
  );
});

test('Jaro / Jaro-Winkler', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (jaro)', () => {
      for (const [a, b] of pairs) {
        jaro(a, b);
      }
    }),
    bench('rapid-fuzzy (jaroWinkler)', () => {
      for (const [a, b] of pairs) {
        jaroWinkler(a, b);
      }
    }),
  );
});
