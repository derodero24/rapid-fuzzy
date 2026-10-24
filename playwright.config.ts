import { defineConfig } from '@playwright/test';

// e2e/serve-browser-app.mjs installs the packed package into a Vite consumer app
// and serves its production build (:4567) and dev server (:4568).
export default defineConfig({
  testDir: './e2e',
  testMatch: 'browser.spec.ts',
  timeout: 60_000,
  retries: 1,
  projects: [
    {
      name: 'vite-build',
      use: { browserName: 'chromium', baseURL: 'http://localhost:4567' },
    },
    {
      name: 'vite-dev',
      use: { browserName: 'chromium', baseURL: 'http://localhost:4568' },
    },
  ],
  webServer: {
    command: 'node e2e/serve-browser-app.mjs',
    port: 4567,
    timeout: 180_000,
    reuseExistingServer: !process.env.CI,
  },
});
