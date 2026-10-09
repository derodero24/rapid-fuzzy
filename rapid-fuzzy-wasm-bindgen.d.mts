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
     * Indices of the matched characters of the item, in ascending order.
     * An ASCII item is counted by character; any other item by grapheme
     * cluster (a user-perceived character, such as an emoji with its
     * modifiers or a letter with its combining marks), so these are not
     * UTF-16 string offsets. `highlight()` and `highlightRanges()` convert
     * them. Empty unless `includePositions` is set to true in SearchOptions.
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
     * Indices of the matched characters of the item, in ascending order.
     * An ASCII item is counted by character; any other item by grapheme
     * cluster (a user-perceived character, such as an emoji with its
     * modifiers or a letter with its combining marks), so these are not
     * UTF-16 string offsets. `highlight()` and `highlightRanges()` convert
     * them. Empty unless `includePositions` is set to true in SearchOptions.
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
     * The combined score (0.0-1.0) of the key scores, as set by
     * `scoreMode` (by default the weighted mean over all keys).
     */
    score: number;
    /**
     * Per-key scores in the same order as the input keys: how well each
     * key matches the query (with `matchMode: "crossKey"`, the share of the
     * query it matches). A score of 0.0 means the item did not match on
     * that key.
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
 * How multi-key search combines the per-key scores (`keyScores`) of an item
 * into its `score` (see `KeySearchOptions.scoreMode`).
 */
export type KeyScoreMode = "weighted" | "matched" | "max";

/**
 * How multi-key search matches the query against the keys of an item (see
 * `KeySearchOptions.matchMode`).
 */
export type KeyMatchMode = "perKey" | "crossKey";

/**
 * Options for multi-key search: `searchKeys()`, `KeyedFuzzyIndex.search()`
 * and the object search built on them (`searchObjects()`,
 * `FuzzyObjectIndex.search()`). The `SearchOptions` fields, plus
 * `scoreMode` and `matchMode`.
 */
export interface KeySearchOptions {
    /**
     * Maximum number of results to return: a non-negative integer, or
     * `Infinity` for no limit. NaN, negative and fractional values throw.
     */
    maxResults?: number;
    /**
     * Minimum combined score (0.0-1.0, see `scoreMode`) to include in
     * results.
     */
    minScore?: number;
    /**
     * Accepted for compatibility with `SearchOptions`, but has no effect:
     * multi-key results have no match positions.
     */
    includePositions?: boolean;
    /**
     * If true, matching is case-sensitive. When false or omitted, matching
     * is smart case: case-insensitive while the query is all lower-case, and
     * case-sensitive once it contains an upper-case letter. `false` does not
     * force case-insensitive matching; lower-case the query for that.
     */
    isCaseSensitive?: boolean;
    /**
     * If true, return all items when the query has no search term: empty,
     * whitespace-only, or only query syntax such as `^` or `!`. Every item
     * then scores 1, in every `scoreMode`. Default is false.
     */
    returnAllOnEmpty?: boolean;
    /**
     * How the per-key scores (`keyScores`) of an item are combined into its
     * `score`. Only keys with a positive weight take part:
     *
     * - `"weighted"` (default): the weighted mean over all keys,
     *   `sum(weight * keyScore) / sum(weight)`. A key that does not match
     *   counts as 0, so an exact match on one key out of several scores only
     *   that key's share of the total weight.
     * - `"matched"`: the weighted mean over the keys that match
     *   (`keyScore > 0`) only. An item whose only matching key matches
     *   exactly scores 1.
     * - `"max"`: the highest score of any key. Weights then only select the
     *   keys that take part (weight > 0).
     *
     * `keyScores` are the same in every mode; `minScore` and `maxResults`
     * apply to the combined score. Equal scores are ordered by the length of
     * the best-matching key's text (the key contributing most to the score:
     * highest `weight * keyScore`, or highest `keyScore` in `"max"` mode;
     * the first one on a tie), then by index. Any other value throws a
     * `TypeError`.
     *
     * With `matchMode: "crossKey"`, `"matched"` counts the weight of each
     * key in proportion to the share of the query it matches, and `"max"`
     * scores each term by the key it matches best (see `matchMode`).
     */
    scoreMode?: KeyScoreMode;
    /**
     * How the query is matched against the keys of an item:
     *
     * - `"perKey"` (default): every key is matched against the whole query,
     *   like `search()` matches an item. A key scores 0 unless it matches
     *   every term of the query and none of its `!term` exclusions.
     * - `"crossKey"`: every term is matched against the keys on its own, so
     *   the terms may match different keys: `"john tokyo"` finds an item
     *   whose name is "John Smith" and whose city is "Tokyo". Every term
     *   must match at least one key, and a `!term` matching any key
     *   excludes the item. A key's score (`keyScores`) is the share of the
     *   query it matches: the scores of the terms it matches, each counting
     *   in proportion to the score of a perfect match of the term (which
     *   grows with its length), so a key matching every term perfectly
     *   scores 1. `scoreMode` combines these key scores: `"weighted"` as
     *   `sum(weight * keyScore) / sum(weight)`, as in `"perKey"` mode;
     *   `"matched"` as `sum(weight * keyScore) / sum(weight * coverage)`,
     *   where a key's coverage is the share of the query made up by the
     *   terms it matches (in `"perKey"` mode, 1 for a key that matches and
     *   0 otherwise); `"max"` by taking each term's score on the key it
     *   matches best. In `"matched"` and `"max"` mode, an item whose every
     *   term matches some key perfectly scores 1.
     *
     * Only keys with a positive weight take part in either mode; keys whose
     * weight is 0 still get `keyScores`. For a query of a single term
     * without exclusions, both modes return the same results. Any other
     * value throws a `TypeError`.
     */
    matchMode?: KeyMatchMode;
}

