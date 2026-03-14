import uFuzzy from '@leeoniya/ufuzzy';
import { closest as fastestLevenshteinClosest } from 'fastest-levenshtein';
import Fuse from 'fuse.js';
import fuzzysort from 'fuzzysort';
import { bench, describe } from 'vitest';

import { closest, FuzzyIndex, search } from '../index.js';

// --- Test data ---

// Small dataset (typical autocomplete)
const smallItems = [
  'apple',
  'banana',
  'grape',
  'orange',
  'pineapple',
  'mango',
  'strawberry',
  'blueberry',
  'raspberry',
  'watermelon',
  'kiwi',
  'peach',
  'plum',
  'cherry',
  'lemon',
  'lime',
  'coconut',
  'avocado',
  'papaya',
  'guava',
];

// Medium dataset (file finder, ~1K items)
const mediumItems = Array.from({ length: 1_000 }, (_, i) => {
  const dirs = ['src', 'lib', 'test', 'docs', 'utils', 'components', 'hooks', 'services'];
  const exts = ['.ts', '.js', '.tsx', '.jsx', '.json', '.md'];
  const names = [
    'index',
    'main',
    'utils',
    'helper',
    'config',
    'types',
    'schema',
    'handler',
    'service',
    'controller',
  ];
  const dir = dirs[i % dirs.length];
  const name = names[i % names.length];
  const ext = exts[i % exts.length];
  return `${dir}/${name}${Math.floor(i / 10)}${ext}`;
});

// Large dataset (~10K items)
const largeItems = Array.from({ length: 10_000 }, (_, i) => {
  const words = [
    'async',
    'await',
    'function',
    'class',
    'interface',
    'type',
    'export',
    'import',
    'const',
    'let',
    'return',
    'promise',
    'observable',
    'subscriber',
    'handler',
    'middleware',
    'controller',
    'service',
    'repository',
    'factory',
  ];
  return `${words[i % words.length]}_${words[(i * 7) % words.length]}_${i}`;
});

// Extra-large dataset (~50K items)
const xlargeItems = Array.from({ length: 50_000 }, (_, i) => {
  const words = [
    'async',
    'await',
    'function',
    'class',
    'interface',
    'type',
    'export',
    'import',
    'const',
    'let',
    'return',
    'promise',
    'observable',
    'subscriber',
    'handler',
    'middleware',
    'controller',
    'service',
    'repository',
    'factory',
  ];
  return `${words[i % words.length]}_${words[(i * 7) % words.length]}_${i}`;
});

// Pre-initialize Fuse instances (real-world usage pattern)
const fuseSmall = new Fuse(smallItems, { threshold: 0.4 });
const fuseMedium = new Fuse(mediumItems, { threshold: 0.4 });
const fuseLarge = new Fuse(largeItems, { threshold: 0.4 });

// Pre-prepare fuzzysort targets
const fuzzysortMediumPrepared = mediumItems.map((item) => fuzzysort.prepare(item));
const fuzzysortLargePrepared = largeItems.map((item) => fuzzysort.prepare(item));
const fuzzysortXlargePrepared = xlargeItems.map((item) => fuzzysort.prepare(item));

// Pre-initialize uFuzzy instance
const uf = new uFuzzy();

// Pre-initialize FuzzyIndex instances (recommended pattern for repeated searches)
const fuzzyIndexSmall = new FuzzyIndex(smallItems);
const fuzzyIndexMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexLarge = new FuzzyIndex(largeItems);
const fuzzyIndexXlarge = new FuzzyIndex(xlargeItems);
const fuzzyIndexClosestMedium = new FuzzyIndex(mediumItems);
const fuzzyIndexClosestLarge = new FuzzyIndex(largeItems);

describe('Fuzzy Search — Small (20 items)', () => {
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
    fuzzysort.go('aple', smallItems, { limit: 5 });
  });

  bench('uFuzzy', () => {
    uf.search(smallItems, 'aple');
  });
});

describe('Fuzzy Search — Medium (1K items)', () => {
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
    fuzzysort.go('utils config', fuzzysortMediumPrepared, { limit: 10 });
  });

  bench('uFuzzy', () => {
    uf.search(mediumItems, 'utils config');
  });
});

describe('Fuzzy Search — Large (10K items)', () => {
  bench('rapid-fuzzy', () => {
    search('handler middleware', largeItems, 10);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexLarge.search('handler middleware', { maxResults: 10 });
  });

  bench('fuse.js', () => {
    fuseLarge.search('handler middleware', { limit: 10 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go('handler middleware', fuzzysortLargePrepared, { limit: 10 });
  });

  bench('uFuzzy', () => {
    uf.search(largeItems, 'handler middleware');
  });
});

describe('Fuzzy Search — Extra Large (50K items)', () => {
  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexXlarge.search('handler middleware', { maxResults: 10 });
  });

  bench('fuzzysort', () => {
    fuzzysort.go('handler middleware', fuzzysortXlargePrepared, { limit: 10 });
  });

  bench('uFuzzy', () => {
    uf.search(xlargeItems, 'handler middleware');
  });
});

describe('Closest Match — Medium (1K items)', () => {
  bench('rapid-fuzzy', () => {
    closest('src/utils42.ts', mediumItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestMedium.closest('src/utils42.ts');
  });

  bench('fastest-levenshtein', () => {
    fastestLevenshteinClosest('src/utils42.ts', mediumItems);
  });
});

describe('Closest Match — Large (10K items)', () => {
  bench('rapid-fuzzy', () => {
    closest('handler_middleware_500', largeItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestLarge.closest('handler_middleware_500');
  });

  bench('fastest-levenshtein', () => {
    fastestLevenshteinClosest('handler_middleware_500', largeItems);
  });
});
