---
title: Unicode and non-Latin scripts
description: How rapid-fuzzy handles case, diacritics, Unicode normalization, grapheme clusters, CJK and Thai text, and lone surrogates.
---

## Case and diacritics

Search folds case with [smart case](/rapid-fuzzy/guides/matching-and-scoring/#case) and folds Latin diacritics, so `cafe` matches `café`. The distance functions compare code points exactly as given. The token-based ratios lower-case their input.

## Normalize your text

Unicode normalization is not applied. These pairs are different text to rapid-fuzzy:

- A precomposed `é` (NFC) and `e` plus a combining accent (NFD).
- Full-width `ＡＢＣ` and `ABC`.
- Half-width `ｶﾞ` and `ガ`.
- Hiragana and katakana.

Normalize items and queries the same way before searching or comparing. `NFKC` also folds full-width and half-width forms:

```ts
const normalize = (s: string) => s.normalize('NFKC');

const index = new FuzzyIndex(items.map(normalize));
index.search(normalize(query));
```

## One character per grapheme

The search matcher keeps only the first code point of each grapheme cluster, so combining marks are ignored. Thai tone and vowel marks, Devanagari vowel signs, NFD accents and the half-width `ﾞ` and `ﾟ` do not have to match: `कि` matches `का` and `कु`, and `ｶﾞ` matches `ｶ`. NFKC normalization fixes the Japanese cases, because it composes `ｶﾞ` into `ガ`, but not the Thai or Devanagari ones.

Match positions count graphemes for the same reason; [`highlight()`](/rapid-fuzzy/guides/highlighting/#positions-are-graphemes) converts them to string offsets.

## Chinese, Japanese, Thai and other scripts without spaces

The matcher gives bonuses at the start of the item and at word boundaries, and these scripts have no spaces between words. A query found verbatim at the start of an item scores 1.0. Found in the middle of one, it scores about 0.44 (1 character) to 0.73 (10 characters), and never more than about 0.77: `東京` in `ここは東京` scores 0.58.

- A `minScore` of 0.5–0.7, reasonable for English, silently drops many such matches. Use a low threshold (0.4 or less), or none, and rely on the ranking.
- A match with gaps at the beginning of an item can outrank a verbatim match in the middle. For `日本語`, `日本の言語` (0.84) ranks above `これは日本語です` (0.64).
- The ideographic space (U+3000) separates query terms, like a regular space.

## Lone surrogates

Strings are converted to UTF-8 on their way into Rust, so a lone UTF-16 surrogate becomes U+FFFD (`�`). You can get one by cutting an emoji in half with `s.slice(0, n)`.

- `search()`, `closest()` and `FuzzyIndex` then return the converted copy, which is not `===` your string. Use `result.index` to get your own.
- The distance functions treat every lone surrogate as the same character.
- The WASI fallback build reads a lone surrogate as the start of a pair and swallows the character after it (a bug in its emnapi runtime), so its results for such strings differ.

Clean such strings with [`String.prototype.toWellFormed()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toWellFormed) before passing them in.

## What is not supported

rapid-fuzzy matches characters. It does not do phonetic matching (Soundex, Metaphone), stemming or lemmatization, or word segmentation for scripts without spaces.
