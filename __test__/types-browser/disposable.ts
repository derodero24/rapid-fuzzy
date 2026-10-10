// With a lib that declares `Symbol.dispose` itself (tsconfig.esnext.json), the
// glue's declaration of it must merge with the lib's, and `using` works.
import { FuzzyIndex } from 'rapid-fuzzy';

export function countWithUsing(items: readonly string[]): number {
  using index = new FuzzyIndex(items);
  return index.search('a').length;
}
