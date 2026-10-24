## Summary

<!-- Brief description of the changes -->

## Related issue

<!-- Closes #123 -->

## Breaking changes / Deprecations

<!-- If this PR introduces a breaking change or deprecates an export, document it here. -->
<!-- Write N/A if not applicable -->

## Checklist

- [ ] Lint passes (`pnpm run check`)
- [ ] TypeScript type-check passes (`pnpm run typecheck`)
- [ ] JS tests pass (`pnpm test`)
- [ ] Rust tests pass (`cargo test --workspace`)
- [ ] Clippy passes (`cargo clippy --workspace --all-targets -- -D warnings`)
- [ ] Build succeeds (`pnpm run build`)
- [ ] Changeset included (if a shipped file changed: `crates/`, the root JS/TS entry points, `npm/`)
- [ ] Benchmarks run for performance-sensitive changes
