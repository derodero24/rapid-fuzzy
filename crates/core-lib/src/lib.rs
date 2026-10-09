pub mod distance;
pub mod search;

/// Format a number like JavaScript's `String(value)` does for the values the
/// argument errors of both bindings report: `NaN`, `Infinity`, `-Infinity`,
/// integers without a decimal point (`-1`) and fractions (`0.5`).
pub(crate) fn js_number(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_string()
    } else if value.is_infinite() {
        if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
    } else {
        value.to_string()
    }
}