/**
 * Options for search functions.
 */
export interface SearchOptions {
    /**
     * Maximum number of results to return: a non-negative integer, or
     * `Infinity` for no limit. NaN, negative and fractional values throw.
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
     * If true, matching is case-sensitive. When false or omitted, matching
     * is smart case: case-insensitive while the query is all lower-case, and
     * case-sensitive once it contains an upper-case letter. `false` does not
     * force case-insensitive matching; lower-case the query for that.
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
    addMany(items: ReadonlyArray<string>): void;
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
     *
     * The index stays usable: it behaves as an empty index (searches
     * return no results) and `add()` / `addMany()` work as before.
     */
    destroy(): void;
    /**
     * Create a new FuzzyIndex, returning a Promise (parity with the Node.js binding).
     *
     * The WebAssembly build has no worker thread, so the index is built
     * synchronously on the calling thread and the returned Promise is already
     * resolved. Prefer the constructor when you do not need a Promise.
     *
     * Invalid input (anything but an array of strings) rejects the returned
     * Promise with a `TypeError` instead of throwing synchronously.
     */
    static fromAsync(items: ReadonlyArray<string>): Promise<FuzzyIndex>;
    /**
     * Create a new FuzzyIndex from an array of strings.
     */
    constructor(items: ReadonlyArray<string>);
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
    addMany(itemsKeyValues: ReadonlyArray<ReadonlyArray<string>>): void;
    /**
     * Add a single item to the index.
     *
     * `keyValues` must have the same length as the number of keys.
     * Throws if the length does not match.
     */
    add(keyValues: ReadonlyArray<string>): void;
    /**
     * Find the index of the closest matching item.
     *
     * Returns the index of the best match, or null if no match is found.
     * If `minScore` is provided, returns null when the best match scores below the threshold.
     * `scoreMode` and `matchMode` work like the `search()` options of the
     * same names (defaults `"weighted"` and `"perKey"`): the result is the
     * first result of
     * `search(query, { maxResults: 1, minScore, scoreMode, matchMode })`.
     */
    closest(query: string, minScore?: number | null, scoreMode?: KeyScoreMode | null, matchMode?: KeyMatchMode | null): number | null;
    /**
     * Reconstruct a KeyedFuzzyIndex from a previously serialized Uint8Array.
     */
    static deserialize(data: Uint8Array): KeyedFuzzyIndex;
    /**
     * Free the internal data. After calling this, the index is empty.
     *
     * The key configuration is kept, so the index stays usable: it behaves
     * as an empty index and `add()` / `addMany()` work as before.
     */
    destroy(): void;
    /**
     * Create a new KeyedFuzzyIndex.
     *
     * `keyTexts[k]` is an array of strings for key `k`, one per item.
     * All inner arrays must have the same length (the number of items).
     * `weights` holds one finite, non-negative weight per key.
     */
    constructor(keyTexts: ReadonlyArray<ReadonlyArray<string>>, weights: ArrayLike<number>);
    /**
     * Remove the item at the given index.
     *
     * Uses swap-remove for O(1) performance. Returns false if out of bounds.
     */
    remove(index: number): boolean;
    /**
     * Search the index for items matching the query.
     *
     * Returns results sorted by combined score (best match first), exactly
     * like `searchKeys()` on the same key texts and weights. The second
     * argument accepts either a number (maxResults) or a KeySearchOptions
     * object, whose `matchMode` selects how the query is matched against
     * the keys and `scoreMode` how the per-key scores are combined.
     */
    search(query: string, options?: number | KeySearchOptions | null): KeySearchResult[];
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
export function closest(query: string, items: ReadonlyArray<string>, minScore?: number | null): string | null;

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
 */
export function damerauLevenshtein(a: string, b: string): number;

/**
 * Compute the Damerau-Levenshtein distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 */
export function damerauLevenshteinBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Uint32Array;

/**
 * Compute the Damerau-Levenshtein distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1` (enabling early termination for better performance).
 */
export function damerauLevenshteinMany(reference: string, candidates: ReadonlyArray<string>, maxDistance?: number | null): Uint32Array;

/**
 * Compute the Hamming distance between two strings.
 *
 * The Hamming distance counts the number of positions at which the corresponding
 * characters differ. It is only defined for strings of equal length.
 * Returns `null` if the strings have different lengths.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 */
export function hamming(a: string, b: string): number | null;

/**
 * Compute the Hamming distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 * Returns `null` for pairs with different lengths.
 */
export function hammingBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): (number | null)[];

