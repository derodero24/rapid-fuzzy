---
"rapid-fuzzy": patch
---

Packaging fixes:

- The Windows on ARM64 addon (`rapid-fuzzy-win32-arm64-msvc`) now links the MSVC C runtime statically, as the x64 addon already did. It no longer needs `VCRUNTIME140.dll` (the Visual C++ Redistributable) and loads on a clean Windows on ARM machine.
- Every package now ships license information. The nine platform packages (`rapid-fuzzy-<platform>`, `rapid-fuzzy-wasm32-wasi`) now include the MIT `LICENSE` (previously missing), and all ten packages include a `THIRD_PARTY_NOTICES` file with the licenses of the Rust crates and toolchain components compiled into the binaries. This includes `nucleo-matcher` (MPL-2.0), which is used unmodified and whose source is available at https://crates.io/crates/nucleo-matcher.
