// Local development shim — re-exports WASM loader + highlight utilities.
// In production (StackBlitz / npm), imports resolve to the `rapid-fuzzy` package directly.
export * from 'rapid-fuzzy-wasm32-wasi';

// Inline highlight utilities (same as ../highlight.js but as ESM).
// Needed because Vite can't CJS-transform files served via /@fs/.
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
