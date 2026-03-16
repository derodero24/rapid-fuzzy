import { describe, expect, it } from 'vitest';

const { searchObjects } = require('../objects.js');

const users = [
  { name: 'John Smith', email: 'john@example.com', role: 'admin' },
  { name: 'Jane Doe', email: 'jane@example.com', role: 'user' },
  { name: 'Bob Johnson', email: 'bob@test.com', role: 'admin' },
];

describe('searchObjects', () => {
  it('should find matches across multiple keys', () => {
    const results = searchObjects('john', users, {
      keys: ['name', 'email'],
    });
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]?.item.name).toBe('John Smith');
  });

  it('should return original objects as items', () => {
    const results = searchObjects('jane', users, {
      keys: ['name'],
    });
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]?.item).toBe(users[1]);
  });

  it('should accept string keys with equal weight', () => {
    const results = searchObjects('john', users, {
      keys: ['name', 'email'],
    });
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]?.keyScores).toHaveLength(2);
  });

  it('should accept weighted key objects', () => {
    const results = searchObjects('john', users, {
      keys: [
        { name: 'name', weight: 2.0 },
        { name: 'email', weight: 1.0 },
      ],
    });
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]?.item.name).toBe('John Smith');
  });

  it('should accept mixed string and weighted keys', () => {
    const results = searchObjects('john', users, {
      keys: ['name', { name: 'email', weight: 0.5 }],
    });
    expect(results.length).toBeGreaterThan(0);
  });

  it('should respect maxResults', () => {
    const results = searchObjects('o', users, {
      keys: ['name', 'email'],
      maxResults: 1,
    });
    expect(results.length).toBeLessThanOrEqual(1);
  });

  it('should respect minScore', () => {
    const results = searchObjects('john', users, {
      keys: ['name'],
      minScore: 0.5,
    });
    for (const r of results) {
      expect(r.score).toBeGreaterThanOrEqual(0.5);
    }
  });

  it('should return scores in 0.0-1.0 range', () => {
    const results = searchObjects('john', users, {
      keys: ['name', 'email'],
    });
    for (const r of results) {
      expect(r.score).toBeGreaterThanOrEqual(0);
      expect(r.score).toBeLessThanOrEqual(1);
    }
  });

  it('should return sorted results (best first)', () => {
    const results = searchObjects('john', users, {
      keys: ['name', 'email'],
    });
    for (let i = 1; i < results.length; i++) {
      expect(results[i - 1]?.score).toBeGreaterThanOrEqual(results[i]?.score ?? 0);
    }
  });

  it('should return empty for empty query', () => {
    const results = searchObjects('', users, { keys: ['name'] });
    expect(results).toEqual([]);
  });

  it('should return empty for empty items', () => {
    const results = searchObjects('test', [], { keys: ['name'] });
    expect(results).toEqual([]);
  });

  it('should support nested key paths', () => {
    const items = [
      { name: 'Alice', address: { city: 'New York', state: 'NY' } },
      { name: 'Bob', address: { city: 'Los Angeles', state: 'CA' } },
    ];
    const results = searchObjects('new york', items, {
      keys: ['address.city'],
    });
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]?.item.name).toBe('Alice');
  });

  it('should handle missing nested keys gracefully', () => {
    const items = [{ name: 'Alice', meta: { tag: 'hello' } }, { name: 'Bob' }] as Array<{
      name: string;
      meta?: { tag: string };
    }>;
    const results = searchObjects('hello', items, {
      keys: ['meta.tag'],
    });
    // Should find Alice (has meta.tag: 'hello') but not crash on Bob (missing meta)
    expect(results.length).toBe(1);
    expect(results[0]?.item.name).toBe('Alice');
  });

  it('should support isCaseSensitive option', () => {
    const results = searchObjects('john', users, {
      keys: ['name'],
      isCaseSensitive: true,
      minScore: 1.0,
    });
    // 'john' (lowercase) won't exactly match 'John Smith' in case-sensitive mode
    expect(results.length).toBe(0);
  });

  it('should not include positions in results', () => {
    const results = searchObjects('john', users, {
      keys: ['name'],
    });
    expect(results.length).toBeGreaterThan(0);
    for (const r of results) {
      expect(r).not.toHaveProperty('positions');
    }
  });

  it('should match single-key results with searchKeys', () => {
    const { searchKeys } = require('../index.js');
    const names = users.map((u) => u.name);
    const objResults = searchObjects('john', users, { keys: ['name'] });
    const keyResults = searchKeys('john', [names], [1.0]);
    expect(objResults.length).toBe(keyResults.length);
    for (let i = 0; i < objResults.length; i++) {
      expect(objResults[i]?.index).toBe(keyResults[i]?.index);
      expect(objResults[i]?.score).toBeCloseTo(keyResults[i]?.score ?? 0);
    }
  });
});

