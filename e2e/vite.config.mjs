import { defineConfig } from 'vite';

export default defineConfig({
  root: __dirname,
  server: {
    port: 4567,
    strictPort: true,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
