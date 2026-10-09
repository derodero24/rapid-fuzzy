'use strict';

const { types } = require('node:util');

const { searchKeys, KeyedFuzzyIndex } = require('./index.js');

/**
 * Whether `value` has a string form of its own (Date, URL, a class with a
 * `toString()` method, ...) rather than the generic `"[object Object]"`.
 * @param {object} value
 * @returns {boolean}
 */
function hasStringForm(value) {
  if (typeof value[Symbol.toPrimitive] === 'function') return true;
  const stringify = value.toString;
  return typeof stringify === 'function' && stringify !== Object.prototype.toString;
}

/**
 * Text of a single value: strings as-is, numbers, bigints and booleans
 * stringified, objects only when they have a string form of their own.
 * Everything else (null, undefined, functions, symbols, plain objects,
 * arrays) has no text.
 * @param {unknown} value
 * @returns {string}
 */
function scalarText(value) {
  switch (typeof value) {
    case 'string':
      return value;
    case 'number':
    case 'bigint':
    case 'boolean':
      return String(value);
    case 'object':
      return value !== null && !Array.isArray(value) && hasStringForm(value) ? String(value) : '';
    default:
      return '';
  }
}

/**
 * The searchable text of a resolved key value. Arrays are indexed as their
 * elements' text joined with spaces; elements without text (including nested
 * arrays and plain objects) are skipped.
 * @param {unknown} value
 * @returns {string}
 */
function toSearchText(value) {
  if (!Array.isArray(value)) return scalarText(value);
  let text = '';
  for (const element of value) {
    const part = scalarText(element);
    if (part !== '') text = text === '' ? part : `${text} ${part}`;
  }
  return text;
}

/**
 * Read `key` from `container`: its own properties, plus getters defined by its
 * class (found on the prototype chain below `Object.prototype` and
 * `Function.prototype`). Inherited methods and data properties such as
 * `constructor` or `toString` are never read.
 * @param {object | Function} container
 * @param {string} key
 * @returns {unknown}
 */
function readKey(container, key) {
  if (Object.hasOwn(container, key)) return container[key];
  let proto = Object.getPrototypeOf(container);
  while (proto !== null && proto !== Object.prototype && proto !== Function.prototype) {
    const descriptor = Object.getOwnPropertyDescriptor(proto, key);
    if (descriptor !== undefined) {
      return descriptor.get === undefined
        ? undefined
        : Reflect.apply(descriptor.get, container, []);
    }
    proto = Object.getPrototypeOf(proto);
  }
  return undefined;
}

/**
 * Resolve a key path (pre-split on `.`) against `item` and return its text.
 * Missing values, and paths that run into a primitive, give `''`.
 * @param {unknown} item
 * @param {string[]} path
 * @returns {string}
 */
function getNestedValue(item, path) {
  let current = item;
  for (const segment of path) {
    if (current === null || (typeof current !== 'object' && typeof current !== 'function')) {
      return '';
    }
    current = readKey(current, segment);
  }
  return toSearchText(current);
}

/**
 * Validate and normalize the `keys` option.
 * @param {unknown} keys
 * @returns {Array<{ name: string; weight: number }>}
 */
function normalizeKeys(keys) {
  if (!Array.isArray(keys) || keys.length === 0) {
    throw new TypeError('options.keys must be a non-empty array');
  }
  return keys.map((key, i) => {
    const config = typeof key === 'string' ? { name: key } : key;
    if (config === null || typeof config !== 'object' || typeof config.name !== 'string') {
      throw new TypeError(
        `options.keys[${i}] must be a key path string or an object with a string name`,
      );
    }
    const weight = config.weight ?? 1.0;
    if (typeof weight !== 'number') {
      throw new TypeError(`options.keys[${i}].weight must be a number`);
    }
    return { name: config.name, weight };
  });
}

/**
 * @param {Array<{ name: string }>} keys
 * @returns {string[][]}
 */
function keyPaths(keys) {
  return keys.map((key) => key.name.split('.'));
}

/**
 * @param {unknown} items
 * @param {string} name
 */
function assertArray(items, name) {
  if (!Array.isArray(items)) {
    throw new TypeError(`${name} must be an array`);
  }
}

/**
 * Key texts in column order (`columns[k][i]` is key `k` of item `i`).
 * Holes in sparse arrays are treated as `undefined` items.
 * @param {ReadonlyArray<unknown>} items
 * @param {string[][]} paths
 * @returns {string[][]}
 */
function keyColumns(items, paths) {
  return paths.map((path) => Array.from(items, (item) => getNestedValue(item, path)));
}

