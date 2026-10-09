//! Hardening tests for the binary index serialization format.
//!
//! Every deserializer must turn arbitrary bytes into either `Ok` or a
//! descriptive `Err`: never a panic (which traps on wasm and aborts Node),
//! and never an allocation sized from an untrusted header field (which
//! aborts the whole process with "memory allocation of N bytes failed").

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::serialization::{
    FUZZY_INDEX_ACCEPTED_MAGICS, FUZZY_INDEX_MAGIC, FUZZY_INDEX_WASM_MAGIC, KEYED_INDEX_MAGIC,
    SERIALIZE_VERSION, deserialize_fuzzy_index, deserialize_items, deserialize_keyed,
    deserialize_keyed_index, serialize_fuzzy_index, serialize_items, serialize_keyed,
    serialize_keyed_index,
};
use rapid_fuzzy_core::search::{FuzzyIndexCore, KeyScoreMode, KeyedFuzzyIndexCore};

// ---------------------------------------------------------------------------
// Allocation tracking
// ---------------------------------------------------------------------------

/// Any single allocation above this size is refused outright, so a parser
/// that trusts a header count fails fast instead of reserving gigabytes of
/// (overcommitted) address space.
const HARD_ALLOCATION_LIMIT: usize = 1 << 30;

/// Global allocator that records the largest single allocation request made
/// by the current thread, so tests can bound the memory a parser asks for.
struct TrackingAllocator;

thread_local! {
    static LARGEST_ALLOCATION: Cell<usize> = const { Cell::new(0) };
}

fn record_allocation(size: usize) {
    // `try_with` fails only during thread teardown; nothing to record then.
    let _ = LARGEST_ALLOCATION.try_with(|largest| {
        if size > largest.get() {
            largest.set(size);
        }
    });
}

// SAFETY: every method forwards to the system allocator with the caller's
// arguments unchanged, or returns null (allocation failure) without
// allocating, which `GlobalAlloc` permits.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        if layout.size() > HARD_ALLOCATION_LIMIT {
            return std::ptr::null_mut();
        }
        // SAFETY: forwarded verbatim from the caller.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        if layout.size() > HARD_ALLOCATION_LIMIT {
            return std::ptr::null_mut();
        }
        // SAFETY: forwarded verbatim from the caller.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarded verbatim from the caller.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation(new_size);
        if new_size > HARD_ALLOCATION_LIMIT {
            return std::ptr::null_mut();
        }
        // SAFETY: forwarded verbatim from the caller.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

/// Run `f` and return its result together with the largest single
/// allocation (in bytes) it requested on this thread.
fn largest_allocation_during<R>(f: impl FnOnce() -> R) -> (R, usize) {
    LARGEST_ALLOCATION.with(|largest| largest.set(0));
    let result = f();
    let largest = LARGEST_ALLOCATION.with(Cell::get);
    (result, largest)
}

/// Upper bound on any single allocation a parser may make for an input of
/// `input_len` bytes: linear in the input, never in a header field.
fn parse_allocation_budget(input_len: usize) -> usize {
    64 * input_len + 4096
}

// ---------------------------------------------------------------------------
// Payload helpers
// ---------------------------------------------------------------------------

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn fuzzy_payload(items: &[&str]) -> Vec<u8> {
    serialize_items(&strings(items), FUZZY_INDEX_MAGIC)
}

fn keyed_payload(columns: &[&[&str]], weights: &[f64]) -> Vec<u8> {
    let key_texts: Vec<Vec<String>> = columns.iter().map(|col| strings(col)).collect();
    serialize_keyed(&key_texts, weights, KEYED_INDEX_MAGIC)
}

fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Header byte offsets.
const VERSION_OFFSET: usize = 4;
const COUNT_OFFSET: usize = 8;
const NUM_KEYS_OFFSET: usize = 8;
const NUM_ITEMS_OFFSET: usize = 12;
/// Offset of the first item's length prefix in a FuzzyIndex payload.
const FIRST_ITEM_LEN_OFFSET: usize = 12;

