#!/usr/bin/env npx tsx
/**
 * Generate SVG bar charts from the benchmark tables in README.md.
 *
 * Usage:
 *   pnpm bench:charts          # parse README and generate chart SVGs
 *   npx tsx scripts/generate-bench-charts.ts
 */

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const ASSET_DIR = join(new URL('.', import.meta.url).pathname, '..', '.github', 'assets');
mkdirSync(ASSET_DIR, { recursive: true });

// ---------------------------------------------------------------------------
// 1. Parse benchmark tables from README
// ---------------------------------------------------------------------------

const readme = readFileSync('README.md', 'utf-8');

interface BarEntry {
  label: string;
  value: number;
}

interface ChartData {
  title: string;
  subtitle?: string;
  groups: { groupLabel: string; bars: BarEntry[] }[];
}

function parseOps(s: string): number | null {
  const m = s.replace(/\*\*/g, '').match(/([\d,]+)\s*ops\/s/);
  return m ? Number(m[1].replace(/,/g, '')) : null;
}

function parseSearchTable(): ChartData {
  const groups: ChartData['groups'] = [];
  const lines = readme.split('\n');

  for (const line of lines) {
    // Match rows: | Small (20 items) | rf | fi | fj | fs | uf |
    const m = line.match(
      /\|\s*(Small|Medium|Large|XL)\s*\([^)]+\)\s*\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|/,
    );
    if (!m) continue;

    const groupLabel = m[1];
    const rf = parseOps(m[2]);
    const fi = parseOps(m[3]);
    const fj = parseOps(m[4]);
    const fs = parseOps(m[5]);
    const uf = parseOps(m[6]);

    const bars: BarEntry[] = [];
    if (rf !== null) bars.push({ label: 'rapid-fuzzy', value: rf });
    if (fi !== null) bars.push({ label: 'rapid-fuzzy (indexed)', value: fi });
    if (fj !== null) bars.push({ label: 'fuse.js', value: fj });
    if (fs !== null) bars.push({ label: 'fuzzysort', value: fs });
    if (uf !== null) bars.push({ label: 'uFuzzy', value: uf });

    groups.push({ groupLabel, bars });
  }

  return { title: 'Fuzzy Search Performance', subtitle: 'ops/s (higher is better)', groups };
}

function parseDistanceTable(): ChartData {
  const groups: ChartData['groups'] = [];
  const lines = readme.split('\n');

  for (const line of lines) {
    const m = line.match(
      /\|\s*(Levenshtein|Normalized Levenshtein|Sorensen-Dice|Jaro-Winkler|Damerau-Levenshtein)\s*\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|/,
    );
    if (!m) continue;

    const groupLabel = m[1];
    const rf = parseOps(m[2]);
    const fl = parseOps(m[3]);
    const lv = parseOps(m[4]);
    const ss = parseOps(m[5]);

    const bars: BarEntry[] = [];
    if (rf !== null) bars.push({ label: 'rapid-fuzzy', value: rf });
    if (fl !== null) bars.push({ label: 'fastest-levenshtein', value: fl });
    if (lv !== null) bars.push({ label: 'leven', value: lv });
    if (ss !== null) bars.push({ label: 'string-similarity', value: ss });

    if (bars.length > 1) groups.push({ groupLabel, bars });
  }

  return { title: 'Distance Function Performance', subtitle: 'ops/s (higher is better)', groups };
}

// ---------------------------------------------------------------------------
// 2. Generate SVG chart
// ---------------------------------------------------------------------------

const COLORS: Record<string, string> = {
  'rapid-fuzzy': '#3b82f6',
  'rapid-fuzzy (indexed)': '#60a5fa',
  'fuse.js': '#94a3b8',
  fuzzysort: '#94a3b8',
  uFuzzy: '#94a3b8',
  'fastest-levenshtein': '#94a3b8',
  leven: '#94a3b8',
  'string-similarity': '#94a3b8',
};

interface ChartLayout {
  barHeight: number;
  barGap: number;
  groupGap: number;
  labelWidth: number;
  barAreaWidth: number;
  paddingX: number;
}

const LAYOUT: ChartLayout = {
  barHeight: 26,
  barGap: 5,
  groupGap: 24,
  labelWidth: 160,
  barAreaWidth: 640 - 160 - 20,
  paddingX: 20,
};

function buildValueText(bar: BarEntry, group: { bars: BarEntry[] }): string {
  const formatted = formatNumber(bar.value);
  let text = `${formatted} ops/s`;

  const isRapidFuzzy = bar.label === 'rapid-fuzzy' || bar.label === 'rapid-fuzzy (indexed)';
  if (!isRapidFuzzy) return text;

  const rfEntry = group.bars.find((b) => b.label === 'rapid-fuzzy');
  const fuseEntry = group.bars.find((b) => b.label === 'fuse.js');
  if (rfEntry && fuseEntry && fuseEntry.value > 0) {
    const mult = Math.round(rfEntry.value / fuseEntry.value);
    if (mult >= 2) text += ` — ${mult}x vs fuse.js`;
  }

  return text;
}

