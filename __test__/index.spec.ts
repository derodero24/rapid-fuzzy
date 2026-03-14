import { describe, expect, it } from 'vitest';

import {
  closest,
  damerauLevenshtein,
  damerauLevenshteinBatch,
  damerauLevenshteinMany,
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
});
