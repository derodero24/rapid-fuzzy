/* tslint:disable */
/* eslint-disable */
/**
 * Options for search functions.
 */
export interface SearchOptions {
    maxResults: number | undefined;
    minScore: number | undefined;
    includePositions: boolean | undefined;
    isCaseSensitive: boolean | undefined;
    returnAllOnEmpty: boolean | undefined;
}


/**
 * A persistent fuzzy search index backed by Rust-side data.
 *
 * Holds items in memory on the Rust side, avoiding repeated FFI overhead
 * for applications that search the same dataset multiple times.
 */
export class FuzzyIndex {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a single item to the index.
     */
    add(item: string): void;
    /**
     * Add multiple items to the index at once.
     */
    addMany(items: string[]): void;
    /**
     * Find the closest matching string in the index.
     *
     * Returns the best match, or null if no match is found.
     */
    closest(query: string, min_score?: number | null): string | undefined;
    /**
     * Reconstruct a FuzzyIndex from a previously serialized Uint8Array.
     */
    static deserialize(data: Uint8Array): FuzzyIndex;
    /**
     * Free the internal data. After calling this, the index is empty.
     */
    destroy(): void;
    /**
     * Create a new FuzzyIndex from an array of strings.
     */
    constructor(items: string[]);
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance. Returns false if out of bounds.
     */
    remove(index: number): boolean;
    /**
     * Search the index for items matching the query.
     *
     * Returns matches sorted by score (best match first) as a JS Array.
     */
    search(query: string, options?: SearchOptions | null): any;
    /**
     * Search the index, returning only indices and scores (no item strings).
     */
    searchIndices(query: string, options?: SearchOptions | null): any;
    /**
     * Serialize the index to a compact binary format (Uint8Array).
     */
    serialize(): Uint8Array;
    /**
     * Return the number of items in the index.
     */
    readonly size: number;
}

/**
 * A persistent multi-key fuzzy search index backed by Rust-side data.
 *
 * Holds key text arrays and weights in memory on the Rust side.
 */
export class KeyedFuzzyIndex {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a single item to the index.
     *
     * `key_values` must be a JS Array of strings with one value per key.
     */
    add(key_values: any): void;
    /**
     * Add multiple items to the index at once.
     *
     * `items_key_values` is a JS Array where each element is an Array of strings
     * (one per key). Throws if any element has the wrong number of key values.
     */
    addMany(items_key_values: any): void;
    /**
     * Find the index of the closest matching item.
     */
    closest(query: string, min_score?: number | null): number | undefined;
    /**
     * Reconstruct a KeyedFuzzyIndex from a previously serialized Uint8Array.
     */
    static deserialize(data: Uint8Array): KeyedFuzzyIndex;
    /**
     * Free the internal data. After calling this, the index is empty.
     */
    destroy(): void;
    /**
     * Create a new KeyedFuzzyIndex.
     *
     * `key_texts` is a JS Array of Arrays of strings (one inner array per key,
     * each inner array has one string per item).
     * `weights` is a JS Array of numbers.
     */
    constructor(key_texts: any, weights: Float64Array);
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance. Returns false if out of bounds.
     */
    remove(index: number): boolean;
    /**
     * Search the index for items matching the query.
     *
     * Returns results sorted by combined weighted score as a JS Array.
     */
    search(query: string, options?: SearchOptions | null): any;
    /**
     * Serialize the index to a compact binary format (Uint8Array).
     */
    serialize(): Uint8Array;
    /**
     * Return the number of items in the index.
     */
    readonly size: number;
}

/**
 * Find the closest matching string from a list.
 *
 * Returns the best match, or null if no match is found.
 */
export function closest(query: string, items: string[], min_score?: number | null): string | undefined;

export function damerauLevenshtein(a: string, b: string): number;

export function damerauLevenshteinBatch(pairs: any): Uint32Array;

export function damerauLevenshteinMany(reference: string, candidates: string[], max_distance?: number | null): Uint32Array;

export function hamming(a: string, b: string): any;

export function hammingBatch(pairs: any): any;

export function hammingMany(reference: string, candidates: string[], max_distance?: number | null): any;

export function indel(a: string, b: string): number;

export function indelBatch(pairs: any): Uint32Array;

export function indelMany(reference: string, candidates: string[], max_distance?: number | null): Uint32Array;

export function jaro(a: string, b: string): number;

export function jaroBatch(pairs: any): Float64Array;

export function jaroMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

export function jaroWinkler(a: string, b: string): number;

export function jaroWinklerBatch(pairs: any): Float64Array;

export function jaroWinklerMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

export function levenshtein(a: string, b: string): number;

export function levenshteinBatch(pairs: any): Uint32Array;

export function levenshteinMany(reference: string, candidates: string[], max_distance?: number | null): Uint32Array;

export function normalizedHamming(a: string, b: string): any;

export function normalizedHammingBatch(pairs: any): any;

export function normalizedHammingMany(reference: string, candidates: string[], score_cutoff?: number | null): any;

export function normalizedIndel(a: string, b: string): number;

export function normalizedIndelBatch(pairs: any): Float64Array;

export function normalizedIndelMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

export function normalizedLevenshtein(a: string, b: string): number;

export function normalizedLevenshteinBatch(pairs: any): Float64Array;

export function normalizedLevenshteinMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

export function partialRatio(a: string, b: string): number;

export function partialRatioBatch(pairs: any): Float64Array;

export function partialRatioMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

/**
 * Perform fuzzy search over a list of strings.
 *
 * Returns matches sorted by score (best match first).
 * Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
 */
export function search(query: string, items: string[], options?: SearchOptions | null): any;

/**
 * Perform fuzzy search across multiple text keys with weights.
 *
 * `key_texts` is a JS Array of Arrays of strings (one inner array per key,
 * each inner array has one string per item).
 * `weights` is a JS Array of numbers specifying the relative importance of each key.
 *
 * Returns results sorted by combined weighted score as a JS Array.
 */
export function searchKeys(query: string, key_texts: any, weights: Float64Array, options?: SearchOptions | null): any;

export function sorensenDice(a: string, b: string): number;

export function sorensenDiceBatch(pairs: any): Float64Array;

export function sorensenDiceMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

export function tokenSetRatio(a: string, b: string): number;

export function tokenSetRatioBatch(pairs: any): Float64Array;

export function tokenSetRatioMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

export function tokenSortRatio(a: string, b: string): number;

export function tokenSortRatioBatch(pairs: any): Float64Array;

export function tokenSortRatioMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;

export function weightedRatio(a: string, b: string): number;

export function weightedRatioBatch(pairs: any): Float64Array;

export function weightedRatioMany(reference: string, candidates: string[], score_cutoff?: number | null): Float64Array;
