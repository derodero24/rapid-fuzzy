import { assert, assertEquals, assertNotEquals } from 'jsr:@std/assert';

// The package imports itself by name: with --conditions=browser (see
// `pnpm run test:deno`) Deno resolves it to the WebAssembly build (browser.mjs),
// which reads the .wasm file (--allow-read). Without it Deno loads the Node.js
// entry and its native addon. `pnpm run test:deno` passes --no-check: Deno type-checks a
// self-referenced package as local files, not with the npm resolution users get
// for `npm:rapid-fuzzy` (whose declarations __test__/types/browser.types.ts covers).
// It also passes --node-modules-dir=manual, so Deno reads the node_modules that
// `pnpm install` created without writing to it: `auto` relinks the dependencies to
// Deno's own resolution, which ignores pnpm-lock.yaml, and `pnpm install` does not
// undo that.
import * as wasm from 'rapid-fuzzy';
import { highlight } from 'rapid-fuzzy/highlight';
import { FuzzyObjectIndex } from 'rapid-fuzzy/objects';

Deno.test('resolves the WebAssembly build', () => {
  assert(import.meta.resolve('rapid-fuzzy').endsWith('/browser.mjs'));
  assert(import.meta.resolve('rapid-fuzzy/objects').endsWith('/browser.mjs'));
});

Deno.test('distance - levenshtein', () => {
  assertEquals(wasm.levenshtein('hello', 'hello'), 0);
  assertEquals(wasm.levenshtein('kitten', 'sitting'), 3);
});

Deno.test('distance - normalizedLevenshtein', () => {
  assertEquals(wasm.normalizedLevenshtein('hello', 'hello'), 1.0);
});

Deno.test('distance - damerauLevenshtein', () => {
  assertEquals(wasm.damerauLevenshtein('hello', 'ehllo'), 1);
});

Deno.test('distance - jaro', () => {
  assertEquals(wasm.jaro('hello', 'hello'), 1.0);
});

Deno.test('distance - jaroWinkler', () => {
  assertEquals(wasm.jaroWinkler('hello', 'hello'), 1.0);
});

Deno.test('distance - sorensenDice', () => {
  assertEquals(wasm.sorensenDice('hello', 'hello'), 1.0);
});

Deno.test('distance - hamming', () => {
  assertEquals(wasm.hamming('hello', 'hello'), 0);
  assertEquals(wasm.hamming('karolin', 'kathrin'), 3);
  assertEquals(wasm.hamming('hello', 'hi'), null);
});

Deno.test('distance - indel', () => {
  assertEquals(wasm.indel('hello', 'hello'), 0);
  assertEquals(wasm.indel('abc', 'ac'), 1);
});

Deno.test('distance - normalizedIndel', () => {
  assertEquals(wasm.normalizedIndel('hello', 'hello'), 1.0);
});

Deno.test('distance - normalizedHamming', () => {
  assertEquals(wasm.normalizedHamming('hello', 'hello'), 1.0);
  assertEquals(wasm.normalizedHamming('hello', 'hi'), null);
});

Deno.test('batch - levenshteinBatch', () => {
  assertEquals(
    wasm.levenshteinBatch([
      ['hello', 'hello'],
      ['hello', 'world'],
    ]),
    new Uint32Array([0, 4]),
  );
});

Deno.test('batch - indelBatch', () => {
  const result = wasm.indelBatch([
    ['hello', 'hello'],
    ['abc', 'ac'],
  ]);
  assertEquals(Array.from(result), [0, 1]);
});

Deno.test('many - levenshteinMany', () => {
  const result = wasm.levenshteinMany('hello', ['hello', 'world', 'help']);
  assertEquals(result.length, 3);
  assertEquals(result[0], 0);
});

Deno.test('many - indelMany', () => {
  const result = wasm.indelMany('abc', ['abc', 'ac', '']);
  assertEquals(result.length, 3);
  assertEquals(result[0], 0);
  assertEquals(result[1], 1);
});

Deno.test('many - normalizedHammingMany', () => {
  const result = wasm.normalizedHammingMany('hello', ['hello', 'world', 'hi']);
  assertEquals(result.length, 3);
  assertEquals(result[0], 1.0);
  // Length mismatches are null, as in the Node.js binding
  assertEquals(result[2], null);
});

Deno.test('token - tokenSortRatio', () => {
  assertEquals(wasm.tokenSortRatio('New York Mets', 'Mets New York'), 1.0);
});

Deno.test('token - partialRatio', () => {
  assertEquals(wasm.partialRatio('hello', 'hello world'), 1.0);
});

Deno.test('token - weightedRatio', () => {
  assertEquals(wasm.weightedRatio('hello', 'hello'), 1.0);
});

Deno.test('search - returns results', () => {
  const results = wasm.search('type', ['TypeScript', 'JavaScript', 'Python']);
  assert(results.length > 0);
  assertEquals(results[0].item, 'TypeScript');
});

Deno.test('search - empty query returns empty', () => {
  assertEquals(wasm.search('', ['hello']), []);
});

Deno.test('closest - returns best match', () => {
  const result = wasm.closest('apple', ['application', 'banana', 'apple pie']);
  assertNotEquals(result, null);
});

Deno.test('closest - empty items returns null', () => {
  assertEquals(wasm.closest('hello', []), null);
});

Deno.test('FuzzyIndex - lifecycle', () => {
  const index = new wasm.FuzzyIndex(['apple', 'banana', 'grape', 'orange']);
  assertEquals(index.size, 4);

  const results = index.search('aple');
  assert(results.length > 0);
  assert(results.some((r: { item: string }) => r.item === 'apple'));

  assertEquals(index.closest('aple'), 'apple');

  index.add('mango');
  assertEquals(index.size, 5);

  index.destroy();
  assertEquals(index.size, 0);
});

Deno.test('highlight - rapid-fuzzy/highlight', () => {
  const [hit] = wasm.search('fzy', ['fuzzy'], { includePositions: true });
  assert(hit !== undefined);
  assertEquals(highlight(hit.item, hit.positions, '[', ']'), '[f]uz[zy]');
});

Deno.test('objects - FuzzyObjectIndex', () => {
  const index = new FuzzyObjectIndex([{ name: 'Jane' }, { name: 'John' }], { keys: ['name'] });
  assertEquals(index.search('jane')[0]?.item, { name: 'Jane' });
  assertEquals(index.closest('jon'), { name: 'John' });
});
