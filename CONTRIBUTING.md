# Contributing to rust-sitter

Thank you for your interest in contributing to rust-sitter! This guide will help you get started with development.

## Setup

### Enable Git Hooks
First, configure git to use the repository's pre-commit hooks:

```bash
git config --local core.hooksPath .githooks
```

## Daily Development Commands

### Formatting
```bash
cargo fmt --all --check
```

### Clippy (fast, core crates only)
```bash
cargo clippy -p rust-sitter -p rust-sitter-glr-core -p rust-sitter-ir -p rust-sitter-tablegen --lib -- -D warnings
```

### Running Tests

Test helpers are centralized in the `glr-test-support` crate and shared across crates.

```bash
# Test core crates
cargo test -p rust-sitter -p rust-sitter-glr-core -p rust-sitter-ir -p rust-sitter-tablegen --lib

# Test with output visible
cargo test -p rust-sitter-glr-core invariants -- --nocapture

# Run a specific test by name
cargo test -p rust-sitter-glr-core parse_table_invariants_minimal_table -- --nocapture

# List available tests
cargo test -p rust-sitter-glr-core -- --list
```

### Update Snapshot Tests
```bash
cargo insta review
```

## Pre-commit Hook Options

The pre-commit hook runs automatically on commit. You can control its behavior with environment variables:

```bash
# Default (fast) - only core crates
.githooks/pre-commit

# Also lint test code
CLIPPY_TESTS=1 .githooks/pre-commit

# Enforce strict documentation
STRICT_DOCS=1 .githooks/pre-commit

# Make rustc warnings fatal
RUSTC_WARN_FATAL=1 .githooks/pre-commit

# Combine options
CLIPPY_TESTS=1 STRICT_DOCS=1 .githooks/pre-commit
```

## Architecture Overview

rust-sitter is a Rust workspace with multiple interconnected crates:

### Core Crates (strict quality requirements)
- **`rust-sitter`** (runtime) - Main runtime library at `/runtime/`
- **`rust-sitter-glr-core`** - GLR parser generation core at `/glr-core/`
- **`rust-sitter-ir`** - Grammar intermediate representation at `/ir/`
- **`rust-sitter-tablegen`** - Table generation and compression at `/tablegen/`

### Supporting Crates
- **`rust-sitter-macro`** - Procedural macros at `/macro/`
- **`rust-sitter-tool`** - Build-time code generation at `/tool/`
- **`rust-sitter-common`** - Shared utilities at `/common/`
- **`example`** - Example grammars and tests at `/example/`
- **`ts-bridge`** - Tree-sitter to GLR bridge at `/tools/ts-bridge/`

## Key Invariants

### ParseTable Construction
When working with `ParseTable`, these invariants must be maintained:
- `EOF` column index **must equal** `token_count + external_token_count`
- `ERROR` lives at column 0; terminals occupy the next `token_count` columns
- `initial_state` must be in range of `state_count`
- `start_symbol` must be a nonterminal present in `nonterminal_to_index`

### GLR Parser
The GLR (Generalized LR) parser supports:
- Multiple actions per state/symbol pair (ActionCell model)
- Runtime forking on shift/reduce and reduce/reduce conflicts
- Ambiguous grammar parsing without manual resolution

## Testing Guidelines

### Test Organization
1. **Grammar Tests**: Add to `/example/src/` with snapshot tests
2. **Compression Tests**: Verify Tree-sitter compatibility in tablegen
3. **FFI Tests**: Ensure Language struct ABI compatibility
4. **Integration Tests**: Validate with real grammars

### Running Specific Test Suites
```bash
# Run invariant tests
cargo test -p rust-sitter-glr-core invariants

# Run with specific features
cargo test --features external_scanners
cargo test --features incremental_glr
cargo test --all-features

# Check test connectivity
./scripts/check-test-connectivity.sh
```

## Environment Variables

### Build-time
- `RUST_SITTER_EMIT_ARTIFACTS=true` - Output generated files to `target/debug/build/` for debugging
- `CARGO_TARGET_DIR` - Override build directory (pre-commit uses `target/precommit`)

### Testing
- `RUST_LOG=debug` - Enable debug logging
- `RUST_BACKTRACE=1` - Show backtraces on panic

## Code Style Guidelines

1. **Follow existing patterns** - Match the style of surrounding code
2. **Use existing libraries** - Check package.json/Cargo.toml before adding dependencies
3. **No unnecessary comments** - Code should be self-documenting
4. **Security** - Never commit secrets or expose sensitive data
5. **Prefer editing over creating** - Modify existing files when possible

### Debug Print Hygiene