function renderBar(
  bar: BarEntry,
  group: { bars: BarEntry[] },
  y: number,
  groupMax: number,
): string[] {
  const { barHeight, labelWidth, barAreaWidth, paddingX } = LAYOUT;
  const barWidth = Math.max(4, (bar.value / groupMax) * barAreaWidth);
  const color = COLORS[bar.label] ?? '#94a3b8';
  const isRapidFuzzy = bar.label === 'rapid-fuzzy' || bar.label === 'rapid-fuzzy (indexed)';
  const x = paddingX + labelWidth;
  const textY = y + barHeight / 2 + 4;

  const valueText = buildValueText(bar, group);
  const textX = barWidth > 200 ? x + barWidth - 8 : x + barWidth + 8;
  const anchor = barWidth > 200 ? 'end' : 'start';
  const fillAttr = barWidth > 200 && isRapidFuzzy ? ' fill="#ffffff"' : '';

  return [
    `<text x="${x - 8}" y="${textY}" class="bar-label" text-anchor="end">${escapeXml(bar.label)}</text>`,
    `<rect x="${x}" y="${y}" width="${barWidth}" height="${barHeight}" rx="4" fill="${color}" opacity="${isRapidFuzzy ? 1 : 0.6}" />`,
    `<text x="${textX}" y="${textY}" class="bar-value" text-anchor="${anchor}"${fillAttr}>${escapeXml(valueText)}</text>`,
  ];
}

function renderGroup(
  group: ChartData['groups'][number],
  startY: number,
): { lines: string[]; endY: number } {
  const { barHeight, barGap, groupGap, paddingX } = LAYOUT;
  const lines: string[] = [];
  let y = startY;

  lines.push(
    `<text x="${paddingX}" y="${y + 14}" class="group-label">${escapeXml(group.groupLabel)}</text>`,
  );
  y += 22;

  const sorted = [...group.bars].sort((a, b) => b.value - a.value);
  const groupMax = sorted[0]?.value ?? 1;

  for (const bar of sorted) {
    lines.push(...renderBar(bar, group, y, groupMax));
    y += barHeight + barGap;
  }

  return { lines, endY: y + groupGap - barGap };
}

function generateSvg(data: ChartData): string {
  const { barHeight, barGap, groupGap, paddingX } = LAYOUT;
  const chartWidth = 640;
  const headerHeight = 52;
  const paddingBottom = 24;
  const groupLabelHeight = 22;

  let totalBarHeight = 0;
  for (const group of data.groups) {
    totalBarHeight +=
      groupLabelHeight + group.bars.length * (barHeight + barGap) + groupGap - barGap;
  }

  const svgWidth = chartWidth + paddingX * 2;
  const svgHeight = headerHeight + totalBarHeight + paddingBottom;

  const lines: string[] = [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${svgWidth} ${svgHeight}" width="${svgWidth}" height="${svgHeight}">`,
    '<style>',
    '  text { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }',
    '  .title { font-size: 16px; font-weight: 600; fill: #1f2937; }',
    '  .subtitle { font-size: 12px; fill: #6b7280; }',
    '  .group-label { font-size: 13px; font-weight: 600; fill: #374151; }',
    '  .bar-label { font-size: 12px; fill: #4b5563; }',
    '  .bar-value { font-size: 12px; fill: #374151; font-weight: 500; }',
    '  .multiplier { font-size: 11px; fill: #059669; font-weight: 600; }',
    '</style>',
    `<rect width="${svgWidth}" height="${svgHeight}" rx="8" fill="#ffffff" />`,
    `<rect x="0.5" y="0.5" width="${svgWidth - 1}" height="${svgHeight - 1}" rx="8" fill="none" stroke="#e5e7eb" />`,
    `<text x="${paddingX}" y="28" class="title">${escapeXml(data.title)}</text>`,
  ];

  if (data.subtitle) {
    lines.push(`<text x="${paddingX}" y="44" class="subtitle">${escapeXml(data.subtitle)}</text>`);
  }

  let y = headerHeight;
  for (const group of data.groups) {
    const result = renderGroup(group, y);
    lines.push(...result.lines);
    y = result.endY;
  }

  lines.push('</svg>');
  return lines.join('\n');
}

function formatNumber(n: number): string {
  return Math.round(n).toLocaleString('en-US');
}

function escapeXml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// ---------------------------------------------------------------------------
// 3. Generate and write SVGs
// ---------------------------------------------------------------------------

const searchData = parseSearchTable();
const distData = parseDistanceTable();

if (searchData.groups.length > 0) {
  const svg = generateSvg(searchData);
  writeFileSync(join(ASSET_DIR, 'bench-search.svg'), svg);
  console.log(`✓ Generated bench-search.svg (${searchData.groups.length} groups)`);
} else {
  console.warn('⚠ No search benchmark data found in README');
}

if (distData.groups.length > 0) {
  const svg = generateSvg(distData);
  writeFileSync(join(ASSET_DIR, 'bench-distance.svg'), svg);
  console.log(`✓ Generated bench-distance.svg (${distData.groups.length} groups)`);
} else {
  console.warn('⚠ No distance benchmark data found in README');
}
