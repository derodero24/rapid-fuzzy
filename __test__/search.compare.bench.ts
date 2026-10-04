// Competitor comparison benchmarks (local use only, excluded from CodSpeed CI)
import uFuzzy from '@leeoniya/ufuzzy';
import { closest as fastestLevenshteinClosest } from 'fastest-levenshtein';
import { Index as FlexSearchIndex } from 'flexsearch';
import Fuse from 'fuse.js';
import fuzzysort from 'fuzzysort';
import MiniSearch from 'minisearch';
import { bench, describe } from 'vitest';

import { closest, FuzzyIndex, search } from '../index.js';
import {
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
} from './bench-fixtures.js';

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
const flexSmall = new FlexSearchIndex();
for (let i = 0; i < smallItems.length; i++) flexSmall.add(i, smallItems[i]);
const flexMedium = new FlexSearchIndex();
for (let i = 0; i < mediumItems.length; i++) flexMedium.add(i, mediumItems[i]);
const flexLarge = new FlexSearchIndex();
for (let i = 0; i < largeItems.length; i++) flexLarge.add(i, largeItems[i]);
const flexXlarge = new FlexSearchIndex();
for (let i = 0; i < xlargeItems.length; i++) flexXlarge.add(i, xlargeItems[i]);
const flexHuge = new FlexSearchIndex();
for (let i = 0; i < hugeItems.length; i++) flexHuge.add(i, hugeItems[i]);

