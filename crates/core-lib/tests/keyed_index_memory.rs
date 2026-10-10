//! Heap usage of `KeyedFuzzyIndexCore`: like `FuzzyIndexCore`, an index owns
//! only its key texts and their search data (no per-index nucleo matcher),
//! `heap_size` reports exactly what it owns, and `destroy` frees the item
//! data.
//!
//! A counting global allocator tracks the live heap bytes allocated by the
//! current thread, so tests running in parallel do not disturb each other.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::{
    FuzzyIndexCore, KeyScoreMode, KeyedFuzzyIndexCore, search_keys_impl,
};

struct CountingAlloc;

thread_local! {
    static LIVE: Cell<isize> = const { Cell::new(0) };
}

fn track(delta: isize) {
    let _ = LIVE.try_with(|live| live.set(live.get() + delta));
}

// SAFETY: forwards every call to the system allocator unchanged.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        track(layout.size().cast_signed());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        track(layout.size().cast_signed());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        track(-(layout.size().cast_signed()));
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        track(new_size.cast_signed() - layout.size().cast_signed());
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAlloc = CountingAlloc;

/// Live heap bytes allocated by this thread.
fn live() -> isize {
    LIVE.with(Cell::get)
}

fn columns() -> Vec<Vec<String>> {
    let mut names: Vec<String> = (0..300).map(|i| format!("name_{i}")).collect();
    names.extend(["café", "Москва", "東京タワー", "e\u{301}cole"].map(String::from));
    let mut emails: Vec<String> = (0..names.len())
        .map(|i| format!("user{i}@example.com"))
        .collect();
    names.shrink_to_fit();
    emails.shrink_to_fit();
    let mut columns = vec![names, emails];
    columns.shrink_to_fit();
    columns
}

fn search(index: &KeyedFuzzyIndexCore, query: &str) -> usize {
    index
        .search(
            query,
            None,
            None,
            CaseMatching::Smart,
            false,
            KeyScoreMode::Weighted,
        )
        .len()
}

/// Search once so that this thread's shared matcher exists.
fn warm_up() {
    let index = KeyedFuzzyIndexCore::new(vec![vec!["warm up".into()]], vec![1.0]).unwrap();
    search(&index, "wu");
}

#[test]
fn heap_size_is_exactly_what_the_index_owns() {
    warm_up();
    let before = live();
    let index = KeyedFuzzyIndexCore::new(columns(), vec![2.0, 1.0]).unwrap();
    assert_eq!(live() - before, index.heap_size().cast_signed());
    assert!(search(&index, "name") > 0);
    assert_eq!(live() - before, index.heap_size().cast_signed());
    drop(index);
    assert_eq!(live(), before);
}

#[test]
fn heap_size_tracks_add_remove_and_destroy() {
    warm_up();
    let before = live();
    let mut index = KeyedFuzzyIndexCore::new(vec![Vec::new(), Vec::new()], vec![1.0, 1.0]).unwrap();
    assert_eq!(live() - before, index.heap_size().cast_signed());

    index.add(vec!["café au lait".into(), "x".into()]).unwrap();
    let rows: Vec<Vec<String>> = (0..200)
        .map(|i| vec![format!("item {i}"), format!("ítem {i}")])
        .collect();
    index.add_many(rows).unwrap();
    assert_eq!(live() - before, index.heap_size().cast_signed());

    let size = index.heap_size();
    assert!(index.remove(0));
    assert!(index.remove(3));
    assert!(index.heap_size() < size);
    assert_eq!(live() - before, index.heap_size().cast_signed());

    index.destroy();
    assert_eq!(live() - before, index.heap_size().cast_signed());
    // Only the (empty) key columns and the weights remain.
    assert!(index.heap_size() < 256, "{} bytes", index.heap_size());
}

#[test]
fn searched_indexes_share_one_matcher() {
    warm_up();
    let key_texts = vec![
        vec!["apple".to_string(), "pear".into(), "banana".into()],
        vec!["red".to_string(), "green".into(), "yellow".into()],
    ];
    let before = live();
    let mut indexes: Vec<KeyedFuzzyIndexCore> = (0..100)
        .map(|_| KeyedFuzzyIndexCore::new(key_texts.clone(), vec![1.0, 1.0]).unwrap())
        .collect();
    for index in &indexes {
        assert_eq!(
            index.search(
                "ap",
                None,
                None,
                CaseMatching::Smart,
                false,
                KeyScoreMode::Weighted
            )[0]
            .index,
            0
        );
    }
    let per_index = (live() - before) / 100;
    // Each index used to own a nucleo matcher (~130 KB).
    assert!(per_index < 2048, "{per_index} bytes per searched index");

    for index in &mut indexes {
        index.destroy();
    }
    let owned: isize = indexes.iter().map(|i| i.heap_size().cast_signed()).sum();
    let vec_bytes = (indexes.capacity() * size_of::<KeyedFuzzyIndexCore>()).cast_signed();
    assert_eq!(live() - before, vec_bytes + owned);
}

#[test]
fn standalone_search_keys_allocates_no_matcher_of_its_own() {
    warm_up();
    let key_texts = vec![vec!["apple".to_string(), "pear".into()]];
    // The first call on this thread used to create a second, keys-only matcher.
    let before = live();
    let results = search_keys_impl("ap", &key_texts, &[1.0], None).unwrap();
    assert_eq!(results[0].index, 0);
    drop(results);
    assert_eq!(live(), before);
}

#[test]
fn ascii_key_texts_are_stored_once() {
    // Like `FuzzyIndexCore`, the index matches ASCII text as its own bytes
    // instead of keeping a second copy: a one-key index costs what a
    // `FuzzyIndexCore` over the same items costs, plus its per-key columns
    // (it used to cost the text again, plus 8 bytes per item). Both get
    // clones, whose strings have no spare capacity.
    let items: Vec<String> = (0..1000)
        .map(|i| format!("handler_repository_service_{i}"))
        .collect();
    let fuzzy = FuzzyIndexCore::new(items.clone()).heap_size();
    let keyed = KeyedFuzzyIndexCore::new(vec![items.clone()], vec![1.0])
        .unwrap()
        .heap_size();
    assert!(
        keyed <= fuzzy + 256,
        "KeyedFuzzyIndexCore: {keyed} bytes, FuzzyIndexCore: {fuzzy} bytes"
    );
}
