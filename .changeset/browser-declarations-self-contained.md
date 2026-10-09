---
"rapid-fuzzy": patch
---

The browser and Cloudflare Workers declarations compile in a strict project without the Node.js types and with `skipLibCheck` off (TypeScript 5.0+, `lib: ["ES2022", "DOM"]`, `"customConditions": ["browser"]` or `["workerd"]`). They used to fail with `Cannot find name 'Buffer'` (the shared object-search types import the Node.js declarations, whose `serialize()` returned `Buffer`) and `Property 'dispose' does not exist on type 'SymbolConstructor'` (the WebAssembly classes declare `[Symbol.dispose]()`, which only `lib: ["ESNext"]` or `@types/node` declared). `serialize()` of `FuzzyIndex`, `KeyedFuzzyIndex` and `FuzzyObjectIndex` is now typed as the exported `NodeBuffer`, which is exactly Node.js's `Buffer` when `@types/node` is loaded and `Uint8Array` otherwise, and the WebAssembly declarations declare `SymbolConstructor.dispose` the way `@types/node` does. Node.js projects see the same types as before.
