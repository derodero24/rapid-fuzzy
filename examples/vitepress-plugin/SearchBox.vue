<!--
  SearchBox.vue — VitePress search component backed by rapid-fuzzy

  Loads the pre-built KeyedFuzzyIndex from /search-index.bin and
  /search-manifest.json at mount time, then queries it on every keystroke.

  Usage (.vitepress/theme/index.ts):
    import { h } from 'vue'
    import DefaultTheme from 'vitepress/theme'
    import SearchBox from '../../examples/vitepress-plugin/SearchBox.vue'

    export default {
      extends: DefaultTheme,
      Layout: () => h(DefaultTheme.Layout, null, {
        'nav-bar-content-after': () => h(SearchBox),
      }),
    }
-->
<script setup lang="ts">
import { useRouter, withBase } from 'vitepress';
import { onMounted, ref, watch } from 'vue';

import type { PageMeta } from './plugin.ts';

// rapid-fuzzy is imported dynamically from the CDN so no bundler configuration
// is needed for the WASM binary. esm.sh resolves the `browser` export condition
// automatically and serves the wasm-bindgen build.
const RAPID_FUZZY_CDN = 'https://esm.sh/rapid-fuzzy';

interface WasmFuzzyResult {
  index: number;
  score: number;
}

interface WasmKeyedFuzzyIndex {
  search(query: string, options?: { maxResults?: number }): WasmFuzzyResult[];
}

interface WasmModule {
  KeyedFuzzyIndex: {
    deserialize(data: Uint8Array): WasmKeyedFuzzyIndex;
  };
}

const router = useRouter();
const query = ref('');
const results = ref<PageMeta[]>([]);
const open = ref(false);

let fuzzyIndex: WasmKeyedFuzzyIndex | null = null;
let manifest: PageMeta[] = [];

onMounted(async () => {
  const [mod, manifestRes, binRes] = await Promise.all([
    import(/* @vite-ignore */ RAPID_FUZZY_CDN) as Promise<WasmModule>,
    fetch(withBase('/search-manifest.json')).then((r) => r.json() as Promise<PageMeta[]>),
    fetch(withBase('/search-index.bin')),
  ]);

  manifest = manifestRes;
  const buf = await binRes.arrayBuffer();
  fuzzyIndex = mod.KeyedFuzzyIndex.deserialize(new Uint8Array(buf));
});

watch(query, (q) => {
  if (!fuzzyIndex || !q.trim()) {
    results.value = [];
    open.value = false;
    return;
  }
  const hits = fuzzyIndex.search(q, { maxResults: 8 });
  results.value = hits.map((h) => manifest[h.index]).filter(Boolean);
  open.value = results.value.length > 0;
});

// biome-ignore lint/correctness/noUnusedVariables: used in Vue template
function navigate(url: string) {
  router.go(url);
  query.value = '';
  open.value = false;
}

// biome-ignore lint/correctness/noUnusedVariables: used in Vue template
function onBlur() {
  // Delay so click on a result fires before hiding the list
  setTimeout(() => {
    open.value = false;
  }, 150);
}
</script>

<template>
  <div class="rf-search">
    <input
      v-model="query"
      type="search"
      class="rf-search__input"
      placeholder="Search..."
      aria-label="Search"
      aria-autocomplete="list"
      :aria-expanded="open"
      @focus="open = results.length > 0"
      @blur="onBlur"
    />
    <ul v-if="open" class="rf-search__results" role="listbox">
      <li
        v-for="page in results"
        :key="page.url"
        class="rf-search__result"
        role="option"
        @mousedown.prevent="navigate(page.url)"
      >
        <span class="rf-search__title">{{ page.title || page.url }}</span>
        <span class="rf-search__url">{{ page.url }}</span>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.rf-search {
  position: relative;
  display: inline-block;
}

.rf-search__input {
  padding: 4px 10px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 4px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  font-size: 14px;
  width: 200px;
  outline: none;
}

.rf-search__input:focus {
  border-color: var(--vp-c-brand-1);
}

.rf-search__results {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  min-width: 280px;
  background: var(--vp-c-bg-elv);
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  list-style: none;
  margin: 0;
  padding: 4px 0;
  z-index: 100;
}

.rf-search__result {
  display: flex;
  flex-direction: column;
  padding: 8px 14px;
  cursor: pointer;
  gap: 2px;
}

.rf-search__result:hover {
  background: var(--vp-c-default-soft);
}

.rf-search__title {
  font-size: 14px;
  color: var(--vp-c-text-1);
}

.rf-search__url {
  font-size: 11px;
  color: var(--vp-c-text-3);
}
</style>
