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

let searchRotation = 0;
let indexRotation = 0;

describe('Fuzzy Search — Small (20 items)', () => {
  bench('rapid-fuzzy', () => {
    search('aple', smallItems, 5);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexSmall.search('aple', { maxResults: 5 });
  });
});

describe('Fuzzy Search — Medium (1K items)', () => {
  bench('rapid-fuzzy', () => {
    search('utils config', mediumItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexMedium.search('utils config', { maxResults: 10 });
  });
});

describe('Fuzzy Search — Large (10K items)', () => {
  bench('rapid-fuzzy', () => {
    search(largeQuery, largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexLarge.search(largeQuery, { maxResults: 10 });
  });
});

describe('Fuzzy Search — Large (10K items), rotating queries', () => {
  bench('rapid-fuzzy', () => {
    const query = rotatingQueries[searchRotation++ % rotatingQueries.length];
    search(query, largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    const query = rotatingQueries[indexRotation++ % rotatingQueries.length];
    fuzzyIndexLargeRotating.search(query, { maxResults: 10 });
  });
});

describe('Fuzzy Search — Large (10K items), type-ahead', () => {
  bench('rapid-fuzzy', () => {
    for (const query of typeAheadQueries) search(query, largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    for (const query of typeAheadQueries) {
      fuzzyIndexLargeTypeAhead.search(query, { maxResults: 10 });
    }
  });
});

describe('Fuzzy Search — Extra Large (50K items)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexXlarge.search(largeQuery, { maxResults: 10 });
  });
});

describe('Fuzzy Search — Huge (100K items)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexHuge.search(largeQuery, { maxResults: 10 });
  });
});

describe('Index Construction — Medium (1K items)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    new FuzzyIndex(mediumItems);
  });
});

describe('Index Construction — Large (10K items)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    new FuzzyIndex(largeItems);
  });
});

describe('Closest Match — Medium (1K items)', () => {
  bench('rapid-fuzzy', () => {
    closest(mediumClosestQuery, mediumItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestMedium.closest(mediumClosestQuery);
  });
});

describe('Closest Match — Large (10K items)', () => {
  bench('rapid-fuzzy', () => {
    closest(largeClosestQuery, largeItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestLarge.closest(largeClosestQuery);
  });
});
