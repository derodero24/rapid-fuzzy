import { describe, expect, it } from 'vitest';

import {
  closest,
  damerauLevenshtein,
  damerauLevenshteinBatch,
  damerauLevenshteinMany,
  FuzzyIndex,
  hamming,
  hammingBatch,
  hammingMany,
  jaro,
  jaroBatch,
  jaroMany,
  jaroWinkler,
  jaroWinklerBatch,
  jaroWinklerMany,
  levenshtein,
  levenshteinBatch,
  levenshteinMany,
  normalizedLevenshtein,
  normalizedLevenshteinBatch,
  normalizedLevenshteinMany,
  partialRatio,
  partialRatioBatch,
  partialRatioMany,
  search,
  searchKeys,
  sorensenDice,
  sorensenDiceBatch,
  sorensenDiceMany,
  tokenSetRatio,
  tokenSetRatioBatch,
  tokenSetRatioMany,
  tokenSortRatio,
  tokenSortRatioBatch,
  tokenSortRatioMany,
  weightedRatio,
  weightedRatioBatch,
  weightedRatioMany,
} from '../index.js';

describe('distance', () => {
  describe('levenshtein', () => {
    it('should return 0 for identical strings', () => {
      expect(levenshtein('hello', 'hello')).toBe(0);
    });

    it('should compute edit distance', () => {
      expect(levenshtein('kitten', 'sitting')).toBe(3);
    });

    it('should handle empty strings', () => {
      expect(levenshtein('', 'abc')).toBe(3);
      expect(levenshtein('abc', '')).toBe(3);
      expect(levenshtein('', '')).toBe(0);
    });
  });

  describe('damerauLevenshtein', () => {
    it('should handle transpositions as single edit', () => {
      // "ab" -> "ba" is 1 transposition (Damerau), but 2 edits (standard Levenshtein)
      expect(damerauLevenshtein('ab', 'ba')).toBe(1);
      expect(levenshtein('ab', 'ba')).toBe(2);
    });

    it('should compute distance', () => {
      expect(damerauLevenshtein('kitten', 'sitting')).toBe(3);
    });
  });

  describe('jaro', () => {
    it('should return 1.0 for identical strings', () => {
      expect(jaro('hello', 'hello')).toBe(1.0);
    });

    it('should return 0.0 for completely different strings', () => {
      expect(jaro('abc', 'xyz')).toBe(0.0);
    });

    it('should return value between 0 and 1', () => {
      const score = jaro('martha', 'marhta');
      expect(score).toBeGreaterThan(0);
      expect(score).toBeLessThanOrEqual(1);
    });
  });

  describe('jaroWinkler', () => {
    it('should give prefix bonus over jaro', () => {
      const jaroScore = jaro('martha', 'marhta');
      const jwScore = jaroWinkler('martha', 'marhta');
      expect(jwScore).toBeGreaterThanOrEqual(jaroScore);
    });

    it('should return 1.0 for identical strings', () => {
      expect(jaroWinkler('test', 'test')).toBe(1.0);
    });
  });

  describe('sorensenDice', () => {
    it('should return 1.0 for identical strings', () => {
      expect(sorensenDice('night', 'night')).toBe(1.0);
    });

    it('should return 0.0 for completely different strings', () => {
      expect(sorensenDice('ab', 'yz')).toBe(0.0);
    });

    it('should compute bigram similarity', () => {
      const score = sorensenDice('night', 'nacht');
      expect(score).toBeGreaterThan(0);
      expect(score).toBeLessThan(1);
    });
  });

  describe('normalizedLevenshtein', () => {
    it('should return 1.0 for identical strings', () => {
      expect(normalizedLevenshtein('hello', 'hello')).toBe(1.0);
    });

    it('should return 0.0 for completely different strings of same length', () => {
      expect(normalizedLevenshtein('abc', 'xyz')).toBe(0.0);
    });

    it('should return value between 0 and 1', () => {
      const score = normalizedLevenshtein('kitten', 'sitting');
      expect(score).toBeGreaterThan(0);
      expect(score).toBeLessThan(1);
    });
  });

  describe('hamming', () => {
    it('should return 0 for identical strings', () => {
      expect(hamming('hello', 'hello')).toBe(0);
    });

    it('should count differing positions', () => {
      expect(hamming('karolin', 'kathrin')).toBe(3);
    });

    it('should return null for different-length strings', () => {
      expect(hamming('hello', 'hi')).toBeNull();
      expect(hamming('ab', 'abc')).toBeNull();
    });

    it('should handle empty strings', () => {
      expect(hamming('', '')).toBe(0);
    });
  });
});

