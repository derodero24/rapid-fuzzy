import type { SearchOptions } from './index';

export interface KeyConfig {
  name: string;
  weight?: number;
}

/**
 * Options for searchObjects(). Extends SearchOptions with key configuration.
 * Note: `includePositions` has no effect for multi-key search.
 */
export interface ObjectSearchOptions extends SearchOptions {
  keys: Array<string | KeyConfig>;
}

export interface ObjectSearchResult<T> {
  item: T;
  index: number;
  score: number;
  keyScores: Array<number>;
}

/**
 * Perform fuzzy search across object arrays with weighted keys.
 *
 * Wraps `searchKeys()` with an ergonomic API that accepts row-oriented
 * objects and returns matched items directly.
 *
 * @example
 * ```typescript
 * const users = [
 *   { name: 'John Smith', email: 'john@example.com' },
 *   { name: 'Jane Doe', email: 'jane@example.com' },
 * ];
 *
 * const results = searchObjects('john', users, {
 *   keys: [{ name: 'name', weight: 2.0 }, 'email'],
 * });
 * // results[0].item → { name: 'John Smith', email: 'john@example.com' }
 * ```
 */
export declare function searchObjects<T>(
  query: string,
  items: Array<T>,
  options: ObjectSearchOptions,
): Array<ObjectSearchResult<T>>;

export interface ObjectIndexOptions {
  keys: Array<string | KeyConfig>;
}

export interface ObjectIndexSearchOptions {
  maxResults?: number;
  minScore?: number;
  isCaseSensitive?: boolean;
  returnAllOnEmpty?: boolean;
}

/**
 * A persistent fuzzy search index for object collections with weighted keys.
 *
 * Pre-computes key texts and stores them on the Rust side for fast repeated
 * searches. Use this when searching the same collection multiple times.
 *
 * @example
 * ```typescript
 * const index = new FuzzyObjectIndex(users, {
 *   keys: [{ name: 'name', weight: 2.0 }, 'email'],
 * });
 *
 * const results = index.search('john');
 * // results[0].item → { name: 'John Smith', ... }
 *
 * index.add({ name: 'New User', email: 'new@example.com' });
 * index.destroy(); // free Rust-side memory
 * ```
 */
export declare class FuzzyObjectIndex<T> {
  constructor(items: Array<T>, options: ObjectIndexOptions);

  /** Number of items in the index. */
  get size(): number;

  /** Search for objects matching the query. */
  search(
    query: string,
    options?: ObjectIndexSearchOptions,
  ): Array<ObjectSearchResult<T>>;

  /** Find the closest matching object, or null if no match. */
  closest(query: string, minScore?: number): T | null;

  /** Add a single item to the index. */
  add(item: T): void;

  /** Add multiple items at once. */
  addMany(items: Array<T>): void;

  /** Remove the item at the given index (swap-remove semantics). */
  remove(index: number): boolean;

  /** Free all internal data. */
  destroy(): void;
}
