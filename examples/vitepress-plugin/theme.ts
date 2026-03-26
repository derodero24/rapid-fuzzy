/**
 * VitePress theme integration for the rapid-fuzzy search plugin.
 *
 * Copy these snippets into your .vitepress/theme/index.ts to replace
 * VitePress's built-in local search with rapid-fuzzy.
 *
 * Step 1 — add the Vite plugin to .vitepress/config.ts:
 *
 *   import { defineConfig } from 'vitepress'
 *   import { rapidFuzzySearch } from 'vitepress-plugin-rapid-fuzzy/plugin'
 *
 *   export default defineConfig({
 *     // Disable the built-in local search
 *     themeConfig: { search: { provider: 'local', options: {} } },
 *     vite: {
 *       plugins: [rapidFuzzySearch(__dirname)],
 *     },
 *   })
 *
 * Step 2 — extend the default theme in .vitepress/theme/index.ts:
 *
 *   import { h } from 'vue'
 *   import DefaultTheme from 'vitepress/theme'
 *   import SearchBox from 'vitepress-plugin-rapid-fuzzy/SearchBox.vue'
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