- Prefer `debugln!(...)` (feature-gated) over raw `eprintln!/println!/dbg!`
- If you temporarily comment a multi-line debug macro, close with `// );`
- Check locally: `python3 tools/check_debug_blocks.py`
- Auto-fix: `python3 tools/check_debug_blocks.py --fix`
- Check only staged files: `python3 tools/check_debug_blocks.py --changed-only`
- Check changes since a commit: `python3 tools/check_debug_blocks.py --since main`

## Submitting Changes

1. Run the pre-commit hook to ensure code quality
2. Update tests if behavior changes
3. Update snapshots with `cargo insta review` if needed
4. Write clear commit messages following conventional commits:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `chore:` for maintenance tasks
   - `test:` for test changes
   - `docs:` for documentation

## Common Tasks

### Adding a New Grammar
1. Create the grammar in `/example/src/`
2. Add corresponding tests with snapshots
3. Run `cargo test -p example` to verify
4. Update snapshots with `cargo insta review`

### Working on GLR Parser
1. IR changes go in `/ir/src/`
2. Parser generation logic in `/glr-core/src/`
3. Table compression in `/tablegen/src/`
4. Test with `cargo test -p rust-sitter-glr-core`

### Debugging Table Generation
```bash
# Enable artifact emission
RUST_SITTER_EMIT_ARTIFACTS=true cargo build

# Check generated files
ls target/debug/build/*/out/
```

## Getting Help

- Check existing issues and discussions on GitHub
- Review the CLAUDE.md file for detailed architecture information
- Run tests with `--nocapture` to see debug output
- Use `RUST_LOG=debug` for verbose logging

## Performance Counters

The `glr-core` crate includes optional performance counters for tracking parser operations:

```bash
# Enable perf counters
cargo test --features perf-counters

# Use in benchmarks
cargo bench --features perf-counters
```

When enabled, the counters track:
- Shifts: Token consumption operations
- Reductions: Rule application operations  
- Forks: GLR parser fork points
- Merges: GLR parser merge operations

## Local benches & feature flags

Some benches depend on evolving APIs and are **opt-in** behind a feature:

```bash
# run unstable benches locally
cargo bench -p rust-sitter --features unstable-benches
```

To smoke-test the Tree-sitter compatibility layer:

```bash
# ts-compat runtime smoke (pure-Rust backend)
cargo test -p rust-sitter --features "ts-compat pure-rust"
```

Strict interface/docs checks:

```bash
cargo check -p rust-sitter --features "strict_docs strict_api"
```

## Benchmarking

### Quick Benchmarks (Development)
```bash
# Fast iteration during development
./scripts/bench-quick.sh

# With additional arguments
./scripts/bench-quick.sh -- --save-baseline my-change
```

### Full Benchmarks (Baselines)
```bash
# Save baseline before changes
cargo bench -p rust-sitter-glr-core --features perf-counters -- --save-baseline before

# Make changes, then compare
cargo bench -p rust-sitter-glr-core --features perf-counters -- --save-baseline after

# Compare baselines with critcmp
critcmp before after
```

### Feature Flags in CI
The CI tests with these feature combinations:
- `perf-counters` - Performance counter tracking
- `test-api` - Internal testing APIs
- All features enabled together
- No default features (core crates only)

## CI Configuration

### Code Quality Checks
The CI enforces:
- **Format**: `cargo fmt --all -- --check`
- **Clippy**: `cargo clippy` with `-D warnings`
- **Docs**: `cargo doc` with `RUSTDOCFLAGS=-D warnings`
- **Compilation**: `cargo check` with `RUSTFLAGS=-D warnings`

### No-Default-Features Testing
The following core crates are tested without default features:
- `rust-sitter-glr-core`
- `rust-sitter-ir`
- `rust-sitter-tablegen`
- `rust-sitter-common`
- `rust-sitter-macro`

This list is maintained in `.github/workflows/core-tests.yml` as the `CORE_CRATES_NO_DEFAULT` environment variable.

### Cross-Platform Testing
The ts-bridge smoke tests run on:
- Ubuntu (latest)
- macOS (latest)
- Windows (latest)

The smoke test verifies symbol exports and linkage across all platforms.

### Fast Benches
- Quick iteration: `./scripts/bench-quick.sh` - Runs benchmarks with `BENCH_QUICK=1` for fast feedback
- Save baselines: `cargo bench ... -- --save-baseline NAME` for comparison
- Compare results: `critcmp before after` to see performance changes

### CI Configuration Notes
- The `CORE_CRATES_NO_DEFAULT` environment variable lives in `.github/workflows/core-tests.yml`
- `RUSTFLAGS=-D warnings` is enforced in CI to catch all warnings
- `cargo fmt --check` runs automatically to ensure consistent formatting
- All cargo commands use `--locked` to ensure reproducible builds