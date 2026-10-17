// Type-checks the browser / edge declarations the way a strict browser-only
// TypeScript project sees them, which the repo's own tsconfig (skipLibCheck,
// @types/node) cannot: every entry is imported by the package's own name,
// with no Node.js types and skipLibCheck off, so that the shipped .d.ts /
// .d.mts files are checked too.
//
// - tsconfig.json: the "browser" export condition, lib ES2022 + DOM
// - tsconfig.workerd.json: the "workerd" export condition (Cloudflare Workers)
// - tsconfig.esnext.json: lib ESNext, which declares Symbol.dispose itself
//
// Run by `pnpm run typecheck`; never executed.
import {
  closest,
  FuzzyIndex,
  KeyedFuzzyIndex,
  type KeySearchOptions,
  levenshtein,
  levenshteinManyU32,
  MatchType,
  type SearchResult,
  search,
} from 'rapid-fuzzy';
import { highlight, highlightRanges } from 'rapid-fuzzy/highlight';
import { FuzzyObjectIndex, type ObjectSearchResult, searchObjects } from 'rapid-fuzzy/objects';

const items = ['apple', 'banana', 'cherry'];

export const results: SearchResult[] = search('app', items, { maxResults: 5 });
export const best: string | null = closest('app', items);
export const distance: number = levenshtein('kitten', 'sitting');
export const distances: Uint32Array = levenshteinManyU32('apple', items);
export const exact: MatchType = MatchType.Exact;

const index = new FuzzyIndex(items);
export const found: number = index.search('app').length;
index[Symbol.dispose]();

const options: KeySearchOptions = { maxResults: 1, scoreMode: 'max', matchMode: 'crossKey' };
const keyed = new KeyedFuzzyIndex([items, items], [2, 1]);
export const keyedResults: number = keyed.search('app', options).length;
// The browser build's serialize() returns a Uint8Array.
export const serialized: Uint8Array = keyed.serialize();
keyed.free();

const people = [{ name: 'Ada Lovelace', city: 'London' }];
const peopleIndex = new FuzzyObjectIndex(people, { keys: ['name', 'city'] });
export const person: { name: string; city: string } | null = peopleIndex.closest('ada');
export const objectResults: ObjectSearchResult<{ name: string; city: string }>[] = searchObjects(
  'ada',
  people,
  { keys: ['name'] },
);

export const marked: string = highlight('apple', [0, 1], '<b>', '</b>');
export const ranges: number = highlightRanges('apple', [0]).length;
