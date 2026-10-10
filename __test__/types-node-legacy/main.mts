// Type-checks the Node.js declarations the way a strict Node.js project sees
// them with an older @types/node release, which the repo's own tsconfig (the
// current @types/node) cannot: the package is imported by its own name, with
// skipLibCheck on as in most projects.
//
// - tsconfig.json: @types/node 22.10.7. Its `Buffer` is generic, and the last
//   overload of its constructor returns `Buffer<ArrayBuffer>` (as in the 20.17
//   to 22.14 releases), so `serialize()` must not be typed from the
//   constructor's instance type, which would make it `Buffer<ArrayBuffer>`
//   instead of `Buffer`.
// - tsconfig.node20.11.json: @types/node 20.11.30. Its `Buffer` is not
//   generic and, with TypeScript 5.7 or later, not assignable to `Uint8Array`,
//   so `deserialize()` must accept a `Buffer` as such.
//
// Run by `pnpm run typecheck`; never executed.
import { readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { FuzzyIndex, KeyedFuzzyIndex, type NodeBuffer } from 'rapid-fuzzy';
import { FuzzyObjectIndex } from 'rapid-fuzzy/objects';

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;

// ─── serialize() returns exactly Node.js's Buffer ───────────────────────────

export type SerializeReturnsBuffer = [
  Expect<Equal<NodeBuffer, Buffer>>,
  Expect<Equal<ReturnType<FuzzyIndex['serialize']>, Buffer>>,
  Expect<Equal<ReturnType<KeyedFuzzyIndex['serialize']>, Buffer>>,
  Expect<Equal<ReturnType<FuzzyObjectIndex<{ name: string }>['serialize']>, Buffer>>,
];

/** Cache the serialized bytes, or reuse bytes written to disk earlier. */
export async function cache(path: string): Promise<number> {
  let bytes = new FuzzyIndex(['apple']).serialize();
  bytes = readFileSync(path);
  bytes = await readFile(path);
  bytes = new KeyedFuzzyIndex([['apple']], [1]).serialize();
  bytes = new FuzzyObjectIndex([{ name: 'apple' }], { keys: ['name'] }).serialize();
  return bytes.length;
}

// ─── deserialize() takes a Buffer as well as any Uint8Array ─────────────────

export function restore(buffer: Buffer, bytes: Uint8Array): number {
  return (
    FuzzyIndex.deserialize(buffer).size +
    FuzzyIndex.deserialize(readFileSync('index.bin')).size +
    FuzzyIndex.deserialize(bytes).size +
    KeyedFuzzyIndex.deserialize(buffer).size +
    KeyedFuzzyIndex.deserialize(bytes).size +
    FuzzyObjectIndex.deserialize(buffer).size +
    FuzzyObjectIndex.deserialize(bytes).size
  );
}
