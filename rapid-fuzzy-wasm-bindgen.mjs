/* @ts-self-types="./rapid-fuzzy-wasm-bindgen.d.mts" */

/**
 * A persistent fuzzy search index backed by Rust-side data.
 *
 * Holds items in memory on the Rust side, avoiding repeated FFI overhead
 * for applications that search the same dataset multiple times.
 */
export class FuzzyIndex {
    static __wrap(ptr) {
        const obj = Object.create(FuzzyIndex.prototype);
        obj.__wbg_ptr = ptr;
        FuzzyIndexFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FuzzyIndexFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fuzzyindex_free(ptr, 0);
    }
    /**
     * Add multiple items to the index at once.
     * @param {string[]} items
     */
    addMany(items) {
        const ptr0 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.fuzzyindex_addMany(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Add a single item to the index.
     * @param {string} item
     */
    add(item) {
        const ptr0 = passStringToWasm0(item, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.fuzzyindex_add(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Find the closest matching string in the index.
     *
     * Returns the best match, or null if no match is found.
     * If `minScore` is provided, returns null when the best match scores below the threshold.
     * @param {string} query
     * @param {number | null} [minScore]
     * @returns {string | null}
     */
    closest(query, minScore) {
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fuzzyindex_closest(this.__wbg_ptr, ptr0, len0, !isLikeNone(minScore), isLikeNone(minScore) ? 0 : minScore);
        return takeObject(ret);
    }
    /**
     * Reconstruct a FuzzyIndex from a previously serialized Uint8Array.
     * @param {Uint8Array} data
     * @returns {FuzzyIndex}
     */
    static deserialize(data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.fuzzyindex_deserialize(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return FuzzyIndex.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Free the internal data. After calling this, the index is empty.
     *
     * The index stays usable: it behaves as an empty index (searches
     * return no results) and `add()` / `addMany()` work as before.
     */
    destroy() {
        wasm.fuzzyindex_destroy(this.__wbg_ptr);
    }
    /**
     * Create a new FuzzyIndex, returning a Promise (parity with the Node.js binding).
     *
     * The WebAssembly build has no worker thread, so the index is built
     * synchronously on the calling thread and the returned Promise is already
     * resolved. Prefer the constructor when you do not need a Promise.
     *
     * Invalid input (anything but an array of strings) rejects the returned
     * Promise with a `TypeError` instead of throwing synchronously.
     * @param {string[]} items
     * @returns {Promise<FuzzyIndex>}
     */
    static fromAsync(items) {
        const ret = wasm.fuzzyindex_fromAsync(addHeapObject(items));
        return takeObject(ret);
    }
    /**
     * Create a new FuzzyIndex from an array of strings.
     * @param {string[]} items
     */
    constructor(items) {
        const ptr0 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fuzzyindex_new(ptr0, len0);
        this.__wbg_ptr = ret;
        FuzzyIndexFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance: the last item moves into the
     * freed slot. Returns false, removing nothing, if `index` is out of
     * range (negative, or not less than `size`). Throws a `TypeError` if
     * `index` is not a number and a `RangeError` if it is not an integer
     * (`NaN`, `±Infinity` or a fraction), like `FuzzyObjectIndex.remove()`.
     * @param {number} index
     * @returns {boolean}
     */
    remove(index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.fuzzyindex_remove(retptr, this.__wbg_ptr, addHeapObject(index));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Search the index, returning only indices and scores (no item strings).
     *
     * The second argument accepts either a number (maxResults) or a SearchOptions object.
     * @param {string} query
     * @param {number | SearchOptions | null} [options]
     * @returns {IndexSearchResult[]}
     */
    searchIndices(query, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.fuzzyindex_searchIndices(retptr, this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Search the index for items matching the query.
     *
     * Returns matches sorted by score (best match first).
     * The second argument accepts either a number (maxResults) or a SearchOptions object.
     * @param {string} query
     * @param {number | SearchOptions | null} [options]
     * @returns {SearchResult[]}
     */
    search(query, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.fuzzyindex_search(retptr, this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Serialize the index to a compact binary format (Uint8Array).
     * @returns {Uint8Array}
     */
    serialize() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.fuzzyindex_serialize(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Return the number of items in the index.
     * @returns {number}
     */
    get size() {
        const ret = wasm.fuzzyindex_size(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) FuzzyIndex.prototype[Symbol.dispose] = FuzzyIndex.prototype.free;

/**
 * A persistent multi-key fuzzy search index backed by Rust-side data.
 *
 * Holds key text arrays and weights in memory on the Rust side.
 */
export class KeyedFuzzyIndex {
    static __wrap(ptr) {
        const obj = Object.create(KeyedFuzzyIndex.prototype);
        obj.__wbg_ptr = ptr;
        KeyedFuzzyIndexFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KeyedFuzzyIndexFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_keyedfuzzyindex_free(ptr, 0);
    }
    /**
     * Add multiple items to the index at once.
     *
     * Each element of `itemsKeyValues` is an array of key values for one item.
     * Throws if any element has the wrong number of key values.
     * @param {string[][]} itemsKeyValues
     */
    addMany(itemsKeyValues) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.keyedfuzzyindex_addMany(retptr, this.__wbg_ptr, addHeapObject(itemsKeyValues));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Add a single item to the index.
     *
     * `keyValues` must have the same length as the number of keys.
     * Throws if the length does not match.
     * @param {string[]} keyValues
     */
    add(keyValues) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.keyedfuzzyindex_add(retptr, this.__wbg_ptr, addHeapObject(keyValues));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Find the index of the closest matching item.
     *
     * Returns the index of the best match, or null if no match is found:
     * the index of the first result of
     * `search(query, { maxResults: 1, minScore, scoreMode, matchMode })`.
     *
     * The second argument accepts either a number (minScore shorthand) or a
     * KeyClosestOptions object: `minScore` makes it return null when the
     * best match scores below the threshold, and `scoreMode` and
     * `matchMode` work like the `search()` options of the same names
     * (defaults `"weighted"` and `"perKey"`).
     * @param {string} query
     * @param {number | KeyClosestOptions | null} [options]
     * @returns {number | null}
     */
    closest(query, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.keyedfuzzyindex_closest(retptr, this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Reconstruct a KeyedFuzzyIndex from a previously serialized Uint8Array.
     * @param {Uint8Array} data
     * @returns {KeyedFuzzyIndex}
     */
    static deserialize(data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.keyedfuzzyindex_deserialize(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return KeyedFuzzyIndex.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Free the internal data. After calling this, the index is empty.
     *
     * The key configuration is kept, so the index stays usable: it behaves
     * as an empty index and `add()` / `addMany()` work as before.
     */
    destroy() {
        wasm.keyedfuzzyindex_destroy(this.__wbg_ptr);
    }
    /**
     * Create a new KeyedFuzzyIndex.
     *
     * `keyTexts[k]` is an array of strings for key `k`, one per item.
     * All inner arrays must have the same length (the number of items).
     * `weights` holds one finite, non-negative weight per key.
     * @param {string[][]} keyTexts
     * @param {ArrayLike<number>} weights
     */
    constructor(keyTexts, weights) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArrayF64ToWasm0(weights, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.keyedfuzzyindex_new(retptr, addHeapObject(keyTexts), ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0;
            KeyedFuzzyIndexFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance: the last item moves into the
     * freed slot. Returns false, removing nothing, if `index` is out of
     * range (negative, or not less than `size`). Throws a `TypeError` if
     * `index` is not a number and a `RangeError` if it is not an integer
     * (`NaN`, `±Infinity` or a fraction), like `FuzzyObjectIndex.remove()`.
     * @param {number} index
     * @returns {boolean}
     */
    remove(index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.keyedfuzzyindex_remove(retptr, this.__wbg_ptr, addHeapObject(index));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Search the index for items matching the query.
     *
     * Returns results sorted by combined score (best match first), exactly
     * like `searchKeys()` on the same key texts and weights. The second
     * argument accepts either a number (maxResults) or a KeySearchOptions
     * object, whose `matchMode` selects how the query is matched against
     * the keys and `scoreMode` how the per-key scores are combined.
     * @param {string} query
     * @param {number | KeySearchOptions | null} [options]
     * @returns {KeySearchResult[]}
     */
    search(query, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.keyedfuzzyindex_search(retptr, this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Serialize the index to a compact binary format (Uint8Array).
     * @returns {Uint8Array}
     */
    serialize() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.keyedfuzzyindex_serialize(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Return the number of items in the index.
     * @returns {number}
     */
    get size() {
        const ret = wasm.keyedfuzzyindex_size(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) KeyedFuzzyIndex.prototype[Symbol.dispose] = KeyedFuzzyIndex.prototype.free;

/**
 * Find the closest matching string from a list.
 *
 * Returns the best match, or null if no match is found.
 * If `minScore` is provided, returns null when the best match scores below the threshold.
 * @param {string} query
 * @param {string[]} items
 * @param {number | null} [minScore]
 * @returns {string | null}
 */
export function closest(query, items, minScore) {
    const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.closest(ptr0, len0, ptr1, len1, !isLikeNone(minScore), isLikeNone(minScore) ? 0 : minScore);
    return takeObject(ret);
}

/**
 * Compute the Damerau-Levenshtein distance between two strings.
 *
 * Like Levenshtein, but also considers transpositions of two adjacent
 * characters as a single edit.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * Takes time proportional to the product of the two lengths: about 0.4 s
 * for two 10,000-character strings with the native addon.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function damerauLevenshtein(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.damerauLevenshtein(ptr0, len0, ptr1, len1);
    return ret >>> 0;
}

/**
 * Compute the Damerau-Levenshtein distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Uint32Array}
 */
export function damerauLevenshteinBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.damerauLevenshteinBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Damerau-Levenshtein distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1`, at most 4294967295 (enabling early termination
 * for better performance). `maxDistance` must be a non-negative integer or
 * `Infinity` (no limit); NaN, negative and fractional values throw an `Error`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [maxDistance]
 * @returns {Uint32Array}
 */
export function damerauLevenshteinMany(reference, candidates, maxDistance) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.damerauLevenshteinMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(maxDistance), isLikeNone(maxDistance) ? 0 : maxDistance);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Hamming distance between two strings.
 *
 * The Hamming distance counts the number of positions at which the corresponding
 * characters differ. It is only defined for strings of equal length.
 * Returns `null` if the strings have different lengths.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number | null}
 */
export function hamming(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.hamming(ptr0, len0, ptr1, len1);
    return takeObject(ret);
}

/**
 * Compute the Hamming distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 * Returns `null` for pairs with different lengths.
 * @param {string[][]} pairs
 * @returns {(number | null)[]}
 */
export function hammingBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.hammingBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Hamming distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * Returns `null` for candidates with a different length than the reference.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will also return `null` (enabling early termination for better performance).
 * `maxDistance` must be a non-negative integer or `Infinity` (no limit); NaN,
 * negative and fractional values throw an `Error`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [maxDistance]
 * @returns {(number | null)[]}
 */
export function hammingMany(reference, candidates, maxDistance) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.hammingMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(maxDistance), isLikeNone(maxDistance) ? 0 : maxDistance);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Indel distance between two strings.
 *
 * The Indel distance counts the minimum number of insertions and deletions
 * (no substitutions) required to transform one string into the other.
 * It equals `len(a) + len(b) - 2 * LCS_length(a, b)`.
 *
 * Useful when substitutions are semantically two operations (one deletion +
 * one insertion), such as in DNA sequence alignment.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function indel(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.indel(ptr0, len0, ptr1, len1);
    return ret >>> 0;
}

/**
 * Compute the Indel distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 * @param {string[][]} pairs
 * @returns {Uint32Array}
 */
export function indelBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.indelBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Indel distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1`, at most 4294967295 (enabling early termination
 * for better performance). `maxDistance` must be a non-negative integer or
 * `Infinity` (no limit); NaN, negative and fractional values throw an `Error`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [maxDistance]
 * @returns {Uint32Array}
 */
export function indelMany(reference, candidates, maxDistance) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.indelMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(maxDistance), isLikeNone(maxDistance) ? 0 : maxDistance);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Jaro similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function jaro(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.jaro(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the Jaro similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function jaroBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.jaroBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Jaro similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function jaroMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.jaroMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Jaro-Winkler similarity between two strings.
 *
 * A modification of Jaro that gives more weight to common prefixes.
 * Returns a value between 0.0 and 1.0.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function jaroWinkler(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.jaroWinkler(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the Jaro-Winkler similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function jaroWinklerBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.jaroWinklerBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Jaro-Winkler similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function jaroWinklerMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.jaroWinklerMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Levenshtein distance between two strings.
 *
 * The Levenshtein distance is the minimum number of single-character edits
 * (insertions, deletions, or substitutions) required to change one string
 * into the other.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function levenshtein(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.levenshtein(ptr0, len0, ptr1, len1);
    return ret >>> 0;
}

/**
 * Compute the Levenshtein distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 * @param {string[][]} pairs
 * @returns {Uint32Array}
 */
export function levenshteinBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.levenshteinBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Levenshtein distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1`, at most 4294967295 (enabling early termination
 * for better performance). `maxDistance` must be a non-negative integer or
 * `Infinity` (no limit); NaN, negative and fractional values throw an `Error`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [maxDistance]
 * @returns {Uint32Array}
 */
export function levenshteinMany(reference, candidates, maxDistance) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.levenshteinMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(maxDistance), isLikeNone(maxDistance) ? 0 : maxDistance);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the normalized Hamming similarity between two strings.
 *
 * Returns `null` if the strings have different lengths.
 * Returns a value between 0.0 (no matching characters) and 1.0 (identical).
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number | null}
 */
export function normalizedHamming(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.normalizedHamming(ptr0, len0, ptr1, len1);
    return takeObject(ret);
}

/**
 * Compute the normalized Hamming similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of scores in the same order as the input pairs.
 * Returns `null` for pairs with different lengths.
 * @param {string[][]} pairs
 * @returns {(number | null)[]}
 */
export function normalizedHammingBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.normalizedHammingBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the normalized Hamming similarity from one reference string to many candidates.
 *
 * Returns an array of scores, one per candidate, in the same order as the input.
 * Returns `null` for candidates with a different length than the reference.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will also return `null` (enabling early termination for better performance).
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {(number | null)[]}
 */
export function normalizedHammingMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.normalizedHammingMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the normalized Indel similarity between two strings.
 *
 * `1 - indel(a, b) / (length of a + length of b)`, counted in characters:
 * the measure behind `fuzz.ratio` in RapidFuzz and fuzzball, on a 0.0-1.0
 * scale (fuzzball also lower-cases and strips punctuation by default; this
 * function does not). Returns a value between 0.0 (completely different)
 * and 1.0 (identical).
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function normalizedIndel(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.normalizedIndel(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the normalized Indel similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function normalizedIndelBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.normalizedIndelBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the normalized Indel similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function normalizedIndelMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.normalizedIndelMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the normalized Levenshtein similarity between two strings.
 *
 * `1 - levenshtein(a, b) / max(length of a, length of b)`, counted in
 * characters. Returns a value between 0.0 (completely different) and 1.0
 * (identical).
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function normalizedLevenshtein(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.normalizedLevenshtein(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the normalized Levenshtein similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function normalizedLevenshteinBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.normalizedLevenshteinBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the normalized Levenshtein similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function normalizedLevenshteinMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.normalizedLevenshteinMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the partial ratio between two strings.
 *
 * Lower-cases both strings and collapses whitespace runs into single
 * spaces, then compares the shorter string with every window of the same
 * length in the longer one and returns the highest normalized Levenshtein
 * similarity. Useful when one string is a substring or truncation of the
 * other; it does not match abbreviations (`MSFT` vs `Microsoft` scores low).
 * Scores can differ from fuzzball's / RapidFuzz's `partial_ratio`, which
 * use a different alignment. Returns a value between 0.0 and 1.0.
 *
 * Takes time proportional to the length of the longer string times the
 * square of the length of the shorter one: about 0.4 s for a
 * 1,000-character string against a 10,000-character one with the native
 * addon.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function partialRatio(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.partialRatio(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the partial ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function partialRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.partialRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the partial ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function partialRatioMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.partialRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Perform fuzzy search over a list of strings.
 *
 * Returns matches sorted by score (best match first).
 * Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
 *
 * The third argument accepts either a number (maxResults for backward
 * compatibility) or a SearchOptions object.
 * @param {string} query
 * @param {string[]} items
 * @param {number | SearchOptions | null} [options]
 * @returns {SearchResult[]}
 */
export function search(query, items, options) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.search(retptr, ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Perform fuzzy search across multiple text keys with weights.
 *
 * `keyTexts[k]` is an array of strings for key `k`, one per item.
 * `weights` specifies the relative importance of each key.
 * `options` is a `KeySearchOptions` object or a number (maxResults); its
 * `matchMode` selects how the query is matched against the keys and its
 * `scoreMode` how the per-key scores are combined.
 *
 * Returns results sorted by combined score (best match first), exactly like
 * `KeyedFuzzyIndex.search` on the same key texts and weights.
 * Throws an `Error` for invalid input (key texts of different lengths, a
 * weight count that differs from the key count, negative, NaN or infinite
 * weights, or weights summing to 0 or Infinity), like the `KeyedFuzzyIndex`
 * constructor, and a `TypeError` for invalid options (such as an unknown
 * `scoreMode` or `matchMode`).
 * @param {string} query
 * @param {string[][]} keyTexts
 * @param {ArrayLike<number>} weights
 * @param {number | KeySearchOptions | null} [options]
 * @returns {KeySearchResult[]}
 */
export function searchKeys(query, keyTexts, weights, options) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF64ToWasm0(weights, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.searchKeys(retptr, ptr0, len0, addHeapObject(keyTexts), ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Sorensen-Dice coefficient between two strings.
 *
 * Compares the bigrams (pairs of consecutive characters) of the two strings
 * after removing all whitespace; case and Unicode normalization are not
 * adjusted. Identical strings score 1.0; otherwise a string with fewer than
 * two characters left scores 0.0. Returns a value between 0.0 and 1.0.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function sorensenDice(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.sorensenDice(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the Sorensen-Dice coefficient for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function sorensenDiceBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.sorensenDiceBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the Sorensen-Dice coefficient from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 * Reference bigrams are pre-computed once and reused for all candidates.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function sorensenDiceMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.sorensenDiceMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the token set ratio between two strings.
 *
 * Lower-cases both strings and splits them into sets of whitespace-separated
 * tokens (duplicates count once). With the shared tokens sorted and joined
 * as `common`, returns the highest normalized Levenshtein similarity among
 * `common + rest of a` vs `common + rest of b`, `common` vs
 * `common + rest of a`, and `common` vs `common + rest of b`. It is 1.0
 * when the tokens of one string are a subset of the other's.
 * Returns a value between 0.0 and 1.0.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function tokenSetRatio(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.tokenSetRatio(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the token set ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function tokenSetRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.tokenSetRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the token set ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function tokenSetRatioMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.tokenSetRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the token sort ratio between two strings.
 *
 * Lower-cases both strings, splits them on whitespace, sorts the tokens and
 * joins them with single spaces, then returns the normalized Levenshtein
 * similarity of the two results, so word order does not matter. Punctuation
 * is kept: `Smith,` and `Smith` are different tokens.
 * Returns a value between 0.0 (completely different) and 1.0 (identical after sorting).
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function tokenSortRatio(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.tokenSortRatio(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the token sort ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function tokenSortRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.tokenSortRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the token sort ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function tokenSortRatioMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.tokenSortRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the weighted ratio between two strings.
 *
 * Returns the highest of: the normalized Levenshtein similarity of the
 * strings as given and after lower-casing and collapsing whitespace,
 * `tokenSortRatio`, `tokenSetRatio` and `partialRatio`. Unlike `WRatio` in
 * fuzzball / RapidFuzz, no score is scaled down or weighted by the length
 * ratio of the strings, so scores are often higher than `WRatio`'s.
 * Includes the cost of `partialRatio` (see there) when the strings differ
 * in length. Returns a value between 0.0 and 1.0.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function weightedRatio(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.weightedRatio(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Compute the weighted ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 * @param {string[][]} pairs
 * @returns {Float64Array}
 */
export function weightedRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.weightedRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Compute the weighted ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [minSimilarity]
 * @returns {Float64Array}
 */
export function weightedRatioMany(reference, candidates, minSimilarity) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.weightedRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(minSimilarity), isLikeNone(minSimilarity) ? 0 : minSimilarity);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_67e7344beaa85059: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_Number_c54e7112a3fa7e3e: function(arg0) {
            const ret = Number(getObject(arg0));
            return ret;
        },
        __wbg_String_8564e559799eccda: function(arg0, arg1) {
            const ret = String(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_boolean_get_7a12af2b3f899c5a: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0e68cf47c9cbd9b0: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_50072d4d6e45c193: function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_is_function_fcda5e3902d732fe: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_5160b3e381865372: function(arg0) {
            const ret = getObject(arg0) === null;
            return ret;
        },
        __wbg___wbindgen_is_object_edb6b15aa3afe12e: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_8c687d0b90d5b524: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_3c30021c243b64cd: function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_number_get_1dc732b810cb937c: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_92ab86bb19cbc12f: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_5d9e815e6fdf150f: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg___wbindgen_typeof_8e630e4d777e2338: function(arg0) {
            const ret = typeof getObject(arg0);
            return addHeapObject(ret);
        },
        __wbg_call_269c5566fbede3eb: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_done_cffed884d87aa22e: function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        },
        __wbg_fuzzyindex_new: function(arg0) {
            const ret = FuzzyIndex.__wrap(arg0);
            return addHeapObject(ret);
        },
        __wbg_get_6cf5a4d4d8ad3c5a: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_get_unchecked_363572bdd397d473: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        },
        __wbg_instanceof_ArrayBuffer_d4ff01f8247925ae: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_598adc0fef426aa8: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_5674713bb7b79043: function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        },
        __wbg_isSafeInteger_8f51c743827d1ec5: function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        },
        __wbg_iterator_22ddeb808cf55a6f: function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        },
        __wbg_length_31bdaf014f5fbde2: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_4e1adc0d42e23620: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_new_1543621bea52a223: function(arg0, arg1) {
            const ret = new RangeError(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_new_1da3429bc3c4541c: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_new_9ca27d4bce9deee9: function(arg0, arg1) {
            const ret = new TypeError(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_new_a32a1ab6c6655abe: function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_new_bebc3f4757acf305: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_ffa92086ea89f79c: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_next_95053e306b1c3aed: function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        },
        __wbg_next_f31ecb8646d2c605: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_prototypesetcall_ae9f5e7459250748: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        },
        __wbg_push_bfdf956ba476f65b: function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        },
        __wbg_reject_bea6d825081bd4d7: function(arg0) {
            const ret = Promise.reject(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_resolve_35ec7e0c6af4c82c: function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_set_13d25b81ab403f5e: function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        },
        __wbg_set_6be42768c690e380: function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        },
        __wbg_value_c227f843d21da141: function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./rapid-fuzzy-wasm-bindgen_bg.js": import0,
    };
}

const FuzzyIndexFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fuzzyindex_free(ptr, 1));
const KeyedFuzzyIndexFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_keyedfuzzyindex_free(ptr, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addHeapObject(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('rapid-fuzzy-wasm-bindgen_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
