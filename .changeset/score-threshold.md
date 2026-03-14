---
"rapid-fuzzy": minor
---

Add score threshold filtering to search() and closest() functions.

search() now accepts a SearchOptions object with maxResults and minScore fields, while maintaining backward compatibility with the existing number argument for maxResults. closest() accepts an optional minScore parameter to return null when the best match is below the threshold.
