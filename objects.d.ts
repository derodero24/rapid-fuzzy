import type { SearchOptions } from './index';

type Primitive = string | number | bigint | boolean | symbol | null | undefined;
type AnyFunction = (...args: never) => unknown;
type IsAny<T> = 0 extends 1 & T ? true : false;

/**
 * Whether paths into `T` cannot be checked: `any`, `unknown`, `object`, `{}`
 * and types with a string index signature accept any key.
 */
type IsWide<T> =
  IsAny<T> extends true
    ? true
    : unknown extends T
      ? true
      : T extends Primitive | AnyFunction | ReadonlyArray<unknown>
        ? false
        : string extends keyof T
          ? true
          : [keyof T] extends [never]
            ? true
            : false;

/** The path segments usable on `T`: property names (methods excluded) or array indices. */
type SegmentOf<T> = T extends Primitive | AnyFunction
  ? never
  : T extends ReadonlyArray<unknown>
    ? `${number}`
    : {
        [K in keyof T & (string | number)]-?: NonNullable<T[K]> extends AnyFunction
          ? never
          : `${K}`;
      }[keyof T & (string | number)];

/** The (non-nullable) value reached from `T` through the segment `S`. */
type Step<T, S extends string> =
  T extends ReadonlyArray<infer E>
    ? NonNullable<E>
    : S extends keyof T
      ? NonNullable<T[S]>
      : S extends `${infer N extends number}`
        ? N extends keyof T
          ? NonNullable<T[N]>
          : never
        : never;

/**
 * `${Done}${P}` when `P` is a valid key path into `T`. Otherwise the valid
 * paths at the first invalid segment (or the valid prefix when the path runs
 * past a leaf value), so the type error lists the alternatives. `Prev` is
 * `Done` without its trailing dot. A non-literal remainder (`${string}`, as
 * produced by {@link KeyPath}) is not checked.
 */
type CheckPath<T, P extends string, Done extends string = '', Prev extends string = never> =
  IsWide<T> extends true
    ? `${Done}${P}`
    : string extends P
      ? `${Done}${P}`
      : [SegmentOf<T>] extends [never]
        ? Prev
        : P extends `${infer Head}.${infer Rest}`
          ? Head extends SegmentOf<T>
            ? CheckPath<Step<T, Head>, Rest, `${Done}${Head}.`, `${Done}${Head}`>
            : `${Done}${SegmentOf<T>}`
          : P extends SegmentOf<T>
            ? `${Done}${P}`
            : `${Done}${SegmentOf<T>}`;

/** Remaining recursion budget for {@link KeyPath}: `Depth[D]` is `D - 1`. */
type Depth = [never, 0];

/** Key paths of a non-nullable `T`, listing at most `D + 1` more segments. */
type PathsOf<T, D extends number> = [D] extends [never]
  ? string
  : IsWide<T> extends true
    ? string
    : {
        [S in SegmentOf<T>]: S | `${S}.${PathsOf<Step<T, S>, Depth[D]>}`;
      }[SegmentOf<T>];

/**
 * The dot-separated key paths of `T`, e.g. `'name' | 'address' | 'address.city'`
 * for `{ name: string; address: { city: string } }`. Use it to type key lists
 * ahead of time (`KeyConfig<T>`, `ObjectSearchOptions<T>`); key names written
 * directly in a {@link searchObjects} / {@link FuzzyObjectIndex} call are
 * checked segment by segment at any depth.
 *
 * Array elements are addressed by index (`'tags.0'`); methods are excluded.
 * To keep the union small, paths are listed two segments deep: a longer path
 * is accepted when its first two segments are valid (`'a.b.${string}'`).
 * Paths into wide types (`any`, `unknown`, `object`, types with an index
 * signature) accept any string.
 */
export type KeyPath<T> = PathsOf<NonNullable<T>, 1>;

