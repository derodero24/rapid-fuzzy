import { bench, describe } from 'vitest';
import { jaro, jaroWinkler, normalizedLevenshtein, sorensenDice } from '../index.js';
import { pairs } from './bench-fixtures.js';

describe('Normalized Similarity', () => {
  bench('rapid-fuzzy (normalizedLevenshtein)', () => {
    for (const [a, b] of pairs) {
      normalizedLevenshtein(a, b);
    }
  });

  bench('rapid-fuzzy (sorensenDice)', () => {
    for (const [a, b] of pairs) {
      sorensenDice(a, b);
    }
  });
});

describe('Jaro / Jaro-Winkler', () => {
  bench('rapid-fuzzy (jaro)', () => {
    for (const [a, b] of pairs) {
      jaro(a, b);
    }
  });

  bench('rapid-fuzzy (jaroWinkler)', () => {
    for (const [a, b] of pairs) {
      jaroWinkler(a, b);
    }
  });
});
