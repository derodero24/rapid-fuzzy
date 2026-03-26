/**
 * rapidFuzzySearch — VitePress Vite plugin
 *
 * At build time this plugin:
 *  1. Walks the docs directory and reads all .md files
 *  2. Extracts frontmatter title and plain text from each page
 *  3. Builds a KeyedFuzzyIndex (title weight=2, body weight=1)
 *  4. Writes search-index.bin and search-manifest.json to the output directory
 *
 * Usage (.vitepress/config.ts):
 *   import { rapidFuzzySearch } from '../examples/vitepress-plugin/plugin'
 *   export default defineConfig({
 *     vite: { plugins: [rapidFuzzySearch('.')] }
 *   })
 */
import { mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { extname, join, relative } from 'node:path';
import { KeyedFuzzyIndex } from 'rapid-fuzzy';
import type { Plugin, ResolvedConfig } from 'vite';

export interface PageMeta {
  url: string;
  title: string;
}

function extractFrontmatterTitle(content: string): string {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (match) {
    const titleLine = match[1].match(/^title:\s*(.+)$/m);
    if (titleLine) return titleLine[1].trim().replace(/^['"]|['"]$/g, '');
  }
  return '';
}

function extractTitle(content: string): string {
  const fmTitle = extractFrontmatterTitle(content);
  if (fmTitle) return fmTitle;
  const headingMatch = content.match(/^#\s+(.+)/m);
  return headingMatch ? headingMatch[1].trim() : '';
}

function extractText(content: string): string {
  return (
    content
      // Remove frontmatter
      .replace(/^---[\s\S]*?---\r?\n/, '')
      // Remove fenced code blocks
      .replace(/```[\s\S]*?```/g, '')
      // Remove inline code
      .replace(/`[^`]+`/g, '')
      // Images
      .replace(/!\[.*?\]\(.*?\)/g, '')
      // Links → keep text
      .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
      // Heading markers
      .replace(/^#{1,6}\s+/gm, '')
      // Bold / italic / strikethrough markers
      .replace(/[*_~]{1,3}/g, '')
      // Blockquotes
      .replace(/^\s*>\s*/gm, '')
      // HTML tags
      .replace(/<[^>]+>/g, '')
      // Collapse whitespace
      .replace(/\s+/g, ' ')
      .trim()
  );
}

function mdFileToUrl(filePath: string, docsDir: string): string {
  const rel = relative(docsDir, filePath).replace(/\\/g, '/');
  const withoutExt = rel.replace(/\.md$/, '');
  if (withoutExt === 'index') return '/';
  if (withoutExt.endsWith('/index')) return `/${withoutExt.slice(0, -'/index'.length)}/`;
  return `/${withoutExt}`;
}

function collectMdFiles(dir: string, results: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    if (entry.startsWith('.') || entry === 'node_modules') continue;
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      collectMdFiles(full, results);
    } else if (extname(entry) === '.md' && !entry.startsWith('_')) {
      results.push(full);
    }
  }
  return results;
}

/**
 * Create the rapid-fuzzy VitePress search plugin.
 *
 * @param docsDir - Absolute path to the VitePress docs root (the directory
 *                  containing `.vitepress/`). Usually `__dirname` of your
 *                  `config.ts`.
 */
export function rapidFuzzySearch(docsDir: string): Plugin {
  let resolvedConfig: ResolvedConfig;

  return {
    name: 'vitepress-plugin-rapid-fuzzy-search',
    apply: 'build',

    configResolved(config) {
      resolvedConfig = config;
    },

    closeBundle() {
      const files = collectMdFiles(docsDir);
      const meta: PageMeta[] = [];
      const titles: string[] = [];
      const texts: string[] = [];

      for (const file of files) {
        const content = readFileSync(file, 'utf-8');
        const title = extractTitle(content);
        meta.push({ url: mdFileToUrl(file, docsDir), title });
        titles.push(title);
        texts.push(extractText(content));
      }

      // Two-key index: key 0 = title (weight 2), key 1 = body text (weight 1)
      const index = new KeyedFuzzyIndex([titles, texts], [2, 1]);
      const serialized = index.serialize();

      const outDir = resolvedConfig.build.outDir;
      mkdirSync(outDir, { recursive: true });
      writeFileSync(join(outDir, 'search-index.bin'), serialized);
      writeFileSync(join(outDir, 'search-manifest.json'), JSON.stringify(meta));
    },
  };
}
