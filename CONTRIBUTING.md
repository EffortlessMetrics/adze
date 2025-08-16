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