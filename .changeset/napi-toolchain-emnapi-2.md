---
'rapid-fuzzy': patch
---

Update the napi-rs toolchain (`napi` 3.12, `@napi-rs/cli` 3.9, emnapi 2.0) and align the `rapid-fuzzy-wasm32-wasi` fallback package with the current napi-rs layout:

- It now declares exact `@emnapi/core` / `@emnapi/runtime` dependencies matching the runtime the WASM binary was built against, instead of relying on peer resolution.
- It ships its own type definitions (`rapid-fuzzy.wasi.d.cts`).
- It no longer carries a `cpu: ["wasm32"]` restriction, so package managers can install it as the fallback on platforms without a prebuilt native binary.
- Its `engines.node` range follows the WASI API requirements (`>=22.13.0 <23.0.0-0 || >=23.5.0`).

The platform package manifests also pick up the current package description, keywords, and bug tracker URL.
