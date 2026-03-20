#!/usr/bin/env node
/**
 * Post-process WASM binaries with wasm-opt for size and speed optimization.
 *
 * Looks for *.wasm files in the project root and runs `wasm-opt -O3` on each.
 * Skips gracefully if wasm-opt is not installed (non-blocking).
 *
 * Usage:
 *   node scripts/optimize-wasm.js
 *   node scripts/optimize-wasm.js --size   # Use -Oz instead of -O3
 */

'use strict';

const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');

// Parse CLI flags
const optimizeForSize = process.argv.includes('--size');
const optLevel = optimizeForSize ? '-Oz' : '-O3';

// Check if wasm-opt is available
function findWasmOpt() {
  try {
    execFileSync('wasm-opt', ['--version'], { stdio: 'pipe' });
    return 'wasm-opt';
  } catch {
    return null;
  }
}

// Find all .wasm files in the project root (not in subdirectories)
function findWasmFiles() {
  return fs
    .readdirSync(ROOT)
    .filter((f) => f.endsWith('.wasm'))
    .map((f) => path.join(ROOT, f));
}

const wasmOpt = findWasmOpt();
if (!wasmOpt) {
  console.log('wasm-opt not found in PATH, skipping WASM optimization.');
  console.log('Install binaryen to enable: https://github.com/WebAssembly/binaryen');
  process.exit(0);
}

const wasmFiles = findWasmFiles();
if (wasmFiles.length === 0) {
  console.log('No .wasm files found in project root, nothing to optimize.');
  process.exit(0);
}

console.log(`Optimizing ${wasmFiles.length} WASM file(s) with wasm-opt ${optLevel}...`);

for (const wasmFile of wasmFiles) {
  const basename = path.basename(wasmFile);
  const originalSize = fs.statSync(wasmFile).size;

  try {
    // wasm-opt in-place: write to temp then rename
    const tmpFile = `${wasmFile}.opt`;
    execFileSync(wasmOpt, [optLevel, '--all-features', '-o', tmpFile, wasmFile], {
      stdio: 'pipe',
      timeout: 120_000,
    });
    fs.renameSync(tmpFile, wasmFile);

    const optimizedSize = fs.statSync(wasmFile).size;
    const reduction = (((originalSize - optimizedSize) / originalSize) * 100).toFixed(1);
    console.log(
      `  ${basename}: ${formatBytes(originalSize)} -> ${formatBytes(optimizedSize)} (${reduction}% smaller)`,
    );
  } catch (err) {
    // Clean up temp file on failure
    const tmpFile = `${wasmFile}.opt`;
    if (fs.existsSync(tmpFile)) {
      fs.unlinkSync(tmpFile);
    }
    console.error(`  ${basename}: optimization failed — ${err.message}`);
    process.exit(1);
  }
}

console.log('WASM optimization complete.');

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  const mb = kb / 1024;
  return `${mb.toFixed(2)} MB`;
}
