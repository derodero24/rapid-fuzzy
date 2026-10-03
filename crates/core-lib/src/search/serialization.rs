//! Compact binary serialization for `FuzzyIndex` and `KeyedFuzzyIndex`.
//!
//! All integers are little-endian. Every string is stored as a `u32` byte
//! length followed by that many bytes of UTF-8.
//!
//! `FuzzyIndex`: `[magic 4B] [version u32] [count u32] [count x string]`
//!
//! `KeyedFuzzyIndex`:
//! `[magic 4B] [version u32] [num_keys u32] [num_items u32]`
//! `[num_keys x f64 weight] [num_keys x num_items strings, column-major]`
//!
//! Deserializers treat their input as untrusted. They never panic, never
//! allocate more than a small multiple of the input length (counts read from
//! the header are checked against the bytes actually present before any
//! allocation), use overflow-free bounds checks so 32-bit targets (wasm)
//! behave like 64-bit ones, and reject truncated data, trailing bytes and
//! invalid UTF-8 with a message that says what is wrong and where. Every
//! payload a deserializer accepts re-serializes to exactly the same bytes.

use std::fmt;

/// Magic bytes identifying a serialized `FuzzyIndex`.
pub const FUZZY_INDEX_MAGIC: &[u8; 4] = b"RFZI";

/// Magic bytes identifying a serialized `FuzzyIndex` (WASM variant).
pub const FUZZY_INDEX_WASM_MAGIC: &[u8; 4] = b"RFUZ";

/// Magic bytes identifying a serialized `KeyedFuzzyIndex`.
pub const KEYED_INDEX_MAGIC: &[u8; 4] = b"RFKI";

/// Current serialization format version.
pub const SERIALIZE_VERSION: u32 = 1;

/// Magic + version + item count.
const ITEMS_HEADER_LEN: usize = 12;
/// Magic + version + key count + item count.
const KEYED_HEADER_LEN: usize = 16;
/// Size of the length prefix in front of every string.
const LEN_PREFIX_SIZE: usize = 4;
/// Size of one serialized weight.
const WEIGHT_SIZE: usize = 8;

// ---------------------------------------------------------------------------
// Format-level API
// ---------------------------------------------------------------------------

/// Serialize a list of items into the `FuzzyIndex` format under `magic`.
///
/// Each item and the item count must fit in a `u32`, which always holds for
/// strings that came from JavaScript.
pub fn serialize_items(items: &[String], magic: &[u8; 4]) -> Vec<u8> {
    let payload_len: usize = items.iter().map(|s| LEN_PREFIX_SIZE + s.len()).sum();
    let mut buf = Vec::with_capacity(ITEMS_HEADER_LEN + payload_len);
    buf.extend_from_slice(magic);
    buf.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
    buf.extend_from_slice(&(items.len() as u32).to_le_bytes());
    for item in items {
        write_string(&mut buf, item);
    }
    buf
}

/// Deserialize a list of items written by [`serialize_items`] under exactly `magic`.
///
/// # Errors
///
/// Returns a message describing the first problem found: a short header,
/// wrong magic, unsupported version, an item count or length that exceeds
/// the payload, invalid UTF-8, or trailing bytes.
pub fn deserialize_items(bytes: &[u8], magic: &[u8; 4]) -> Result<Vec<String>, String> {
    parse_items(bytes, &[magic])
}

/// Serialize keyed-index data into the `KeyedFuzzyIndex` format under `magic`.
///
/// `key_texts` holds one column per key, all of the same length, and
/// `weights` one entry per key.
pub fn serialize_keyed(key_texts: &[Vec<String>], weights: &[f64], magic: &[u8; 4]) -> Vec<u8> {
    let num_keys = weights.len();
    let num_items = key_texts.first().map_or(0, Vec::len);
    let payload_len: usize = key_texts
        .iter()
        .flatten()
        .map(|s| LEN_PREFIX_SIZE + s.len())
        .sum();

    let mut buf = Vec::with_capacity(KEYED_HEADER_LEN + num_keys * WEIGHT_SIZE + payload_len);
    buf.extend_from_slice(magic);
    buf.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
    buf.extend_from_slice(&(num_keys as u32).to_le_bytes());
    buf.extend_from_slice(&(num_items as u32).to_le_bytes());
    for &w in weights {
        buf.extend_from_slice(&w.to_le_bytes());
    }
    for item in key_texts.iter().flatten() {
        write_string(&mut buf, item);
    }
    buf
}

