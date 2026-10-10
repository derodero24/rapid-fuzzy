import { expect, test } from '@playwright/test';

// The page is a consumer app that imports the packed package through Vite
// (see e2e/serve-browser-app.mjs); each project runs it from a production
// build and from the dev server.
test.beforeEach(async ({ page }) => {
  await page.goto('/');
  // Wait for WASM to load (up to 30s)
  await page.waitForFunction(() => window.__ready === true || window.__error !== undefined, {
    timeout: 30_000,
  });
  const error = await page.evaluate(() => window.__error);
  if (error) {
    throw new Error(`WASM module failed to load: ${error}`);
  }
});

test.describe('exports', () => {
  test('all expected functions are exported', async ({ page }) => {
    const exported = await page.evaluate(() => window.__results.exports);
    const expected = [
      'closest',
      'search',
      'searchKeys',
      'FuzzyIndex',
      'KeyedFuzzyIndex',
      'MatchType',
      'highlight',
      'highlightRanges',
      'searchObjects',
      'FuzzyObjectIndex',
      ...['levenshtein', 'normalizedLevenshtein', 'damerauLevenshtein', 'hamming', 'indel']
        .concat(['normalizedIndel', 'normalizedHamming', 'jaro', 'jaroWinkler', 'sorensenDice'])
        .concat(['tokenSortRatio', 'tokenSetRatio', 'partialRatio', 'weightedRatio'])
        .flatMap((name) => [name, `${name}Batch`, `${name}Many`]),
      ...['levenshtein', 'damerauLevenshtein', 'indel', 'hamming'].map((n) => `${n}ManyU32`),
      ...['jaro', 'jaroWinkler', 'sorensenDice', 'normalizedLevenshtein', 'normalizedIndel']
        .concat(['tokenSortRatio', 'tokenSetRatio', 'partialRatio', 'weightedRatio'])
        .concat(['normalizedHamming'])
        .map((n) => `${n}ManyF64`),
    ];
    expect(exported).toEqual(expected.sort());
  });
});

test.describe('package entry points', () => {
  test('the bundler serves the .wasm referenced by the browser entry', async ({ page }) => {
    const requests = await page.evaluate(() => window.__results.wasmRequests);
    expect(requests).toHaveLength(1);
  });

  test('MatchType and the TypedArray variants', async ({ page }) => {
    const results = await page.evaluate(() => window.__results);
    expect(results.matchType).toBe('Prefix');
    expect(results.matchTypeEnum).toEqual({
      Exact: 'Exact',
      Prefix: 'Prefix',
      Contains: 'Contains',
      Fuzzy: 'Fuzzy',
    });
    expect(results.levenshteinManyU32).toEqual({ typed: true, values: [0, 4] });
    expect(results.hammingManyU32).toEqual([0, 0xffffffff]);
  });

  test('highlight from the main entry and rapid-fuzzy/highlight', async ({ page }) => {
    const results = await page.evaluate(() => window.__results);
    expect(results.highlight).toBe('<b>f</b>uz<b>zy</b>');
    expect(results.highlightSubpath).toBe('[f]uz[zy]');
    expect(results.highlightRanges).toEqual([
      { start: 0, end: 1, matched: true },
      { start: 1, end: 3, matched: false },
      { start: 3, end: 5, matched: true },
    ]);
  });

  test('object search from rapid-fuzzy/objects', async ({ page }) => {
    const results = await page.evaluate(() => window.__results);
    expect(results.searchObjects).toEqual(['jane@example.com']);
    expect(results.objectIndexSearch[0]).toBe('John Smith');
    expect(results.objectIndexSearch).toContain('Johnny Cash');
    expect(results.objectIndexClosest).toBe('Jane Doe');
    expect(results.objectIndexClosestOptions).toEqual([null, 'John Smith', null, 'Jane Doe']);
    expect(results.objectIndexSize).toBe(3);
    expect(results.objectsFromMainEntry).toBe(true);
  });
});

test.describe('distance functions', () => {
  test('levenshtein', async ({ page }) => {
    const dist = await page.evaluate(() => window.__results.levenshtein);
    expect(dist).toBe(3);
  });

  test('levenshtein identical strings', async ({ page }) => {
    const dist = await page.evaluate(() => window.__results.levenshteinIdentical);
    expect(dist).toBe(0);
  });

  test('normalizedLevenshtein', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.normalizedLevenshtein);
    expect(score).toBe(1.0);
  });

  test('damerauLevenshtein', async ({ page }) => {
    const dist = await page.evaluate(() => window.__results.damerauLevenshtein);
    expect(dist).toBe(1);
  });

  test('jaro', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.jaro);
    expect(score).toBe(1.0);
  });

  test('jaroWinkler', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.jaroWinkler);
    expect(score).toBe(1.0);
  });

  test('sorensenDice', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.sorensenDice);
    expect(score).toBe(1.0);
  });

  test('hamming', async ({ page }) => {
    const dist = await page.evaluate(() => window.__results.hamming);
    expect(dist).toBe(3);
  });

  test('hamming returns null for different lengths', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.hammingNull);
    // wasm-bindgen single functions return JsValue::NULL which Playwright preserves as null
    expect(result).toBeNull();
  });

  test('indel', async ({ page }) => {
    const dist = await page.evaluate(() => window.__results.indel);
    expect(dist).toBe(1);
  });

  test('normalizedIndel', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.normalizedIndel);
    expect(score).toBe(1.0);
  });

  test('normalizedHamming', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.normalizedHamming);
    expect(score).toBe(1.0);
  });

  test('normalizedHamming returns null for different lengths', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.normalizedHammingNull);
    expect(result).toBeNull();
  });
});

