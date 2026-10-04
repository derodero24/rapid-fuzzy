#!/usr/bin/env node
/**
 * Generate THIRD_PARTY_NOTICES: the license notices of the third-party code
 * compiled into the binaries rapid-fuzzy ships.
 *
 * - The Rust crates linked into the N-API addon for every target in
 *   package.json `napi.targets` (including the WASI build) and into the
 *   wasm-bindgen browser build, taken from Cargo.lock with
 *   `cargo tree --edges normal,no-proc-macro`, so build scripts, proc macros
 *   and dev-dependencies (which never end up in a binary) are excluded.
 * - Their license texts, copied from the crate packages in the Cargo registry
 *   (or from scripts/third-party-licenses/<crate>/ for the few crates that
 *   publish none).
 * - The toolchain components linked in as well: the Rust standard library,
 *   wasi-libc and emnapi (WASI build only), with the license texts kept in
 *   scripts/third-party-licenses/toolchain/ and in the emnapi package.
 *
 * The release workflow copies the file into every npm package.
 *
 * Usage:
 *   cargo fetch --locked                          # crate sources, once
 *   node scripts/third-party-notices.js           # rewrite THIRD_PARTY_NOTICES
 *   node scripts/third-party-notices.js --check   # fail if it is out of date
 */

'use strict';

const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const OUTPUT = path.join(ROOT, 'THIRD_PARTY_NOTICES');
const FALLBACK_DIR = path.join(__dirname, 'third-party-licenses');
const LICENSE_FILE = /^(licen[cs]e|copying|copyright|notice|unlicense)/i;
const RULE = '-'.repeat(80);

/**
 * @typedef {object} CargoPackage
 * @property {string} name
 * @property {string} version
 * @property {string | null} license
 * @property {string | null} repository
 * @property {string} manifest_path
 */

/**
 * @typedef {object} Component
 * @property {string} title  Name and version.
 * @property {string} license  SPDX expression.
 * @property {string} url
 * @property {string[]} notes  Extra lines printed under the component.
 * @property {{ file: string, text: string }[]} texts
 */

/** @param {string[]} args */
function cargo(args) {
  return execFileSync('cargo', args, {
    cwd: ROOT,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'inherit'],
  });
}

/**
 * @param {string} a
 * @param {string} b
 */
function compare(a, b) {
  if (a === b) {
    return 0;
  }
  return a < b ? -1 : 1;
}

/** @param {string} text */
function normalize(text) {
  return `${text.replace(/\r\n?/g, '\n').replace(/\s+$/, '')}\n`;
}

/**
 * License files of a directory, sorted by name.
 * @param {string} dir
 * @param {(file: string) => boolean} accept
 */
function readTexts(dir, accept) {
  if (!fs.existsSync(dir)) {
    return [];
  }
  return fs
    .readdirSync(dir)
    .filter((file) => accept(file) && fs.statSync(path.join(dir, file)).isFile())
    .sort()
    .map((file) => ({ file, text: normalize(fs.readFileSync(path.join(dir, file), 'utf8')) }));
}

/** The `name@version` keys of every registry crate linked into a shipped build. */
function shippedCrates() {
  /** @type {{ napi: { targets: string[] } }} */
  const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf8'));
  const builds = [
    ...pkg.napi.targets.map((target) => ({ crate: 'rapid-fuzzy', target })),
    { crate: 'rapid-fuzzy-wasm', target: 'wasm32-unknown-unknown' },
  ];
  /** @type {Set<string>} */
  const keys = new Set();
  for (const { crate, target } of builds) {
    const tree = cargo([
      'tree',
      '--locked',
      '--package',
      crate,
      '--target',
      target,
      '--edges',
      'normal,no-proc-macro',
      '--prefix',
      'none',
      '--format',
      '{p}',
    ]);
    for (const line of tree.split('\n')) {
      const match = /^(\S+) v(\S+)(?: \((.*)\))?$/.exec(line.trim());
      // A parenthesised path marks a workspace crate (our own MIT code);
      // "(*)" only marks a repeated subtree.
      if (match?.[1] && match[2] && (match[3] === undefined || match[3] === '*')) {
        keys.add(`${match[1]}@${match[2]}`);
      }
    }
  }
  return keys;
}

