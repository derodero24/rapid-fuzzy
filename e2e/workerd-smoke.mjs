// Smoke test of the Cloudflare Workers entry point (workerd.mjs, selected by the
// package's "workerd" export condition), run in Node.js by `pnpm run test:workerd`.
// workerd.mjs imports the .wasm with a `?module` suffix and instantiates it with
// initSync({ module }) at startup, a path the browser, Bun and Deno tests never
// take (they load browser.mjs). e2e/workerd-hooks.mjs makes Node.js handle that
// import the way Wrangler does. Needs the wasm-bindgen build
// (`pnpm run build:wasm-bindgen`).
import assert from 'node:assert/strict';

const entry = import.meta.resolve('rapid-fuzzy');
assert.ok(
  entry.endsWith('/workerd.mjs'),
  `rapid-fuzzy resolved to ${entry}: use --conditions=workerd`,
);
assert.ok(import.meta.resolve('rapid-fuzzy/objects').endsWith('/workerd.mjs'));
assert.ok(import.meta.resolve('rapid-fuzzy/highlight').endsWith('/highlight.browser.mjs'));

// The example worker, called the way the Workers runtime calls it.
const { default: worker } = await import('../examples/cloudflare-workers/worker.js');
async function get(query) {
  const response = await worker.fetch(new Request(`https://worker.example/?${query}`));
  assert.equal(response.status, 200);
  return response.json();
}
const found = await get('q=typscript');
assert.equal(found[0]?.item, 'TypeScript');
assert.ok(found[0].score > 0 && found[0].score < 1);
assert.equal(await get('q=tsc&closest'), 'TypeScript');
assert.equal(await get('q=zzz&closest'), null);

// The rest of the entry point's API.
const rf = await import('rapid-fuzzy');
assert.equal(rf.levenshtein('kitten', 'sitting'), 3);
assert.deepEqual(Array.from(rf.levenshteinMany('kitten', ['sitting', 'mitten'])), [3, 1]);
assert.equal(rf.closest('tsc', ['TypeScript', 'Rust']), 'TypeScript');
const index = new rf.FuzzyIndex(['apple', 'apricot', 'banana']);
assert.deepEqual(
  index.search('ap').map((r) => r.item),
  ['apple', 'apricot'],
);
const keyed = new rf.KeyedFuzzyIndex(
  [
    ['John Smith', 'Jane Doe'],
    ['john@example.com', 'jane@example.com'],
  ],
  [2, 1],
);
assert.equal(keyed.closest('jane'), 1);
assert.equal(rf.MatchType.Exact, 'Exact');

const { FuzzyObjectIndex, searchObjects } = await import('rapid-fuzzy/objects');
const users = [{ name: 'John Smith' }, { name: 'Jane Doe' }];
assert.equal(searchObjects('jane', users, { keys: ['name'] })[0]?.item, users[1]);
assert.equal(new FuzzyObjectIndex(users, { keys: ['name'] }).search('john')[0]?.index, 0);

const { highlight } = await import('rapid-fuzzy/highlight');
const [hit] = rf.search('tsc', ['TypeScript'], { includePositions: true });
assert.equal(highlight(hit.item, hit.positions, '[', ']'), '[T]ype[Sc]ript');

console.log('workerd.mjs smoke test passed');