/**
 * Check a `number | options` argument, so that both builds throw the same
 * `TypeError` for anything else.
 * @param {unknown} options
 * @param {string} shorthand - The option a number sets.
 * @param {string} type - The name of the options type.
 */
function assertOptions(options, shorthand, type) {
  if (options != null && typeof options !== 'number' && typeof options !== 'object') {
    throw new TypeError(
      `options must be a number (${shorthand}) or a ${type} object, got ${typeof options}`,
    );
  }
}

/**
 * Perform fuzzy search across object arrays with weighted keys.
 *
 * Wraps `searchKeys()` with an ergonomic API that accepts row-oriented
 * objects and returns matched items directly.
 *
 * @template T
 * @param {string} query - The search query.
 * @param {ReadonlyArray<T>} items - Array of objects to search.
 * @param {object} options - Search options with keys configuration.
 * @param {ReadonlyArray<string | { name: string; weight?: number }>} options.keys - Keys to search.
 * @param {number} [options.maxResults] - Maximum results to return.
 * @param {number} [options.minScore] - Minimum score threshold.
 * @param {boolean} [options.isCaseSensitive] - Enable case-sensitive matching.
 * @param {'weighted' | 'matched' | 'max'} [options.scoreMode] - How the
 *   per-key scores are combined (default `'weighted'`).
 * @param {'perKey' | 'crossKey'} [options.matchMode] - How the query is
 *   matched against the keys (default `'perKey'`; with `'crossKey'` the
 *   query's terms may match different keys).
 * @returns {Array<{ item: T; index: number; score: number; keyScores: number[] }>}
 */
function searchObjects(query, items, options) {
  if (options === null || typeof options !== 'object') {
    throw new TypeError('options must be an object with a non-empty keys array');
  }
  const { keys, ...searchOpts } = options;
  const normalizedKeys = normalizeKeys(keys);
  assertArray(items, 'items');

  const keyTexts = keyColumns(items, keyPaths(normalizedKeys));
  const weights = normalizedKeys.map((k) => k.weight);
  const nativeOpts = Object.keys(searchOpts).length > 0 ? searchOpts : undefined;

  return searchKeys(query, keyTexts, weights, nativeOpts).map((r) => ({
    item: items[r.index],
    index: r.index,
    score: r.score,
    keyScores: r.keyScores,
  }));
}

/**
 * A persistent fuzzy search index for object collections with weighted keys.
 *
 * Pre-computes key texts and stores them on the Rust side for fast repeated
 * searches. Use this when searching the same collection multiple times.
 *
 * The JS-side `#items` array mirrors the native index slot for slot. Every
 * mutating method computes and validates everything that can throw before it
 * touches either side, so a failed call leaves both unchanged.
 *
 * @template T
 */
class FuzzyObjectIndex {
  /** @type {T[]} */
  #items;
  /** @type {KeyedFuzzyIndex} */
  #index;
  /** @type {Array<{ name: string; weight: number }>} */
  #keys;
  /** @type {string[][]} */
  #paths;

  static #SENTINEL = Symbol('FuzzyObjectIndex.internal');

  /**
   * @param {ReadonlyArray<T>} items - Array of objects to index.
   * @param {object} options - Index configuration.
   * @param {ReadonlyArray<string | { name: string; weight?: number }>} options.keys - Keys to search.
   */
  constructor(items, options, _sentinel) {
    if (_sentinel === FuzzyObjectIndex.#SENTINEL) return;
    const keys = normalizeKeys(options?.keys);
    assertArray(items, 'items');
    const paths = keyPaths(keys);
    this.#index = new KeyedFuzzyIndex(
      keyColumns(items, paths),
      keys.map((k) => k.weight),
    );
    this.#items = Array.from(items);
    this.#keys = keys;
    this.#paths = paths;
  }

  /** Return the number of items in the index. */
  get size() {
    return this.#index.size;
  }

