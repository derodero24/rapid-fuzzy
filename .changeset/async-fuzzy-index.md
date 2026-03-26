---
"rapid-fuzzy": minor
---

Add `FuzzyIndex.fromAsync(items)` static factory that constructs the index on the libuv thread pool, returning `Promise<FuzzyIndex>`. For large datasets this keeps the JavaScript event loop unblocked during index construction — useful in Next.js API routes, Nuxt server handlers, and other environments where blocking is a concern.
