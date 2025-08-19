# Developer Guide - rust-sitter

## Quick Commands Cheat Sheet

### Default Build & Test
```bash
# Build all workspace members
cargo build --workspace

# Run all tests
cargo test --workspace

# Build with release optimizations
cargo build --workspace --release
```

### Strict Checks
```bash
# Enable strict API and documentation checks
cargo check -p rust-sitter --features "strict_docs strict_api"

# Run with all strict features
cargo test -p rust-sitter --features "strict_docs strict_api"
```

### Feature Testing
```bash
# Test ts-compat with pure-rust backend
cargo test -p rust-sitter --features "ts-compat pure-rust"

# Test incremental GLR feature
cargo test --workspace --features incremental_glr

# Test feature powerset (comprehensive)
cargo hack test -p rust-sitter --feature-powerset
```

### Benchmarks (Unstable)
```bash
# Build benchmarks without running (faster)
cargo bench -p rust-sitter --features unstable-benches --no-run

# Run benchmarks
cargo bench -p rust-sitter --features unstable-benches
```

### API Stability Checks
```bash
# Check for breaking changes locally (against baseline tag)
cargo semver-checks check-release -p rust-sitter --baseline-rev v0.8.0-dev.api-freeze-1

# Generate public API report
cargo public-api -p rust-sitter > public-api.txt

# Check API diff
cargo public-api --diff-git-checks origin/main...HEAD -p rust-sitter
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-targets

# Update dependencies
cargo update

# Check for security advisories
cargo deny check advisories
```

### Grammar Development
```bash
# Build a specific grammar
cargo build -p rust-sitter-python

# Test a grammar with snapshot tests
cargo test -p rust-sitter-example
cargo insta review  # Review snapshot changes

# Build with debug artifacts
RUST_SITTER_EMIT_ARTIFACTS=true cargo build -p rust-sitter-example
# Check artifacts in target/debug/build/<crate>-<hash>/out/
```

### ts-bridge Tool
```bash
# Build production version (requires Tree-sitter libs)
cargo build -p ts-bridge

# Build development version with stubs
cargo build -p ts-bridge --features stub-ts

# Run ABI verification
cargo run -p ts-bridge --bin tsb-abi-check

# Extract parse tables from grammar
cargo run -p ts-bridge -- path/to/grammar.so output.json tree_sitter_<lang>
```

## Making API Changes

### When You Need to Change the API

1. **Add new API (prefer additive changes)**
   - Keep old API with `#[deprecated(since = "0.9.0", note = "Use new_method instead")]`
   - Document migration path clearly

2. **Update documentation and examples**
   - Update all code examples
   - Add migration guide to CHANGELOG.md

3. **Bump version per semver**
   - Pre-1.0: minor version bump = breaking change
   - Post-1.0: major version bump = breaking change

4. **Move baseline tag after release**
   ```bash
   git tag -f v0.8.0-dev.api-freeze-1 <new-commit>
   git push --tags --force
   ```

### API Contract Files

- `runtime/tests/api_contract.rs` - Tests that enforce API stability
- `runtime/tests/doc_coverage.rs` - Documentation coverage tests
- `.github/workflows/ci.yml` - CI checks for breaking changes
- `scripts/check-breaking-changes.sh` - Local validation script

## Environment Variables

- `RUST_SITTER_EMIT_ARTIFACTS=true` - Output generated grammar files for debugging
- `RUST_LOG=debug` - Enable debug logging
- `RUST_BACKTRACE=1` - Show backtraces on panic

## Common Issues & Solutions

### Tests Disconnected
```bash
# Check for disabled tests
./scripts/check-test-connectivity.sh

# Re-enable test files
mv test.rs.disabled test.rs
```

### Breaking Change Detected
```bash
# If intentional, document in CHANGELOG and bump version
# Update Cargo.toml version
# Move baseline tag after release
```

### Feature Conflicts
```bash
# Some features are mutually exclusive
# Build specific packages when needed:
cargo build -p ts-bridge --features stub-ts
cargo build -p example --features pure-rust
```

## Release Checklist

1. [ ] Run full test suite: `cargo test --workspace --all-features`
2. [ ] Check for breaking changes: `cargo semver-checks check-release -p rust-sitter`
3. [ ] Update CHANGELOG.md
4. [ ] Bump versions in Cargo.toml files
5. [ ] Create release tag: `git tag v0.8.0`
6. [ ] Move API baseline tag if needed
7. [ ] Push tags: `git push --tags`
8. [ ] Publish to crates.io: `cargo publish -p rust-sitter`