import { bench, describe } from 'vitest';
import { jaro, jaroWinkler, normalizedLevenshtein, sorensenDice } from '../index.js';

// Test data — realistic string pairs of varying length and similarity
const pairs: [string, string][] = [
  ['kitten', 'sitting'],
  ['saturday', 'sunday'],
  ['rosettacode', 'raisethysword'],
  ['pneumonoultramicroscopicsilicovolcanoconiosis', 'ultramicroscopically'],
  ['the quick brown fox jumps over the lazy dog', 'the fast brown fox leaps over the lazy dog'],
  ['abcdefghijklmnopqrstuvwxyz', 'zyxwvutsrqponmlkjihgfedcba'],
];

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