/**
 * Merge the entries of a crate linked in more than one version when they are
 * identical, which they are unless its license changed between versions;
 * otherwise tell them apart by version.
 *
 * Entries carry no version so that the file, which CI checks, only changes
 * when a crate is added or removed or its license text changes, and not with
 * the version bumps of automated dependency updates.
 * @param {Component[]} components  One per package, in the order of `packages`.
 * @param {CargoPackage[]} packages
 * @returns {Component[]}
 */
function withoutDuplicates(components, packages) {
  /** @param {Component} c */
  const key = (c) => JSON.stringify([c.license, c.url, c.notes, c.texts]);
  return components.flatMap((c, i) => {
    const same = components.filter((other) => other.title === c.title);
    if (same.length === 1) {
      return [c];
    }
    if (same.every((other) => key(other) === key(c))) {
      return same[0] === c ? [c] : [];
    }
    return [{ ...c, title: `${c.title} ${packages[i]?.version}` }];
  });
}

/** @returns {Component[]} */
function crateComponents() {
  /** @type {{ packages: CargoPackage[] }} */
  const metadata = JSON.parse(cargo(['metadata', '--format-version', '1', '--locked']));
  const byKey = new Map(metadata.packages.map((p) => [`${p.name}@${p.version}`, p]));
  const packages = [...shippedCrates()].map((key) => {
    const p = byKey.get(key);
    if (!p) {
      throw new Error(`${key} is missing from cargo metadata`);
    }
    return p;
  });

  // Plain code-unit order: locale-aware sorting could differ between machines.
  const components = packages
    .sort((a, b) => compare(a.name, b.name) || compare(a.version, b.version))
    .map((p) => {
      const crateDir = path.dirname(p.manifest_path);
      let texts = readTexts(crateDir, (file) => LICENSE_FILE.test(file));
      /** @type {string[]} */
      const notes = [];
      if (texts.length === 0) {
        texts = readTexts(path.join(FALLBACK_DIR, p.name), (file) => file !== 'README.md');
        notes.push('The crate package ships no license file; texts taken from its repository.');
      }
      if (texts.length === 0) {
        throw new Error(
          `${p.name} ${p.version} ships no license file and ${path.relative(ROOT, FALLBACK_DIR)}/${p.name}/ does not exist (see its README.md)`,
        );
      }
      if (/\bMPL-2\.0\b/.test(p.license ?? '')) {
        notes.push(
          `MPL-2.0: rapid-fuzzy uses ${p.name} unmodified, in the version listed in the Cargo.lock`,
          'of the rapid-fuzzy release. Its Source Code Form is available at',
          `https://crates.io/crates/${p.name} and ${p.repository ?? 'its repository'}.`,
        );
      }
      return {
        // No version: the file would otherwise change with every routine
        // dependency bump (see withoutDuplicates).
        title: p.name,
        license: p.license ?? 'unknown',
        url: p.repository ?? `https://crates.io/crates/${p.name}`,
        notes,
        texts,
      };
    });
  return withoutDuplicates(components, packages);
}

/**
 * License texts of a toolchain component, kept in
 * scripts/third-party-licenses/toolchain/<name>/ (see its README.md).
 * @param {string} name
 */
function toolchainTexts(name) {
  const texts = readTexts(
    path.join(FALLBACK_DIR, 'toolchain', name),
    (file) => file !== 'README.md',
  );
  if (texts.length === 0) {
    throw new Error(`scripts/third-party-licenses/toolchain/${name}/ holds no license texts`);
  }
  return texts;
}

