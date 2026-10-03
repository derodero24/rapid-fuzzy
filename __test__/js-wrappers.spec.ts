import { describe, expect, it } from 'vitest';

import { FuzzyIndex, KeyedFuzzyIndex } from '../index.js';
import { FuzzyObjectIndex, searchObjects } from '../objects.js';

interface Fruit {
  n: string;
}

const fruits: Fruit[] = [{ n: 'apple' }, { n: 'banana' }, { n: 'cherry' }];

/** Search for an exact name and return the item the index maps it to. */
function lookup(index: FuzzyObjectIndex<Fruit>, name: string): Fruit | undefined {
  return index.search(name, { maxResults: 1, minScore: 1 })[0]?.item;
}

/** Assert that every name in `expected` maps back to the object holding it. */
function expectConsistent(index: FuzzyObjectIndex<Fruit>, expected: readonly Fruit[]): void {
  expect(index.size).toBe(expected.length);
  for (const item of expected) {
    expect(lookup(index, item.n)).toBe(item);
  }
}

// Avoid `any` while still passing deliberately wrong values to the API.
function unsafe<T>(value: unknown): T {
  return value as T;
}

describe('FuzzyObjectIndex.remove() argument validation', () => {
  it.each([
    ['NaN', Number.NaN],
    ['a fraction', 1.5],
    ['Infinity', Number.POSITIVE_INFINITY],
    ['-Infinity', Number.NEGATIVE_INFINITY],
  ])('throws a RangeError for %s and leaves the index unchanged', (_label, value) => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    expect(() => index.remove(value)).toThrow(RangeError);
    expectConsistent(index, fruits);
  });

  it.each([
    ['a string', '1'],
    ['null', null],
    ['undefined', undefined],
  ])('throws a TypeError for %s and leaves the index unchanged', (_label, value) => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    expect(() => index.remove(unsafe<number>(value))).toThrow(TypeError);
    expectConsistent(index, fruits);
  });

  it('returns false for out-of-range integers', () => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    expect(index.remove(-1)).toBe(false);
    expect(index.remove(3)).toBe(false);
    expect(index.remove(2 ** 32)).toBe(false);
    expectConsistent(index, fruits);
  });

  it('keeps items and results in sync after a swap-remove', () => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    expect(index.remove(0)).toBe(true);
    // cherry moved into slot 0
    expect(index.search('cherry', { maxResults: 1 })[0]).toMatchObject({
      item: fruits[2],
      index: 0,
    });
    expectConsistent(index, [fruits[2] as Fruit, fruits[1] as Fruit]);
  });
});

describe('FuzzyObjectIndex keeps its items in sync with the native index', () => {
  it('survives a long random sequence of add / addMany / remove', () => {
    let seed = 42;
    const random = (n: number): number => {
      seed = (seed * 1103515245 + 12345) % 2 ** 31;
      return seed % n;
    };
    let counter = 0;
    const make = (): Fruit => ({ n: `item${String(counter++).padStart(4, '0')}` });

    const model: Fruit[] = [make(), make(), make()];
    const index = new FuzzyObjectIndex(model, { keys: ['n'] });

    for (let step = 0; step < 300; step++) {
      const op = random(4);
      if (op === 0) {
        const item = make();
        index.add(item);
        model.push(item);
      } else if (op === 1) {
        const batch = [make(), make()];
        index.addMany(batch);
        model.push(...batch);
      } else if (model.length > 0) {
        const at = random(model.length);
        expect(index.remove(at)).toBe(true);
        const last = model.pop() as Fruit;
        if (at < model.length) model[at] = last;
      }
    }
    expectConsistent(index, model);
  });

  it('does not add anything when reading a key throws in add()', () => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    const bad = {
      get n(): string {
        throw new Error('boom');
      },
    };
    expect(() => index.add(bad)).toThrow('boom');
    expectConsistent(index, fruits);
  });

  it('does not add anything when reading a key throws in addMany()', () => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    const good = { n: 'damson' };
    const bad = {
      get n(): string {
        throw new Error('boom');
      },
    };
    expect(() => index.addMany([good, bad])).toThrow('boom');
    expectConsistent(index, fruits);
    expect(index.search('damson')).toEqual([]);
  });

  it('treats holes in sparse arrays as undefined items', () => {
    // biome-ignore lint/suspicious/noSparseArray: testing sparse input on purpose
    const sparse: Array<Fruit | undefined> = [{ n: 'apple' }, , { n: 'cherry' }];
    const index = new FuzzyObjectIndex(sparse, { keys: ['n'] });
    expect(index.size).toBe(3);
    expect(index.search('cherry')[0]).toMatchObject({ item: { n: 'cherry' }, index: 2 });
    index.addMany(sparse);
    expect(index.size).toBe(6);
  });
});

