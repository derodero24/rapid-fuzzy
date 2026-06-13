import type { SearchOptions } from './index';

export interface KeyConfig {
  /**
   * Object key to search. Supports dot-separated paths for nested values,
   * e.g. `'address.city'`.
   */
  name: string;
  /** Relative weight of this key in the combined score. Defaults to `1.0`. */
  weight?: number;
}

/**
 * Options for searchObjects(). Extends SearchOptions with key configuration.
 *
 * Note: `includePositions` has no effect for multi-key search — match positions
 * are per-key and are not merged, so the `positions`/`matchType` fields from
 * single-key {@link SearchOptions} are not populated. All other SearchOptions
 * fields (maxResults, minScore, isCaseSensitive, returnAllOnEmpty) apply.
 */
export interface ObjectSearchOptions extends SearchOptions {
  keys: Array<string | KeyConfig>;
}

export interface ObjectSearchResult<T> {
  /** The matched object from the original input array. */
  item: T;
  /** Index of the matched item in the original input array. */
  index: number;
  /** Combined weighted score normalized to the 0.0–1.0 range. */
  score: number;
  /**
   * Per-key scores in the same order as the configured keys.
   * A score of 0.0 means the item did not match on that key.
   */
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
  search(query: string, options?: ObjectIndexSearchOptions): Array<ObjectSearchResult<T>>;

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

  /**
   * Serialize the index and its items to a Buffer.
   *
   * Items must be JSON-serializable. Pass the result to
   * `FuzzyObjectIndex.deserialize()` to reconstruct the index.
   */
  serialize(): Buffer;

  /**
   * Reconstruct a `FuzzyObjectIndex` from a Buffer produced by `serialize()`.
   */
  static deserialize<T>(buffer: Buffer): FuzzyObjectIndex<T>;
}
