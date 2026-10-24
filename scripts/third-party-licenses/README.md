# Fallback license texts

`scripts/third-party-notices.js` copies each crate's license files from its
source package in the Cargo registry into `THIRD_PARTY_NOTICES`. A few crates
publish no license file in their package; their texts are kept here, one
directory per crate name, copied verbatim from the crate's repository:

| Crate        | Source                                                          |
| ------------ | --------------------------------------------------------------- |
| `napi`       | https://github.com/napi-rs/napi-rs/blob/main/LICENSE            |
| `napi-sys`   | https://github.com/napi-rs/napi-rs/blob/main/LICENSE            |
| `gloo-utils` | https://github.com/rustwasm/gloo (LICENSE-MIT, LICENSE-APACHE) |

If the generator reports a crate without license files, add its texts here.

## Toolchain components

`toolchain/` holds the notices of code the Rust toolchain links into the
binaries, copied verbatim:

| Directory              | Files and source                                                                                                                                                                                     |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `toolchain/rust-std`   | `COPYRIGHT`, `LICENSE-MIT`, `LICENSE-APACHE` and `LICENSES/Unicode-3.0.txt` (as `LICENSE-Unicode-3.0`) of https://github.com/rust-lang/rust                                                           |
| `toolchain/wasi-libc`  | https://github.com/WebAssembly/wasi-libc: `LICENSE`, `LICENSE-MIT`, `libc-bottom-half/cloudlibc/LICENSE`, `libc-top-half/musl/COPYRIGHT`, `fts/musl-fts/COPYING`, the notice at the top of `dlmalloc/src/malloc.c`; emmalloc's license is Emscripten's `LICENSE` (https://github.com/emscripten-core/emscripten) |

wasi-libc is linked into the WASI build by the Rust `wasm32-wasip1-threads`
target. Update these files when a license text changes upstream.
