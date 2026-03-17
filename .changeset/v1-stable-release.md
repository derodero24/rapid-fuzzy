---
"rapid-fuzzy": major
---

First stable release. The public API is now considered stable and will follow semver strictly.

Key highlights since 0.x:
- 9 string distance algorithms with batch and many variants
- Fuzzy search powered by nucleo with query syntax (exclude, prefix, suffix, exact)
- FuzzyIndex and FuzzyObjectIndex for persistent Rust-side indexing with incremental cache
- Object search with weighted keys (searchObjects, FuzzyObjectIndex)
- Match highlighting utilities (highlight, highlightRanges)
- Subpath exports for tree-shaking (rapid-fuzzy/highlight, rapid-fuzzy/objects)
- Node.js native (napi-rs) + WASM (browser/Deno/Bun) dual distribution
- Full TypeScript support with auto-generated type definitions
