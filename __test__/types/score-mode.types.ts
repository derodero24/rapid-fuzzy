// Type-level tests for the `scoreMode` option of multi-key search (#781):
// the Node.js declarations (index.d.ts, objects.d.ts), the wasm-bindgen glue
// (rapid-fuzzy-wasm-bindgen.d.mts) and the browser entry (browser.d.mts).
// Checked by `pnpm run typecheck` under the repo's strict tsconfig; never
// executed. `@ts-expect-error` marks calls that must be rejected.

import type * as Browser from '../../browser.mjs' with { 'resolution-mode': 'import' };
import {
  FuzzyIndex,
  KeyedFuzzyIndex,
  type KeyScoreMode,
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

type Modes = 'weighted' | 'matched' | 'max';

export type ScoreModeTypeChecks = [
  // Exactly the three string literals, in every build.
  Expect<Equal<KeyScoreMode, Modes>>,
  Expect<Equal<IndexMjs.KeyScoreMode, Modes>>,
  Expect<Equal<Wasm.KeyScoreMode, Modes>>,
  Expect<Equal<Browser.KeyScoreMode, Modes>>,
  Expect<Equal<KeySearchOptions['scoreMode'], KeyScoreMode | undefined>>,
  Expect<Equal<ObjectSearchOptions['scoreMode'], KeyScoreMode | undefined>>,
  Expect<Equal<ObjectIndexSearchOptions['scoreMode'], KeyScoreMode | undefined>>,
  Expect<Equal<NonNullable<Wasm.KeySearchOptions['scoreMode']>, Modes>>,
  // Plain search options have no scoreMode.
  Expect<Equal<'scoreMode' extends keyof SearchOptions ? true : false, false>>,
  Expect<Equal<'scoreMode' extends keyof Wasm.SearchOptions ? true : false, false>>,
  // The closest() overloads take the mode as their third argument.
  Expect<Equal<Parameters<KeyedFuzzyIndex['closest']>[2], KeyScoreMode | undefined | null>>,
  Expect<
    Equal<Parameters<FuzzyObjectIndex<unknown>['closest']>[2], KeyScoreMode | undefined | null>
  >,
  Expect<Equal<NonNullable<Parameters<Wasm.KeyedFuzzyIndex['closest']>[2]>, Modes>>,
];

declare const matrix: ReadonlyArray<readonly string[]>;
declare const words: readonly string[];
interface User {
  name: string;
  email: string;
}
declare const users: readonly User[];

const keyed = new KeyedFuzzyIndex(matrix, [1]);
const objectIndex = new FuzzyObjectIndex(users, { keys: ['name', 'email'] });
const modes: readonly KeyScoreMode[] = ['weighted', 'matched', 'max'];

// ─── Every mode is accepted wherever keyed search takes options ─────────────

for (const scoreMode of modes) {
  searchKeys('a', matrix, [1], { scoreMode });
  keyed.search('a', { scoreMode, minScore: 0.5, maxResults: 3 });
  keyed.closest('a', 0.5, scoreMode);
  keyed.closest('a', undefined, scoreMode);
  keyed.closest('a', null, scoreMode);
  searchObjects('a', users, { keys: ['name'], scoreMode });
  objectIndex.search('a', { scoreMode });
  objectIndex.closest('a', undefined, scoreMode);
}
searchKeys('a', matrix, [1], { scoreMode: 'matched' });
keyed.search('a', { scoreMode: 'max' });
searchObjects('a', users, { keys: ['email'], scoreMode: 'weighted' });
objectIndex.search('a', { scoreMode: 'matched' });
// exactOptionalPropertyTypes: an explicit undefined means "default".
searchKeys('a', matrix, [1], { scoreMode: undefined });
objectIndex.search('a', { scoreMode: undefined });
keyed.closest('a', 0.5, undefined);
keyed.closest('a', 0.5, null);
objectIndex.closest('a', 0.5, null);

// ─── Anything else is rejected ──────────────────────────────────────────────

// @ts-expect-error -- not a score mode
searchKeys('a', matrix, [1], { scoreMode: 'mean' });
// @ts-expect-error -- modes are lower-case
keyed.search('a', { scoreMode: 'Max' });
// @ts-expect-error -- not a score mode
keyed.closest('a', 0.5, 'sum');
// @ts-expect-error -- not a string
keyed.closest('a', 0.5, 1);
// @ts-expect-error -- not a score mode
searchObjects('a', users, { keys: ['name'], scoreMode: 'average' });
// @ts-expect-error -- not a score mode
objectIndex.search('a', { scoreMode: '' });
// @ts-expect-error -- not a score mode
objectIndex.closest('a', undefined, 'matches');
const dynamicMode: string = 'max';
// @ts-expect-error -- a plain string is not a KeyScoreMode
searchKeys('a', matrix, [1], { scoreMode: dynamicMode });

// ─── Not an option of single-key search ─────────────────────────────────────

// @ts-expect-error -- search() has no scoreMode
search('a', words, { scoreMode: 'max' });
// @ts-expect-error -- FuzzyIndex.search() has no scoreMode
new FuzzyIndex(words).search('a', { scoreMode: 'max' });
// @ts-expect-error -- FuzzyIndex.searchIndices() has no scoreMode
new FuzzyIndex(words).searchIndices('a', { scoreMode: 'max' });

// ─── SearchOptions values are still accepted by keyed search ────────────────

declare const searchOptions: SearchOptions;
searchKeys('a', matrix, [1], searchOptions);
keyed.search('a', searchOptions);
searchKeys('a', matrix, [1], { includePositions: true, isCaseSensitive: true });
searchObjects('a', users, { keys: ['name'], includePositions: false });
const keyOptions: KeySearchOptions = { ...searchOptions, scoreMode: 'matched' };
const asSearchOptions: SearchOptions = keyOptions;
void asSearchOptions;

// ─── wasm-bindgen glue ──────────────────────────────────────────────────────

declare const wasmKeyed: Wasm.KeyedFuzzyIndex;
declare const wasmSearchKeys: typeof Wasm.searchKeys;
declare const wasmSearch: typeof Wasm.search;
wasmSearchKeys('a', matrix, [1], { scoreMode: 'matched' });
wasmKeyed.search('a', { scoreMode: 'max', maxResults: 2 });
wasmKeyed.closest('a');
wasmKeyed.closest('a', 0.5);
wasmKeyed.closest('a', null, 'max');
// @ts-expect-error -- not a score mode
wasmSearchKeys('a', matrix, [1], { scoreMode: 'avg' });
// @ts-expect-error -- not a score mode
wasmKeyed.closest('a', null, 'avg');
// @ts-expect-error -- search() has no scoreMode
wasmSearch('a', words, { scoreMode: 'max' });
