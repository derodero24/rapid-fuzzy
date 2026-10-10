/**
 * VitePress theme integration for the rapid-fuzzy search plugin.
 *
 * Copy these snippets into your .vitepress/theme/index.ts to replace
 * VitePress's built-in local search with rapid-fuzzy. The import paths assume
 * a .vitepress/ directory at the root of this repository; adjust them to
 * where you copy plugin.ts and SearchBox.vue.
 *
 * Step 1 — add the Vite plugin to .vitepress/config.ts. Leave
 * themeConfig.search unset: `provider: 'local'` would turn VitePress's
 * built-in search on next to this one.
 *
 *   import { fileURLToPath } from 'node:url'
 *   import { defineConfig } from 'vitepress'
 *   import { rapidFuzzySearch } from '../examples/vitepress-plugin/plugin'
 *
 *   export default defineConfig({
 *     vite: {
 *       // The docs root: the parent of .vitepress/ (`__dirname` here would be
 *       // .vitepress/ itself, which contains no pages)
 *       plugins: [rapidFuzzySearch(fileURLToPath(new URL('..', import.meta.url)))],
 *     },
 *   })
 *
 * Step 2 — extend the default theme in .vitepress/theme/index.ts:
 *
 *   import { h } from 'vue'
 *   import DefaultTheme from 'vitepress/theme'
 *   import SearchBox from '../../examples/vitepress-plugin/SearchBox.vue'
 *
 *   export default {
 *     extends: DefaultTheme,
 *     Layout: () =>
 *       h(DefaultTheme.Layout, null, {
 *         'nav-bar-content-after': () => h(SearchBox),
 *       }),
 *   }
 */

import DefaultTheme from 'vitepress/theme';
import { h } from 'vue';
import SearchBox from './SearchBox.vue';

export default {
  extends: DefaultTheme,
  Layout: () =>
    h(DefaultTheme.Layout, null, {
      'nav-bar-content-after': () => h(SearchBox),
    }),
};
