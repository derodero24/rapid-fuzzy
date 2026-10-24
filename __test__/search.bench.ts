import { test } from 'vitest';
import { closest, FuzzyIndex, search } from '../index.js';
import * as fixtures from './bench-fixtures.js';

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

// Every query below matches items: a query that matches nothing measures an
// early exit, not a search.

// --- Pre-initialize search instances ---

const fuzzyIndexSmall = new FuzzyIndex(smallItems);
const fuzzyIndexMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexLarge = new FuzzyIndex(largeItems);
const fuzzyIndexLargeRotating = new FuzzyIndex(largeItems);
const fuzzyIndexLargeTypeAhead = new FuzzyIndex(largeItems);
const fuzzyIndexXlarge = new FuzzyIndex(xlargeItems);
const fuzzyIndexHuge = new FuzzyIndex(hugeItems);
const fuzzyIndexClosestMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexClosestLarge = new FuzzyIndex(largeItems);

test('Fuzzy Search — Small (20 items)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search('aple', smallItems, 5);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexSmall.search('aple', { maxResults: 5 });
    }),
  );
});

test('Fuzzy Search — Medium (1K items)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search('utils config', mediumItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexMedium.search('utils config', { maxResults: 10 });
    }),
  );
});

test('Fuzzy Search — Large (10K items)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search(largeQuery, largeItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexLarge.search(largeQuery, { maxResults: 10 });
    }),
  );
});

test('Fuzzy Search — Large (10K items), rotating queries', async ({ bench }) => {
  const nextSearchQuery = cycle(rotatingQueries);
  const nextIndexQuery = cycle(rotatingQueries);
  await bench.compare(
    bench('rapid-fuzzy', () => {
      search(nextSearchQuery(), largeItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexLargeRotating.search(nextIndexQuery(), { maxResults: 10 });
    }),
  );
});

test('Fuzzy Search — Large (10K items), type-ahead', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      for (const query of typeAheadQueries) search(query, largeItems, 10);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      for (const query of typeAheadQueries) {
        fuzzyIndexLargeTypeAhead.search(query, { maxResults: 10 });
      }
    }),
  );
});

test('Fuzzy Search — Extra Large (50K items)', async ({ bench }) => {
  await bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexXlarge.search(largeQuery, { maxResults: 10 });
  }).run();
});

test('Fuzzy Search — Huge (100K items)', async ({ bench }) => {
  await bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexHuge.search(largeQuery, { maxResults: 10 });
  }).run();
});

test('Index Construction — Medium (1K items)', async ({ bench }) => {
  await bench('rapid-fuzzy (FuzzyIndex)', () => {
    new FuzzyIndex(mediumItems);
  }).run();
});

test('Index Construction — Large (10K items)', async ({ bench }) => {
  await bench('rapid-fuzzy (FuzzyIndex)', () => {
    new FuzzyIndex(largeItems);
  }).run();
});

test('Closest Match — Medium (1K items)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      closest(mediumClosestQuery, mediumItems);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexClosestMedium.closest(mediumClosestQuery);
    }),
  );
});

test('Closest Match — Large (10K items)', async ({ bench }) => {
  await bench.compare(
    bench('rapid-fuzzy', () => {
      closest(largeClosestQuery, largeItems);
    }),
    bench('rapid-fuzzy (FuzzyIndex)', () => {
      fuzzyIndexClosestLarge.closest(largeClosestQuery);
    }),
  );
});
