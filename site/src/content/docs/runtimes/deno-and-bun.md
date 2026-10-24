---
title: Deno and Bun
description: Use rapid-fuzzy in Deno and Bun with the native addon, or switch to the WebAssembly build with the browser condition.
---

## Deno

Import rapid-fuzzy with the `npm:` specifier:

```ts
import { search } from 'npm:rapid-fuzzy';

console.log(search('typscript', ['TypeScript', 'JavaScript', 'Python'])[0]?.item); // 'TypeScript'
```

By default Deno loads the Node.js entry, which uses the native addon. That gives you the full Node.js API, including `FuzzyObjectIndex` serialization, and needs `--allow-ffi`:

```sh
deno run --allow-read --allow-env --allow-ffi main.ts
```

To use the WebAssembly build instead, without the FFI permission, add the `browser` condition. It reads its `.wasm` file from the npm cache, so it needs `--allow-read`:

```sh
deno run --conditions=browser --allow-read main.ts
```

## Bun

Bun loads the native addon by default, which is the fastest option:

```sh
bun add rapid-fuzzy
bun run main.ts
```

To use the WebAssembly build, for example to test the code path your browser bundle takes, run with the `browser` condition:

```sh
bun --conditions=browser run main.ts
```

`bun build` keeps the WebAssembly file's `new URL(...)` reference but does not emit the file: copy `node_modules/rapid-fuzzy/rapid-fuzzy-wasm-bindgen_bg.wasm` next to the bundle (see [Bundlers](/rapid-fuzzy/runtimes/browsers/#bundlers)).
