#!/usr/bin/env node
/**
 * Build the wasm-bindgen WASM package and copy output files to the project root.
 *
 * The glue is generated with `--target web`: an ES module whose WebAssembly
 * instance is created explicitly (`init()` / `initSync()`) instead of through
 * the WebAssembly ESM integration that `--target bundler` relies on, which
 * Vite, esbuild, Bun, Deno and Cloudflare Workers do not support. The
 * entry points that instantiate it (browser.mjs, workerd.mjs) are generated
 * afterwards by scripts/build-browser.js.
 *
 * The package is "type": "commonjs", so the ES module glue is renamed to
 * `.mjs` (and its declarations to `.d.mts`); every tool then parses it as
 * ESM regardless of the package type.
 *
 * wasm-pack always outputs a package.json to the out-dir which would clobber the
 * project root package.json. This script works around that by using a temporary
 * directory and copying only the needed files.
 *
 * Usage: node scripts/build-wasm-bindgen.js
 */

'use strict';

const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { inputFieldsAcceptUndefined } = require('./declarations.js');

const ROOT = path.resolve(__dirname, '..');
const CRATE_DIR = path.join(ROOT, 'crates', 'wasm');
const OUT_NAME = 'rapid-fuzzy-wasm-bindgen';

// wasm-pack output file -> project root file.
const FILES_TO_COPY = {
  [`${OUT_NAME}.js`]: `${OUT_NAME}.mjs`,
  [`${OUT_NAME}.d.ts`]: `${OUT_NAME}.d.mts`,
  [`${OUT_NAME}_bg.wasm`]: `${OUT_NAME}_bg.wasm`,
};

// Output of the former `--target bundler` build; removed so a stale copy
// cannot be published or imported by mistake.
const OBSOLETE_FILES = [
  `${OUT_NAME}.js`,
  `${OUT_NAME}_bg.js`,
  `${OUT_NAME}.d.ts`,
  `${OUT_NAME}_bg.wasm.d.ts`,
];

/**
 * Post-process the glue:
 *
 * - Point the `@ts-self-types` pragma (read by Deno) at the renamed
 *   declarations.
 * - Fail if an export takes a `Vec<String>` (or another `Vec<JsValue>`-based)
 *   parameter again, which the glue passes with `passArrayJsValueToWasm0`:
 *   wasm-bindgen converts such an array inside the wasm call and throws from
 *   there on an element of the wrong type, which leaks the converted strings
 *   and the call's shadow-stack space and leaves an index permanently borrowed.
 *   `string[]` parameters are taken as `JsValue` and checked with
 *   `strings_from_js` instead (crates/wasm/src/convert.rs).
 */
function rewriteGlue(source) {
  const from = `/* @ts-self-types="./${OUT_NAME}.d.ts" */`;
  if (!source.startsWith(from)) {
    throw new Error(`build-wasm-bindgen: expected the glue to start with ${from}`);
  }
  if (source.includes('passArrayJsValueToWasm0')) {
    throw new Error(
      'build-wasm-bindgen: an export takes a Vec<String> / Vec<JsValue> parameter; take a JsValue and convert it with strings_from_js (crates/wasm/src/convert.rs)',
    );
  }
  return `/* @ts-self-types="./${OUT_NAME}.d.mts" */${source.slice(from.length)}`;
}

/**
 * Refine the declarations like scripts/patch-binding.js refines the Node.js
 * ones:
 *
 * - Array parameters accept readonly arrays: `string[]` parameters become
 *   `ReadonlyArray<string>` and `string[][]` ones
 *   `ReadonlyArray<ReadonlyArray<string>>`. The bindings never modify their
 *   arguments. Return types are left as generated.
 * - The optional fields of the options objects accept an explicit `undefined`
 *   (`field?: T | undefined`, see scripts/declarations.js): tsify declares an
 *   optional `Option<T>` field as `field?: T`.
 */
function rewriteDeclarations(dts) {
  const declaration = /^(\s*(?:export function \w+|(?:static )?\w+|constructor))\(/;
  const arraysRewritten = dts
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
          const params = line
            .slice(open, i)
            .replace(/\bstring\[\]\[\]/g, 'ReadonlyArray<ReadonlyArray<string>>')
            .replace(/\bstring\[\]/g, 'ReadonlyArray<string>');
          return line.slice(0, open) + params + line.slice(i);
        }
      }
      return line;
    })
    .join('\n');
  const rewritten = inputFieldsAcceptUndefined(arraysRewritten, `${OUT_NAME}.d.mts`);
  // Fail loudly if wasm-bindgen changes its output format and the rewrite stops applying.
  for (const expected of [
    'export function levenshteinBatch(pairs: ReadonlyArray<ReadonlyArray<string>>)',
    'export function search(query: string, items: ReadonlyArray<string>,',
    '    constructor(items: ReadonlyArray<string>);',
    '    maxResults?: number | undefined;',
    '    scoreMode?: KeyScoreMode | undefined;',
    '    matchMode?: KeyMatchMode | undefined;',
  ]) {
    if (!rewritten.includes(expected)) {
      throw new Error(`build-wasm-bindgen: expected \`${expected}\` in the declarations`);
    }
  }
  return rewritten;
}

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'wasm-pack-'));

try {
  console.log('Building wasm-bindgen package...');
  execFileSync(
    'wasm-pack',
    ['build', '--target', 'web', CRATE_DIR, '--out-dir', tmpDir, '--out-name', OUT_NAME],
    { stdio: 'inherit' },
  );

  console.log('Copying output files to project root...');
  for (const [file, target] of Object.entries(FILES_TO_COPY)) {
    const src = path.join(tmpDir, file);
    if (!fs.existsSync(src)) {
      throw new Error(`build-wasm-bindgen: ${file} not found in wasm-pack output`);
    }
    if (target.endsWith('.mjs')) {
      fs.writeFileSync(path.join(ROOT, target), rewriteGlue(fs.readFileSync(src, 'utf8')));
    } else if (target.endsWith('.d.mts')) {
      fs.writeFileSync(path.join(ROOT, target), rewriteDeclarations(fs.readFileSync(src, 'utf8')));
    } else {
      fs.copyFileSync(src, path.join(ROOT, target));
    }
    console.log(`  ${file} -> ${target}`);
  }
  for (const file of OBSOLETE_FILES) {
    fs.rmSync(path.join(ROOT, file), { force: true });
  }

  require('./build-browser.js').generate();

  console.log('wasm-bindgen build complete.');
} finally {
  fs.rmSync(tmpDir, { recursive: true, force: true });
}
