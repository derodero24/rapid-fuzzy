//! The checks both bindings apply to numeric arguments read from JavaScript
//! as doubles, which napi-rs and wasm-bindgen would otherwise wrap modulo
//! 2^32 (or compare against silently).

use rapid_fuzzy_core::distance::{self as d, DistanceError, check_max_distance};
use rapid_fuzzy_core::search::{FuzzyIndexCore, KeyedFuzzyIndexCore, check_remove_index};

#[test]
fn remove_index_accepts_integers_in_range() {
    assert_eq!(check_remove_index(0.0), Ok(Some(0)));
    assert_eq!(check_remove_index(-0.0), Ok(Some(0)));
    assert_eq!(check_remove_index(2.0), Ok(Some(2)));
    assert_eq!(check_remove_index(f64::from(u32::MAX)), Ok(Some(u32::MAX)));
}

#[test]
fn remove_index_is_out_of_range_when_negative_or_beyond_u32() {
    // Not wrapped modulo 2^32: 2^32 used to remove item 0, -2^32 + 1 item 1.
    for value in [
        -1.0,
        -4_294_967_295.0,
        4_294_967_296.0,
        4_294_967_297.0,
        1e300,
    ] {
        assert_eq!(check_remove_index(value), Ok(None), "{value}");
    }
}

#[test]
fn remove_index_rejects_non_integers_like_fuzzy_object_index() {
    // The messages of `FuzzyObjectIndex.remove()`'s RangeError.
    for (value, shown) in [
        (f64::NAN, "NaN"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
        (1.5, "1.5"),
        (0.9, "0.9"),
        (-0.5, "-0.5"),
    ] {
        assert_eq!(
            check_remove_index(value),
            Err(format!("index must be an integer, got {shown}")),
        );
    }
}

#[test]
fn checked_out_of_range_indexes_remove_nothing() {
    let items = || vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let mut index = FuzzyIndexCore::new(items());
    let mut keyed = KeyedFuzzyIndexCore::new(vec![items()], vec![1.0]).unwrap();
    for value in [-1.0, 3.0, 4_294_967_296.0] {
        let checked = check_remove_index(value).unwrap();
        let removed = checked.is_some_and(|i| index.remove(i));
        let removed_keyed = checked.is_some_and(|i| keyed.remove(i));
        assert!(!removed && !removed_keyed, "{value}");
    }
    assert_eq!((index.size(), keyed.size()), (3, 3));
}

#[test]
fn max_distance_accepts_integers_and_infinity() {
    assert_eq!(check_max_distance(None), Ok(None));
    assert_eq!(check_max_distance(Some(f64::INFINITY)), Ok(None));
    assert_eq!(check_max_distance(Some(0.0)), Ok(Some(0)));
    assert_eq!(check_max_distance(Some(-0.0)), Ok(Some(0)));
    assert_eq!(check_max_distance(Some(2.0)), Ok(Some(2)));
    assert_eq!(
        check_max_distance(Some(f64::from(u32::MAX) - 1.0)),
        Ok(Some(u32::MAX - 1))
    );
    // No distance reaches u32::MAX, so larger thresholds mean no limit
    // instead of wrapping modulo 2^32 (2^32 used to mean 0).
    for value in [f64::from(u32::MAX), 4_294_967_296.0, 4_294_967_298.0, 1e300] {
        assert_eq!(check_max_distance(Some(value)), Ok(None), "{value}");
    }
}

#[test]
fn max_distance_rejects_nan_negative_and_fractional_values() {
    for (value, shown) in [
        (f64::NAN, "NaN"),
        (f64::NEG_INFINITY, "-Infinity"),
        (-1.0, "-1"),
        (-4_294_967_295.0, "-4294967295"),
        (2.9, "2.9"),
        (0.5, "0.5"),
    ] {
        let err = check_max_distance(Some(value)).unwrap_err();
        assert!(
            matches!(err, DistanceError::InvalidMaxDistance(v) if v.to_bits() == value.to_bits()),
            "{err:?}"
        );
        assert_eq!(
            err.to_string(),
            format!("maxDistance must be a non-negative integer or Infinity, got {shown}")
        );
    }
}

#[test]
fn unlimited_max_distances_give_the_unfiltered_distances() {
    let candidates: Vec<String> = [
        "kitten",
        "sitting",
        "kitchen",
        "",
        "a much longer candidate",
    ]
    .map(String::from)
    .to_vec();
    for value in [f64::INFINITY, 4_294_967_296.0, f64::from(u32::MAX)] {
        let max = check_max_distance(Some(value)).unwrap();
        assert_eq!(
            d::levenshtein_many("kitten", &candidates, max),
            d::levenshtein_many("kitten", &candidates, None)
        );
        assert_eq!(
            d::damerau_levenshtein_many("kitten", &candidates, max),
            d::damerau_levenshtein_many("kitten", &candidates, None)
        );
        assert_eq!(
            d::indel_many("kitten", &candidates, max),
            d::indel_many("kitten", &candidates, None)
        );
        assert_eq!(
            d::hamming_many("kitten", &candidates, max),
            d::hamming_many("kitten", &candidates, None)
        );
    }
}
