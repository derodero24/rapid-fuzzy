---
"rapid-fuzzy": patch
---

The `maxDistance` threshold of `levenshteinMany`, `damerauLevenshteinMany`, `indelMany`, `hammingMany` and their `*ManyU32` variants is validated like `maxResults`, in the Node.js, WASI and browser builds. It used to be converted to a 32-bit unsigned integer, wrapping it modulo 2^32: `Infinity`, `NaN` and `2 ** 32` became 0, so every candidate that was not identical to the reference silently came back as exceeding the threshold (`1`, or `null` for `hammingMany`). Now `Infinity`, like any value of at least 2^32 - 1, means no limit, and `NaN`, negative and fractional values throw (an `InvalidArg` error in Node.js, an `Error` in the browser build; the message shows the value as JavaScript does, such as `got -1e+21`). This includes `-1`, which used to disable the limit by wrapping to 4294967295, and fractions such as `2.9`, which were truncated: pass `Infinity` (or omit the argument) for no limit and an integer threshold otherwise.
