import { bench, describe } from 'vitest';
import { damerauLevenshtein, hamming, levenshtein, levenshteinBatch } from '../index.js';

// Test data — realistic string pairs of varying length and similarity
const pairs: [string, string][] = [
  ['kitten', 'sitting'],
  ['saturday', 'sunday'],
  ['rosettacode', 'raisethysword'],
  ['pneumonoultramicroscopicsilicovolcanoconiosis', 'ultramicroscopically'],
  ['the quick brown fox jumps over the lazy dog', 'the fast brown fox leaps over the lazy dog'],
  ['abcdefghijklmnopqrstuvwxyz', 'zyxwvutsrqponmlkjihgfedcba'],
];

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

// Equal-length pairs for Hamming distance
const equalLengthPairs: [string, string][] = [
  ['karolin', 'kathrin'],
  ['saturday', 'sunturdy'],
  ['abcdefgh', 'abcdefgz'],
  ['10101010', '01010101'],
  ['the quick brown fox jumps', 'the swift brown fox leaps'],
  ['abcdefghijklmnopqrstuvwxyz', 'zyxwvutsrqponmlkjihgfedcba'],
];

describe('Hamming Distance', () => {
  bench('rapid-fuzzy', () => {
    for (const [a, b] of equalLengthPairs) {
      hamming(a, b);
    }
  });
});
