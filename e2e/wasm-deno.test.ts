// Deno does not support the WASI CJS loader (Context is not supported),
// so we use the browser loader which works via @napi-rs/wasm-runtime.
const wasm = await import('../rapid-fuzzy.wasi-browser.js');

Deno.test('distance - levenshtein', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.levenshtein('hello', 'hello'), 0);
  assertEquals(wasm.levenshtein('kitten', 'sitting'), 3);
});

Deno.test('distance - normalizedLevenshtein', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.normalizedLevenshtein('hello', 'hello'), 1.0);
});

Deno.test('distance - damerauLevenshtein', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.damerauLevenshtein('hello', 'ehllo'), 1);
});

Deno.test('distance - jaro', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.jaro('hello', 'hello'), 1.0);
});

Deno.test('distance - jaroWinkler', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.jaroWinkler('hello', 'hello'), 1.0);
});

Deno.test('distance - sorensenDice', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.sorensenDice('hello', 'hello'), 1.0);
});

Deno.test('batch - levenshteinBatch', () => {
  const { assertEquals } = await_assert();
  assertEquals(
    wasm.levenshteinBatch([
      ['hello', 'hello'],
      ['hello', 'world'],
    ]),
    [0, 4],
  );
});

Deno.test('many - levenshteinMany', () => {
  const { assertEquals } = await_assert();
  const result = wasm.levenshteinMany('hello', ['hello', 'world', 'help']);
  assertEquals(result.length, 3);
  assertEquals(result[0], 0);
});

Deno.test('token - tokenSortRatio', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.tokenSortRatio('New York Mets', 'Mets New York'), 1.0);
});

Deno.test('token - partialRatio', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.partialRatio('hello', 'hello world'), 1.0);
});

Deno.test('token - weightedRatio', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.weightedRatio('hello', 'hello'), 1.0);
});

Deno.test('search - returns results', () => {
  const { assert, assertEquals } = await_assert();
  const results = wasm.search('type', ['TypeScript', 'JavaScript', 'Python']);
  assert(results.length > 0);
  assertEquals(results[0].item, 'TypeScript');
});

Deno.test('search - empty query returns empty', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.search('', ['hello']), []);
});

Deno.test('closest - returns best match', () => {
  const { assertNotEquals } = await_assert();
  const result = wasm.closest('apple', ['application', 'banana', 'apple pie']);
  assertNotEquals(result, null);
});

Deno.test('closest - empty items returns null', () => {
  const { assertEquals } = await_assert();
  assertEquals(wasm.closest('hello', []), null);
});

Deno.test('FuzzyIndex - lifecycle', () => {
  const { assertEquals, assert } = await_assert();
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

// Helper: lazily import Deno's assertion module
function await_assert() {
  // Deno std assertions are available globally via Deno.test
  return {
    assert: (expr: boolean, msg?: string) => {
      if (!expr) throw new Error(msg ?? 'Assertion failed');
    },
    assertEquals: (actual: unknown, expected: unknown) => {
      if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
      }
    },
    assertNotEquals: (actual: unknown, expected: unknown) => {
      if (JSON.stringify(actual) === JSON.stringify(expected)) {
        throw new Error(`Expected value to differ from ${JSON.stringify(expected)}`);
      }
    },
  };
}
