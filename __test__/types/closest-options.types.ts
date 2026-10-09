// Type-level tests for the options of `KeyedFuzzyIndex.closest()` and
// `FuzzyObjectIndex.closest()`: `closest(query, options?)`, where `options` is
// a `minScore` number or a `KeyClosestOptions` object. Covers the Node.js
// declarations (index.d.ts, objects.d.ts), the wasm-bindgen glue
// (rapid-fuzzy-wasm-bindgen.d.mts) and the browser entries (browser.d.mts,
// objects.browser.d.mts). Checked by `pnpm run typecheck` under the repo's
// strict tsconfig (including `exactOptionalPropertyTypes`); never executed.
// `@ts-expect-error` marks calls that must be rejected.

import type * as Browser from '../../browser.mjs' with { 'resolution-mode': 'import' };
import {
  type KeyClosestOptions,
  KeyedFuzzyIndex,
  type KeyMatchMode,
  type KeyScoreMode,
  type KeySearchOptions,
} from '../../index.js';
import type * as IndexMjs from '../../index.mjs' with { 'resolution-mode': 'import' };
import type * as BrowserObjects from '../../objects.browser.mjs' with {
  'resolution-mode': 'import',
};
import { FuzzyObjectIndex } from '../../objects.js';
import type * as Wasm from '../../rapid-fuzzy-wasm-bindgen.mjs' with {
  'resolution-mode': 'import',
};

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;

interface User {
  name: string;
  city: string;
}

/** The documented shape: the three `KeySearchOptions` fields closest() reads. */
interface ClosestOptionsShape {
  minScore?: number | undefined;
  scoreMode?: KeyScoreMode | undefined;
  matchMode?: KeyMatchMode | undefined;
}

type ClosestParameters<Options> = [query: string, options?: number | Options | undefined | null];

export type ClosestOptionsChecks = [
  // Exactly minScore, scoreMode and matchMode, typed like their KeySearchOptions fields.
  Expect<Equal<KeyClosestOptions, ClosestOptionsShape>>,
  Expect<Equal<KeyClosestOptions, Pick<KeySearchOptions, 'minScore' | 'scoreMode' | 'matchMode'>>>,
  Expect<Equal<IndexMjs.KeyClosestOptions, KeyClosestOptions>>,
  Expect<Equal<NonNullable<KeyClosestOptions['scoreMode']>, 'weighted' | 'matched' | 'max'>>,
  Expect<Equal<NonNullable<KeyClosestOptions['matchMode']>, 'perKey' | 'crossKey'>>,
  // closest(query, options?) on both Node.js indexes, with the same options type.
  Expect<Equal<Parameters<KeyedFuzzyIndex['closest']>, ClosestParameters<KeyClosestOptions>>>,
  Expect<
    Equal<Parameters<FuzzyObjectIndex<User>['closest']>, ClosestParameters<KeyClosestOptions>>
  >,
  Expect<Equal<ReturnType<KeyedFuzzyIndex['closest']>, number | null>>,
  Expect<Equal<ReturnType<FuzzyObjectIndex<User>['closest']>, User | null>>,
  // The wasm-bindgen glue and the browser entries declare the same API.
  Expect<Equal<Wasm.KeyClosestOptions, KeyClosestOptions>>,
  Expect<Equal<Browser.KeyClosestOptions, KeyClosestOptions>>,
  Expect<
    Equal<
      NonNullable<Parameters<Wasm.KeyedFuzzyIndex['closest']>[1]>,
      number | Wasm.KeyClosestOptions
    >
  >,
  Expect<Equal<Parameters<Wasm.KeyedFuzzyIndex['closest']>['length'], 1 | 2>>,
  Expect<Equal<ReturnType<Wasm.KeyedFuzzyIndex['closest']>, number | null>>,
  Expect<
    Equal<
      Parameters<Browser.FuzzyObjectIndex<User>['closest']>,
      Parameters<FuzzyObjectIndex<User>['closest']>
    >
  >,
  Expect<Equal<typeof BrowserObjects.FuzzyObjectIndex, typeof Browser.FuzzyObjectIndex>>,
];

declare const matrix: ReadonlyArray<readonly string[]>;
declare const users: readonly User[];
declare const wasmKeyed: Wasm.KeyedFuzzyIndex;
declare const browser: typeof Browser;

const keyed = new KeyedFuzzyIndex(matrix, [1]);
const objectIndex = new FuzzyObjectIndex(users, { keys: ['name', 'city'] });
const browserIndex = new browser.FuzzyObjectIndex(users, { keys: ['name', 'city'] });

// ─── Accepted ───────────────────────────────────────────────────────────────

const scoreModes = ['weighted', 'matched', 'max'] as const;
const matchModes = ['perKey', 'crossKey'] as const;
for (const scoreMode of scoreModes) {
  for (const matchMode of matchModes) {
    const options = { minScore: 0.5, scoreMode, matchMode };
    keyed.closest('a', options);
    objectIndex.closest('a', options);
    browserIndex.closest('a', options);
    wasmKeyed.closest('a', options);
  }
}

