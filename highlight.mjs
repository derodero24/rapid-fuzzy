// ESM re-export — single source of truth is highlight.js (CJS).
// Node.js detects named exports from CJS via static analysis.
// Bundlers (webpack, vite, rollup) handle CJS interop natively.
export { highlight, highlightRanges } from './highlight.js';
