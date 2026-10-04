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