test.describe('batch functions', () => {
  test('levenshteinBatch', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.levenshteinBatch);
    expect(result).toEqual([0, 4]);
  });

  test('jaroBatch', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.jaroBatch);
    expect(result).toHaveLength(2);
    expect(result[0]).toBe(1.0);
    expect(result[1]).toBe(0.0);
  });

  test('hammingBatch', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.hammingBatch);
    expect(result).toEqual([0, 3]);
  });

  test('indelBatch', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.indelBatch);
    expect(result).toEqual([0, 1]);
  });

  test('normalizedIndelBatch', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.normalizedIndelBatch);
    expect(result).toHaveLength(2);
    expect(result[0]).toBe(1.0);
  });

  test('normalizedHammingBatch', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.normalizedHammingBatch);
    expect(result).toHaveLength(2);
    expect(result[0]).toBe(1.0);
  });
});

test.describe('many functions', () => {
  test('levenshteinMany', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.levenshteinMany);
    expect(result).toHaveLength(3);
    expect(result[0]).toBe(0);
  });

  test('jaroMany', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.jaroMany);
    expect(result).toHaveLength(2);
    expect(result[0]).toBe(1.0);
  });

  test('hammingMany', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.hammingMany);
    expect(result).toHaveLength(3);
    expect(result[0]).toBe(0);
    // Length mismatches are null, as in the Node.js binding
    expect(result[2]).toBeNull();
  });

  test('indelMany', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.indelMany);
    expect(result).toHaveLength(3);
    expect(result[0]).toBe(0);
    expect(result[1]).toBe(1);
  });

  test('normalizedIndelMany', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.normalizedIndelMany);
    expect(result).toHaveLength(2);
    expect(result[0]).toBe(1.0);
  });

  test('normalizedHammingMany', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.normalizedHammingMany);
    expect(result).toHaveLength(3);
    expect(result[0]).toBe(1.0);
    // Length mismatches are null, as in the Node.js binding
    expect(result[2]).toBeNull();
  });

  test('thresholds that are not numbers throw a TypeError', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.thresholdTypeErrors);
    expect(result).toEqual([
      'TypeError: maxDistance must be a number, got string',
      'TypeError: maxDistance must be a number, got string',
      'TypeError: maxDistance must be a number, got boolean',
      'TypeError: minSimilarity must be a number, got string',
      'TypeError: minScore must be a number, got boolean',
      'TypeError: minScore must be a number, got object',
    ]);
  });
});

test.describe('token-based functions', () => {
  test('tokenSortRatio', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.tokenSortRatio);
    expect(score).toBe(1.0);
  });

  test('tokenSetRatio', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.tokenSetRatio);
    expect(score).toBe(1.0);
  });

  test('partialRatio', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.partialRatio);
    expect(score).toBe(1.0);
  });

  test('weightedRatio', async ({ page }) => {
    const score = await page.evaluate(() => window.__results.weightedRatio);
    expect(score).toBe(1.0);
  });
});

test.describe('search', () => {
  test('search returns sorted results', async ({ page }) => {
    const results = await page.evaluate(() => window.__results.search);
    expect(results.length).toBeGreaterThan(0);
    // First result should be one of the "Type*" prefix matches
    expect(results[0].item).toMatch(/^Type/);
    expect(results[0]).toHaveProperty('score');
    expect(results[0]).toHaveProperty('index');
    // Results should be sorted by score descending
    for (let i = 1; i < results.length; i++) {
      expect(results[i].score).toBeLessThanOrEqual(results[i - 1].score);
    }
  });

  test('search returns empty for empty query', async ({ page }) => {
    const results = await page.evaluate(() => window.__results.searchEmpty);
    expect(results).toEqual([]);
  });
});

test.describe('closest', () => {
  test('closest returns best match', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.closest);
    expect(result).not.toBeNull();
  });

  test('closest returns null for empty items', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.closestEmpty);
    expect(result).toBeNull();
  });
});

test.describe('FuzzyIndex', () => {
  test('constructor and size', async ({ page }) => {
    const size = await page.evaluate(() => window.__results.indexSize);
    expect(size).toBe(4);
  });

  test('search returns results', async ({ page }) => {
    const results = await page.evaluate(() => window.__results.indexSearch);
    expect(results.length).toBeGreaterThan(0);
    expect(results.some((r: { item: string }) => r.item === 'apple')).toBe(true);
  });

  test('closest returns best match', async ({ page }) => {
    const result = await page.evaluate(() => window.__results.indexClosest);
    expect(result).toBe('apple');
  });

  test('add increases size', async ({ page }) => {
    const size = await page.evaluate(() => window.__results.indexSizeAfterAdd);
    expect(size).toBe(5);
  });

  test('destroy clears index', async ({ page }) => {
    const size = await page.evaluate(() => window.__results.indexSizeAfterDestroy);
    expect(size).toBe(0);
  });
});
