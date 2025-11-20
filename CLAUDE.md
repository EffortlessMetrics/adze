# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Use TDD. Red-Green-Refactor, spec driven design. User-story driven design.

## Requirements

### Minimum Rust Version (MSRV)
- **Rust 1.89.0** or later
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

## Development Environment Setup

### Recommended: Nix Development Shell (Infrastructure-as-Code)

The project provides a **Nix flake** that defines a reproducible development environment matching CI exactly. This is the recommended setup for contributors.

**Benefits**:
- ✅ One command setup: `nix develop`
- ✅ Identical environment to CI (no "works on my machine")
- ✅ All dependencies included (Rust, system libraries, tools)
- ✅ Consistent across Linux, macOS, and Windows (WSL)
- ✅ Isolated from system packages

**One-time Setup**:
```bash
# Install Nix (if not already installed)
curl -L https://nixos.org/nix/install | sh

# Enable flakes (required)
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Restart your shell to apply changes
exec $SHELL
```

**Using the Nix Shell**:
```bash
# Enter the development shell
nix develop

# You're now in a shell with all dependencies!
# Run CI commands using just:
just ci-all          # Run complete CI suite
just ci-test         # Run tests only
just ci-perf         # Run performance benchmarks

# Exit the shell when done
exit
```

**One-liner (without entering shell)**:
```bash
# Run CI commands directly
nix develop . --command just ci-all
nix develop . --command just ci-test
nix develop . --command cargo build
```

**Available Shells**:
- `nix develop .#default` - Standard development environment (default)
- `nix develop .#ci` - Minimal CI environment
- `nix develop .#perf` - Performance profiling (includes flamegraph, heaptrack)

**Environment Variables** (automatically set in Nix shell):
- `RUST_BACKTRACE=1` - Enable backtraces
- `RUST_TEST_THREADS=2` - Concurrency cap for stable tests
- `RAYON_NUM_THREADS=4` - Rayon thread pool limit
- `TOKIO_WORKER_THREADS=2` - Tokio async runtime limit

**📚 Documentation**:
- **[Nix Quickstart Guide](docs/guides/NIX_QUICKSTART.md)** - Complete setup instructions (5-10 minutes)
- **[Nix Troubleshooting](docs/guides/NIX_TROUBLESHOOTING.md)** - Common issues and solutions
- **[Migrating to Nix](docs/guides/MIGRATING_TO_NIX.md)** - For existing contributors with traditional setup
- [ADR-0008: Nix Development Environment](docs/adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md) - Design rationale
- [Strategic Implementation Plan](docs/plans/STRATEGIC_IMPLEMENTATION_PLAN.md) - Roadmap
- [Nix CI Workflow](.github/workflows/nix-ci.yml) - CI configuration

### Alternative: Traditional Setup (Manual)

If you prefer not to use Nix, you can set up the environment manually:

**Install Rust**:
```bash
# Using rustup (respects rust-toolchain.toml)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup show  # Install toolchain specified in rust-toolchain.toml
```

**Install System Dependencies**:

*Linux (Debian/Ubuntu)*:
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  cmake \
  clang \
  libtree-sitter-dev
