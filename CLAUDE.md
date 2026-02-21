# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Use TDD. Red-Green-Refactor, spec driven design. User-story driven design.

## Requirements

### Minimum Rust Version (MSRV)
- **Rust 1.92.0** or later
- **Rust 2024 Edition** - all workspace crates use the latest edition
- Components: `rustfmt`, `clippy` (automatically configured via `rust-toolchain.toml`)

### System Dependencies
- **libtree-sitter-dev**: Required for ts-bridge tool (production mode)
- **libclang**: Required for binding generation in some features
- **Git**: Version control and automated testing workflows

### Supported Platforms
- Linux (primary development and CI)
- macOS (tested via CI)
- Windows (tested via CI)
- WebAssembly (wasm32-unknown-unknown, wasm32-wasi)

## Common Development Commands

### Building
```bash
# Build all workspace members
cargo build

# Build with release optimizations
cargo build --release

# Build a specific package
cargo build -p adze
cargo build -p adze-macro
cargo build -p adze-tool
```

### Testing
```bash
# Run all tests in the workspace
cargo test

# Run tests for a specific package
cargo test -p adze
cargo test -p adze-macro
cargo test -p adze-tool

# Run a specific test
cargo test test_name

# Run tests with output displayed
cargo test -- --nocapture

# Update snapshot tests (uses insta)
cargo insta review

# For integration tests that need internal debug helpers, enable the test-api feature:
cargo test -p adze-glr-core --features test-api

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

Adze is an AST-first grammar toolchain for Rust. Define the shape of your syntax in Rust, then parse into that shape. Build-time: types -> macros -> IR -> tables. Run-time: text -> GLR -> trees -> typed values.

Adze is a Rust workspace consisting of multiple interconnected crates that work together to generate Tree-sitter parsers from Rust code annotations:

### Core Components

1. **`adze` (runtime crate)** - The main runtime library that users depend on
   - Located in `/runtime/`
   - Provides the `Extract` trait and core parsing functionality
   - Supports two Tree-sitter backends via features:
     - `tree-sitter-c2rust` (default): Pure Rust implementation for WASM support
     - `tree-sitter-standard`: Standard C runtime

2. **`adze-macro` (proc-macro crate)** - Procedural macros for grammar definition
   - Located in `/macro/`
   - Provides attributes: `#[adze::grammar]`, `#[adze::language]`, `#[adze::leaf]`, etc.
   - Only defines the macro interfaces; actual expansion logic is in common crate

3. **`adze-tool` (build tool)** - Build-time code generation
   - Located in `/tool/`
   - Called from `build.rs` to generate Tree-sitter grammar JSON and C parser code
   - Key function: `build_parsers()` which processes annotated Rust files

4. **`adze-common`** - Shared utilities
   - Located in `/common/`
   - Contains grammar expansion logic used by both macro and tool crates

5. **`example`** - Example grammars and usage patterns
   - Located in `/example/`
   - Contains arithmetic, optional, repetition, and word grammar examples
   - Uses snapshot testing with `insta` for parser output verification

6. **`golden-tests`** - Integration testing with production grammars
   - Located in `/golden-tests/`
   - Tests adze-generated parsers against Tree-sitter reference implementations
   - Supports Python and JavaScript grammar validation
   - Uses SHA256 hash verification for parse tree consistency
   - Provides UPDATE_GOLDEN mode for reference generation

### Pure-Rust Implementation Status

**Major Achievement (August 2025)**: The pure-Rust implementation successfully compiles the Python grammar with:
- 273 symbols
- 57 fields  
- Full external scanner support for indentation tracking
- FFI-compatible Tree-sitter `LANGUAGE` struct generation

This demonstrates that the pure-Rust toolchain can handle production-grade, complex grammars with external scanners.

### New Pure-Rust Implementation Components

7. **`adze-ir`** - Grammar Intermediate Representation
   - Located in `/ir/`
   - Defines the IR for representing grammars with GLR support
   - Supports precedence, associativity, field mappings, and fragile tokens
   - Includes grammar optimization (`optimizer.rs`)
   - Includes grammar validation (`validation.rs`)

