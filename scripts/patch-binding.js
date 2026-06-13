#!/usr/bin/env node
/**
 * Patch napi-rs auto-generated files to include JS-only utilities.
 *
 * Run automatically after `napi build` via the `build` npm script.
 * Appends highlight and typed-array exports to index.js, index.d.ts, and browser.js.
 */

'use strict';

const fs = require('node:fs');

const MARKER = '// --- JS utilities (appended by scripts/patch-binding.js) ---';

function patchFile(path, patch) {
  let content = fs.readFileSync(path, 'utf-8');
  // Remove previous patch if present
  const markerIdx = content.indexOf(MARKER);
  if (markerIdx !== -1) {
    content = `${content.slice(0, markerIdx).trimEnd()}\n`;
  }
  fs.writeFileSync(path, `${content}\n${MARKER}\n${patch}\n`);
  console.log(`  patched ${path}`);
}

// --- index.js (CJS) ---
patchFile(
  'index.js',
  [
    "const _hl = require('./highlight.js');",
    'module.exports.highlight = _hl.highlight;',
    'module.exports.highlightRanges = _hl.highlightRanges;',
    '// TypedArray variants — return Uint32Array / Float64Array instead of Array<number>',
    'module.exports.levenshteinManyU32 = (r, c, d) => new Uint32Array(nativeBinding.levenshteinMany(r, c, d));',
    'module.exports.damerauLevenshteinManyU32 = (r, c, d) => new Uint32Array(nativeBinding.damerauLevenshteinMany(r, c, d));',
    'module.exports.indelManyU32 = (r, c, d) => new Uint32Array(nativeBinding.indelMany(r, c, d));',
    'module.exports.jaroManyF64 = (r, c, s) => new Float64Array(nativeBinding.jaroMany(r, c, s));',
    'module.exports.jaroWinklerManyF64 = (r, c, s) => new Float64Array(nativeBinding.jaroWinklerMany(r, c, s));',
    'module.exports.sorensenDiceManyF64 = (r, c, s) => new Float64Array(nativeBinding.sorensenDiceMany(r, c, s));',
    'module.exports.normalizedLevenshteinManyF64 = (r, c, s) => new Float64Array(nativeBinding.normalizedLevenshteinMany(r, c, s));',
    'module.exports.normalizedIndelManyF64 = (r, c, s) => new Float64Array(nativeBinding.normalizedIndelMany(r, c, s));',
    'module.exports.tokenSortRatioManyF64 = (r, c, s) => new Float64Array(nativeBinding.tokenSortRatioMany(r, c, s));',
    'module.exports.tokenSetRatioManyF64 = (r, c, s) => new Float64Array(nativeBinding.tokenSetRatioMany(r, c, s));',
    'module.exports.partialRatioManyF64 = (r, c, s) => new Float64Array(nativeBinding.partialRatioMany(r, c, s));',
    'module.exports.weightedRatioManyF64 = (r, c, s) => new Float64Array(nativeBinding.weightedRatioMany(r, c, s));',
    '// hamming variants return null for length mismatches or filtered candidates;',
    '// the TypedArray cannot hold null, so those slots use a sentinel:',
    '//   hammingManyU32 -> 0xffffffff (4294967295), normalizedHammingManyF64 -> NaN.',
    'module.exports.hammingManyU32 = (r, c, d) => Uint32Array.from(nativeBinding.hammingMany(r, c, d), (v) => (v == null ? 0xffffffff : v));',
    'module.exports.normalizedHammingManyF64 = (r, c, s) => Float64Array.from(nativeBinding.normalizedHammingMany(r, c, s), (v) => (v == null ? Number.NaN : v));',
  ].join('\n'),
);

// --- index.d.ts ---
patchFile(
  'index.d.ts',
  [
    "export { highlight, highlightRanges, HighlightRange } from './highlight';",
    '/** TypedArray variants — identical to the `*Many` counterparts but return a typed array instead of `Array<number>`, reducing GC pressure for large candidate sets. */',
    'export declare function levenshteinManyU32(reference: string, candidates: Array<string>, maxDistance?: number | undefined | null): Uint32Array;',
    'export declare function damerauLevenshteinManyU32(reference: string, candidates: Array<string>, maxDistance?: number | undefined | null): Uint32Array;',
    'export declare function indelManyU32(reference: string, candidates: Array<string>, maxDistance?: number | undefined | null): Uint32Array;',
    'export declare function jaroManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function jaroWinklerManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function sorensenDiceManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function normalizedLevenshteinManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function normalizedIndelManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function tokenSortRatioManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function tokenSetRatioManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function partialRatioManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    'export declare function weightedRatioManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
    '/**',
    ' * TypedArray variant of `hammingMany`. Slots that `hammingMany` returns as `null`',
    ' * (length mismatch, or filtered out by `maxDistance`) become the sentinel `0xffffffff`',
    ' * (4294967295), since a Uint32Array cannot hold `null`. Check for it with',
    ' * `value === 0xffffffff` before treating a slot as a real distance.',
    ' */',
    'export declare function hammingManyU32(reference: string, candidates: Array<string>, maxDistance?: number | undefined | null): Uint32Array;',
    '/**',
    ' * TypedArray variant of `normalizedHammingMany`. Slots that `normalizedHammingMany`',
    ' * returns as `null` (length mismatch, or filtered out by `minSimilarity`) become `NaN`,',
    ' * since a Float64Array cannot hold `null`. Check for it with `Number.isNaN(value)`.',
    ' */',
    'export declare function normalizedHammingManyF64(reference: string, candidates: Array<string>, minSimilarity?: number | undefined | null): Float64Array;',
  ].join('\n'),
);

// --- browser.js (ESM) ---
// Overwrite entirely: use wasm-bindgen output instead of the napi-rs WASI package.
fs.writeFileSync(
  'browser.js',
  [
    "export * from './rapid-fuzzy-wasm-bindgen.js';",
    '',
    `${MARKER}`,
    "export { highlight, highlightRanges } from './highlight.mjs';",
    '',
  ].join('\n'),
);
console.log('  patched browser.js');

console.log('Done — JS utilities patched into binding files.');
