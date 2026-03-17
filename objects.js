'use strict';

const { searchKeys, KeyedFuzzyIndex } = require('./index.js');

/**
 * Get a nested property value from an object using a dot-separated path.
 * @param {Record<string, unknown>} obj
 * @param {string} path
 * @returns {string}
 */
function getNestedValue(obj, path) {
  let current = obj;
  for (const key of path.split('.')) {
    if (current == null) return '';
    current = current[key];
  }
  return current == null ? '' : String(current);
}

/**
 * Perform fuzzy search across object arrays with weighted keys.
 *
 * Wraps `searchKeys()` with an ergonomic API that accepts row-oriented
 * objects and returns matched items directly.
 *
 * @template T
 * @param {string} query - The search query.
 * @param {T[]} items - Array of objects to search.
 * @param {object} options - Search options with keys configuration.
 * @param {Array<string | { name: string; weight?: number }>} options.keys - Keys to search.
 * @param {number} [options.maxResults] - Maximum results to return.
 * @param {number} [options.minScore] - Minimum score threshold.
 * @param {boolean} [options.isCaseSensitive] - Enable case-sensitive matching.
 * @returns {Array<{ item: T; index: number; score: number; keyScores: number[] }>}
 */
function searchObjects(query, items, options) {
  if (!options?.keys?.length) {
    throw new TypeError('options.keys must be a non-empty array');
  }
  const { keys, ...searchOpts } = options;

  const normalizedKeys = keys.map((k) =>
    typeof k === 'string' ? { name: k, weight: 1.0 } : { name: k.name, weight: k.weight ?? 1.0 },
  );

  const keyTexts = normalizedKeys.map((k) => items.map((item) => getNestedValue(item, k.name)));
  const weights = normalizedKeys.map((k) => k.weight);

  const nativeOpts = Object.keys(searchOpts).length > 0 ? searchOpts : undefined;

  const results = searchKeys(query, keyTexts, weights, nativeOpts);

  return results.map((r) => ({
    item: items[r.index],
    index: r.index,
    score: r.score,
    keyScores: r.keyScores,
  }));
}

/**
 * A persistent fuzzy search index for object collections with weighted keys.
 *
 * Pre-computes key texts and stores them on the Rust side for fast repeated
 * searches. Use this when searching the same collection multiple times.
 *
 * @template T
 */
class FuzzyObjectIndex {
  /** @type {T[]} */
  #items;
  /** @type {KeyedFuzzyIndex} */
  #index;
  /** @type {Array<{ name: string; weight: number }>} */
  #keys;

  /**
   * @param {T[]} items - Array of objects to index.
   * @param {object} options - Index configuration.
   * @param {Array<string | { name: string; weight?: number }>} options.keys - Keys to search.
   */
  constructor(items, options) {
    if (!options?.keys?.length) {
      throw new TypeError('options.keys must be a non-empty array');
    }
    this.#keys = options.keys.map((k) =>
      typeof k === 'string' ? { name: k, weight: 1.0 } : { name: k.name, weight: k.weight ?? 1.0 },
    );
    this.#items = [...items];

    const keyTexts = this.#keys.map((k) => items.map((item) => getNestedValue(item, k.name)));
    const weights = this.#keys.map((k) => k.weight);
    this.#index = new KeyedFuzzyIndex(keyTexts, weights);
  }

  /** Return the number of items in the index. */
  get size() {
    return this.#index.size;
  }

  /**
   * Search the index for objects matching the query.
   * @param {string} query
   * @param {object} [options]
   * @param {number} [options.maxResults]
   * @param {number} [options.minScore]
   * @param {boolean} [options.isCaseSensitive]
   * @returns {Array<{ item: T; index: number; score: number; keyScores: number[] }>}
   */
  search(query, options) {
    const results = this.#index.search(query, options);
    return results.map((r) => ({
      item: this.#items[r.index],
      index: r.index,
      score: r.score,
      keyScores: r.keyScores,
    }));
  }

  /**
   * Find the closest matching object.
   * @param {string} query
   * @param {number} [minScore]
   * @returns {T | null}
   */
  closest(query, minScore) {
    const results = this.#index.search(query, { maxResults: 1, minScore });
    return results.length > 0 ? this.#items[results[0].index] : null;
  }

  /**
   * Add a single item to the index.
   * @param {T} item
   */
  add(item) {
    this.#items.push(item);
    this.#index.add(this.#keys.map((k) => getNestedValue(item, k.name)));
  }

  /**
   * Add multiple items to the index at once.
   * @param {T[]} items
   */
  addMany(items) {
    for (const item of items) {
      this.#items.push(item);
    }
    this.#index.addMany(items.map((item) => this.#keys.map((k) => getNestedValue(item, k.name))));
  }

  /**
   * Remove the item at the given index.
   * Uses swap-remove semantics for O(1) performance.
   * @param {number} index
   * @returns {boolean}
   */
  remove(index) {
    if (index < 0 || index >= this.#items.length) return false;
    // Swap-remove to match Rust-side behavior
    const lastIdx = this.#items.length - 1;
    if (index !== lastIdx) {
      this.#items[index] = this.#items[lastIdx];
    }
    this.#items.pop();
    return this.#index.remove(index);
  }

  /** Free all internal data. */
  destroy() {
    this.#items = [];
    this.#index.destroy();
  }
}

module.exports = { searchObjects, FuzzyObjectIndex };
