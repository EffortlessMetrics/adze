# Developer Guide - rust-sitter

## Prerequisites

### System Requirements
- **Rust 1.89.0+** with 2024 edition support
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

### GLR Development Workflow
```bash
# Test GLR parser engine integration (runtime2 directory)
cd runtime2 && cargo test --features glr-core

# Test GLR with incremental parsing
cargo test -p rust-sitter-runtime --features incremental_glr

# Test all GLR feature combinations
cargo test --features "glr-core,incremental" --workspace

# Run GLR integration tests specifically 
cd runtime2 && cargo test glr_parse

# Test incremental parsing with performance monitoring  
RUST_SITTER_LOG_PERFORMANCE=true cargo test -p rust-sitter-runtime test_incremental

# Run GLR stress tests (if available) 
cd runtime2 && cargo test --release --features glr-core -- --ignored
```

### Performance Testing & Monitoring
```bash
# Enable performance logging during tests
export RUST_SITTER_LOG_PERFORMANCE=true

# Use capped testing for consistent benchmarks
cargo test-safe --features incremental_glr

# Monitor subtree reuse during incremental parsing
cargo test -p rust-sitter-runtime test_incremental_parsing_reuse_counter -- --nocapture

# Run performance regression tests
cargo bench --features incremental_glr -- incremental

# Test with concurrency caps for stability
RUST_TEST_THREADS=2 RAYON_NUM_THREADS=4 cargo test --workspace

# PR #58 Validation Testing - Node Metadata & Incremental Parsing
cargo test -p rust-sitter-runtime pr58_validation_test -- --nocapture
cargo test -p rust-sitter-runtime ts_compat_node_test -- --nocapture

# Test Direct Forest Splicing incremental algorithm
cargo test -p rust-sitter-runtime test_incremental_forest_splicing -- --nocapture

# Validate 16x performance improvements with performance logging
RUST_SITTER_LOG_PERFORMANCE=true cargo test -p rust-sitter-runtime incremental_glr_comprehensive_test -- --nocapture
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

## GLR Symbol Normalization Architecture

### Overview
The GLR parser implementation includes a comprehensive symbol normalization system that converts complex grammar symbols into auxiliary rules. This is required because GLR algorithms (FIRST/FOLLOW computation, LR item generation) expect symbols to be in normalized form.

### Normalization Pipeline
```
Original Grammar → Symbol Analysis → Auxiliary Rule Generation → GLR Processing
     ↓                    ↓                     ↓                    ↓
Complex Symbols    Detect Complex      Create _auxNNNN Rules    FIRST/FOLLOW
(Optional, Repeat) → Nested Patterns → (Terminal/NonTerminal) → Computation
```

### Symbol Transformation Patterns

#### 1. Optional Symbols (`Symbol::Optional`)
```rust
// Input:  rule -> symbol?
// Output: rule -> _aux1001
//         _aux1001 -> symbol
//         _aux1001 -> ε
```

#### 2. Repeat Symbols (`Symbol::Repeat`)  
```rust
// Input:  rule -> symbol*
// Output: rule -> _aux1002
//         _aux1002 -> _aux1002 symbol  // Left-recursive for efficiency
//         _aux1002 -> ε
```

#### 3. Sequence Symbols (`Symbol::Sequence`)
```rust
// Input:  rule -> (symbol1 symbol2 symbol3)
// Output: rule -> _aux1003
//         _aux1003 -> symbol1 symbol2 symbol3
```

#### 4. Choice Symbols (`Symbol::Choice`)
```rust
// Input:  rule -> (symbol1 | symbol2 | symbol3)
// Output: rule -> _aux1004
//         _aux1004 -> symbol1
//         _aux1004 -> symbol2  
//         _aux1004 -> symbol3
```

### Implementation Details

#### Auxiliary Symbol Management
- **Symbol ID Range**: Starts at `max_existing_id + 1000`, ends at `60000` (within u16 bounds)
- **Naming Convention**: `_aux{symbol_id}` for generated rule names  
- **Production ID Assignment**: Sequential allocation to avoid conflicts
- **Recursive Processing**: Handles nested complex symbols (e.g., `Optional(Repeat(...))`)

#### Integration Points
1. **Automatic Integration**: `FirstFollowSets::compute()` automatically normalizes grammars
2. **Manual Normalization**: `Grammar::normalize()` for explicit control
3. **Idempotency**: Multiple normalization calls have no effect
4. **Backward Compatibility**: Existing simple grammars work unchanged

### Testing and Debugging

#### Normalization Tests
```bash
# Run comprehensive normalization tests
cargo test -p rust-sitter-ir --test test_normalization