/**
 * Compute the Hamming distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * Returns `null` for candidates with a different length than the reference.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will also return `null` (enabling early termination for better performance).
 */
export function hammingMany(reference: string, candidates: ReadonlyArray<string>, maxDistance?: number | null): (number | null)[];

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
 */
export function indel(a: string, b: string): number;

/**
 * Compute the Indel distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 */
export function indelBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Uint32Array;

/**
 * Compute the Indel distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1` (enabling early termination for better performance).
 */
export function indelMany(reference: string, candidates: ReadonlyArray<string>, maxDistance?: number | null): Uint32Array;

/**
 * Compute the Jaro similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 */
export function jaro(a: string, b: string): number;

/**
 * Compute the Jaro similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function jaroBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the Jaro similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function jaroMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

/**
 * Compute the Jaro-Winkler similarity between two strings.
 *
 * A modification of Jaro that gives more weight to common prefixes.
 * Returns a value between 0.0 and 1.0.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 */
export function jaroWinkler(a: string, b: string): number;

/**
 * Compute the Jaro-Winkler similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function jaroWinklerBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the Jaro-Winkler similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function jaroWinklerMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

/**
 * Compute the Levenshtein distance between two strings.
 *
 * The Levenshtein distance is the minimum number of single-character edits
 * (insertions, deletions, or substitutions) required to change one string
 * into the other.
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 */
export function levenshtein(a: string, b: string): number;

/**
 * Compute the Levenshtein distance for multiple pairs of strings in a single call.
 *
 * Returns an array of distances in the same order as the input pairs.
 * Each pair must be an array of exactly two strings `[a, b]`.
 */
export function levenshteinBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Uint32Array;

/**
 * Compute the Levenshtein distance from one reference string to many candidates.
 *
 * Returns an array of distances, one per candidate, in the same order as the input.
 * If `maxDistance` is provided, candidates with distance exceeding the threshold
 * will return `maxDistance + 1` (enabling early termination for better performance).
 */
export function levenshteinMany(reference: string, candidates: ReadonlyArray<string>, maxDistance?: number | null): Uint32Array;

/**
 * Compute the normalized Hamming similarity between two strings.
 *
 * Returns `null` if the strings have different lengths.
 * Returns a value between 0.0 (no matching characters) and 1.0 (identical).
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 */
export function normalizedHamming(a: string, b: string): number | null;