fn expect_err<T: std::fmt::Debug>(result: Result<T, String>, needle: &str) -> String {
    match result {
        Ok(value) => panic!("expected an error containing {needle:?}, got Ok({value:?})"),
        Err(message) => {
            assert!(
                message.contains(needle),
                "error {message:?} does not contain {needle:?}"
            );
            message
        }
    }
}

// ---------------------------------------------------------------------------
// FuzzyIndex payloads: round-trips
// ---------------------------------------------------------------------------

#[test]
fn items_roundtrip() {
    for items in [
        &[][..],
        &[""][..],
        &["apple", "banana", "cherry"][..],
        &["東京", "café", "🎉 party", "", "a\u{0}b"][..],
    ] {
        let bytes = fuzzy_payload(items);
        assert_eq!(deserialize_items(&bytes, FUZZY_INDEX_MAGIC).unwrap(), items);
    }
}

// ---------------------------------------------------------------------------
// FuzzyIndex payloads: header fields set to boundary values
// ---------------------------------------------------------------------------

#[test]
fn items_version_field_boundaries() {
    let base = fuzzy_payload(&["alpha", "beta"]);
    for version in [0, 2, u32::MAX] {
        let mut bytes = base.clone();
        set_u32(&mut bytes, VERSION_OFFSET, version);
        let message = expect_err(
            deserialize_items(&bytes, FUZZY_INDEX_MAGIC),
            "Unsupported format version",
        );
        assert!(message.contains(&version.to_string()), "{message}");
    }
    let mut bytes = base.clone();
    set_u32(&mut bytes, VERSION_OFFSET, SERIALIZE_VERSION);
    assert!(deserialize_items(&bytes, FUZZY_INDEX_MAGIC).is_ok());
}

#[test]
fn items_count_field_boundaries() {
    let base = fuzzy_payload(&["alpha", "beta"]);
    let cases = [
        (0, "trailing"),
        (1, "trailing"),
        (3, "truncated"),
        (0x4000_0000, "item count"),
        (0x8000_0000, "item count"),
        (u32::MAX, "item count"),
    ];
    for (count, needle) in cases {
        let mut bytes = base.clone();
        set_u32(&mut bytes, COUNT_OFFSET, count);
        let (result, largest) =
            largest_allocation_during(|| deserialize_items(&bytes, FUZZY_INDEX_MAGIC));
        expect_err(result, needle);
        assert!(largest <= parse_allocation_budget(bytes.len()));
    }
}

#[test]
fn items_length_field_boundaries() {
    let base = fuzzy_payload(&["alpha", "beta"]);
    // 0xFFFF_FFF8 + the 16-byte offset of the first item's text wraps to 8
    // on 32-bit targets; the bounds check must not be fooled by that.
    for len in [0x8000_0000, 0xFFFF_FFF0, 0xFFFF_FFF8, 0xFFFF_FFFC, u32::MAX] {
        let mut bytes = base.clone();
        set_u32(&mut bytes, FIRST_ITEM_LEN_OFFSET, len);
        expect_err(deserialize_items(&bytes, FUZZY_INDEX_MAGIC), "truncated");
    }
    // Shorter lengths desynchronize the stream; any error is fine, a panic is not.
    for len in [0, 1, 2, 4, 6, 7] {
        let mut bytes = base.clone();
        set_u32(&mut bytes, FIRST_ITEM_LEN_OFFSET, len);
        assert!(deserialize_items(&bytes, FUZZY_INDEX_MAGIC).is_err());
    }
}

