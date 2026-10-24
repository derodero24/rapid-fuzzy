---
title: Errors and edge cases
description: What rapid-fuzzy returns or throws for empty input, invalid options, destroyed indexes and corrupt serialized data.
---

## Return values

- `hamming()` and `normalizedHamming()` return `null` when the strings have different lengths.
- `closest()` returns `null` when no item matches the query, when the best match scores below `minScore`, or when the list is empty.
- `remove(index)` on an index returns `false` for an index out of range (negative, or not less than `size`).
- An index after `destroy()` does not throw. It behaves as an empty index and accepts new items.

## What throws

Invalid arguments throw before any work is done. In Node.js most of them are an `Error` with the code `InvalidArg`. In the WebAssembly build they are an `Error`, a `TypeError` or a `RangeError`. Check for `instanceof Error` rather than a specific class.

- `maxResults`, and the `maxDistance` threshold of `levenshteinMany`, `damerauLevenshteinMany`, `indelMany`, `hammingMany` and their `*ManyU32` variants, must be a non-negative integer or `Infinity` (no limit). `NaN`, negative and fractional values throw.
- A `NaN` `minScore` or `minSimilarity` throws. Any other number is accepted.
- `remove(index)` throws a `TypeError` for an index that is not a number, and a `RangeError` for one that is not an integer (`NaN`, `±Infinity` or a fraction).
- `*Batch` functions throw when a pair is not exactly two strings.
- In the WebAssembly build, a value that is not a string where a string is expected, an array that is not an array of strings, and a threshold that is not a number (such as `'2'` or `true`) throw a `TypeError`.
- `searchObjects()` and `FuzzyObjectIndex` throw a `TypeError` when `options.keys` is missing or empty.
- Object and keyed search throw when a weight is negative, `NaN` or infinite, or when the weights sum to 0 or overflow. The lower-level `searchKeys()` and `KeyedFuzzyIndex` also throw when the key text arrays have different lengths, or when the number of weights differs from the number of keys.
- An unknown `scoreMode` or `matchMode` throws.
- `FuzzyIndex.deserialize()`, `KeyedFuzzyIndex.deserialize()` and `FuzzyObjectIndex.deserialize()` throw on corrupt data and on data from another format version.
- `addMany()` on `KeyedFuzzyIndex` and `FuzzyObjectIndex` is atomic: when one item is invalid, nothing is added, and the error names the item (`item 3`).
