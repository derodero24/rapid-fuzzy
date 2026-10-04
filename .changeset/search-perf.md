---
"rapid-fuzzy": patch
---

Faster large searches and much lower memory use for `FuzzyIndex`, `KeyedFuzzyIndex` and `FuzzyObjectIndex`. Search results are unchanged.

- Large searches use several CPU cores in the native (Node.js, Bun, Deno) builds. A search is split across threads only when it is estimated to take more than about a quarter of a millisecond; smaller searches run on the calling thread as before. On 4 cores, `FuzzyIndex` searches over 10k-1M file paths are about 2-3.5x faster, including with `includePositions`; `search()` over large arrays gains 1.2-1.8x, as converting its input strings remains the main cost. Results are identical to a single-threaded search. Set `RAYON_NUM_THREADS=1` to keep searches on one thread. The WebAssembly builds are unchanged.
- Ranking large result sets is about 3x faster.
- `FuzzyIndex`, `KeyedFuzzyIndex` and `FuzzyObjectIndex` no longer allocate their own ~100-130 KB matcher: all indexes, `search()` and `searchKeys()` share one per thread. 10,000 small `FuzzyIndex` objects used ~1 GB, now ~13 MB; 3,000 small `KeyedFuzzyIndex` objects used ~290 MB, now ~6 MB.
- `destroy()` now frees all of the index's item data immediately (a `KeyedFuzzyIndex` keeps only its key configuration).
- `FuzzyIndex` and `KeyedFuzzyIndex` (and so `FuzzyObjectIndex`) report their native memory to V8, so the garbage collector reclaims indexes that are dropped without `destroy()`. The memory is freed once the event loop runs the indexes' finalizers: in an `async` loop that awaits between iterations, 300 unreachable 50k-item `FuzzyIndex` objects used to grow memory by ~1.45 GB and now by ~105 MB, and 200 unreachable 20k-item `KeyedFuzzyIndex` objects by ~920 MB and now by ~110 MB. A tight synchronous loop gives the finalizers no chance to run (memory still grows by ~1.2 GB in the first case), so call `destroy()` on indexes created in such a loop. As with allocating a large `Buffer`, building an index of more than ~64 MB can trigger a garbage collection.
- ASCII items are no longer stored twice: `FuzzyIndex` uses 30-45% less native memory and builds faster.