```

*macOS*:
```bash
brew install cmake pkg-config openssl
```

*Windows*:
```powershell
# Install via chocolatey
choco install cmake pkgconfiglite
```

**Install Development Tools**:
```bash
cargo install just cargo-nextest cargo-insta
```

**Set Environment Variables** (add to your shell rc file):
```bash
export RUST_BACKTRACE=1
export RUST_TEST_THREADS=2
export RAYON_NUM_THREADS=4
```

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
  - Performance instrumentation via `RUST_SITTER_LOG_PERFORMANCE` environment variable
  - Node count, tree depth, and conversion time metrics
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
   - Production-ready with real Tree-sitter runtime linking (requires libtree-sitter-dev)
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
   - `RUST_SITTER_LOG_PERFORMANCE=true`: Enables performance logging for GLR forest-to-tree conversion
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
5. Test table generation with `cargo test -p rust-sitter-tablegen`
6. Verify Language struct layout matches Tree-sitter ABI exactly

### Testing Guidelines

1. **Grammar Tests**: Add new grammars to `/example/src/` with corresponding snapshot tests
2. **Compression Tests**: Verify table compression maintains Tree-sitter compatibility
3. **FFI Tests**: Ensure generated Language structs match C ABI requirements
4. **Integration Tests**: Test with real Tree-sitter grammars for validation
5. **GLR Runtime Tests**: Test GLR integration and performance with `runtime2/tests/glr_parse.rs`
6. **Incremental Parsing Tests**: Verify subtree reuse with `runtime/tests/property_incremental_test.rs`
7. **Feature Flag Tests**: Test all feature combinations (`default`, `glr-core`, `incremental`, `all-features`)

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

#### Test Status Update (January 2025)
All previously disabled test files have been successfully re-enabled and integrated back into the test suite. The test connectivity safeguards are now working correctly to prevent any tests from being silently disconnected in the future.

**Current Status**: All test files are properly connected and running. No `.rs.disabled` files exist in the repository.

To check test connectivity locally, run:
```bash
./scripts/check-test-connectivity.sh
```

### Recent Achievements (August 2025)

#### **GLR Parser Implementation - Production Ready** ✅
Successfully transformed rust-sitter from a simple LR parser to a true GLR (Generalized LR) parser that can handle ambiguous grammars. The implementation is now production-ready with complete runtime integration and comprehensive API stabilization.

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

#### **GLR Runtime Integration - Production Complete** ✅ *(September 2025)*
Successfully completed full GLR integration in runtime2 with PR #14 merge ("runtime2: wire parser to GLR engine"), delivering a production-ready parsing solution with Tree-sitter API compatibility and seamless incremental parsing.

**Production Deployment Achievements:**
1. **Complete GLR Runtime API**: Production-ready parser integration in `runtime2/src/parser.rs`
   - Tree-sitter compatible API: `Parser::new()`, `set_language()`, `parse()`, `parse_utf8()`
   - Automatic GLR engine routing with feature-gated compilation
   - Language validation ensures parse table and tokenizer are present in GLR mode
   - Graceful fallback behavior when GLR features are disabled

2. **High-Performance Forest-to-Tree Pipeline**: Optimized conversion in `runtime2/src/builder.rs`
   - Zero-overhead forest-to-tree conversion with performance instrumentation
   - Real-time metrics: node count, tree depth, conversion time via `RUST_SITTER_LOG_PERFORMANCE`
   - Memory-efficient tree construction with arena allocation support
   - Smart caching and input comparison optimization

3. **Integrated Incremental Parsing**: Seamless incremental support through standard Parser API
   - Automatic route selection between incremental and full parsing
   - Conservative subtree reuse maintaining GLR correctness
   - Enhanced `Tree::edit()` with comprehensive `EditError` handling
   - Feature compatibility: works with or without incremental features enabled

4. **Production Readiness Features**:
   - Memory safety: checked arithmetic operations throughout parsing pipeline
   - Error resilience: comprehensive error handling and validation
   - Performance monitoring: built-in instrumentation with zero runtime cost when disabled
   - Thread safety: concurrent parsing support with bounded resource usage

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

### External Lexer Utilities (PR #67 Complete)

External lexer utilities provide FFI-compatible lexer functionality for integrating with Tree-sitter compatible systems and external scanners.

**Key Features:**
- **FFI Compatibility**: Full compatibility with Tree-sitter external scanner interface
- **Column Tracking**: Accurate position tracking with newline handling
- **Range Detection**: Support for included range boundaries
- **EOF Handling**: Robust end-of-input detection
- **Memory Safety**: Safe pointer handling with comprehensive null checks

**Core API:**
```rust
pub struct ExternalLexer {
    input: &'static [u8],
    position: usize,
    column: u32,
    // ... internal fields
}

impl ExternalLexer {
    /// Create a new external lexer
    pub fn new(input: &'static [u8], start_byte: usize, start_column: u32) -> Self;
    
    /// Tree-sitter FFI compatible methods
    pub unsafe extern "C" fn lookahead(lexer: *mut c_void) -> u32;
    pub unsafe extern "C" fn advance(lexer: *mut c_void, skip: bool);
    pub unsafe extern "C" fn mark_end(lexer: *mut c_void);
    pub unsafe extern "C" fn get_column(lexer: *mut c_void) -> u32;
    pub unsafe extern "C" fn is_at_included_range_start(lexer: *mut c_void) -> bool;
    pub unsafe extern "C" fn eof(lexer: *mut c_void) -> bool;
}
```

**Usage Example:**
```rust
use rust_sitter::external_lexer::ExternalLexer;

// Create external lexer for use with Tree-sitter external scanners
let input = b"hello\nworld";
let mut ext_lexer = ExternalLexer::new(input, 0, 0);

// Convert to Tree-sitter TSLexer for FFI compatibility
let ts_lexer = create_ts_lexer(&mut ext_lexer);

// Use with external scanner functions
unsafe {
    let ch = ExternalLexer::lookahead(&mut ts_lexer as *mut _ as *mut c_void);
    ExternalLexer::advance(&mut ts_lexer as *mut _ as *mut c_void, false);
    let col = ExternalLexer::get_column(&mut ts_lexer as *mut _ as *mut c_void);
}
```

**Testing Coverage:**
- ✅ All external lexer tests (column tracking, EOF, range detection) pass
- ✅ Full runtime test suite passes (128/128 tests)
- ✅ Clippy passes without warnings
- ✅ Tree-sitter FFI compatibility verified

**Integration with Query Parser:**
External lexer utilities complement the enhanced query parser error handling (also in PR #67):
- **Robust Predicate Validation**: Enhanced parsing validates predicate identifiers
- **Standalone Predicate Detection**: Proper error messages for invalid predicate usage
- **Precise Error Positioning**: Accurate byte positions for debugging

### Known Issues (Being Addressed)

1. **GLR Runtime Optimization**: Fork/merge logic needs performance tuning for large files
2. **Incremental Parsing**: GLR incremental parsing algorithms need implementation
3. **Disabled Test Re-enablement**: Several test files need to be re-enabled after GLR stabilization (see Test Connectivity section above)

### Recently Resolved Issues

✅ **External Scanner FFI** (PR #67): Integration with Tree-sitter external scanners is complete with comprehensive external lexer utilities and FFI compatibility

✅ **Query Parser Error Handling** (PR #67): Enhanced query parser with robust predicate validation and precise error reporting
