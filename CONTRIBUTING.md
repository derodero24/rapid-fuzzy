# Contributing to rapid-fuzzy

Thank you for your interest in contributing! This guide covers everything you need to get started.

## Prerequisites

- [Rust](https://rustup.rs/) ≥ 1.85 (stable toolchain)
- [Node.js](https://nodejs.org/) ≥ 20
- [pnpm](https://pnpm.io/) ≥ 10
- [Git](https://git-scm.com/)

## Development setup

```bash
# Clone the repository
git clone https://github.com/derodero24/rapid-fuzzy.git
cd rapid-fuzzy

# Install JS dependencies (skip lifecycle scripts to avoid running native builds)
pnpm install --ignore-scripts

# Build the native Node.js addon
pnpm run build

# Run tests
pnpm test          # JS/TS tests
cargo test         # Rust unit tests
```

### GitHub Codespaces / Dev Containers

You can skip local setup entirely by using [GitHub Codespaces](https://github.com/codespaces) or [VS Code Dev Containers](https://code.visualstudio.com/docs/devcontainers/containers). The repository includes a devcontainer configuration with all required tools pre-installed.

## Project structure

```
rapid-fuzzy/
├── crates/
│   ├── core/        ← Rust core library (distance functions, fuzzy search)
│   └── bench/       ← Rust benchmarks (Criterion)
├── __test__/        ← Vitest tests and JS benchmarks
├── npm/             ← Platform-specific binary packages
└── .github/
    └── workflows/   ← CI, CodeQL
```

## Workflow

All changes must start from a GitHub Issue.

1. Check existing issues or open a new one describing the problem or feature
2. Fork the repository and create a branch:
   ```
   git checkout -b type/issue-<number>-<short-summary>
   ```
3. Make your changes
4. Run the full verification suite (see below)
5. Open a pull request targeting the `develop` branch

### Branch naming

| Type | Example |
|------|---------|
| Feature | `feat/issue-42-batch-api` |
| Bug fix | `fix/issue-13-levenshtein-edge-case` |
| Documentation | `docs/issue-28-contributing` |
| CI/tooling | `ci/issue-20-path-filters` |

## Verification

Before pushing, run all of the following and ensure they pass:

```bash
pnpm run check        # Biome lint and format check
pnpm run typecheck    # TypeScript type check (requires prior build)
pnpm test             # Vitest tests
cargo test            # Rust unit tests
cargo clippy          # Rust lint
```

The pre-push Git hook runs all of these automatically.

## Commit messages

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): short description
```

**Types:** `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `ci`, `chore`, `style`, `revert`

**Scopes:** `core`, `bindings`, `wasm`, `bench`, `ci`, `docs`

**Examples:**
```
feat(core): add batch distance computation API
fix(core): handle empty string edge case in levenshtein
perf(bench): add batch variant benchmark
docs: update README with benchmark results
ci: add dependency audit workflow
```

## Changesets

If your change modifies anything under `crates/` or `src/`, add a changeset:

```bash
pnpm changeset
```

Select the bump type:
- `patch` — bug fixes, internal refactoring
- `minor` — new features, new public APIs
- `major` — breaking changes

## Pull request checklist

- [ ] Tests pass (`pnpm test`, `cargo test`)
- [ ] Lint passes (`pnpm run check`, `cargo clippy`)
- [ ] TypeScript types checked (`pnpm run typecheck`)
- [ ] Changeset added (if `crates/` or `src/` changed)
- [ ] PR title follows Conventional Commits format
- [ ] Issue linked in PR body (`Closes #<number>`)

## Benchmarks

If your change affects performance, run the benchmarks and include results in the PR:

```bash
pnpm run bench        # JS benchmarks (vs fuse.js, leven, etc.)
cargo bench           # Rust internal benchmarks
```

## Code style

**Rust:** Formatted with `rustfmt` (default settings). Linted with `cargo clippy`. Avoid `unsafe` code outside napi-rs internals.

**TypeScript/JavaScript:** Formatted and linted with [Biome](https://biomejs.dev/).

## Questions?

Open a [GitHub Discussion](https://github.com/derodero24/rapid-fuzzy/discussions) or file an issue.
