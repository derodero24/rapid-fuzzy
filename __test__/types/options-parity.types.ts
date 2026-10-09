// Type-level tests for the options objects of the two builds: the Node.js
// declarations (index.d.ts), the wasm-bindgen glue
// (rapid-fuzzy-wasm-bindgen.d.mts) and the browser entry (browser.d.mts)
// declare the same option types, and every optional field accepts an explicit
// `undefined` (`field?: T | undefined`), which both builds treat as "not
// set". Checked by `pnpm run typecheck` under the repo's strict tsconfig
// (including `exactOptionalPropertyTypes`); never executed.
// `@ts-expect-error` marks calls that must be rejected.

import type * as Browser from '../../browser.mjs' with { 'resolution-mode': 'import' };
import type * as Node from '../../index.js';
import type * as Wasm from '../../rapid-fuzzy-wasm-bindgen.mjs' with {
  'resolution-mode': 'import',
};

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;

type MutuallyAssignable<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;

/** `T` with every field present and set to `undefined`. */
type AllUndefined<T> = { [K in keyof T]-?: undefined };

/** Whether every field of `T` accepts an explicit `undefined`. */
type AcceptsUndefined<T> = AllUndefined<T> extends T ? true : false;

/** Whether some field of `T` is required. */
type HasRequiredField<T> = Partial<T> extends T ? false : true;

export type OptionTypeChecks = [
  // The same types in every build.
  Expect<Equal<Wasm.SearchOptions, Node.SearchOptions>>,
  Expect<Equal<Wasm.KeySearchOptions, Node.KeySearchOptions>>,
  Expect<Equal<Wasm.KeyClosestOptions, Node.KeyClosestOptions>>,
  Expect<Equal<Browser.SearchOptions, Node.SearchOptions>>,
  Expect<Equal<Browser.KeySearchOptions, Node.KeySearchOptions>>,
  Expect<Equal<Browser.KeyClosestOptions, Node.KeyClosestOptions>>,
  Expect<MutuallyAssignable<Wasm.SearchOptions, Node.SearchOptions>>,
  Expect<MutuallyAssignable<Wasm.KeySearchOptions, Node.KeySearchOptions>>,
  Expect<MutuallyAssignable<Wasm.KeyClosestOptions, Node.KeyClosestOptions>>,
  Expect<MutuallyAssignable<Browser.ObjectIndexSearchOptions, Node.ObjectIndexSearchOptions>>,
  // Every field is optional and accepts undefined.
  Expect<AcceptsUndefined<Wasm.SearchOptions>>,
  Expect<AcceptsUndefined<Wasm.KeySearchOptions>>,
  Expect<AcceptsUndefined<Wasm.KeyClosestOptions>>,
  Expect<AcceptsUndefined<Browser.SearchOptions>>,
  Expect<AcceptsUndefined<Browser.KeySearchOptions>>,
  Expect<AcceptsUndefined<Browser.KeyClosestOptions>>,
  Expect<AcceptsUndefined<Node.SearchOptions>>,
  Expect<AcceptsUndefined<Node.KeySearchOptions>>,
  Expect<AcceptsUndefined<Node.KeyClosestOptions>>,
  Expect<AcceptsUndefined<Node.ObjectIndexSearchOptions>>,
  Expect<Equal<HasRequiredField<Wasm.SearchOptions>, false>>,
  Expect<Equal<HasRequiredField<Wasm.KeySearchOptions>, false>>,
  Expect<Equal<HasRequiredField<Wasm.KeyClosestOptions>, false>>,
  // The fields are typed exactly, undefined aside.
  Expect<Equal<Wasm.SearchOptions['maxResults'], number | undefined>>,
  Expect<Equal<Wasm.SearchOptions['includePositions'], boolean | undefined>>,
  Expect<Equal<Wasm.KeySearchOptions['scoreMode'], Wasm.KeyScoreMode | undefined>>,
  Expect<Equal<Wasm.KeySearchOptions['matchMode'], Wasm.KeyMatchMode | undefined>>,
  Expect<Equal<Wasm.KeyClosestOptions['minScore'], number | undefined>>,
  // Results are unchanged: an absent matchType is omitted, never undefined.
  Expect<Equal<AcceptsUndefined<Pick<Wasm.SearchResult, 'matchType'>>, false>>,
];

