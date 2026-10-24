// Competitor comparison benchmarks (local use only, not run in CI)
import uFuzzy from '@leeoniya/ufuzzy';
import { closest as fastestLevenshteinClosest } from 'fastest-levenshtein';
import { Index as FlexSearchIndex } from 'flexsearch';
import Fuse from 'fuse.js';
import fuzzysort from 'fuzzysort';
import MiniSearch from 'minisearch';
import { test, vi } from 'vitest';

import { closest, FuzzyIndex, search } from '../index.js';
import * as fixtures from './bench-fixtures.js';

// Vitest gives each benchmark test 60 s. The type-ahead group takes about 30 s
// on a shared 4-CPU VM (tinybench takes at least 64 samples, and fuse.js runs
// the 12 searches 4 times a second), so leave room for slower machines.
vi.setConfig({ testTimeout: 300_000 });

// Vitest's module runner turns imported bindings into getters: copy the
// fixtures into local constants so the measured functions don't call a getter
// on every iteration.
const {
  cycle,
  hugeItems,
  largeClosestQuery,
  largeItems,
  largeQuery,
  mediumClosestQuery,
  mediumItems,
  rotatingQueries,
  smallItems,
  typeAheadQueries,
  xlargeItems,
} = fixtures;

// Every query matches items in rapid-fuzzy: a query that matches nothing
// measures an early exit, not a search. Result counts differ between libraries
// (fuse.js, for example, finds nothing for abbreviations such as "ctrl").

// --- Pre-initialize search instances ---

// Fuse.js
const fuseSmall = new Fuse(smallItems, { threshold: 0.4 });
const fuseMedium = new Fuse(mediumItems, { threshold: 0.4 });
const fuseLarge = new Fuse(largeItems, { threshold: 0.4 });

// fuzzysort (prepared targets). v4 defaults to threshold 0.5, which drops weak
// matches before scoring; threshold 0 keeps every match like the other libraries.
const fuzzysortMediumPrepared = mediumItems.map((item) => fuzzysort.prepare(item));
const fuzzysortLargePrepared = largeItems.map((item) => fuzzysort.prepare(item));
const fuzzysortXlargePrepared = xlargeItems.map((item) => fuzzysort.prepare(item));
const fuzzysortHugePrepared = hugeItems.map((item) => fuzzysort.prepare(item));

// uFuzzy
const uf = new uFuzzy();

// FlexSearch
function createFlexSearch(items: readonly string[]) {
  const idx = new FlexSearchIndex();
  for (const [i, item] of items.entries()) idx.add(i, item);
  return idx;
}
const flexSmall = createFlexSearch(smallItems);
const flexMedium = createFlexSearch(mediumItems);
const flexLarge = createFlexSearch(largeItems);
const flexXlarge = createFlexSearch(xlargeItems);
const flexHuge = createFlexSearch(hugeItems);

// MiniSearch
function createMiniSearch(items: readonly string[]) {
  const ms = new MiniSearch({ fields: ['text'], storeFields: ['text'] });
  ms.addAll(items.map((text, id) => ({ id, text })));
  return ms;
}
const miniSmall = createMiniSearch(smallItems);
const miniMedium = createMiniSearch(mediumItems);
const miniLarge = createMiniSearch(largeItems);
const miniXlarge = createMiniSearch(xlargeItems);
const miniHuge = createMiniSearch(hugeItems);

// rapid-fuzzy FuzzyIndex
const fuzzyIndexSmall = new FuzzyIndex(smallItems);
const fuzzyIndexMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexLarge = new FuzzyIndex(largeItems);
const fuzzyIndexXlarge = new FuzzyIndex(xlargeItems);
const fuzzyIndexHuge = new FuzzyIndex(hugeItems);
const fuzzyIndexLargeRotating = new FuzzyIndex(largeItems);
const fuzzyIndexLargeTypeAhead = new FuzzyIndex(largeItems);
const fuzzyIndexClosestMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexClosestLarge = new FuzzyIndex(largeItems);

test('Fuzzy Search — Small 20 (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search('aple', smallItems, 5);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexSmall.search('aple', { maxResults: 5 });
    }),
    bench('fuse.js', () => {
      fuseSmall.search('aple', { limit: 5 });
    }),
    bench('fuzzysort', () => {
      fuzzysort.go('aple', smallItems, { limit: 5, threshold: 0 });
    }),
    bench('uFuzzy', () => {
      uf.search(smallItems, 'aple');
    }),
    bench('FlexSearch', () => {
      flexSmall.search('aple', { limit: 5 });
    }),
    bench('MiniSearch', () => {
      miniSmall.search('aple', { fuzzy: 0.2, prefix: true });
    }),
  );
});

test('Fuzzy Search — Medium 1K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search('utils config', mediumItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexMedium.search('utils config', { maxResults: 10 });
    }),
    bench('fuse.js', () => {
      fuseMedium.search('utils config', { limit: 10 });
    }),
    bench('fuzzysort', () => {
      fuzzysort.go('utils config', fuzzysortMediumPrepared, { limit: 10, threshold: 0 });
    }),
    bench('uFuzzy', () => {
      uf.search(mediumItems, 'utils config');
    }),
    bench('FlexSearch', () => {
      flexMedium.search('utils config', { limit: 10 });
    }),
    bench('MiniSearch', () => {
      miniMedium.search('utils config', { fuzzy: 0.2, prefix: true });
    }),
  );
});

