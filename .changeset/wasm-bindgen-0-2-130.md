---
"rapid-fuzzy": patch
---

The browser build uses wasm-bindgen 0.2.130, whose glue passes JavaScript values to the WebAssembly module through a reference table instead of a heap array. The Rust dependencies of the native and WebAssembly builds are refreshed as well.