describe('batch distance', () => {
  describe('levenshteinBatch', () => {
    it('should compute distances for multiple pairs', () => {
      const result = levenshteinBatch([
        ['kitten', 'sitting'],
        ['', ''],
        ['abc', 'abc'],
      ]);
      expect(result).toEqual([3, 0, 0]);
    });

    it('should return empty array for empty input', () => {
      expect(levenshteinBatch([])).toEqual([]);
    });
  });

  describe('levenshteinMany', () => {
    it('should compute distances from one string to many candidates', () => {
      const result = levenshteinMany('kitten', ['sitting', '', 'kitten']);
      expect(result).toEqual([3, 6, 0]);
    });

    it('should return empty array for empty candidates', () => {
      expect(levenshteinMany('hello', [])).toEqual([]);
    });
  });

  describe('damerauLevenshteinBatch', () => {
    it('should compute distances for multiple pairs', () => {
      const result = damerauLevenshteinBatch([
        ['ab', 'ba'],
        ['abc', 'abc'],
      ]);
      expect(result).toEqual([1, 0]);
    });
  });

  describe('damerauLevenshteinMany', () => {
    it('should compute distances from one string to many candidates', () => {
      const result = damerauLevenshteinMany('ab', ['ba', 'ab', 'xyz']);
      expect(result).toEqual([1, 0, 3]);
    });
  });

  describe('jaroBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = jaroBatch([
        ['hello', 'hello'],
        ['abc', 'xyz'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBe(0.0);
    });
  });

  describe('jaroMany', () => {
    it('should compute scores from one string to many candidates', () => {
      const result = jaroMany('hello', ['hello', 'world']);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeGreaterThan(0);
    });
  });

  describe('jaroWinklerBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = jaroWinklerBatch([
        ['test', 'test'],
        ['martha', 'marhta'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeGreaterThan(0.96);
    });
  });

  describe('jaroWinklerMany', () => {
    it('should compute scores from one string to many candidates', () => {
      const result = jaroWinklerMany('test', ['test', 'text']);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeGreaterThan(0);
    });
  });

  describe('sorensenDiceBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = sorensenDiceBatch([
        ['night', 'night'],
        ['ab', 'yz'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBe(0.0);
    });
  });

  describe('sorensenDiceMany', () => {
    it('should compute scores from one string to many candidates', () => {
      const result = sorensenDiceMany('night', ['night', 'nacht']);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeGreaterThan(0);
    });
  });

  describe('normalizedLevenshteinBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = normalizedLevenshteinBatch([
        ['hello', 'hello'],
        ['abc', 'xyz'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBe(0.0);
    });
  });

  describe('normalizedLevenshteinMany', () => {
    it('should compute scores from one string to many candidates', () => {
      const result = normalizedLevenshteinMany('hello', ['hello', 'world']);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeGreaterThan(0);
      expect(result[1]).toBeLessThan(1);
    });
  });

  describe('hammingBatch', () => {
    it('should compute distances for multiple pairs', () => {
      const result = hammingBatch([
        ['hello', 'hello'],
        ['hello', 'world'],
      ]);
      expect(result).toEqual([0, 4]);
    });

    it('should return null for different-length pairs', () => {
      const result = hammingBatch([
        ['hello', 'hi'],
        ['abc', 'abc'],
      ]);
      expect(result[0]).toBeNull();
      expect(result[1]).toBe(0);
    });
  });

  describe('hammingMany', () => {
    it('should compute distances from one string to many candidates', () => {
      const result = hammingMany('hello', ['hello', 'world', 'hi']);
      expect(result[0]).toBe(0);
      expect(result[1]).toBe(4);
      expect(result[2]).toBeNull();
    });
  });

  describe('_many threshold parameters', () => {
    it('levenshteinMany should support maxDistance', () => {
      const result = levenshteinMany('kitten', ['kitten', 'sitting', 'abcdef'], 2);
      expect(result[0]).toBe(0); // exact match
      expect(result[1]).toBe(3); // exceeds threshold → maxDistance + 1
      expect(result[2]).toBe(3); // exceeds threshold → maxDistance + 1
    });

    it('damerauLevenshteinMany should support maxDistance', () => {
      const result = damerauLevenshteinMany('ab', ['ab', 'ba', 'xyz'], 1);
      expect(result[0]).toBe(0); // exact match
      expect(result[1]).toBe(1); // within threshold
      expect(result[2]).toBe(2); // exceeds → maxDistance + 1
    });

    it('jaroMany should support minSimilarity', () => {
      const result = jaroMany('hello', ['hello', 'world'], 0.9);
      expect(result[0]).toBe(1.0); // exact match passes
      expect(result[1]).toBe(0.0); // below threshold → 0.0
    });

    it('jaroWinklerMany should support minSimilarity', () => {
      const result = jaroWinklerMany('hello', ['hello', 'world'], 0.9);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBe(0.0);
    });

    it('normalizedLevenshteinMany should support minSimilarity', () => {
      const result = normalizedLevenshteinMany('hello', ['hello', 'world'], 0.9);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBe(0.0);
    });
  });
});