#[test]
fn items_large_length_after_offset_does_not_wrap() {
    // count = 1, item length = 0xFFFF_FFF8, followed by 16 bytes.
    let mut bytes = Vec::new();
    bytes.extend_from_slice(FUZZY_INDEX_MAGIC);
    bytes.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0xFFFF_FFF8u32.to_le_bytes());
    bytes.extend_from_slice(b"abcdefghijklmnop");
    expect_err(deserialize_items(&bytes, FUZZY_INDEX_MAGIC), "truncated");
}

#[test]
fn items_structural_errors_are_precise() {
    expect_err(deserialize_items(&[], FUZZY_INDEX_MAGIC), "too short");
    expect_err(
        deserialize_items(b"RFZI\x01\x00", FUZZY_INDEX_MAGIC),
        "too short",
    );

    let mut bytes = fuzzy_payload(&["alpha"]);
    bytes[..4].copy_from_slice(b"XXXX");
    expect_err(
        deserialize_items(&bytes, FUZZY_INDEX_MAGIC),
        "bad magic bytes",
    );

    let mut bytes = fuzzy_payload(&["alpha", "beta"]);
    bytes.extend_from_slice(b"xyz");
    let message = expect_err(deserialize_items(&bytes, FUZZY_INDEX_MAGIC), "trailing");
    assert!(message.contains('3'), "{message}");

    let mut bytes = fuzzy_payload(&["alpha", "beta"]);
    let last = bytes.len() - 1;
    bytes[last] = 0xFF;
    let message = expect_err(deserialize_items(&bytes, FUZZY_INDEX_MAGIC), "UTF-8");
    assert!(message.contains("item 1"), "{message}");
}

// ---------------------------------------------------------------------------
// KeyedFuzzyIndex payloads
// ---------------------------------------------------------------------------

#[test]
fn keyed_roundtrip() {
    let cases: [(&[&[&str]], &[f64]); 4] = [
        (&[&["John", "Jane"], &["john@x", "jane@x"]], &[2.0, 1.0]),
        (&[&[], &[]], &[1.0, 1.0]),
        (&[&["東京", ""]], &[0.5]),
        (&[&["a"], &["b"], &["c"]], &[0.0, 1.0, 0.0]),
    ];
    for (columns, weights) in cases {
        let bytes = keyed_payload(columns, weights);
        let (key_texts, decoded_weights) = deserialize_keyed(&bytes, KEYED_INDEX_MAGIC).unwrap();
        assert_eq!(decoded_weights, weights);
        assert_eq!(key_texts.len(), columns.len());
        for (decoded, original) in key_texts.iter().zip(columns) {
            assert_eq!(decoded, original);
        }
    }
}

/// The exact ~29-byte payload from the audit: a valid two-key index whose
/// `num_items` was overwritten with `u32::MAX`. Before the fix this made
/// `Vec::with_capacity` request ~103 GB and abort the process.
#[test]
fn keyed_huge_num_items_does_not_allocate() {
    let mut bytes = keyed_payload(&[&["a", "b"], &["c", "d"]], &[1.0, 2.0]);
    set_u32(&mut bytes, NUM_ITEMS_OFFSET, u32::MAX);
    let (result, largest) =
        largest_allocation_during(|| deserialize_keyed(&bytes, KEYED_INDEX_MAGIC));
    expect_err(result, "Invalid data");
    assert!(
        largest <= parse_allocation_budget(bytes.len()),
        "parser requested {largest} bytes for a {}-byte input",
        bytes.len()
    );
}

#[test]
fn keyed_num_keys_field_boundaries() {
    let base = keyed_payload(&[&["a", "b"], &["c", "d"]], &[1.0, 2.0]);
    // 0x2000_0001 * 8 wraps to 8 on 32-bit targets.
    for num_keys in [1, 3, 0x2000_0001, 0x4000_0000, u32::MAX] {
        let mut bytes = base.clone();
        set_u32(&mut bytes, NUM_KEYS_OFFSET, num_keys);
        let (result, largest) =
            largest_allocation_during(|| deserialize_keyed(&bytes, KEYED_INDEX_MAGIC));
        assert!(result.is_err(), "num_keys={num_keys} should be rejected");
        assert!(largest <= parse_allocation_budget(bytes.len()));
    }
}

