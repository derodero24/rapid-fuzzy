import { describe, expect, it } from 'vitest';

import { FuzzyIndex, type SearchOptions, type SearchResult, search } from '../index.js';

/** mulberry32: small deterministic PRNG so failures are reproducible. */
function rng(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const WORDS = [
  'src',
  'index',
  'handler',
  'Repository',
  'service',
  'lib',
  'python3',
  'usr',
  'ctrl',
  'components',
  'café',
  'Москва',
  '東京',
] as const;

/** Long path-like items, so that searches over them are split across threads. */
function corpus(seed: number, n: number): string[] {
  const random = rng(seed);
  const pick = (): string => WORDS[Math.floor(random() * WORDS.length)] ?? 'src';
  return Array.from({ length: n }, () => {
    const words = 2 + Math.floor(random() * 20);
    return Array.from({ length: words }, pick).join('/');
  });
}

/** Ranking order: score descending, then shorter item (in UTF-8 bytes), then lower index. */
function expectRanked(results: readonly SearchResult[]): void {
  for (let i = 1; i < results.length; i++) {
    const a = results[i - 1];
    const b = results[i];
    if (a === undefined || b === undefined) throw new Error('unreachable');
    const aLen = Buffer.byteLength(a.item);
    const bLen = Buffer.byteLength(b.item);
    const ordered =
      a.score > b.score ||
      (a.score === b.score && (aLen < bLen || (aLen === bLen && a.index < b.index)));
    expect(ordered, `results ${i - 1} and ${i} are out of order`).toBe(true);
  }
}

describe('large searches (parallel scoring)', () => {
  const items = corpus(42, 6000);
  const index = new FuzzyIndex(items);
  const cases: ReadonlyArray<[string, SearchOptions]> = [
    ['hndlr', {}],
    ['src index', { includePositions: true }],
    ['usr ctrl', { maxResults: 25 }],
    ['москва', { includePositions: true, minScore: 0.2 }],
    ['^src !python', {}],
  ];

  it.each(cases)('FuzzyIndex matches search() for %j', (query, options) => {
    const expected = search(query, items, options);
    expect(expected.length).toBeGreaterThan(0);
    expect(index.search(query, options)).toEqual(expected);
    expectRanked(expected);
    for (const result of expected) {
      expect(result.item).toBe(items[result.index]);
    }
    expect(index.searchIndices(query, options)).toEqual(
      expected.map(({ index: i, score, positions, matchType }) => ({
        index: i,
        score,
        positions,
        matchType,
      })),
    );
  });

  it('returns the same results on every run', () => {
    const first = index.search('src index', { includePositions: true });
    for (let run = 0; run < 3; run++) {
      // A different query in between resets the incremental cache.
      index.search('zz');
      expect(index.search('src index', { includePositions: true })).toEqual(first);
    }
  });
});

describe('FuzzyIndex memory', () => {
  it('stays usable after destroy()', () => {
    const index = new FuzzyIndex(['apple', 'banana']);
    index.search('ap');
    index.destroy();
    expect(index.size).toBe(0);
    expect(index.search('ap')).toEqual([]);
    expect(index.search('', { returnAllOnEmpty: true })).toEqual([]);
    index.add('apricot');
    index.addMany(['avocado']);
    expect(index.remove(1)).toBe(true);
    expect(index.remove(5)).toBe(false);
    expect(index.search('apr').map((r) => r.item)).toEqual(['apricot']);
  });

  it('lets the garbage collector reclaim indexes that are never destroyed', async () => {
    // Each index holds ~8 MB of native memory behind a tiny JavaScript
    // object. The index reports that memory to V8, so V8 collects
    // unreachable indexes without an explicit destroy() or gc(); before, it
    // saw no memory pressure and none of them was ever collected.
    const big = Array.from({ length: 8 }, (_, i) => String.fromCharCode(97 + i).repeat(1 << 20));
    let finalized = 0;
    const registry = new FinalizationRegistry<number>(() => {
      finalized++;
    });
    const created = 32;
    for (let i = 0; i < created; i++) {
      registry.register(new FuzzyIndex(big), i);
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(finalized).toBeGreaterThanOrEqual(created / 2);
  }, 30_000);

  it('builds through fromAsync() and deserialize() and destroys them', async () => {
    const items = corpus(7, 1000);
    const built = await FuzzyIndex.fromAsync(items);
    const restored = FuzzyIndex.deserialize(built.serialize());
    expect(restored.search('hndlr')).toEqual(built.search('hndlr'));
    built.destroy();
    restored.destroy();
    expect(built.size + restored.size).toBe(0);
  });
});