describe('search', () => {
  const items = ['apple', 'banana', 'grape', 'orange', 'pineapple', 'mango'];

  it('should find fuzzy matches', () => {
    const results = search('aple', items);
    expect(results.length).toBeGreaterThan(0);
    expect(results.some((r) => r.item === 'apple')).toBe(true);
  });

  it('should return results sorted by score (best first)', () => {
    const results = search('an', items);
    for (let i = 1; i < results.length; i++) {
      const prev = results[i - 1];
      const curr = results[i];
      if (prev && curr) {
        expect(prev.score).toBeGreaterThanOrEqual(curr.score);
      }
    }
  });

  it('should respect maxResults', () => {
    const results = search('a', items, 2);
    expect(results.length).toBeLessThanOrEqual(2);
  });

  it('should return empty array for no matches', () => {
    const results = search('zzzzz', items);
    expect(results).toEqual([]);
  });

  it('should include index in results', () => {
    const results = search('banana', items);
    expect(results.length).toBeGreaterThan(0);
    const bananaResult = results.find((r) => r.item === 'banana');
    expect(bananaResult).toBeDefined();
    expect(bananaResult?.index).toBe(1);
  });

  it('should handle empty items array', () => {
    const results = search('test', []);
    expect(results).toEqual([]);
  });

  it('should return scores in 0.0-1.0 range', () => {
    const results = search('app', items);
    for (const r of results) {
      expect(r.score).toBeGreaterThanOrEqual(0);
      expect(r.score).toBeLessThanOrEqual(1);
    }
  });

  it('should return score of 1.0 for exact match', () => {
    const results = search('apple', items);
    const exact = results.find((r) => r.item === 'apple');
    expect(exact).toBeDefined();
    expect(exact?.score).toBeCloseTo(1.0);
  });

  it('should return lower scores for partial matches', () => {
    const results = search('apple', ['apple', 'pineapple', 'application']);
    const exact = results.find((r) => r.item === 'apple');
    const partial = results.find((r) => r.item === 'pineapple');
    expect(exact).toBeDefined();
    expect(partial).toBeDefined();
    if (exact && partial) {
      expect(exact.score).toBeGreaterThan(partial.score);
    }
  });

  it('should accept SearchOptions object', () => {
    const results = search('a', items, { maxResults: 2 });
    expect(results.length).toBeLessThanOrEqual(2);
  });

  it('should filter by minScore', () => {
    const all = search('apple', items);
    const filtered = search('apple', items, { minScore: 0.5 });
    expect(filtered.length).toBeLessThanOrEqual(all.length);
    for (const r of filtered) {
      expect(r.score).toBeGreaterThanOrEqual(0.5);
    }
  });

  it('should return only exact matches with minScore 1.0', () => {
    const results = search('apple', ['apple', 'application', 'banana'], {
      minScore: 1.0,
    });
    expect(results.length).toBe(1);
    expect(results[0]?.item).toBe('apple');
  });

  it('should combine maxResults and minScore', () => {
    const results = search('app', items, { maxResults: 1, minScore: 0.1 });
    expect(results.length).toBeLessThanOrEqual(1);
    if (results.length > 0) {
      expect(results[0]?.score).toBeGreaterThanOrEqual(0.1);
    }
  });

  it('should treat number arg as maxResults (backward compat)', () => {
    const withNumber = search('a', items, 2);
    const withOptions = search('a', items, { maxResults: 2 });
    expect(withNumber.length).toBe(withOptions.length);
  });

  describe('match positions', () => {
    it('should return positions when includePositions is true', () => {
      const results = search('hello', ['hello world'], {
        includePositions: true,
      });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.positions.length).toBeGreaterThan(0);
    });

    it('should return empty positions by default', () => {
      const results = search('hello', ['hello world']);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.positions).toEqual([]);
    });

    it('should return empty positions when includePositions is false', () => {
      const results = search('hello', ['hello world'], {
        includePositions: false,
      });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.positions).toEqual([]);
    });

    it('should return all positions for exact match', () => {
      const results = search('hello', ['hello'], { includePositions: true });
      expect(results.length).toBe(1);
      expect(results[0]?.positions).toEqual([0, 1, 2, 3, 4]);
    });

    it('should return sorted positions', () => {
      const results = search('hlo', ['hello world'], {
        includePositions: true,
      });
      expect(results.length).toBeGreaterThan(0);
      const positions = results[0]?.positions ?? [];
      for (let i = 1; i < positions.length; i++) {
        expect(positions[i]).toBeGreaterThan(positions[i - 1] ?? 0);
      }
    });

    it('should return positions within item bounds', () => {
      const results = search('app', ['apple', 'application'], {
        includePositions: true,
      });
      for (const r of results) {
        for (const pos of r.positions) {
          expect(pos).toBeLessThan(r.item.length);
        }
      }
    });

    it('should produce same scores with or without positions', () => {
      const withPos = search('apple', items, { includePositions: true });
      const withoutPos = search('apple', items);
      expect(withPos.length).toBe(withoutPos.length);
      for (let i = 0; i < withPos.length; i++) {
        expect(withPos[i]?.item).toBe(withoutPos[i]?.item);
        expect(withPos[i]?.score).toBeCloseTo(withoutPos[i]?.score ?? 0);
      }
    });

    it('should work with maxResults and minScore', () => {
      const results = search('app', items, {
        maxResults: 2,
        minScore: 0.1,
        includePositions: true,
      });
      expect(results.length).toBeLessThanOrEqual(2);
      for (const r of results) {
        expect(r.score).toBeGreaterThanOrEqual(0.1);
        expect(r.positions.length).toBeGreaterThan(0);
      }
    });
  });

  describe('matchType classification', () => {
    it('should classify exact match when includePositions is true', () => {
      const results = search('apple', ['apple', 'pineapple'], { includePositions: true });
      const exact = results.find((r) => r.item === 'apple');
      expect(exact?.matchType).toBe('Exact');
    });

    it('should classify prefix match', () => {
      const results = search('app', ['application'], { includePositions: true });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.matchType).toBe('Prefix');
    });

    it('should classify contains match', () => {
      const results = search('apple', ['pineapple'], { includePositions: true });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.matchType).toBe('Contains');
    });

    it('should classify fuzzy match', () => {
      const results = search('adf', ['abcdef'], { includePositions: true });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.matchType).toBe('Fuzzy');
    });

    it('should be undefined when includePositions is false', () => {
      const results = search('hello', ['hello']);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.matchType).toBeUndefined();
    });

    it('should be set when includePositions is true', () => {
      const results = search('hello', ['hello'], { includePositions: true });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.matchType).toBe('Exact');
    });

    it('should classify case-insensitive exact match', () => {
      const results = search('apple', ['Apple'], { includePositions: true, minScore: 1.0 });
      expect(results.length).toBe(1);
      expect(results[0]?.matchType).toBe('Exact');
    });
  });

  describe('case sensitivity', () => {
    it('should match case-insensitively by default (smart case)', () => {
      const results = search('apple', ['Apple', 'apple', 'APPLE'], {
        minScore: 1.0,
      });
      expect(results.length).toBe(3);
    });

    it('should match case-sensitively when isCaseSensitive is true', () => {
      const results = search('apple', ['Apple', 'apple', 'APPLE'], {
        isCaseSensitive: true,
        minScore: 1.0,
      });
      expect(results.length).toBe(1);
      expect(results[0]?.item).toBe('apple');
    });

    it('should match uppercase query case-sensitively', () => {
      const results = search('APPLE', ['Apple', 'apple', 'APPLE'], {
        isCaseSensitive: true,
        minScore: 1.0,
      });
      expect(results.length).toBe(1);
      expect(results[0]?.item).toBe('APPLE');
    });

    it('should use smart case when isCaseSensitive is false', () => {
      const results = search('apple', ['Apple', 'apple', 'APPLE'], {
        isCaseSensitive: false,
        minScore: 1.0,
      });
      expect(results.length).toBe(3);
    });

    it('should work with other options', () => {
      const results = search('app', ['Apple', 'apple', 'Application', 'APPLE'], {
        isCaseSensitive: true,
        maxResults: 2,
        includePositions: true,
      });
      expect(results.length).toBeLessThanOrEqual(2);
      for (const r of results) {
        expect(r.positions.length).toBeGreaterThan(0);
      }
    });
  });

  describe('returnAllOnEmpty', () => {
    const items = ['apple', 'banana', 'grape'];

    it('should return all items when query is empty', () => {
      const results = search('', items, { returnAllOnEmpty: true });
      expect(results.length).toBe(3);
      expect(results.map((r) => r.item)).toEqual(items);
    });

    it('should return items in original index order', () => {
      const results = search('', items, { returnAllOnEmpty: true });
      for (let i = 0; i < results.length; i++) {
        expect(results[i]?.index).toBe(i);
      }
    });

    it('should set score to 1.0 for all results', () => {
      const results = search('', items, { returnAllOnEmpty: true });
      for (const r of results) {
        expect(r.score).toBe(1.0);
      }
    });

    it('should return empty positions and no matchType', () => {
      const results = search('', items, { returnAllOnEmpty: true });
      for (const r of results) {
        expect(r.positions).toEqual([]);
        expect(r.matchType).toBeUndefined();
      }
    });

    it('should respect maxResults', () => {
      const results = search('', items, { returnAllOnEmpty: true, maxResults: 2 });
      expect(results.length).toBe(2);
    });

    it('should treat whitespace-only query as empty', () => {
      const results = search('  ', items, { returnAllOnEmpty: true });
      expect(results.length).toBe(3);
    });

    it('should return empty when returnAllOnEmpty is false (default)', () => {
      expect(search('', items)).toEqual([]);
      expect(search('', items, { returnAllOnEmpty: false })).toEqual([]);
    });

    it('should perform normal search when query is non-empty', () => {
      const results = search('apple', items, { returnAllOnEmpty: true });
      expect(results.length).toBeGreaterThan(0);
      expect(results.length).toBeLessThan(items.length);
    });
  });
});

