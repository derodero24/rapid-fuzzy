/** A range within a string, indicating whether it was matched. */
export interface HighlightRange {
  /** Start index (inclusive). */
  start: number;
  /** End index (exclusive). */
  end: number;
  /** Whether this range was part of the match. */
  matched: boolean;
}

/**
 * Highlight matched characters in a search result string.
 *
 * Use with `SearchResult.positions` from a search with `includePositions: true`.
 *
 * @example String markers
 * ```typescript
 * const results = search('fzy', ['fuzzy'], { includePositions: true });
 * highlight(results[0].item, results[0].positions, '<b>', '</b>');
 * // → '<b>f</b>u<b>zy</b>'
 * ```
 *
 * @example Callback (React, custom DOM, etc.)
 * ```typescript
 * highlight(result.item, result.positions, (matched) => `<mark>${matched}</mark>`);
 * ```
 */
export declare function highlight(
  item: string,
  positions: Array<number>,
  open: string,
  close: string,
): string;
export declare function highlight(
  item: string,
  positions: Array<number>,
  callback: (matched: string) => string,
): string;

/**
 * Convert matched positions into an array of ranges for custom rendering.
 *
 * Each range indicates a contiguous segment of the string and whether it was
 * part of the match. Useful for building custom highlight components.
 *
 * @example
 * ```typescript
 * const ranges = highlightRanges(result.item, result.positions);
 * // → [{ start: 0, end: 1, matched: true }, { start: 1, end: 2, matched: false }, ...]
 * ```
 */
export declare function highlightRanges(
  item: string,
  positions: Array<number>,
): Array<HighlightRange>;
