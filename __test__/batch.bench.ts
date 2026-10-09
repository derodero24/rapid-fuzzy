import { test } from 'vitest';
import {
  levenshtein,
  levenshteinMany,
  normalizedLevenshtein,
  normalizedLevenshteinMany,
} from '../index.js';
import * as fixtures from './bench-fixtures.js';

// Vitest's module runner turns imported bindings into getters: copy the
// fixtures into local constants so the measured functions don't call a getter
// on every iteration.
const { manyCandidates } = fixtures;

test('Levenshtein Distance — Many (1K candidates)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (many)', () => {
      levenshteinMany('kitten', manyCandidates);
    }),
    bench('rapid-fuzzy (loop)', () => {
      for (const c of manyCandidates) {
        levenshtein('kitten', c);
      }
    }),
  );
});

test('Normalized Levenshtein — Many (1K candidates)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (many)', () => {
      normalizedLevenshteinMany('kitten', manyCandidates);
    }),
    bench('rapid-fuzzy (loop)', () => {
      for (const c of manyCandidates) {
        normalizedLevenshtein('kitten', c);
      }
    }),
  );
});
