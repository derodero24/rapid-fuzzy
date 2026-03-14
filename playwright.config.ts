import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  testMatch: 'browser.spec.ts',
  timeout: 60_000,
  retries: 1,
  use: {
    baseURL: 'http://localhost:4567',
  },
  projects: [
    {
      name: 'chromium',
      use: { browserName: 'chromium' },
    },
  ],
  webServer: {
    command: 'npx vite --config e2e/vite.config.mjs',
    port: 4567,
    reuseExistingServer: !process.env.CI,
  },
});
