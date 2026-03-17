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

const corsHeaders = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

export default defineConfig({
  base: process.env.GITHUB_ACTIONS ? '/rapid-fuzzy/' : '/',
  build: {
    // WASM entry uses top-level await (requires es2022+)
    target: 'es2022',
  },
  server: {
    headers: corsHeaders,
  },
  preview: {
    headers: corsHeaders,
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
