import { resolve } from 'node:path';
import { defineConfig } from 'vite';

// When running locally from the repo (after `pnpm run build:wasm`),
// aliases resolve to the local WASM build + highlight utilities.
// On StackBlitz, remove the `resolve.alias` block — imports resolve
// to the `rapid-fuzzy` npm package directly.
const root = resolve(__dirname, '..');

export default defineConfig({
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  resolve: {
    alias: {
      'rapid-fuzzy': resolve(__dirname, 'rapid-fuzzy-local.js'),
      'rapid-fuzzy-wasm32-wasi': resolve(root, 'rapid-fuzzy.wasi-browser.js'),
    },
  },
});
