//! Conversions between JS values and Rust types at the wasm-bindgen boundary.
//!
//! Every conversion that can fail runs inside an exported function and reports
//! failure as an `Err`, which wasm-bindgen throws as a JS exception only after
//! the function has returned and Rust has dropped everything it allocated.
//! Failing inside `FromWasmAbi` instead (what `#[tsify(from_wasm_abi)]` does)
//! unwinds straight past the wasm frames and leaks every argument converted so
//! far, so arguments that need validation are taken as plain JS handles.

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