describe('closest', () => {
  const items = ['apple', 'banana', 'grape', 'orange'];

  it('should return the best match', () => {
    const result = closest('aple', items);
    expect(result).toBe('apple');
  });

  it('should return exact match when available', () => {
    const result = closest('banana', items);
    expect(result).toBe('banana');
  });

  it('should return null for empty items', () => {
    const result = closest('test', []);
    expect(result).toBeNull();
  });

  it('should return null when best match is below minScore', () => {
    const result = closest('hello', ['xyz', 'abc'], 0.99);
    expect(result).toBeNull();
  });

  it('should return match when above minScore', () => {
    const result = closest('apple', ['apple', 'banana'], 0.5);
    expect(result).toBe('apple');
  });
});

describe('token-based matching', () => {
  describe('tokenSortRatio', () => {
    it('should return 1.0 for reordered tokens', () => {
      expect(tokenSortRatio('New York Mets', 'Mets New York')).toBe(1.0);
    });

    it('should be case-insensitive', () => {
      expect(tokenSortRatio('john smith', 'SMITH JOHN')).toBe(1.0);
    });

    it('should return high score for similar reordered strings', () => {
      const score = tokenSortRatio('John A. Smith', 'Smith, John A');
      expect(score).toBeGreaterThan(0.8);
    });

    it('should return 1.0 for identical strings', () => {
      expect(tokenSortRatio('hello world', 'hello world')).toBe(1.0);
    });

    it('should return 1.0 for both empty', () => {
      expect(tokenSortRatio('', '')).toBe(1.0);
    });

    it('should return 0.0 for one empty', () => {
      expect(tokenSortRatio('hello', '')).toBe(0.0);
    });
  });

  describe('tokenSortRatioBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = tokenSortRatioBatch([
        ['New York Mets', 'Mets New York'],
        ['abc', 'xyz'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });

    it('should return empty array for empty input', () => {
      expect(tokenSortRatioBatch([])).toEqual([]);
    });
  });

  describe('tokenSortRatioMany', () => {
    it('should compute scores from reference to candidates', () => {
      const result = tokenSortRatioMany('New York Mets', ['Mets New York', 'completely different']);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });
  });

  describe('tokenSetRatio', () => {
    it('should return 1.0 for reordered tokens', () => {
      expect(tokenSetRatio('Mariners vs Yankees', 'Yankees vs Mariners')).toBe(1.0);
    });

    it('should handle subset tokens gracefully', () => {
      const score = tokenSetRatio('Great Gatsby', 'The Great Gatsby by Fitzgerald');
      expect(score).toBeGreaterThan(0.7);
    });

    it('should return 1.0 for both empty', () => {
      expect(tokenSetRatio('', '')).toBe(1.0);
    });

    it('should return low score for no shared tokens', () => {
      const score = tokenSetRatio('abc def', 'xyz uvw');
      expect(score).toBeLessThan(0.5);
    });
  });

  describe('tokenSetRatioBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = tokenSetRatioBatch([
        ['Mariners vs Yankees', 'Yankees vs Mariners'],
        ['abc', 'xyz'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });

    it('should return empty array for empty input', () => {
      expect(tokenSetRatioBatch([])).toEqual([]);
    });
  });

  describe('tokenSetRatioMany', () => {
    it('should compute scores from reference to candidates', () => {
      const result = tokenSetRatioMany('Mariners vs Yankees', [
        'Yankees vs Mariners',
        'something else',
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });
  });

  describe('partialRatio', () => {
    it('should return 1.0 for substring match', () => {
      expect(partialRatio('hello', 'hello world')).toBe(1.0);
    });

    it('should return 1.0 for identical strings', () => {
      expect(partialRatio('hello', 'hello')).toBe(1.0);
    });

    it('should return high score for partial overlap', () => {
      const score = partialRatio('cat', 'scattered');
      expect(score).toBeGreaterThan(0.5);
    });

    it('should return 1.0 for both empty', () => {
      expect(partialRatio('', '')).toBe(1.0);
    });

    it('should return 0.0 for one empty', () => {
      expect(partialRatio('hello', '')).toBe(0.0);
    });
  });

  describe('partialRatioBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = partialRatioBatch([
        ['hello', 'hello world'],
        ['abc', 'xyz'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });

    it('should return empty array for empty input', () => {
      expect(partialRatioBatch([])).toEqual([]);
    });
  });

  describe('partialRatioMany', () => {
    it('should compute scores from reference to candidates', () => {
      const result = partialRatioMany('hello', ['hello world', 'xyz']);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });
  });

  describe('weightedRatio', () => {
    it('should return 1.0 for identical strings', () => {
      expect(weightedRatio('hello', 'hello')).toBe(1.0);
    });

    it('should return 1.0 for reordered tokens', () => {
      expect(weightedRatio('New York Mets', 'Mets New York')).toBe(1.0);
    });

    it('should return 1.0 for substring match', () => {
      expect(weightedRatio('hello', 'hello world')).toBe(1.0);
    });

    it('should be at least as good as normalizedLevenshtein', () => {
      const a = 'test string';
      const b = 'testing strings';
      expect(weightedRatio(a, b)).toBeGreaterThanOrEqual(normalizedLevenshtein(a, b));
    });

    it('should return low score for completely different strings', () => {
      expect(weightedRatio('abc', 'xyz')).toBeLessThan(0.5);
    });
  });

  describe('weightedRatioBatch', () => {
    it('should compute scores for multiple pairs', () => {
      const result = weightedRatioBatch([
        ['hello', 'hello'],
        ['abc', 'xyz'],
      ]);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });

    it('should return empty array for empty input', () => {
      expect(weightedRatioBatch([])).toEqual([]);
    });
  });

  describe('weightedRatioMany', () => {
    it('should compute scores from reference to candidates', () => {
      const result = weightedRatioMany('hello', ['hello', 'xyz']);
      expect(result[0]).toBe(1.0);
      expect(result[1]).toBeLessThan(0.5);
    });
  });
});

