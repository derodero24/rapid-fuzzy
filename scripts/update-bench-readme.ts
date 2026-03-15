#!/usr/bin/env npx tsx
/**
 * Run benchmarks and update README.md with the latest numbers.
 *
 * Usage:
 *   pnpm bench:readme          # run benchmarks and update README
 *   npx tsx scripts/update-bench-readme.ts
 */

import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';

// ---------------------------------------------------------------------------
// 1. Run benchmarks and capture output
// ---------------------------------------------------------------------------

console.log('Running benchmarks…');
const raw = execSync('pnpm run bench', {
  encoding: 'utf-8',
  timeout: 600_000,
  env: { ...process.env, FORCE_COLOR: '0' },
  stdio: ['pipe', 'pipe', 'pipe'],
});

// Strip ANSI escape codes
// biome-ignore lint/suspicious/noControlCharactersInRegex: ANSI escape stripping requires matching ESC (0x1B)
const clean = raw.replace(/\x1b\[[0-9;]*m/g, '');

// ---------------------------------------------------------------------------
// 2. Parse results — extract (name, hz) from each "· name  hz …" line
// ---------------------------------------------------------------------------

interface BenchEntry {
  name: string;
  hz: number;
}

const entries: BenchEntry[] = [];
for (const line of clean.split('\n')) {
  //  "   · rapid-fuzzy  179,371.64  …"
  const m = line.match(/^\s+·\s+(.+?)\s{2,}([\d,]+(?:\.\d+)?)\s/);
  if (m) {
    entries.push({ name: m[1].trim(), hz: Number(m[2].replace(/,/g, '')) });
  }
}

// Group by benchmark suite (detect suite headers)
interface Suite {
  suite: string;
  results: Map<string, number>;
}

const suites: Suite[] = [];
let currentSuite = '';
let currentMap = new Map<string, number>();

for (const line of clean.split('\n')) {
  // "  ✓ __test__/search.bench.ts > Fuzzy Search — Small (20 items)"
  const sm = line.match(/[✓]\s+\S+\s+>\s+(.+?)(?:\s+\d+ms)?$/);
  if (sm) {
    if (currentSuite) {
      suites.push({ suite: currentSuite, results: currentMap });
    }
    currentSuite = sm[1].trim();
    currentMap = new Map();
    continue;
  }
  const em = line.match(/^\s+·\s+(.+?)\s{2,}([\d,]+(?:\.\d+)?)\s/);
  if (em && currentSuite) {
    currentMap.set(em[1].trim(), Number(em[2].replace(/,/g, '')));
  }
}
if (currentSuite) {
  suites.push({ suite: currentSuite, results: currentMap });
}

// ---------------------------------------------------------------------------
// 3. Build markdown tables
// ---------------------------------------------------------------------------

function fmt(n: number): string {
  return Math.round(n).toLocaleString('en-US');
}

function best(vals: (number | null)[]): number {
  return Math.max(...vals.filter((v): v is number => v !== null));
}

function cell(val: number | null, isBest: boolean): string {
  if (val === null) return '—';
  const s = `${fmt(val)} ops/s`;
  return isBest ? `**${s}**` : s;
}

function findSuite(name: string): Map<string, number> | undefined {
  return suites.find((s) => s.suite === name)?.results;
}

// Distance table
const distLines: string[] = [];
distLines.push('| Function | rapid-fuzzy | fastest-levenshtein | leven | string-similarity |');
distLines.push('|---|---:|---:|---:|---:|');

const levSuite = findSuite('Levenshtein Distance');
const normSuite = findSuite('Normalized Similarity');
const jaroSuite = findSuite('Jaro / Jaro-Winkler');
const damerauSuite = findSuite('Damerau-Levenshtein');

if (levSuite) {
  const rf = levSuite.get('rapid-fuzzy') ?? null;
  const fl = levSuite.get('fastest-levenshtein') ?? null;
  const lv = levSuite.get('leven') ?? null;
  const b = best([rf, fl, lv]);
  distLines.push(
    `| Levenshtein | ${cell(rf, rf === b)} | ${cell(fl, fl === b)} | ${cell(lv, lv === b)} | — |`,
  );
}
if (normSuite) {
  const nl = normSuite.get('rapid-fuzzy (normalizedLevenshtein)') ?? null;
  distLines.push(`| Normalized Levenshtein | ${cell(nl, true)} | — | — | — |`);

  const sd = normSuite.get('rapid-fuzzy (sorensenDice)') ?? null;
  const ss = normSuite.get('string-similarity (compareTwoStrings / Dice)') ?? null;
  const b = best([sd, ss]);
  distLines.push(
    `| Sorensen-Dice | ${cell(sd, sd !== null && sd >= b)} | — | — | ${cell(ss, ss !== null && ss >= b)} |`,
  );
}
if (jaroSuite) {
  const jw = jaroSuite.get('rapid-fuzzy (jaroWinkler)') ?? null;
  distLines.push(`| Jaro-Winkler | ${cell(jw, true)} | — | — | — |`);
}
if (damerauSuite) {
  const dl = damerauSuite.get('rapid-fuzzy') ?? null;
  distLines.push(`| Damerau-Levenshtein | ${cell(dl, true)} | — | — | — |`);
}

// Search table (includes FuzzyIndex column)
const searchLines: string[] = [];
searchLines.push('| Dataset size | rapid-fuzzy | rapid-fuzzy (indexed) | fuse.js | fuzzysort |');
searchLines.push('|---|---:|---:|---:|---:|');

for (const size of ['Small (20 items)', 'Medium (1K items)', 'Large (10K items)']) {
  const s = findSuite(`Fuzzy Search — ${size}`);
  if (!s) continue;
  const rf = s.get('rapid-fuzzy') ?? null;
  const fi = s.get('rapid-fuzzy (FuzzyIndex)') ?? null;
  const fj = s.get('fuse.js') ?? null;
  const fs = s.get('fuzzysort') ?? null;
  const b = best([rf, fi, fj, fs]);
  searchLines.push(
    `| ${size.replace(/\s*\(/, ' (').replace(' items)', ' items)')} | ${cell(rf, rf === b)} | ${cell(fi, fi === b)} | ${cell(fj, fj === b)} | ${cell(fs, fs === b)} |`,
  );
}

// Closest table (includes FuzzyIndex column)
const closestLines: string[] = [];
closestLines.push('| Dataset size | rapid-fuzzy | rapid-fuzzy (indexed) | fastest-levenshtein |');
closestLines.push('|---|---:|---:|---:|');

for (const size of ['Medium (1K items)', 'Large (10K items)']) {
  const s = findSuite(`Closest Match — ${size}`);
  if (!s) continue;
  const rf = s.get('rapid-fuzzy') ?? null;
  const fi = s.get('rapid-fuzzy (FuzzyIndex)') ?? null;
  const fl = s.get('fastest-levenshtein') ?? null;
  const b = best([rf, fi, fl].filter((v): v is number => v !== null));
  closestLines.push(
    `| ${size} | ${cell(rf, rf !== null && rf >= b)} | ${cell(fi, fi !== null && fi >= b)} | ${cell(fl, fl !== null && fl >= b)} |`,
  );
}

// ---------------------------------------------------------------------------
// 4. Replace tables in README
// ---------------------------------------------------------------------------

const readmePath = 'README.md';
let readme = readFileSync(readmePath, 'utf-8');

function replaceSection(
  content: string,
  afterLiteral: string,
  beforeLiteral: string,
  table: string,
): string {
  const afterIdx = content.indexOf(afterLiteral);
  if (afterIdx === -1) {
    console.warn(`  ⚠ Could not find "${afterLiteral}"`);
    return content;
  }
  // Find the first table after the header (starts with "|")
  const afterEnd = content.indexOf('\n\n', afterIdx) + 2;
  const tableStart = content.indexOf('|', afterEnd);

  const beforeIdx = content.indexOf(beforeLiteral, tableStart);
  if (beforeIdx === -1) {
    console.warn(`  ⚠ Could not find "${beforeLiteral}"`);
    return content;
  }
  // Go back to find the end of the table (last line before beforeLiteral)
  const tableEnd = content.lastIndexOf('\n', beforeIdx - 2) + 1;

  return `${content.slice(0, tableStart)}${table}\n\n${content.slice(tableEnd)}`;
}

readme = replaceSection(readme, '### Distance Functions', '> **Note**', distLines.join('\n'));
readme = replaceSection(
  readme,
  '### Search Performance',
  '### Closest Match',
  searchLines.join('\n'),
);
readme = replaceSection(
  readme,
  '### Closest Match (Levenshtein-based)',
  '> In indexed mode (`FuzzyIndex`)',
  closestLines.join('\n'),
);

writeFileSync(readmePath, readme);
console.log('✓ README.md updated with latest benchmark numbers.');
