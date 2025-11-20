# Contributing to rust-sitter

Thank you for your interest in contributing to rust-sitter! This guide will help you get started with development.

---

## ⚡ Fast Path (3 Steps)

**Never contributed before? Start here:**

1. **Learn the basics** (optional if you know rust-sitter)
   - Read [QUICK_START.md](./QUICK_START.md) - 5-minute tutorial

2. **Pick a task**
   - Open [GAPS.md](./GAPS.md)
   - Search for "Good first issue" or tasks marked ≤4 hours
   - Look for tasks in the "Enable Ignored Tests" section

3. **Use the template**
   - Create an issue using `.github/ISSUE_TEMPLATE/enable_test.md`
   - Or use `.github/ISSUE_TEMPLATE/feature.md` for larger work

**That's it!** For detailed setup and workflows, continue reading below.

---

## 🚀 Quick Start for Contributors

**Looking for something to work on?** Check out **[GAPS.md](./GAPS.md)** for:
- 📋 20 ignored tests ready to be fixed
- 🔄 Incremental parsing implementation tasks
- 🔍 Query system completion tasks
- 📈 Performance benchmarking opportunities
- 📚 Documentation needs

**Each task includes**:
- Estimated time to complete
- Difficulty level (beginner/intermediate/advanced)
- Step-by-step implementation guidance
- Clear acceptance criteria

**[→ Browse Available Tasks in GAPS.md](./GAPS.md)**

---

## Setup

### Prerequisites
- `jq` - JSON processor (required for crate-aware checks)
- `rg` (ripgrep) - Fast text search (optional but recommended)
- Git with bash support (Git Bash on Windows)

### Enable Git Hooks
Configure git to use the repository's pre-commit and pre-push hooks:

```bash
# Set hooks path
git config core.hooksPath .githooks

# Ensure scripts are executable (after clone)
git update-index --chmod=+x .githooks/pre-commit .githooks/pre-push \
  scripts/affected-crates.sh scripts/check-goto-indexing.sh

# Recommended: Set locale for consistent formatting
export LC_ALL=C.UTF-8
export LANG=C.UTF-8
```

### Hook Behavior
The git hooks provide fast, crate-aware quality checks:

**Pre-commit (fast path)**
- Formats only staged files with `rustfmt`
- Runs clippy on affected crates only
- Blocks commits with conflict markers
- Guards against partial staging issues
- Uses default features for speed

**Pre-push (full validation)**
- Runs clippy across entire workspace
- Tests with feature matrix (default + tree-sitter-c2rust)
- Ensures code is merge-ready

### Environment Variables
Control hook behavior with these flags:

```bash
# Normal commit (fast, affected crates only)
git commit -m "fix: parser logic"

# Extended checks at commit time
RUN_EXTENDED=1 git commit -m "feat: new feature"

# Include quick per-crate tests
RUN_QUICK_TESTS=1 git commit -m "test: add coverage"

# Skip hooks temporarily (use sparingly)
git commit --no-verify -m "WIP: debugging"
```

---

## Policy Enforcement

**rust-sitter uses automated policy enforcement** to maintain quality, security, and performance standards. All contributions must pass these policies before merge.

### Three-Layer Enforcement

1. **Layer 1: Pre-commit Hooks** (Local, <5s)
   - Runs automatically before each commit
   - Fast feedback on basic issues
   - Can be bypassed for WIP commits

2. **Layer 2: Verification Scripts** (Local, <60s)
   - Self-service validation before push
   - Comprehensive quality + security checks
   - Recommended before pushing

3. **Layer 3: CI Workflows** (Remote, required)
   - Safety net that cannot be bypassed
   - Required for PR merge
   - Branch protection enforced

### Quick Start

**Using Nix (Recommended)**:
```bash
# Enter development shell (auto-installs pre-commit hooks)
nix develop

# Pre-commit hooks are now active!
git add .
git commit -m "feat: Your changes"
# Hooks run automatically: fmt, clippy, connectivity checks
```

