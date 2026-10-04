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
