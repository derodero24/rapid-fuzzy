// Type-level tests for the declarations of the browser / edge entry points
// (browser.d.mts for "rapid-fuzzy", objects.browser.d.mts for
// "rapid-fuzzy/objects"). Checked by `pnpm run typecheck` under the repo's
// strict tsconfig; never executed.
import type * as Browser from '../../browser.mjs' with { 'resolution-mode': 'import' };
import type * as NodeEntry from '../../index.mjs' with { 'resolution-mode': 'import' };
import type * as BrowserObjects from '../../objects.browser.mjs' with {
  'resolution-mode': 'import',
};

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;
type IsAny<T> = 0 extends 1 & T ? true : false;

type BrowserExports = keyof typeof Browser;
// napi-rs's loader marker (index.d.ts, @napi-rs/cli >= 3.10) tells which Node.js
// binding was loaded; the WebAssembly build has no such binding.
type NodeExports = Exclude<keyof typeof NodeEntry, '__napiBindingTarget'>;
type ExportsWithAny = {
  [K in BrowserExports]: IsAny<(typeof Browser)[K]> extends true ? K : never;
}[BrowserExports];

export type DeclarationChecks = [
  // The same runtime exports as the Node.js ES module entry.
  Expect<Equal<Exclude<BrowserExports, NodeExports>, never>>,
  Expect<Equal<Exclude<NodeExports, BrowserExports>, never>>,
  Expect<Equal<ExportsWithAny, never>>,
  // FuzzyObjectIndex without the Buffer-based serialize() / deserialize().
  Expect<Equal<'serialize' extends keyof Browser.FuzzyObjectIndex<unknown> ? true : false, false>>,
  Expect<Equal<'deserialize' extends keyof typeof Browser.FuzzyObjectIndex ? true : false, false>>,
  Expect<Equal<keyof typeof BrowserObjects, 'FuzzyObjectIndex' | 'searchObjects'>>,
  Expect<Equal<typeof BrowserObjects.FuzzyObjectIndex, typeof Browser.FuzzyObjectIndex>>,
  // MatchType is both the string union and its runtime object.
  Expect<Equal<Browser.MatchType, 'Exact' | 'Prefix' | 'Contains' | 'Fuzzy'>>,
  Expect<Equal<(typeof Browser.MatchType)['Contains'], 'Contains'>>,
  // TypedArray variants.
  Expect<Equal<ReturnType<typeof Browser.levenshteinManyU32>, Uint32Array>>,
  Expect<Equal<ReturnType<typeof Browser.jaroWinklerManyF64>, Float64Array>>,
  Expect<Equal<ReturnType<typeof Browser.hammingManyU32>, Uint32Array>>,
  Expect<Equal<ReturnType<typeof Browser.normalizedHammingManyF64>, Float64Array>>,
  Expect<Equal<Parameters<typeof Browser.hammingManyU32>, Parameters<typeof Browser.hammingMany>>>,
];

// ─── Usage ──────────────────────────────────────────────────────────────────

declare const browser: typeof Browser;

const users = [{ name: 'Jane Doe', address: { city: 'Denver' } }];
const index = new browser.FuzzyObjectIndex(users, { keys: ['name', 'address.city'] });
export const city: string | undefined = index.search('jane')[0]?.item.address.city;
export const closest: { name: string } | null = index.closest('jane');
export const found: number = browser.searchObjects('jane', users, { keys: ['name'] }).length;
export const isIndex: boolean = (index as unknown) instanceof browser.FuzzyObjectIndex;

// @ts-expect-error serialize() is Node.js-only
index.serialize();
// @ts-expect-error deserialize() is Node.js-only
browser.FuzzyObjectIndex.deserialize(new Uint8Array());
// @ts-expect-error keys are required
new browser.FuzzyObjectIndex(users, {});

export const matchType: Browser.MatchType = browser.MatchType.Fuzzy;
export const marked: string = browser.highlight('fuzzy', [0], '<b>', '</b>');
export const distances: Uint32Array = browser.levenshteinManyU32('a', ['a', 'b']);
