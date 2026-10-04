// Type-level regression tests for the review of the 2.2 release candidate.
// Checked by `pnpm run typecheck` under the repo's strict tsconfig; the
// functions are never called.
import type * as Browser from '../../browser.mjs' with { 'resolution-mode': 'import' };
import type * as Node from '../../index.mjs' with { 'resolution-mode': 'import' };

// ─── Readonly array arguments are accepted by both builds ───────────────────

export function readonlyArguments(b: typeof Browser, n: typeof Node): void {
  const pairs: ReadonlyArray<readonly [string, string]> = [['kitten', 'sitting']];
  const candidates: readonly string[] = ['sitting', 'kitten'];
  const columns: ReadonlyArray<readonly string[]> = [candidates, candidates];
  for (const entry of [b, n]) {
    entry.levenshteinBatch(pairs);
    entry.levenshteinMany('kitten', candidates);
    entry.levenshteinManyU32('kitten', candidates);
    entry.jaroWinklerBatch(pairs);
    entry.jaroWinklerManyF64('kitten', candidates);
    entry.tokenSetRatioBatch(pairs);
    entry.normalizedHammingMany('kitten', candidates);
    entry.search('kit', candidates, { maxResults: Number.POSITIVE_INFINITY });
    entry.closest('kit', candidates);
    entry.searchKeys('kit', columns, [1, 1], 2);
    void entry.FuzzyIndex.fromAsync(candidates);
    const index = new entry.FuzzyIndex(candidates);
    index.addMany(candidates);
    const keyed = new entry.KeyedFuzzyIndex(columns, [1, 1]);
    keyed.add(candidates);
    keyed.addMany(columns);
    keyed.search('kit', 2);
    keyed.search('kit', { maxResults: Number.POSITIVE_INFINITY });
  }
}

// ─── Literal key names inside generic functions ─────────────────────────────

import { FuzzyObjectIndex, searchObjects } from '../../objects.js';

interface Located {
  name: string;
  address?: { city: string; geo: { lat: number } } | null | undefined;
  greet(): string;
}

// A constrained item type parameter accepts the key paths its constraint
// has, as string keys and as KeyConfig names, at any depth. (Method names are
// not told apart from properties there; concrete item types reject them.)
export function constrained<T extends Located>(items: readonly T[]): void {
  searchObjects('q', items, { keys: ['name'] });
  searchObjects('q', items, { keys: ['name', { name: 'address.city', weight: 2 }] });
  searchObjects('q', items, { keys: ['address.geo.lat'], maxResults: 1 });
  const index = new FuzzyObjectIndex(items, { keys: ['name', { name: 'name', weight: 2 }] });
  index.search('q', 2);
  // Names the constraint does not have are still rejected.
  // @ts-expect-error -- typo in a key name
  searchObjects('q', items, { keys: ['nmae'] });
  // @ts-expect-error -- typo in a nested key name
  searchObjects('q', items, { keys: ['address.ctiy'] });
  // @ts-expect-error -- path continues past a string value
  new FuzzyObjectIndex(items, { keys: ['name.length'] });
}

// Keys typed as plain strings are accepted for any item type parameter.
export function unconstrained<T>(items: readonly T[], keys: readonly string[]): void {
  searchObjects('q', items, { keys });
  new FuzzyObjectIndex(items, { keys: [...keys, { name: keys[0] ?? 'name', weight: 2 }] });
}

// Concrete item types keep the precise check.
declare const located: readonly Located[];
searchObjects('q', located, { keys: ['name', 'address.geo.lat'] });
// @ts-expect-error -- methods are not searchable keys
searchObjects('q', located, { keys: ['greet'] });
// @ts-expect-error -- path continues past a string value
searchObjects('q', located, { keys: ['name.length'] });

// ─── The browser FuzzyObjectIndex checks key names like the Node.js one ─────

export function browserKeys(b: typeof Browser): void {
  new b.FuzzyObjectIndex(located, { keys: ['name', { name: 'address.city', weight: 2 }] });
  // @ts-expect-error -- typo in a key name
  new b.FuzzyObjectIndex(located, { keys: ['nmae'] });
  // @ts-expect-error -- typo in a key config name
  new b.FuzzyObjectIndex(located, { keys: [{ name: 'address.ctiy' }] });
  const dynamic: string[] = ['name'];
  const index: Browser.FuzzyObjectIndex<Located> = new b.FuzzyObjectIndex(located, {
    keys: dynamic,
  });
  index.search('q', 2);
}
