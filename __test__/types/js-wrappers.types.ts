// Type-level tests for the hand-written wrappers (objects.d.ts, highlight.d.ts)
// and the patched napi declarations (index.d.ts). Checked by
// `pnpm run typecheck` under the repo's strict tsconfig (including
// `exactOptionalPropertyTypes` and `isolatedModules`); never executed.
// `@ts-expect-error` marks calls that must be rejected.

import { type HighlightOptions, highlight, highlightRanges } from '../../highlight.js';
import type * as rootEntry from '../../index.js';
import {
  closest,
  FuzzyIndex,
  KeyedFuzzyIndex,
  levenshteinMany,
  MatchType,
  type SearchOptions,
  search,
  searchKeys,
} from '../../index.js';
import {
  FuzzyObjectIndex,
  type KeyConfig,
  type KeyPath,
  type ObjectIndexSearchOptions,
  type ObjectSearchOptions,
  type ObjectSearchResult,
  searchObjects,
} from '../../objects.js';

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;

interface User {
  name: string;
  email: string;
  age: number;
  address: { city: string; geo?: { lat: number } | undefined };
  tags: readonly string[];
  meta: Record<string, string>;
  greet(): string;
}

declare const user: User;
const users: readonly User[] = [user];

// ─── Readonly inputs ────────────────────────────────────────────────────────

const words: readonly string[] = ['apple', 'banana'];
const matrix: ReadonlyArray<readonly string[]> = [words];
search('a', words);
search('a', words, 5);
closest('a', words);
levenshteinMany('a', words);
searchKeys('a', matrix, [1] as const);
const fuzzyIndex = new FuzzyIndex(words);
fuzzyIndex.addMany(words);
void FuzzyIndex.fromAsync(words);
const keyed = new KeyedFuzzyIndex(matrix, [1] as const);
keyed.add(words);
keyed.addMany(matrix);
searchObjects('a', users, { keys: ['name'] });
const objectIndex = new FuzzyObjectIndex(users, { keys: ['name'] as const });
objectIndex.addMany(users);
highlight('apple', [0, 1] as const, '<b>', '</b>');
highlightRanges('apple', [0] as const);

// ─── exactOptionalPropertyTypes: optional fields accept `undefined` ─────────

const searchOptions: SearchOptions = {
  maxResults: undefined,
  minScore: undefined,
  includePositions: undefined,
  isCaseSensitive: undefined,
  returnAllOnEmpty: undefined,
};
search('a', words, searchOptions);
const indexSearchOptions: ObjectIndexSearchOptions = { maxResults: undefined, minScore: undefined };
objectIndex.search('a', indexSearchOptions);
const keyConfig: KeyConfig<User> = { name: 'email', weight: undefined };
const highlightOptions: HighlightOptions = { escapeHtml: undefined };
highlight('a', [0], '<b>', '</b>', highlightOptions);
objectIndex.closest('a', undefined);

// ─── maxResults shorthand ───────────────────────────────────────────────────

keyed.search('a', 5);
objectIndex.search('a', 5);
objectIndex.search('a', null);
// @ts-expect-error -- a string is not a valid options argument
objectIndex.search('a', 'x');
// @ts-expect-error -- searchObjects needs an options object carrying the keys
searchObjects('a', users, 5);

// ─── Key paths ──────────────────────────────────────────────────────────────

type UserPath = KeyPath<User>;

export type KeyPathChecks = [
  Expect<Equal<Extract<UserPath, 'name'>, 'name'>>,
  Expect<Equal<Extract<UserPath, 'address.city'>, 'address.city'>>,
  // Paths are listed two segments deep; longer ones need a valid prefix.
  Expect<Equal<'address.geo.lat' extends UserPath ? true : false, true>>,
  Expect<Equal<'address.zip.code' extends UserPath ? true : false, false>>,
  Expect<Equal<Extract<UserPath, 'tags'>, 'tags'>>,
  Expect<Equal<'tags.0' extends UserPath ? true : false, true>>,
  Expect<Equal<'meta.anything' extends UserPath ? true : false, true>>,
  // Methods are not key paths.
  Expect<Equal<Extract<UserPath, 'greet'>, never>>,
  Expect<Equal<'nmae' extends UserPath ? true : false, false>>,
  // Wide types accept any string.
  // biome-ignore lint/suspicious/noExplicitAny: checking the fallback for `any`
  Expect<Equal<KeyPath<any>, string>>,
  Expect<Equal<KeyPath<unknown>, string>>,
  Expect<Equal<KeyPath<object>, string>>,
  Expect<Equal<KeyPath<Record<string, number>>, string>>,
  // Generic defaults keep the previous, unchecked shapes.
  Expect<Equal<KeyConfig['name'], string>>,
  Expect<Equal<ObjectSearchOptions['keys'], ReadonlyArray<string | KeyConfig>>>,
];

