pub mod distance;
pub mod search;

/// Format a number like JavaScript's `String(value)`, for the argument errors
/// of both bindings, so that they show a number the way a JavaScript caller
/// (and `FuzzyObjectIndex`, which formats its own errors) writes it: `NaN`,
/// `Infinity`, `-1`, `0.5`, `0` for `-0`, and exponential notation below
/// 1e-6 and from 1e21 on (`1e-7`, `-1e+21`).
///
/// Both languages print the shortest digits that read back as the same
/// double, so only the notation differs: Rust's `Display` never uses
/// exponents, and its `{:e}` omits the `+` of a positive exponent.
pub(crate) fn js_number(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_string()
    } else if value.is_infinite() {
        if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
    } else if value == 0.0 {
        "0".to_string()
    } else if (1e-6..1e21).contains(&value.abs()) {
        value.to_string()
    } else {
        let exponential = format!("{value:e}");
        match exponential.split_once('e') {
            Some((digits, exponent)) if !exponent.starts_with('-') => {
                format!("{digits}e+{exponent}")
            }
            _ => exponential,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::js_number;

    #[test]
    fn js_number_formats_like_javascript_string() {
        // Each pair is a value and what `String(value)` gives in JavaScript.
        for (value, expected) in [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
            (0.0, "0"),
            (-0.0, "0"),
            (-1.0, "-1"),
            (0.5, "0.5"),
            (-2.9, "-2.9"),
            (123.456, "123.456"),
            (0.1 + 0.2, "0.30000000000000004"),
            (4_294_967_296.0, "4294967296"),
            // Plain notation down to 1e-6 and below 1e21 ...
            (0.000_001, "0.000001"),
            (-0.000_001_5, "-0.0000015"),
            (999_999_999_999_999_900_000.0, "999999999999999900000"),
            // ... and exponential notation outside that range.
            (1e-7, "1e-7"),
            (-1.5e-7, "-1.5e-7"),
            (9.99e-7, "9.99e-7"),
            (5e-324, "5e-324"),
            (f64::MIN_POSITIVE, "2.2250738585072014e-308"),
            (1e21, "1e+21"),
            (-1e21, "-1e+21"),
            (1.234_5e25, "1.2345e+25"),
            (f64::MAX, "1.7976931348623157e+308"),
        ] {
            assert_eq!(js_number(value), expected, "{value:e}");
        }
    }
}