describe('key path resolution', () => {
  it.each(['constructor', 'toString', 'hasOwnProperty', '__proto__', 'constructor.name'])(
    'does not read the inherited property %s',
    (key) => {
      const results = searchObjects('function', [{ a: 1 }], { keys: [key] });
      expect(results).toEqual([]);
      expect(searchObjects('object', [{ a: 1 }], { keys: [key] })).toEqual([]);
      expect(searchObjects('Object', [{ a: 1 }], { keys: [key] })).toEqual([]);
    },
  );

  it('does not read inherited data properties', () => {
    const item: { secret?: string } = Object.create({ secret: 'hidden' });
    expect(searchObjects('hidden', [item], { keys: ['secret'] })).toEqual([]);
  });

  it('reads getters defined by a class', () => {
    class Person {
      constructor(
        readonly first: string,
        readonly last: string,
      ) {}
      get fullName(): string {
        return `${this.first} ${this.last}`;
      }
    }
    const people = [new Person('Ada', 'Lovelace'), new Person('Alan', 'Turing')];
    const results = searchObjects('lovelace', people, { keys: ['fullName'] });
    expect(results.map((r) => r.item)).toEqual([people[0]]);
  });

  it('reads own accessor properties', () => {
    const item = Object.defineProperty({}, 'label', { get: () => 'computed', enumerable: false });
    expect(searchObjects('computed', [item], { keys: ['label'] })).toHaveLength(1);
  });

  it('does not index plain objects as "[object Object]"', () => {
    const items = [{ a: { b: 1 } }];
    expect(searchObjects('object', items, { keys: ['a'] })).toEqual([]);
  });

  it('handles null-prototype objects', () => {
    const bare: Record<string, string> = Object.create(null);
    bare.b = 'inside';
    expect(searchObjects('x', [{ a: Object.create(null) }], { keys: ['a'] })).toEqual([]);
    expect(searchObjects('inside', [{ a: bare }], { keys: ['a.b'] })).toHaveLength(1);
  });

  it('skips functions and symbols', () => {
    const items = [{ f: function hello() {}, s: Symbol('sym') }];
    // Function-valued keys are not key paths at the type level either.
    const functionKey: string = 'f';
    expect(searchObjects('hello', items, { keys: [functionKey] })).toEqual([]);
    expect(searchObjects('function', items, { keys: [functionKey] })).toEqual([]);
    expect(searchObjects('sym', items, { keys: ['s'] })).toEqual([]);
  });

  it('converts numbers, bigints and booleans to strings', () => {
    const items = [{ n: 42, b: true, big: 12345678901234567890n }];
    expect(searchObjects('42', items, { keys: ['n'] })).toHaveLength(1);
    expect(searchObjects('true', items, { keys: ['b'] })).toHaveLength(1);
    expect(searchObjects('12345678901234567890', items, { keys: ['big'] })).toHaveLength(1);
  });

  it('keeps objects that define their own string form', () => {
    class Version {
      constructor(readonly value: string) {}
      toString(): string {
        return `v${this.value}`;
      }
    }
    const items = [{ v: new Version('1.2.3'), d: new Date(Date.UTC(2001, 0, 1, 12)) }];
    expect(searchObjects('v1.2.3', items, { keys: ['v'] })).toHaveLength(1);
    expect(searchObjects('2001', items, { keys: ['d'] })).toHaveLength(1);
  });

  it('joins arrays of primitives with spaces', () => {
    const items = [{ tags: ['red', 'green', 7, true] }];
    expect(searchObjects('green', items, { keys: ['tags'] })).toHaveLength(1);
    expect(searchObjects('red green 7 true', items, { keys: ['tags'] })).toHaveLength(1);
    // No comma separator (String(array) would produce "red,green,7,true").
    expect(searchObjects(',', items, { keys: ['tags'] })).toEqual([]);
  });

  it('skips non-primitive array elements', () => {
    const items = [{ tags: ['red', { x: 1 }, () => 1, null, undefined, ['nested']] }];
    expect(searchObjects('red', items, { keys: ['tags'] })).toHaveLength(1);
    expect(searchObjects('object', items, { keys: ['tags'] })).toEqual([]);
    expect(searchObjects('nested', items, { keys: ['tags'] })).toEqual([]);
  });

  it('supports numeric segments to index into arrays', () => {
    const items = [{ tags: ['red', 'green'] }, { tags: ['blue'] }];
    const results = searchObjects('green', items, { keys: ['tags.1'] });
    expect(results.map((r) => r.index)).toEqual([0]);
  });

  it('does not descend into primitives', () => {
    const keys: string[] = ['a.length'];
    expect(searchObjects('5', [{ a: 'hello' }], { keys })).toEqual([]);
  });

  it('applies the same rules in FuzzyObjectIndex', () => {
    const items = [{ a: { b: 1 }, tags: ['x', 'y'] }];
    const keys: string[] = ['a', 'constructor', 'tags'];
    const index = new FuzzyObjectIndex(items, { keys });
    expect(index.search('object')).toEqual([]);
    expect(index.search('function')).toEqual([]);
    expect(index.search(',')).toEqual([]);
    index.add({ a: { b: 2 }, tags: ['object'] });
    expect(index.search('object').map((r) => r.index)).toEqual([1]);
  });
});

