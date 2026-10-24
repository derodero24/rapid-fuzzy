---
title: Performance and untrusted input
description: How long rapid-fuzzy's synchronous functions can block on long input, and how to keep servers and UIs responsive.
---

Every function is synchronous: it blocks the calling thread (the Node.js event loop, or the browser's main thread) until it returns. Most calls on short strings take microseconds. Several functions, however, take time proportional to the product of their input lengths, so long input can block for seconds, for example text pasted into a search box or sent to your server.

Measured with the native addon on the [benchmark machine](/rapid-fuzzy/benchmarks/):

| Call | Input | Time |
| --- | --- | ---: |
| `partialRatio`, `weightedRatio` | 1,000 vs 10,000 characters | 0.4 s |
| `partialRatio`, `weightedRatio` | 2,000 vs 20,000 characters | 2.9 s |
| `partialRatio` | 5,000 vs 50,000 characters | 43 s |
| `damerauLevenshtein` | two 10,000-character strings | 0.4 s |
| `damerauLevenshtein` | two 30,000-character strings | 3.8 s |
| `levenshtein` | two 100,000-character strings | 0.5 s |
| `tokenSetRatio` | two 100,000-character strings | 0.45 s |
| `search` | 1,000 items of 10,000 characters | 0.5 s |
| `jaroWinkler`, `sorensenDice` | two 100,000-character strings | < 0.05 s |

The WebAssembly build is slower than the native addon.

## Keeping an application responsive

- **Cap input lengths** before calling rapid-fuzzy. Queries rarely need more than 100–200 characters, and `partialRatio`, `weightedRatio` and `damerauLevenshtein` should only see strings of bounded length.
- **Use an index** (`FuzzyIndex` or `FuzzyObjectIndex`) for repeated searches over the same items, and pass `maxResults`.
- **Prefer `*Many` and typed-array variants** for one string against many candidates.
- **Move large jobs off the main thread** with `worker_threads` in Node.js or a Web Worker in the browser. `FuzzyIndex.fromAsync()` builds an index on the libuv thread pool in Node.js.

## Threads

With the native addon, a search that is estimated to take more than about a quarter of a millisecond is split across a thread pool, with one thread per CPU. Set `RAYON_NUM_THREADS` to limit the pool. Smaller searches run on the calling thread. The WebAssembly build is single-threaded.

## Limits

- The WebAssembly build can use at most 4 GB of memory (65,536 pages of 64 KB).
- Only index construction has an asynchronous variant, `FuzzyIndex.fromAsync()`.
