import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const binding = require('./index.js');

export const {
  closest,
  damerauLevenshtein,
  damerauLevenshteinBatch,
  damerauLevenshteinMany,
  jaro,
  jaroBatch,
  jaroMany,
  jaroWinkler,
  jaroWinklerBatch,
  jaroWinklerMany,
  levenshtein,
  levenshteinBatch,
  levenshteinMany,
  normalizedLevenshtein,
  normalizedLevenshteinBatch,
  normalizedLevenshteinMany,
  search,
  sorensenDice,
  sorensenDiceBatch,
  sorensenDiceMany,
} = binding;
