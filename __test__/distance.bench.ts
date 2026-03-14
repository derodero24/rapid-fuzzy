// Competitors
import { distance as fastestLevenshteinDistance } from 'fastest-levenshtein';
import leven from 'leven';
import stringSimilarity from 'string-similarity';
import { bench, describe } from 'vitest';
import {
  damerauLevenshtein,
  jaro,
  jaroWinkler,
  levenshtein,
  levenshteinBatch,
  normalizedLevenshtein,
  sorensenDice,
} from '../index.js';

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

  bench('fastest-levenshtein', () => {
    for (const [a, b] of pairs) {
      fastestLevenshteinDistance(a, b);
    }
  });

  bench('leven', () => {
    for (const [a, b] of pairs) {
      leven(a, b);
    }
  });
});

describe('Normalized Similarity', () => {
  bench('rapid-fuzzy (normalizedLevenshtein)', () => {
    for (const [a, b] of pairs) {
      normalizedLevenshtein(a, b);
    }
  });

  bench('string-similarity (compareTwoStrings / Dice)', () => {
    for (const [a, b] of pairs) {
      stringSimilarity.compareTwoStrings(a, b);
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

describe('Damerau-Levenshtein', () => {
  bench('rapid-fuzzy', () => {
    for (const [a, b] of pairs) {
      damerauLevenshtein(a, b);
    }
  });
});
