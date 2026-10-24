---
title: Highlighting matches
description: Turn match positions into highlighted HTML or ranges for React, Vue and custom DOM rendering with highlight() and highlightRanges().
---

Ask for positions with `includePositions: true`, then pass them to `highlight()` or `highlightRanges()`:

```ts
import { search, highlight, highlightRanges } from 'rapid-fuzzy';

const [match] = search('fzy', ['fuzzy'], { includePositions: true });
const { item, positions } = match!;

// String markers
highlight(item, positions, '<b>', '</b>');
// → '<b>f</b>uz<b>zy</b>'

// A callback for custom markup, joined into one string
highlight(item, positions, (matched) => `<mark>${matched}</mark>`);

// Ranges for custom rendering (React, JSX, DOM)
highlightRanges(item, positions);
// → [{ start: 0, end: 1, matched: true }, { start: 1, end: 3, matched: false },
//    { start: 3, end: 5, matched: true }]
```

## Escaping HTML

`highlight()` does not escape HTML by default. When the item text is untrusted and you insert the result as HTML, pass `{ escapeHtml: true }` as the last argument. It escapes the matched and the unmatched text before adding your markers:

```ts
highlight('<b>&', [1], '<mark>', '</mark>', { escapeHtml: true });
// → '&lt;<mark>b</mark>&gt;&amp;'
```

## Rendering ranges in React

`highlightRanges()` returns UTF-16 offsets, so each range is `item.slice(start, end)`. Rendering them as elements avoids building HTML strings at all:

```tsx
import { highlightRanges } from 'rapid-fuzzy/highlight';

function Highlighted({ text, positions }: { text: string; positions: number[] }) {
  return (
    <>
      {highlightRanges(text, positions).map(({ start, end, matched }) =>
        matched ? <mark key={start}>{text.slice(start, end)}</mark> : text.slice(start, end),
      )}
    </>
  );
}
```

## Positions are graphemes

Positions count an ASCII item by character, and any other item by grapheme cluster: an emoji with its modifiers, or a letter with its combining marks, is one position. They are therefore not string offsets for non-ASCII items. `highlight()` and `highlightRanges()` convert them for you.

## A lighter import

`highlight` and `highlightRanges` are also exported from `rapid-fuzzy/highlight`, which loads neither the native addon nor the WebAssembly module. Use it in components that only render results, so they do not pull in the search engine.
