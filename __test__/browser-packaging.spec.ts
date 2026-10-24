// Packaging of the browser / edge build: package.json export conditions, the
// generated entry points (scripts/build-browser.js) and their runtime behaviour.
//
// Runtime tests load browser.mjs, which instantiates the compiled
// `rapid-fuzzy-wasm-bindgen_bg.wasm` (a build artifact of
// `pnpm run build:wasm-bindgen`). Without it they are skipped, except where
// RAPID_FUZZY_REQUIRE_WASM_BINDGEN is set (CI).
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, normalize } from 'node:path';
import { describe, expect, it } from 'vitest';
import * as highlightCjs from '../highlight.js';
import * as napi from '../index.js';
import {
  FuzzyObjectIndex as NodeFuzzyObjectIndex,
  type ObjectIndexSearchOptions,
  searchObjects,
} from '../objects.js';

const ROOT = join(__dirname, '..');
const WASM_PATH = join(ROOT, 'rapid-fuzzy-wasm-bindgen_bg.wasm');
const wasmAvailable = existsSync(WASM_PATH);
if (!wasmAvailable && process.env.RAPID_FUZZY_REQUIRE_WASM_BINDGEN) {
  throw new Error(`wasm-bindgen binary is required but missing: ${WASM_PATH}`);
}

/** An export target: a path, or conditions mapping to nested targets. */
type Target = string | { [condition: string]: Target };
type Conditions = { [condition: string]: Target };
interface Manifest {
  type: string;
  main: string;
  module?: string;
  browser: string;
  types: string;
  sideEffects: string[];
  files: string[];
  exports: Record<string, string | Conditions>;
}
const pkg = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8')) as Manifest;

/** Every subpath export with its conditions. */
function subpaths(): Array<[string, Conditions]> {
  return Object.entries(pkg.exports).flatMap(([subpath, target]) =>
    typeof target === 'string' ? [] : [[subpath, target] as [string, Conditions]],
  );
}

/** The innermost condition objects of a target (`{ types, default }` pairs) with their paths. */
function leaves(target: Target, path: string[] = []): Array<[string[], Conditions]> {
  if (typeof target === 'string') return [];
  const nested = Object.entries(target).filter(([, value]) => typeof value === 'object');
  return nested.length === 0
    ? [[path, target]]
    : nested.flatMap(([condition, value]) => leaves(value, [...path, condition]));
}

