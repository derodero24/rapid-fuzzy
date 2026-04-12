/// Magic bytes identifying a serialized FuzzyIndex.
pub const FUZZY_INDEX_MAGIC: &[u8; 4] = b"RFZI";

/// Magic bytes identifying a serialized FuzzyIndex (WASM variant).
pub const FUZZY_INDEX_WASM_MAGIC: &[u8; 4] = b"RFUZ";

/// Magic bytes identifying a serialized KeyedFuzzyIndex.
pub const KEYED_INDEX_MAGIC: &[u8; 4] = b"RFKI";

/// Current serialization format version.
pub const SERIALIZE_VERSION: u32 = 1;

/// Serialize a list of items into a compact binary format.
///
/// Format: `[magic 4B] [version u32 LE] [count u32 LE] [items...]`
/// Each item: `[len u32 LE] [utf-8 bytes]`
pub fn serialize_items(items: &[String], magic: &[u8; 4]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(magic);
    buf.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
    buf.extend_from_slice(&(items.len() as u32).to_le_bytes());
    for item in items {
        buf.extend_from_slice(&(item.len() as u32).to_le_bytes());
        buf.extend_from_slice(item.as_bytes());
    }
    buf
}

/// Deserialize a list of items from a compact binary format.
///
/// Returns the list of items on success, or an error message on failure.
pub fn deserialize_items(bytes: &[u8], magic: &[u8; 4]) -> Result<Vec<String>, String> {
    let header_size = magic.len() + 4 + 4; // magic + version + count

    if bytes.len() < header_size {
        return Err("Invalid data: too short".into());
    }

    if &bytes[0..4] != magic {
        return Err("Invalid data: bad magic bytes".into());
    }

    let version = u32::from_le_bytes(
        bytes[4..8]
            .try_into()
            .map_err(|_| "Invalid data: truncated header".to_string())?,
    );
    if version != SERIALIZE_VERSION {
        return Err(format!(
            "Unsupported format version: expected {SERIALIZE_VERSION}, got {version}"
        ));
    }

    let count = u32::from_le_bytes(
        bytes[8..12]
            .try_into()
            .map_err(|_| "Invalid data: truncated header".to_string())?,
    ) as usize;
    let mut offset = header_size;

    // Reject obviously invalid counts before allocating.
    // Each item needs at least 4 bytes (length field), so count cannot
    // exceed the remaining payload divided by 4.
    let max_possible = bytes.len().saturating_sub(header_size) / 4;
    if count > max_possible {
        return Err("Invalid data: item count exceeds payload size".into());
    }

    let mut items = Vec::with_capacity(count);

    for _ in 0..count {
        if offset + 4 > bytes.len() {
            return Err("Invalid data: truncated".into());
        }
        let len = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| "Invalid data: truncated".to_string())?,
        ) as usize;
        offset += 4;
        if offset + len > bytes.len() {
            return Err("Invalid data: truncated".into());
        }
        let s = std::str::from_utf8(&bytes[offset..offset + len])
            .map_err(|e| format!("Invalid UTF-8: {e}"))?;
        items.push(s.to_owned());
        offset += len;
    }

    if offset != bytes.len() {
        return Err("Invalid data: trailing bytes".into());
    }

    Ok(items)
}

/// Serialize a keyed index into a compact binary format.
///
/// Format:
///   `[magic 4B] [version u32 LE] [num_keys u32 LE] [num_items u32 LE]`
///   `[weights: num_keys x f64 LE]`
///   `[key_texts column-major: for each key, for each item: [len u32 LE][utf-8 bytes]]`
pub fn serialize_keyed(key_texts: &[Vec<String>], weights: &[f64], magic: &[u8; 4]) -> Vec<u8> {
    let num_keys = weights.len();
    let num_items = key_texts.first().map_or(0, |v| v.len());

    let mut buf = Vec::new();
    buf.extend_from_slice(magic);
    buf.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());
    buf.extend_from_slice(&(num_keys as u32).to_le_bytes());
    buf.extend_from_slice(&(num_items as u32).to_le_bytes());

    for &w in weights {
        buf.extend_from_slice(&w.to_le_bytes());
    }

    for key_col in key_texts {
        for item in key_col {
            buf.extend_from_slice(&(item.len() as u32).to_le_bytes());
            buf.extend_from_slice(item.as_bytes());
        }
    }

    buf
}

/// Deserialize a keyed index from a compact binary format.
///
/// Returns `(key_texts, weights)` on success, or an error message on failure.
pub fn deserialize_keyed(
    bytes: &[u8],
    magic: &[u8; 4],
) -> Result<(Vec<Vec<String>>, Vec<f64>), String> {
    let header_size = 4 + 4 + 4 + 4; // magic + version + num_keys + num_items

    if bytes.len() < header_size {
        return Err("Invalid data: too short".into());
    }

    if &bytes[0..4] != magic {
        return Err("Invalid data: bad magic bytes".into());
    }

    let version = u32::from_le_bytes(
        bytes[4..8]
            .try_into()
            .map_err(|_| "Invalid data: truncated header".to_string())?,
    );
    if version != SERIALIZE_VERSION {
        return Err(format!(
            "Unsupported format version: expected {SERIALIZE_VERSION}, got {version}"
        ));
    }

    let num_keys = u32::from_le_bytes(
        bytes[8..12]
            .try_into()
            .map_err(|_| "Invalid data: truncated header".to_string())?,
    ) as usize;

    let num_items = u32::from_le_bytes(
        bytes[12..16]
            .try_into()
            .map_err(|_| "Invalid data: truncated header".to_string())?,
    ) as usize;

    let mut offset = header_size;

    // Read weights (num_keys x 8 bytes each)
    let weights_size = num_keys * 8;
    if offset + weights_size > bytes.len() {
        return Err("Invalid data: truncated weights".into());
    }
    let mut weights = Vec::with_capacity(num_keys);
    for _ in 0..num_keys {
        let w = f64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| "Invalid data: truncated weight".to_string())?,
        );
        weights.push(w);
        offset += 8;
    }

    // Read key_texts column-major
    let mut key_texts: Vec<Vec<String>> = Vec::with_capacity(num_keys);
    for _ in 0..num_keys {
        let mut col = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            if offset + 4 > bytes.len() {
                return Err("Invalid data: truncated".into());
            }
            let len = u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .map_err(|_| "Invalid data: truncated".to_string())?,
            ) as usize;
            offset += 4;
            if offset + len > bytes.len() {
                return Err("Invalid data: truncated".into());
            }
            let s = std::str::from_utf8(&bytes[offset..offset + len])
                .map_err(|e| format!("Invalid UTF-8: {e}"))?;
            col.push(s.to_owned());
            offset += len;
        }
        key_texts.push(col);
    }

    if offset != bytes.len() {
        return Err("Invalid data: trailing bytes".into());
    }

    Ok((key_texts, weights))
}
