# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Use TDD. Red-Green-Refactor, spec driven design. User-story driven design.

## Common Development Commands

### Building
```bash
# Build all workspace members
cargo build

# Build with release optimizations
cargo build --release

# Build a specific package
cargo build -p rust-sitter
cargo build -p rust-sitter-macro
cargo build -p rust-sitter-tool
```

### Testing
```bash
# Run all tests in the workspace
cargo test

# Run tests for a specific package
cargo test -p rust-sitter
cargo test -p rust-sitter-macro
cargo test -p rust-sitter-tool

# Run a specific test
cargo test test_name

# Run tests with output displayed
cargo test -- --nocapture

# Update snapshot tests (uses insta)
cargo insta review

# For integration tests that need internal debug helpers, enable the test-api feature:
cargo test -p rust-sitter-glr-core --features test-api

# Concurrency-capped testing (recommended for stability)
cargo t2                    # Run tests with 2 threads
cargo test-safe            # Run tests with safe defaults
cargo test-ultra-safe      # Run tests with 1 thread
./scripts/test-capped.sh   # Run tests with automatic concurrency detection
./scripts/test-local.sh    # Local test runner with nextest fallback
```

### Linting and Formatting
```bash
# Run clippy on all workspace members
cargo clippy --all

# Run clippy and fail on warnings
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

## Architecture Overview

Rust Sitter is a Rust workspace consisting of multiple interconnected crates that work together to generate Tree-sitter parsers from Rust code annotations:

### Core Components

1. **`rust-sitter` (runtime crate)** - The main runtime library that users depend on
   - Located in `/runtime/`
   - Provides the `Extract` trait and core parsing functionality
   - Supports two Tree-sitter backends via features:
     - `tree-sitter-c2rust` (default): Pure Rust implementation for WASM support
     - `tree-sitter-standard`: Standard C runtime

2. **`rust-sitter-macro` (proc-macro crate)** - Procedural macros for grammar definition
   - Located in `/macro/`
   - Provides attributes: `#[rust_sitter::grammar]`, `#[rust_sitter::language]`, `#[rust_sitter::leaf]`, etc.
   - Only defines the macro interfaces; actual expansion logic is in common crate

3. **`rust-sitter-tool` (build tool)** - Build-time code generation
   - Located in `/tool/`
   - Called from `build.rs` to generate Tree-sitter grammar JSON and C parser code
   - Key function: `build_parsers()` which processes annotated Rust files

4. **`rust-sitter-common`** - Shared utilities
   - Located in `/common/`
   - Contains grammar expansion logic used by both macro and tool crates

5. **`example`** - Example grammars and usage patterns
   - Located in `/example/`
   - Contains arithmetic, optional, repetition, and word grammar examples
   - Uses snapshot testing with `insta` for parser output verification

### Pure-Rust Implementation Status

**Major Achievement (August 2025)**: The pure-Rust implementation successfully compiles the Python grammar with:
- 273 symbols
- 57 fields  
- Full external scanner support for indentation tracking
- FFI-compatible Tree-sitter `LANGUAGE` struct generation

This demonstrates that the pure-Rust toolchain can handle production-grade, complex grammars with external scanners.

### New Pure-Rust Implementation Components

6. **`rust-sitter-ir`** - Grammar Intermediate Representation
   - Located in `/ir/`
   - Defines the IR for representing grammars with GLR support
   - Supports precedence, associativity, field mappings, and fragile tokens
   - Includes grammar optimization (`optimizer.rs`)
   - Includes grammar validation (`validation.rs`)

7. **`rust-sitter-glr-core`** - GLR Parser Generation Core
   - Located in `/glr-core/`
   - Implements FIRST/FOLLOW set computation
   - LR(1) item sets and canonical collection building
   - Conflict detection and GLR fork/merge logic
   - Advanced conflict resolution strategies (`advanced_conflict.rs`)

8. **`rust-sitter-tablegen`** - Table Generation and Compression
   - Located in `/tablegen/`
   - Implements Tree-sitter's table compression algorithms
   - Generates static Language objects with FFI compatibility
   - Produces NODE_TYPES JSON metadata

### Enhanced Runtime Components

The runtime crate (`/runtime/`) now includes:
- **`error_recovery.rs`** - Comprehensive error recovery strategies
- **`visitor.rs`** - Parse tree visitor API for traversal and analysis
- **`serialization.rs`** - Tree serialization in multiple formats