describe('unicode', () => {
  describe('CJK characters', () => {
    it('should compute distance for CJK strings', () => {
      // '東京' -> '京都': replace '東'->'京', replace '京'->'都' = 2
      expect(levenshtein('東京', '京都')).toBe(2);
    });

    it('should return 0 for identical CJK strings', () => {
      expect(levenshtein('日本語', '日本語')).toBe(0);
    });

    it('should handle similarity for CJK strings', () => {
      // '日本語' and '日本人' share the bigram '日本'
      const score = sorensenDice('日本語', '日本人');
      expect(score).toBeGreaterThan(0);
      expect(score).toBeLessThan(1);
    });
  });

  describe('emoji', () => {
    it('should compute distance for emoji strings', () => {
      // Each emoji is one Unicode scalar value; '🌍' -> '🌎' = 1 substitution
      expect(levenshtein('👋🌍', '👋🌎')).toBe(1);
    });

    it('should return 0 for identical emoji strings', () => {
      expect(levenshtein('🎉🎊', '🎉🎊')).toBe(0);
    });
  });

  describe('diacritics and accented characters', () => {
    it('should compute similarity for accented characters', () => {
      // 'café' vs 'cafe': 'é' != 'e', so distance = 1
      const score = jaroWinkler('café', 'cafe');
      expect(score).toBeGreaterThan(0.8);
      expect(score).toBeLessThan(1);
    });

    it('should return 1.0 for identical accented strings', () => {
      expect(normalizedLevenshtein('naïve', 'naïve')).toBe(1.0);
    });
  });

  describe('mixed scripts', () => {
    it('should compute distance for mixed-script strings', () => {
      // 'hello世界' -> 'hello世間': replace '界'->'間' = 1
      const dist = levenshtein('hello世界', 'hello世間');
      expect(dist).toBe(1);
    });

    it('should handle batch with Unicode pairs', () => {
      const result = levenshteinBatch([
        ['東京', '京都'],
        ['café', 'cafe'],
      ]);
      expect(result).toHaveLength(2);
      expect(result[0]).toBe(2);
      expect(result[1]).toBe(1);
    });
  });

  describe('token-based with Unicode', () => {
    it('should sort CJK tokens correctly', () => {
      expect(tokenSortRatio('東京 日本', '日本 東京')).toBe(1.0);
    });

    it('should handle emoji tokens', () => {
      expect(tokenSortRatio('🎉 🎊', '🎊 🎉')).toBe(1.0);
    });

    it('should compute partial ratio for CJK substrings', () => {
      const score = partialRatio('東京', '東京タワー');
      expect(score).toBeGreaterThan(0.5);
    });
  });

  describe('search with Unicode', () => {
    it('should find Unicode matches', () => {
      const results = search('東', ['東京', '大阪', '京都']);
      expect(results.length).toBeGreaterThan(0);
    });

    it('should find closest Unicode match', () => {
      const result = closest('東京', ['大阪', '京都', '東京都']);
      expect(result).not.toBeNull();
    });
  });

  describe('multi-term search', () => {
    const contacts = ['John Smith', 'Smith, John A.', 'Jane Doe', 'John Doe', 'Bob Smith'];

    it('should require all terms to match (AND semantics)', () => {
      const results = search('john smith', contacts);
      const items = results.map((r) => r.item);
      expect(items).toContain('John Smith');
      expect(items).toContain('Smith, John A.');
      expect(items).not.toContain('John Doe');
      expect(items).not.toContain('Bob Smith');
    });

    it('should return more results for single term than multi-term', () => {
      const single = search('john', contacts);
      const multi = search('john smith', contacts);
      expect(single.length).toBeGreaterThan(multi.length);
    });

    it('should return positions spanning all matched terms', () => {
      const results = search('john smith', contacts, { includePositions: true });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.positions.length).toBeGreaterThan(0);
    });

    it('should work with FuzzyIndex', () => {
      const { FuzzyIndex } = require('../index.js');
      const idx = new FuzzyIndex(contacts);
      const results = idx.search('john smith');
      const items = results.map((r: { item: string }) => r.item);
      expect(items).toContain('John Smith');
      expect(items).not.toContain('John Doe');
    });

    it('single-word query should behave identically', () => {
      const results = search('john', contacts);
      expect(results.length).toBe(3); // John Smith, Smith John A., John Doe
    });
  });
});

