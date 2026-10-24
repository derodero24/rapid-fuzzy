import { spawnSync } from 'node:child_process';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { FuzzyIndex, KeyedFuzzyIndex } from '../index.js';
import { FuzzyObjectIndex } from '../objects.js';

const ROOT = path.resolve(__dirname, '..');

const U32_MAX = 0xffff_ffff;

/** Byte offsets of the header fields shared by both formats. */
const VERSION_OFFSET = 4;
/** FuzzyIndex: item count. KeyedFuzzyIndex: key count. */
const COUNT_OFFSET = 8;
/** KeyedFuzzyIndex only: item count. */
const KEYED_ITEMS_OFFSET = 12;

function withU32(buffer: Buffer, offset: number, value: number): Buffer {
  const copy = Buffer.from(buffer);
  copy.writeUInt32LE(value, offset);
  return copy;
}

function keyedIndex(): KeyedFuzzyIndex {
  return new KeyedFuzzyIndex(
    [
      ['a', 'b'],
      ['c', 'd'],
    ],
    [1, 2],
  );
}

type Kind = 'FuzzyIndex' | 'KeyedFuzzyIndex' | 'FuzzyObjectIndex';

interface ChildOutcome {
  status: number | null;
  signal: NodeJS.Signals | null;
  stdout: string;
  stderr: string;
}

/**
 * Deserialize `bytes` in a separate Node.js process.
 *
 * A Rust allocation failure or panic aborts the whole process and cannot be
 * caught from JavaScript, so it has to be observed from the outside.
 */
