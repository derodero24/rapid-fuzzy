#!/usr/bin/env node
/**
 * Sync the version from package.json to crates/core/Cargo.toml.
 *
 * Run after `changeset version` to keep Cargo.toml in sync with package.json.
 * This is needed because changesets only manages npm package versions.
 *
 * Usage:
 *   node scripts/sync-cargo-version.js
 */

'use strict';

const fs = require('node:fs');
const path = require('node:path');

const PKG_PATH = path.resolve(__dirname, '..', 'package.json');
const CARGO_PATH = path.resolve(__dirname, '..', 'crates', 'core', 'Cargo.toml');

const pkg = JSON.parse(fs.readFileSync(PKG_PATH, 'utf-8'));
const version = pkg.version;

const cargo = fs.readFileSync(CARGO_PATH, 'utf-8');
const updated = cargo.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);

if (cargo === updated) {
  console.log(`Cargo.toml already at version ${version}`);
} else {
  fs.writeFileSync(CARGO_PATH, updated);
  console.log(`Synced Cargo.toml version to ${version}`);
}
