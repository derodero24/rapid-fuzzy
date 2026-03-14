'use strict';

const { searchKeys } = require('./index.js');

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
  const { keys, ...searchOpts } = options;

  const normalizedKeys = keys.map((k) =>
    typeof k === 'string' ? { name: k, weight: 1.0 } : { name: k.name, weight: k.weight ?? 1.0 },
  );

  const keyTexts = normalizedKeys.map((k) => items.map((item) => getNestedValue(item, k.name)));
  const weights = normalizedKeys.map((k) => k.weight);

  const nativeOpts =
    searchOpts.maxResults != null ||
    searchOpts.minScore != null ||
    searchOpts.isCaseSensitive != null
      ? searchOpts
      : undefined;

  const results = searchKeys(query, keyTexts, weights, nativeOpts);

  return results.map((r) => ({
    item: items[r.index],
    index: r.index,
    score: r.score,
    keyScores: r.keyScores,
  }));
}

module.exports = { searchObjects };
