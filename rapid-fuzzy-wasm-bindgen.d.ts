/* tslint:disable */
/* eslint-disable */
/**
 * A lightweight search result containing only index and score (no item string).
 */
export interface IndexSearchResult {
    /**
     * The index of the item in the original input array.
     */
    index: number;
    /**
     * The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
     */
    score: number;
    /**
     * Indices of matched characters in the item string.
     * Empty unless `includePositions` is set to true in SearchOptions.
     */
    positions: number[];
    /**
     * How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
     * Only present when `includePositions` is set to true in SearchOptions.
     */
    matchType?: MatchType;
}

/**
 * A single fuzzy search result with the matched item and its score.
 */
export interface SearchResult {
    /**
     * The original string that matched.
     */
    item: string;
    /**
     * The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
     */
    score: number;
    /**
     * The index of the item in the original input array.
     */
    index: number;
    /**
     * Indices of matched characters in the item string.
     * Empty unless `includePositions` is set to true in SearchOptions.
     */
    positions: number[];
    /**
     * How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
     * Only present when `includePositions` is set to true in SearchOptions.
     */
    matchType?: MatchType;
}

/**
 * A single result from multi-key fuzzy search.
 */
export interface KeySearchResult {
    /**
     * The index of the item in the original input array.
     */
    index: number;
    /**
     * The combined weighted score normalized to 0.0-1.0 range.
     */
    score: number;
    /**
     * Per-key scores in the same order as the input keys.
     * A score of 0.0 means the item did not match on that key.
     */
    keyScores: number[];
}

/**
 * Classification of how a query matched an item.
 *
 * Derived from the matched character positions:
 * - **Exact**: all positions consecutive from index 0, covering every character in the item.
 * - **Prefix**: all positions consecutive from index 0, but the item is longer.
 * - **Contains**: all positions consecutive (a substring match), not starting at 0.
 * - **Fuzzy**: positions have gaps (character-level fuzzy match).
 */
export type MatchType = "Exact" | "Prefix" | "Contains" | "Fuzzy";

/**
 * Options for search functions.
 */