describe('FuzzyObjectIndex', () => {
  const { FuzzyObjectIndex } = require('../objects.js');

  const users = [
    { name: 'John Smith', email: 'john@example.com' },
    { name: 'Jane Doe', email: 'jane@example.com' },
    { name: 'Bob Johnson', email: 'bob@example.com' },
  ];

  it('should construct and report size', () => {
    const index = new FuzzyObjectIndex(users, { keys: ['name', 'email'] });
    expect(index.size).toBe(3);
  });

  it('should search with weighted keys', () => {
    const index = new FuzzyObjectIndex(users, {
      keys: [{ name: 'name', weight: 2.0 }, 'email'],
    });
    const results = index.search('john');
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].item.name).toBe('John Smith');
    expect(results[0].score).toBeGreaterThan(0);
    expect(results[0].keyScores.length).toBe(2);
  });

  it('should find closest match', () => {
    const index = new FuzzyObjectIndex(users, { keys: ['name'] });
    const result = index.closest('jane');
    expect(result).not.toBeNull();
    expect(result?.name).toBe('Jane Doe');
  });

  it('should return null from closest when minScore is too high', () => {
    const index = new FuzzyObjectIndex(users, { keys: ['name'] });
    const result = index.closest('zzzzz', 0.99);
    expect(result).toBeNull();
  });

  it('should support add', () => {
    const index = new FuzzyObjectIndex(users, { keys: ['name'] });
    expect(index.size).toBe(3);
    index.add({ name: 'Alice Wonder', email: 'alice@example.com' });
    expect(index.size).toBe(4);
    const results = index.search('alice');
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].item.name).toBe('Alice Wonder');
  });

  it('should support addMany', () => {
    const index = new FuzzyObjectIndex([], { keys: ['name'] });
    index.addMany(users);
    expect(index.size).toBe(3);
  });

  it('should support remove with swap-remove semantics', () => {
    const index = new FuzzyObjectIndex(users, { keys: ['name'] });
    expect(index.remove(1)).toBe(true); // Remove Jane Doe
    expect(index.size).toBe(2);
    expect(index.remove(10)).toBe(false); // Out of bounds
  });

  it('should support destroy', () => {
    const index = new FuzzyObjectIndex(users, { keys: ['name'] });
    index.destroy();
    expect(index.size).toBe(0);
  });

  it('should support maxResults and minScore options', () => {
    const index = new FuzzyObjectIndex(users, { keys: ['name'] });
    const results = index.search('o', { maxResults: 1 });
    expect(results.length).toBeLessThanOrEqual(1);
  });

  it('should match searchObjects results for same data', () => {
    const index = new FuzzyObjectIndex(users, {
      keys: [{ name: 'name', weight: 2.0 }, 'email'],
    });
    const indexResults = index.search('john');
    const directResults = searchObjects('john', users, {
      keys: [{ name: 'name', weight: 2.0 }, 'email'],
    });

    expect(indexResults.length).toBe(directResults.length);
    for (let i = 0; i < indexResults.length; i++) {
      expect(indexResults[i].item).toEqual(directResults[i].item);
      expect(Math.abs(indexResults[i].score - directResults[i].score)).toBeLessThan(0.001);
    }
  });

  it('should support nested key paths', () => {
    const data = [
      { user: { name: 'Alice' }, id: 1 },
      { user: { name: 'Bob' }, id: 2 },
    ];
    const index = new FuzzyObjectIndex(data, { keys: ['user.name'] });
    const results = index.search('alice');
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].item.user.name).toBe('Alice');
  });
});
