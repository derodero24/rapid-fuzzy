#!/usr/bin/env node
/**
 * Load a release-built binding and exercise each family of exports once.
 *
 * The release workflow runs this on every target's own OS/architecture/libc
 * before anything is published, so a binary that cannot be loaded there
 * (wrong libc, missing DLL, unsupported instruction, missing N-API symbol) or
 * that returns garbage stops the release instead of reaching npm.
 *
 * Usage:
 *   node .github/scripts/smoke-test.cjs <binding>
 *
 * <binding> is a `rapid-fuzzy.<platform>.node` file or, for the WASI build,
 * `rapid-fuzzy.wasi.cjs` (run with NAPI_RS_FORCE_WASI=error so the loader
 * cannot fall back to a native binary). The file must sit in the repository
 * root, where the generated loader (index.js) looks for local bindings: the
 * script also loads index.js and checks that it resolved this exact binding.
 */

'use strict';

const assert = require('node:assert/strict');
const path = require('node:path');

const root = path.resolve(__dirname, '..', '..');
const arg = process.argv[2];
if (!arg) {
  process.stderr.write('usage: node .github/scripts/smoke-test.cjs <binding>\n');
  process.exit(2);
}
const bindingPath = path.resolve(arg);
assert.equal(
  path.dirname(bindingPath),
  root,
  `${arg} must be in the repository root so that index.js can find it`,
);

/** @param {typeof import('../../index.js')} b */
function exercise(b) {
  assert.equal(b.levenshtein('kitten', 'sitting'), 3);
  assert.equal(b.damerauLevenshtein('ca', 'ac'), 1);
  assert.equal(b.hamming('karolin', 'kathrin'), 3);
  assert.ok(Math.abs(b.jaroWinkler('martha', 'marhta') - 0.9611) < 1e-3);
  assert.equal(b.tokenSortRatio('fuzzy wuzzy', 'wuzzy fuzzy'), 1);
  assert.deepEqual(Array.from(b.levenshteinMany('kitten', ['sitting', 'kitten'])), [3, 0]);

  const items = ['handler', 'middleware', 'controller', 'café au lait'];
  const results = b.search('hndlr', items);
  assert.equal(results[0]?.item, 'handler');
  assert.equal(b.closest('cafe', items), 'café au lait');

  const index = new b.FuzzyIndex(items);
  assert.equal(index.search('mdlwr')[0]?.item, 'middleware');
}

const binding = require(bindingPath);
exercise(binding);

const loader = require(path.join(root, 'index.js'));
assert.equal(
  loader.FuzzyIndex,
  binding.FuzzyIndex,
  'index.js resolved a different binding than the one under test',
);
exercise(loader);

process.stdout.write(
  `smoke test passed: ${path.basename(bindingPath)} on ${process.platform}-${process.arch} (node ${process.version})\n`,
);