#[test]
fn keyed_num_items_field_boundaries() {
    let base = keyed_payload(&[&["a", "b"], &["c", "d"]], &[1.0, 2.0]);
    for num_items in [0, 1, 3, 0x4000_0000, 0x8000_0000, u32::MAX] {
        let mut bytes = base.clone();
        set_u32(&mut bytes, NUM_ITEMS_OFFSET, num_items);
        let (result, largest) =
            largest_allocation_during(|| deserialize_keyed(&bytes, KEYED_INDEX_MAGIC));
        assert!(result.is_err(), "num_items={num_items} should be rejected");
        assert!(largest <= parse_allocation_budget(bytes.len()));
    }
}

#[test]
fn keyed_zero_keys_with_items_is_rejected() {
    let mut bytes = keyed_payload(&[&[], &[]], &[1.0, 1.0]);
    // Rewrite as num_keys = 0, num_items = 5, no weights.
    bytes.truncate(16);
    set_u32(&mut bytes, NUM_KEYS_OFFSET, 0);
    set_u32(&mut bytes, NUM_ITEMS_OFFSET, 5);
    expect_err(deserialize_keyed(&bytes, KEYED_INDEX_MAGIC), "no keys");
}

#[test]
fn keyed_text_length_field_boundaries() {
    let base = keyed_payload(&[&["a", "b"], &["c", "d"]], &[1.0, 2.0]);
    // First key text length prefix follows the 16-byte header and 2 weights.
    let first_len = 16 + 2 * 8;
    for len in [0x8000_0000, 0xFFFF_FFF0, 0xFFFF_FFF8, u32::MAX] {
        let mut bytes = base.clone();
        set_u32(&mut bytes, first_len, len);
        expect_err(deserialize_keyed(&bytes, KEYED_INDEX_MAGIC), "truncated");
    }
}

#[test]
fn keyed_structural_errors_are_precise() {
    expect_err(deserialize_keyed(&[], KEYED_INDEX_MAGIC), "too short");
    expect_err(
        deserialize_keyed(&fuzzy_payload(&[]), KEYED_INDEX_MAGIC),
        "too short",
    );

    let bytes = keyed_payload(&[&["a"]], &[1.0]);
    expect_err(
        deserialize_keyed(&bytes, FUZZY_INDEX_MAGIC),
        "bad magic bytes",
    );

    let mut bytes = keyed_payload(&[&["a"]], &[1.0]);
    bytes.push(0);
    let message = expect_err(deserialize_keyed(&bytes, KEYED_INDEX_MAGIC), "trailing");
    assert!(message.contains('1'), "{message}");

    let bytes = keyed_payload(&[&["a", "b"]], &[1.0]);
    // Cut inside the weights.
    expect_err(
        deserialize_keyed(&bytes[..20], KEYED_INDEX_MAGIC),
        "weights",
    );

    let mut bytes = keyed_payload(&[&["a", "b"], &["c", "d"]], &[1.0, 1.0]);
    let last = bytes.len() - 1;
    bytes[last] = 0xC0;
    let message = expect_err(deserialize_keyed(&bytes, KEYED_INDEX_MAGIC), "UTF-8");
    assert!(message.contains("key 1"), "{message}");
    assert!(message.contains("item 1"), "{message}");
}

// ---------------------------------------------------------------------------
// Deterministic mutation fuzzing
// ---------------------------------------------------------------------------

/// SplitMix64: tiny deterministic PRNG so failures are reproducible.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as usize
        }
    }

    fn byte(&mut self) -> u8 {
        self.next_u64() as u8
    }
}

const INTERESTING_U32: [u32; 12] = [
    0,
    1,
    2,
    3,
    4,
    0xFF,
    0x7FFF_FFFF,
    0x8000_0000,
    0x2000_0001,
    0xFFFF_FFF0,
    0xFFFF_FFF8,
    u32::MAX,
];