# Test specific transformation patterns
cargo test -p rust-sitter-ir test_optional_normalization
cargo test -p rust-sitter-ir test_repeat_normalization  
cargo test -p rust-sitter-ir test_nested_complex_symbols
```

#### Debug Commands
```bash
# View normalization artifacts
RUST_SITTER_EMIT_ARTIFACTS=true cargo test test_json_language_generation

# Debug auxiliary symbol creation
RUST_LOG=trace cargo test -p rust-sitter-ir normalization -- --nocapture

# Verify grammar structure after normalization
cargo test -p rust-sitter-ir grammar_invariants -- --nocapture
```

#### Performance Characteristics
- **Memory Usage**: 1-3 auxiliary rules per complex symbol
- **Runtime Impact**: Zero (normalization happens at compile-time)
- **Compilation Overhead**: Minimal (single grammar clone + transform)
- **Symbol ID Space**: Uses ~1000 IDs per complex grammar

### Error Recovery and Validation

#### Common Validation Errors
1. **Symbol ID Overflow**: Too many auxiliary symbols exceed u16 limit
2. **Recursive Definitions**: Self-referencing complex symbols  
3. **Production ID Conflicts**: Duplicate production assignments

#### Error Handling Strategy
```rust
match grammar.normalize() {
    Ok(()) => /* Continue with GLR processing */,
    Err(GrammarError::SymbolIdOverflow { max_id, requested_id }) => {
        // Reduce grammar complexity or increase ID space
    }
    Err(GrammarError::RecursiveDefinition { symbol, chain }) => {
        // Break recursive cycles or restructure grammar
    }
    Err(e) => /* Handle other grammar errors */
}
```

## Environment Variables

### Core Development
- `RUST_SITTER_EMIT_ARTIFACTS=true` - Output generated grammar files for debugging
- `RUST_LOG=debug` - Enable debug logging
- `RUST_BACKTRACE=1` - Show backtraces on panic

### GLR Performance & Monitoring
- `RUST_SITTER_LOG_PERFORMANCE=true` - Enable detailed GLR forest-to-tree conversion metrics
- `RUST_TEST_THREADS=N` - Control Rust test concurrency (default: 2 for stability)
- `RAYON_NUM_THREADS=N` - Limit rayon thread pool size (default: 4)
- `TOKIO_WORKER_THREADS=N` - Control tokio worker thread count (default: 2)
- `CARGO_BUILD_JOBS=N` - Limit cargo parallel build jobs (default: 4)

### Testing & Debugging
- `TIMEOUT=NNNs` - Set timeout for test scripts (e.g., `TIMEOUT=600s`)
- `RUST_SITTER_TEST_QUIET=true` - Reduce test output verbosity
- `RUST_SITTER_DISABLE_REUSE=true` - Disable subtree reuse for debugging

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
echo $RUST_SITTER_LOG_PERFORMANCE

# Monitor subtree reuse effectiveness
cargo test property_incremental_test -- --nocapture | grep -i reuse

# Use capped testing to avoid resource exhaustion
cargo test-safe --features incremental_glr
```

#### Incremental Parsing Not Working
```bash
# Verify feature flags are enabled
cargo test -p rust-sitter-runtime --features incremental_glr --lib

# Check for forest splicing vs full parse fallbacks
RUST_SITTER_LOG_PERFORMANCE=true cargo test -p rust-sitter-runtime test_incremental

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

#### Symbol Normalization Issues (Production Ready)
The GLR parser requires complex grammar symbols to be normalized into auxiliary rules. Common issues and solutions:

```bash
# Test for ComplexSymbolsNotNormalized errors
cargo test test_json_language_generation -p rust-sitter-tablegen

