# Contributing to Adze

Thank you for your interest in contributing! This guide covers everything you need to get started.

## Quick Start

1. **Fork and clone** the repository
2. **Install prerequisites**: Rust 1.92+ (via `rustup`), `jq`, optionally `rg` (ripgrep)
3. **Build**: `cargo build`
4. **Test**: `cargo test`
5. **Find work**: Check [open issues](https://github.com/EffortlessMetrics/adze/issues) labeled `good first issue`

## Setup

### Prerequisites

- **Rust 1.92.0+** with `rustfmt` and `clippy` (configured via `rust-toolchain.toml`)
- **jq** for crate-aware checks
- **rg** (ripgrep) - optional but recommended
- **libtree-sitter-dev** - only needed for ts-bridge tool

### Enable Git Hooks

```bash
git config core.hooksPath .githooks

# Ensure scripts are executable
git update-index --chmod=+x .githooks/pre-commit .githooks/pre-push \
  scripts/affected-crates.sh scripts/check-goto-indexing.sh
```

### Hook Behavior

**Pre-commit (fast path)**
- Formats only staged files with `rustfmt`
- Runs clippy on affected crates only
- Blocks commits with conflict markers

**Pre-push (full validation)**
- Runs clippy across entire workspace
- Tests with feature matrix

## Development Workflow

### Building

```bash
cargo build                    # Build all workspace members
cargo build -p adze     # Build a specific package
```

### Testing

```bash
cargo test                              # Run all tests
cargo test -p adze-glr-core      # Test a specific crate
cargo test test_name -- --nocapture     # Run one test with output
cargo insta review                      # Update snapshot tests
```

For stable test execution with concurrency caps:

```bash
cargo t2                    # Run tests with 2 threads
cargo test-safe            # Run tests with safe defaults
./scripts/test-local.sh    # Local test runner with timeout handling
```

### Linting

```bash
cargo fmt --all --check                 # Check formatting
cargo clippy --all -- -D warnings       # Lint with warnings as errors
```

## Making Changes

### Code Style

- **Follow existing patterns** in the surrounding code
- **No unnecessary comments** - code should be self-documenting
- **Prefer editing over creating** - modify existing files when possible
- **Use `debugln!(...)` over raw `eprintln!/println!/dbg!`** for debug output

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation changes
- `test:` test changes
- `chore:` maintenance tasks
- `refactor:` code restructuring without behavior change
- `perf:` performance improvements

### Pull Requests

1. Create a feature branch from `main`
2. Make your changes with clear, focused commits
3. Run the pre-commit hook or lint manually before pushing
4. Open a PR against `main`
5. Fill out the PR template

PRs are squash-merged. Your PR title becomes the commit message, so make it descriptive.

## Release Readiness Checklist

- [ ] Update crate versions and changelog entries together (`CHANGELOG.md` and `book/src/appendix/changelog.md`).
- [ ] Verify all required checks pass locally:
  - `cargo fmt --all --check`
  - `cargo clippy --all -- -D warnings`
  - `cargo test --all-features`
- [ ] Decide and set release-surface mode (`RELEASE_SURFACE_MODE=fixed` or `auto`) and optional `RELEASE_CRATE_FILE` before validating.
- [ ] Decide fixed-mode release-surface strictness (`strict_publish_surface` in workflow or `STRICT_PUBLISH_SURFACE` locally) before running the Release workflow.
- [ ] If using manual Release workflow dispatch, set `release_surface_mode` and `release_crate_file` as needed.
- [ ] Run release validation for crates scheduled for publish (`cargo publish --dry-run -p <crate>`).
- [ ] Confirm docs and migration notes are updated for any API/behavior changes.
- [ ] Get maintainer sign-off for compatibility notes and known issues before tagging.

## Architecture Overview

Adze is a Rust workspace with multiple interconnected crates:

### Core Crates

| Crate | Location | Purpose |
|-------|----------|---------|
| `adze` | `/runtime/` | Main runtime library |
| `adze-glr-core` | `/glr-core/` | GLR parser generation |
| `adze-ir` | `/ir/` | Grammar intermediate representation |
| `adze-tablegen` | `/tablegen/` | Table generation and compression |

### Supporting Crates

| Crate | Location | Purpose |
|-------|----------|---------|
| `adze-macro` | `/macro/` | Procedural macros |
| `adze-tool` | `/tool/` | Build-time code generation |
| `adze-common` | `/common/` | Shared utilities |
| `example` | `/example/` | Example grammars and tests |
| `ts-bridge` | `/tools/ts-bridge/` | Tree-sitter to GLR bridge |

### Key Design Patterns

1. **Two-stage processing**: Macros mark types at compile time; the build tool generates parsers at build time
2. **GLR parsing**: Multi-action cells enable parsing ambiguous grammars without manual conflict resolution
3. **Feature-gated backends**: `tree-sitter-c2rust` (pure Rust, WASM-compatible) or `tree-sitter-standard` (C runtime)

## Testing Guidelines

- **Grammar tests**: Add to `/example/src/` with snapshot tests
- **GLR tests**: Test parser generation in `/glr-core/`
- **Integration tests**: Validate with real grammars via golden tests
- **Feature flag tests**: Ensure all feature combinations work

```bash
cargo test --features external_scanners
cargo test --features incremental_glr
cargo test --all-features
./scripts/check-test-connectivity.sh    # Verify all tests are connected
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ADZE_EMIT_ARTIFACTS=true` | Output generated grammar files for debugging |
| `ADZE_LOG_PERFORMANCE=true` | Enable GLR performance logging |
| `RUST_TEST_THREADS=N` | Limit test thread concurrency |
| `RAYON_NUM_THREADS=N` | Control rayon thread pool size |

## Getting Help

- Browse [open issues](https://github.com/EffortlessMetrics/adze/issues) and [discussions](https://github.com/EffortlessMetrics/adze/discussions)
- Check `CLAUDE.md` for detailed architecture information
- Run tests with `--nocapture` for debug output
- Use `RUST_LOG=debug` for verbose logging