describe('key configuration validation', () => {
  it.each([
    ['a number', 1],
    ['null', null],
    ['an object without a name', { weight: 1 }],
    ['a non-string name', { name: 5 }],
    ['a non-numeric weight', { name: 'n', weight: '2' }],
  ])('rejects %s as a key', (_label, key) => {
    expect(() => searchObjects('a', fruits, { keys: [unsafe<string>(key)] })).toThrow(TypeError);
    expect(() => new FuzzyObjectIndex(fruits, { keys: [unsafe<string>(key)] })).toThrow(TypeError);
  });

  it('rejects a non-array items argument', () => {
    expect(() => searchObjects('a', unsafe<Fruit[]>('apple'), { keys: ['n'] })).toThrow(TypeError);
    expect(() => new FuzzyObjectIndex(unsafe<Fruit[]>(null), { keys: ['n'] })).toThrow(TypeError);
  });
});

describe('FuzzyObjectIndex.deserialize()', () => {
  const serialized = (): Buffer => new FuzzyObjectIndex(fruits, { keys: ['n'] }).serialize();

  it('accepts a plain Uint8Array copy', () => {
    const restored = FuzzyObjectIndex.deserialize<Fruit>(new Uint8Array(serialized()));
    expect(restored.size).toBe(3);
    expect(lookup(restored, 'cherry')).toEqual({ n: 'cherry' });
  });

  it('accepts a view at a non-zero offset of a larger buffer', () => {
    const bytes = serialized();
    const backing = new Uint8Array(bytes.length + 7);
    backing.set(bytes, 3);
    const view = backing.subarray(3, 3 + bytes.length);
    expect(FuzzyObjectIndex.deserialize<Fruit>(view).size).toBe(3);
    const dataView = new DataView(backing.buffer, 3, bytes.length);
    expect(FuzzyObjectIndex.deserialize<Fruit>(dataView).size).toBe(3);
  });

  it('accepts an ArrayBuffer', () => {
    const bytes = serialized();
    const copy = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    expect(FuzzyObjectIndex.deserialize<Fruit>(copy).size).toBe(3);
  });

  it.each([
    ['an empty buffer', Buffer.alloc(0)],
    ['a truncated header', Buffer.from([1, 0])],
    ['a metadata length past the end', Buffer.from([255, 0, 0, 0, 1, 2, 3])],
    ['metadata that is not JSON', Buffer.from([3, 0, 0, 0, 1, 2, 3])],
    ['metadata of the wrong shape', withMeta('{"items":{},"keys":[]}')],
    ['metadata without keys', withMeta('{"items":[]}')],
    ['metadata with a malformed key', withMeta('{"items":[],"keys":[{"name":1,"weight":1}]}')],
    ['metadata that is not an object', withMeta('null')],
  ])('throws a descriptive Error for %s', (_label, data) => {
    let caught: unknown;
    try {
      FuzzyObjectIndex.deserialize(data);
    } catch (err) {
      caught = err;
    }
    expect(caught).toBeInstanceOf(Error);
    expect(caught).not.toBeInstanceOf(SyntaxError);
    expect((caught as Error).message).toMatch(/^Invalid FuzzyObjectIndex data/);
  });

  it('throws a descriptive Error when the native index part is corrupt', () => {
    const bytes = serialized();
    const metaLen = bytes.readUInt32LE(0);
    const corrupt = Buffer.from(bytes);
    corrupt.write('XXXX', 4 + metaLen, 'latin1');
    expect(() => FuzzyObjectIndex.deserialize(corrupt)).toThrow(/^Invalid FuzzyObjectIndex data/);
  });

  it('throws when the item count does not match the native index', () => {
    const bytes = serialized();
    const metaLen = bytes.readUInt32LE(0);
    const meta = JSON.parse(bytes.subarray(4, 4 + metaLen).toString('utf8'));
    meta.items.pop();
    const tampered = Buffer.concat([withMeta(JSON.stringify(meta)), bytes.subarray(4 + metaLen)]);
    expect(() => FuzzyObjectIndex.deserialize(tampered)).toThrow(/item count/);
  });

  it.each([
    ['a string', 'abc'],
    ['a number', 42],
    ['null', null],
    ['an array', [1, 2, 3]],
  ])('throws a TypeError for %s', (_label, data) => {
    expect(() => FuzzyObjectIndex.deserialize(unsafe<Uint8Array>(data))).toThrow(TypeError);
  });
});

