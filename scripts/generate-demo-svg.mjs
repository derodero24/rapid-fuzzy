#!/usr/bin/env node
/**
 * Generate a terminal-style SVG showing rapid-fuzzy demo output.
 * Usage: node scripts/generate-demo-svg.mjs
 */

import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const outPath = join(__dirname, '..', '.github', 'assets', 'demo.svg');

const lineHeight = 20;
const padding = { top: 48, left: 16, right: 16, bottom: 16 };
const titleBarHeight = 32;

// Terminal content lines with color codes
// Colors: dim=#6c7086, cyan=#89dceb, green=#a6e3a1, text=#cdd6f4, yellow=#f9e2af
const lines = [
  { text: '$ node -e "import(\'rapid-fuzzy\').then(rf => ...)"', color: '#6c7086' },
  { text: '' },
  { text: '▸ Fuzzy Search', color: '#89dceb', bold: true },
  { text: '' },
  { text: "> search('typscript', items)", color: '#6c7086' },
  { text: '  TypeScript     score: 0.86', color: '#cdd6f4' },
  { text: '' },
  { text: '▸ Query Syntax — exclude + prefix', color: '#89dceb', bold: true },
  { text: '' },
  { text: "> search('^java !script', items)", color: '#6c7086' },
  { text: '  Java           score: 1.00', color: '#cdd6f4' },
  { text: '' },
  { text: '▸ FuzzyIndex — persistent index', color: '#89dceb', bold: true },
  { text: '' },
  { text: '> const index = new FuzzyIndex(items)', color: '#6c7086' },
  { text: "  search('rust') → Rust         0.062ms", color: '#cdd6f4' },
  { text: "  search('swif') → Swift        0.016ms", color: '#cdd6f4' },
  { text: "  search('hask') → Haskell      0.012ms", color: '#cdd6f4' },
  { text: '' },
  { text: '▸ String Distance', color: '#89dceb', bold: true },
  { text: '' },
  { text: "> levenshtein('kitten', 'sitting')", color: '#6c7086' },
  { text: '  3 edits', color: '#cdd6f4' },
  { text: "> jaroWinkler('MARTHA', 'MARHTA')", color: '#6c7086' },
  { text: '  0.961 similarity', color: '#cdd6f4' },
];

const width = 580;
const contentHeight = lines.length * lineHeight;
const height = titleBarHeight + padding.top - titleBarHeight + contentHeight + padding.bottom;

function escapeXml(s) {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/'/g, '&apos;');
}

const svgLines = [];
svgLines.push(
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">`,
);
svgLines.push('<style>');
svgLines.push(
  '  text { font-family: "JetBrains Mono", "SF Mono", "Fira Code", "Cascadia Code", monospace; font-size: 13px; }',
);
svgLines.push('</style>');

// Background with rounded corners
svgLines.push(`<rect width="${width}" height="${height}" rx="8" fill="#1e1e2e" />`);

// Title bar
svgLines.push(`<rect width="${width}" height="${titleBarHeight}" rx="8" fill="#181825" />`);
svgLines.push(`<rect y="${titleBarHeight - 1}" width="${width}" height="1" fill="#313244" />`);

// Window controls (traffic lights)
svgLines.push('<circle cx="20" cy="16" r="6" fill="#f38ba8" />');
svgLines.push('<circle cx="38" cy="16" r="6" fill="#f9e2af" />');
svgLines.push('<circle cx="56" cy="16" r="6" fill="#a6e3a1" />');

// Title text
svgLines.push(
  `<text x="${width / 2}" y="20" text-anchor="middle" fill="#6c7086">rapid-fuzzy</text>`,
);

// Content
let y = titleBarHeight + 20;
for (const line of lines) {
  if (line.text) {
    const fill = line.color || '#cdd6f4';
    const weight = line.bold ? ' font-weight="600"' : '';
    svgLines.push(
      `<text x="${padding.left}" y="${y}" fill="${fill}"${weight}>${escapeXml(line.text)}</text>`,
    );
  }
  y += lineHeight;
}

svgLines.push('</svg>');

const svg = svgLines.join('\n');
writeFileSync(outPath, svg);
console.log(`✓ Generated ${outPath} (${(svg.length / 1024).toFixed(1)} KB)`);
