#!/usr/bin/env node
/**
 * Extract release notes for a specific version from CHANGELOG.md.
 *
 * Reads the changelog, finds the section for the given version, cleans up
 * commit hash prefixes, and appends an installation snippet.
 *
 * Usage:
 *   node scripts/extract-changelog.js [version]
 *   If no version is provided, reads from package.json.
 */

'use strict';

const fs = require('node:fs');
const path = require('node:path');

const version = process.argv[2] || require(path.join(process.cwd(), 'package.json')).version;
const changelog = fs.readFileSync(path.join(process.cwd(), 'CHANGELOG.md'), 'utf8');

// Find section for this version: between "## X.Y.Z" and next "## "
const escapedVersion = version.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const sectionRegex = new RegExp(`## ${escapedVersion}\\n([\\s\\S]*?)(?=\\n## |$)`);
const match = changelog.match(sectionRegex);

if (!match) {
  console.error(`No changelog entry found for version ${version}`);
  process.exit(1);
}

// Clean up: remove commit hash prefixes (e.g., "- abc1234: " → "- ")
const notes = match[1].replace(/^- [a-f0-9]{7}: /gm, '- ').trim();

// Add installation section
const output = `${notes}

### Installation

\`\`\`bash
npm install rapid-fuzzy@${version}
\`\`\``;

process.stdout.write(output);