// MiniSearch
function createMiniSearch(items: string[]) {
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

describe('Fuzzy Search — Small 20 (vs competitors)', () => {
  bench('rapid-fuzzy', () => {
    search('aple', smallItems, 5);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexSmall.search('aple', { maxResults: 5 });
  });

  bench('fuse.js', () => {
    fuseSmall.search('aple', { limit: 5 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go('aple', smallItems, { limit: 5, threshold: 0 });
  });

  bench('uFuzzy', () => {
    uf.search(smallItems, 'aple');
  });

  bench('FlexSearch', () => {
    flexSmall.search('aple', { limit: 5 });
  });

  bench('MiniSearch', () => {
    miniSmall.search('aple', { fuzzy: 0.2, prefix: true });
  });
});

describe('Fuzzy Search — Medium 1K (vs competitors)', () => {
  bench('rapid-fuzzy', () => {
    search('utils config', mediumItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexMedium.search('utils config', { maxResults: 10 });
  });

  bench('fuse.js', () => {
    fuseMedium.search('utils config', { limit: 10 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go('utils config', fuzzysortMediumPrepared, { limit: 10, threshold: 0 });
  });

  bench('uFuzzy', () => {
    uf.search(mediumItems, 'utils config');
  });

  bench('FlexSearch', () => {
    flexMedium.search('utils config', { limit: 10 });
  });

  bench('MiniSearch', () => {
    miniMedium.search('utils config', { fuzzy: 0.2, prefix: true });
  });
});

describe('Fuzzy Search — Large 10K (vs competitors)', () => {
  bench('rapid-fuzzy', () => {
    search(largeQuery, largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexLarge.search(largeQuery, { maxResults: 10 });
  });

  bench('fuse.js', () => {
    fuseLarge.search(largeQuery, { limit: 10 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go(largeQuery, fuzzysortLargePrepared, { limit: 10, threshold: 0 });
  });

  bench('uFuzzy', () => {
    uf.search(largeItems, largeQuery);
  });

  bench('FlexSearch', () => {
    flexLarge.search(largeQuery, { limit: 10 });
  });

  bench('MiniSearch', () => {
    miniLarge.search(largeQuery, { fuzzy: 0.2, prefix: true });
  });
});

describe('Fuzzy Search — Large 10K, rotating queries (vs competitors)', () => {
  const next = (() => {
    const counters = new Map<string, number>();
    return (name: string): string => {
      const n = counters.get(name) ?? 0;
      counters.set(name, n + 1);
      return rotatingQueries[n % rotatingQueries.length];
    };
  })();

  bench('rapid-fuzzy', () => {
    search(next('rapid-fuzzy'), largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexLargeRotating.search(next('FuzzyIndex'), { maxResults: 10 });
  });

  bench('fuse.js', () => {
    fuseLarge.search(next('fuse.js'), { limit: 10 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go(next('fuzzysort'), fuzzysortLargePrepared, { limit: 10, threshold: 0 });
  });

  bench('uFuzzy', () => {
    uf.search(largeItems, next('uFuzzy'));
  });
});

describe('Fuzzy Search — Large 10K, type-ahead (vs competitors)', () => {
  bench('rapid-fuzzy', () => {
    for (const query of typeAheadQueries) search(query, largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    for (const query of typeAheadQueries) {
      fuzzyIndexLargeTypeAhead.search(query, { maxResults: 10 });
    }
  });

  bench('fuse.js', () => {
    for (const query of typeAheadQueries) fuseLarge.search(query, { limit: 10 });
  });

  bench('fuzzysort', () => {
    for (const query of typeAheadQueries) {
      fuzzysort.go(query, fuzzysortLargePrepared, { limit: 10, threshold: 0 });
    }
  });

  bench('uFuzzy', () => {
    for (const query of typeAheadQueries) uf.search(largeItems, query);
  });
});

describe('Fuzzy Search — Extra Large 50K (vs competitors)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexXlarge.search(largeQuery, { maxResults: 10 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go(largeQuery, fuzzysortXlargePrepared, { limit: 10, threshold: 0 });
  });

  bench('uFuzzy', () => {
    uf.search(xlargeItems, largeQuery);
  });

  bench('FlexSearch', () => {
    flexXlarge.search(largeQuery, { limit: 10 });
  });

  bench('MiniSearch', () => {
    miniXlarge.search(largeQuery, { fuzzy: 0.2, prefix: true });
  });
});

describe('Fuzzy Search — Huge 100K (vs competitors)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexHuge.search(largeQuery, { maxResults: 10 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go(largeQuery, fuzzysortHugePrepared, { limit: 10, threshold: 0 });
  });

  bench('uFuzzy', () => {
    uf.search(hugeItems, largeQuery);
  });

  bench('FlexSearch', () => {
    flexHuge.search(largeQuery, { limit: 10 });
  });

  bench('MiniSearch', () => {
    miniHuge.search(largeQuery, { fuzzy: 0.2, prefix: true });
  });
});

describe('Index Construction — Medium 1K (vs competitors)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    new FuzzyIndex(mediumItems);
  });

  bench('fuse.js', () => {
    new Fuse(mediumItems, { threshold: 0.4 });
  });

  bench('fuzzysort (prepare)', () => {
    mediumItems.map((item) => fuzzysort.prepare(item));
  });

  bench('FlexSearch', () => {
    const idx = new FlexSearchIndex();
    for (let i = 0; i < mediumItems.length; i++) idx.add(i, mediumItems[i]);
  });

  bench('MiniSearch', () => {
    createMiniSearch(mediumItems);
  });
});

describe('Index Construction — Large 10K (vs competitors)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    new FuzzyIndex(largeItems);
  });

  bench('fuse.js', () => {
    new Fuse(largeItems, { threshold: 0.4 });
  });

  bench('fuzzysort (prepare)', () => {
    largeItems.map((item) => fuzzysort.prepare(item));
  });

  bench('FlexSearch', () => {
    const idx = new FlexSearchIndex();
    for (let i = 0; i < largeItems.length; i++) idx.add(i, largeItems[i]);
  });

  bench('MiniSearch', () => {
    createMiniSearch(largeItems);
  });
});

describe('Closest Match — Medium 1K (vs competitors)', () => {
  bench('rapid-fuzzy', () => {
    closest(mediumClosestQuery, mediumItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestMedium.closest(mediumClosestQuery);
  });

  bench('fastest-levenshtein', () => {
    fastestLevenshteinClosest(mediumClosestQuery, mediumItems);
  });
});

describe('Closest Match — Large 10K (vs competitors)', () => {
  bench('rapid-fuzzy', () => {
    closest(largeClosestQuery, largeItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestLarge.closest(largeClosestQuery);
  });

  bench('fastest-levenshtein', () => {
    fastestLevenshteinClosest(largeClosestQuery, largeItems);
  });
});
