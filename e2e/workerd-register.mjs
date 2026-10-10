// Preloaded with `node --import` by `pnpm run test:workerd`: installs the hooks
// that let Node.js run the Cloudflare Workers entry (see e2e/workerd-hooks.mjs).
// module.registerHooks() (Node.js 22.15+ and 23.5+) runs them in this thread;
// module.register(), which Node.js 26 deprecates (DEP0205), is only the
// fallback for earlier Node.js 22 releases.
import nodeModule from 'node:module';
import { load, resolve } from './workerd-hooks.mjs';

if (typeof nodeModule.registerHooks === 'function') {
  nodeModule.registerHooks({ resolve, load });
} else {
  nodeModule.register('./workerd-hooks.mjs', import.meta.url);
}