8. **`adze-glr-core`** - GLR Parser Generation Core
   - Located in `/glr-core/`
   - Implements FIRST/FOLLOW set computation
   - LR(1) item sets and canonical collection building
   - Conflict detection and GLR fork/merge logic
   - Advanced conflict resolution strategies (`advanced_conflict.rs`)

9. **`adze-tablegen`** - Table Generation and Compression
   - Located in `/tablegen/`
   - Implements Tree-sitter's table compression algorithms
   - Generates static Language objects with FFI compatibility
   - Produces NODE_TYPES JSON metadata

### Enhanced Runtime Components

The runtime crate (`/runtime/`) now includes:
- **`error_recovery.rs`** - Comprehensive error recovery strategies
- **`visitor.rs`** - Parse tree visitor API for traversal and analysis
- **`serialization.rs`** - Tree serialization in multiple formats

The runtime2 crate (`/runtime2/`) - **Production Ready GLR Runtime** - includes:
- **`parser.rs`** - GLR-compatible Parser API with Tree-sitter compatibility
  - Feature-gated GLR engine routing with `#[cfg(feature = "glr-core")]`
  - Automatic fallback to full parsing when incremental features are disabled
  - Comprehensive error handling and language validation
- **`engine.rs`** - GLR engine adapter and forest management
  - GLR-core Driver integration with parse table validation
  - Forest enum for GLR parse forest representation
  - Token processing pipeline with UTF-8 validation
- **`builder.rs`** - Forest-to-tree conversion with performance monitoring
  - Efficient conversion from GLR parse forests to Tree-sitter compatible trees
  - Performance instrumentation via `ADZE_LOG_PERFORMANCE` environment variable
  - Node count, tree depth, and conversion time metrics
- **`tree.rs`** - Enhanced Tree implementation with incremental editing support
  - Feature-gated incremental parsing via `#[cfg(feature = "incremental")]`
  - Comprehensive `EditError` handling for overflow/underflow protection
  - Deep cloning support for non-destructive tree analysis
  - Checked arithmetic operations to prevent integer vulnerabilities
  - Tree cursor API for efficient traversal

The tool crate (`/tool/`) now includes:
- **`visualization.rs`** - Grammar and tree visualization tools

10. **`ts-bridge`** - Tree-sitter to GLR Bridge Tool
   - Located in `/tools/ts-bridge/`
   - Extracts parse tables from compiled Tree-sitter grammars  
   - Features ABI stability guards (v15 pinning with SHA verification)
   - Production-ready with real Tree-sitter runtime linking (requires libtree-sitter-dev)
   - Includes comprehensive parity testing framework

### Key Design Patterns

1. **Grammar Definition Flow**:
   - User defines grammar using Rust types with macro annotations
   - `build.rs` calls `adze_tool::build_parsers()` at build time
   - Tool extracts grammar from Rust code and generates Tree-sitter JSON grammar
   - Tree-sitter generates C parser from JSON
   - C parser is compiled and linked into the final binary

2. **Two-Stage Processing**:
   - Compile-time: Macros mark types but don't generate parser code
   - Build-time: Tool reads the marked types and generates actual parser

