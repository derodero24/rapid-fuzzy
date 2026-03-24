// Competitor comparison benchmarks (local use only, excluded from CodSpeed CI)
import uFuzzy from '@leeoniya/ufuzzy';
import { closest as fastestLevenshteinClosest } from 'fastest-levenshtein';
import { Index as FlexSearchIndex } from 'flexsearch';
import Fuse from 'fuse.js';
import fuzzysort from 'fuzzysort';
import MiniSearch from 'minisearch';
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

// 100K dataset (scaling benchmark)
const hugeItems = Array.from({ length: 100_000 }, (_, i) => {
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

// --- Pre-initialize search instances ---

// Fuse.js
const fuseSmall = new Fuse(smallItems, { threshold: 0.4 });
const fuseMedium = new Fuse(mediumItems, { threshold: 0.4 });
const fuseLarge = new Fuse(largeItems, { threshold: 0.4 });

// fuzzysort (prepared targets)
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
    fuzzysort.go('aple', smallItems, { limit: 5 });
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
    fuzzysort.go('utils config', fuzzysortMediumPrepared, { limit: 10 });
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

  bench('FlexSearch', () => {
    flexLarge.search('handler middleware', { limit: 10 });
  });

  bench('MiniSearch', () => {
    miniLarge.search('handler middleware', { fuzzy: 0.2, prefix: true });
  });
});

describe('Fuzzy Search — Extra Large 50K (vs competitors)', () => {
  bench('fuzzysort', () => {
    fuzzysort.go('handler middleware', fuzzysortXlargePrepared, { limit: 10 });
  });

  bench('uFuzzy', () => {
    uf.search(xlargeItems, 'handler middleware');
  });

  bench('FlexSearch', () => {
    flexXlarge.search('handler middleware', { limit: 10 });
  });

  bench('MiniSearch', () => {
    miniXlarge.search('handler middleware', { fuzzy: 0.2, prefix: true });
  });
});

describe('Fuzzy Search — Huge 100K (vs competitors)', () => {
  bench('fuzzysort', () => {
    fuzzysort.go('handler middleware', fuzzysortHugePrepared, { limit: 10 });
  });

  bench('uFuzzy', () => {
    uf.search(hugeItems, 'handler middleware');
  });

  bench('FlexSearch', () => {
    flexHuge.search('handler middleware', { limit: 10 });
  });

  bench('MiniSearch', () => {
    miniHuge.search('handler middleware', { fuzzy: 0.2, prefix: true });
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
    closest('src/utils42.ts', mediumItems);
  });

  bench('rapid-fuzzy (FuzzyIndex)', () => {
    fuzzyIndexClosestMedium.closest('src/utils42.ts');
  });

  bench('fastest-levenshtein', () => {
    fastestLevenshteinClosest('src/utils42.ts', mediumItems);
  });
});

describe('Closest Match — Large 10K (vs competitors)', () => {
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
