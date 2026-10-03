export * from './rapid-fuzzy-wasm-bindgen.js';

// --- JS utilities (appended by scripts/patch-binding.js) ---
import {
  damerauLevenshteinMany,
  hammingMany,
  indelMany,
  jaroMany,
  jaroWinklerMany,
  levenshteinMany,
  normalizedHammingMany,
  normalizedIndelMany,
  normalizedLevenshteinMany,
  partialRatioMany,
  sorensenDiceMany,
  tokenSetRatioMany,
  tokenSortRatioMany,
  weightedRatioMany,
} from './rapid-fuzzy-wasm-bindgen.js';
export { highlight, highlightRanges } from './highlight.mjs';
// Runtime values of the MatchType string enum (the Node.js binding exports the same object).
export const MatchType = Object.freeze({ Exact: 'Exact', Prefix: 'Prefix', Contains: 'Contains', Fuzzy: 'Fuzzy' });
// TypedArray variants: the wasm-bindgen *Many functions already return typed arrays.
export const levenshteinManyU32 = levenshteinMany;
export const damerauLevenshteinManyU32 = damerauLevenshteinMany;
export const indelManyU32 = indelMany;
export const jaroManyF64 = jaroMany;
export const jaroWinklerManyF64 = jaroWinklerMany;
export const sorensenDiceManyF64 = sorensenDiceMany;
export const normalizedLevenshteinManyF64 = normalizedLevenshteinMany;
export const normalizedIndelManyF64 = normalizedIndelMany;
export const tokenSortRatioManyF64 = tokenSortRatioMany;
export const tokenSetRatioManyF64 = tokenSetRatioMany;
export const partialRatioManyF64 = partialRatioMany;
export const weightedRatioManyF64 = weightedRatioMany;
// Same sentinels as index.js: 0xffffffff / NaN where hammingMany returns null.
export const hammingManyU32 = (r, c, d) => Uint32Array.from(hammingMany(r, c, d), (v) => (v == null ? 0xffffffff : v));
export const normalizedHammingManyF64 = (r, c, s) => Float64Array.from(normalizedHammingMany(r, c, s), (v) => (v == null ? Number.NaN : v));
