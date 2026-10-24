---
title: Node.js and SSR
description: Use rapid-fuzzy's native addon in Node.js, the WASI fallback for other platforms, and Next.js or Vite server-side rendering.
---

In Node.js 22 and later, `rapid-fuzzy` loads a native addon built with [napi-rs](https://napi.rs). Both `import` and `require` work:

```ts
import { search, FuzzyIndex } from 'rapid-fuzzy'; // ES modules
const { search, FuzzyIndex } = require('rapid-fuzzy'); // CommonJS
```

## Prebuilt binaries

npm installs the binary for your platform as an optional dependency, such as `rapid-fuzzy-linux-x64-gnu`. Binaries are published for:

| OS | Architectures |
| --- | --- |
| macOS | x64, arm64 |
| Linux (glibc 2.17+) | x64, arm64 |
| Linux (musl, e.g. Alpine) | x64, arm64 |
| Windows | x64, arm64 |

Do not install with `--omit=optional` (or `--no-optional`), which skips them. If the binary is missing, see [Troubleshooting](/rapid-fuzzy/runtimes/troubleshooting/#cannot-find-native-binding).

## Other platforms: the WASI fallback

On a platform without a prebuilt binary, install the WebAssembly (WASI) build next to the package. It needs Node.js 22.13+ or 23.5+:

```sh
npm install rapid-fuzzy rapid-fuzzy-wasm32-wasi
```

The loader falls back to it when no native binary loads. Set `NAPI_RS_FORCE_WASI=true` to prefer it over the native addon.

## Threads

Large searches use a thread pool with one thread per CPU; set `RAYON_NUM_THREADS` to limit it. Everything else runs on the calling thread and blocks the event loop while it runs, so read [Performance and untrusted input](/rapid-fuzzy/guides/performance/) before you pass user input to the slower functions.

## Server-side rendering

Bundlers must not bundle a native module into server code. Mark `rapid-fuzzy` as external.

**Next.js**

```js
// next.config.js
const nextConfig = {
  serverExternalPackages: ['rapid-fuzzy'],
};
```

**Vite SSR**

```js
// vite.config.js
export default {
  ssr: {
    external: ['rapid-fuzzy'],
  },
};
```

In client code, bundlers pick the WebAssembly build through the `browser` export condition with no configuration. This is tested with Next.js 16 client components (Turbopack and webpack) and Vite 8. The Next.js and Vercel Edge runtime is not supported: use the default Node.js runtime for server code.

## Using the WebAssembly build in Node.js

To run the same code path as your browser bundle, for example in tests, start Node.js 22.3 or later with the `browser` condition:

```sh
node --conditions=browser main.mjs
```

Earlier Node.js 22 releases lack `process.getBuiltinModule()`, which the WebAssembly build uses to read its `.wasm` file. `require('rapid-fuzzy')` always loads the native addon, because the WebAssembly build is ES-module-only.
