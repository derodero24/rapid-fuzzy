---
title: Browsers and bundlers
description: Load rapid-fuzzy's WebAssembly build with Vite, webpack, Next.js, esbuild or Bun, type-check against browser declarations, or import it from a CDN.
---

In the browser, `rapid-fuzzy` resolves to a WebAssembly build made with wasm-bindgen. It is selected through the package's `browser` export condition, so you import it exactly as in Node.js:

```ts
import { search, FuzzyIndex } from 'rapid-fuzzy';
```

The module instantiates the WebAssembly binary with top-level `await` while it is imported. The API is synchronous and ready once the `import` resolves; there is no `init()` to call. No `SharedArrayBuffer` and no COOP/COEP headers are needed.

The download is a ~500 KB `.wasm` file (~185 KB gzipped) and ~80 KB of JavaScript glue (~10 KB gzipped). To keep it off your initial page load, import it dynamically when the search UI opens:

```ts
const { FuzzyIndex } = await import('rapid-fuzzy');
```

## Differences from Node.js

The browser build exports the same functions as the Node.js entry, and `rapid-fuzzy/highlight` and `rapid-fuzzy/objects` work as well. The differences:

- `FuzzyObjectIndex` has no `serialize()` and `deserialize()`, which exchange Node.js Buffers. `FuzzyIndex` and `KeyedFuzzyIndex` serialize to a `Uint8Array`.
- `FuzzyIndex.fromAsync()` builds the index synchronously.
- It is single-threaded.
- It is ES-module-only, so `require('rapid-fuzzy')` keeps loading the Node.js entry even where the `browser` condition is set (for example in Jest with `jest-environment-jsdom`).

## Bundlers

| Bundler | Setup |
| --- | --- |
| Vite 8 (build and dev server), webpack 5, Next.js 16 (Turbopack and webpack) | None. The `.wasm` is emitted as an asset from its `new URL(..., import.meta.url)` reference. |
| esbuild, `bun build` | Copy `node_modules/rapid-fuzzy/rapid-fuzzy-wasm-bindgen_bg.wasm` next to the output bundle. These bundlers keep the `new URL('./rapid-fuzzy-wasm-bindgen_bg.wasm', import.meta.url)` reference but do not emit the file. |

The build target must support top-level `await` (ES2022). The defaults of the bundlers above do.

## TypeScript

TypeScript does not apply the `browser` condition on its own, so a browser project type-checks against the Node.js declarations, which for example list `FuzzyObjectIndex#serialize()`. To check against the browser build's declarations, add the condition to `tsconfig.json` (TypeScript 5.0+, with `"moduleResolution": "bundler"`, `"node16"` or `"nodenext"`):

```json
{
  "compilerOptions": {
    "moduleResolution": "bundler",
    "customConditions": ["browser"]
  }
}
```

These declarations compile on their own with `skipLibCheck` off. They need neither `@types/node` nor a `lib` newer than `ES2022` + `DOM`. For Cloudflare Workers, use `"customConditions": ["workerd"]`.

## CDN, without a bundler

The browser build is plain ES modules, so it runs straight from a CDN that serves the package files unmodified, such as [jsDelivr](https://www.jsdelivr.com/) or [unpkg](https://unpkg.com/):

```html
<script type="module">
  import { search, FuzzyIndex } from 'https://cdn.jsdelivr.net/npm/rapid-fuzzy@2/browser.mjs';

  console.log(search('typscript', ['TypeScript', 'JavaScript', 'Python'])[0].item); // 'TypeScript'

  const index = new FuzzyIndex(['TypeScript', 'JavaScript', 'Python']);
  console.log(index.search('typscript')[0].item); // 'TypeScript'
</script>
```

Pin an exact version in production, such as `rapid-fuzzy@2.2.0`. `highlight.browser.mjs` provides `highlight` and `highlightRanges` alone, without loading the WebAssembly module. See [`examples/cdn-usage`](https://github.com/derodero24/rapid-fuzzy/tree/develop/examples/cdn-usage) for a complete page.

## Serving the `.wasm` file

The browser build fetches `rapid-fuzzy-wasm-bindgen_bg.wasm` from the URL its `new URL(..., import.meta.url)` reference resolves to.

- Serve it with the `application/wasm` MIME type. It loads without it, but more slowly and with a console warning.
- If it is on another origin, allow the request with CORS headers.
- If the request returns 404, your bundler did not emit the file: copy it next to your bundle.
