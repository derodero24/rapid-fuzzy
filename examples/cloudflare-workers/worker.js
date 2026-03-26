import { closest, FuzzyIndex } from 'rapid-fuzzy';

// Create the index once at module scope.
// It is shared across all requests in the same isolate, avoiding
// the per-request overhead of rebuilding the index.
const ITEMS = [
  'TypeScript',
  'JavaScript',
  'Python',
  'Rust',
  'Go',
  'TypeSpec',
  'WebAssembly',
  'Cloudflare Workers',
];
const index = new FuzzyIndex(ITEMS);

// biome-ignore lint/style/noDefaultExport: required by Cloudflare Workers
export default {
  /**
   * Handle incoming requests.
   *
   * GET /?q=<query>          → fuzzy search results (JSON array)
   * GET /?q=<query>&closest  → closest single match (JSON string)
   */
  fetch(request) {
    const { searchParams } = new URL(request.url);
    const query = searchParams.get('q') ?? '';

    if (searchParams.has('closest')) {
      const match = closest(query, ITEMS);
      return Response.json(match ?? null);
    }

    const results = index.search(query);
    return Response.json(results);
  },
};
