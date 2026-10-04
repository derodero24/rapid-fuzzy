// Consumer app for e2e/browser.spec.ts. e2e/serve-browser-app.mjs installs the
// packed rapid-fuzzy tarball next to it and serves it through Vite, so these
// imports resolve exactly as they do for users (the "browser" export condition).
import * as rf from 'rapid-fuzzy';
import { highlight, highlightRanges } from 'rapid-fuzzy/highlight';
import { FuzzyObjectIndex, searchObjects } from 'rapid-fuzzy/objects';

const out = document.getElementById('output');

/** Plain arrays, so typed arrays survive the JSON round trip to Playwright. */
const list = (values) => Array.from(values);

try {
  const results = {};

  // Distance functions
  results.levenshtein = rf.levenshtein('kitten', 'sitting');
  results.levenshteinIdentical = rf.levenshtein('hello', 'hello');
  results.normalizedLevenshtein = rf.normalizedLevenshtein('hello', 'hello');
  results.damerauLevenshtein = rf.damerauLevenshtein('hello', 'ehllo');
  results.jaro = rf.jaro('hello', 'hello');
  results.jaroWinkler = rf.jaroWinkler('hello', 'hello');
  results.sorensenDice = rf.sorensenDice('hello', 'hello');
  results.hamming = rf.hamming('karolin', 'kathrin');
  results.hammingNull = rf.hamming('hello', 'hi');
  results.indel = rf.indel('abc', 'ac');
  results.normalizedIndel = rf.normalizedIndel('hello', 'hello');
  results.normalizedHamming = rf.normalizedHamming('hello', 'hello');
  results.normalizedHammingNull = rf.normalizedHamming('hello', 'hi');

  // Batch functions
  results.levenshteinBatch = list(
    rf.levenshteinBatch([
      ['hello', 'hello'],
      ['hello', 'world'],
    ]),
  );
  results.jaroBatch = list(
    rf.jaroBatch([
      ['hello', 'hello'],
      ['abc', 'xyz'],
    ]),
  );
  results.hammingBatch = list(
    rf.hammingBatch([
      ['hello', 'hello'],
      ['karolin', 'kathrin'],
    ]),
  );
  results.indelBatch = list(
    rf.indelBatch([
      ['hello', 'hello'],
      ['abc', 'ac'],
    ]),
  );
  results.normalizedIndelBatch = list(
    rf.normalizedIndelBatch([
      ['hello', 'hello'],
      ['hello', 'world'],
    ]),
  );
  results.normalizedHammingBatch = list(
    rf.normalizedHammingBatch([
      ['hello', 'hello'],
      ['hello', 'world'],
    ]),
  );

  // Many functions
  results.levenshteinMany = list(rf.levenshteinMany('hello', ['hello', 'world', 'help']));
  results.jaroMany = list(rf.jaroMany('hello', ['hello', 'world']));
  results.hammingMany = list(rf.hammingMany('hello', ['hello', 'world', 'hi']));
  results.indelMany = list(rf.indelMany('abc', ['abc', 'ac', '']));
  results.normalizedIndelMany = list(rf.normalizedIndelMany('hello', ['hello', 'world']));
  results.normalizedHammingMany = list(rf.normalizedHammingMany('hello', ['hello', 'world', 'hi']));

  // TypedArray variants
  const u32 = rf.levenshteinManyU32('hello', ['hello', 'world']);
  results.levenshteinManyU32 = { typed: u32 instanceof Uint32Array, values: list(u32) };
  results.hammingManyU32 = list(rf.hammingManyU32('hello', ['hello', 'hi']));

  // Token-based functions
  results.tokenSortRatio = rf.tokenSortRatio('New York Mets', 'Mets New York');
  results.tokenSetRatio = rf.tokenSetRatio('Mariners vs Yankees', 'Yankees vs Mariners');
  results.partialRatio = rf.partialRatio('hello', 'hello world');
  results.weightedRatio = rf.weightedRatio('hello', 'hello');

  // Search
  results.search = rf.search('type', ['TypeScript', 'JavaScript', 'Python', 'TypeSpec']);
  results.searchEmpty = rf.search('', ['hello', 'world']);
  results.matchType = rf.search('type', ['TypeScript'], { includePositions: true })[0]?.matchType;
  results.matchTypeEnum = { ...rf.MatchType };

  // Closest: null (not undefined) when nothing matches.
  results.closest = rf.closest('apple', ['application', 'banana', 'apple pie']);
  results.closestEmpty = rf.closest('hello', []);

  // FuzzyIndex
  const index = new rf.FuzzyIndex(['apple', 'banana', 'grape', 'orange']);
  results.indexSize = index.size;
  results.indexSearch = index.search('aple');
  results.indexClosest = index.closest('aple');
  index.add('mango');
  results.indexSizeAfterAdd = index.size;
  index.destroy();
  results.indexSizeAfterDestroy = index.size;

  // highlight (main entry and the "rapid-fuzzy/highlight" subpath)
  const [hit] = rf.search('fzy', ['fuzzy'], { includePositions: true });
  results.highlight = rf.highlight(hit.item, hit.positions, '<b>', '</b>');
  results.highlightSubpath = highlight(hit.item, hit.positions, (s) => `[${s}]`);
  results.highlightRanges = highlightRanges(hit.item, hit.positions);

  // Object search ("rapid-fuzzy/objects")
  const users = [
    { name: 'John Smith', email: 'john@example.com', address: { city: 'Boston' } },
    { name: 'Jane Doe', email: 'jane@example.com', address: { city: 'Denver' } },
  ];
  const keys = [{ name: 'name', weight: 2 }, 'email', 'address.city'];
  results.searchObjects = searchObjects('jane', users, { keys }).map((r) => r.item.email);
  const objectIndex = new FuzzyObjectIndex(users, { keys });
  objectIndex.add({
    name: 'Johnny Cash',
    email: 'cash@example.com',
    address: { city: 'Nashville' },
  });
  results.objectIndexSearch = objectIndex.search('john').map((r) => r.item.name);
  results.objectIndexClosest = objectIndex.closest('denvr')?.name ?? null;
  results.objectIndexSize = objectIndex.size;
  results.objectsFromMainEntry = rf.FuzzyObjectIndex === FuzzyObjectIndex;

  results.exports = Object.keys(rf).sort();
  // The .wasm the bundler emitted for `new URL(..., import.meta.url)`.
  results.wasmRequests = performance
    .getEntriesByType('resource')
    .map((entry) => entry.name)
    .filter((name) => new URL(name).pathname.endsWith('.wasm'));

  window.__results = results;
  window.__ready = true;
  out.textContent = JSON.stringify(results, null, 2);
} catch (err) {
  window.__error = `${err.name}: ${err.message}`;
  out.textContent = `Error: ${err.message}\n${err.stack}`;
}
