# Developer Guide - adze

## Prerequisites

### System Requirements
- **Rust 1.92.0+** with 2024 edition support
- **libtree-sitter-dev**: Required for ts-bridge tool functionality
- **libclang-dev**: Required for some feature bindings
- **Git**: For version control and automated workflows

### Platform Support
- Linux (primary development)
- macOS (CI tested) 
- Windows (CI tested)
- WebAssembly targets

### Installation
```bash
# On Ubuntu/Debian
sudo apt-get install libtree-sitter-dev libclang-dev

# On macOS via Homebrew  
brew install tree-sitter

# On Windows
# Use vcpkg or manually install Tree-sitter development libraries
```

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
cargo check -p adze --features "strict_docs strict_api"

# Run with all strict features
cargo test -p adze --features "strict_docs strict_api"
```

### Feature Testing
```bash
# Test ts-compat with pure-rust backend
cargo test -p adze --features "ts-compat pure-rust"

# Test incremental GLR feature
cargo test --workspace --features incremental_glr

# Test feature powerset (comprehensive)
cargo hack test -p adze --feature-powerset
```

### GLR Development Workflow
```bash
# Test GLR parser engine integration (runtime2 directory)
cd runtime2 && cargo test --features glr-core

# Test GLR with incremental parsing
cargo test -p adze-runtime --features incremental_glr

# Test all GLR feature combinations
cargo test --features "glr-core,incremental" --workspace

# Run GLR integration tests specifically 
cd runtime2 && cargo test glr_parse

# Test incremental parsing with performance monitoring  
ADZE_LOG_PERFORMANCE=true cargo test -p adze-runtime test_incremental

# Run GLR stress tests (if available) 
cd runtime2 && cargo test --release --features glr-core -- --ignored
```

### Performance Testing & Monitoring
```bash
# Enable performance logging during tests
export ADZE_LOG_PERFORMANCE=true

# Use capped testing for consistent benchmarks
cargo test-safe --features incremental_glr

# Monitor subtree reuse during incremental parsing
cargo test -p adze-runtime test_incremental_parsing_reuse_counter -- --nocapture

# Run performance regression tests
cargo bench --features incremental_glr -- incremental

# Test with concurrency caps for stability
RUST_TEST_THREADS=2 RAYON_NUM_THREADS=4 cargo test --workspace
```

### Benchmarks (Unstable)
```bash
# Build benchmarks without running (faster)
cargo bench -p adze --features unstable-benches --no-run

# Run benchmarks
cargo bench -p adze --features unstable-benches
```

### API Stability Checks
```bash
# Check for breaking changes locally (against baseline tag)
cargo semver-checks check-release -p adze --baseline-rev v0.8.0-dev.api-freeze-1

# Generate public API report
cargo public-api -p adze > public-api.txt

# Check API diff
cargo public-api --diff-git-checks origin/main...HEAD -p adze
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
cargo build -p adze-python

# Test a grammar with snapshot tests
cargo test -p adze-example
cargo insta review  # Review snapshot changes

# Build with debug artifacts
ADZE_EMIT_ARTIFACTS=true cargo build -p adze-example
# Check artifacts in target/debug/build/<crate>-<hash>/out/
```

### ts-bridge Tool
```bash
# Build ts-bridge (requires Tree-sitter libs)
cargo build -p ts-bridge

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

### Core Development
- `ADZE_EMIT_ARTIFACTS=true` - Output generated grammar files for debugging
- `RUST_LOG=debug` - Enable debug logging
- `RUST_BACKTRACE=1` - Show backtraces on panic

### GLR Performance & Monitoring
- `ADZE_LOG_PERFORMANCE=true` - Enable detailed GLR forest-to-tree conversion metrics
- `RUST_TEST_THREADS=N` - Control Rust test concurrency (default: 2 for stability)
- `RAYON_NUM_THREADS=N` - Limit rayon thread pool size (default: 4)
- `TOKIO_WORKER_THREADS=N` - Control tokio worker thread count (default: 2)
- `CARGO_BUILD_JOBS=N` - Limit cargo parallel build jobs (default: 4)