declare const wasm: typeof Wasm;
declare const browser: typeof Browser;
declare const node: typeof Node;
declare const words: readonly string[];
declare const matrix: ReadonlyArray<readonly string[]>;
interface User {
  name: string;
}
declare const users: readonly User[];

// ─── Explicit undefined in every field (exactOptionalPropertyTypes) ─────────

const unsetSearch = {
  maxResults: undefined,
  minScore: undefined,
  includePositions: undefined,
  isCaseSensitive: undefined,
  returnAllOnEmpty: undefined,
};
const unsetModes = { scoreMode: undefined, matchMode: undefined };
const unsetClosest = { minScore: undefined, ...unsetModes };

wasm.search('a', words, unsetSearch);
wasm.search('a', words, { maxResults: undefined });
const wasmIndex = new wasm.FuzzyIndex(words);
wasmIndex.search('a', unsetSearch);
wasmIndex.searchIndices('a', { minScore: undefined, includePositions: undefined });
wasm.searchKeys('a', matrix, [1], { ...unsetSearch, ...unsetModes });
wasm.searchKeys('a', matrix, [1], { scoreMode: undefined });
const wasmKeyed = new wasm.KeyedFuzzyIndex(matrix, [1]);
wasmKeyed.search('a', { ...unsetSearch, ...unsetModes });
wasmKeyed.search('a', { matchMode: undefined, maxResults: 3 });
wasmKeyed.closest('a', unsetClosest);
wasmKeyed.closest('a', { minScore: undefined });

browser.search('a', words, unsetSearch);
browser.searchKeys('a', matrix, [1], { ...unsetSearch, ...unsetModes });
new browser.KeyedFuzzyIndex(matrix, [1]).closest('a', unsetClosest);
browser.searchObjects('a', users, { keys: ['name'], ...unsetSearch, ...unsetModes });
const browserObjects = new browser.FuzzyObjectIndex(users, { keys: ['name'] });
browserObjects.search('a', { maxResults: undefined, scoreMode: undefined });
browserObjects.closest('a', unsetClosest);

// Typed ahead of time.
const wasmSearchOptions: Wasm.SearchOptions = unsetSearch;
const wasmKeySearchOptions: Wasm.KeySearchOptions = { ...unsetSearch, ...unsetModes };
const wasmClosestOptions: Wasm.KeyClosestOptions = unsetClosest;
const browserSearchOptions: Browser.SearchOptions = { minScore: undefined };

// ─── Options typed for one build work with the other ────────────────────────

declare const nodeSearchOptions: Node.SearchOptions;
declare const nodeKeySearchOptions: Node.KeySearchOptions;
declare const nodeClosestOptions: Node.KeyClosestOptions;
wasm.search('a', words, nodeSearchOptions);
wasm.searchKeys('a', matrix, [1], nodeKeySearchOptions);
wasmKeyed.closest('a', nodeClosestOptions);
browser.search('a', words, nodeSearchOptions);
node.search('a', words, wasmSearchOptions);
node.search('a', words, browserSearchOptions);
node.searchKeys('a', matrix, [1], wasmKeySearchOptions);
new node.KeyedFuzzyIndex(matrix, [1]).closest('a', wasmClosestOptions);

// ─── Still rejected ─────────────────────────────────────────────────────────

// @ts-expect-error -- null is not undefined
wasm.search('a', words, { maxResults: null });
// @ts-expect-error -- null fields are rejected at runtime
wasm.searchKeys('a', matrix, [1], { scoreMode: null });
// @ts-expect-error -- null fields are rejected at runtime
wasmKeyed.closest('a', { matchMode: null });
// @ts-expect-error -- maxResults must be a number
wasm.search('a', words, { maxResults: '5' });
// @ts-expect-error -- an unknown field
wasm.search('a', words, { maxResult: undefined });
