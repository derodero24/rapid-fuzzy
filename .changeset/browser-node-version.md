---
"rapid-fuzzy": patch
---

Loading the WebAssembly build in Node.js (`node --conditions=browser`) needs Node.js 22.3 or later, which provides the `process.getBuiltinModule()` it uses to read its `.wasm` file. On Node.js 22.0-22.2 the import failed with an unexplained `TypeError: fetch failed`; it now fails with an error that names the Node.js version it needs (the original error is its `cause`). The native addon, which Node.js loads without the `browser` condition, still supports every Node.js 22 release.
