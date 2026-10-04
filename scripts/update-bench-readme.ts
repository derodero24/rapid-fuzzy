#!/usr/bin/env npx tsx
/**
 * Run the comparison benchmarks and update the benchmark tables in README.md.
 *
 * Usage:
 *   pnpm bench:readme          # build the native addon in release mode first
 *   npx tsx scripts/update-bench-readme.ts
 *   BENCH_OUTPUT=bench.txt npx tsx scripts/update-bench-readme.ts  # reuse saved output
 *
 * The tables live between `<!-- bench:<name>:start -->` and
 * `<!-- bench:<name>:end -->` markers in README.md; the `bench:env` block
 * records the machine and Node.js version the numbers were measured on.
 * Run `pnpm bench:charts` afterwards to regenerate the SVG charts.
 */

import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { arch, cpus, platform } from 'node:os';

const BENCH_FILES = [
  '__test__/search.compare.bench.ts',
  '__test__/distance.compare.bench.ts',
  '__test__/distance.bench.ts',
  '__test__/similarity.bench.ts',
];

// ---------------------------------------------------------------------------
// 1. Run benchmarks and capture output
// ---------------------------------------------------------------------------

// BENCH_OUTPUT=<file> reuses the saved output of a previous run instead.
const savedOutput = process.env.BENCH_OUTPUT;
console.log(savedOutput ? `Reading benchmark output from ${savedOutput}…` : 'Running benchmarks…');
const raw = savedOutput
  ? readFileSync(savedOutput, 'utf-8')
  : execSync(`pnpm exec vitest bench --run ${BENCH_FILES.join(' ')}`, {
      encoding: 'utf-8',
      timeout: 1_800_000,
      env: { ...process.env, FORCE_COLOR: '0' },
      stdio: ['pipe', 'pipe', 'pipe'],
    });

