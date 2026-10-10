// Type-level tests that the Node.js declarations match what the binding
// accepts and returns at runtime, and agree with the browser declarations.
// Checked by `pnpm run typecheck` under the repo's strict tsconfig; the
// functions are never called.
import type * as Browser from '../../browser.mjs' with { 'resolution-mode': 'import' };
import type * as Node from '../../index.mjs' with { 'resolution-mode': 'import' };

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? true : false;
type Expect<T extends true> = T;

// ─── Hamming results are numbers or null, never undefined ──────────────────

export type HammingResults = [
  Expect<Equal<ReturnType<typeof Node.hammingBatch>, Array<number | null>>>,
  Expect<Equal<ReturnType<typeof Node.hammingMany>, Array<number | null>>>,
  Expect<Equal<ReturnType<typeof Node.normalizedHammingBatch>, Array<number | null>>>,
  Expect<Equal<ReturnType<typeof Node.normalizedHammingMany>, Array<number | null>>>,
  // Shared Node.js / browser code can use one result type.
  Expect<Equal<ReturnType<typeof Browser.hammingMany>, (number | null)[]>>,
  Expect<Equal<ReturnType<typeof Browser.normalizedHammingBatch>, (number | null)[]>>,
];

export function hammingResults(n: typeof Node, b: typeof Browser): Array<number | null> {
  const fromNode: Array<number | null> = n.hammingMany('kitten', ['sitten']);
  const fromBrowser: Array<number | null> = b.hammingMany('kitten', ['sitten']);
  return [...fromNode, ...fromBrowser];
}

// ─── serialize() returns exactly Node.js's Buffer when @types/node is loaded ─
// (`NodeBuffer` falls back to `Uint8Array` in programs without the Node.js
// types; __test__/types-browser checks that case, and
// __test__/types-node-legacy older @types/node releases.)

export type SerializeReturnsBuffer = [
  Expect<Equal<Node.NodeBuffer, Buffer>>,
  Expect<Equal<ReturnType<Node.FuzzyIndex['serialize']>, Buffer>>,
  Expect<Equal<ReturnType<Node.KeyedFuzzyIndex['serialize']>, Buffer>>,
  Expect<Equal<ReturnType<Node.FuzzyObjectIndex<unknown>['serialize']>, Buffer>>,
];

export function bufferMethods(index: Node.FuzzyIndex): string {
  return index.serialize().toString('base64');
}

// ─── deserialize() takes any Uint8Array, not only a Buffer ──────────────────

export function deserializeUint8Array(n: typeof Node, bytes: Uint8Array): number {
  // Data read from fetch(), IndexedDB or the browser build's serialize().
  const index = n.FuzzyIndex.deserialize(bytes);
  const keyed = n.KeyedFuzzyIndex.deserialize(bytes);
  const fromBuffer = n.FuzzyIndex.deserialize(Buffer.from(bytes));
  return index.size + keyed.size + fromBuffer.size;
}
