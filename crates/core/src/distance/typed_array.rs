//! `Uint32Array` / `Float64Array` results for the `*ManyU32` / `*ManyF64`
//! functions.
//!
//! napi-rs's own `Uint32Array` / `Float64Array` hand the Rust allocation to
//! Node.js as an *external* ArrayBuffer, and Node.js refuses to transfer
//! external buffers (`postMessage(array, [array.buffer])` throws a
//! `DataCloneError`). [`TypedArrayResult`] instead copies the values into a
//! buffer allocated by the JS engine, so the result is an ordinary typed array
//! that can be moved to a worker; the copy is a `memcpy`, negligible next to
//! computing the values.

use napi::bindgen_prelude::{ToNapiValue, TypeName};
use napi::{ValueType, sys};

/// Element types that have a JS typed array.
pub trait TypedArrayElement: Copy {
    /// `napi_typedarray_type` of the matching JS typed array.
    const ARRAY_TYPE: sys::napi_typedarray_type;
    /// Name of the matching JS typed array.
    const ARRAY_NAME: &'static str;

    /// Convert through napi-rs's typed array (used on wasm, see below).
    #[cfg(target_family = "wasm")]
    unsafe fn napi_rs_typed_array(
        env: sys::napi_env,
        values: Vec<Self>,
    ) -> napi::Result<sys::napi_value>;
}

impl TypedArrayElement for u32 {
    const ARRAY_TYPE: sys::napi_typedarray_type = sys::TypedarrayType::uint32_array;
    const ARRAY_NAME: &'static str = "Uint32Array";

    #[cfg(target_family = "wasm")]
    unsafe fn napi_rs_typed_array(
        env: sys::napi_env,
        values: Vec<Self>,
    ) -> napi::Result<sys::napi_value> {
        use napi::bindgen_prelude::Uint32Array;
        unsafe { Uint32Array::to_napi_value(env, Uint32Array::new(values)) }
    }
}

impl TypedArrayElement for f64 {
    const ARRAY_TYPE: sys::napi_typedarray_type = sys::TypedarrayType::float64_array;
    const ARRAY_NAME: &'static str = "Float64Array";

    #[cfg(target_family = "wasm")]
    unsafe fn napi_rs_typed_array(
        env: sys::napi_env,
        values: Vec<Self>,
    ) -> napi::Result<sys::napi_value> {
        use napi::bindgen_prelude::Float64Array;
        unsafe { Float64Array::to_napi_value(env, Float64Array::new(values)) }
    }
}

/// Values returned to JS as a typed array that owns an ordinary
/// (engine-allocated, transferable) ArrayBuffer.
pub struct TypedArrayResult<T: TypedArrayElement>(pub Vec<T>);

impl<T: TypedArrayElement> TypeName for TypedArrayResult<T> {
    fn type_name() -> &'static str {
        T::ARRAY_NAME
    }

    fn value_type() -> ValueType {
        ValueType::Object
    }
}

impl<T: TypedArrayElement> ToNapiValue for TypedArrayResult<T> {
    #[cfg(not(target_family = "wasm"))]
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
        let values = val.0.as_slice();
        let byte_len = std::mem::size_of_val(values);
        let mut data = std::ptr::null_mut();
        let mut buffer = std::ptr::null_mut();
        napi::check_status!(
            unsafe { sys::napi_create_arraybuffer(env, byte_len, &mut data, &mut buffer) },
            "Failed to create the ArrayBuffer of a {}",
            T::ARRAY_NAME
        )?;
        if byte_len > 0 {
            // SAFETY: `data` is the start of the `byte_len` bytes just
            // allocated for `buffer`, so it is valid for writes of `byte_len`
            // bytes and cannot overlap `values`.
            unsafe {
                std::ptr::copy_nonoverlapping(values.as_ptr().cast::<u8>(), data.cast(), byte_len)
            };
        }
        let mut array = std::ptr::null_mut();
        napi::check_status!(
            unsafe {
                sys::napi_create_typedarray(env, T::ARRAY_TYPE, values.len(), buffer, 0, &mut array)
            },
            "Failed to create a {}",
            T::ARRAY_NAME
        )?;
        Ok(array)
    }

    /// Under emnapi (wasm) `napi_create_arraybuffer` hands out a separate copy
    /// in wasm memory that is not synced back, while its external buffers are
    /// copied into ordinary ArrayBuffers anyway, so napi-rs's typed arrays
    /// already give the right result there.
    #[cfg(target_family = "wasm")]
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
        unsafe { T::napi_rs_typed_array(env, val.0) }
    }
}