describe('extended query syntax', () => {
  describe('exclusion (!term)', () => {
    const fruits = ['apple pie', 'apple juice', 'cherry pie', 'banana split'];

    it('should exclude items matching negated term', () => {
      const results = search('apple !pie', fruits);
      const items = results.map((r) => r.item);
      expect(items).toContain('apple juice');
      expect(items).not.toContain('apple pie');
    });

    it('should not return items matching only the excluded term', () => {
      const results = search('apple !pie', fruits);
      const items = results.map((r) => r.item);
      expect(items).not.toContain('cherry pie');
      expect(items).not.toContain('banana split');
    });

    it('should return only non-matching items with negation-only query', () => {
      const results = search('!pie', fruits);
      const items = results.map((r) => r.item);
      expect(items).toContain('apple juice');
      expect(items).toContain('banana split');
      expect(items).not.toContain('apple pie');
      expect(items).not.toContain('cherry pie');
    });

    it('should return empty when all items match the excluded term', () => {
      const results = search('!pie', ['pie', 'pie chart']);
      expect(results).toEqual([]);
    });
  });

  describe('prefix (^term)', () => {
    const items = ['apple', 'pineapple', 'application', 'maple'];

    it('should match only items starting with prefix term', () => {
      const results = search('^app', items);
      const matched = results.map((r) => r.item);
      expect(matched).toContain('apple');
      expect(matched).toContain('application');
      expect(matched).not.toContain('pineapple');
      expect(matched).not.toContain('maple');
    });

    it('should return empty when no items start with prefix', () => {
      const results = search('^xyz', items);
      expect(results).toEqual([]);
    });
  });

  describe('suffix (term$)', () => {
    const items = ['apple pie', 'pie chart', 'shepherd pie', 'pied piper'];

    it('should match only items ending with suffix term', () => {
      const results = search('pie$', items);
      const matched = results.map((r) => r.item);
      expect(matched).toContain('apple pie');
      expect(matched).toContain('shepherd pie');
      expect(matched).not.toContain('pie chart');
      expect(matched).not.toContain('pied piper');
    });

    it('should return empty when no items end with suffix', () => {
      const results = search('xyz$', items);
      expect(results).toEqual([]);
    });
  });

  describe("literal ('term)", () => {
    const items = ['react', 'preact', 'reactive', 'redux'];

    it('should match exact substring without fuzzy logic', () => {
      const results = search("'react", items);
      const matched = results.map((r) => r.item);
      expect(matched).toContain('react');
      expect(matched).toContain('preact');
      expect(matched).toContain('reactive');
      expect(matched).not.toContain('redux');
    });

    it('should return empty when no items contain the literal', () => {
      const results = search("'xyz", items);
      expect(results).toEqual([]);
    });
  });

  describe('combined patterns', () => {
    it('should combine prefix and exclusion', () => {
      const items = ['apple pie', 'apple juice', 'cherry pie', 'apple cider'];
      const results = search('^apple !pie', items);
      const matched = results.map((r) => r.item);
      expect(matched).toContain('apple juice');
      expect(matched).toContain('apple cider');
      expect(matched).not.toContain('apple pie');
      expect(matched).not.toContain('cherry pie');
    });

    it('should combine prefix and suffix', () => {
      const items = ['apple pie', 'apple juice', 'apricot pie'];
      const results = search('^apple pie$', items);
      const matched = results.map((r) => r.item);
      expect(matched).toContain('apple pie');
      expect(matched).not.toContain('apple juice');
      expect(matched).not.toContain('apricot pie');
    });
  });

  describe('FuzzyIndex parity', () => {
    it('should support exclusion', () => {
      const idx = new FuzzyIndex(['apple pie', 'apple juice', 'cherry pie']);
      const results = idx.search('apple !pie');
      const items = results.map((r) => r.item);
      expect(items).toContain('apple juice');
      expect(items).not.toContain('apple pie');
    });

    it('should support prefix', () => {
      const idx = new FuzzyIndex(['apple', 'pineapple', 'application']);
      const results = idx.search('^app');
      const items = results.map((r) => r.item);
      expect(items).toContain('apple');
      expect(items).toContain('application');
      expect(items).not.toContain('pineapple');
    });

    it('should support suffix', () => {
      const idx = new FuzzyIndex(['apple pie', 'pie chart', 'shepherd pie']);
      const results = idx.search('pie$');
      const items = results.map((r) => r.item);
      expect(items).toContain('apple pie');
      expect(items).toContain('shepherd pie');
      expect(items).not.toContain('pie chart');
    });

    it('should support literal', () => {
      const idx = new FuzzyIndex(['react', 'preact', 'reactive', 'redux']);
      const results = idx.search("'react");
      const items = results.map((r) => r.item);
      expect(items).toContain('react');
      expect(items).toContain('preact');
      expect(items).toContain('reactive');
      expect(items).not.toContain('redux');
    });

    it('should support combined patterns', () => {
      const idx = new FuzzyIndex(['apple pie', 'apple juice', 'cherry pie', 'apple cider']);
      const results = idx.search('^apple !pie');
      const items = results.map((r) => r.item);
      expect(items).toContain('apple juice');
      expect(items).toContain('apple cider');
      expect(items).not.toContain('apple pie');
      expect(items).not.toContain('cherry pie');
    });
  });

  describe('edge cases', () => {
    it('should handle diacritics with prefix syntax', () => {
      const idx = new FuzzyIndex(['café au lait', 'cafe mocha', 'cappuccino']);
      const results = idx.search('^café');
      expect(results.map((r) => r.item)).toContain('café au lait');
    });

    it('should return positions with extended syntax when includePositions is true', () => {
      const results = search('^app', ['apple', 'pineapple'], {
        includePositions: true,
      });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.positions.length).toBeGreaterThan(0);
    });

    it('should respect maxResults with extended syntax', () => {
      const items = Array.from({ length: 20 }, (_, i) => `apple${i}`);
      const results = search('^apple', items, { maxResults: 3 });
      expect(results.length).toBeLessThanOrEqual(3);
    });

    it('should respect minScore with extended syntax', () => {
      const results = search('^app', ['apple', 'application'], {
        minScore: 0.5,
      });
      for (const r of results) {
        expect(r.score).toBeGreaterThanOrEqual(0.5);
      }
    });
  });
});

