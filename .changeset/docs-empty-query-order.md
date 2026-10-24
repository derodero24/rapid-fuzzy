---
"rapid-fuzzy": patch
---

Documentation: with `returnAllOnEmpty`, a query without a search term returns every item in index order, each with score 1, so `maxResults` keeps the first items. The declarations of `SearchOptions` and `KeySearchOptions` and the README's tie-breaking rule now say so. The browser build's `SearchOptions.returnAllOnEmpty` also names queries made only of syntax such as `^`, as the Node.js declarations already did.