export const keyedResults: Array<number | null> = [
  keyed.closest('a'),
  keyed.closest('a', undefined),
  keyed.closest('a', null),
  // A number is the minScore shorthand of rapid-fuzzy 2.1.
  keyed.closest('a', 0.5),
  keyed.closest('a', {}),
  keyed.closest('a', { minScore: 0.5 }),
  keyed.closest('a', { scoreMode: 'matched' }),
  keyed.closest('a', { matchMode: 'crossKey' }),
  keyed.closest('a', { minScore: 0.5, scoreMode: 'max', matchMode: 'crossKey' }),
  // exactOptionalPropertyTypes: an explicit undefined means "default".
  keyed.closest('a', { minScore: undefined, scoreMode: undefined, matchMode: undefined }),
  wasmKeyed.closest('a'),
  wasmKeyed.closest('a', undefined),
  wasmKeyed.closest('a', null),
  wasmKeyed.closest('a', 0.5),
  wasmKeyed.closest('a', {}),
  wasmKeyed.closest('a', { minScore: 0.5, scoreMode: 'max', matchMode: 'crossKey' }),
  wasmKeyed.closest('a', { minScore: undefined, scoreMode: undefined, matchMode: undefined }),
];

export const objectResults: Array<User | null> = [
  objectIndex.closest('a'),
  objectIndex.closest('a', null),
  objectIndex.closest('a', 0.5),
  objectIndex.closest('a', {}),
  objectIndex.closest('a', { minScore: 0.5, scoreMode: 'matched', matchMode: 'perKey' }),
  objectIndex.closest('a', { minScore: undefined, scoreMode: undefined, matchMode: undefined }),
  browserIndex.closest('a'),
  browserIndex.closest('a', 0.5),
  browserIndex.closest('a', { scoreMode: 'max', matchMode: 'crossKey' }),
  browserIndex.closest('a', { minScore: undefined, scoreMode: undefined, matchMode: undefined }),
];

// Options typed ahead of time.
const nodeOptions: KeyClosestOptions = { scoreMode: 'max' };
keyed.closest('a', nodeOptions);
objectIndex.closest('a', nodeOptions);
browserIndex.closest('a', nodeOptions);
const wasmOptions: Wasm.KeyClosestOptions = { matchMode: 'crossKey' };
wasmKeyed.closest('a', wasmOptions);
// Options of one build are options of the other.
keyed.closest('a', wasmOptions);
browserIndex.closest('a', wasmOptions);

// ─── Rejected ───────────────────────────────────────────────────────────────

// @ts-expect-error -- not a score mode
keyed.closest('a', { scoreMode: 'mean' });
// @ts-expect-error -- modes are lower-case
keyed.closest('a', { scoreMode: 'Max' });
// @ts-expect-error -- not a match mode
keyed.closest('a', { matchMode: 'cross' });
// @ts-expect-error -- a match mode is not a score mode
keyed.closest('a', { scoreMode: 'crossKey' });
// @ts-expect-error -- maxResults is not a closest() option
keyed.closest('a', { maxResults: 1 });
// @ts-expect-error -- isCaseSensitive is not a closest() option
keyed.closest('a', { minScore: 0.5, isCaseSensitive: true });
// @ts-expect-error -- the modes are no longer positional arguments
keyed.closest('a', 0.5, 'max');
// @ts-expect-error -- the modes are no longer positional arguments
keyed.closest('a', null, 'max', 'crossKey');
// @ts-expect-error -- minScore is a number
keyed.closest('a', '0.5');
// @ts-expect-error -- null fields are rejected at runtime
keyed.closest('a', { minScore: null });
// @ts-expect-error -- null fields are rejected at runtime
keyed.closest('a', { scoreMode: null });
const dynamicMode: string = 'max';
// @ts-expect-error -- a plain string is not a KeyScoreMode
keyed.closest('a', { scoreMode: dynamicMode });

// @ts-expect-error -- not a score mode
objectIndex.closest('a', { scoreMode: 'average' });
// @ts-expect-error -- not a match mode
objectIndex.closest('a', { matchMode: 'terms' });
// @ts-expect-error -- maxResults is not a closest() option
objectIndex.closest('a', { minScore: 0.5, maxResults: 1 });
// @ts-expect-error -- the modes are no longer positional arguments
objectIndex.closest('a', undefined, 'max');
// @ts-expect-error -- the modes are no longer positional arguments
objectIndex.closest('a', 0.5, null, 'crossKey');

// @ts-expect-error -- not a score mode
browserIndex.closest('a', { scoreMode: 'avg' });
// @ts-expect-error -- maxResults is not a closest() option
browserIndex.closest('a', { maxResults: 1 });
// @ts-expect-error -- the modes are no longer positional arguments
browserIndex.closest('a', 0.5, 'max');

// @ts-expect-error -- not a score mode
wasmKeyed.closest('a', { scoreMode: 'avg' });
// @ts-expect-error -- not a match mode
wasmKeyed.closest('a', { matchMode: 'cross' });
// @ts-expect-error -- maxResults is not a closest() option
wasmKeyed.closest('a', { maxResults: 1 });
// @ts-expect-error -- the modes are no longer positional arguments
wasmKeyed.closest('a', null, 'max');
// @ts-expect-error -- the modes are no longer positional arguments
wasmKeyed.closest('a', 0.5, 'max', 'crossKey');
