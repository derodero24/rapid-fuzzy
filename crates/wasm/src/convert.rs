//! Conversions between JS values and Rust types at the wasm-bindgen boundary.
//!
//! Every conversion that can fail runs inside an exported function and reports
//! failure as an `Err`, which wasm-bindgen throws as a JS exception only after
//! the function has returned and Rust has dropped everything it allocated.
//! Failing inside `FromWasmAbi` instead (what `#[tsify(from_wasm_abi)]` and a
//! `Vec<String>` parameter do, through `throw_str`) unwinds straight past the
//! wasm frames: it leaks every argument converted so far and the shadow-stack
//! space of the call, and leaves the `RefCell` borrow of `self` taken, so a
//! method of an index then fails for good ("recursive use of an object"). So
//! arguments that need validation, including every `string[]`, are taken as
//! plain JS handles (`JsValue`) and converted with the functions below, and
//! scripts/build-wasm-bindgen.js fails the build if a `Vec<String>` (or other
//! `Vec<JsValue>`-based) parameter comes back.

use rapid_fuzzy_core::search::{check_remove_index, invalid_index_type};
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

/// A JS `Error` with the given message.
pub(crate) fn error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}

/// A JS `TypeError`, for arguments of the wrong type or shape.
pub(crate) fn type_error(message: &str) -> JsValue {
    js_sys::TypeError::new(message).into()
}

/// A JS `RangeError`, for numeric arguments outside their valid values.
pub(crate) fn range_error(message: &str) -> JsValue {
    js_sys::RangeError::new(message).into()
}

/// Read the `index` argument of `FuzzyIndex.remove()` and
/// `KeyedFuzzyIndex.remove()` exactly like `FuzzyObjectIndex.remove()` reads
/// its own (see [`check_remove_index`]): a `TypeError` for a value that is
/// not a number, a `RangeError` for a number that is not an integer, and
/// `None` (nothing to remove) for an integer out of range.
///
/// The index is taken as a JS handle: a `u32` parameter wrapped it modulo
/// 2^32 (`NaN`, `2 ** 32` and `undefined` removed item 0), and an `f64` one
/// would still convert `null`, booleans and strings to numbers.
pub(crate) fn remove_index_from_js(index: &JsValue) -> Result<Option<u32>, JsValue> {
    let Some(value) = index.as_f64() else {
        let type_of = index.js_typeof().as_string().unwrap_or_default();
        return Err(type_error(&invalid_index_type(&type_of)));
    };
    check_remove_index(value).map_err(|message| range_error(&message))
}

/// Deserialize a JS argument, throwing a `TypeError` that names `what`.
pub(crate) fn from_js<T: DeserializeOwned>(value: JsValue, what: &str) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|e| type_error(&format!("Invalid {what}: {e}")))
}

/// Read a `string[][]` argument such as the `[a, b]` pairs of the `*Batch`
/// functions or the per-key text columns of `searchKeys`.
pub(crate) fn string_matrix_from_js(
    value: JsValue,
    what: &str,
) -> Result<Vec<Vec<String>>, JsValue> {
    from_js(value, what)
}

/// Read a `string[]` argument, throwing a `TypeError` for anything but an
/// array of strings.
pub(crate) fn strings_from_js(value: &JsValue) -> Result<Vec<String>, JsValue> {
    if !js_sys::Array::is_array(value) {
        return Err(type_error("Expected an array of strings"));
    }
    value
        .unchecked_ref::<js_sys::Array>()
        .iter()
        .enumerate()
        .map(|(i, item)| {
            item.as_string()
                .ok_or_else(|| type_error(&format!("Expected a string at index {i}")))
        })
        .collect()
}

/// Serialize a value (e.g. a result array) into a JS value.
pub(crate) fn to_js<T: Serialize + ?Sized>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| error(&format!("Failed to convert the result: {e}")))
}

/// `Some(value)` as a JS value, `None` as `null` (matching the Node.js binding).
pub(crate) fn or_null<T: Into<JsValue>>(value: Option<T>) -> JsValue {
    value.map_or(JsValue::NULL, Into::into)
}

/// A JS array with `null` in place of each `None`.
pub(crate) fn nullable_array<T: Copy + Into<JsValue>>(values: &[Option<T>]) -> js_sys::Array {
    values.iter().map(|&value| or_null(value)).collect()
}
