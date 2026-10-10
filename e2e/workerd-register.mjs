// Preloaded with `node --import` by `pnpm run test:workerd`: installs the hooks
// that let Node.js run the Cloudflare Workers entry (see e2e/workerd-hooks.mjs).
import { register } from 'node:module';

register('./workerd-hooks.mjs', import.meta.url);