fn mutate(rng: &mut Rng, input: &[u8]) -> Vec<u8> {
    let mut bytes = input.to_vec();
    let rounds = 1 + rng.below(3);
    for _ in 0..rounds {
        match rng.below(8) {
            // Flip one bit.
            0 if !bytes.is_empty() => {
                let i = rng.below(bytes.len());
                bytes[i] ^= 1 << rng.below(8);
            }
            // Overwrite one byte.
            1 if !bytes.is_empty() => {
                let i = rng.below(bytes.len());
                bytes[i] = rng.byte();
            }
            // Insert bytes.
            2 => {
                let at = rng.below(bytes.len() + 1);
                let n = 1 + rng.below(8);
                for _ in 0..n {
                    bytes.insert(at, rng.byte());
                }
            }
            // Delete a range.
            3 if !bytes.is_empty() => {
                let at = rng.below(bytes.len());
                let n = 1 + rng.below((bytes.len() - at).min(8));
                bytes.drain(at..at + n);
            }
            // Truncate.
            4 => {
                let keep = rng.below(bytes.len() + 1);
                bytes.truncate(keep);
            }
            // Append garbage.
            5 => {
                let n = 1 + rng.below(8);
                for _ in 0..n {
                    bytes.push(rng.byte());
                }
            }
            // Overwrite an aligned or unaligned u32 with an interesting value.
            6 if bytes.len() >= 4 => {
                let at = if rng.below(2) == 0 {
                    4 * rng.below(bytes.len() / 4)
                } else {
                    rng.below(bytes.len() - 3)
                };
                let value = INTERESTING_U32[rng.below(INTERESTING_U32.len())];
                bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
            }
            // Overwrite a header u32 field with an interesting value.
            _ if bytes.len() >= 16 => {
                let at = 4 * (1 + rng.below(3));
                let value = INTERESTING_U32[rng.below(INTERESTING_U32.len())];
                bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
            }
            _ => {}
        }
    }
    bytes
}

const FUZZ_CASES_PER_SEED: usize = 10_000;

fn fuzzy_seeds() -> Vec<Vec<u8>> {
    vec![
        fuzzy_payload(&[]),
        fuzzy_payload(&["apple"]),
        fuzzy_payload(&["apple", "banana", "cherry", "date"]),
        fuzzy_payload(&["東京", "café", "🎉", "", "naïve"]),
        serialize_items(&strings(&["legacy", "wasm"]), FUZZY_INDEX_WASM_MAGIC),
    ]
}

fn keyed_seeds() -> Vec<Vec<u8>> {
    vec![
        keyed_payload(&[], &[]),
        keyed_payload(&[&[], &[]], &[1.0, 1.0]),
        keyed_payload(&[&["John", "Jane"], &["john@x", "jane@x"]], &[2.0, 1.0]),
        keyed_payload(&[&["東京", "", "🎉"]], &[0.5]),
        keyed_payload(&[&["a"], &["b"], &["c"]], &[0.0, 1.0, 0.0]),
    ]
}

/// Run `check` on `FUZZ_CASES_PER_SEED` mutations of every seed and return
/// how many mutated inputs were accepted (so tests can make sure the
/// "accepted" branch is actually exercised).
fn fuzz(rng_seed: u64, seeds: &[Vec<u8>], mut check: impl FnMut(&[u8]) -> bool) -> usize {
    let mut rng = Rng(rng_seed);
    let mut accepted = 0;
    for seed in seeds {
        for _ in 0..FUZZ_CASES_PER_SEED {
            let bytes = mutate(&mut rng, seed);
            if check(&bytes) {
                accepted += 1;
            }
        }
    }
    accepted
}

