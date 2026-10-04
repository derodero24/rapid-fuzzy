// Shared benchmark test data — deterministic and reusable across bench files.
// Changes to this file will trigger CodSpeed benchmarks.

// --- String pairs (for distance / similarity / ratio benchmarks) ---

/** Realistic string pairs of varying length and similarity */
export const pairs: [string, string][] = [
  ['kitten', 'sitting'],
  ['saturday', 'sunday'],
  ['rosettacode', 'raisethysword'],
  ['pneumonoultramicroscopicsilicovolcanoconiosis', 'ultramicroscopically'],
  ['the quick brown fox jumps over the lazy dog', 'the fast brown fox leaps over the lazy dog'],
  ['abcdefghijklmnopqrstuvwxyz', 'zyxwvutsrqponmlkjihgfedcba'],
];

/** Equal-length pairs for Hamming distance */
export const equalLengthPairs: [string, string][] = [
  ['karolin', 'kathrin'],
  ['saturday', 'sunturdy'],
  ['abcdefgh', 'abcdefgz'],
  ['10101010', '01010101'],
  ['the quick brown fox jumps', 'the swift brown fox leaps'],
  ['abcdefghijklmnopqrstuvwxyz', 'zyxwvutsrqponmlkjihgfedcba'],
];

// --- Candidate arrays (for batch / 1:N benchmarks) ---

/** 1K candidates for batch comparison benchmarks */
export const manyCandidates = Array.from({ length: 1_000 }, (_, i) => {
  const words = [
    'kitten',
    'sitting',
    'saturday',
    'sunday',
    'hello',
    'world',
    'fuzzy',
    'search',
    'match',
    'distance',
  ];
  return `${words[i % words.length]}${i}`;
});

// --- Datasets (for search benchmarks) ---

const SEARCH_WORDS = [
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

function generateSearchItems(n: number): string[] {
  return Array.from({ length: n }, (_, i) => {
    const w1 = SEARCH_WORDS[i % SEARCH_WORDS.length];
    const w2 = SEARCH_WORDS[(i * 7) % SEARCH_WORDS.length];
    return `${w1}_${w2}_${i}`;
  });
}

/** Small dataset — typical autocomplete (20 items) */
export const smallItems = [
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

/** Medium dataset — file finder (~1K items) */
export const mediumItems = Array.from({ length: 1_000 }, (_, i) => {
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

/** Large dataset (~10K items) */
export const largeItems = generateSearchItems(10_000);

/** Extra-large dataset (~50K items) */
export const xlargeItems = generateSearchItems(50_000);

/** Huge dataset (~100K items) */
export const hugeItems = generateSearchItems(100_000);

// --- Queries (for search benchmarks) ---

/** Query for the 10K-100K datasets: matches the 5% of items named `handler_repository_<n>`. */
export const largeQuery = 'handler repo';

/**
 * Queries cycled through one per iteration, so consecutive searches never
 * refine each other and every iteration scans the whole dataset. They mix
 * whole words and abbreviations, and each matches 5-10% of the 10K dataset in
 * rapid-fuzzy (result counts differ between libraries).
 */
export const rotatingQueries = [
  'handler repo',
  'middleware type',
  'observable',
  'service fac',
  'ctrl',
  'prms',
  'import',
  'repository',
];

/** Keystrokes of a user typing `largeQuery`, one search per keystroke. */
export const typeAheadQueries = Array.from({ length: largeQuery.length }, (_, i) =>
  largeQuery.slice(0, i + 1),
);

/** `closest()` query for the 1K dataset; its best match is `utils/index42.ts`. */
export const mediumClosestQuery = 'utils/index42';

/**
 * `closest()` query for the 10K dataset with a missing letter; its best match
 * is `handler_repository_514`.
 */
export const largeClosestQuery = 'handler_repositry_514';
