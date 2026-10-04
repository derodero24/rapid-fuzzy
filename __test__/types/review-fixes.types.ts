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
