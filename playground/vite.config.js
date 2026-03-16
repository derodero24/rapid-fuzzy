import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { defineConfig } from 'vite';

// When running locally from the repo (after `pnpm run build:wasm`),
// aliases resolve to the local WASM build + highlight utilities.
// On StackBlitz or clean checkouts, aliases are skipped and imports
// resolve to the `rapid-fuzzy` npm package directly.
const root = resolve(__dirname, '..');
const localWasmEntry = resolve(root, 'rapid-fuzzy.wasi-browser.js');
const useLocalAliases = existsSync(localWasmEntry);

export default defineConfig({
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  resolve: {
    alias: useLocalAliases
      ? {
          'rapid-fuzzy': resolve(__dirname, 'rapid-fuzzy-local.js'),
          'rapid-fuzzy-wasm32-wasi': localWasmEntry,
        }
      : {},
  },
});