/** @returns {Component[]} */
function toolchainComponents() {
  const emnapiDir = path.join(ROOT, 'node_modules', 'emnapi');
  if (!fs.existsSync(path.join(emnapiDir, 'package.json'))) {
    throw new Error('node_modules/emnapi is missing; run `pnpm install` first');
  }
  /** @type {{ license: string }} */
  const emnapi = JSON.parse(fs.readFileSync(path.join(emnapiDir, 'package.json'), 'utf8'));
  return [
    {
      title: 'Rust standard library',
      license: 'MIT OR Apache-2.0',
      url: 'https://github.com/rust-lang/rust',
      notes: [
        'core, alloc and std, and the crates they bundle, are compiled into every binary.',
        'The notices of the crates the standard library depends on are listed in',
        'share/doc/rust/COPYRIGHT-library.html of every Rust toolchain.',
      ],
      texts: toolchainTexts('rust-std'),
    },
    {
      title: 'wasi-libc',
      license: 'Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT',
      url: 'https://github.com/WebAssembly/wasi-libc',
      notes: [
        'WASI build only: linked by the Rust wasm32-wasip1-threads target. Portions are',
        'derived from dlmalloc (CC0), emmalloc (MIT), cloudlibc (BSD-2-Clause), musl (MIT)',
        'and musl-fts (BSD-3-Clause), whose notices are included below.',
      ],
      texts: toolchainTexts('wasi-libc'),
    },
    {
      title: 'emnapi',
      license: emnapi.license,
      url: 'https://github.com/toyobayashi/emnapi',
      notes: ['WASI build only: its C library implements Node-API inside the WebAssembly module.'],
      texts: readTexts(emnapiDir, (file) => LICENSE_FILE.test(file)),
    },
  ];
}

function render() {
  const crates = crateComponents();
  const toolchain = toolchainComponents();

  // License texts are printed once and referenced by number: most crates
  // share the same Apache-2.0 text.
  /** @type {string[]} */
  const uniqueTexts = [];
  /** @param {Component} c */
  const describe = (c) => {
    const lines = [`${c.title} (${c.license})`, `  ${c.url}`];
    for (const note of c.notes) {
      lines.push(`  ${note}`);
    }
    if (c.texts.length > 0) {
      const refs = c.texts.map(({ file, text }) => {
        let index = uniqueTexts.indexOf(text);
        if (index === -1) {
          index = uniqueTexts.push(text) - 1;
        }
        return `[${index + 1}] ${file}`;
      });
      lines.push(`  License texts: ${refs.join(', ')}`);
    }
    return lines.join('\n');
  };

  const sections = [
    'THIRD-PARTY SOFTWARE NOTICES',
    '============================',
    '',
    'rapid-fuzzy is licensed under the MIT License (see LICENSE). Its compiled',
    'binaries - the Node.js addons in the rapid-fuzzy-<platform> packages, the',
    'WebAssembly module in rapid-fuzzy-wasm32-wasi and rapid-fuzzy-wasm-bindgen_bg.wasm',
    'in rapid-fuzzy - include the third-party components below, which are',
    'distributed under the licenses reproduced at the end of this file.',
    '',
    'Generated by scripts/third-party-notices.js from Cargo.lock and the installed',
    'emnapi package; do not edit by hand.',
    '',
    'Rust crates',
    '-----------',
    '',
    crates.map(describe).join('\n\n'),
    '',
    'Toolchain components',
    '--------------------',
    '',
    toolchain.map(describe).join('\n\n'),
    '',
    'License texts',
    '-------------',
    '',
    uniqueTexts.map((text, i) => `[${i + 1}]\n${RULE}\n${text}${RULE}`).join('\n\n'),
  ];
  return `${sections.join('\n')}\n`;
}

const generated = render();
if (process.argv.includes('--check')) {
  // A Windows checkout may have converted the file to CRLF.
  const current = fs.existsSync(OUTPUT)
    ? fs.readFileSync(OUTPUT, 'utf8').replace(/\r\n/g, '\n')
    : '';
  if (current !== generated) {
    process.stderr.write(
      'THIRD_PARTY_NOTICES is out of date; run `node scripts/third-party-notices.js`.\n',
    );
    process.exit(1);
  }
  process.stdout.write('THIRD_PARTY_NOTICES is up to date.\n');
} else {
  fs.writeFileSync(OUTPUT, generated);
  process.stdout.write(`Wrote ${path.relative(ROOT, OUTPUT)}.\n`);
}