/**
 * Compute the normalized Hamming similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of scores in the same order as the input pairs.
 * Returns `null` for pairs with different lengths.
 */
export function normalizedHammingBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): (number | null)[];

/**
 * Compute the normalized Hamming similarity from one reference string to many candidates.
 *
 * Returns an array of scores, one per candidate, in the same order as the input.
 * Returns `null` for candidates with a different length than the reference.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will also return `null` (enabling early termination for better performance).
 */
export function normalizedHammingMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): (number | null)[];

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
 */
export function normalizedIndel(a: string, b: string): number;

/**
 * Compute the normalized Indel similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function normalizedIndelBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the normalized Indel similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function normalizedIndelMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

/**
 * Compute the normalized Levenshtein similarity between two strings.
 *
 * `1 - levenshtein(a, b) / max(length of a, length of b)`, counted in
 * characters. Returns a value between 0.0 (completely different) and 1.0
 * (identical).
 *
 * Compares the Unicode code points of the strings as given: case, whitespace
 * and Unicode normalization (NFC vs NFD) are not adjusted.
 */
export function normalizedLevenshtein(a: string, b: string): number;

/**
 * Compute the normalized Levenshtein similarity for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function normalizedLevenshteinBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the normalized Levenshtein similarity from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates with similarity below the threshold
 * will return `0.0` (enabling early termination for better performance).
 */
export function normalizedLevenshteinMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

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
 */
export function partialRatio(a: string, b: string): number;

/**
 * Compute the partial ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function partialRatioBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the partial ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function partialRatioMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

/**
 * Perform fuzzy search over a list of strings.
 *
 * Returns matches sorted by score (best match first).
 * Scores are normalized to a 0.0-1.0 range where 1.0 is a perfect match.
 *
 * The third argument accepts either a number (maxResults for backward
 * compatibility) or a SearchOptions object.
 */
export function search(query: string, items: ReadonlyArray<string>, options?: number | SearchOptions | null): SearchResult[];

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
 */
export function searchKeys(query: string, keyTexts: ReadonlyArray<ReadonlyArray<string>>, weights: ArrayLike<number>, options?: number | KeySearchOptions | null): KeySearchResult[];

/**
 * Compute the Sorensen-Dice coefficient between two strings.
 *
 * Compares the bigrams (pairs of consecutive characters) of the two strings
 * after removing all whitespace; case and Unicode normalization are not
 * adjusted. Identical strings score 1.0; otherwise a string with fewer than
 * two characters left scores 0.0. Returns a value between 0.0 and 1.0.
 */
export function sorensenDice(a: string, b: string): number;

/**
 * Compute the Sorensen-Dice coefficient for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function sorensenDiceBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the Sorensen-Dice coefficient from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 * Reference bigrams are pre-computed once and reused for all candidates.
 */
export function sorensenDiceMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

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
 */
export function tokenSetRatio(a: string, b: string): number;

/**
 * Compute the token set ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function tokenSetRatioBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the token set ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function tokenSetRatioMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

/**
 * Compute the token sort ratio between two strings.
 *
 * Lower-cases both strings, splits them on whitespace, sorts the tokens and
 * joins them with single spaces, then returns the normalized Levenshtein
 * similarity of the two results, so word order does not matter. Punctuation
 * is kept: `Smith,` and `Smith` are different tokens.
 * Returns a value between 0.0 (completely different) and 1.0 (identical after sorting).
 */
export function tokenSortRatio(a: string, b: string): number;

/**
 * Compute the token sort ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function tokenSortRatioBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the token sort ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function tokenSortRatioMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

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
 */
export function weightedRatio(a: string, b: string): number;

/**
 * Compute the weighted ratio for multiple pairs of strings in a single call.
 *
 * Returns an array of similarity scores in the same order as the input pairs.
 */
export function weightedRatioBatch(pairs: ReadonlyArray<ReadonlyArray<string>>): Float64Array;

/**
 * Compute the weighted ratio from one reference string to many candidates.
 *
 * Returns an array of similarity scores, one per candidate, in the same order as the input.
 * If `minSimilarity` is provided, candidates scoring below the threshold return `0.0`.
 */
