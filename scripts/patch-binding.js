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

// --- Object search wrappers and declaration refinements (index.js, index.d.ts) ---
// Appended after the JS utilities above, in a section of its own.

const WRAPPERS_MARKER = '// --- JS wrappers (appended by scripts/patch-binding.js) ---';

function appendSection(path, section, transform = (content) => content) {
  let content = fs.readFileSync(path, 'utf-8');
  const markerIdx = content.indexOf(WRAPPERS_MARKER);
  if (markerIdx !== -1) {
    content = `${content.slice(0, markerIdx).trimEnd()}\n`;
  }
  fs.writeFileSync(path, transform(`${content}\n${WRAPPERS_MARKER}\n${section}\n`));
  console.log(`  patched ${path} (JS wrappers)`);
}

/**
 * Widen array parameters in the declarations to `ReadonlyArray`: the native
 * functions only read their inputs, so readonly arrays are valid arguments.
 * Return types are left as mutable arrays. Idempotent.
 */
function readonlyArrayParams(dts) {
  const declaration = /^(\s*(?:export declare function \w+|(?:static )?\w+|constructor))\(/;
  return dts
    .split('\n')
    .map((line) => {
      const match = declaration.exec(line);
      if (!match) return line;
      // Find the parenthesis closing the parameter list.
      const open = match[1].length;
      let depth = 0;
      for (let i = open; i < line.length; i++) {
        if (line[i] === '(') depth++;
        else if (line[i] === ')' && --depth === 0) {
          const params = line.slice(open, i).replace(/\bArray</g, 'ReadonlyArray<');
          return line.slice(0, open) + params + line.slice(i);
        }
      }
      return line;
    })
    .join('\n');
}

/**
 * Let the optional fields of an input interface accept an explicit `undefined`
 * (`field?: T | undefined`), as required under `exactOptionalPropertyTypes`.
 * The native conversion treats `undefined` as "not set". Idempotent.
 */
function optionalFieldsAcceptUndefined(dts, name) {
  const start = dts.indexOf(`export interface ${name} {`);
  const end = start === -1 ? -1 : dts.indexOf('\n}', start);
  if (end === -1) {
    throw new Error(`patch-binding: interface ${name} not found in index.d.ts`);
  }
  const body = dts
    .slice(start, end)
    .replace(/^(\s+\w+\?: )(.+)$/gm, (field, head, type) =>
      /\bundefined\b/.test(type) ? field : `${head}${type} | undefined`,
    );
  return dts.slice(0, start) + body + dts.slice(end);
}

function refineDeclarations(dts) {
  const refined = optionalFieldsAcceptUndefined(readonlyArrayParams(dts), 'SearchOptions');
  // Fail loudly if napi-rs changes its output format and the refinements stop applying.
  for (const expected of [
    'export declare function search(query: string, items: ReadonlyArray<string>,',
    '  constructor(items: ReadonlyArray<string>)',
    '  maxResults?: number | undefined',
  ]) {
    if (!refined.includes(expected)) {
      throw new Error(`patch-binding: expected \`${expected}\` in index.d.ts`);
    }
  }
  return refined;
}

appendSection(
  'index.js',
  [
    '// searchObjects / FuzzyObjectIndex live in objects.js, which requires this module:',
    '// resolve them on first access to avoid a require cycle. Assigning replaces the getter.',
    "for (const name of ['searchObjects', 'FuzzyObjectIndex']) {",
    '  Object.defineProperty(module.exports, name, {',
    '    enumerable: true,',
    '    configurable: true,',
    "    get: () => require('./objects.js')[name],",
    '    set: (value) => {',
    '      Object.defineProperty(module.exports, name, { value, writable: true, enumerable: true, configurable: true });',
    '    },',
    '  });',
    '}',
  ].join('\n'),
);

appendSection(
  'index.d.ts',
  [
    "export { FuzzyObjectIndex, searchObjects } from './objects';",
    "export type { KeyConfig, KeyPath, ObjectIndexOptions, ObjectIndexSearchOptions, ObjectSearchOptions, ObjectSearchResult } from './objects';",
    "export type { HighlightOptions } from './highlight';",
  ].join('\n'),
  refineDeclarations,
);

console.log('Done — JS utilities patched into binding files.');
