# Next Steps: Getting Started with rust-sitter Development

Welcome! This guide will get you up to speed with developing rust-sitter.

## Before You Start (Required Reading)

1. **[README.md](./README.md)** (5 min)
   - What is rust-sitter?
   - Current status (early development, v0.8.0-dev)
   - What works and what doesn't

2. **[PROJECT_STATE_v0.8.0-dev.md](./PROJECT_STATE_v0.8.0-dev.md)** (10-15 min)
   - Honest capability assessment
   - Test results (379/385 passing)
   - Critical gaps
   - Timeline to production

3. **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** (15-20 min)
   - 5-phase roadmap
   - Current priorities
   - What needs to be done

4. **[CLAUDE.md](./CLAUDE.md)** (15-20 min)
   - Architecture overview
   - Development commands
   - Testing guidelines
   - Code standards

## Development Environment Setup

### 1. Prerequisites

**Required**:
```bash
# Rust 1.89.0 or later (Rust 2024 Edition)
rustup update
rustc --version  # Should be 1.89.0+

# Git
git --version   # Should be present

# Package management
cargo --version
```

**Optional but Recommended**:
```bash
# For system dependencies (Linux)
sudo apt-get install libtree-sitter-dev libclang-dev

# For macOS
brew install tree-sitter
```

### 2. Clone and Setup

```bash
# Clone the repository
git clone https://github.com/EffortlessMetrics/rust-sitter.git
cd rust-sitter

# Create a development branch
git checkout -b feature/your-feature-name
```

### 3. Verify Setup

```bash
# Build all workspace members
cargo build

# Run tests (should see 379 passing)
cargo test --lib

# Format check
cargo fmt --check

# Lint check
cargo clippy --all
```

## Available Work

### 🚨 CRITICAL: Phase 1 - Transform Function Execution (3-4 weeks)

**Why this matters**: Blocks 95%+ of real-world grammars from parsing correctly.

**Current Status**: Infrastructure exists, but transform function execution is incomplete.

**Key Files**:
- `runtime/src/parser_v4.rs` - Lexer type conversion (main focus)
- `runtime/src/external_lexer.rs` - External lexer interface
- `glr-core/src/lib.rs` - GLR core parser

**Tasks** (in order):
```
[ ] 1. Implement TSLexState Type Conversion
    Goal: Convert custom lexer state to Tree-sitter TSLexState
    File: runtime/src/parser_v4.rs (line ~145)
    Impact: 6 python-simple tests will start passing
    Tests: cargo test -p rust-sitter-glr-core

[ ] 2. Generate Transform Execution Code
    Goal: Execute transform functions instead of capturing them
    File: runtime/src/parser_v4.rs (lexer initialization)
    Impact: Lexer will correctly process string/number literals
    Tests: cargo test test_python_simple

[ ] 3. Integrate with Lexer State Management
    Goal: Ensure transform results are properly managed
    File: glr-core/src/lib.rs (token processing)
    Impact: Complete lexer pipeline functional
    Tests: cargo test --all

[ ] 4. Test with python-simple Grammar
    Goal: Verify 6 failing tests now pass
    File: runtime/tests/glr_parse.rs
    Command: cargo test -p rust-sitter test_python_simple
    Success: All 379 tests pass (currently 373)

[ ] 5. Document Implementation
    Goal: Update code comments and IMPLEMENTATION_PLAN.md
    File: IMPLEMENTATION_PLAN.md (Phase 1 complete)
    Impact: Help future maintainers understand solution
```