/** Relative static import / re-export specifiers of an ES module. */
function staticImports(file: string): string[] {
  const source = readFileSync(join(ROOT, file), 'utf8');
  const specifiers = [...source.matchAll(/^(?:import|export)\b[^;'"]*?(?:from\s*)?'([^']+)';/gm)];
  return specifiers.map((m) => m[1] ?? '');
}

/** Files reachable from `entry` through static imports (the module graph). */
function moduleGraph(entry: string): string[] {
  const seen = new Set<string>();
  const visit = (file: string): void => {
    if (seen.has(file)) return;
    seen.add(file);
    if (!file.endsWith('.mjs')) return;
    for (const specifier of staticImports(file)) {
      expect(specifier, `${file} imports ${specifier}`).toMatch(/^\.\//);
      visit(normalize(join(dirname(file), specifier.replace(/\?module$/, ''))));
    }
  };
  visit(entry);
  return [...seen];
}

/**
 * Resolve `specifier` from inside the package with Node.js, extra conditions and
 * either `import` (ES module) or `require` (CommonJS) resolution.
 */
function nodeResolve(specifier: string, conditions: string[], mode: 'import' | 'require'): string {
  const flags = conditions.map((c) => `--conditions=${c}`);
  const resolve =
    mode === 'import'
      ? `import.meta.resolve(${JSON.stringify(specifier)})`
      : `require.resolve(${JSON.stringify(specifier)})`;
  const resolved = execFileSync(
    process.execPath,
    [
      ...flags,
      `--input-type=${mode === 'import' ? 'module' : 'commonjs'}`,
      '-e',
      `process.stdout.write(${resolve})`,
    ],
    { cwd: ROOT, encoding: 'utf8' },
  );
  // require.resolve returns a native path (backslashes on Windows);
  // import.meta.resolve returns a file: URL.
  return resolved.split(/[\\/]/).at(-1) ?? resolved;
}

describe('package.json', () => {
  it('lists the export conditions most specific first, with types first in each', () => {
    for (const [subpath, conditions] of subpaths()) {
      if (subpath === './package.json') continue;
      expect(Object.keys(conditions), subpath).toEqual([
        'workerd',
        'browser',
        'import',
        'require',
        'default',
      ]);
      // require() callers (e.g. Jest with jest-environment-jsdom, which sets the
      // "browser" condition) keep the CommonJS entry: the WebAssembly build is
      // ES-module-only (top-level await).
      for (const condition of ['browser']) {
        const target = conditions[condition];
        expect(
          typeof target === 'object' && Object.keys(target),
          `${subpath} ${condition}`,
        ).toEqual(['require', 'default']);
        expect(typeof target === 'object' && target.require).toEqual(conditions.require);
      }
      for (const [path, leaf] of leaves(conditions)) {
        expect(Object.keys(leaf), `${subpath} ${path.join('.')}`).toEqual(['types', 'default']);
      }
    }
  });

  it('resolves browsers and Cloudflare Workers to the WebAssembly build', () => {
    const cases: Array<[string, string[], 'import' | 'require', string]> = [
      ['rapid-fuzzy', [], 'import', 'index.mjs'],
      ['rapid-fuzzy', [], 'require', 'index.js'],
      ['rapid-fuzzy', ['browser'], 'import', 'browser.mjs'],
      ['rapid-fuzzy', ['browser'], 'require', 'index.js'],
      // Deno keeps the Node.js entry (the native addon); --conditions=browser opts in.
      ['rapid-fuzzy', ['deno', 'node'], 'import', 'index.mjs'],
      ['rapid-fuzzy', ['deno', 'node', 'browser'], 'import', 'browser.mjs'],
      ['rapid-fuzzy/objects', ['deno', 'node'], 'import', 'objects.mjs'],
      ['rapid-fuzzy', ['workerd', 'worker', 'browser'], 'import', 'workerd.mjs'],
      ['rapid-fuzzy/highlight', [], 'import', 'highlight.mjs'],
      ['rapid-fuzzy/highlight', ['browser'], 'import', 'highlight.browser.mjs'],
      ['rapid-fuzzy/highlight', ['browser'], 'require', 'highlight.js'],
      ['rapid-fuzzy/highlight', ['workerd'], 'import', 'highlight.browser.mjs'],
      ['rapid-fuzzy/objects', [], 'import', 'objects.mjs'],
      ['rapid-fuzzy/objects', ['browser'], 'import', 'browser.mjs'],
      ['rapid-fuzzy/objects', ['browser'], 'require', 'objects.js'],
      ['rapid-fuzzy/objects', ['workerd'], 'import', 'workerd.mjs'],
    ];
    for (const [specifier, conditions, mode, expected] of cases) {
      expect(nodeResolve(specifier, conditions, mode), `${mode} ${specifier} [${conditions}]`).toBe(
        expected,
      );
    }
    // Bundlers without "exports" support read the legacy fields.
    expect(pkg.browser).toBe('./browser.mjs');
    expect(pkg.module).toBeUndefined();
  });

  it('publishes every file the manifest points to', () => {
    const exported = subpaths()
      .flatMap(([, conditions]) => leaves(conditions))
      .flatMap(([, leaf]) => Object.values(leaf))
      .filter((target): target is string => typeof target === 'string');
    const targets = new Set([pkg.main, pkg.types, pkg.browser, ...exported]);
    for (const target of targets) {
      const file = target.replace(/^\.\//, '');
      expect(pkg.files, file).toContain(file);
      expect(existsSync(join(ROOT, file)), file).toBe(true);
    }
  });

  it('ships no ES module syntax in .js files (the package is CommonJS)', () => {
    expect(pkg.type).toBe('commonjs');
    for (const file of pkg.files.filter((f) => f.endsWith('.js'))) {
      const source = readFileSync(join(ROOT, file), 'utf8');
      expect(source.match(/^(?:import|export)\s/gm) ?? [], file).toEqual([]);
    }
  });

  it('keeps the browser module graph plain, self-contained ES modules', () => {
    for (const entry of ['browser.mjs', 'workerd.mjs', 'highlight.browser.mjs']) {
      for (const file of moduleGraph(entry)) {
        expect(file, `${entry} -> ${file}`).toMatch(/\.(mjs|wasm)$/);
        expect(pkg.files, `${entry} -> ${file}`).toContain(file);
      }
    }
    // browser.mjs fetches the .wasm through the URL pattern bundlers emit as an
    // asset; workerd.mjs imports it (Wrangler bundles it as a WebAssembly.Module).
    expect(readFileSync(join(ROOT, 'browser.mjs'), 'utf8')).toContain(
      "new URL('./rapid-fuzzy-wasm-bindgen_bg.wasm', import.meta.url)",
    );
    expect(moduleGraph('workerd.mjs')).toContain('rapid-fuzzy-wasm-bindgen_bg.wasm');
  });

  it('marks the modules that instantiate WebAssembly as having side effects', () => {
    // With "sideEffects": false, bundlers drop an entry whose exports are all
    // re-exports, and with it the instantiation of the WebAssembly module.
    for (const entry of ['./browser.mjs', './workerd.mjs', './index.js', './index.mjs']) {
      expect(pkg.sideEffects).toContain(entry);
    }
  });

  it('commits generated browser files that match scripts/build-browser.js', () => {
    const { render } = require('../scripts/build-browser.js') as {
      render(): Record<string, string>;
    };
    for (const [file, content] of Object.entries(render())) {
      expect(readFileSync(join(ROOT, file), 'utf8'), file).toBe(content);
    }
  });
});

describe('browser.mjs in Node.js before 22.3', () => {
  // Node.js 22.0-22.2 have no process.getBuiltinModule(), which browser.mjs uses
  // to read its .wasm under --conditions=browser, and their fetch() cannot read
  // file: URLs. Needs no binary: loading fails before the file is read.
  it('fails with an error naming the Node.js version it needs', () => {
    const code = [
      'delete process.getBuiltinModule;',
      "await import('rapid-fuzzy').then(",
      "  () => process.stdout.write('loaded'),",
      '  (error) => process.stdout.write(JSON.stringify([error.message, String(error.cause)])),',
      ');',
    ].join('\n');
    const stdout = execFileSync(
      process.execPath,
      ['--conditions=browser', '--input-type=module', '-e', code],
      { cwd: ROOT, encoding: 'utf8' },
    );
    expect(stdout).not.toBe('loaded');
    const [message, cause] = JSON.parse(stdout) as [string, string];
    expect(message).toMatch(/^rapid-fuzzy: the WebAssembly build needs Node\.js 22\.3 or later/);
    expect(message).toContain('rapid-fuzzy-wasm-bindgen_bg.wasm');
    expect(message).toContain('native addon');
    expect(cause).toMatch(/fetch failed/);
  });
});

describe('highlight.browser.mjs ("rapid-fuzzy/highlight" in browsers)', () => {
  it('matches highlight.js', async () => {
    // Typed through highlight.d.mts, which "rapid-fuzzy/highlight" uses in browsers.
    const specifier: string = '../highlight.browser.mjs';
    const esm = (await import(specifier)) as typeof highlightCjs;
    const cjsExports = require('../highlight.js') as Record<string, unknown>;
    expect(Object.keys(esm).sort()).toEqual(Object.keys(cjsExports).sort());
    const positions = [0, 3, 4];
    expect(esm.highlightRanges('fuzzy', positions)).toEqual(
      highlightCjs.highlightRanges('fuzzy', positions),
    );
    expect(esm.highlight('fuzzy', positions, '<b>', '</b>')).toBe(
      highlightCjs.highlight('fuzzy', positions, '<b>', '</b>'),
    );
    const upper = (s: string): string => s.toUpperCase();
    expect(esm.highlight('fuzzy', positions, upper)).toBe(
      highlightCjs.highlight('fuzzy', positions, upper),
    );
  });
});

describe.skipIf(!wasmAvailable)('browser.mjs (the WebAssembly build)', () => {
  type BrowserModule = typeof import('../browser.mjs', { with: { 'resolution-mode': 'import' }});
  const load = async (): Promise<BrowserModule> =>
    (await import('../browser.mjs')) as BrowserModule;

  it('exports exactly what the Node.js ES module entry exports', async () => {
    const browser = await load();
    const nodeEsm = (await import('../index.mjs')) as Record<string, unknown>;
    // Except the napi-rs loader's marker of the Node.js binding it loaded
    // (browser.types.ts excludes it too).
    const nodeNames = Object.keys(nodeEsm).filter((name) => name !== '__napiBindingTarget');
    expect(Object.keys(nodeEsm)).toContain('__napiBindingTarget');
    expect(Object.keys(browser).sort()).toEqual(nodeNames.sort());
  });

  it('is ready to use as soon as the import resolves', async () => {
    const browser = await load();
    expect(browser.levenshtein('kitten', 'sitting')).toBe(3);
    const items = ['TypeScript', 'JavaScript', 'Python', 'TypeSpec'];
    expect(browser.search('typscript', items)).toEqual(napi.search('typscript', items));
    const index = new browser.FuzzyIndex(items);
    expect(index.search('typ')).toEqual(new napi.FuzzyIndex(items).search('typ'));
    expect(index.closest('pyton')).toBe('Python');
    index.free();
  });

  it('exposes MatchType and the TypedArray variants like index.js', async () => {
    const browser = await load();
    // MatchType is a `const enum` in index.d.ts, so read the runtime object untyped.
    const nodeMatchType = (napi as unknown as Record<string, Record<string, string>>).MatchType;
    const ownProperties = (object: object | undefined): Record<string, unknown> =>
      Object.fromEntries(
        Object.getOwnPropertyNames(object ?? {}).map((k) => [
          k,
          (object as Record<string, unknown>)[k],
        ]),
      );
    expect(ownProperties(browser.MatchType)).toEqual(ownProperties(nodeMatchType));
    expect(Object.isFrozen(browser.MatchType)).toBe(true);
    const u32 = browser.levenshteinManyU32('kitten', ['sitting', 'kitten']);
    expect(u32).toBeInstanceOf(Uint32Array);
    expect(Array.from(u32)).toEqual([3, 0]);
    expect(browser.jaroManyF64('abc', ['abc'])).toBeInstanceOf(Float64Array);
    expect(Array.from(browser.hammingManyU32('abc', ['abd', 'abcd']))).toEqual(
      Array.from(napi.hammingManyU32('abc', ['abd', 'abcd'])),
    );
    const f64 = Array.from(browser.normalizedHammingManyF64('ab', ['ab', 'abc']));
    expect(f64[0]).toBe(1);
    expect(f64[1]).toBeNaN();
  });

  it('includes highlight()', async () => {
    const browser = await load();
    const [hit] = browser.search('fzy', ['fuzzy'], { includePositions: true });
    expect(hit && browser.highlight(hit.item, hit.positions, '<b>', '</b>')).toBe(
      '<b>f</b>uz<b>zy</b>',
    );
  });

  describe('object search (#730)', () => {
    const users = [
      { name: 'John Smith', email: 'john@example.com', address: { city: 'Boston' } },
      { name: 'Jane Doe', email: 'jane@example.com', address: { city: 'Denver' } },
      { name: 'Bob Johnson', email: 'bob@example.com', address: { city: 'Austin' } },
    ];
    const keys = [{ name: 'name', weight: 2 }, 'email', 'address.city'];

    it('searchObjects matches the Node.js implementation', async () => {
      const browser = await load();
      for (const query of ['john', 'jane', 'austin', 'zzz']) {
        expect(browser.searchObjects(query, users, { keys })).toEqual(
          searchObjects(query, users, { keys }),
        );
      }
      expect(() => browser.searchObjects('a', users, { keys: [] })).toThrow(TypeError);
    });

    it('FuzzyObjectIndex matches the Node.js implementation', async () => {
      const browser = await load();
      const index = new browser.FuzzyObjectIndex(users, { keys });
      const native = new NodeFuzzyObjectIndex(users, { keys });
      expect(index.size).toBe(3);
      expect(index.search('john')).toEqual(native.search('john'));
      expect(index.closest('dnever')).toEqual(native.closest('dnever'));
      const extra = { name: 'Johnny', email: 'j@example.com', address: { city: 'Miami' } };
      index.add(extra);
      native.add(extra);
      expect(index.remove(0)).toBe(native.remove(0));
      expect(index.search('john')).toEqual(native.search('john'));
      index.destroy();
      expect(index.size).toBe(0);
    });

    it('passes matchMode through like the Node.js implementation (#782)', async () => {
      const browser = await load();
      const index = new browser.FuzzyObjectIndex(users, { keys });
      const native = new NodeFuzzyObjectIndex(users, { keys });
      for (const matchMode of ['perKey', 'crossKey'] as const) {
        for (const scoreMode of ['weighted', 'matched', 'max'] as const) {
          for (const query of ['john boston', 'smith example', 'john !boston', 'denver']) {
            const options = { matchMode, scoreMode };
            const expected = searchObjects(query, users, { keys, ...options });
            expect(browser.searchObjects(query, users, { keys, ...options })).toEqual(expected);
            expect(index.search(query, options)).toEqual(native.search(query, options));
            const closestOptions = { minScore: 0.6, ...options };
            expect(index.closest(query, closestOptions)).toEqual(
              native.closest(query, closestOptions),
            );
          }
        }
      }
      // The terms of 'john boston' are in different keys of John Smith.
      expect(index.search('john boston')).toEqual([]);
      expect(index.closest('john boston', { scoreMode: 'max', matchMode: 'crossKey' })).toEqual(
        users[0],
      );
      const bogus = 'cross' as 'crossKey';
      expect(() => index.search('john', { matchMode: bogus })).toThrow(TypeError);
      expect(() => index.closest('john', { matchMode: bogus })).toThrow(TypeError);
      expect(() => browser.searchObjects('john', users, { keys, matchMode: bogus })).toThrow(
        TypeError,
      );
      index.destroy();
    });

    it('passes scoreMode through like the Node.js implementation (#781)', async () => {
      const browser = await load();
      const index = new browser.FuzzyObjectIndex(users, { keys });
      const native = new NodeFuzzyObjectIndex(users, { keys });
      for (const scoreMode of ['weighted', 'matched', 'max'] as const) {
        for (const query of ['john', 'boston', 'example']) {
          const expected = searchObjects(query, users, { keys, scoreMode });
          expect(browser.searchObjects(query, users, { keys, scoreMode })).toEqual(expected);
          expect(index.search(query, { scoreMode })).toEqual(native.search(query, { scoreMode }));
          expect(index.closest(query, { minScore: 0.6, scoreMode })).toEqual(
            native.closest(query, { minScore: 0.6, scoreMode }),
          );
        }
      }
      // A single exact match on one key out of three scores 1 in 'matched' mode.
      expect(index.closest('boston', 0.9)).toBeNull();
      expect(index.closest('boston', { minScore: 0.9, scoreMode: 'matched' })).toEqual(users[0]);
      const bogus = 'mean' as 'max';
      expect(() => index.search('john', { scoreMode: bogus })).toThrow(TypeError);
      expect(() => index.closest('john', { scoreMode: bogus })).toThrow(TypeError);
      expect(() => browser.searchObjects('john', users, { keys, scoreMode: bogus })).toThrow(
        TypeError,
      );
      index.destroy();
    });

    it('reads search options like the Node.js implementation', async () => {
      const browser = await load();
      const index = new browser.FuzzyObjectIndex(users, { keys });
      const native = new NodeFuzzyObjectIndex(users, { keys });
      class Getters {
        get maxResults(): number {
          return 1;
        }
        get scoreMode(): 'max' {
          return 'max';
        }
        get matchMode(): 'crossKey' {
          return 'crossKey';
        }
      }
      const options: ObjectIndexSearchOptions[] = [
        new Getters(),
        Object.create({ maxResults: 1 }) as ObjectIndexSearchOptions,
        Object.create({ scoreMode: 'max', matchMode: 'crossKey' }) as ObjectIndexSearchOptions,
      ];
      for (const opts of options) {
        for (const query of ['john', 'john boston', 'example']) {
          expect(index.search(query, opts)).toEqual(native.search(query, opts));
        }
      }
      // Options from a getter or the prototype chain take effect.
      expect(native.search('john', new Getters())).toHaveLength(1);
      expect(native.search('john boston', new Getters())).toHaveLength(1);
      index.destroy();
    });

    it('rejects a null scoreMode or matchMode option like the Node.js implementation', async () => {
      const browser = await load();
      const index = new browser.FuzzyObjectIndex(users, { keys });
      const native = new NodeFuzzyObjectIndex(users, { keys });
      for (const options of [{ scoreMode: null }, { matchMode: null }]) {
        const opts = options as unknown as ObjectIndexSearchOptions;
        const field = Object.keys(options)[0];
        const message = new RegExp(`^${field} must be .*, got null$`);
        expect(() => native.search('john', opts)).toThrow(message);
        expect(() => searchObjects('john', users, { keys, ...opts })).toThrow(message);
        expect(() => index.search('john', opts)).toThrow(TypeError);
        expect(() => index.search('john', opts)).toThrow(
          new RegExp(`${field} must be .*, got null$`),
        );
        expect(() => browser.searchObjects('john', users, { keys, ...opts })).toThrow(TypeError);
        expect(() => native.closest('john', opts)).toThrow(message);
        expect(() => index.closest('john', opts)).toThrow(TypeError);
        expect(() => index.closest('john', opts)).toThrow(
          new RegExp(`${field} must be .*, got null$`),
        );
      }
      // A null options argument means the defaults.
      expect(index.closest('john', null)).toEqual(native.closest('john', null));
      expect(index.closest('john', null)).toEqual(users[0]);
      index.destroy();
    });

    it('reads closest() options like the Node.js implementation', async () => {
      const browser = await load();
      const index = new browser.FuzzyObjectIndex(users, { keys });
      const native = new NodeFuzzyObjectIndex(users, { keys });
      const options: Array<number | napi.KeyClosestOptions | undefined | null> = [
        undefined,
        null,
        {},
        0,
        0.6,
        0.9,
        1,
        { minScore: 0.9 },
        { scoreMode: 'matched' },
        { minScore: 0.9, scoreMode: 'matched' },
        { scoreMode: 'max', matchMode: 'crossKey' },
        { minScore: 1, scoreMode: 'max', matchMode: 'crossKey' },
        { minScore: undefined, scoreMode: undefined, matchMode: undefined },
      ];
      let found = 0;
      for (const opts of options) {
        for (const query of ['john', 'boston', 'john boston', 'smith example', 'zzz', '']) {
          const expected = native.closest(query, opts);
          if (expected !== null) found++;
          expect(index.closest(query, opts)).toEqual(expected);
        }
      }
      expect(found).toBeGreaterThan(20);
      // A number is a shorthand for minScore.
      expect(index.closest('boston', 0.9)).toEqual(index.closest('boston', { minScore: 0.9 }));
      // Options that are neither a number nor an object are rejected alike.
      for (const bad of ['0.9', true]) {
        const message = `options must be a number (minScore) or a KeyClosestOptions object, got ${typeof bad}`;
        const opts = bad as unknown as number;
        expect(() => native.closest('john', opts)).toThrow(new TypeError(message));
        expect(() => index.closest('john', opts)).toThrow(new TypeError(message));
        const searchMessage = `options must be a number (maxResults) or a KeySearchOptions object, got ${typeof bad}`;
        expect(() => native.search('john', opts)).toThrow(new TypeError(searchMessage));
        expect(() => index.search('john', opts)).toThrow(new TypeError(searchMessage));
      }
      index.destroy();
    });

    it('leaves out the Buffer-based serialize() / deserialize()', async () => {
      const browser = await load();
      expect('serialize' in browser.FuzzyObjectIndex.prototype).toBe(false);
      expect('deserialize' in browser.FuzzyObjectIndex).toBe(false);
    });

    it('comes from objects.browser.mjs, the ES module build of objects.js', async () => {
      const browser = await load();
      const objects = (await import('../objects.browser.mjs')) as Record<string, unknown>;
      expect(Object.keys(objects).sort()).toEqual(['FuzzyObjectIndex', 'searchObjects']);
      expect(objects.FuzzyObjectIndex).toBe(browser.FuzzyObjectIndex);
      expect(objects.searchObjects).toBe(browser.searchObjects);
    });
  });

  it('loads in Node.js under --conditions=browser (process.getBuiltinModule fallback)', () => {
    const code = [
      "const rf = await import('rapid-fuzzy');",
      "const { searchObjects } = await import('rapid-fuzzy/objects');",
      'process.stdout.write(JSON.stringify([',
      "  import.meta.resolve('rapid-fuzzy').split('/').pop(),",
      "  rf.levenshtein('kitten', 'sitting'),",
      "  searchObjects('jane', [{ name: 'Jane' }], { keys: ['name'] }).length,",
      ']));',
    ].join('\n');
    const stdout = execFileSync(
      process.execPath,
      ['--conditions=browser', '--input-type=module', '-e', code],
      { cwd: ROOT, encoding: 'utf8' },
    );
    expect(JSON.parse(stdout)).toEqual(['browser.mjs', 3, 1]);
  });
});
