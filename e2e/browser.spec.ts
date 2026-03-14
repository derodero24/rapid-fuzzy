import { expect, test } from '@playwright/test';

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
    const allPresent = await page.evaluate(() => window.__results.allExportsPresent);
    const missing = await page.evaluate(() => window.__results.missingExports);
    expect(allPresent).toBe(true);
    expect(missing).toEqual([]);
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
    expect(results[0].item).toBe('TypeScript');
    expect(results[0]).toHaveProperty('score');
    expect(results[0]).toHaveProperty('index');
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