#[test]
fn fuzz_items_parser() {
    let accepted = fuzz(0x5EED_0001, &fuzzy_seeds(), |bytes| {
        let magic: &[u8; 4] = match bytes.first_chunk() {
            Some(magic) => magic,
            None => FUZZY_INDEX_MAGIC,
        };
        let (result, largest) = largest_allocation_during(|| deserialize_items(bytes, magic));
        assert!(
            largest <= parse_allocation_budget(bytes.len()),
            "requested {largest} bytes for {bytes:02x?}"
        );
        // The encoding is canonical: anything accepted re-encodes to
        // exactly the same bytes.
        result.is_ok_and(|items| serialize_items(&items, magic) == bytes)
    });
    assert!(accepted > 0);
}

#[test]
fn fuzz_keyed_parser() {
    let accepted = fuzz(0x5EED_0002, &keyed_seeds(), |bytes| {
        let (result, largest) =
            largest_allocation_during(|| deserialize_keyed(bytes, KEYED_INDEX_MAGIC));
        assert!(
            largest <= parse_allocation_budget(bytes.len()),
            "requested {largest} bytes for {bytes:02x?}"
        );
        match result {
            Ok((key_texts, weights)) => {
                assert_eq!(
                    serialize_keyed(&key_texts, &weights, KEYED_INDEX_MAGIC),
                    bytes
                );
                true
            }
            Err(_) => false,
        }
    });
    assert!(accepted > 0);
}

// ---------------------------------------------------------------------------
// FuzzyIndex: index-level API used by the bindings
// ---------------------------------------------------------------------------

#[test]
fn fuzzy_index_writes_the_shared_magic() {
    let index = FuzzyIndexCore::new(strings(&["apple", "banana"]));
    let bytes = serialize_fuzzy_index(&index);
    assert_eq!(&bytes[..4], FUZZY_INDEX_MAGIC);
    assert_eq!(bytes, fuzzy_payload(&["apple", "banana"]));
}

#[test]
fn fuzzy_index_reads_every_accepted_magic() {
    assert_eq!(
        FUZZY_INDEX_ACCEPTED_MAGICS,
        &[FUZZY_INDEX_MAGIC, FUZZY_INDEX_WASM_MAGIC]
    );
    let items = strings(&["apple", "東京", ""]);
    for magic in FUZZY_INDEX_ACCEPTED_MAGICS {
        let restored = deserialize_fuzzy_index(&serialize_items(&items, magic)).unwrap();
        assert_eq!(restored.items(), items);
        // Re-serializing upgrades a legacy payload to the shared magic.
        assert_eq!(
            serialize_fuzzy_index(&restored),
            serialize_items(&items, FUZZY_INDEX_MAGIC)
        );
    }
}

#[test]
fn fuzzy_index_rejects_other_magics_with_a_hint() {
    let mut bytes = fuzzy_payload(&["a"]);
    bytes[..4].copy_from_slice(KEYED_INDEX_MAGIC);
    let message = expect_err(
        deserialize_fuzzy_index(&bytes).map(|i| i.size()),
        "bad magic bytes",
    );
    assert!(message.contains("\"RFZI\" or \"RFUZ\""), "{message}");
    assert!(message.contains("KeyedFuzzyIndex"), "{message}");

    bytes[..4].copy_from_slice(&[0, 1, 2, 3]);
    let message = expect_err(
        deserialize_fuzzy_index(&bytes).map(|i| i.size()),
        "bad magic bytes",
    );
    assert!(message.contains("0x00010203"), "{message}");

    for magic in [FUZZY_INDEX_MAGIC, FUZZY_INDEX_WASM_MAGIC] {
        let bytes = serialize_items(&strings(&["a", "b", "c"]), magic);
        let message = expect_err(
            deserialize_keyed(&bytes, KEYED_INDEX_MAGIC),
            "bad magic bytes",
        );
        assert!(message.contains("serialized FuzzyIndex"), "{message}");
    }
}

