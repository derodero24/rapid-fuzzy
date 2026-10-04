---
"rapid-fuzzy": patch
---

Faster large searches and much lower memory use for `FuzzyIndex`. Search results are unchanged.

- Large searches use several CPU cores in the native (Node.js, Bun, Deno) builds. A search is split across threads only when it is estimated to take more than about a quarter of a millisecond; smaller searches run on the calling thread as before. On 4 cores, `FuzzyIndex` searches over 10k-1M file paths are about 2-3.5x faster, including with `includePositions`; `search()` over large arrays gains 1.2-1.8x, as converting its input strings remains the main cost. Results are identical to a single-threaded search. Set `RAYON_NUM_THREADS=1` to keep searches on one thread. The WebAssembly builds are unchanged.
- Ranking large result sets is about 3x faster.
- A `FuzzyIndex` no longer allocates its own ~130 KB matcher: all indexes and `search()` share one per thread. 10,000 small indexes used ~1 GB, now ~13 MB.
- `destroy()` now frees all of the index's native memory immediately.
- `FuzzyIndex` reports its native memory to V8, so the garbage collector reclaims indexes that are dropped without `destroy()` (300 unreachable 50k-item indexes used to grow memory by ~1.85 GB; now ~125 MB). As with allocating a large `Buffer`, building an index of more than ~64 MB can trigger a garbage collection.
- ASCII items are no longer stored twice: `FuzzyIndex` uses 30-45% less native memory and builds faster.