### Testing & Debugging
- `TIMEOUT=NNNs` - Set timeout for test scripts (e.g., `TIMEOUT=600s`)
- `ADZE_TEST_QUIET=true` - Reduce test output verbosity
- `ADZE_DISABLE_REUSE=true` - Disable subtree reuse for debugging

## Common Issues & Solutions

### Tests Disconnected
```bash
# Check for disabled tests
./scripts/check-test-connectivity.sh

# Re-enable test files
mv test.rs.disabled test.rs
```

### GLR Runtime Issues

#### Performance Problems
```bash
# Check if performance logging is enabled
echo $ADZE_LOG_PERFORMANCE

# Monitor subtree reuse effectiveness
cargo test property_incremental_test -- --nocapture | grep -i reuse

# Use capped testing to avoid resource exhaustion
cargo test-safe --features incremental_glr
```

#### Incremental Parsing Not Working
```bash
# Verify feature flags are enabled
cargo test -p adze-runtime --features incremental_glr --lib

# Check for forest splicing vs full parse fallbacks
ADZE_LOG_PERFORMANCE=true cargo test -p adze-runtime test_incremental

# Test with simplified input to isolate issues
cargo test glr_incremental_reuse
```

#### GLR Engine Integration Issues
```bash
# Verify GLR core is properly linked
cd runtime2 && cargo test --features glr-core basic

# Check language validation
cd runtime2 && cargo test glr_parse_simple

# Test forest-to-tree conversion  
cd runtime2 && cargo test --features glr-core -- forest
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
cargo build -p example --features pure-rust
```

## Release Checklist

1. [ ] Run full test suite: `cargo test --workspace --all-features`
2. [ ] Check for breaking changes: `cargo semver-checks check-release -p adze`
3. [ ] Update CHANGELOG.md
4. [ ] Bump versions in Cargo.toml files
5. [ ] Create release tag: `git tag v0.8.0`
6. [ ] Move API baseline tag if needed
7. [ ] Push tags: `git push --tags`
8. [ ] Publish to crates.io: `cargo publish -p adze`

## GLR Grammar Normalization Development (v0.6.0)

### Enhanced SymbolMetadata Testing
```bash
# Test new SymbolMetadata fields and validation
cargo test -p adze-ir test_symbol_metadata_complete -- --nocapture

# Validate GLR-specific symbol classification
cargo test -p adze-glr-core test_symbol_classification

# Test is_extra, is_fragile, and is_terminal fields
cargo test -p adze test_symbol_metadata_fields

# Validate symbol_id assignment and uniqueness
cargo test -p adze-common test_symbol_id_uniqueness
```

### Memory Safety Development Practices
```rust
// Always validate spans before access (v0.6.0+)
fn safe_span_access(input: &[u8], start: usize, end: usize) -> Result<&[u8], ParseError> {
    if start <= end && end <= input.len() {
        Ok(&input[start..end])
    } else {
        Err(ParseError::InvalidSpan { start, end, len: input.len() })
    }
}

// Use enhanced SymbolMetadata with validation
let metadata = SymbolMetadata {
    name: "test_symbol".to_string(),
    visible: true,
    is_extra: false,
    is_fragile: false, 
    is_terminal: true,
    symbol_id: SymbolId::new(42),
};
metadata.validate()?; // Comprehensive validation
```

### FFI Safety Guidelines (v0.6.0)
- **Use Safe Mock Language**: All FFI testing now uses safe mock language approach
- **Proactive Bounds Checking**: Validate all span operations before use
- **Memory-Safe Struct Generation**: Enhanced validation in generated structures
- **Comprehensive Error Recovery**: Robust error handling prevents memory violations

## Summary - v0.6.0 Enhancements

This developer guide reflects the major improvements in adze v0.6.0:

- **Memory Safety Breakthrough**: Eliminated FFI segmentation faults through comprehensive safety improvements
- **GLR Grammar Normalization**: Enhanced SymbolMetadata with new fields for complete symbol classification  
- **Code Quality**: Resolved all clippy warnings and implemented consistent formatting
- **Test Infrastructure**: Enhanced coverage with 55+ GLR tests, 127+ runtime tests, and comprehensive safety validation

Follow these updated practices for safe, effective development in the adze ecosystem.