#[test]
fn fuzzy_index_roundtrips_every_state() {
    let mut index = FuzzyIndexCore::new(strings(&["apple", "banana", "cherry"]));
    let check = |index: &FuzzyIndexCore| {
        let bytes = serialize_fuzzy_index(index);
        let restored = deserialize_fuzzy_index(&bytes).unwrap();
        assert_eq!(restored.items(), index.items());
        assert_eq!(serialize_fuzzy_index(&restored), bytes);
    };
    check(&index);
    index.add("date".into());
    check(&index);
    assert!(index.remove(0));
    check(&index);
    index.destroy();
    check(&index);
    check(&FuzzyIndexCore::new(Vec::new()));
}

/// The bytes of `bytes` with a legacy FuzzyIndex magic normalized to the
/// one `serialize_fuzzy_index` writes.
fn with_current_fuzzy_magic(bytes: &[u8]) -> Vec<u8> {
    let mut normalized = bytes.to_vec();
    if normalized.starts_with(FUZZY_INDEX_WASM_MAGIC) {
        normalized[..4].copy_from_slice(FUZZY_INDEX_MAGIC);
    }
    normalized
}

#[test]
fn fuzz_fuzzy_index() {
    // Building an index has a fixed cost (the matcher's scratch space) on
    // top of the per-item data, so measure it instead of hard-coding it.
    let (_, baseline) = largest_allocation_during(|| FuzzyIndexCore::new(Vec::new()));
    let accepted = fuzz(0x5EED_0003, &fuzzy_seeds(), |bytes| {
        let (result, largest) = largest_allocation_during(|| deserialize_fuzzy_index(bytes));
        assert!(
            largest <= baseline + parse_allocation_budget(bytes.len()),
            "requested {largest} bytes for {bytes:02x?}"
        );
        match result {
            Ok(index) => {
                assert_eq!(
                    serialize_fuzzy_index(&index),
                    with_current_fuzzy_magic(bytes)
                );
                true
            }
            Err(message) => {
                assert!(
                    message.starts_with("Invalid data: ")
                        || message.starts_with("Unsupported format version: "),
                    "{message}"
                );
                false
            }
        }
    });
    assert!(accepted > 0);
}

// ---------------------------------------------------------------------------
// KeyedFuzzyIndex: index-level API used by the bindings
// ---------------------------------------------------------------------------

fn keyed_index(columns: &[&[&str]], weights: &[f64]) -> KeyedFuzzyIndexCore {
    let key_texts = columns.iter().map(|col| strings(col)).collect();
    KeyedFuzzyIndexCore::new(key_texts, weights.to_vec()).unwrap()
}

fn assert_same_keyed_state(a: &KeyedFuzzyIndexCore, b: &KeyedFuzzyIndexCore) {
    assert_eq!(a.size(), b.size());
    assert_eq!(a.key_texts(), b.key_texts());
    let bits = |w: &[f64]| w.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(a.weights()), bits(b.weights()));
}

#[test]
fn keyed_index_roundtrips_every_state() {
    let check = |index: &KeyedFuzzyIndexCore| {
        let bytes = serialize_keyed_index(index);
        let restored = deserialize_keyed_index(&bytes).unwrap();
        assert_same_keyed_state(&restored, index);
        assert_eq!(serialize_keyed_index(&restored), bytes);
    };

    let mut index = keyed_index(&[&["John", "Jane"], &["john@x", "jane@x"]], &[2.0, 1.0]);
    check(&index);
    index.add(strings(&["Bob", "bob@x"])).unwrap();
    check(&index);
    assert!(index.remove(0));
    check(&index);
    // Keys without items.
    check(&keyed_index(&[&[], &[]], &[1.0, 0.0]));
    // Zero-weight keys next to a positive one, and a negative zero.
    check(&keyed_index(&[&["a"], &["b"], &["c"]], &[0.0, 1.0, -0.0]));
    // A destroyed index used to serialize to bytes its own deserializer
    // rejected ("Total weight must be greater than zero").
    index.destroy();
    check(&index);
}