  /**
   * @param {T} item
   * @returns {string[]}
   */
  #keyValues(item) {
    return this.#paths.map((path) => getNestedValue(item, path));
  }

  /**
   * Search the index for objects matching the query.
   * @param {string} query
   * @param {number | { maxResults?: number; minScore?: number; isCaseSensitive?: boolean; returnAllOnEmpty?: boolean; scoreMode?: 'weighted' | 'matched' | 'max'; matchMode?: 'perKey' | 'crossKey' } | null} [options]
   *   A KeySearchOptions object, or a number as a shorthand for `maxResults`.
   * @returns {Array<{ item: T; index: number; score: number; keyScores: number[] }>}
   */
  search(query, options) {
    assertOptions(options, 'maxResults', 'SearchOptions');
    return this.#index.search(query, options).map((r) => ({
      item: this.#items[r.index],
      index: r.index,
      score: r.score,
      keyScores: r.keyScores,
    }));
  }

  /**
   * Find the closest matching object: the item of the first result of
   * `search(query, { maxResults: 1, minScore, scoreMode, matchMode })`.
   * @param {string} query
   * @param {number | { minScore?: number; scoreMode?: 'weighted' | 'matched' | 'max'; matchMode?: 'perKey' | 'crossKey' } | null} [options]
   *   A KeyClosestOptions object, or a number as a shorthand for `minScore`.
   *   `scoreMode` (default `'weighted'`) selects how the per-key scores are
   *   combined, `matchMode` (default `'perKey'`) how the query is matched
   *   against the keys.
   * @returns {T | null}
   */
  closest(query, options) {
    assertOptions(options, 'minScore', 'KeyClosestOptions');
    const index = this.#index.closest(query, options);
    return index === null ? null : this.#items[index];
  }

  /**
   * Add a single item to the index.
   * @param {T} item
   */
  add(item) {
    this.#index.add(this.#keyValues(item));
    this.#items.push(item);
  }

  /**
   * Add multiple items to the index at once. Either every item is added or,
   * if reading a key throws, none is.
   * @param {ReadonlyArray<T>} items
   */
  addMany(items) {
    assertArray(items, 'items');
    const added = Array.from(items);
    this.#index.addMany(added.map((item) => this.#keyValues(item)));
    for (const item of added) {
      this.#items.push(item);
    }
  }

  /**
   * Remove the item at the given index.
   * Uses swap-remove semantics for O(1) performance: the last item moves into
   * the freed slot.
   * @param {number} index - An integer; out-of-range values return false.
   * @returns {boolean}
   */
  remove(index) {
    if (typeof index !== 'number') {
      throw new TypeError(`index must be a number, got ${typeof index}`);
    }
    if (!Number.isInteger(index)) {
      throw new RangeError(`index must be an integer, got ${index}`);
    }
    if (index < 0 || index >= this.#items.length) return false;
    if (!this.#index.remove(index)) return false;
    // Mirror the Rust-side swap-remove on the JS items array.
    const last = this.#items.pop();
    if (index < this.#items.length) {
      this.#items[index] = last;
    }
    return true;
  }

  /**
   * Free all items. The index stays usable: it behaves as an empty index and
   * accepts new items through `add()` / `addMany()`.
   */
  destroy() {
    this.#items = [];
    this.#index.destroy();
  }

  /**
   * Serialize the index and its items to a Buffer for storage or transfer.
   *
   * Items must be JSON-serializable. The returned Buffer can be passed to
   * `FuzzyObjectIndex.deserialize()` to reconstruct the index without
   * re-processing the original data.
   *
   * @returns {Buffer}
   */
  serialize() {
    const metaBuf = Buffer.from(JSON.stringify({ items: this.#items, keys: this.#keys }), 'utf8');
    const indexBuf = this.#index.serialize();
    const header = Buffer.allocUnsafe(4);
    header.writeUInt32LE(metaBuf.length, 0);
    return Buffer.concat([header, metaBuf, indexBuf]);
  }

  /**
   * @param {Array<T>} items
   * @param {Array<{ name: string; weight: number }>} keys
   * @param {KeyedFuzzyIndex} index
   * @returns {FuzzyObjectIndex<T>}
   */
  static #fromState(items, keys, index) {
    const instance = new FuzzyObjectIndex(undefined, undefined, FuzzyObjectIndex.#SENTINEL);
    instance.#items = items;
    instance.#keys = keys;
    instance.#paths = keyPaths(keys);
    instance.#index = index;
    return instance;
  }

  /**
   * Reconstruct a `FuzzyObjectIndex` from data produced by `serialize()`.
   *
   * Accepts a Buffer, any other `ArrayBufferView` (`Uint8Array`, `DataView`,
   * ...) or an `ArrayBuffer`. Throws a `TypeError` for other values and an
   * `Error` whose message starts with `"Invalid FuzzyObjectIndex data"` when
   * the data is malformed.
   *
   * @template T
   * @param {ArrayBufferView | ArrayBuffer} data - Data previously returned by `serialize()`.
   * @returns {FuzzyObjectIndex<T>}
   */
  static deserialize(data) {
    const bytes = toBytes(data);
    if (bytes.byteLength < 4) {
      throw invalidData('too short');
    }
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const metaEnd = 4 + view.getUint32(0, true);
    if (metaEnd > bytes.byteLength) {
      throw invalidData('metadata length exceeds the data size');
    }
    const { items, keys } = parseMeta(bytes.subarray(4, metaEnd));

    const native = Buffer.from(
      bytes.buffer,
      bytes.byteOffset + metaEnd,
      bytes.byteLength - metaEnd,
    );
    let index;
    try {
      index = KeyedFuzzyIndex.deserialize(native);
    } catch (err) {
      throw invalidData(err instanceof Error ? err.message : String(err), err);
    }
    if (index.size !== items.length) {
      throw invalidData(`item count mismatch (${items.length} items, ${index.size} indexed)`);
    }
    index = reconcileKeys(index, nativeWeights(native), items, keys);
    return FuzzyObjectIndex.#fromState(items, keys, index);
  }
}

/**
 * @param {string} reason
 * @param {unknown} [cause]
 * @returns {Error}
 */
function invalidData(reason, cause) {
  const message = `Invalid FuzzyObjectIndex data: ${reason}`;
  return cause === undefined ? new Error(message) : new Error(message, { cause });
}

/**
 * Check the keys of the metadata against the native index they were
 * serialized with, and return the index to use.
 * @param {KeyedFuzzyIndex} index - The deserialized native index.
 * @param {number[]} weights - Its key weights (see {@link nativeWeights}).
 * @param {unknown[]} items
 * @param {Array<{ name: string; weight: number }>} keys
 * @returns {KeyedFuzzyIndex}
 */
function reconcileKeys(index, weights, items, keys) {
  if (weights.length === 0 && items.length === 0) {
    // rapid-fuzzy 2.1 cleared the native key configuration on destroy(), so
    // its data kept the keys only in the metadata: rebuild the empty index.
    try {
      return new KeyedFuzzyIndex(
        keys.map(() => []),
        keys.map((k) => k.weight),
      );
    } catch (err) {
      throw invalidData(err instanceof Error ? err.message : String(err), err);
    }
  }
  if (weights.length !== keys.length) {
    throw invalidData(`key count mismatch (${keys.length} keys, ${weights.length} indexed)`);
  }
  const k = keys.findIndex((key, i) => key.weight !== weights[i]);
  if (k !== -1) {
    throw invalidData(
      `weight of key ${k} (${keys[k].name}) is ${keys[k].weight}, the native index has ${weights[k]}`,
    );
  }
  return index;
}

/**
 * The key weights stored in a serialized `KeyedFuzzyIndex` that
 * `KeyedFuzzyIndex.deserialize()` accepted: `[magic 4B] [version u32]
 * [num_keys u32] [num_items u32] [num_keys x f64 weight] ...`, little-endian.
 * @param {Uint8Array} bytes
 * @returns {number[]}
 */
function nativeWeights(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const numKeys = view.getUint32(8, true);
  return Array.from({ length: numKeys }, (_, k) => view.getFloat64(16 + 8 * k, true));
}

/**
 * View `data` as bytes without copying.
 * @param {unknown} data
 * @returns {Uint8Array}
 */
function toBytes(data) {
  if (ArrayBuffer.isView(data)) {
    return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  if (types.isAnyArrayBuffer(data)) {
    return new Uint8Array(data);
  }
  throw new TypeError(
    'FuzzyObjectIndex.deserialize() expects a Buffer, Uint8Array, ArrayBufferView or ArrayBuffer',
  );
}

/**
 * @param {unknown} key
 * @returns {key is { name: string; weight: number }}
 */
function isStoredKey(key) {
  return (
    key !== null &&
    typeof key === 'object' &&
    typeof key.name === 'string' &&
    typeof key.weight === 'number'
  );
}

/**
 * Parse and validate the JSON metadata block written by `serialize()`.
 * @param {Uint8Array} bytes
 * @returns {{ items: unknown[]; keys: Array<{ name: string; weight: number }> }}
 */
function parseMeta(bytes) {
  let meta;
  try {
    meta = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  } catch (err) {
    throw invalidData('metadata is not valid UTF-8 JSON', err);
  }
  if (
    meta === null ||
    typeof meta !== 'object' ||
    !Array.isArray(meta.items) ||
    !Array.isArray(meta.keys) ||
    meta.keys.length === 0 ||
    !meta.keys.every(isStoredKey)
  ) {
    throw invalidData('metadata must hold an items array and a non-empty keys array');
  }
  return {
    items: meta.items,
    keys: meta.keys.map((key) => ({ name: key.name, weight: key.weight })),
  };
}

module.exports = { searchObjects, FuzzyObjectIndex };
