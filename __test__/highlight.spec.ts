import { describe, expect, it } from 'vitest';

import { highlight, highlightRanges } from '../highlight.js';
import { search } from '../index.js';

describe('highlight', () => {
  it('wraps matched characters with string markers', () => {
    const result = highlight('fuzzy', [0, 2, 3], '<b>', '</b>');
    expect(result).toBe('<b>f</b>u<b>zz</b>y');
  });

  it('returns the original string when positions is empty', () => {
    expect(highlight('hello', [], '<b>', '</b>')).toBe('hello');
  });

  it('handles all characters matched', () => {
    const result = highlight('abc', [0, 1, 2], '<b>', '</b>');
    expect(result).toBe('<b>abc</b>');
  });

  it('handles non-contiguous positions', () => {
    const result = highlight('abcde', [0, 2, 4], '[', ']');
    expect(result).toBe('[a]b[c]d[e]');
  });

  it('works with callback mode', () => {
    const result = highlight('fuzzy', [0, 2, 3], (m) => `<mark>${m}</mark>`);
    expect(result).toBe('<mark>f</mark>u<mark>zz</mark>y');
  });

  it('handles single character match', () => {
    const result = highlight('hello', [0], '<b>', '</b>');
    expect(result).toBe('<b>h</b>ello');
  });

  it('handles match at end of string', () => {
    const result = highlight('hello', [4], '<b>', '</b>');
    expect(result).toBe('hell<b>o</b>');
  });

  it('integrates with search results', () => {
    const results = search('fzy', ['fuzzy', 'fizzy', 'busy'], {
      includePositions: true,
    });
    expect(results.length).toBeGreaterThan(0);

    const highlighted = highlight(
      results[0]?.item ?? '',
      results[0]?.positions ?? [],
      '<b>',
      '</b>',
    );
    expect(highlighted).toContain('<b>');
    expect(highlighted).toContain('</b>');
    expect(highlighted.replace(/<\/?b>/g, '')).toBe(results[0]?.item);
  });

  it('handles HTML-safe callback for React-like usage', () => {
    const result = highlight('test', [0, 2], (m) => `<span class="match">${m}</span>`);
    expect(result).toBe('<span class="match">t</span>e<span class="match">s</span>t');
  });

  it('handles Unicode characters', () => {
    const result = highlight('東京タワー', [0, 2], '<b>', '</b>');
    expect(result).toBe('<b>東</b>京<b>タ</b>ワー');
  });
});

describe('highlightRanges', () => {
  it('returns ranges for matched and unmatched segments', () => {
    const ranges = highlightRanges('fuzzy', [0, 2, 3]);
    expect(ranges).toEqual([
      { start: 0, end: 1, matched: true },
      { start: 1, end: 2, matched: false },
      { start: 2, end: 4, matched: true },
      { start: 4, end: 5, matched: false },
    ]);
  });

  it('returns single unmatched range when no positions', () => {
    const ranges = highlightRanges('hello', []);
    expect(ranges).toEqual([{ start: 0, end: 5, matched: false }]);
  });

  it('returns single matched range when all matched', () => {
    const ranges = highlightRanges('abc', [0, 1, 2]);
    expect(ranges).toEqual([{ start: 0, end: 3, matched: true }]);
  });

  it('returns empty array for empty string', () => {
    expect(highlightRanges('', [])).toEqual([]);
  });

  it('handles alternating matches', () => {
    const ranges = highlightRanges('abcde', [0, 2, 4]);
    expect(ranges).toEqual([
      { start: 0, end: 1, matched: true },
      { start: 1, end: 2, matched: false },
      { start: 2, end: 3, matched: true },
      { start: 3, end: 4, matched: false },
      { start: 4, end: 5, matched: true },
    ]);
  });

  it('integrates with search results', () => {
    const results = search('test', ['testing', 'best test'], {
      includePositions: true,
    });
    expect(results.length).toBeGreaterThan(0);

    const ranges = highlightRanges(results[0]?.item ?? '', results[0]?.positions ?? []);

    // Ranges should cover the entire string
    expect(ranges[0]?.start).toBe(0);
    expect(ranges[ranges.length - 1]?.end).toBe(results[0]?.item.length);

    // Ranges should be contiguous
    for (let i = 1; i < ranges.length; i++) {
      expect(ranges[i]?.start).toBe(ranges[i - 1]?.end);
    }
  });
});