export interface KeyConfig<T = unknown> {
  /**
   * Key path to search, e.g. `'name'` or `'address.city'` for nested values.
   *
   * Each segment reads an own property of the current value (a getter
   * defined by its class also counts); inherited methods and properties such
   * as `constructor` are never read. Numeric segments index into arrays.
   *
   * The resolved value is indexed as: strings as-is; numbers, bigints and
   * booleans via `String()`; objects with a string form of their own (`Date`,
   * a class with `toString()`) via `String()`; arrays as their elements'
   * text joined with spaces. Missing values, `null`, functions, symbols and
   * plain objects index as an empty string.
   */
  name: KeyPath<T>;
  /** Relative weight of this key in the combined score. Defaults to `1.0`. */
  weight?: number | undefined;
}

/**
 * Options for searchObjects(). Extends SearchOptions with key configuration.
 *
 * Note: `includePositions` has no effect for multi-key search — match positions
 * are per-key and are not merged, so the `positions`/`matchType` fields from
 * single-key {@link SearchOptions} are not populated. All other SearchOptions
 * fields (maxResults, minScore, isCaseSensitive, returnAllOnEmpty) apply.
 */
export interface ObjectSearchOptions<T = unknown> extends SearchOptions {
  keys: ReadonlyArray<KeyPath<T> | KeyConfig<T>>;
}

export interface ObjectSearchResult<T> {
  /** The matched object. */
  item: T;
  /**
   * Position of the matched item: its index in the `items` array passed to
   * {@link searchObjects}, or its current position in a
   * {@link FuzzyObjectIndex}. Index positions change when items are removed:
   * `remove()` moves the last item into the freed slot.
   */
  index: number;
  /** Combined weighted score normalized to the 0.0–1.0 range. */
  score: number;
  /**
   * Per-key scores in the same order as the configured keys.
   * A score of 0.0 means the item did not match on that key.
   */
  keyScores: Array<number>;
}

/** Any key entry, without checking the name against an item type. */
type KeyEntry = string | { readonly name: string; readonly weight?: number | undefined };

/**
 * The key names written in an options object, captured for checking: `S`
 * collects the plain string entries, `C` the names of the `KeyConfig` entries.
 */
interface KeyNames<S extends string, C extends string> {
  readonly keys: ReadonlyArray<S | { readonly name: C; readonly weight?: number | undefined }>;
}

/**
 * `K` when every name in it is a key path of `T`; otherwise the valid
 * alternatives, so the offending name is reported. A plain `string` is not
 * checked.
 */
type CheckedKey<T, K extends string> = string extends K ? K : CheckPath<NonNullable<T>, K>;

/** Excludes `T` from type inference (like `NoInfer`, for TypeScript < 5.4). */
type NoInference<T> = [T][T extends unknown ? 0 : never];

/**
 * Options accepted for items of type `T`: key names written as literals are
 * checked against `T`; keys typed as plain `string` are accepted.
 */
type CheckedOptions<T, S extends string, C extends string, Options> = Options & {
  readonly keys: ReadonlyArray<KeyEntry>;
} & KeyNames<S, C> &
  NoInference<KeyNames<CheckedKey<T, S>, CheckedKey<T, C>>>;

/**
 * Perform fuzzy search across object arrays with weighted keys.
 *
 * Wraps `searchKeys()` with an ergonomic API that accepts row-oriented
 * objects and returns matched items directly. Key names written as string
 * literals are type-checked against the item type (see {@link KeyPath}).
 *
 * @example
 * ```typescript
 * const users = [
 *   { name: 'John Smith', email: 'john@example.com' },
 *   { name: 'Jane Doe', email: 'jane@example.com' },
 * ];
 *
 * const results = searchObjects('john', users, {
 *   keys: [{ name: 'name', weight: 2.0 }, 'email'],
 * });
 * // results[0].item → { name: 'John Smith', email: 'john@example.com' }
 * ```
 */
