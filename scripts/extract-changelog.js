#!/usr/bin/env node
/**
 * Extract release notes for a specific version from CHANGELOG.md.
 *
 * Reads the changelog, finds the section for the given version, cleans up
 * commit hash prefixes, replaces changeset headings with user-friendly ones,
 * and appends an installation snippet with a changelog comparison link.
 *
 * Usage:
 *   node scripts/extract-changelog.js [version]
 *   If no version is provided, reads from package.json.
 */

'use strict';

const fs = require('node:fs');
const path = require('node:path');

const pkg = require(path.join(process.cwd(), 'package.json'));
const version = process.argv[2] || pkg.version;
const changelog = fs.readFileSync(path.join(process.cwd(), 'CHANGELOG.md'), 'utf8');

// Find section for this version: between "## X.Y.Z" and next "## "
const escapedVersion = version.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const sectionRegex = new RegExp(`## ${escapedVersion}\\n([\\s\\S]*?)(?=\\n## |$)`);
const match = changelog.match(sectionRegex);

if (!match) {
  console.error(`No changelog entry found for version ${version}`);
  process.exit(1);
}

const notes = match[1]
  // Remove commit hash prefixes (e.g., "- abc1234: " → "- ")
  .replace(/^- [a-f0-9]{7}: /gm, '- ')
  // Replace changeset headings with user-friendly ones
  .replace(/^### Major Changes$/gm, '### Breaking Changes')
  .replace(/^### Minor Changes$/gm, '### Features')
  .replace(/^### Patch Changes$/gm, '### Fixes & Improvements')
  .trim();

// Find previous version for changelog comparison link
const versions = Array.from(changelog.matchAll(/^## (\d+\.\d+\.\d+)$/gm), (m) => m[1]);
const currentIdx = versions.indexOf(version);
const prevVersion =
  currentIdx >= 0 && currentIdx < versions.length - 1 ? versions[currentIdx + 1] : null;

// Build repo URL for changelog comparison link
const rawUrl = typeof pkg.repository === 'string' ? pkg.repository : pkg.repository?.url || '';
const repoUrl = rawUrl.replace(/^git\+/, '').replace(/\.git$/, '');

// Build output
const parts = [notes];

parts.push(`### Installation

\`\`\`bash
npm install rapid-fuzzy@${version}
\`\`\``);

if (prevVersion && repoUrl) {
  parts.push(`**Full Changelog**: ${repoUrl}/compare/v${prevVersion}...v${version}`);
}

process.stdout.write(`${parts.join('\n\n')}\n`);