#[test]
fn destroyed_keyed_index_restores_as_destroyed() {
    let mut index = keyed_index(&[&["a", "b"]], &[1.0]);
    index.destroy();
    let bytes = serialize_keyed_index(&index);
    // destroy() keeps the key configuration: the payload has keys, no items.
    let no_items: &[&str] = &[];
    assert_eq!(bytes, keyed_payload(&[no_items], &[1.0]));

    let mut restored = deserialize_keyed_index(&bytes).unwrap();
    assert_eq!(restored.size(), 0);
    assert_eq!(restored.weights(), index.weights());
    // Behaves exactly like the destroyed original.
    restored.add(strings(&["x"])).unwrap();
    index.add(strings(&["x"])).unwrap();
    assert_eq!(
        serialize_keyed_index(&restored),
        serialize_keyed_index(&index)
    );
}

#[test]
fn legacy_zero_key_payload_restores_as_destroyed() {
    // Before destroy() kept the key configuration, a destroyed index
    // serialized with zero keys (and its own deserializer rejected it).
    let bytes = keyed_payload(&[], &[]);
    let mut restored = deserialize_keyed_index(&bytes).unwrap();
    assert_eq!(restored.size(), 0);
    assert!(restored.key_texts().is_empty());
    assert!(restored.weights().is_empty());
    assert!(
        restored
            .search(
                "a",
                None,
                None,
                CaseMatching::Smart,
                true,
                KeyScoreMode::Weighted
            )
            .is_empty()
    );
    assert_eq!(
        restored.add(strings(&["x"])).unwrap_err(),
        "Expected 0 key values, got 1"
    );
    assert_eq!(serialize_keyed_index(&restored), bytes);
}

#[test]
fn keyed_index_rejects_invalid_weights_precisely() {
    let cases: [(&[f64], &str); 6] = [
        (&[1.0, f64::NAN], "weight of key 1 is NaN"),
        (&[f64::INFINITY], "weight of key 0 is inf"),
        (&[f64::NEG_INFINITY, 1.0], "weight of key 0 is -inf"),
        (&[1.0, 2.0, -1.0], "weight of key 2 is -1"),
        (&[0.0], "total weight must be greater than zero"),
        (&[0.0, -0.0], "total weight must be greater than zero"),
    ];
    for (weights, needle) in cases {
        let columns: Vec<&[&str]> = weights.iter().map(|_| &["text"][..]).collect();
        let bytes = keyed_payload(&columns, weights);
        // The format-level parser returns weights as stored...
        let (_, stored) = deserialize_keyed(&bytes, KEYED_INDEX_MAGIC).unwrap();
        assert_eq!(stored.len(), weights.len());
        // ...and the index-level one validates them like the constructor.
        let message = expect_err(deserialize_keyed_index(&bytes).map(|i| i.size()), needle);
        assert!(message.starts_with("Invalid data: "), "{message}");
    }
}

#[test]
fn fuzz_keyed_index() {
    let (_, baseline) = largest_allocation_during(|| keyed_index(&[&[]], &[1.0]));
    let accepted = fuzz(0x5EED_0004, &keyed_seeds(), |bytes| {
        let (result, largest) = largest_allocation_during(|| deserialize_keyed_index(bytes));
        assert!(
            largest <= baseline + parse_allocation_budget(bytes.len()),
            "requested {largest} bytes for {bytes:02x?}"
        );
        match result {
            Ok(index) => {
                assert_eq!(serialize_keyed_index(&index), bytes);
                true
            }
            Err(message) => {
                assert!(
                    message.starts_with("Invalid data: ")
                        || message.starts_with("Unsupported format version: "),
                    "{message}"
                );
                false
            }
        }
    });
    assert!(accepted > 0);
}