# Run normalization-specific tests
cargo test -p rust-sitter-ir --test test_normalization

# Debug normalization process
RUST_LOG=debug cargo test test_json_language_generation -p rust-sitter-tablegen

# Verify auxiliary symbol creation
cargo test -p rust-sitter-ir test_complex_symbol_normalization -- --nocapture
```

**Common Normalization Errors**:

1. **ComplexSymbolsNotNormalized Error**:
   ```
   Error: Complex symbols like 'Repeat(Sequence(...))' need normalization before FIRST/FOLLOW computation
   ```
   
   **Solution**: The GLR core automatically normalizes grammars, but if you see this error:
   ```bash
   # Verify GLR-core integration
   cargo test -p rust-sitter-glr-core first_follow_sets
   
   # Check grammar structure manually
   cargo test -p rust-sitter-ir grammar_normalization_idempotent
   ```

2. **SymbolIdOverflow Error**:
   ```
   Error: Too many auxiliary symbols created during normalization
   ```
   
   **Solution**: Reduce grammar complexity or optimize symbol usage:
   ```bash
   # Check grammar statistics
   cargo test -p rust-sitter-ir grammar_stats -- --nocapture
   
   # Optimize grammar before normalization
   cargo test -p rust-sitter-ir grammar_optimization
   ```

3. **Auxiliary Symbol Name Conflicts**:
   ```
   Error: Auxiliary symbol '_aux1001' conflicts with existing grammar
   ```
   
   **Solution**: Ensure auxiliary symbols start at safe offset:
   ```rust
   // Grammar should reserve symbol IDs > 1000 for auxiliary symbols
   let max_user_id = grammar.max_symbol_id();  // Should be < 1000
   ```

#### GLR Test Patterns and Expectations
GLR parsers produce trees with different structure than traditional parsers. When writing tests for GLR functionality, follow these patterns established in PR #64:

```bash
# GLR tests should expect grammar start symbol as root
# For JSON grammar with start rule "value":
assert_eq!(root.kind(), "value");  // Root is start symbol
let content = root.child(0);       // Content is child of start symbol
assert_eq!(content.kind(), "number" | "object" | "array");

# GLR tree navigation expects multi-level hierarchies  
let mut cursor = tree.root_node().walk();
assert_eq!(cursor.node().kind(), "value");      // Start at grammar root
assert!(cursor.goto_first_child());             // Navigate to content
assert_eq!(cursor.node().kind(), "array");      // Content type
assert!(cursor.goto_first_child());             // Navigate into content structure
assert_eq!(cursor.node().kind(), "lbracket");   // Terminal symbols
```

**Key GLR Testing Principles:**
- **Grammar-Compliant Structure**: Trees reflect grammar productions, not content-centric views
- **Start Symbol Root**: Root node is always the grammar's start symbol (e.g., `value`, `module`)  
- **Multi-Level Navigation**: Content appears as children of grammar symbols, not directly as root
- **Feature Gating**: Use `#![cfg(not(feature = "incremental_glr"))]` for tests not yet compatible
- **Proper Assertions**: Test both tree structure and content at appropriate levels

**Example Test Structure (from PR #64):**
```rust
// ✅ Correct: Grammar-compliant expectations
let root = tree.root_node();
assert_eq!(root.kind(), "value");           // Grammar start symbol
assert_eq!(root.child_count(), 1);          // Contains one child
let number_node = root.child(0).unwrap();   // Get content child
assert_eq!(number_node.kind(), "number");   // Verify content type

// ❌ Incorrect: Content-centric expectations  
let root = tree.root_node();
assert_eq!(root.kind(), "number");          // Wrong - expects content directly
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
2. [ ] Check for breaking changes: `cargo semver-checks check-release -p rust-sitter`
3. [ ] Update CHANGELOG.md
4. [ ] Bump versions in Cargo.toml files
5. [ ] Create release tag: `git tag v0.8.0`
6. [ ] Move API baseline tag if needed
7. [ ] Push tags: `git push --tags`
8. [ ] Publish to crates.io: `cargo publish -p rust-sitter`