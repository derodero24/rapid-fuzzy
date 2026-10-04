/** A range within a string, indicating whether it was matched. */
export interface HighlightRange {
  /** Start offset in UTF-16 code units (inclusive), for `item.slice(start, end)`. */
  start: number;
  /** End offset in UTF-16 code units (exclusive). */
  end: number;
  /** Whether this range was part of the match. */
  matched: boolean;
}

/** Options for {@link highlight}. */
export interface HighlightOptions {
  /**
   * HTML-escape the text of `item` (`&`, `<`, `>`, `"` and `'`), both matched
   * and unmatched parts, before the markers are added. The markers and the
   * callback's return value are inserted unchanged. Defaults to `false`.
   */
  escapeHtml?: boolean | undefined;
}

/**
 * Highlight matched characters in a search result string.
 *
 * Use with `SearchResult.positions` from a search with `includePositions: true`.
 * Positions count characters the way search results do: an ASCII item by
 * character, any other item by grapheme cluster (an emoji with its
 * modifiers or a letter with its combining marks is one position), so a
 * marker never splits one. Grapheme clusters are found with
 * `Intl.Segmenter` (by code point where it is unavailable).
 * The result is a single string: the unmatched text, and the matched text
 * wrapped in the markers (or passed through the callback), joined together.
 *
 * `highlight()` does not escape anything by default. When building HTML from
 * untrusted items, pass `{ escapeHtml: true }`; to render with a UI framework
 * such as React, build elements from {@link highlightRanges} instead.
 *
 * @example String markers
 * ```typescript
 * const results = search('fzy', ['fuzzy'], { includePositions: true });
 * highlight(results[0].item, results[0].positions, '<b>', '</b>');
 * // → '<b>f</b>uz<b>zy</b>'
 * ```
 *
 * @example Callback (custom markup, returned as a string)
 * ```typescript
 * highlight(result.item, result.positions, (matched) => `<mark>${matched}</mark>`);
 * ```
 *
 * @example Safe HTML from untrusted text
 * ```typescript
 * highlight('<b>&', [1], '<mark>', '</mark>', { escapeHtml: true });
 * // → '&lt;<mark>b</mark>&gt;&amp;'
 * ```
 */
export declare function highlight(
  item: string,
  positions: ReadonlyArray<number>,
  open: string,
  close: string,
  options?: HighlightOptions | undefined,
): string;
export declare function highlight(
  item: string,
  positions: ReadonlyArray<number>,
  callback: (matched: string) => string,
  options?: HighlightOptions | undefined,
): string;

/**
 * Convert matched positions into an array of ranges for custom rendering.
 *
 * Each range indicates a contiguous segment of the string and whether it was
 * part of the match. `positions` are counted like in {@link highlight}; the
 * ranges are UTF-16 offsets, so `item.slice(start, end)` is the segment.
 * Useful for building custom highlight components, e.g.
 * React elements, where the framework takes care of escaping.
 *
 * @example
 * ```typescript
 * highlightRanges('fuzzy', [0, 3, 4]);
 * // → [{ start: 0, end: 1, matched: true }, { start: 1, end: 3, matched: false },
 * //    { start: 3, end: 5, matched: true }]
 * ```
 */
export declare function highlightRanges(
  item: string,
  positions: ReadonlyArray<number>,
): Array<HighlightRange>;
