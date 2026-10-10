---
"rapid-fuzzy": patch
---

The Node.js ES module entry (`import ... from 'rapid-fuzzy'`) exports `__napiBindingTarget`, like the CommonJS entry. The declarations of both entries list it, so `import { __napiBindingTarget } from 'rapid-fuzzy'` type-checked but then failed when the module was linked (`The requested module 'rapid-fuzzy' does not provide an export named '__napiBindingTarget'`).