searchObjects('a', users, { keys: ['name', 'address.city', { name: 'email', weight: 2 }] });
searchObjects('a', users, { keys: ['tags', 'tags.1', 'meta.team'], maxResults: 3 });
// @ts-expect-error -- typo in a key name
searchObjects('a', users, { keys: ['nmae'] });
// @ts-expect-error -- typo in a key config name
searchObjects('a', users, { keys: [{ name: 'emial', weight: 2 }] });
// @ts-expect-error -- nested path that does not exist
searchObjects('a', users, { keys: ['address.zip'] });
// @ts-expect-error -- methods are not searchable keys
searchObjects('a', users, { keys: ['greet'] });

new FuzzyObjectIndex(users, { keys: ['email', { name: 'address.geo.lat', weight: 0.5 }] });
// @ts-expect-error -- typo in a key name
new FuzzyObjectIndex(users, { keys: ['emial'] });

// Keys typed as plain strings (e.g. from configuration) are still accepted.
const dynamicKeys: string[] = ['name'];
searchObjects('a', users, { keys: dynamicKeys });
new FuzzyObjectIndex(users, { keys: dynamicKeys });
const declaredOptions: ObjectSearchOptions = { keys: ['name'], maxResults: 1 };
searchObjects('a', users, declaredOptions);
const typedOptions: ObjectSearchOptions<User> = { keys: ['address.city'] };
searchObjects('a', users, typedOptions);
// @ts-expect-error -- ObjectSearchOptions<User> checks its keys too
const badTypedOptions: ObjectSearchOptions<User> = { keys: ['nmae'] };
void badTypedOptions;

// Paths written in a call are checked segment by segment at any depth, and
// recursive item types are fine.
interface TreeNode {
  label: string;
  parent?: TreeNode | undefined;
  children: TreeNode[];
}
declare const nodes: TreeNode[];
searchObjects('a', nodes, {
  keys: ['label', 'parent.parent.parent.parent.label', 'children.0.children.1.label'],
});
// @ts-expect-error -- typo four levels down
searchObjects('a', nodes, { keys: ['parent.parent.parent.lable'] });
// @ts-expect-error -- path continues past a string value
searchObjects('a', nodes, { keys: ['label.length'] });

// ─── Result and instance types ──────────────────────────────────────────────

const results = searchObjects('a', users, { keys: ['name'] });
const inferredIndex = new FuzzyObjectIndex(users, { keys: ['name'] });
const annotatedIndex: FuzzyObjectIndex<User> = inferredIndex;
const restored = FuzzyObjectIndex.deserialize<User>(new Uint8Array(0));

export type ResultChecks = [
  Expect<Equal<typeof results, Array<ObjectSearchResult<User>>>>,
  Expect<Equal<ReturnType<typeof annotatedIndex.search>, Array<ObjectSearchResult<User>>>>,
  Expect<Equal<ReturnType<typeof annotatedIndex.closest>, User | null>>,
  Expect<Equal<ReturnType<typeof restored.closest>, User | null>>,
  Expect<Equal<ObjectSearchResult<User>['item'], User>>,
];

FuzzyObjectIndex.deserialize(new ArrayBuffer(0));
FuzzyObjectIndex.deserialize(new DataView(new ArrayBuffer(0)));
FuzzyObjectIndex.deserialize(Buffer.alloc(0));
// @ts-expect-error -- not binary data
FuzzyObjectIndex.deserialize('serialized');

// ─── CommonJS entry exposes the object wrappers ─────────────────────────────

export type EntryChecks = [
  Expect<Equal<typeof rootEntry.searchObjects, typeof searchObjects>>,
  Expect<Equal<typeof rootEntry.FuzzyObjectIndex, typeof FuzzyObjectIndex>>,
];

// ─── MatchType is usable under isolatedModules ──────────────────────────────

const exact: MatchType = MatchType.Exact;
const matchTypes: readonly MatchType[] = [
  MatchType.Exact,
  MatchType.Prefix,
  MatchType.Contains,
  MatchType.Fuzzy,
];
void exact;
void matchTypes;
void keyConfig;

// ─── highlight() overloads ──────────────────────────────────────────────────

highlight('a', [0], (s) => `<i>${s}</i>`, { escapeHtml: true });
// @ts-expect-error -- unknown option
highlight('a', [0], '<b>', '</b>', { escape: true });
