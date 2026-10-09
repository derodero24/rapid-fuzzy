// Type-level tests for the `matchMode` option of multi-key search (#782):
// the Node.js declarations (index.d.ts, objects.d.ts), the wasm-bindgen glue
// (rapid-fuzzy-wasm-bindgen.d.mts) and the browser entry (browser.d.mts).
// Checked by `pnpm run typecheck` under the repo's strict tsconfig; never
// executed. `@ts-expect-error` marks calls that must be rejected.

import type * as Browser from '../../browser.mjs' with { 'resolution-mode': 'import' };
import {
  FuzzyIndex,
  KeyedFuzzyIndex,
  type KeyMatchMode,
  type KeySearchOptions,
  type SearchOptions,
  search,
  searchKeys,
} from '../../index.js';
import type * as IndexMjs from '../../index.mjs' with { 'resolution-mode': 'import' };
import {
  FuzzyObjectIndex,
  type ObjectIndexSearchOptions,
  type ObjectSearchOptions,
  searchObjects,
} from '../../objects.js';
import type * as Wasm from '../../rapid-fuzzy-wasm-bindgen.mjs' with {
  'resolution-mode': 'import',
};

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;

type Modes = 'perKey' | 'crossKey';

export type MatchModeTypeChecks = [
  // Exactly the two string literals, in every build.
  Expect<Equal<KeyMatchMode, Modes>>,
  Expect<Equal<IndexMjs.KeyMatchMode, Modes>>,
  Expect<Equal<Wasm.KeyMatchMode, Modes>>,
  Expect<Equal<Browser.KeyMatchMode, Modes>>,
  Expect<Equal<KeySearchOptions['matchMode'], KeyMatchMode | undefined>>,
  Expect<Equal<ObjectSearchOptions['matchMode'], KeyMatchMode | undefined>>,
  Expect<Equal<ObjectIndexSearchOptions['matchMode'], KeyMatchMode | undefined>>,
  Expect<Equal<NonNullable<Wasm.KeySearchOptions['matchMode']>, Modes>>,
  // Plain search options have no matchMode.
  Expect<Equal<'matchMode' extends keyof SearchOptions ? true : false, false>>,
  Expect<Equal<'matchMode' extends keyof Wasm.SearchOptions ? true : false, false>>,
  // The closest() overloads take the mode as their fourth argument.
  Expect<Equal<Parameters<KeyedFuzzyIndex['closest']>[3], KeyMatchMode | undefined | null>>,
  Expect<
    Equal<Parameters<FuzzyObjectIndex<unknown>['closest']>[3], KeyMatchMode | undefined | null>
  >,
  Expect<Equal<NonNullable<Parameters<Wasm.KeyedFuzzyIndex['closest']>[3]>, Modes>>,
];

declare const matrix: ReadonlyArray<readonly string[]>;
declare const words: readonly string[];
interface Resident {
  name: string;
  city: string;
}
declare const residents: readonly Resident[];

const keyed = new KeyedFuzzyIndex(matrix, [1]);
const objectIndex = new FuzzyObjectIndex(residents, { keys: ['name', 'city'] });
const modes: readonly KeyMatchMode[] = ['perKey', 'crossKey'];

// ─── Every mode is accepted wherever keyed search takes options ─────────────

for (const matchMode of modes) {
  searchKeys('a', matrix, [1], { matchMode });
  keyed.search('a', { matchMode, scoreMode: 'max', minScore: 0.5, maxResults: 3 });
  keyed.closest('a', 0.5, 'matched', matchMode);
  keyed.closest('a', undefined, undefined, matchMode);
  keyed.closest('a', null, null, matchMode);
  searchObjects('a', residents, { keys: ['name', 'city'], matchMode });
  objectIndex.search('a', { matchMode });
  objectIndex.closest('a', undefined, undefined, matchMode);
}
searchKeys('john tokyo', matrix, [1], { matchMode: 'crossKey' });
keyed.search('a', { matchMode: 'perKey' });
searchObjects('john tokyo', residents, { keys: ['name', 'city'], matchMode: 'crossKey' });
objectIndex.search('a', { matchMode: 'crossKey', scoreMode: 'matched' });
// exactOptionalPropertyTypes: an explicit undefined means "default".
searchKeys('a', matrix, [1], { matchMode: undefined });
objectIndex.search('a', { matchMode: undefined });
keyed.closest('a', 0.5, 'max', undefined);
keyed.closest('a', 0.5, 'max', null);
objectIndex.closest('a', 0.5, null, null);

// ─── Anything else is rejected ──────────────────────────────────────────────

// @ts-expect-error -- not a match mode
searchKeys('a', matrix, [1], { matchMode: 'cross' });
// @ts-expect-error -- modes are camelCase
keyed.search('a', { matchMode: 'CrossKey' });
// @ts-expect-error -- not a match mode
keyed.closest('a', 0.5, 'max', 'any');
// @ts-expect-error -- not a string
keyed.closest('a', 0.5, 'max', true);
// @ts-expect-error -- not a match mode
searchObjects('a', residents, { keys: ['name'], matchMode: 'perkey' });
// @ts-expect-error -- not a match mode
objectIndex.search('a', { matchMode: '' });
// @ts-expect-error -- not a match mode
objectIndex.closest('a', undefined, undefined, 'terms');
// @ts-expect-error -- a score mode is not a match mode
objectIndex.closest('a', undefined, 'crossKey');
const dynamicMode: string = 'crossKey';
// @ts-expect-error -- a plain string is not a KeyMatchMode
searchKeys('a', matrix, [1], { matchMode: dynamicMode });

// ─── Not an option of single-key search ─────────────────────────────────────

// @ts-expect-error -- search() has no matchMode
search('a', words, { matchMode: 'crossKey' });
// @ts-expect-error -- FuzzyIndex.search() has no matchMode
new FuzzyIndex(words).search('a', { matchMode: 'crossKey' });

// ─── wasm-bindgen glue ──────────────────────────────────────────────────────

declare const wasmKeyed: Wasm.KeyedFuzzyIndex;
declare const wasmSearchKeys: typeof Wasm.searchKeys;
declare const wasmSearch: typeof Wasm.search;
wasmSearchKeys('a', matrix, [1], { matchMode: 'crossKey' });
wasmKeyed.search('a', { matchMode: 'crossKey', scoreMode: 'max', maxResults: 2 });
wasmKeyed.closest('a', null, null, 'crossKey');
wasmKeyed.closest('a', 0.5, 'matched', 'perKey');
// @ts-expect-error -- not a match mode
wasmSearchKeys('a', matrix, [1], { matchMode: 'cross' });
// @ts-expect-error -- not a match mode
wasmKeyed.closest('a', null, null, 'cross');
// @ts-expect-error -- search() has no matchMode
wasmSearch('a', words, { matchMode: 'crossKey' });
