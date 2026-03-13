import { describe, expect, it } from 'vitest';

import {
  closest,
  damerauLevenshtein,
  jaro,
  jaroWinkler,
  levenshtein,
  normalizedLevenshtein,
  search,
  sorensenDice,
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
      expect(results[i - 1].score).toBeGreaterThanOrEqual(results[i].score);
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
});
