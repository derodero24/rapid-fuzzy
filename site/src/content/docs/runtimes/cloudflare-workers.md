---
title: Cloudflare Workers
description: Run rapid-fuzzy in Cloudflare Workers, where Wrangler bundles the WebAssembly build through the workerd export condition.
---

Install the package and import it in your Worker. Wrangler resolves the `workerd` export condition and bundles the WebAssembly binary as a precompiled module. No `nodejs_compat` flag is needed.

```sh
npm install rapid-fuzzy
```

```js
// worker.js
import { FuzzyIndex } from 'rapid-fuzzy';

// Build the index once at module scope; it is shared across requests
const index = new FuzzyIndex(['hello', 'world', 'foo', 'bar']);

export default {
  fetch(request) {
    const query = new URL(request.url).searchParams.get('q') ?? '';
    return Response.json(index.search(query.slice(0, 200), 10));
  },
};
```

```toml
# wrangler.toml
name = "rapid-fuzzy-worker"
main = "worker.js"
compatibility_date = "2025-01-01"
```

Cap the query length, as above: Workers limit CPU time per request, and every function is synchronous (see [Performance and untrusted input](/rapid-fuzzy/guides/performance/)).

For type-checking, add `"customConditions": ["workerd"]` to `tsconfig.json`. See [`examples/cloudflare-workers`](https://github.com/derodero24/rapid-fuzzy/tree/develop/examples/cloudflare-workers) for a complete project, tested with Wrangler 4.