function deserializeInChild(kind: Kind, bytes: Buffer): ChildOutcome {
  const script = `
    const { FuzzyIndex, KeyedFuzzyIndex } = require('./index.js');
    const { FuzzyObjectIndex } = require('./objects.js');
    const classes = { FuzzyIndex, KeyedFuzzyIndex, FuzzyObjectIndex };
    const bytes = Buffer.from(process.argv[1], 'base64');
    try {
      classes[process.argv[2]].deserialize(bytes);
      process.stdout.write(JSON.stringify({ threw: false }));
    } catch (error) {
      process.stdout.write(JSON.stringify({
        threw: true,
        isError: error instanceof Error,
        message: String(error && error.message),
      }));
    }
  `;
  const result = spawnSync(process.execPath, ['-e', script, bytes.toString('base64'), kind], {
    cwd: ROOT,
    encoding: 'utf8',
    timeout: 60_000,
  });
  return {
    status: result.status,
    signal: result.signal,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

interface ChildResult {
  threw: boolean;
  isError?: boolean;
  message?: string;
}

function expectCleanError(kind: Kind, bytes: Buffer, message: RegExp): void {
  const outcome = deserializeInChild(kind, bytes);
  // The process must survive: no abort ("memory allocation of N bytes
  // failed"), no panic, no signal.
  expect(outcome.stderr).not.toMatch(/memory allocation|panicked/);
  expect(outcome.signal).toBeNull();
  expect(outcome.status).toBe(0);
  const result = JSON.parse(outcome.stdout) as ChildResult;
  expect(result.threw).toBe(true);
  expect(result.isError).toBe(true);
  expect(result.message).toMatch(message);
}

describe('deserializing corrupt data never crashes the process', () => {
  it('KeyedFuzzyIndex with num_items = u32::MAX (29-byte audit payload)', () => {
    const bytes = withU32(keyedIndex().serialize(), KEYED_ITEMS_OFFSET, U32_MAX);
    expectCleanError('KeyedFuzzyIndex', bytes, /^Invalid data: /);
  });

  it.each([1, 3, 0x2000_0001, 0x4000_0000, U32_MAX])('KeyedFuzzyIndex with num_keys = %i', (n) => {
    const bytes = withU32(keyedIndex().serialize(), COUNT_OFFSET, n);
    expectCleanError('KeyedFuzzyIndex', bytes, /^Invalid data: /);
  });

  it.each([3, 0x4000_0000, 0x8000_0000, U32_MAX])('KeyedFuzzyIndex with num_items = %i', (n) => {
    const bytes = withU32(keyedIndex().serialize(), KEYED_ITEMS_OFFSET, n);
    expectCleanError('KeyedFuzzyIndex', bytes, /^Invalid data: /);
  });

  it.each([0x4000_0000, U32_MAX])('FuzzyIndex with item count = %i', (n) => {
    const bytes = withU32(new FuzzyIndex(['alpha', 'beta']).serialize(), COUNT_OFFSET, n);
    expectCleanError('FuzzyIndex', bytes, /^Invalid data: item count \d+ exceeds the payload/);
  });

  it('FuzzyIndex with an item length that wraps a 32-bit offset', () => {
    const bytes = withU32(new FuzzyIndex(['alpha', 'beta']).serialize(), 12, 0xffff_fff8);
    expectCleanError('FuzzyIndex', bytes, /^Invalid data: truncated: item 0/);
  });

  it('FuzzyObjectIndex whose embedded index declares u32::MAX items', () => {
    const objects = new FuzzyObjectIndex([{ name: 'a' }, { name: 'b' }], { keys: ['name'] });
    const bytes = Buffer.from(objects.serialize());
    // The embedded KeyedFuzzyIndex payload ends the buffer.
    const indexStart = 4 + bytes.readUInt32LE(0);
    bytes.writeUInt32LE(U32_MAX, indexStart + KEYED_ITEMS_OFFSET);
    // objects.js wraps the native error (kept as `cause`) with its own prefix.
    expectCleanError(
      'FuzzyObjectIndex',
      bytes,
      /^Invalid FuzzyObjectIndex data: Invalid data: 1 keys x 4294967295 items/,
    );
  });
});

describe('deserialization errors are precise JS Errors', () => {
  function errorOf(fn: () => unknown): Error {
    try {
      fn();
    } catch (error) {
      expect(error).toBeInstanceOf(Error);
      return error as Error;
    }
    throw new Error('expected the call to throw');
  }

  it('reports a short header with its length', () => {
    expect(errorOf(() => FuzzyIndex.deserialize(Buffer.from([1, 2]))).message).toBe(
      'Invalid data: too short: the header needs 12 bytes, got 2',
    );
  });

  it('names the expected and actual magic', () => {
    const keyed = keyedIndex().serialize();
    expect(errorOf(() => FuzzyIndex.deserialize(keyed)).message).toBe(
      'Invalid data: bad magic bytes: expected "RFZI" or "RFUZ", got "RFKI" (this is a serialized KeyedFuzzyIndex)',
    );
    const fuzzy = new FuzzyIndex(['a', 'b']).serialize();
    expect(errorOf(() => KeyedFuzzyIndex.deserialize(fuzzy)).message).toBe(
      'Invalid data: bad magic bytes: expected "RFKI", got "RFZI" (this is a serialized FuzzyIndex)',
    );
  });

  it('reports unsupported versions', () => {
    const bytes = withU32(new FuzzyIndex(['a']).serialize(), VERSION_OFFSET, 2);
    expect(errorOf(() => FuzzyIndex.deserialize(bytes)).message).toBe(
      'Unsupported format version: expected 1, got 2',
    );
  });

  it('reports trailing bytes', () => {
    const bytes = Buffer.concat([new FuzzyIndex(['a']).serialize(), Buffer.from([0, 0, 0])]);
    expect(errorOf(() => FuzzyIndex.deserialize(bytes)).message).toBe(
      'Invalid data: 3 trailing bytes after the last item (at byte 17)',
    );
  });

  it('reports invalid UTF-8 with the item it belongs to', () => {
    const bytes = Buffer.from(new FuzzyIndex(['ok', 'xy']).serialize());
    bytes[bytes.length - 1] = 0xff;
    expect(errorOf(() => FuzzyIndex.deserialize(bytes)).message).toMatch(
      /^Invalid data: item 1 at byte 18 is not valid UTF-8/,
    );
  });

  it('reports invalid weights with the key they belong to', () => {
    const bytes = Buffer.from(keyedIndex().serialize());
    bytes.writeDoubleLE(-1, 16 + 8);
    expect(errorOf(() => KeyedFuzzyIndex.deserialize(bytes)).message).toBe(
      'Invalid data: weight of key 1 is -1; weights must be finite non-negative numbers',
    );
  });
});

describe('FuzzyIndex magic is shared by the Node.js and browser builds', () => {
  it('writes "RFZI"', () => {
    expect(new FuzzyIndex(['a']).serialize().subarray(0, 4).toString('latin1')).toBe('RFZI');
  });

  it('reads indexes written by the rapid-fuzzy <= 2.1.1 browser build ("RFUZ")', () => {
    const items = ['TypeScript', 'JavaScript', '東京'];
    const legacy = Buffer.from(new FuzzyIndex(items).serialize());
    legacy.write('RFUZ', 0, 'latin1');

    const restored = FuzzyIndex.deserialize(legacy);
    expect(restored.size).toBe(items.length);
    expect(restored.search('script').map((r) => r.item)).toEqual(['TypeScript', 'JavaScript']);
    // Re-serializing upgrades to the shared magic.
    expect(restored.serialize().subarray(0, 4).toString('latin1')).toBe('RFZI');
  });
});

describe('serialize/deserialize round-trips every index state', () => {
  it('destroyed KeyedFuzzyIndex', () => {
    const index = keyedIndex();
    index.destroy();
    const restored = KeyedFuzzyIndex.deserialize(index.serialize());
    expect(restored.size).toBe(0);
    expect(restored.search('a')).toEqual([]);
    expect(Buffer.compare(restored.serialize(), index.serialize())).toBe(0);
  });

  it('KeyedFuzzyIndex with keys but no items', () => {
    const index = new KeyedFuzzyIndex([[], []], [1, 1]);
    const restored = KeyedFuzzyIndex.deserialize(index.serialize());
    expect(restored.size).toBe(0);
    restored.add(['x', 'y']);
    expect(restored.size).toBe(1);
  });

  it('destroyed FuzzyIndex', () => {
    const index = new FuzzyIndex(['a', 'b']);
    index.destroy();
    expect(FuzzyIndex.deserialize(index.serialize()).size).toBe(0);
  });

  it('destroyed FuzzyObjectIndex', () => {
    const index = new FuzzyObjectIndex([{ name: 'a' }], { keys: ['name'] });
    index.destroy();
    const restored = FuzzyObjectIndex.deserialize(index.serialize());
    expect(restored.size).toBe(0);
    expect(restored.search('a')).toEqual([]);
  });
});

// deserialize() is declared to take a Uint8Array: bytes from fetch(),
// IndexedDB or the browser build arrive as plain Uint8Arrays, not Buffers.
describe('deserialize() accepts any Uint8Array', () => {
  it('FuzzyIndex', () => {
    const bytes = new Uint8Array(new FuzzyIndex(['TypeScript', '東京']).serialize());
    expect(Buffer.isBuffer(bytes)).toBe(false);
    const restored = FuzzyIndex.deserialize(bytes);
    expect(restored.size).toBe(2);
    expect(restored.closest('東')).toBe('東京');
  });

  it('KeyedFuzzyIndex', () => {
    const bytes = new Uint8Array(keyedIndex().serialize());
    const restored = KeyedFuzzyIndex.deserialize(bytes);
    expect(restored.size).toBe(keyedIndex().size);
    expect(restored.serialize()).toEqual(keyedIndex().serialize());
  });

  it('a view into a larger buffer', () => {
    const serialized = new FuzzyIndex(['apple']).serialize();
    const larger = new Uint8Array(serialized.length + 8);
    larger.set(serialized, 4);
    const restored = FuzzyIndex.deserialize(larger.subarray(4, 4 + serialized.length));
    expect(restored.closest('aple')).toBe('apple');
  });
});