The runtime2 crate (`/runtime2/`) now includes:
- **`tree.rs`** - Enhanced Tree implementation with incremental editing support
  - Feature-gated incremental parsing via `#[cfg(feature = "incremental")]`
  - Comprehensive `EditError` handling for overflow/underflow protection
  - Deep cloning support for non-destructive tree analysis
  - Checked arithmetic operations to prevent integer vulnerabilities
  - Tree cursor API for efficient traversal

The tool crate (`/tool/`) now includes:
- **`visualization.rs`** - Grammar and tree visualization tools

9. **`ts-bridge`** - Tree-sitter to GLR Bridge Tool
   - Located in `/tools/ts-bridge/`
   - Extracts parse tables from compiled Tree-sitter grammars
   - Features ABI stability guards (v15 pinning with SHA verification)
   - Supports feature-gated development (stub) and production builds
   - Includes comprehensive parity testing framework

### Key Design Patterns

1. **Grammar Definition Flow**:
   - User defines grammar using Rust types with macro annotations
   - `build.rs` calls `rust_sitter_tool::build_parsers()` at build time
   - Tool extracts grammar from Rust code and generates Tree-sitter JSON grammar
   - Tree-sitter generates C parser from JSON
   - C parser is compiled and linked into the final binary

2. **Two-Stage Processing**:
   - Compile-time: Macros mark types but don't generate parser code
   - Build-time: Tool reads the marked types and generates actual parser