test('Fuzzy Search — Large 10K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search(largeQuery, largeItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexLarge.search(largeQuery, { maxResults: 10 });
    }),
    bench('fuse.js', () => {
      fuseLarge.search(largeQuery, { limit: 10 });
    }),
    bench('fuzzysort', () => {
      fuzzysort.go(largeQuery, fuzzysortLargePrepared, { limit: 10, threshold: 0 });
    }),
    bench('uFuzzy', () => {
      uf.search(largeItems, largeQuery);
    }),
    bench('FlexSearch', () => {
      flexLarge.search(largeQuery, { limit: 10 });
    }),
    bench('MiniSearch', () => {
      miniLarge.search(largeQuery, { fuzzy: 0.2, prefix: true });
    }),
  );
});

test('Fuzzy Search — Large 10K, rotating queries (vs competitors)', async ({ bench }) => {
  // One cycle per benchmark, so each one searches the same sequence of queries.
  const nextQuery = {
    rapidFuzzy: cycle(rotatingQueries),
    fuzzyIndex: cycle(rotatingQueries),
    fuse: cycle(rotatingQueries),
    fuzzysort: cycle(rotatingQueries),
    uFuzzy: cycle(rotatingQueries),
  };
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search(nextQuery.rapidFuzzy(), largeItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexLargeRotating.search(nextQuery.fuzzyIndex(), { maxResults: 10 });
    }),
    bench('fuse.js', () => {
      fuseLarge.search(nextQuery.fuse(), { limit: 10 });
    }),
    bench('fuzzysort', () => {
      fuzzysort.go(nextQuery.fuzzysort(), fuzzysortLargePrepared, { limit: 10, threshold: 0 });
    }),
    bench('uFuzzy', () => {
      uf.search(largeItems, nextQuery.uFuzzy());
    }),
  );
});

test('Fuzzy Search — Large 10K, type-ahead (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      for (const query of typeAheadQueries) search(query, largeItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      for (const query of typeAheadQueries) {
        fuzzyIndexLargeTypeAhead.search(query, { maxResults: 10 });
      }
    }),
    bench('fuse.js', () => {
      for (const query of typeAheadQueries) fuseLarge.search(query, { limit: 10 });
    }),
    bench('fuzzysort', () => {
      for (const query of typeAheadQueries) {
        fuzzysort.go(query, fuzzysortLargePrepared, { limit: 10, threshold: 0 });
      }
    }),
    bench('uFuzzy', () => {
      for (const query of typeAheadQueries) uf.search(largeItems, query);
    }),
  );
});

test('Fuzzy Search — Extra Large 50K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexXlarge.search(largeQuery, { maxResults: 10 });
    }),
    bench('fuzzysort', () => {
      fuzzysort.go(largeQuery, fuzzysortXlargePrepared, { limit: 10, threshold: 0 });
    }),
    bench('uFuzzy', () => {
      uf.search(xlargeItems, largeQuery);
    }),
    bench('FlexSearch', () => {
      flexXlarge.search(largeQuery, { limit: 10 });
    }),
    bench('MiniSearch', () => {
      miniXlarge.search(largeQuery, { fuzzy: 0.2, prefix: true });
    }),
  );
});

test('Fuzzy Search — Huge 100K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexHuge.search(largeQuery, { maxResults: 10 });
    }),
    bench('fuzzysort', () => {
      fuzzysort.go(largeQuery, fuzzysortHugePrepared, { limit: 10, threshold: 0 });
    }),
    bench('uFuzzy', () => {
      uf.search(hugeItems, largeQuery);
    }),
    bench('FlexSearch', () => {
      flexHuge.search(largeQuery, { limit: 10 });
    }),
    bench('MiniSearch', () => {
      miniHuge.search(largeQuery, { fuzzy: 0.2, prefix: true });
    }),
  );
});

test('Index Construction — Medium 1K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      new FuzzyIndex(mediumItems);
    }),
    bench('fuse.js', () => {
      new Fuse(mediumItems, { threshold: 0.4 });
    }),
    bench('fuzzysort (prepare)', () => {
      mediumItems.map((item) => fuzzysort.prepare(item));
    }),
    bench('FlexSearch', () => {
      createFlexSearch(mediumItems);
    }),
    bench('MiniSearch', () => {
      createMiniSearch(mediumItems);
    }),
  );
});

test('Index Construction — Large 10K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      new FuzzyIndex(largeItems);
    }),
    bench('fuse.js', () => {
      new Fuse(largeItems, { threshold: 0.4 });
    }),
    bench('fuzzysort (prepare)', () => {
      largeItems.map((item) => fuzzysort.prepare(item));
    }),
    bench('FlexSearch', () => {
      createFlexSearch(largeItems);
    }),
    bench('MiniSearch', () => {
      createMiniSearch(largeItems);
    }),
  );
});

test('Closest Match — Medium 1K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      closest(mediumClosestQuery, mediumItems);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexClosestMedium.closest(mediumClosestQuery);
    }),
    bench('fastest-levenshtein', () => {
      fastestLevenshteinClosest(mediumClosestQuery, mediumItems);
    }),
  );
});

test('Closest Match — Large 10K (vs competitors)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      closest(largeClosestQuery, largeItems);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexClosestLarge.closest(largeClosestQuery);
    }),
    bench('fastest-levenshtein', () => {
      fastestLevenshteinClosest(largeClosestQuery, largeItems);
    }),
  );
});
