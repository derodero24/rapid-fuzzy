#!/usr/bin/env node
/**
 * Patch napi-rs auto-generated files to include JS-only utilities.
 *
 * Run automatically after `napi build` via the `build` npm script.
 * Appends the highlight exports to index.js and index.d.ts, and regenerates
 * the browser / edge entry points (scripts/build-browser.js).
 * (The `*ManyU32` / `*ManyF64` typed-array variants are native exports.)
 */

'use strict';

const fs = require('node:fs');
const { inputFieldsAcceptUndefined } = require('./declarations.js');

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

// --- Browser / edge entry points (browser.mjs, workerd.mjs, ...) ---
// Generated from the wasm-bindgen glue and the JS wrappers (highlight.js,
// objects.js), so they are refreshed whenever those change.
require('./build-browser.js').generate();

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
 * The string-literal types of the `scoreMode` and `matchMode` fields of
 * `KeySearchOptions` and `KeyClosestOptions`, which the Rust side types as
 * `KeyScoreMode` / `KeyMatchMode` (`#[napi(ts_type)]`): napi-rs declares
 * string enums as TS enums, which do not accept string literals. Keep in sync
 * with `KeyScoreMode::from_name` and `KeyMatchMode::from_name` in
 * crates/core-lib/src/search/keys.rs.
 */
const KEY_MODE_TYPES = [
  [
    'KeyScoreMode',
    [
      '/**',
      ' * How multi-key search (`searchKeys()`, `KeyedFuzzyIndex`, `searchObjects()`,',
      ' * `FuzzyObjectIndex`) combines the per-key scores of an item into its score:',
      " * `'weighted'` (the default), `'matched'` or `'max'`. See",
      ' * `KeySearchOptions.scoreMode`.',
      ' */',
      "export type KeyScoreMode = 'weighted' | 'matched' | 'max'",
    ],
  ],
  [
    'KeyMatchMode',
    [
      '/**',
      ' * How multi-key search (`searchKeys()`, `KeyedFuzzyIndex`, `searchObjects()`,',
      ' * `FuzzyObjectIndex`) matches the query against the keys of an item:',
      " * `'perKey'` (the default: every key against the whole query) or",
      " * `'crossKey'` (every term against the keys on its own). See",
      ' * `KeySearchOptions.matchMode`.',
      ' */',
      "export type KeyMatchMode = 'perKey' | 'crossKey'",
    ],
  ],
];

/** Declare the key mode types after the `KeySearchOptions` interface. Idempotent. */
function declareKeyModeTypes(dts) {
  let declared = dts;
  for (const [name, lines] of KEY_MODE_TYPES) {
    if (declared.includes(`export type ${name} =`)) continue;
    const start = declared.indexOf('export interface KeySearchOptions {');
    const end = start === -1 ? -1 : declared.indexOf('\n}\n', start);
    if (end === -1) {
      throw new Error('patch-binding: interface KeySearchOptions not found in index.d.ts');
    }
    const at = end + '\n}\n'.length;
    declared = `${declared.slice(0, at)}\n${lines.join('\n')}\n${declared.slice(at)}`;
  }
  return declared;
}

function refineDeclarations(dts) {
  let refined = readonlyArrayParams(dts);
  refined = inputFieldsAcceptUndefined(refined, 'index.d.ts');
  refined = declareKeyModeTypes(refined);
  // Fail loudly if napi-rs changes its output format and the refinements stop applying.
  for (const expected of [
    'export declare function search(query: string, items: ReadonlyArray<string>,',
    '  constructor(items: ReadonlyArray<string>)',
    '  maxResults?: number | undefined',
    '  scoreMode?: KeyScoreMode | undefined',
    '  matchMode?: KeyMatchMode | undefined',
    'closest(query: string, options?: number | KeyClosestOptions | undefined | null): number | null',
    "export type KeyScoreMode = 'weighted' | 'matched' | 'max'",
    "export type KeyMatchMode = 'perKey' | 'crossKey'",
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
