// Node.js module customization hooks that load the Cloudflare Workers entry
// (workerd.mjs) the way Wrangler bundles it, for e2e/workerd-smoke.mjs.
// Registered by e2e/workerd-register.mjs (`pnpm run test:workerd`).
import { fileURLToPath } from 'node:url';

const PACKAGE_JSON = new URL('../package.json', import.meta.url).href;

/**
 * Resolve `rapid-fuzzy` and its subpaths as the package in this repository,
 * as if it were installed, also from modules outside the package such as
 * examples/cloudflare-workers/worker.js (which has no node_modules). The
 * package's export conditions apply as usual, so `--conditions=workerd`
 * selects workerd.mjs.
 */
export function resolve(specifier, context, nextResolve) {
  if (specifier === 'rapid-fuzzy' || specifier.startsWith('rapid-fuzzy/')) {
    return nextResolve(specifier, { ...context, parentURL: PACKAGE_JSON });
  }
  return nextResolve(specifier, context);
}

/**
 * `import wasmModule from './x.wasm?module'`: Wrangler bundles such an import
 * as a precompiled WebAssembly.Module (Workers cannot compile WebAssembly
 * from bytes at runtime), so the default export is a module compiled from
 * the file.
 */
export function load(url, context, nextLoad) {
  const target = new URL(url);
  if (
    target.protocol === 'file:' &&
    target.search === '?module' &&
    target.pathname.endsWith('.wasm')
  ) {
    target.search = '';
    const path = JSON.stringify(fileURLToPath(target));
    return {
      format: 'module',
      shortCircuit: true,
      source: [
        "import { readFileSync } from 'node:fs';",
        `export default new WebAssembly.Module(readFileSync(${path}));`,
        '',
      ].join('\n'),
    };
  }
  return nextLoad(url, context);
}
