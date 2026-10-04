#!/usr/bin/env node
/**
 * Patch napi-rs auto-generated files to include JS-only utilities.
 *
 * Run automatically after `napi build` via the `build` npm script.
 * Appends the highlight exports to index.js, index.d.ts, and browser.js.
 * (The `*ManyU32` / `*ManyF64` typed-array variants are native exports.)
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
  ].join('\n'),
);

// --- index.d.ts ---
patchFile(
  'index.d.ts',
  "export { highlight, highlightRanges, HighlightRange } from './highlight';",
);

// --- browser.js (ESM) ---
// Overwrite entirely: use wasm-bindgen output instead of the napi-rs WASI package.
// The JS-only exports below give the browser entry the same runtime surface as
// index.js, whose declarations (index.d.ts) browser users are typed against.
fs.writeFileSync(
  'browser.js',
  [
    "export * from './rapid-fuzzy-wasm-bindgen.js';",
    '',
    `${MARKER}`,
    'import {',
    '  damerauLevenshteinMany,',
    '  hammingMany,',
    '  indelMany,',
    '  jaroMany,',
    '  jaroWinklerMany,',
    '  levenshteinMany,',
    '  normalizedHammingMany,',
    '  normalizedIndelMany,',
    '  normalizedLevenshteinMany,',
    '  partialRatioMany,',
    '  sorensenDiceMany,',
    '  tokenSetRatioMany,',
    '  tokenSortRatioMany,',
    '  weightedRatioMany,',
    "} from './rapid-fuzzy-wasm-bindgen.js';",
    "export { highlight, highlightRanges } from './highlight.mjs';",
    '// Runtime values of the MatchType string enum (the Node.js binding exports the same object).',
    "export const MatchType = Object.freeze({ Exact: 'Exact', Prefix: 'Prefix', Contains: 'Contains', Fuzzy: 'Fuzzy' });",
    '// TypedArray variants: the wasm-bindgen *Many functions already return typed arrays.',
    'export const levenshteinManyU32 = levenshteinMany;',
    'export const damerauLevenshteinManyU32 = damerauLevenshteinMany;',
    'export const indelManyU32 = indelMany;',
    'export const jaroManyF64 = jaroMany;',
    'export const jaroWinklerManyF64 = jaroWinklerMany;',
    'export const sorensenDiceManyF64 = sorensenDiceMany;',
    'export const normalizedLevenshteinManyF64 = normalizedLevenshteinMany;',
    'export const normalizedIndelManyF64 = normalizedIndelMany;',
    'export const tokenSortRatioManyF64 = tokenSortRatioMany;',
    'export const tokenSetRatioManyF64 = tokenSetRatioMany;',
    'export const partialRatioManyF64 = partialRatioMany;',
    'export const weightedRatioManyF64 = weightedRatioMany;',
    '// Same sentinels as index.js: 0xffffffff / NaN where hammingMany returns null.',
    'export const hammingManyU32 = (r, c, d) => Uint32Array.from(hammingMany(r, c, d), (v) => (v == null ? 0xffffffff : v));',
    'export const normalizedHammingManyF64 = (r, c, s) => Float64Array.from(normalizedHammingMany(r, c, s), (v) => (v == null ? Number.NaN : v));',
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