export interface SearchOptions {
    /**
     * Maximum number of results to return.
     */
    maxResults?: number;
    /**
     * Minimum normalized score (0.0-1.0) to include in results.
     */
    minScore?: number;
    /**
     * If true, include matched character positions in results.
     */
    includePositions?: boolean;
    /**
     * If true, matching is case-sensitive. Default is smart case
     * (case-insensitive unless the query contains uppercase characters).
     */
    isCaseSensitive?: boolean;
    /**
     * If true, return all items when the query is empty (or whitespace-only).
     * Useful for filter-as-you-type UIs where the full list should appear
     * before the user starts typing. Default is false.
     */
    returnAllOnEmpty?: boolean;
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
     * Add multiple items to the index at once.
     */
    addMany(items: string[]): void;
    /**
     * Add a single item to the index.
     */
    add(item: string): void;
    /**
     * Find the closest matching string in the index.
     *
     * Returns the best match, or null if no match is found.
     * If `minScore` is provided, returns null when the best match scores below the threshold.
     */
    closest(query: string, minScore?: number | null): string | null;
    /**
     * Reconstruct a FuzzyIndex from a previously serialized Uint8Array.
     */
    static deserialize(data: Uint8Array): FuzzyIndex;
    /**
     * Free the internal data. After calling this, the index is empty.
     */
    destroy(): void;
    /**
     * Create a new FuzzyIndex, returning a Promise (parity with the Node.js binding).
     *
     * The WebAssembly build has no worker thread, so the index is built
     * synchronously on the calling thread and the returned Promise is already
     * resolved. Prefer the constructor when you do not need a Promise.
     */
    static fromAsync(items: string[]): Promise<FuzzyIndex>;
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
     * Search the index, returning only indices and scores (no item strings).
     *
     * The second argument accepts either a number (maxResults) or a SearchOptions object.
     */
    searchIndices(query: string, options?: number | SearchOptions | null): IndexSearchResult[];
    /**
     * Search the index for items matching the query.
     *
     * Returns matches sorted by score (best match first).
     * The second argument accepts either a number (maxResults) or a SearchOptions object.
     */
    search(query: string, options?: number | SearchOptions | null): SearchResult[];
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
     * Add multiple items to the index at once.
     *
     * Each element of `itemsKeyValues` is an array of key values for one item.
     * Throws if any element has the wrong number of key values.
     */
    addMany(itemsKeyValues: string[][]): void;
    /**
     * Add a single item to the index.
     *
     * `keyValues` must have the same length as the number of keys.
     * Throws if the length does not match.
     */
    add(keyValues: string[]): void;
    /**
     * Find the index of the closest matching item.
     *
     * Returns the index of the best match, or null if no match is found.
     * If `minScore` is provided, returns null when the best match scores below the threshold.
     */
    closest(query: string, minScore?: number | null): number | null;
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
     * `keyTexts[k]` is an array of strings for key `k`, one per item.
     * All inner arrays must have the same length (the number of items).
     * `weights` holds one finite, non-negative weight per key.
     */
    constructor(keyTexts: string[][], weights: ArrayLike<number>);
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance. Returns false if out of bounds.
     */
    remove(index: number): boolean;
    /**
     * Search the index for items matching the query.
     *
     * Returns results sorted by combined weighted score (best match first).
     */
    search(query: string, options?: SearchOptions | null): KeySearchResult[];
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
 * If `minScore` is provided, returns null when the best match scores below the threshold.
 */
export function closest(query: string, items: string[], minScore?: number | null): string | null;

/**
 * Compute the Damerau-Levenshtein distance between two strings.
 *
 * Like Levenshtein, but also considers transpositions of two adjacent
 * characters as a single edit.
 */
export function damerauLevenshtein(a: string, b: string): number;

/**
 * Compute the Damerau-Levenshtein distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 */
export function damerauLevenshteinBatch(pairs: string[][]): Uint32Array;

/**
 * Compute the Damerau-Levenshtein distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1` (enabling early termination for better performance).
 */
export function damerauLevenshteinMany(reference: string, candidates: string[], maxDistance?: number | null): Uint32Array;

/**
 * Compute the Hamming distance between two strings.
 *
 * The Hamming distance counts the number of positions at which the corresponding
 * characters differ. It is only defined for strings of equal length.
 * Returns `null` if the strings have different lengths.
 */
export function hamming(a: string, b: string): number | null;

/**
 * Compute the Hamming distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 * Returns `null` for pairs with different lengths.
 */
export function hammingBatch(pairs: string[][]): (number | null)[];

/**
 * Compute the Hamming distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * Returns `null` for candidates with a different length than the reference.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will also return `null` (enabling early termination for better performance).
 */
export function hammingMany(reference: string, candidates: string[], maxDistance?: number | null): (number | null)[];

/**
 * Compute the Indel distance between two strings.
 *
 * The Indel distance counts the minimum number of insertions and deletions
 * (no substitutions) required to transform one string into the other.
 * It equals `len(a) + len(b) - 2 * LCS_length(a, b)`.
 *
 * Useful when substitutions are semantically two operations (one deletion +
 * one insertion), such as in DNA sequence alignment.
 */
export function indel(a: string, b: string): number;

/**
 * Compute the Indel distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 */
export function indelBatch(pairs: string[][]): Uint32Array;

/**
 * Compute the Indel distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1` (enabling early termination for better performance).
 */
export function indelMany(reference: string, candidates: string[], maxDistance?: number | null): Uint32Array;

/**
 * Compute the Jaro similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 */
export function jaro(a: string, b: string): number;

/**
 * Compute the Jaro similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function jaroBatch(pairs: string[][]): Float64Array;

/**
 * Compute the Jaro similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function jaroMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Compute the Jaro-Winkler similarity between two strings.
 *
 * A modification of Jaro that gives more weight to common prefixes.
 * Returns a value between 0.0 and 1.0.
 */
export function jaroWinkler(a: string, b: string): number;

/**
 * Compute the Jaro-Winkler similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function jaroWinklerBatch(pairs: string[][]): Float64Array;

/**
 * Compute the Jaro-Winkler similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function jaroWinklerMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Compute the Levenshtein distance between two strings.
 *
 * The Levenshtein distance is the minimum number of single-character edits
 * (insertions, deletions, or substitutions) required to change one string
 * into the other.
 */
export function levenshtein(a: string, b: string): number;

/**
 * Compute the Levenshtein distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 */
export function levenshteinBatch(pairs: string[][]): Uint32Array;

/**
 * Compute the Levenshtein distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1` (enabling early termination for better performance).
 */
export function levenshteinMany(reference: string, candidates: string[], maxDistance?: number | null): Uint32Array;

/**
 * Compute the normalized Hamming similarity between two strings.
 *
 * Returns `null` if the strings have different lengths.
 * Returns a value between 0.0 (no matching characters) and 1.0 (identical).
 */
export function normalizedHamming(a: string, b: string): number | null;

/**
 * Compute the normalized Hamming similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of scores in the same order as the input pairs.
 * Returns `null` for pairs with different lengths.
 */
export function normalizedHammingBatch(pairs: string[][]): (number | null)[];

/**
 * Compute the normalized Hamming similarity from one reference string to many candidates.
 *
 * Returns an array of scores, one per candidate, in the same order as the input.
 * Returns `null` for candidates with a different length than the reference.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will also return `null` (enabling early termination for better performance).
 */
export function normalizedHammingMany(reference: string, candidates: string[], minSimilarity?: number | null): (number | null)[];

/**
 * Compute the normalized Indel similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 */
export function normalizedIndel(a: string, b: string): number;

/**
 * Compute the normalized Indel similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function normalizedIndelBatch(pairs: string[][]): Float64Array;

/**
 * Compute the normalized Indel similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function normalizedIndelMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Compute the normalized Levenshtein similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 */
export function normalizedLevenshtein(a: string, b: string): number;

/**
 * Compute the normalized Levenshtein similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function normalizedLevenshteinBatch(pairs: string[][]): Float64Array;

/**
 * Compute the normalized Levenshtein similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function normalizedLevenshteinMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Compute the partial ratio between two strings.
 *
 * Finds the best matching substring of the shorter string within the longer string
 * using a sliding window approach. Returns the highest normalized Levenshtein
 * similarity across all windows. Useful for matching when one string is a
 * substring or abbreviation of the other. Returns a value between 0.0 and 1.0.
 */
export function partialRatio(a: string, b: string): number;

/**
 * Compute the partial ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function partialRatioBatch(pairs: string[][]): Float64Array;

/**
 * Compute the partial ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function partialRatioMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Perform fuzzy search over a list of strings.
 *
 * Returns matches sorted by score (best match first).
 * Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
 *
 * The third argument accepts either a number (maxResults for backward
 * compatibility) or a SearchOptions object.
 */
export function search(query: string, items: string[], options?: number | SearchOptions | null): SearchResult[];

/**
 * Perform fuzzy search across multiple text keys with weights.
 *
 * `keyTexts[k]` is an array of strings for key `k`, one per item.
 * `weights` specifies the relative importance of each key.
 * `options` is a `SearchOptions` object or a number (maxResults).
 *
 * Returns results sorted by combined weighted score (best match first),
 * exactly like `KeyedFuzzyIndex.search` on the same key texts and weights.
 * Throws an `Error` for invalid input (key texts of different lengths, a
 * weight count that differs from the key count, negative, NaN or infinite
 * weights, or weights summing to 0 or Infinity), like the `KeyedFuzzyIndex`
 * constructor.
 */
export function searchKeys(query: string, keyTexts: string[][], weights: ArrayLike<number>, options?: number | SearchOptions | null): KeySearchResult[];

/**
 * Compute the Sorensen-Dice coefficient between two strings.
 *
 * Uses bigrams (pairs of consecutive characters) to measure similarity.
 * Returns a value between 0.0 and 1.0.
 */
export function sorensenDice(a: string, b: string): number;

/**
 * Compute the Sorensen-Dice coefficient for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function sorensenDiceBatch(pairs: string[][]): Float64Array;

/**
 * Compute the Sorensen-Dice coefficient from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 * Reference bigrams are pre-computed once and reused for all candidates.
 */
export function sorensenDiceMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Compute the token set ratio between two strings.
 *
 * Compares the intersection and differences of token sets from both strings.
 * Returns the maximum similarity among comparisons of the intersection with
 * each remainder. Highly effective for strings with shared tokens but
 * different lengths. Returns a value between 0.0 and 1.0.
 */
export function tokenSetRatio(a: string, b: string): number;

/**
 * Compute the token set ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function tokenSetRatioBatch(pairs: string[][]): Float64Array;

/**
 * Compute the token set ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function tokenSetRatioMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Compute the token sort ratio between two strings.
 *
 * Splits both strings into tokens, sorts them alphabetically, then computes
 * the normalized Levenshtein similarity. This makes the comparison
 * order-independent, ideal for matching names or addresses where word order varies.
 * Returns a value between 0.0 (completely different) and 1.0 (identical after sorting).
 */
export function tokenSortRatio(a: string, b: string): number;

/**
 * Compute the token sort ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function tokenSortRatioBatch(pairs: string[][]): Float64Array;

/**
 * Compute the token sort ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function tokenSortRatioMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;

/**
 * Compute the weighted ratio between two strings.
 *
 * Returns the maximum score across normalized Levenshtein, token sort ratio,
 * token set ratio, and partial ratio. This provides a single "best effort"
 * similarity score that automatically selects the most appropriate algorithm.
 * Returns a value between 0.0 and 1.0.
 */
export function weightedRatio(a: string, b: string): number;

/**
 * Compute the weighted ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function weightedRatioBatch(pairs: string[][]): Float64Array;

/**
 * Compute the weighted ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function weightedRatioMany(reference: string, candidates: string[], minSimilarity?: number | null): Float64Array;
