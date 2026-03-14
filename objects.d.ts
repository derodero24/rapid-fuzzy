import type { SearchOptions } from './index';

export interface KeyConfig {
  name: string;
  weight?: number;
}

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
