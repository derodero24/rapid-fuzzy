import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const binding = require('./index.js');

export const {
  FuzzyIndex,
  KeyedFuzzyIndex,
  MatchType,
  closest,
  searchKeys,
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
  partialRatio,
  partialRatioBatch,
  partialRatioMany,
  search,
  sorensenDice,
  sorensenDiceBatch,
  sorensenDiceMany,
  tokenSetRatio,
  tokenSetRatioBatch,
  tokenSetRatioMany,
  tokenSortRatio,
  tokenSortRatioBatch,
  tokenSortRatioMany,
  weightedRatio,
  weightedRatioBatch,
  weightedRatioMany,
  highlight,
  highlightRanges,
} = { ...binding, ...require('./highlight.js') };

export const { searchObjects, FuzzyObjectIndex } = require('./objects.js');
