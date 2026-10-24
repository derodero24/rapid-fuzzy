// Web server for e2e/browser.spec.ts (started by playwright.config.ts).
//
// Packs this repository with `npm pack`, installs the tarball into a copy of
// e2e/browser-app (under node_modules/.cache, outside the package itself) and
// serves that consumer app with Vite, the way users consume the package:
//   http://localhost:4567  production build (`vite build` + `vite preview`)
//   http://localhost:4568  dev server (dependency pre-bundling)
// Requires the wasm-bindgen binary (`pnpm run build:wasm-bindgen`).
import { execFileSync } from 'node:child_process';
import { cpSync, existsSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { build, createServer, preview } from 'vite';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const APP = join(ROOT, 'node_modules', '.cache', 'rapid-fuzzy-e2e');
const WASM = join(ROOT, 'rapid-fuzzy-wasm-bindgen_bg.wasm');

if (!existsSync(WASM)) {
  throw new Error(`${WASM} is missing: run \`pnpm run build:wasm-bindgen\` first`);
}

rmSync(APP, { recursive: true, force: true });
cpSync(join(ROOT, 'e2e', 'browser-app'), APP, { recursive: true });
writeFileSync(
  join(APP, 'package.json'),
  `${JSON.stringify({ name: 'rapid-fuzzy-e2e-app', private: true, type: 'module' }, null, 2)}\n`,
);

const packDir = mkdtempSync(join(tmpdir(), 'rapid-fuzzy-pack-'));
try {
  // npm 10 still runs the "prepare" script (lefthook install) here despite
  // --ignore-scripts; that needs a git checkout, as `pnpm install` does.
  execFileSync(
    'npm',
    ['pack', '--ignore-scripts', '--loglevel=warn', '--pack-destination', packDir],
    {
      cwd: ROOT,
      stdio: ['ignore', 'ignore', 'inherit'],
    },
  );
  const tarball = readdirSync(packDir).find((file) => file.endsWith('.tgz'));
  if (!tarball) throw new Error('npm pack produced no tarball');
  execFileSync(
    'npm',
    [
      'install',
      '--no-audit',
      '--no-fund',
      '--ignore-scripts',
      '--no-package-lock',
      '--loglevel=warn',
      join(packDir, tarball),
    ],
    { cwd: APP, stdio: ['ignore', 'ignore', 'inherit'] },
  );
} finally {
  rmSync(packDir, { recursive: true, force: true });
}

// No Vite configuration: the package must work with the defaults.
const config = { root: APP, configFile: false, logLevel: 'warn' };
await build(config);
const dev = await createServer({ ...config, server: { port: 4568, strictPort: true } });
await dev.listen();
await preview({ ...config, preview: { port: 4567, strictPort: true } });
console.log('rapid-fuzzy e2e app: build on http://localhost:4567, dev on http://localhost:4568');
