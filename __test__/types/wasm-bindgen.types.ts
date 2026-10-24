// Type-level tests for the generated wasm-bindgen declarations
// (rapid-fuzzy-wasm-bindgen.d.mts). Checked by `pnpm run typecheck` under the
// repo's strict tsconfig; never executed.
import type * as Wasm from '../../rapid-fuzzy-wasm-bindgen.mjs' with {
  'resolution-mode': 'import',
};
import type {
  IndexSearchResult,
  KeySearchResult,
  MatchType,
  SearchOptions,
  SearchResult,
} from '../../rapid-fuzzy-wasm-bindgen.mjs' with { 'resolution-mode': 'import' };

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;

// ─── No `any` in any exported signature ─────────────────────────────────────

type IsAny<T> = 0 extends 1 & T ? true : false;
type AnyInTuple<T extends readonly unknown[]> = true extends {
  [K in keyof T]: IsAny<T[K]>;
}[number]
  ? true
  : false;
type AnyInFunction<F> = F extends (...args: infer A) => infer R
  ? IsAny<R> extends true
    ? true
    : AnyInTuple<A>
  : false;
type AnyInMembers<T> = true extends {
  [K in keyof T]: T[K] extends (...args: never) => unknown ? AnyInFunction<T[K]> : IsAny<T[K]>;
}[keyof T]
  ? true
  : false;
type AnyInExport<E> = E extends abstract new (
  ...args: infer A
) => infer I
  ? true extends AnyInTuple<A> | AnyInMembers<I> | AnyInMembers<E>
    ? true
    : false
  : E extends (...args: never) => unknown
    ? AnyInFunction<E>
    : IsAny<E>;
type ExportsWithAny = {
  [K in keyof typeof Wasm]: AnyInExport<(typeof Wasm)[K]> extends true ? K : never;
}[keyof typeof Wasm];

type AnyInObject<T> = true extends { [K in keyof T]-?: IsAny<T[K]> }[keyof T] ? true : false;

// ─── Declared shapes ────────────────────────────────────────────────────────

export type DeclarationChecks = [
  Expect<Equal<ExportsWithAny, never>>,
  Expect<Equal<AnyInObject<SearchOptions>, false>>,
  Expect<Equal<AnyInObject<SearchResult>, false>>,
  Expect<Equal<AnyInObject<IndexSearchResult>, false>>,
  Expect<Equal<AnyInObject<KeySearchResult>, false>>,
  Expect<Equal<MatchType, 'Exact' | 'Prefix' | 'Contains' | 'Fuzzy'>>,
  // Every option is optional (#727): `{}` is a complete SearchOptions.
  Expect<Equal<SearchOptions, Partial<SearchOptions>>>,
  Expect<Equal<SearchResult['matchType'], MatchType | undefined>>,
  // Return types (#728).
  Expect<Equal<ReturnType<typeof Wasm.search>, SearchResult[]>>,
  Expect<Equal<ReturnType<typeof Wasm.closest>, string | null>>,
  Expect<Equal<ReturnType<typeof Wasm.searchKeys>, KeySearchResult[]>>,
  Expect<Equal<ReturnType<Wasm.FuzzyIndex['search']>, SearchResult[]>>,
  Expect<Equal<ReturnType<Wasm.FuzzyIndex['searchIndices']>, IndexSearchResult[]>>,
  Expect<Equal<ReturnType<Wasm.FuzzyIndex['closest']>, string | null>>,
  Expect<Equal<ReturnType<typeof Wasm.FuzzyIndex.fromAsync>, Promise<Wasm.FuzzyIndex>>>,
  Expect<Equal<ReturnType<Wasm.KeyedFuzzyIndex['search']>, KeySearchResult[]>>,
  Expect<Equal<ReturnType<Wasm.KeyedFuzzyIndex['closest']>, number | null>>,
  Expect<Equal<ReturnType<typeof Wasm.hamming>, number | null>>,
  Expect<Equal<ReturnType<typeof Wasm.normalizedHamming>, number | null>>,
  Expect<Equal<ReturnType<typeof Wasm.hammingBatch>, (number | null)[]>>,
  Expect<Equal<ReturnType<typeof Wasm.hammingMany>, (number | null)[]>>,
  Expect<Equal<ReturnType<typeof Wasm.normalizedHammingBatch>, (number | null)[]>>,
  Expect<Equal<ReturnType<typeof Wasm.normalizedHammingMany>, (number | null)[]>>,
  Expect<Equal<ReturnType<typeof Wasm.levenshteinBatch>, Uint32Array>>,
  Expect<Equal<ReturnType<typeof Wasm.levenshteinMany>, Uint32Array>>,
  Expect<Equal<ReturnType<typeof Wasm.jaroWinklerMany>, Float64Array>>,
];

// ─── Call sites a strict consumer writes ────────────────────────────────────

declare const wasm: typeof Wasm;

export function exerciseSearch(items: string[]): SearchResult[] {
  const options: SearchOptions = { maxResults: 5 };
  const empty: SearchOptions = {};
  wasm.search('query', items, options);
  wasm.search('query', items, empty);
  wasm.search('query', items, null);
  wasm.search('query', items);
  // @ts-expect-error -- maxResults must be a number
  wasm.search('query', items, { maxResults: '5' });
  // @ts-expect-error -- a string is neither maxResults nor SearchOptions
  wasm.search('query', items, '5');
  // maxResults shorthand, like the Node.js binding.
  return wasm.search('query', items, 5);
}

export function exerciseIndexes(items: string[]): number | null {
  const index = new wasm.FuzzyIndex(items);
  const best: string | null = index.closest('query', 0.5);
  const hits: IndexSearchResult[] = index.searchIndices('query', 3);
  const restored: Wasm.FuzzyIndex = wasm.FuzzyIndex.deserialize(index.serialize());
  void best;
  void hits;
  void restored;

  const keyTexts = [items, items];
  // Weights accept plain arrays (like the Node.js binding) and Float64Array.
  const keyed = new wasm.KeyedFuzzyIndex(keyTexts, [2, 1]);
  const keyedFromTyped = new wasm.KeyedFuzzyIndex(keyTexts, new Float64Array([2, 1]));
  keyed.add(['name', 'email']);
  keyed.addMany([['name', 'email']]);
  // @ts-expect-error -- key values are strings
  keyed.add([1, 2]);
  void keyedFromTyped;
  const results: KeySearchResult[] = wasm.searchKeys('query', keyTexts, [1, 1], {
    maxResults: 3,
  });
  void results;
  return keyed.closest('query');
}

export function exerciseDistance(): (number | null)[] {
  const pairs = [
    ['kitten', 'sitting'],
    ['flaw', 'lawn'],
  ];
  const distances: Uint32Array = wasm.levenshteinBatch(pairs);
  const capped: Uint32Array = wasm.levenshteinMany('kitten', ['sitting'], 2);
  const similar: Float64Array = wasm.jaroWinklerMany('kitten', ['sitting'], 0.8);
  const single: number | null = wasm.hamming('abc', 'abd');
  void distances;
  void capped;
  void similar;
  void single;
  return wasm.hammingMany('abc', ['abd', 'abcd'], 1);
}
