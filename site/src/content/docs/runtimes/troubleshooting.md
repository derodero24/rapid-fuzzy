---
title: Troubleshooting
description: Fix "Cannot find native binding", WebAssembly loading failures and SSR errors with rapid-fuzzy.
---

## Cannot find native binding

The native binary comes in a platform package, such as `rapid-fuzzy-linux-x64-gnu`, that npm installs as an optional dependency. This error usually means that package was not installed.

1. Remove both `node_modules` and `package-lock.json`, then install again. A known npm bug can leave platform packages out of the lockfile ([npm/cli#4828](https://github.com/npm/cli/issues/4828)).
2. Do not install with `--omit=optional` (or `--no-optional`).
3. If your platform has no prebuilt binary, install the [WASI fallback](/rapid-fuzzy/runtimes/node/#other-platforms-the-wasi-fallback).

## The WebAssembly file fails to load in the browser

The browser build fetches `rapid-fuzzy-wasm-bindgen_bg.wasm` relative to its own module.

- **404**: your bundler did not emit the file. Copy it next to your bundle; esbuild and `bun build` need this (see [Bundlers](/rapid-fuzzy/runtimes/browsers/#bundlers)).
- **Wrong MIME type warning**: serve `.wasm` files as `application/wasm`.
- **CORS error**: allow the request when the file is on another origin.
- **"top-level await is not available"**: raise your bundler's build target to ES2022 or later.

## Errors in server-side rendering

Externalize `rapid-fuzzy` in the server build so the native module is not bundled; see [Server-side rendering](/rapid-fuzzy/runtimes/node/#server-side-rendering). Cloudflare Workers use the WebAssembly build automatically. The Next.js and Vercel Edge runtime is not supported.

## `--conditions=browser` fails in Node.js

The WebAssembly build needs Node.js 22.3 or later in Node.js, for `process.getBuiltinModule()`. On older releases it fails with an error that names the version it needs.

## Still stuck?

Search the [issues](https://github.com/derodero24/rapid-fuzzy/issues) or open a new one with your runtime, its version, your bundler and the full error.
