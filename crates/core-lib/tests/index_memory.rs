//! Heap usage of `FuzzyIndexCore`: an index owns only its items and their
//! search data (no per-index nucleo matcher), `heap_size` reports exactly
//! what it owns, and `destroy` frees all of it.
//!
//! A counting global allocator tracks the live heap bytes allocated by the
//! current thread, so tests running in parallel do not disturb each other.
//! Every corpus here is small enough that searches stay on the calling
//! thread.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::FuzzyIndexCore;

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
        track(layout.size() as isize);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        track(layout.size() as isize);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        track(-(layout.size() as isize));
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        track(new_size as isize - layout.size() as isize);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAlloc = CountingAlloc;

/// Live heap bytes allocated by this thread.
fn live() -> isize {
    LIVE.with(Cell::get)
}

fn items() -> Vec<String> {
    let mut items: Vec<String> = (0..500).map(|i| format!("item_{i}_text")).collect();
    items.extend(["café", "Москва", "東京タワー", "e\u{301}cole", "naïve"].map(String::from));
    items.shrink_to_fit();
    items
}

/// Search once so that this thread's shared matcher exists.
fn warm_up() {
    let index = FuzzyIndexCore::new(vec!["warm up".into()]);
    index.search_impl("wu", None, None, true, CaseMatching::Smart);
}

#[test]
fn heap_size_is_exactly_what_the_index_owns() {
    warm_up();
    let before = live();
    let index = FuzzyIndexCore::new(items());
    assert_eq!(live() - before, index.heap_size() as isize);
    drop(index);
    assert_eq!(live(), before);
}

#[test]
fn heap_size_tracks_add_remove_and_destroy() {
    warm_up();
    let mut index = FuzzyIndexCore::new(Vec::new());
    assert_eq!(index.heap_size(), 0);
    let base = live();

    index.add("café au lait".into());
    index.add_many(items());
    assert_eq!(live() - base, index.heap_size() as isize);

    let size = index.heap_size();
    assert!(index.remove(0));
    assert!(index.remove(3));
    assert!(index.heap_size() < size);
    assert_eq!(live() - base, index.heap_size() as isize);

    index.destroy();
    assert_eq!(index.heap_size(), 0);
    assert_eq!(live(), base);
}

#[test]
fn searched_indexes_share_one_matcher() {
    warm_up();
    let items = vec!["apple".to_string(), "pear".into(), "banana".into()];
    let before = live();
    let mut indexes: Vec<FuzzyIndexCore> = (0..100)
        .map(|_| FuzzyIndexCore::new(items.clone()))
        .collect();
    for index in &indexes {
        assert_eq!(
            index.search_impl("ap", None, None, true, CaseMatching::Smart)[0].item,
            "apple"
        );
    }
    let per_index = (live() - before) / 100;
    // Each index used to own a nucleo matcher (~130 KB).
    assert!(per_index < 1024, "{per_index} bytes per searched index");

    // `destroy` frees everything, including the incremental-search cache.
    for index in &mut indexes {
        index.search_impl("a", None, None, false, CaseMatching::Smart);
        index.destroy();
        assert_eq!(index.heap_size(), 0);
    }
    let vec_bytes = (indexes.capacity() * size_of::<FuzzyIndexCore>()) as isize;
    assert_eq!(live() - before, vec_bytes);
}