export function weightedRatioMany(reference: string, candidates: ReadonlyArray<string>, minSimilarity?: number | null): Float64Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_fuzzyindex_free: (a: number, b: number) => void;
    readonly __wbg_keyedfuzzyindex_free: (a: number, b: number) => void;
    readonly closest: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly damerauLevenshtein: (a: number, b: number, c: number, d: number) => number;
    readonly damerauLevenshteinBatch: (a: number, b: number) => void;
    readonly damerauLevenshteinMany: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly fuzzyindex_add: (a: number, b: number, c: number) => void;
    readonly fuzzyindex_addMany: (a: number, b: number, c: number) => void;
    readonly fuzzyindex_closest: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly fuzzyindex_deserialize: (a: number, b: number, c: number) => void;
    readonly fuzzyindex_destroy: (a: number) => void;
    readonly fuzzyindex_fromAsync: (a: number) => number;
    readonly fuzzyindex_new: (a: number, b: number) => number;
    readonly fuzzyindex_remove: (a: number, b: number) => number;
    readonly fuzzyindex_search: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly fuzzyindex_searchIndices: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly fuzzyindex_serialize: (a: number, b: number) => void;
    readonly fuzzyindex_size: (a: number) => number;
    readonly hamming: (a: number, b: number, c: number, d: number) => number;
    readonly hammingBatch: (a: number, b: number) => void;
    readonly hammingMany: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly indel: (a: number, b: number, c: number, d: number) => number;
    readonly indelBatch: (a: number, b: number) => void;
    readonly indelMany: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly jaro: (a: number, b: number, c: number, d: number) => number;
    readonly jaroBatch: (a: number, b: number) => void;
    readonly jaroMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly jaroWinkler: (a: number, b: number, c: number, d: number) => number;
    readonly jaroWinklerBatch: (a: number, b: number) => void;
    readonly jaroWinklerMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly keyedfuzzyindex_add: (a: number, b: number, c: number) => void;
    readonly keyedfuzzyindex_addMany: (a: number, b: number, c: number) => void;
    readonly keyedfuzzyindex_closest: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly keyedfuzzyindex_deserialize: (a: number, b: number, c: number) => void;
    readonly keyedfuzzyindex_destroy: (a: number) => void;
    readonly keyedfuzzyindex_new: (a: number, b: number, c: number, d: number) => void;
    readonly keyedfuzzyindex_remove: (a: number, b: number) => number;
    readonly keyedfuzzyindex_search: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly keyedfuzzyindex_serialize: (a: number, b: number) => void;
    readonly keyedfuzzyindex_size: (a: number) => number;
    readonly levenshtein: (a: number, b: number, c: number, d: number) => number;
    readonly levenshteinBatch: (a: number, b: number) => void;
    readonly levenshteinMany: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly normalizedHamming: (a: number, b: number, c: number, d: number) => number;
    readonly normalizedHammingBatch: (a: number, b: number) => void;
    readonly normalizedHammingMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly normalizedIndel: (a: number, b: number, c: number, d: number) => number;
    readonly normalizedIndelBatch: (a: number, b: number) => void;
    readonly normalizedIndelMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly normalizedLevenshtein: (a: number, b: number, c: number, d: number) => number;
    readonly normalizedLevenshteinBatch: (a: number, b: number) => void;
    readonly normalizedLevenshteinMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly partialRatio: (a: number, b: number, c: number, d: number) => number;
    readonly partialRatioBatch: (a: number, b: number) => void;
    readonly partialRatioMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly search: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly searchKeys: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly sorensenDice: (a: number, b: number, c: number, d: number) => number;
    readonly sorensenDiceBatch: (a: number, b: number) => void;
    readonly sorensenDiceMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly tokenSetRatio: (a: number, b: number, c: number, d: number) => number;
    readonly tokenSetRatioBatch: (a: number, b: number) => void;
    readonly tokenSetRatioMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly tokenSortRatio: (a: number, b: number, c: number, d: number) => number;
    readonly tokenSortRatioBatch: (a: number, b: number) => void;
    readonly tokenSortRatioMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly weightedRatio: (a: number, b: number, c: number, d: number) => number;
    readonly weightedRatioBatch: (a: number, b: number) => void;
    readonly weightedRatioMany: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
