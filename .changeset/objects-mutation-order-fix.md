---
"rapid-fuzzy": patch
---

Fix mutation ordering in `objects.js` where the JS items array was mutated before Rust operations completed, which could cause state divergence if the Rust call threw an error.