**Manual Setup** (if not using Nix):
```bash
# Install pre-commit framework
pip install pre-commit

# Install hooks
pre-commit install
pre-commit install --hook-type commit-msg

# Hooks are now active
```

### Daily Workflow

**1. Make changes and commit**:
```bash
git add .
git commit -m "feat: Add new feature"
# Pre-commit hooks run automatically (<5s)
```

**2. Validate before pushing** (recommended):
```bash
# Quick quality check
./scripts/check-quality.sh

# Quick security check
./scripts/check-security.sh

# Or run both
./scripts/pre-push.sh
```

**3. Push changes**:
```bash
git push origin feature-branch
# CI validates everything (~40 min)
```

**4. Create PR**:
- CI must pass (policy + secrets workflows)
- At least 1 approving review required

### What Gets Checked

**Quality Policies**:
- ✅ Code formatting (`cargo fmt`)
- ✅ Zero clippy warnings (`cargo clippy -- -D warnings`)
- ✅ 100% test pass rate (`cargo test`)
- ✅ Zero doc warnings (`cargo doc`)
- ✅ Test connectivity (no `.rs.disabled` files)

**Security Policies**:
- 🔒 No vulnerabilities (`cargo audit`)
- 🔒 License compliance (`cargo deny`)
- 🔒 No secrets (pattern + entropy + TruffleHog)
- 🔒 No sensitive files (`.pem`, `.key`, etc.)

**Performance Policies** (PR only):
- 📊 Benchmark comparison (5% regression threshold)

### Bypassing Policies (Use Sparingly)

**Skip pre-commit hooks** (WIP commits only):
```bash
git commit --no-verify -m "wip: Incomplete work"
```

> ⚠️ **Warning**: CI will fail if issues remain. Only use for WIP commits on feature branches.

**Skip verification scripts**: Just don't run them (but CI will catch issues)

**Skip CI**: Not possible - required for PR merge

### Troubleshooting

**Problem**: Pre-commit hooks not running
```bash
# Reinstall hooks
pre-commit install

# Or enter Nix shell
nix develop
```

**Problem**: Verification script fails
```bash
# Use Nix for reproducible environment
nix develop --command ./scripts/check-quality.sh

# Or install required tools
cargo install cargo-audit cargo-deny
```

**Problem**: CI fails but local checks pass
```bash
# Use Nix shell (matches CI environment)
nix develop --command ./scripts/pre-push.sh

# Or check concurrency settings
RUST_TEST_THREADS=2 cargo test
```

### Policy Documentation

For complete policy documentation, see:
- [POLICIES.md](./POLICIES.md) - Policy overview and reference
- [Policy Enforcement Guide](./docs/guides/POLICY_ENFORCEMENT.md) - Technical implementation details
- [ADR-0010: Policy-as-Code](./docs/adr/ADR-0010-POLICY-AS-CODE.md) - Architecture decisions

---

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

### Clippy Quarantine (`.clippy-quarantine`)

We run `cargo clippy` per-package with `--no-deps`. Some crates currently have work-in-progress Clippy issues; to avoid blocking the whole workspace, each such crate is listed in `.clippy-quarantine` (one crate name per line). Lines beginning with `#` are ignored.

Format:
```
# One crate name per line
rust-sitter-benchmarks
rust-sitter-go
```

Workflow:
- To run a full triage locally: `./scripts/clippy-collect.sh` — produces `clippy-report/` with per-crate logs
- To reproduce an individual failure: `cargo clippy -p <crate> --all-targets --no-deps -- -D warnings`
- Once a crate is fully cleaned, remove it from `.clippy-quarantine`, commit, and push:
  ```bash
  sed -i '/^rust-sitter-go$/d' .clippy-quarantine
  git add .clippy-quarantine
  git commit -m "chore: remove rust-sitter-go from clippy quarantine"
  git push
  ```
- CI uploads `clippy-report/` for failing runs to help reviewers triage

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