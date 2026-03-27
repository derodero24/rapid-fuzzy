/**
 * A persistent fuzzy search index backed by Rust-side data.
 *
 * Holds items in memory on the Rust side, avoiding repeated FFI overhead
 * for applications that search the same dataset multiple times.
 */
export class FuzzyIndex {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
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
     * Add a single item to the index.
     * @param {string} item
     */
    add(item) {
        const ptr0 = passStringToWasm0(item, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.fuzzyindex_add(this.__wbg_ptr, ptr0, len0);
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
     * Find the closest matching string in the index.
     *
     * Returns the best match, or null if no match is found.
     * @param {string} query
     * @param {number | null} [min_score]
     * @returns {string | undefined}
     */
    closest(query, min_score) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.fuzzyindex_closest(retptr, this.__wbg_ptr, ptr0, len0, !isLikeNone(min_score), isLikeNone(min_score) ? 0 : min_score);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v2;
            if (r0 !== 0) {
                v2 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
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
     */
    destroy() {
        wasm.fuzzyindex_destroy(this.__wbg_ptr);
    }
    /**
     * Create a new FuzzyIndex from an array of strings.
     * @param {string[]} items
     */
    constructor(items) {
        const ptr0 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fuzzyindex_new(ptr0, len0);
        this.__wbg_ptr = ret >>> 0;
        FuzzyIndexFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance. Returns false if out of bounds.
     * @param {number} index
     * @returns {boolean}
     */
    remove(index) {
        const ret = wasm.fuzzyindex_remove(this.__wbg_ptr, index);
        return ret !== 0;
    }
    /**
     * Search the index for items matching the query.
     *
     * Returns matches sorted by score (best match first) as a JS Array.
     * @param {string} query
     * @param {SearchOptions | null} [options]
     * @returns {any}
     */
    search(query, options) {
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fuzzyindex_search(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Search the index, returning only indices and scores (no item strings).
     * @param {string} query
     * @param {SearchOptions | null} [options]
     * @returns {any}
     */
    searchIndices(query, options) {
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fuzzyindex_searchIndices(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
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
        ptr = ptr >>> 0;
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
     * Add a single item to the index.
     *
     * `key_values` must be a JS Array of strings with one value per key.
     * @param {any} key_values
     */
    add(key_values) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.keyedfuzzyindex_add(retptr, this.__wbg_ptr, addHeapObject(key_values));
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
     * Add multiple items to the index at once.
     *
     * `items_key_values` is a JS Array where each element is an Array of strings
     * (one per key). Throws if any element has the wrong number of key values.
     * @param {any} items_key_values
     */
    addMany(items_key_values) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.keyedfuzzyindex_addMany(retptr, this.__wbg_ptr, addHeapObject(items_key_values));
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
     * @param {string} query
     * @param {number | null} [min_score]
     * @returns {number | undefined}
     */
    closest(query, min_score) {
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.keyedfuzzyindex_closest(this.__wbg_ptr, ptr0, len0, !isLikeNone(min_score), isLikeNone(min_score) ? 0 : min_score);
        return ret === 0x100000001 ? undefined : ret;
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
     */
    destroy() {
        wasm.keyedfuzzyindex_destroy(this.__wbg_ptr);
    }
    /**
     * Create a new KeyedFuzzyIndex.
     *
     * `key_texts` is a JS Array of Arrays of strings (one inner array per key,
     * each inner array has one string per item).
     * `weights` is a JS Array of numbers.
     * @param {any} key_texts
     * @param {Float64Array} weights
     */
    constructor(key_texts, weights) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArrayF64ToWasm0(weights, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.keyedfuzzyindex_new(retptr, addHeapObject(key_texts), ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            KeyedFuzzyIndexFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance. Returns false if out of bounds.
     * @param {number} index
     * @returns {boolean}
     */
    remove(index) {
        const ret = wasm.keyedfuzzyindex_remove(this.__wbg_ptr, index);
        return ret !== 0;
    }
    /**
     * Search the index for items matching the query.
     *
     * Returns results sorted by combined weighted score as a JS Array.
     * @param {string} query
     * @param {SearchOptions | null} [options]
     * @returns {any}
     */
    search(query, options) {
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.keyedfuzzyindex_search(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
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
 * @param {string} query
 * @param {string[]} items
 * @param {number | null} [min_score]
 * @returns {string | undefined}
 */
export function closest(query, items, min_score) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.closest(retptr, ptr0, len0, ptr1, len1, !isLikeNone(min_score), isLikeNone(min_score) ? 0 : min_score);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        let v3;
        if (r0 !== 0) {
            v3 = getStringFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
        }
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Uint32Array}
 */
export function damerauLevenshteinBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.damerauLevenshteinBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [max_distance]
 * @returns {Uint32Array}
 */
export function damerauLevenshteinMany(reference, candidates, max_distance) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.damerauLevenshteinMany(retptr, ptr0, len0, ptr1, len1, isLikeNone(max_distance) ? 0x100000001 : (max_distance) >>> 0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} a
 * @param {string} b
 * @returns {any}
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
 * @param {any} pairs
 * @returns {any}
 */
export function hammingBatch(pairs) {
    const ret = wasm.hammingBatch(addHeapObject(pairs));
    return takeObject(ret);
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [max_distance]
 * @returns {any}
 */
export function hammingMany(reference, candidates, max_distance) {
    const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.hammingMany(ptr0, len0, ptr1, len1, isLikeNone(max_distance) ? 0x100000001 : (max_distance) >>> 0);
    return takeObject(ret);
}

/**
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
 * @param {any} pairs
 * @returns {Uint32Array}
 */
export function indelBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.indelBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [max_distance]
 * @returns {Uint32Array}
 */
export function indelMany(reference, candidates, max_distance) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.indelMany(retptr, ptr0, len0, ptr1, len1, isLikeNone(max_distance) ? 0x100000001 : (max_distance) >>> 0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function jaroBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.jaroBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function jaroMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.jaroMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function jaroWinklerBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.jaroWinklerBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function jaroWinklerMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.jaroWinklerMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Uint32Array}
 */
export function levenshteinBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.levenshteinBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [max_distance]
 * @returns {Uint32Array}
 */
export function levenshteinMany(reference, candidates, max_distance) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.levenshteinMany(retptr, ptr0, len0, ptr1, len1, isLikeNone(max_distance) ? 0x100000001 : (max_distance) >>> 0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} a
 * @param {string} b
 * @returns {any}
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
 * @param {any} pairs
 * @returns {any}
 */
export function normalizedHammingBatch(pairs) {
    const ret = wasm.normalizedHammingBatch(addHeapObject(pairs));
    return takeObject(ret);
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {any}
 */
export function normalizedHammingMany(reference, candidates, score_cutoff) {
    const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.normalizedHammingMany(ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
    return takeObject(ret);
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function normalizedIndelBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.normalizedIndelBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function normalizedIndelMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.normalizedIndelMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function normalizedLevenshteinBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.normalizedLevenshteinBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function normalizedLevenshteinMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.normalizedLevenshteinMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function partialRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.partialRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function partialRatioMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.partialRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
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
 * @param {string} query
 * @param {string[]} items
 * @param {SearchOptions | null} [options]
 * @returns {any}
 */
export function search(query, items, options) {
    const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.search(ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Perform fuzzy search across multiple text keys with weights.
 *
 * `key_texts` is a JS Array of Arrays of strings (one inner array per key,
 * each inner array has one string per item).
 * `weights` is a JS Array of numbers specifying the relative importance of each key.
 *
 * Returns results sorted by combined weighted score as a JS Array.
 * @param {string} query
 * @param {any} key_texts
 * @param {Float64Array} weights
 * @param {SearchOptions | null} [options]
 * @returns {any}
 */
export function searchKeys(query, key_texts, weights, options) {
    const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(weights, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.searchKeys(ptr0, len0, addHeapObject(key_texts), ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function sorensenDiceBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.sorensenDiceBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function sorensenDiceMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.sorensenDiceMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function tokenSetRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.tokenSetRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function tokenSetRatioMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.tokenSetRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function tokenSortRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.tokenSortRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function tokenSortRatioMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.tokenSortRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
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
 * @param {any} pairs
 * @returns {Float64Array}
 */
export function weightedRatioBatch(pairs) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.weightedRatioBatch(retptr, addHeapObject(pairs));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} reference
 * @param {string[]} candidates
 * @param {number | null} [score_cutoff]
 * @returns {Float64Array}
 */
export function weightedRatioMany(reference, candidates, score_cutoff) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(reference, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.weightedRatioMany(retptr, ptr0, len0, ptr1, len1, !isLikeNone(score_cutoff), isLikeNone(score_cutoff) ? 0 : score_cutoff);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayF64FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 8, 8);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
export function __wbg_Error_83742b46f01ce22d(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
}
export function __wbg_Number_a5a435bd7bbec835(arg0) {
    const ret = Number(getObject(arg0));
    return ret;
}
export function __wbg_String_8564e559799eccda(arg0, arg1) {
    const ret = String(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_boolean_get_c0f3f60bac5a78d1(arg0) {
    const v = getObject(arg0);
    const ret = typeof(v) === 'boolean' ? v : undefined;
    return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
}
export function __wbg___wbindgen_debug_string_5398f5bb970e0daa(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_in_41dbb8413020e076(arg0, arg1) {
    const ret = getObject(arg0) in getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_is_function_3c846841762788c1(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
}
export function __wbg___wbindgen_is_object_781bc9f159099513(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
}
export function __wbg___wbindgen_is_undefined_52709e72fb9f179c(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
}
export function __wbg___wbindgen_jsval_loose_eq_5bcc3bed3c69e72b(arg0, arg1) {
    const ret = getObject(arg0) == getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_number_get_34bb9d9dcfa21373(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
}
export function __wbg___wbindgen_string_get_395e606bd0ee4427(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_throw_6ddd609b62940d55(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
}
export function __wbg_call_e133b57c9155d22c() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_done_08ce71ee07e3bd17(arg0) {
    const ret = getObject(arg0).done;
    return ret;
}
export function __wbg_get_326e41e095fb2575() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_get_unchecked_329cfe50afab7352(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
}
export function __wbg_get_with_ref_key_6412cf3094599694(arg0, arg1) {
    const ret = getObject(arg0)[getObject(arg1)];
    return addHeapObject(ret);
}
export function __wbg_instanceof_ArrayBuffer_101e2bf31071a9f6(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_Uint8Array_740438561a5b956d(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_isArray_33b91feb269ff46e(arg0) {
    const ret = Array.isArray(getObject(arg0));
    return ret;
}
export function __wbg_isSafeInteger_ecd6a7f9c3e053cd(arg0) {
    const ret = Number.isSafeInteger(getObject(arg0));
    return ret;
}
export function __wbg_iterator_d8f549ec8fb061b1() {
    const ret = Symbol.iterator;
    return addHeapObject(ret);
}
export function __wbg_length_b3416cf66a5452c8(arg0) {
    const ret = getObject(arg0).length;
    return ret;
}
export function __wbg_length_ea16607d7b61445b(arg0) {
    const ret = getObject(arg0).length;
    return ret;
}
export function __wbg_new_5f486cdf45a04d78(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_new_a70fbab9066b301f() {
    const ret = new Array();
    return addHeapObject(ret);
}
export function __wbg_new_ab79df5bd7c26067() {
    const ret = new Object();
    return addHeapObject(ret);
}
export function __wbg_next_11b99ee6237339e3() { return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments); }
export function __wbg_next_e01a967809d1aa68(arg0) {
    const ret = getObject(arg0).next;
    return addHeapObject(ret);
}
export function __wbg_prototypesetcall_d62e5099504357e6(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
}
export function __wbg_set_282384002438957f(arg0, arg1, arg2) {
    getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
}
export function __wbg_set_6be42768c690e380(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
}
export function __wbg_value_21fc78aab0322612(arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000001(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000002(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
}
export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
}
export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
}
const FuzzyIndexFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fuzzyindex_free(ptr >>> 0, 1));
const KeyedFuzzyIndexFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_keyedfuzzyindex_free(ptr >>> 0, 1));

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
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
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


let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}
