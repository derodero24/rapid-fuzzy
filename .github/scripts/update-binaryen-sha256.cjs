#!/usr/bin/env node
/**
 * Refresh BINARYEN_SHA256 in scripts/install-binaryen.sh for the
 * BINARYEN_VERSION it pins.
 *
 * Renovate bumps BINARYEN_VERSION through a regex manager but cannot know the
 * new archive's checksum, so it runs this script as a post-upgrade task (see
 * renovate.json and the allowed command in .github/workflows/renovate.yml).
 * It can also be run by hand after editing the version:
 *
 *   node .github/scripts/update-binaryen-sha256.cjs
 *
 * The archive is downloaded and hashed, and the result must match the
 * `.sha256` asset published with the same GitHub release. That protects
 * against a truncated or corrupted download, not against a compromised
 * release (both files come from the same place); the pinned value is what
 * lets CI detect an asset that changes after the bump was reviewed.
 */

'use strict';

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const SCRIPT = path.resolve(__dirname, '..', '..', 'scripts', 'install-binaryen.sh');
const VERSION_RE = /^BINARYEN_VERSION="(version_\d+)"$/m;
const SHA_RE = /^BINARYEN_SHA256="[0-9a-f]{64}"$/m;

/** @param {string} url */
async function download(url) {
  const res = await fetch(url, { redirect: 'follow' });
  if (!res.ok) {
    throw new Error(`GET ${url}: HTTP ${res.status}`);
  }
  return Buffer.from(await res.arrayBuffer());
}

async function main() {
  const source = fs.readFileSync(SCRIPT, 'utf8');
  const version = VERSION_RE.exec(source)?.[1];
  if (!version || !SHA_RE.test(source)) {
    throw new Error(`${SCRIPT}: BINARYEN_VERSION / BINARYEN_SHA256 lines not found`);
  }

  const archive = `binaryen-${version}-x86_64-linux.tar.gz`;
  const url = `https://github.com/WebAssembly/binaryen/releases/download/${version}/${archive}`;
  const actual = crypto
    .createHash('sha256')
    .update(await download(url))
    .digest('hex');
  const published = (await download(`${url}.sha256`)).toString('utf8').trim().split(/\s+/)[0];
  if (actual !== published) {
    throw new Error(`${archive}: sha256 ${actual} does not match published ${published}`);
  }

  const updated = source.replace(SHA_RE, `BINARYEN_SHA256="${actual}"`);
  if (updated !== source) {
    fs.writeFileSync(SCRIPT, updated);
  }
  process.stdout.write(`${archive}: ${actual}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
