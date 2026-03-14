---
"rapid-fuzzy": patch
---

Fix npm package missing index.js, index.d.ts, and browser.js

These napi-rs loader files were incorrectly gitignored, causing them to be excluded from the published npm package. CJS `require('rapid-fuzzy')` now works correctly.