function withMeta(json: string): Buffer {
  const meta = Buffer.from(json, 'utf8');
  const header = Buffer.alloc(4);
  header.writeUInt32LE(meta.length, 0);
  return Buffer.concat([header, meta]);
}

describe('destroy() leaves a usable, empty index', () => {
  it('FuzzyObjectIndex behaves as empty and accepts new items', () => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    index.destroy();
    expect(index.size).toBe(0);
    expect(index.search('apple')).toEqual([]);
    expect(index.search('', { returnAllOnEmpty: true })).toEqual([]);
    expect(index.closest('apple')).toBeNull();
    expect(index.remove(0)).toBe(false);

    const date = { n: 'date' };
    index.add(date);
    index.addMany([{ n: 'elderberry' }]);
    expect(index.size).toBe(2);
    expect(lookup(index, 'date')).toBe(date);

    const restored = FuzzyObjectIndex.deserialize<Fruit>(index.serialize());
    expect(restored.size).toBe(2);
    expect(lookup(restored, 'elderberry')).toEqual({ n: 'elderberry' });
  });

  it('FuzzyObjectIndex can be destroyed twice and serialized while empty', () => {
    const index = new FuzzyObjectIndex(fruits, { keys: ['n'] });
    index.destroy();
    index.destroy();
    const restored = FuzzyObjectIndex.deserialize<Fruit>(index.serialize());
    expect(restored.size).toBe(0);
    restored.add({ n: 'fig' });
    expect(lookup(restored, 'fig')).toEqual({ n: 'fig' });
  });

  it('KeyedFuzzyIndex keeps its keys and accepts new items', () => {
    const index = new KeyedFuzzyIndex([['apple'], ['red']], [2, 1]);
    index.destroy();
    expect(index.size).toBe(0);
    expect(index.search('apple')).toEqual([]);
    index.add(['banana', 'yellow']);
    index.addMany([['cherry', 'red']]);
    expect(index.size).toBe(2);
    expect(index.search('cherry')[0]?.index).toBe(1);
    expect(() => index.add(['only one'])).toThrow();
    expect(KeyedFuzzyIndex.deserialize(index.serialize()).size).toBe(2);
  });

  it('matches FuzzyIndex, which also accepts items after destroy()', () => {
    const index = new FuzzyIndex(['apple']);
    index.destroy();
    index.add('banana');
    expect(index.size).toBe(1);
  });
});