/// Deserialize keyed-index data written by [`serialize_keyed`] under exactly `magic`.
///
/// Returns `(key_texts, weights)`. Weights are returned as stored, without
/// validation.
///
/// # Errors
///
/// Returns a message describing the first problem found: a short header,
/// wrong magic, unsupported version, zero keys with a non-zero item count
/// (only a destroyed index has zero keys, and it has no items), key or item
/// counts that exceed the payload, a truncated weight or key text, invalid
/// UTF-8, or trailing bytes.
pub fn deserialize_keyed(
    bytes: &[u8],
    magic: &[u8; 4],
) -> Result<(Vec<Vec<String>>, Vec<f64>), String> {
    let mut reader = Reader::new(bytes);
    read_preamble(&mut reader, KEYED_HEADER_LEN, &[magic])?;
    let num_keys = reader.count("key count")?;
    let num_items = reader.count("item count")?;

    if num_keys == 0 && num_items != 0 {
        return Err(format!(
            "Invalid data: header declares {num_items} items but no keys"
        ));
    }

    // Check the declared sizes against the bytes actually present before
    // allocating anything sized by them.
    if num_keys > reader.remaining() / WEIGHT_SIZE {
        return Err(format!(
            "Invalid data: truncated weights: {num_keys} keys need {} bytes, but only {} remain",
            num_keys as u64 * WEIGHT_SIZE as u64,
            reader.remaining()
        ));
    }
    let mut weights = Vec::with_capacity(num_keys);
    for key in 0..num_keys {
        weights.push(reader.weight(key)?);
    }

    // Every key text needs at least its length prefix.
    let min_text_bytes = num_keys
        .checked_mul(num_items)
        .and_then(|n| n.checked_mul(LEN_PREFIX_SIZE));
    if min_text_bytes.is_none_or(|n| n > reader.remaining()) {
        return Err(format!(
            "Invalid data: {num_keys} keys x {num_items} items need at least {} bytes of key text, but only {} remain",
            num_keys as u128 * num_items as u128 * LEN_PREFIX_SIZE as u128,
            reader.remaining()
        ));
    }
    let mut key_texts = Vec::with_capacity(num_keys);
    for key in 0..num_keys {
        let mut column = Vec::with_capacity(num_items);
        for item in 0..num_items {
            column.push(reader.string(Slot::KeyText { key, item })?);
        }
        key_texts.push(column);
    }

    reader.finish("the last key text")?;
    Ok((key_texts, weights))
}

// ---------------------------------------------------------------------------
// Parsing internals
// ---------------------------------------------------------------------------

fn write_string(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn parse_items(bytes: &[u8], accepted_magics: &[&[u8; 4]]) -> Result<Vec<String>, String> {
    let mut reader = Reader::new(bytes);
    read_preamble(&mut reader, ITEMS_HEADER_LEN, accepted_magics)?;
    let count = reader.count("item count")?;

    // Every item needs at least its length prefix; check before allocating.
    let max_items = reader.remaining() / LEN_PREFIX_SIZE;
    if count > max_items {
        return Err(format!(
            "Invalid data: item count {count} exceeds the payload: {} remaining bytes hold at most {max_items} items",
            reader.remaining()
        ));
    }
    let mut items = Vec::with_capacity(count);
    for item in 0..count {
        items.push(reader.string(Slot::Item(item))?);
    }

    reader.finish("the last item")?;
    Ok(items)
}

/// Check the header length, magic and version.
fn read_preamble(
    reader: &mut Reader<'_>,
    header_len: usize,
    accepted_magics: &[&[u8; 4]],
) -> Result<(), String> {
    if reader.remaining() < header_len {
        return Err(format!(
            "Invalid data: too short: the header needs {header_len} bytes, got {}",
            reader.remaining()
        ));
    }
    let magic: [u8; 4] = reader
        .array()
        .ok_or_else(|| reader.truncated("magic bytes"))?;
    if !accepted_magics.iter().any(|m| **m == magic) {
        return Err(bad_magic(magic, accepted_magics));
    }
    let version = reader.u32("format version")?;
    if version != SERIALIZE_VERSION {
        return Err(format!(
            "Unsupported format version: expected {SERIALIZE_VERSION}, got {version}"
        ));
    }
    Ok(())
}

fn bad_magic(found: [u8; 4], accepted_magics: &[&[u8; 4]]) -> String {
    let expected = accepted_magics
        .iter()
        .map(|m| display_magic(**m))
        .collect::<Vec<_>>()
        .join(" or ");
    let hint = if &found == KEYED_INDEX_MAGIC {
        " (this is a serialized KeyedFuzzyIndex)"
    } else if &found == FUZZY_INDEX_MAGIC || &found == FUZZY_INDEX_WASM_MAGIC {
        " (this is a serialized FuzzyIndex)"
    } else {
        ""
    };
    format!(
        "Invalid data: bad magic bytes: expected {expected}, got {}{hint}",
        display_magic(found)
    )
}

/// Show magic bytes as `"RFZI"` when printable, else as `0x52465a00`.
fn display_magic(magic: [u8; 4]) -> String {
    if magic.iter().all(u8::is_ascii_graphic) {
        let text: String = magic.iter().copied().map(char::from).collect();
        format!("\"{text}\"")
    } else {
        format!("0x{:08x}", u32::from_be_bytes(magic))
    }
}

/// Convert a `u32` read from the payload to `usize`.
///
/// Saturates on targets where `usize` is narrower than 32 bits; the value is
/// then rejected by the bounds checks that follow.
fn to_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// Which string a parse error refers to.
#[derive(Clone, Copy)]
enum Slot {
    Item(usize),
    KeyText { key: usize, item: usize },
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Item(item) => write!(f, "item {item}"),
            Self::KeyText { key, item } => write!(f, "key {key} item {item}"),
        }
    }
}

