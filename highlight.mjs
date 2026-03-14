// ESM version of highlight utilities — for browser bundlers and ESM-only environments.
// Keep in sync with highlight.js (CJS version).

/**
 * Convert matched positions into an array of ranges for custom rendering.
 *
 * @param {string} item - The original string from the search result.
 * @param {number[]} positions - Array of matched character indices.
 * @returns {Array<{start: number, end: number, matched: boolean}>} Array of ranges.
 */
export function highlightRanges(item, positions) {
  if (!item) return [];
  if (!positions || positions.length === 0) {
    return [{ start: 0, end: item.length, matched: false }];
  }

  const set = new Set(positions);
  const ranges = [];
  let i = 0;

  while (i < item.length) {
    const matched = set.has(i);
    const start = i;
    while (i < item.length && set.has(i) === matched) i++;
    ranges.push({ start, end: i, matched });
  }

  return ranges;
}

/**
 * Highlight matched characters in a search result string.
 *
 * @param {string} item - The original string from the search result.
 * @param {number[]} positions - Array of matched character indices.
 * @param {string | ((substring: string) => string)} openOrCallback - Opening tag or callback.
 * @param {string} [close] - Closing tag (required when openOrCallback is a string).
 * @returns {string} The highlighted string.
 */
export function highlight(item, positions, openOrCallback, close) {
  if (!positions || positions.length === 0) return item;

  const ranges = highlightRanges(item, positions);
  const useCallback = typeof openOrCallback === 'function';

  const parts = [];
  for (const range of ranges) {
    const segment = item.slice(range.start, range.end);
    if (range.matched) {
      parts.push(useCallback ? openOrCallback(segment) : openOrCallback + segment + (close ?? ''));
    } else {
      parts.push(segment);
    }
  }

  return parts.join('');
}