3. **Incremental Parsing Flow** (PR #28):
   - Trees support in-place editing via `Tree::edit()` for efficient incremental parsing
   - Edit operations validate ranges and use checked arithmetic to prevent overflow/underflow
   - Nodes affected by edits are marked as "dirty" for selective re-parsing
   - Deep cloning enables safe analysis without affecting original trees
   - Feature-gated implementation allows optional dependency on incremental parsing

   **Memory Safety Improvements**:
   - All position arithmetic uses `checked_add()` and `checked_sub()` to prevent integer overflow/underflow
   - Range validation prevents invalid edit operations that could corrupt tree structure
   - Comprehensive `EditError` enum provides specific error types for debugging
   - Recursive tree operations are bounded to prevent stack overflow on malformed inputs
   - Deep cloning creates fully independent tree copies without shared references

4. **Environment Variables**:
   - `RUST_SITTER_EMIT_ARTIFACTS=true`: Outputs generated grammar files to `target/debug/build/<crate>-<hash>/out/` for debugging

### Working with the Codebase

When making changes:
1. Grammar expansion logic is shared between macro and tool in the `common` crate
2. The macro crate only provides attribute definitions, not implementations
3. The tool crate handles all build-time code generation
4. Test changes using the example crate which has comprehensive snapshot tests
5. Use `cargo insta review` to update snapshots when grammar output changes intentionally

### Pure-Rust Implementation Development

When working on the pure-Rust implementation:
1. The IR crate defines the grammar representation - modify this for new grammar features
2. The GLR core implements the parser generation algorithms - this is where conflict resolution happens
3. The tablegen crate handles compression - ensure bit-for-bit compatibility with Tree-sitter
4. Use `emit_ir!()` macro to debug grammar extraction
5. Test table generation with `cargo test -p rust-sitter-tablegen`
6. Verify Language struct layout matches Tree-sitter ABI exactly

### Testing Guidelines

1. **Grammar Tests**: Add new grammars to `/example/src/` with corresponding snapshot tests
2. **Compression Tests**: Verify table compression maintains Tree-sitter compatibility
3. **FFI Tests**: Ensure generated Language structs match C ABI requirements
4. **Integration Tests**: Test with real Tree-sitter grammars for validation

### Cap Concurrency Implementation

**Goal:** Eliminate fork/PID/file-descriptor storms and stabilize E2E/visual + unit tests across machines by bounding concurrency.

**Implementation:**
```bash
# Use capped test aliases
cargo t2                    # Run tests with 2 threads
cargo test-safe            # Run tests with safe defaults
cargo test-ultra-safe      # Run tests with 1 thread

# Use preflight script for system pressure monitoring
scripts/preflight.sh       # Check system pressure and set caps
scripts/test-capped.sh     # Run tests with automatic concurrency caps
scripts/test-local.sh      # Local test runner with nextest fallback and timeout

# Container limits (optional)
docker-compose -f docker-compose.test.yml up rust-tests
```

**Concurrency Defaults:**
- Rust test threads: **2** (via `RUST_TEST_THREADS`)
- Rayon thread pool: **4** (via `RAYON_NUM_THREADS`)
- Tokio worker threads: **2** (via `TOKIO_WORKER_THREADS`)
- Tokio blocking threads: **8** (via `TOKIO_BLOCKING_THREADS`)
- Cargo build jobs: **4** (via `CARGO_BUILD_JOBS`)
- Scientific libs (BLAS): **1** thread each (prevents CPU storms)

**Environment Variables:**
All caps are configurable via environment variables. The `preflight.sh` script automatically degrades to ultra-safe mode (all caps = 1) if the system is under high PID pressure (>85% of pid_max).

**Code Integration:**
```rust
// In test setup or main application:
use rust_sitter::concurrency_caps;
concurrency_caps::init_concurrency_caps(); // Set up capped thread pools

// For bounded parallel operations:
let results = concurrency_caps::bounded_parallel_map(items, 4, |x| process(x));
```

**CI Integration:**
The CI pipeline automatically uses these caps via environment variables set in `.github/workflows/ci.yml`. All `cargo test` commands include `-- --test-threads=$RUST_TEST_THREADS`.

**Troubleshooting Concurrency Issues:**

*Problem*: Tests fail with "Too many open files" or "Cannot create thread"
*Solution*: 
```bash
# Check system pressure
./scripts/preflight.sh

# Use ultra-safe mode
cargo test-ultra-safe

# Check actual caps being used
env | grep -E "(RUST_TEST|RAYON|TOKIO|CARGO)_"
```

*Problem*: Slow test execution or timeouts  
*Solution*:
```bash
# Use local test runner with timeout handling
./scripts/test-local.sh

# Or specify timeout manually
TIMEOUT=600s ./scripts/test-local.sh
```

*Problem*: Inconsistent test results across machines
*Solution*:
```bash
# Use capped testing consistently
./scripts/test-capped.sh

# Or set explicit caps
RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 cargo test
```

### Test Connectivity Safeguards

The project includes multiple layers of protection to prevent tests from being silently disconnected or disabled:

#### 1. CI Test Connectivity Job
The `.github/workflows/ci.yml` includes a `test-connectivity` job that:
- **Blocks commits** containing `.rs.disabled` files (hard failure)
- **Enforces non-zero test counts** for all crates across all feature combinations
- **Reports per-crate test counts** in PR summaries for easy comparison
- **Detects orphaned test files** that might not be connected to the test harness
- **Surfaces `#[ignore]` tests** for visibility
- Runs for all feature combinations: `default`, `external_scanners`, `incremental_glr`, and `all-features`

#### 2. Pre-commit Hook
Located at `.git/hooks/pre-commit`, this hook:
- Prevents accidentally committing `.rs.disabled` files
- Warns about existing disabled files in the repository
- Suggests using `#[ignore]` attribute instead of file renaming

#### 3. Local Verification Script
The `scripts/check-test-connectivity.sh` script allows developers to:
- Check for disabled test files
- Count tests per feature set
- Report per-crate test discovery
- Find `#[ignore]` tests
- Detect potentially orphaned test modules
- Get actionable recommendations for test health

#### Currently Disabled Tests
The following test files are currently disabled and need attention:
- `runtime/tests/golden_tests.rs.disabled`
- `runtime/tests/test_complete_example.rs.disabled`
- `runtime/tests/test_glr_parsing.rs.disabled`
- `runtime/tests/test_pure_rust_e2e.rs.disabled`
- `runtime/tests/test_pure_rust_real_grammar.rs.disabled`
- `runtime/tests/test_query_predicates.rs.disabled`

**Action Required**: These files should either be:
1. Re-enabled by removing the `.disabled` suffix and fixing any issues
2. Marked with `#[ignore]` if they need to remain disabled temporarily
3. Removed if they are no longer relevant

To check test connectivity locally, run:
```bash
./scripts/check-test-connectivity.sh
```

### Recent Achievements (August 2025)

#### **GLR Parser Implementation - Production Ready** ✅
Successfully transformed rust-sitter from a simple LR parser to a true GLR (Generalized LR) parser that can handle ambiguous grammars. The implementation is now production-ready with comprehensive API stabilization and infrastructure improvements.

**Key Technical Changes:**
1. **Action Table Architecture**: Restructured from `Vec<Vec<Action>>` to `Vec<Vec<Vec<Action>>>` (ActionCell model)
   - Each cell can now hold multiple conflicting actions
   - Enables runtime forking when shift/reduce or reduce/reduce conflicts occur
   - Maintains all valid parse paths simultaneously

2. **Python Grammar Success**: Fixed critical "State 0" bug
   - **Problem**: Python files starting with `def` couldn't be parsed due to single-action limitation
   - **Root Cause**: Empty module rule `REPEAT(_statement)` creates shift/reduce conflict in state 0
   - **Solution**: GLR parser now maintains both shift and reduce actions, handling:
     - Empty Python files (reduce to empty module)
     - Files starting with statements (shift the token)
     - All 273 symbols with 57 fields compile correctly
     - Full external scanner support for indentation

3. **Comprehensive Implementation**: Updated 20+ files across the codebase
   - Core parser logic in `glr-core/lib.rs`
   - Table compression in `tablegen/compress.rs`
   - Runtime decoders in `runtime/decoder.rs` and all parser implementations
   - Error recovery, incremental parsing, and visitor patterns all updated

4. **Infrastructure Stabilization (August 2025)**:
   - **SymbolMetadata API Standardization**: Field names unified (`is_visible` → `visible`, `is_terminal` → `terminal`) with new GLR-specific fields for enhanced metadata support
   - **Concurrency Caps System**: Implemented bounded thread pools and resource management to eliminate fork/PID storms and ensure stable testing across machines
   - **Test Runner Infrastructure**: Added `scripts/preflight.sh`, `scripts/test-capped.sh`, and `scripts/test-local.sh` for reliable test execution
   - **Grammar Loading Pipeline**: Completed parse table generation infrastructure for production use

### Previous Fixes (August 2025)

1. **Type System Alignment**: Fixed critical `SymbolId` type mismatch between crates
2. **External Scanner Integration**: Corrected `ScanResult` struct and scanner traits
3. **FFI Code Generation**: Fixed attribute syntax and external scanner signatures
4. **Symbol Registration**: Resolved symbol registration panics

### GLR Parser Architecture

The pure-Rust implementation now features a production-ready GLR parser:

1. **Multi-Action Cells**: Each state/symbol pair can have multiple valid actions
2. **Runtime Forking**: Parser dynamically forks on conflicts, exploring all valid paths
3. **Conflict Preservation**: Precedence/associativity order actions but don't eliminate them
4. **Ambiguity Support**: Can parse inherently ambiguous grammars without manual resolution

### What This Enables

- **Complex Language Support**: Can now parse languages like C++, Rust, and other ambiguous grammars
- **Better Error Recovery**: Multiple parse paths improve error recovery strategies
- **Research Applications**: Foundation for grammar inference and language analysis tools
- **WASM Compatibility**: Pure-Rust implementation enables browser-based parsing

### New Tools (January 2025)

#### ts-bridge: Tree-sitter to GLR Runtime Bridge
The ts-bridge tool extracts parse tables from compiled Tree-sitter grammars for use with rust-sitter's GLR runtime:

**Building:**
```bash
# Production build (with vendored headers)
cargo build -p ts-bridge

# Development build (with stubs for testing)
cargo build -p ts-bridge --features stub-ts

# Run ABI verification
cargo run -p ts-bridge --bin tsb-abi-check
./tools/ts-bridge/scripts/abi-hash.sh
```

**Testing:**
```bash
# Run basic tests (works with stubs)
cargo test -p ts-bridge --test basic --features stub-ts

# Run parity tests (requires actual Tree-sitter libraries)
cargo test -p ts-bridge --features with-grammars
```

**Usage:**
```bash
# Extract parse tables from a compiled grammar
cargo run -p ts-bridge -- path/to/libtree-sitter-json.so output.json tree_sitter_json
```

**Key Features:**
- ABI stability with Tree-sitter v15 (vendored headers + SHA verification)
- Dynamic buffer allocation (no truncation for large action cells)
- Feature-gated builds for development vs production
- Comprehensive parity testing against Tree-sitter

### Known Issues (Being Addressed)

1. **GLR Runtime Optimization**: Fork/merge logic needs performance tuning for large files
2. **External Scanner FFI**: Integration with C scanners needs final touches  
3. **Incremental Parsing**: GLR incremental parsing algorithms need implementation
4. **ts-bridge Linking**: Production builds need actual Tree-sitter library linking (undefined symbols)
5. **Disabled Test Re-enablement**: Several test files need to be re-enabled after GLR stabilization (see Test Connectivity section above)