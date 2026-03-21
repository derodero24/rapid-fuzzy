import { bench, describe } from 'vitest';
import { closest, FuzzyIndex, search } from '../index.js';
import { hugeItems, largeItems, mediumItems, smallItems, xlargeItems } from './bench-fixtures.js';

// --- Pre-initialize search instances ---

const fuzzyIndexSmall = new FuzzyIndex(smallItems);
const fuzzyIndexMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexLarge = new FuzzyIndex(largeItems);
const fuzzyIndexXlarge = new FuzzyIndex(xlargeItems);
const fuzzyIndexHuge = new FuzzyIndex(hugeItems);
const fuzzyIndexClosestMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexClosestLarge = new FuzzyIndex(largeItems);

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
    search('handler middleware', largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexLarge.search('handler middleware', { maxResults: 10 });
  });
});

describe('Fuzzy Search — Extra Large (50K items)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexXlarge.search('handler middleware', { maxResults: 10 });
  });
});

describe('Fuzzy Search — Huge (100K items)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexHuge.search('handler middleware', { maxResults: 10 });
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
    closest('src/utils42.ts', mediumItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestMedium.closest('src/utils42.ts');
  });
});

describe('Closest Match — Large (10K items)', () => {
  bench('rapid-fuzzy', () => {
    closest('handler_middleware_500', largeItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestLarge.closest('handler_middleware_500');
  });
});
