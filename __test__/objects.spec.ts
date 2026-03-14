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

  it('should pass through includePositions option', () => {
    const results = searchObjects('john', users, {
      keys: ['name'],
      includePositions: true,
    });
    expect(results.length).toBeGreaterThan(0);
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