export declare function searchObjects<T, S extends string = string, C extends string = string>(
  query: string,
  items: ReadonlyArray<T>,
  options: CheckedOptions<T, S, C, SearchOptions>,
): Array<ObjectSearchResult<T>>;

export interface ObjectIndexOptions<T = unknown> {
  keys: ReadonlyArray<KeyPath<T> | KeyConfig<T>>;
}

export interface ObjectIndexSearchOptions {
  maxResults?: number | undefined;
  minScore?: number | undefined;
  isCaseSensitive?: boolean | undefined;
  returnAllOnEmpty?: boolean | undefined;
}

/**
 * A persistent fuzzy search index for object collections with weighted keys.
 *
 * Pre-computes key texts and stores them on the Rust side for fast repeated
 * searches. Use this when searching the same collection multiple times.
 *
 * The type parameters after `T` only capture the key names passed to the
 * constructor so they can be checked against `T`; annotate variables as
 * `FuzzyObjectIndex<T>`.
 *
 * @example
 * ```typescript
 * const index = new FuzzyObjectIndex(users, {
 *   keys: [{ name: 'name', weight: 2.0 }, 'email'],
 * });
 *
 * const results = index.search('john');
 * // results[0].item → { name: 'John Smith', ... }
 *
 * index.add({ name: 'New User', email: 'new@example.com' });
 * index.destroy(); // free Rust-side memory
 * ```
 */
export declare class FuzzyObjectIndex<T, S extends string = string, C extends string = string> {
  /**
   * @throws {TypeError} If `items` is not an array or `options.keys` is not a
   *   non-empty array of key paths / key configs.
   */
  constructor(items: ReadonlyArray<T>, options: CheckedOptions<T, S, C, unknown>);

  /** Number of items in the index. */
  get size(): number;

  /**
   * Search for objects matching the query.
   *
   * The second argument accepts a number (maxResults shorthand) or an options
   * object, like `FuzzyIndex.search()`.
   *
   * @throws {TypeError} If `options` is neither a number nor an object.
   */
  search(
    query: string,
    options?: number | ObjectIndexSearchOptions | undefined | null,
  ): Array<ObjectSearchResult<T>>;

  /** Find the closest matching object, or null if no match. */
  closest(query: string, minScore?: number | undefined | null): T | null;

  /** Add a single item to the index. */
  add(item: T): void;

  /**
   * Add multiple items at once. If reading a key of any item throws, no item
   * is added.
   */
  addMany(items: ReadonlyArray<T>): void;

  /**
   * Remove the item at the given position (swap-remove semantics: the last
   * item moves into the freed slot, so its `index` changes).
   *
   * Returns `false` when `index` is out of range.
   *
   * @throws {TypeError} If `index` is not a number.
   * @throws {RangeError} If `index` is not an integer (e.g. `NaN` or `1.5`).
   */
  remove(index: number): boolean;

  /**
   * Remove all items and free their Rust-side memory.
   *
   * The index stays usable: afterwards it behaves as an empty index, and
   * `add()` / `addMany()` work as usual.
   */
  destroy(): void;

  /**
   * Serialize the index and its items to a Buffer.
   *
   * Items must be JSON-serializable. Pass the result to
   * `FuzzyObjectIndex.deserialize()` to reconstruct the index.
   */
  serialize(): Buffer;

  /**
   * Reconstruct a `FuzzyObjectIndex` from data produced by `serialize()`:
   * a Buffer, any other `ArrayBufferView` (e.g. `Uint8Array`) or an `ArrayBuffer`.
   *
   * @throws {TypeError} If `data` is not binary data.
   * @throws {Error} If `data` is malformed (the message starts with
   *   `"Invalid FuzzyObjectIndex data"`).
   */
  static deserialize<T = unknown>(data: ArrayBufferView | ArrayBufferLike): FuzzyObjectIndex<T>;
}