describe('FuzzyIndex', () => {
  const items = ['apple', 'banana', 'grape', 'orange', 'pineapple', 'mango'];

  describe('constructor and size', () => {
    it('should create an index with correct size', () => {
      const index = new FuzzyIndex(items);
      expect(index.size).toBe(6);
    });

    it('should handle empty items', () => {
      const index = new FuzzyIndex([]);
      expect(index.size).toBe(0);
    });
  });

  describe('search', () => {
    it('should find fuzzy matches', () => {
      const index = new FuzzyIndex(items);
      const results = index.search('aple');
      expect(results.length).toBeGreaterThan(0);
      expect(results.some((r) => r.item === 'apple')).toBe(true);
    });

    it('should return results sorted by score', () => {
      const index = new FuzzyIndex(items);
      const results = index.search('an');
      for (let i = 1; i < results.length; i++) {
        expect(results[i - 1]?.score).toBeGreaterThanOrEqual(results[i]?.score ?? 0);
      }
    });

    it('should respect maxResults as number', () => {
      const index = new FuzzyIndex(items);
      const results = index.search('a', 2);
      expect(results.length).toBeLessThanOrEqual(2);
    });

    it('should accept SearchOptions object', () => {
      const index = new FuzzyIndex(items);
      const results = index.search('a', { maxResults: 2 });
      expect(results.length).toBeLessThanOrEqual(2);
    });

    it('should filter by minScore', () => {
      const index = new FuzzyIndex(items);
      const results = index.search('apple', { minScore: 0.5 });
      for (const r of results) {
        expect(r.score).toBeGreaterThanOrEqual(0.5);
      }
    });

    it('should return positions when includePositions is true', () => {
      const index = new FuzzyIndex(['hello']);
      const results = index.search('hello', { includePositions: true });
      expect(results.length).toBe(1);
      expect(results[0]?.positions).toEqual([0, 1, 2, 3, 4]);
    });

    it('should return empty positions by default', () => {
      const index = new FuzzyIndex(['hello']);
      const results = index.search('hello');
      expect(results[0]?.positions).toEqual([]);
    });

    it('should return scores in 0.0-1.0 range', () => {
      const index = new FuzzyIndex(items);
      const results = index.search('app');
      for (const r of results) {
        expect(r.score).toBeGreaterThanOrEqual(0);
        expect(r.score).toBeLessThanOrEqual(1);
      }
    });

    it('should return empty array for empty query', () => {
      const index = new FuzzyIndex(items);
      expect(index.search('')).toEqual([]);
    });

    it('should produce same results as standalone search', () => {
      const { search } = require('../index.js');
      const index = new FuzzyIndex(items);
      const indexResults = index.search('apple');
      const standaloneResults = search('apple', items);
      expect(indexResults.length).toBe(standaloneResults.length);
      for (let i = 0; i < indexResults.length; i++) {
        expect(indexResults[i]?.item).toBe(standaloneResults[i]?.item);
        expect(indexResults[i]?.score).toBeCloseTo(standaloneResults[i]?.score ?? 0);
      }
    });

    it('should support isCaseSensitive option', () => {
      const index = new FuzzyIndex(['Apple', 'apple', 'APPLE']);
      const results = index.search('apple', { isCaseSensitive: true, minScore: 1.0 });
      expect(results.length).toBe(1);
      expect(results[0]?.item).toBe('apple');
    });

    it('should default to smart case matching', () => {
      const index = new FuzzyIndex(['Apple', 'apple', 'APPLE']);
      const results = index.search('apple', { minScore: 1.0 });
      expect(results.length).toBe(3);
    });

    it('should include matchType when includePositions is true', () => {
      const index = new FuzzyIndex(['apple', 'apple juice', 'pineapple']);
      const results = index.search('apple', { includePositions: true });
      const exact = results.find((r) => r.item === 'apple');
      const prefix = results.find((r) => r.item === 'apple juice');
      const contains = results.find((r) => r.item === 'pineapple');
      expect(exact?.matchType).toBe('Exact');
      expect(prefix?.matchType).toBe('Prefix');
      expect(contains?.matchType).toBe('Contains');
    });

    it('should not include matchType without includePositions', () => {
      const index = new FuzzyIndex(['apple']);
      const results = index.search('apple');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]?.matchType).toBeUndefined();
    });

    it('should have consistent matchType with standalone search', () => {
      const items = ['apple', 'apple juice', 'pineapple'];
      const index = new FuzzyIndex(items);
      const indexResults = index.search('apple', { includePositions: true });
      const standaloneResults = search('apple', items, { includePositions: true });
      for (let i = 0; i < indexResults.length; i++) {
        expect(indexResults[i]?.matchType).toBe(standaloneResults[i]?.matchType);
      }
    });
  });

  describe('searchIndices', () => {
    it('should return indices and scores without item strings', () => {
      const index = new FuzzyIndex(items);
      const results = index.searchIndices('aple');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]).toHaveProperty('index');
      expect(results[0]).toHaveProperty('score');
      expect(results[0]).not.toHaveProperty('item');
    });

    it('should return matching indices consistent with search()', () => {
      const index = new FuzzyIndex(items);
      const searchResults = index.search('grape');
      const indexResults = index.searchIndices('grape');
      expect(indexResults.length).toBe(searchResults.length);
      for (let i = 0; i < indexResults.length; i++) {
        expect(indexResults[i]?.index).toBe(searchResults[i]?.index);
        expect(indexResults[i]?.score).toBe(searchResults[i]?.score);
      }
    });

    it('should respect maxResults', () => {
      const index = new FuzzyIndex(items);
      const results = index.searchIndices('a', 2);
      expect(results.length).toBeLessThanOrEqual(2);
    });

    it('should include positions when requested', () => {
      const index = new FuzzyIndex(['apple']);
      const results = index.searchIndices('apple', { includePositions: true });
      expect(results.length).toBe(1);
      expect(results[0]?.positions.length).toBeGreaterThan(0);
    });
  });

  describe('closest', () => {
    it('should return the best match', () => {
      const index = new FuzzyIndex(items);
      expect(index.closest('aple')).toBe('apple');
    });

    it('should return null for empty index', () => {
      const index = new FuzzyIndex([]);
      expect(index.closest('test')).toBeNull();
    });

    it('should respect minScore', () => {
      const index = new FuzzyIndex(['xyz']);
      expect(index.closest('hello', 0.99)).toBeNull();
    });
  });

  describe('returnAllOnEmpty', () => {
    it('should return all items when query is empty', () => {
      const index = new FuzzyIndex(['apple', 'banana', 'grape']);
      const results = index.search('', { returnAllOnEmpty: true });
      expect(results.length).toBe(3);
      expect(results.map((r) => r.item)).toEqual(['apple', 'banana', 'grape']);
    });

    it('should respect maxResults', () => {
      const index = new FuzzyIndex(['apple', 'banana', 'grape']);
      const results = index.search('', { returnAllOnEmpty: true, maxResults: 1 });
      expect(results.length).toBe(1);
    });

    it('should return empty when option is not set', () => {
      const index = new FuzzyIndex(['apple']);
      expect(index.search('')).toEqual([]);
    });
  });

  describe('add and addMany', () => {
    it('should add a single item', () => {
      const index = new FuzzyIndex(['apple']);
      index.add('banana');
      expect(index.size).toBe(2);
      expect(index.closest('banana')).toBe('banana');
    });

    it('should add multiple items', () => {
      const index = new FuzzyIndex([]);
      index.addMany(['apple', 'banana', 'grape']);
      expect(index.size).toBe(3);
    });
  });

  describe('remove', () => {
    it('should remove item at index', () => {
      const index = new FuzzyIndex(['apple', 'banana', 'grape']);
      expect(index.remove(1)).toBe(true);
      expect(index.size).toBe(2);
    });

    it('should return false for out-of-bounds index', () => {
      const index = new FuzzyIndex(['apple']);
      expect(index.remove(5)).toBe(false);
    });
  });

  describe('destroy', () => {
    it('should clear all items', () => {
      const index = new FuzzyIndex(items);
      index.destroy();
      expect(index.size).toBe(0);
      expect(index.search('apple')).toEqual([]);
    });
  });
});