// Strip ANSI escape codes
// biome-ignore lint/suspicious/noControlCharactersInRegex: ANSI escape stripping requires matching ESC (0x1B)
const clean = raw.replace(/\x1b\[[0-9;]*m/g, '');

// ---------------------------------------------------------------------------
// 2. Parse results — suites ("✓ file > suite") and their "· name  hz …" lines
// ---------------------------------------------------------------------------

const suites = new Map<string, Map<string, number>>();
let current: Map<string, number> | undefined;

for (const line of clean.split('\n')) {
  // "  ✓ __test__/search.compare.bench.ts > Fuzzy Search — Small 20 (vs competitors) 7450ms"
  const sm = line.match(/✓\s+\S+\s+>\s+(.+?)(?:\s+\d+ms)?$/);
  if (sm) {
    current = new Map();
    suites.set(sm[1].trim(), current);
    continue;
  }
  // "   · rapid-fuzzy  179,371.64  …"
  const em = line.match(/^\s+·\s+(.+?)\s{2,}([\d,]+(?:\.\d+)?)\s/);
  if (em && current) {
    current.set(em[1].trim(), Number(em[2].replace(/,/g, '')));
  }
}

function result(suite: string, bench: string): number | null {
  const s = suites.get(suite);
  if (!s) {
    console.warn(`  ⚠ Suite not found: "${suite}"`);
    return null;
  }
  const value = s.get(bench);
  if (value === undefined) {
    console.warn(`  ⚠ Benchmark not found: "${suite}" > "${bench}"`);
    return null;
  }
  return value;
}

// ---------------------------------------------------------------------------
// 3. Build markdown tables
// ---------------------------------------------------------------------------

function fmt(n: number): string {
  return Math.round(n).toLocaleString('en-US');
}

/** Format a row, marking the highest value in bold. */
function row(label: string, values: (number | null)[], suffix = ' ops/s'): string {
  const present = values.filter((v): v is number => v !== null);
  const best = present.length > 1 ? Math.max(...present) : Number.NaN;
  const cells = values.map((v) => {
    if (v === null) return '—';
    const text = `${fmt(v)}${suffix}`;
    return v === best ? `**${text}**` : text;
  });
  return `| ${label} | ${cells.join(' | ')} |`;
}

const searchRows: [label: string, suite: string][] = [
  ['Small (20 items)', 'Fuzzy Search — Small 20 (vs competitors)'],
  ['Medium (1K items)', 'Fuzzy Search — Medium 1K (vs competitors)'],
  ['Large (10K items)', 'Fuzzy Search — Large 10K (vs competitors)'],
  [
    'Large (10K items, rotating queries)',
    'Fuzzy Search — Large 10K, rotating queries (vs competitors)',
  ],
  [
    'Large (10K items, type-ahead, 12 searches)',
    'Fuzzy Search — Large 10K, type-ahead (vs competitors)',
  ],
  ['Extra large (50K items)', 'Fuzzy Search — Extra Large 50K (vs competitors)'],
  ['Huge (100K items)', 'Fuzzy Search — Huge 100K (vs competitors)'],
];

const searchLines = [
  '| Dataset | rapid-fuzzy | rapid-fuzzy (indexed) | fuse.js | fuzzysort | uFuzzy |',
  '|---|---:|---:|---:|---:|---:|',
];
for (const [label, suite] of searchRows) {
  const s = suites.get(suite);
  const get = (bench: string) => s?.get(bench) ?? null;
  if (!s) console.warn(`  ⚠ Suite not found: "${suite}"`);
  searchLines.push(
    row(label, [
      get('rapid-fuzzy'),
      get('rapid-fuzzy (FuzzyIndex)'),
      get('fuse.js'),
      get('fuzzysort'),
      get('uFuzzy'),
    ]),
  );
}

const closestLines = [
  '| Dataset | rapid-fuzzy | rapid-fuzzy (indexed) | fastest-levenshtein |',
  '|---|---:|---:|---:|',
];
for (const [label, suite] of [
  ['Medium (1K items)', 'Closest Match — Medium 1K (vs competitors)'],
  ['Large (10K items)', 'Closest Match — Large 10K (vs competitors)'],
]) {
  closestLines.push(
    row(label, [
      result(suite, 'rapid-fuzzy'),
      result(suite, 'rapid-fuzzy (FuzzyIndex)'),
      result(suite, 'fastest-levenshtein'),
    ]),
  );
}

const lev = 'Levenshtein Distance (vs competitors)';
const norm = 'Normalized Similarity (vs competitors)';
const distanceLines = [
  '| Function | rapid-fuzzy | fastest-levenshtein | leven | string-similarity |',
  '|---|---:|---:|---:|---:|',
  row('Levenshtein', [
    result(lev, 'rapid-fuzzy'),
    result(lev, 'fastest-levenshtein'),
    result(lev, 'leven'),
    null,
  ]),
  row('Normalized Levenshtein', [
    result(norm, 'rapid-fuzzy (normalizedLevenshtein)'),
    null,
    null,
    null,
  ]),
  row('Sorensen-Dice', [
    result(norm, 'rapid-fuzzy (sorensenDice)'),
    null,
    null,
    result(norm, 'string-similarity (compareTwoStrings / Dice)'),
  ]),
  row('Jaro-Winkler', [
    result('Jaro / Jaro-Winkler', 'rapid-fuzzy (jaroWinkler)'),
    null,
    null,
    null,
  ]),
  row('Damerau-Levenshtein', [result('Damerau-Levenshtein', 'rapid-fuzzy'), null, null, null]),
  row('Hamming', [result('Hamming Distance', 'rapid-fuzzy'), null, null, null]),
];

const token = 'Token-Based Ratio (vs competitors)';
const many = 'Levenshtein Distance — Many 1K (vs competitors)';
const loopRf = result(many, 'rapid-fuzzy (loop)');
const loopFl = result(many, 'fastest-levenshtein (loop)');
const ratioLines = [
  '| Function | rapid-fuzzy | fuzzball |',
  '|---|---:|---:|',
  row('Token sort ratio', [
    result(token, 'rapid-fuzzy (tokenSortRatio)'),
    result(token, 'fuzzball (token_sort_ratio)'),
  ]),
  row('Token set ratio', [
    result(token, 'rapid-fuzzy (tokenSetRatio)'),
    result(token, 'fuzzball (token_set_ratio)'),
  ]),
  row('Weighted ratio (`weightedRatio` / `WRatio`)', [
    result(token, 'rapid-fuzzy (weightedRatio)'),
    result(token, 'fuzzball (WRatio)'),
  ]),
  `| Levenshtein, 1 vs 1,000 candidates | ${fmtOrDash(result(many, 'rapid-fuzzy (many)'))} (\`levenshteinMany\`) | — |`,
  `| ↳ same, one call per candidate | ${fmtOrDash(loopRf)} (\`levenshtein\`) | fastest-levenshtein: ${fmtOrDash(loopFl)} |`,
];

function fmtOrDash(v: number | null): string {
  return v === null ? '—' : `${fmt(v)} ops/s`;
}

const cpuList = cpus();
const date = new Date().toISOString().slice(0, 10);
// BENCH_MACHINE adds a note about the machine, e.g. "a shared cloud VM".
const machineNote = process.env.BENCH_MACHINE ? `, ${process.env.BENCH_MACHINE}` : '';
const envText = `Measured on ${date} with Node.js ${process.version} on ${platform()} ${arch()} (${cpuList[0]?.model.trim() ?? 'unknown CPU'}, ${cpuList.length} logical CPUs${machineNote}), using the release build of the native addon and [Vitest bench](https://vitest.dev/guide/features.html#benchmarking). Numbers vary by up to about ±10% between runs; treat them as relative, not absolute.`;

// ---------------------------------------------------------------------------
// 4. Replace the marked blocks in README
// ---------------------------------------------------------------------------

const readmePath = 'README.md';
let readme = readFileSync(readmePath, 'utf-8');

function replaceBlock(content: string, name: string, body: string): string {
  const start = `<!-- bench:${name}:start -->`;
  const end = `<!-- bench:${name}:end -->`;
  const from = content.indexOf(start);
  const to = content.indexOf(end);
  if (from === -1 || to === -1 || to < from) {
    console.warn(`  ⚠ Could not find the ${start} … ${end} block`);
    return content;
  }
  return `${content.slice(0, from + start.length)}\n${body}\n${content.slice(to)}`;
}

readme = replaceBlock(readme, 'env', envText);
readme = replaceBlock(readme, 'search', searchLines.join('\n'));
readme = replaceBlock(readme, 'closest', closestLines.join('\n'));
readme = replaceBlock(readme, 'distance', distanceLines.join('\n'));
readme = replaceBlock(readme, 'ratio', ratioLines.join('\n'));

writeFileSync(readmePath, readme);
console.log('✓ README.md updated with the latest benchmark numbers.');
console.log('  Review the prose around the tables (ratios, key takeaways) by hand.');
