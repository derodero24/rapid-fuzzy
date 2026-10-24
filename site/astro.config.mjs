import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import starlightLinksValidator from 'starlight-links-validator';
import starlightLlmsTxt from 'starlight-llms-txt';
import starlightTypeDoc, { typeDocSidebarGroup } from 'starlight-typedoc';

const repository = 'https://github.com/derodero24/rapid-fuzzy';

export default defineConfig({
  site: 'https://derodero24.github.io',
  base: '/rapid-fuzzy',
  trailingSlash: 'always',
  integrations: [
    starlight({
      title: 'rapid-fuzzy',
      description:
        'Fuzzy search and string similarity for JavaScript and TypeScript, powered by Rust: native Node.js bindings and a WebAssembly build for browsers, Deno, Bun and Cloudflare Workers.',
      logo: { src: './src/assets/logo.svg', alt: 'rapid-fuzzy' },
      favicon: '/favicon.svg',
      social: [
        { icon: 'github', label: 'GitHub', href: repository },
        { icon: 'npm', label: 'npm', href: 'https://www.npmjs.com/package/rapid-fuzzy' },
      ],
      editLink: { baseUrl: `${repository}/edit/develop/site/` },
      lastUpdated: true,
      customCss: ['./src/styles/custom.css'],
      head: [
        {
          tag: 'meta',
          attrs: {
            property: 'og:image',
            content: 'https://derodero24.github.io/rapid-fuzzy/og.png',
          },
        },
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
      ],
      plugins: [
        starlightTypeDoc({
          entryPoints: ['node_modules/rapid-fuzzy/index.d.ts'],
          tsconfig: './typedoc.tsconfig.json',
          sidebar: { label: 'API reference', collapsed: true },
          typeDoc: {
            disableSources: true,
            readme: 'none',
            parametersFormat: 'table',
            interfacePropertiesFormat: 'table',
            enumMembersFormat: 'table',
          },
        }),
        starlightLlmsTxt({
          projectName: 'rapid-fuzzy',
          description:
            'rapid-fuzzy is a fuzzy search and string similarity library for JavaScript and TypeScript with a Rust core: native Node.js bindings (napi-rs) and a WebAssembly build for browsers, Deno, Bun and Cloudflare Workers.',
        }),
        starlightLinksValidator({ errorOnLocalLinks: false }),
      ],
      sidebar: [
        {
          label: 'Start here',
          items: [
            { label: 'Getting started', slug: 'getting-started' },
            { label: 'Playground', slug: 'playground' },
            { label: 'Why rapid-fuzzy?', slug: 'comparison' },
            { label: 'Benchmarks', slug: 'benchmarks' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Fuzzy search', slug: 'guides/fuzzy-search' },
            { label: 'How matching works', slug: 'guides/matching-and-scoring' },
            { label: 'Object search', slug: 'guides/object-search' },
            { label: 'Persistent indexes', slug: 'guides/indexes' },
            { label: 'Highlighting matches', slug: 'guides/highlighting' },
            { label: 'String similarity', slug: 'guides/string-similarity' },
            { label: 'Choosing an algorithm', slug: 'guides/choosing-an-algorithm' },
            { label: 'UI frameworks', slug: 'guides/ui-frameworks' },
            { label: 'Unicode and non-Latin scripts', slug: 'guides/unicode' },
            { label: 'Performance and untrusted input', slug: 'guides/performance' },
            { label: 'Errors and edge cases', slug: 'guides/errors' },
          ],
        },
        {
          label: 'Runtimes',
          items: [
            { label: 'Node.js and SSR', slug: 'runtimes/node' },
            { label: 'Browsers and bundlers', slug: 'runtimes/browsers' },
            { label: 'Cloudflare Workers', slug: 'runtimes/cloudflare-workers' },
            { label: 'Deno and Bun', slug: 'runtimes/deno-and-bun' },
            { label: 'Troubleshooting', slug: 'runtimes/troubleshooting' },
          ],
        },
        {
          label: 'Migration guides',
          collapsed: true,
          items: [{ autogenerate: { directory: 'migration' } }],
        },
        typeDocSidebarGroup,
      ],
    }),
  ],
});