/// Cursor over untrusted bytes.
///
/// Reads shrink the remaining slice instead of adding to an offset, so no
/// bounds check can overflow, whatever the width of `usize`.
struct Reader<'a> {
    rest: &'a [u8],
    total_len: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            rest: bytes,
            total_len: bytes.len(),
        }
    }

    fn remaining(&self) -> usize {
        self.rest.len()
    }

    fn offset(&self) -> usize {
        self.total_len - self.rest.len()
    }

    /// Error for a fixed-size field that is cut off (built lazily, only on failure).
    fn truncated(&self, what: impl fmt::Display) -> String {
        format!(
            "Invalid data: truncated: missing {what} at byte {}",
            self.offset()
        )
    }

    /// Consume the next `n` bytes, or nothing if fewer remain.
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let (head, tail) = self.rest.split_at_checked(n)?;
        self.rest = tail;
        Some(head)
    }

    /// Consume the next `N` bytes, or nothing if fewer remain.
    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (head, tail) = self.rest.split_first_chunk::<N>()?;
        self.rest = tail;
        Some(*head)
    }

    fn u32(&mut self, what: &str) -> Result<u32, String> {
        self.array()
            .map(u32::from_le_bytes)
            .ok_or_else(|| self.truncated(what))
    }

    fn count(&mut self, what: &str) -> Result<usize, String> {
        self.u32(what).map(to_usize)
    }

    fn weight(&mut self, key: usize) -> Result<f64, String> {
        self.array()
            .map(f64::from_le_bytes)
            .ok_or_else(|| self.truncated(format_args!("the weight of key {key}")))
    }

    /// Read one length-prefixed UTF-8 string.
    fn string(&mut self, slot: Slot) -> Result<String, String> {
        let at = self.offset();
        let len = self
            .array()
            .map(u32::from_le_bytes)
            .ok_or_else(|| self.truncated(format_args!("the length of {slot}")))?;
        let bytes = self.take(to_usize(len)).ok_or_else(|| {
            format!(
                "Invalid data: truncated: {slot} at byte {at} declares {len} bytes, but only {} remain",
                self.remaining()
            )
        })?;
        let text = std::str::from_utf8(bytes)
            .map_err(|e| format!("Invalid data: {slot} at byte {at} is not valid UTF-8: {e}"))?;
        Ok(text.to_owned())
    }

    /// Require that the whole input has been consumed.
    fn finish(&self, after: &str) -> Result<(), String> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Invalid data: {} trailing bytes after {after} (at byte {})",
                self.remaining(),
                self.offset()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_never_overflows() {
        let bytes = [1u8, 2, 3, 4, 5];
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.take(2), Some(&[1u8, 2][..]));
        // On 32-bit targets `offset + len` for these lengths wraps around;
        // the reader must reject them without consuming anything.
        for n in [usize::MAX, usize::MAX - 1, usize::MAX - 2, 4] {
            assert_eq!(reader.take(n), None);
            assert_eq!(reader.offset(), 2);
        }
        assert_eq!(reader.take(3), Some(&[3u8, 4, 5][..]));
        assert_eq!(reader.take(0), Some(&[][..]));
        assert_eq!(reader.take(1), None);
    }

    #[test]
    fn to_usize_is_lossless_on_32_and_64_bit() {
        assert_eq!(to_usize(0), 0);
        assert_eq!(to_usize(u32::MAX) as u64, u64::from(u32::MAX));
    }

    #[test]
    fn display_magic_formats() {
        assert_eq!(display_magic(*b"RFZI"), "\"RFZI\"");
        assert_eq!(display_magic([0, 1, 0xAB, b'Z']), "0x0001ab5a");
        // Space is not graphic, so this falls back to hex.
        assert_eq!(display_magic(*b"RF I"), "0x52462049");
    }
}