3. **Incremental Parsing Flow** (PR #62 - Experimental, Currently Disabled):

   **IMPORTANT: Current Status**:
   The incremental parsing path is currently **disabled** and falls back to fresh parsing
   for consistency reasons. The architecture has known issues that cause behavioral
   differences between incremental and fresh parsing. See `glr_incremental.rs:281-297`
   for details.

   **Infrastructure (present but not active)**:
   - **Production API**: `Parser::reparse()` method exists in main parser API
   - **GLR Integration**: Automatic routing exists with fallback to fresh parsing
   - **Direct Forest Splicing**: Algorithm implemented but bypassed
   - **Feature-Gated**: Requires `incremental_glr` feature

   **Known Issues Preventing Enable**:
   - Error tracking: hardcoded is_error: false in subtree creation
   - Root kind determination: uses forest symbols vs actual parse results
   - Token-level vs grammar-level parsing differences

   **Algorithm Design (for future work)**:
   - **Chunk Identification**: Finds unchanged prefix/suffix token ranges
   - **Middle-Only Parsing**: Parses ONLY the edited middle segment
   - **Forest Extraction**: Extracts reusable nodes from old forest
   - **Surgical Splicing**: Combines prefix + new middle + suffix forests
   - **Conservative Reuse**: Only reuses subtrees completely outside edit ranges

   **Legacy Memory Safety** (PR #28 - runtime crate):
   - Trees support in-place editing via `Tree::edit()` for efficient incremental parsing
   - Edit operations validate ranges and use checked arithmetic to prevent overflow/underflow
   - Comprehensive `EditError` enum provides specific error types for debugging
   - Deep cloning creates fully independent tree copies without shared references

4. **Environment Variables**:
   - `ADZE_EMIT_ARTIFACTS=true`: Outputs generated grammar files to `target/debug/build/<crate>-<hash>/out/` for debugging
   - `ADZE_LOG_PERFORMANCE=true`: Enables performance logging for GLR forest-to-tree conversion
   - `RUST_TEST_THREADS=N`: Limits Rust test thread concurrency (default: 2 for stability)
   - `RAYON_NUM_THREADS=N`: Controls rayon thread pool size (default: 4)

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
5. Test table generation with `cargo test -p adze-tablegen`
6. Verify Language struct layout matches Tree-sitter ABI exactly

### Testing Guidelines

1. **Grammar Tests**: Add new grammars to `/example/src/` with corresponding snapshot tests
2. **Compression Tests**: Verify table compression maintains Tree-sitter compatibility
3. **FFI Tests**: Ensure generated Language structs match C ABI requirements
4. **Integration Tests**: Test with real Tree-sitter grammars for validation
5. **GLR Runtime Tests**: Test GLR integration and performance with `runtime2/tests/glr_parse.rs`
6. **GLR Incremental Parsing Tests** (Implementation Complete - September 2025):
   - **GLR Incremental Tests**: `runtime/src/glr_incremental.rs` - Tests GLR-aware incremental parsing with fork tracking
   - **External Scanner Integration**: `runtime/tests/external_scanner_test.rs` - Tests scanner lifecycle and range handling
   - **Tree Bridge Tests**: `runtime/tests/tree_bridge_test.rs` - Tests forest-to-tree conversion with grammar compliance
   - **Property-Based Tests**: `runtime/tests/property_incremental_test.rs` - Randomized incremental behavior validation
   - **Feature Flag Tests**: Ensures graceful fallback when `incremental_glr` disabled or external scanners unavailable
   - **Conservative Reuse**: Tests temporary fallback to fresh parsing for consistency guarantees
7. **Feature Flag Tests**: Test all feature combinations (`default`, `glr-core`, `incremental`, `incremental_glr`, `all-features`)
8. **Golden Tests**: Validate adze parsers against Tree-sitter reference implementations with `cargo test -p adze-golden-tests`
9. **Serialization Tests**: Comprehensive roundtrip testing for JSON and S-expression formats with `runtime/tests/test_serialization_roundtrip.rs`

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
use adze::concurrency_caps;
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

#### Test Status Update (January 2025)
All previously disabled test files have been successfully re-enabled and integrated back into the test suite. The test connectivity safeguards are now working correctly to prevent any tests from being silently disconnected in the future.

**Current Status**: All test files are properly connected and running. No `.rs.disabled` files exist in the repository.

To check test connectivity locally, run:
```bash
./scripts/check-test-connectivity.sh
```

### Completed Milestones

#### GLR Incremental Parsing *(September 2025)*
GLR-aware incremental parsing in `runtime/src/glr_incremental.rs` with fork-aware subtree reuse, ambiguity preservation, and direct forest splicing. External scanner integration (`runtime/src/external_scanner/`) provides pure-Rust `ExternalScanner` trait with C FFI compatibility. Tree bridge (`runtime/src/tree_bridge.rs`) handles grammar-compliant forest-to-tree conversion. Currently uses conservative fallback to fresh parsing for consistency. 130+ tests validated across feature combinations.

#### Symbol Normalization *(September 2025)*
`Grammar::normalize()` in `ir/src/lib.rs` recursively processes complex symbols (`Optional`, `Repeat`, `Choice`, `Sequence`) into auxiliary rules. `FirstFollowSets::compute()` in `glr-core` auto-normalizes. Resolved `ComplexSymbolsNotNormalized` errors, unblocking the full Grammar -> FIRST/FOLLOW -> LR(1) -> Parse Tables pipeline. Symbol IDs allocated at `max_existing_id + 1000`.

#### Test Stabilization *(PR #64, September 2025)*
Fixed GLR tree bridge test failures: corrected expectations so GLR parsers produce trees rooted at grammar start symbols (`value`), not immediate content nodes. Added `#![cfg(not(feature = "incremental_glr"))]` guards. Established correct GLR testing patterns.

#### External Scanner Integration *(PR #59)*
Pure-Rust `ExternalScanner` trait with `scan()`, `serialize()`, `deserialize()` methods. C FFI compatibility maintained. Verified with Python indentation tracking and JavaScript template literals.

#### Incremental Parsing & Node Metadata API *(PR #58)*
Tree-sitter compatible Node API (`kind()`, `start_byte()`, `end_byte()`, `is_error()`, etc.). Direct Forest Splicing algorithm achieves 16x speedup with 999/1000 subtree reuse for single-token edits. Conservative fallback for ambiguous scenarios.

#### GLR Parser Implementation *(PR #56, August 2025)*
ActionCell architecture: `Vec<Vec<Vec<Action>>>` enabling multi-action cells with runtime forking. Fixed critical "State 0" bug for Python grammar (273 symbols, 57 fields). SymbolMetadata API standardized. Concurrency caps system implemented.

#### GLR Runtime Integration *(PR #14, September 2025)*
Production-ready GLR runtime in `runtime2/src/parser.rs` with Tree-sitter compatible API. Forest-to-tree pipeline in `builder.rs` with optional performance instrumentation via `ADZE_LOG_PERFORMANCE`. Incremental parsing integrated through standard Parser API.

#### Golden Test Integration *(PR #11)*
Golden tests validate adze parsers against Tree-sitter reference implementations using SHA256 hash verification. Serialization roundtrip testing with 100+ cases covering JSON, S-expressions, and Unicode edge cases.

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
The ts-bridge tool extracts parse tables from compiled Tree-sitter grammars for use with adze's GLR runtime:

**Building:**
```bash
# Production build (requires libtree-sitter-dev system package)
cargo build -p ts-bridge

# Run ABI verification
cargo run -p ts-bridge --bin tsb-abi-check
./tools/ts-bridge/scripts/abi-hash.sh
```

**Testing:**
```bash
# Run basic tests
cargo test -p ts-bridge --test basic

# Run parity tests (requires actual Tree-sitter libraries)
cargo test -p ts-bridge --features with-grammars
```

**Usage:**
```bash
# Extract parse tables from a compiled grammar
cargo run -p ts-bridge -- path/to/libtree-sitter-json.so output.json tree_sitter_json
```

**Key Features:**
- ABI stability with Tree-sitter v15 (SHA verification and runtime checks)
- Dynamic buffer allocation (no truncation for large action cells)
- Production-ready with real Tree-sitter runtime (libtree-sitter-dev required)
- Comprehensive parity testing against Tree-sitter

### Known Issues (Being Addressed)

1. **GLR Runtime Optimization**: Fork/merge logic needs performance tuning for large files
2. **External Scanner FFI**: Integration with C scanners needs final touches
3. **Incremental Parsing Disabled**: The incremental parsing path in `glr_incremental.rs` is
   currently disabled and falls back to fresh parsing. The infrastructure exists but has
   architectural issues causing behavioral inconsistencies. See `glr_incremental.rs:281-297`.

**Resolved Issues**:
- ✅ **EOF Symbol Layout** (PR #90, Issue #89): Fixed EOF/non-terminal symbol ID collision in abi_builder.rs
- ✅ **EOF Symbol Handling** (PR #67): Fixed hardcoded SymbolId(0) to use parse_table.eof_symbol
- ✅ **Test Connectivity** (August 2025): All test files properly connected and running
