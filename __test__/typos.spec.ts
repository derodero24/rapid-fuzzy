// The `maxTypos` option: typo-tolerant matching on top of fuzzy matching.
import { describe, expect, it } from 'vitest';

import {
  FuzzyIndex,
  FuzzyObjectIndex,
  KeyedFuzzyIndex,
  type SearchResult,
  search,
  searchKeys,
  searchObjects,
} from '../index.js';

const LANGUAGES = [
  'TypeScript',
  'JavaScript',
  'CoffeeScript',
  'PureScript',
  'typescript-eslint',
  'Python',
  'Rust',
  'Café crème',
  'Zürich',
];

const items = (results: readonly SearchResult[]): string[] => results.map((r) => r.item);

describe('maxTypos', () => {
  it('finds items despite substituted, swapped and extra letters', () => {
    for (const query of ['tpyescript', 'typozcript', 'typescirpt', 'typesscript']) {
      expect(items(search(query, LANGUAGES))).not.toContain('TypeScript');
      expect(items(search(query, LANGUAGES, { maxTypos: 2 }))).toContain('TypeScript');
    }
    expect(items(search('zurihc', LANGUAGES, { maxTypos: 1 }))).toEqual(['Zürich']);
  });

  it('keeps the scores of matches without typos and ranks typo matches below perfect ones', () => {
    const exact = search('typescript', LANGUAGES, { includePositions: true });
    const tolerant = search('typescript', LANGUAGES, { includePositions: true, maxTypos: 2 });
    for (const result of exact) {
      expect(tolerant.find((r) => r.index === result.index)).toEqual(result);
    }
    const typo = search('typescrypt', ['TypeScript', 'TypeScrypt'], { maxTypos: 1 });
    expect(items(typo)).toEqual(['TypeScrypt', 'TypeScript']);
    expect(typo[1]?.score).toBeLessThan(1);
  });

  it('tolerates no typo in short terms or in exact, prefix, suffix and negated terms', () => {
    expect(search('rsut', ['Rust'], { maxTypos: 3 })).toHaveLength(1);
    expect(search('rxt', ['Rust'], { maxTypos: 3 })).toEqual([]);
    for (const query of ["'tpyescript", '^tpyescript', 'tpyescript$']) {
      expect(search(query, LANGUAGES, { maxTypos: 2 })).toEqual([]);
    }
  });

  it('returns match positions within the item', () => {
    const [result] = search('tpyescript', ['TypeScript'], { includePositions: true, maxTypos: 2 });
    expect(result?.positions.length).toBeGreaterThan(0);
    expect(result?.positions.every((p) => p < 'TypeScript'.length)).toBe(true);
  });

  it('gives FuzzyIndex the results of search', () => {
    const index = new FuzzyIndex(LANGUAGES);
    for (const query of ['t', 'tp', 'tpy', 'tpye', 'tpyescript', 'zurihc', 'cafw']) {
      const options = { includePositions: true, maxTypos: 2 };
      expect(index.search(query, options)).toEqual(search(query, LANGUAGES, options));
      expect(index.searchIndices(query, options).map((r) => r.index)).toEqual(
        search(query, LANGUAGES, options).map((r) => r.index),
      );
    }
  });

  it('applies to multi-key and object search', () => {
    const users = [
      { name: 'Alice Johnson', city: 'Tokyo' },
      { name: 'Carol Williams', city: 'New York' },
      { name: 'Dave Brown', city: 'Paris' },
    ];
    const keys = ['name', 'city'];
    expect(searchObjects('wilaims', users, { keys })).toEqual([]);
    expect(searchObjects('wilaims', users, { keys, maxTypos: 1 }).map((r) => r.index)).toEqual([1]);
    const crossKey = { keys, maxTypos: 1, matchMode: 'crossKey' } as const;
    expect(searchObjects('dave pairs', users, crossKey).map((r) => r.index)).toEqual([2]);

    const objectIndex = new FuzzyObjectIndex(users, { keys });
    expect(objectIndex.search('wilaims', { maxTypos: 1 }).map((r) => r.index)).toEqual([1]);
    expect(objectIndex.closest('wilaims', { maxTypos: 1 })).toBe(users[1]);
    expect(objectIndex.closest('wilaims')).toBeNull();

    const columns = [users.map((u) => u.name), users.map((u) => u.city)];
    const keyed = new KeyedFuzzyIndex(columns, [1, 1]);
    expect(keyed.search('wilaims', { maxTypos: 1 })).toEqual(
      searchKeys('wilaims', columns, [1, 1], { maxTypos: 1 }),
    );
    expect(keyed.closest('wilaims', { maxTypos: 1 })).toBe(1);
  });

  it('accepts Infinity and rejects other non-integers', () => {
    expect(
      items(search('tpyescript', LANGUAGES, { maxTypos: Number.POSITIVE_INFINITY })),
    ).toContain('TypeScript');
    expect(search('tpyescript', LANGUAGES, { maxTypos: undefined })).toEqual([]);
    for (const maxTypos of [-1, 0.5, Number.NaN]) {
      expect(() => search('a', LANGUAGES, { maxTypos })).toThrow(
        'maxTypos must be a non-negative integer or Infinity',
      );
      expect(() => new FuzzyIndex(LANGUAGES).search('a', { maxTypos })).toThrow(
        'maxTypos must be a non-negative integer or Infinity',
      );
    }
  });
});