**Reference**:
- See [CRITICAL_ISSUES_SUMMARY.md](./CRITICAL_ISSUES_SUMMARY.md) - Issue #74 (Transform Functions)
- See [IMPLEMENTATION_PLAN.md - Phase 1](./IMPLEMENTATION_PLAN.md#phase-1-3-4-weeks-transform-function-execution)

### ⚡ HIGH: Phase 2 - Real Performance Benchmarks (2 weeks)

**Why this matters**: Current benchmarks measure character iteration, not real parsing.

**Tasks**:
```
[ ] 1. Replace mock benchmarks with real parsing
    Files: benches/*.rs
    Command: cargo bench

[ ] 2. Compare against Tree-sitter baseline
    Goal: Honest comparison, not optimistic claims

[ ] 3. Update README with real results
    File: README.md performance section
```

**Reference**: [IMPLEMENTATION_PLAN.md - Phase 2](./IMPLEMENTATION_PLAN.md#phase-2-2-weeks-real-performance-benchmarks)

### 🔌 HIGH: Phase 3 - External Scanner Support (4-6 weeks)

**Why this matters**: Enables parsing Python, C++, Ruby, and other languages with context-sensitive tokens.

**Tasks**:
```
[ ] 1. Python indentation scanner
    Goal: Track indentation level across tokens
    Impact: Python grammar becomes parseable

[ ] 2. C++ raw string scanner
    Goal: Handle R"delimiter(raw string)delimiter"
    Impact: C++ grammar becomes parseable

[ ] 3. Ruby heredoc scanner
    Goal: Handle <<HEREDOC multi-line strings
    Impact: Ruby grammar becomes parseable
```

**Reference**: [IMPLEMENTATION_PLAN.md - Phase 3](./IMPLEMENTATION_PLAN.md#phase-3-4-6-weeks-external-scanner-support)

### 📊 MEDIUM: Phase 4 - Comprehensive Grammar Testing (2-3 weeks)

**Why this matters**: Validates that common grammars actually work.

**Tasks**:
```
[ ] 1. Create 50+ grammar test suite
[ ] 2. Document compatibility matrix
[ ] 3. Identify blocking issues
```

**Reference**: [IMPLEMENTATION_PLAN.md - Phase 4](./IMPLEMENTATION_PLAN.md#phase-4-2-3-weeks-comprehensive-testing)

## Code Style & Standards

### MSRV & Edition
```bash
# Rust 1.89.0 (Rust 2024 Edition)
rustc --version              # Verify 1.89.0+
cargo check                  # Will fail if using newer features
```

### Formatting
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# All crates must pass formatting
cargo fmt --all -- --check
```

### Linting
```bash
# Run clippy
cargo clippy --all

# Run with warnings as errors
cargo clippy --all -- -D warnings
```

### Testing
```bash
# Run all tests (uses safe defaults)
cargo test

# Run specific crate
cargo test -p rust-sitter-glr-core

# Run with output
cargo test -- --nocapture

# Run with concurrency cap (recommended)
cargo test-ultra-safe        # 1 thread
cargo test-safe              # 2 threads
cargo t2                      # 2 threads (alias)
```

## Common Development Commands

### Building
```bash
# Build all workspace members
cargo build

# Build release
cargo build --release

# Build specific package
cargo build -p rust-sitter-glr-core
```

### Testing
```bash
# Run tests (safe defaults)
cargo test

# Update snapshot tests
cargo insta review

# Run tests for Phase 1 work
cargo test -p rust-sitter test_python

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Documentation
```bash
# Build and open documentation
cargo doc --open

# Build documentation book
cd book && mdbook serve

# View in browser: http://localhost:3000
```

### Debugging
```bash
# Print IR for grammar debugging
RUST_SITTER_EMIT_ARTIFACTS=true cargo build

# Log parser performance
RUST_SITTER_LOG_PERFORMANCE=true cargo test

# Set concurrency explicitly
RUST_TEST_THREADS=1 cargo test
```

## First Commit Checklist

Before submitting your first PR:

- [ ] Code formatted: `cargo fmt`
- [ ] Clippy passes: `cargo clippy --all -- -D warnings`
- [ ] Tests pass: `cargo test`
- [ ] MSRV verified: Rust 1.89.0+
- [ ] Commits have clear messages
- [ ] Changes documented in code comments
- [ ] No unrelated changes

## Making Your First Contribution

### Step 1: Pick an Issue

**Recommended for First-Time Contributors**:
1. Phase 1 sub-tasks (transform functions)
   - Well-scoped work
   - Clear success criteria
   - High impact

2. Documentation improvements
   - Help update docs
   - Lower risk
   - Great for learning codebase

### Step 2: Create a Branch

```bash
# Start from main
git checkout main
git pull origin main

# Create feature branch
git checkout -b feature/my-feature-name

# Make your changes
# ...

# Run tests and format
cargo test
cargo fmt
cargo clippy --all
```

### Step 3: Commit and Push

```bash
# Stage your changes
git add .

# Create commit with clear message
git commit -m "type: brief description

Optional longer explanation of the change,
why it was needed, and any tradeoffs made.

Related: #123 (if fixing an issue)
"

# Push to your fork
git push origin feature/my-feature-name
```

### Step 4: Create a Pull Request

Include in PR description:
- What does this change do?
- Why is it needed?
- How does it address the issue?
- What testing did you do?

Reference: [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for context

## Getting Help

### For Architecture Questions
- See [CLAUDE.md](./CLAUDE.md) Architecture Overview section
- See [PURE_RUST_IMPLEMENTATION.md](./PURE_RUST_IMPLEMENTATION.md)
- See book/src/development/architecture.md

### For Testing Questions
- See [CLAUDE.md](./CLAUDE.md) Testing Guidelines section
- Run: `cargo test --help`
- Check test examples in: `example/src/`, `runtime/tests/`

### For Specific Issues
- See [CRITICAL_ISSUES_SUMMARY.md](./CRITICAL_ISSUES_SUMMARY.md) for known issues
- See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for planned work
- Check GitHub issues and discussions

### For Code Review
- Code reviews focus on:
  - Correctness and safety
  - MSRV compatibility
  - Test coverage
  - Documentation
  - Performance impact

## Quick Reference

### Test Status
- ✅ 379/385 tests passing (98.4%)
- ❌ 6 failures in python-simple (transform functions)
- 📍 Location: `runtime/tests/glr_parse.rs`

### Key Files to Know
- `runtime/src/parser_v4.rs` - Main GLR lexer/parser
- `glr-core/src/lib.rs` - GLR core algorithms
- `tablegen/src/compress.rs` - Table compression
- `IMPLEMENTATION_PLAN.md` - Development roadmap
- `CLAUDE.md` - Architecture details

### Important Paths
- Grammar definitions: `/example/src/`
- Tests: `/runtime/tests/`, `/glr-core/tests/`
- Documentation: `/book/src/`, `/docs/`
- Archived docs: `/docs/archive/`

## Current Development Status

**v0.8.0-dev** (November 2025)
- **Test Status**: 379/385 passing (98.4%)
- **Blocking Issue**: Transform functions (Phase 1)
- **Timeline**: 13-19 weeks to v1.0.0
- **MSRV**: Rust 1.89.0

**Current Focus**: Phase 1 - Transform Function Execution
- Why: Blocks 95%+ of real grammars
- Duration: 3-4 weeks
- Impact: High (enables most grammar types)

## Next Actions

1. **Read Required Documentation**
   - [ ] README.md (5 min)
   - [ ] PROJECT_STATE_v0.8.0-dev.md (15 min)
   - [ ] IMPLEMENTATION_PLAN.md (20 min)
   - [ ] CLAUDE.md (20 min)

2. **Setup Development Environment**
   - [ ] Clone repository
   - [ ] Verify Rust 1.89.0+
   - [ ] Run `cargo build && cargo test`

3. **Explore the Codebase**
   - [ ] Read PURE_RUST_IMPLEMENTATION.md
   - [ ] Look at example grammars in `/example/src/`
   - [ ] Run: `cargo doc --open` and browse

4. **Pick Your First Task**
   - [ ] Phase 1 sub-task (high impact)
   - [ ] Documentation improvement (low risk)
   - [ ] Test addition (good learning)

5. **Make Your First Contribution**
   - [ ] Create feature branch
   - [ ] Make changes
   - [ ] Run tests and format
   - [ ] Create pull request

---

## Helpful Links

- **Main Documentation**: [DOCUMENTATION.md](./DOCUMENTATION.md)
- **Architecture Guide**: [CLAUDE.md](./CLAUDE.md)
- **Implementation Plan**: [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)
- **Known Issues**: [CRITICAL_ISSUES_SUMMARY.md](./CRITICAL_ISSUES_SUMMARY.md)
- **API Docs**: [book/src/](./book/src/)
- **GitHub Issues**: https://github.com/EffortlessMetrics/rust-sitter/issues

---

**Ready to get started?** Pick a task above and create your first PR! We're excited to have you contribute.

Questions? Check the FAQ in [DOCUMENTATION.md](./DOCUMENTATION.md) or open a discussion on GitHub.