describe('FuzzyIndex serialization', () => {
  it('should round-trip serialize and deserialize', () => {
    const items = ['TypeScript', 'JavaScript', 'Python', 'Rust'];
    const index = new FuzzyIndex(items);
    const original = index.search('type');

    const buffer = index.serialize();
    const restored = FuzzyIndex.deserialize(buffer);
    const restoredResults = restored.search('type');

    expect(restoredResults.length).toBe(original.length);
    for (let i = 0; i < original.length; i++) {
      expect(restoredResults[i]?.item).toBe(original[i]?.item);
      expect(restoredResults[i]?.score).toBeCloseTo(original[i]?.score ?? 0);
    }
  });

  it('should round-trip with Unicode data', () => {
    const items = ['東京', '大阪', 'café', 'naïve', '🎉 party'];
    const index = new FuzzyIndex(items);
    const buffer = index.serialize();
    const restored = FuzzyIndex.deserialize(buffer);
    expect(restored.size).toBe(items.length);
  });

  it('should round-trip an empty index', () => {
    const index = new FuzzyIndex([]);
    const buffer = index.serialize();
    const restored = FuzzyIndex.deserialize(buffer);
    expect(restored.size).toBe(0);
  });

  it('should throw on invalid magic bytes', () => {
    const buffer = Buffer.from([
      0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    expect(() => FuzzyIndex.deserialize(buffer)).toThrow();
  });

  it('should throw on truncated data', () => {
    const index = new FuzzyIndex(['hello', 'world']);
    const buffer = index.serialize();
    const truncated = buffer.subarray(0, buffer.length - 3);
    expect(() => FuzzyIndex.deserialize(truncated)).toThrow();
  });

  it('should throw on too-short data', () => {
    const buffer = Buffer.from([0x01, 0x02]);
    expect(() => FuzzyIndex.deserialize(buffer)).toThrow();
  });
});

describe('KeyedFuzzyIndex error propagation', () => {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const { KeyedFuzzyIndex } = require('../index.js') as {
    KeyedFuzzyIndex: typeof import('../index.js').KeyedFuzzyIndex;
  };

  it('should throw on negative weights', () => {
    expect(() => new KeyedFuzzyIndex([['a']], [-1.0])).toThrow();
  });

  it('should throw on NaN weights', () => {
    expect(() => new KeyedFuzzyIndex([['a']], [Number.NaN])).toThrow();
  });

  it('should throw on Infinity weights', () => {
    expect(() => new KeyedFuzzyIndex([['a']], [Number.POSITIVE_INFINITY])).toThrow();
  });

  it('should throw on zero total weight', () => {
    expect(() => new KeyedFuzzyIndex([['a']], [0.0])).toThrow();
  });

  it('should throw on mismatched key_texts column lengths', () => {
    expect(() => new KeyedFuzzyIndex([['a', 'b'], ['c']], [1.0, 1.0])).toThrow();
  });

  it('should throw on mismatched weights count', () => {
    expect(() => new KeyedFuzzyIndex([['a']], [1.0, 2.0])).toThrow();
  });

  it('should throw when add() receives wrong number of key values', () => {
    const index = new KeyedFuzzyIndex([['a'], ['b']], [1.0, 1.0]);
    expect(() => index.add(['only_one'])).toThrow();
  });
});

describe('ESM/CJS export parity', () => {
  it('should have matching exports between CJS and ESM', () => {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const fs = require('node:fs') as typeof import('node:fs');
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const cjsBinding = require('../index.js') as Record<string, unknown>;
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const highlightModule = require('../highlight.js') as Record<string, unknown>;
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const objectsModule = require('../objects.js') as Record<string, unknown>;

    const esmContent = fs.readFileSync(require.resolve('../index.mjs'), 'utf8');

    // Internal classes consumed only by wrapper modules (not part of public ESM API)
    const internalOnly = new Set(['KeyedFuzzyIndex']);

    // Collect all CJS exports across the three modules
    const allCjsExports = new Set([
      ...Object.keys(cjsBinding),
      ...Object.keys(highlightModule),
      ...Object.keys(objectsModule),
    ]);

    // Extract named exports from ESM destructuring patterns
    // e.g. export const { FuzzyIndex, search, ... } = { ... };
    const destructureMatches = [...esmContent.matchAll(/export const \{([^}]+)\}/g)];
    const allEsmExports = new Set(
      destructureMatches.flatMap((m) =>
        (m[1] ?? '')
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean),
      ),
    );

    // Every public function/class in CJS should be available in ESM
    const missingFromEsm: string[] = [];
    for (const name of allCjsExports) {
      if (!internalOnly.has(name) && !allEsmExports.has(name)) {
        missingFromEsm.push(name);
      }
    }

    expect(missingFromEsm).toEqual([]);
  });
});

describe('searchKeys', () => {
  // Simulate: [{name: "John Smith", email: "john@example.com"},
  //            {name: "Jane Doe", email: "jane@example.com"},
  //            {name: "Bob Johnson", email: "bob@test.com"}]
  const names = ['John Smith', 'Jane Doe', 'Bob Johnson'];
  const emails = ['john@example.com', 'jane@example.com', 'bob@test.com'];

  it('should find matches across multiple keys', () => {
    const results = searchKeys('john', [names, emails], [1, 1]);
    expect(results.length).toBeGreaterThan(0);
    // John Smith should rank first (matches on both name and email)
    expect(results[0]?.index).toBe(0);
  });

  it('should respect key weights', () => {
    // With equal weights, "john" matches John Smith on both keys
    const equalWeights = searchKeys('john', [names, emails], [1, 1]);
    expect(equalWeights[0]?.index).toBe(0);

    // Search for "bob" with name weighted higher
    const results = searchKeys('bob', [names, emails], [2, 1]);
    expect(results.length).toBeGreaterThan(0);
    // Bob Johnson should match (name key has higher weight)
    expect(results[0]?.index).toBe(2);
  });

  it('should return scores in 0.0-1.0 range', () => {
    const results = searchKeys('john', [names, emails], [1, 1]);
    for (const r of results) {
      expect(r.score).toBeGreaterThanOrEqual(0);
      expect(r.score).toBeLessThanOrEqual(1);
    }
  });

  it('should return sorted results (best first)', () => {
    const results = searchKeys('john', [names, emails], [1, 1]);
    for (let i = 1; i < results.length; i++) {
      expect(results[i - 1]?.score).toBeGreaterThanOrEqual(results[i]?.score ?? 0);
    }
  });

  it('should include per-key scores', () => {
    const results = searchKeys('john', [names, emails], [1, 1]);
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]?.keyScores).toHaveLength(2);
  });

  it('should respect maxResults', () => {
    const results = searchKeys('o', [names, emails], [1, 1], { maxResults: 1 });
    expect(results.length).toBeLessThanOrEqual(1);
  });

  it('should respect minScore', () => {
    const results = searchKeys('john', [names, emails], [1, 1], { minScore: 0.3 });
    for (const r of results) {
      expect(r.score).toBeGreaterThanOrEqual(0.3);
    }
  });

  it('should return empty for empty query', () => {
    expect(searchKeys('', [names, emails], [1, 1])).toEqual([]);
  });

  it('should return empty for empty items', () => {
    expect(searchKeys('test', [[], []], [1, 1])).toEqual([]);
  });

  it('should match single-key results with standard search', () => {
    const items = ['apple', 'application', 'banana'];
    const keyResults = searchKeys('apple', [items], [1]);
    const stdResults = search('apple', items);
    expect(keyResults.length).toBe(stdResults.length);
    for (let i = 0; i < keyResults.length; i++) {
      expect(keyResults[i]?.index).toBe(stdResults[i]?.index);
      expect(keyResults[i]?.score).toBeCloseTo(stdResults[i]?.score ?? 0);
    }
  });
});
