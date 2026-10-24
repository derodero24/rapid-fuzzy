//! Lifecycle tests for `KeyedFuzzyIndexCore`: batch insertion atomicity and
//! the state left behind by `destroy()`.

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::serialization::{
    KEYED_INDEX_MAGIC, deserialize_keyed, serialize_keyed,
};
use rapid_fuzzy_core::search::{KeyScoreMode, KeyedFuzzyIndexCore};

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_string()).collect()
}

/// Two keys (name, email), three items.
fn make_index() -> KeyedFuzzyIndexCore {
    KeyedFuzzyIndexCore::new(
        vec![
            strings(&["John Smith", "Jane Doe", "Bob Johnson"]),
            strings(&["john@example.com", "jane@example.com", "bob@example.com"]),
        ],
        vec![2.0, 1.0],
    )
    .unwrap()
}

fn search_indices(index: &KeyedFuzzyIndexCore, query: &str) -> Vec<u32> {
    index
        .search(
            query,
            None,
            None,
            CaseMatching::Smart,
            false,
            KeyScoreMode::Weighted,
        )
        .into_iter()
        .map(|r| r.index)
        .collect()
}

#[test]
fn add_many_appends_every_row() {
    let mut index = make_index();
    index
        .add_many(vec![
            strings(&["Alice Cooper", "alice@example.com"]),
            strings(&["Carol King", "carol@example.com"]),
        ])
        .unwrap();
    assert_eq!(index.size(), 5);
    assert_eq!(search_indices(&index, "carol").first(), Some(&4));
    assert_eq!(index.key_texts()[0][3], "Alice Cooper");
    assert_eq!(index.key_texts()[1][4], "carol@example.com");
}

#[test]
fn add_many_with_no_rows_is_a_no_op() {
    let mut index = make_index();
    index.add_many(Vec::new()).unwrap();
    assert_eq!(index.size(), 3);
}

#[test]
fn add_many_rejects_a_ragged_row_without_adding_any_row() {
    let mut index = make_index();
    let err = index
        .add_many(vec![
            strings(&["Alice Cooper", "alice@example.com"]),
            strings(&["Carol King"]),
            strings(&["Dave Grohl", "dave@example.com"]),
        ])
        .unwrap_err();
    assert!(
        err.contains("item 1"),
        "error should name the bad row: {err}"
    );
    assert!(
        err.contains("Expected 2"),
        "error should name the key count: {err}"
    );

    // Nothing was added, and every column still has one entry per item.
    assert_eq!(index.size(), 3);
    assert!(index.key_texts().iter().all(|col| col.len() == 3));
    assert!(search_indices(&index, "alice").is_empty());
    assert!(search_indices(&index, "dave").is_empty());

    // The index is still fully usable.
    index
        .add(strings(&["Alice Cooper", "alice@example.com"]))
        .unwrap();
    assert_eq!(search_indices(&index, "alice"), vec![3]);
}

#[test]
fn add_many_rejects_a_row_with_too_many_values() {
    let mut index = make_index();
    let err = index.add_many(vec![strings(&["a", "b", "c"])]).unwrap_err();
    assert!(err.contains("item 0"), "{err}");
    assert_eq!(index.size(), 3);
}

#[test]
fn destroy_empties_the_index() {
    let mut index = make_index();
    index.destroy();
    assert_eq!(index.size(), 0);
    assert!(search_indices(&index, "john").is_empty());
    assert!(
        index
            .search(
                "",
                None,
                None,
                CaseMatching::Smart,
                true,
                KeyScoreMode::Weighted
            )
            .is_empty()
    );
    assert!(!index.remove(0));
}

#[test]
fn destroy_keeps_the_key_configuration_so_items_can_be_added_again() {
    let mut index = make_index();
    index.destroy();

    assert_eq!(index.weights(), &[2.0, 1.0]);
    index
        .add(strings(&["Alice Cooper", "alice@example.com"]))
        .unwrap();
    index
        .add_many(vec![strings(&["Bob Dylan", "bob@example.com"])])
        .unwrap();
    assert_eq!(index.size(), 2);

    let results = index.search(
        "alice",
        None,
        None,
        CaseMatching::Smart,
        false,
        KeyScoreMode::Weighted,
    );
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].index, 0);
    assert_eq!(results[0].key_scores.len(), 2);

    // The key count is still enforced.
    assert!(index.add(strings(&["only one"])).is_err());
}

#[test]
fn destroyed_index_round_trips_through_serialization() {
    let mut index = make_index();
    index.destroy();

    let bytes = serialize_keyed(index.key_texts(), index.weights(), KEYED_INDEX_MAGIC);
    let (key_texts, weights) = deserialize_keyed(&bytes, KEYED_INDEX_MAGIC).unwrap();
    let restored = KeyedFuzzyIndexCore::new(key_texts, weights).unwrap();
    assert_eq!(restored.size(), 0);
    assert_eq!(restored.weights(), &[2.0, 1.0]);
}
