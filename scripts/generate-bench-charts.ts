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
    // Match rows like: | Small (20 items) | 179,222 ops/s | 109,059 ops/s | ...
    const m = line.match(/\|\s*(Small|Medium|Large)\s*\([^)]+\)\s*\|([^|]+)\|([^|]+)\|([^|]+)\|/);
    if (!m) continue;

    const groupLabel = m[1];
    const rf = parseOps(m[2]);
    const fj = parseOps(m[3]);
    const fs = parseOps(m[4]);

    const bars: BarEntry[] = [];
    if (rf !== null) bars.push({ label: 'rapid-fuzzy', value: rf });
    if (fj !== null) bars.push({ label: 'fuse.js', value: fj });
    if (fs !== null) bars.push({ label: 'fuzzysort', value: fs });

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
  'fuse.js': '#94a3b8',
  fuzzysort: '#94a3b8',
  'fastest-levenshtein': '#94a3b8',
  leven: '#94a3b8',
  'string-similarity': '#94a3b8',
};

function generateSvg(data: ChartData): string {
  const barHeight = 26;
  const barGap = 5;
  const groupGap = 24;
  const labelWidth = 160;
  const chartWidth = 640;
  const barAreaWidth = chartWidth - labelWidth - 20;
  const headerHeight = 52;
  const paddingX = 20;
  const paddingBottom = 24;

  // Calculate total height
  let totalBarHeight = 0;
  for (const group of data.groups) {
    totalBarHeight += group.bars.length * (barHeight + barGap) + groupGap;
  }

  const svgWidth = chartWidth + paddingX * 2;
  const svgHeight = headerHeight + totalBarHeight + paddingBottom;

  const lines: string[] = [];
  lines.push(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${svgWidth} ${svgHeight}" width="${svgWidth}" height="${svgHeight}">`,
  );
  lines.push('<style>');
  lines.push(
    '  text { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }',
  );
  lines.push('  .title { font-size: 16px; font-weight: 600; fill: #1f2937; }');
  lines.push('  .subtitle { font-size: 12px; fill: #6b7280; }');
  lines.push('  .group-label { font-size: 13px; font-weight: 600; fill: #374151; }');
  lines.push('  .bar-label { font-size: 12px; fill: #4b5563; }');
  lines.push('  .bar-value { font-size: 12px; fill: #374151; font-weight: 500; }');
  lines.push('  .multiplier { font-size: 11px; fill: #059669; font-weight: 600; }');
  lines.push('</style>');

  // Background
  lines.push(`<rect width="${svgWidth}" height="${svgHeight}" rx="8" fill="#ffffff" />`);
  lines.push(
    `<rect x="0.5" y="0.5" width="${svgWidth - 1}" height="${svgHeight - 1}" rx="8" fill="none" stroke="#e5e7eb" />`,
  );

  // Title
  lines.push(`<text x="${paddingX}" y="28" class="title">${escapeXml(data.title)}</text>`);
  if (data.subtitle) {
    lines.push(`<text x="${paddingX}" y="44" class="subtitle">${escapeXml(data.subtitle)}</text>`);
  }

  let y = headerHeight;

  for (const group of data.groups) {
    // Group label
    lines.push(
      `<text x="${paddingX}" y="${y + 14}" class="group-label">${escapeXml(group.groupLabel)}</text>`,
    );
    y += 22;

    // Sort bars by value descending
    const sorted = [...group.bars].sort((a, b) => b.value - a.value);
    // Per-group scale: largest bar in this group fills the available width
    const groupMax = sorted[0]?.value ?? 1;
    const rfEntry = group.bars.find((b) => b.label === 'rapid-fuzzy');

    for (const bar of sorted) {
      const barWidth = Math.max(4, (bar.value / groupMax) * barAreaWidth);
      const color = COLORS[bar.label] ?? '#94a3b8';
      const isRapidFuzzy = bar.label === 'rapid-fuzzy';
      const x = paddingX + labelWidth;

      // Label
      lines.push(
        `<text x="${x - 8}" y="${y + barHeight / 2 + 4}" class="bar-label" text-anchor="end">${escapeXml(bar.label)}</text>`,
      );

      // Bar
      lines.push(
        `<rect x="${x}" y="${y}" width="${barWidth}" height="${barHeight}" rx="4" fill="${color}" opacity="${isRapidFuzzy ? 1 : 0.6}" />`,
      );

      // Value + multiplier inside or after bar
      const formatted = formatNumber(bar.value);
      let valueText = `${formatted} ops/s`;

      // Show "Nx faster" for rapid-fuzzy vs fuse.js
      if (isRapidFuzzy && rfEntry) {
        const fuseEntry = group.bars.find((b) => b.label === 'fuse.js');
        if (fuseEntry && fuseEntry.value > 0) {
          const mult = Math.round(rfEntry.value / fuseEntry.value);
          if (mult >= 2) {
            valueText += ` — ${mult}x vs fuse.js`;
          }
        }
      }

      const textX = barWidth > 200 ? x + barWidth - 8 : x + barWidth + 8;
      const anchor = barWidth > 200 ? 'end' : 'start';
      const textFill = barWidth > 200 && isRapidFuzzy ? '#ffffff' : '';
      const cls =
        isRapidFuzzy && rfEntry && group.bars.some((b) => b.label === 'fuse.js')
          ? 'bar-value'
          : 'bar-value';

      lines.push(
        `<text x="${textX}" y="${y + barHeight / 2 + 4}" class="${cls}" text-anchor="${anchor}"${textFill ? ` fill="${textFill}"` : ''}>${escapeXml(valueText)}</text>`,
      );

      y += barHeight + barGap;
    }

    y += groupGap - barGap;
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
