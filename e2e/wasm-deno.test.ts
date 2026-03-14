import { assert, assertEquals, assertNotEquals } from 'jsr:@std/assert';

// Deno does not support the WASI CJS loader (Context is not supported),
// so we use the browser loader which works via @napi-rs/wasm-runtime.
const wasm = await import('../rapid-fuzzy.wasi-browser.js');

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

Deno.test('batch - levenshteinBatch', () => {
  assertEquals(
    wasm.levenshteinBatch([
      ['hello', 'hello'],
      ['hello', 'world'],
    ]),
    [0, 4],
  );
});

Deno.test('many - levenshteinMany', () => {
  const result = wasm.levenshteinMany('hello', ['hello', 'world', 'help']);
  assertEquals(result.length, 3);
  assertEquals(result[0], 0);
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
