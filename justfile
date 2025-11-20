#!/usr/bin/env just --justfile
# Rust-sitter development shortcuts
#
# CI Integration Commands (Nix-compatible):
#   just ci-all          - Run complete CI suite (fmt, clippy, test, doc)
#   just ci-perf         - Run performance benchmarks
#   just help            - Show all available commands
#
# Original Development Commands:
#   See below for fmt, clippy, test, build, etc.

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Default recipe (show help)
default:
    @just --list

# Show help with command categories
help:
    @echo "🦀 rust-sitter development commands"
    @echo ""
    @echo "CI Integration (Nix-compatible):"
    @echo "  just ci-all          - Run complete CI suite"
    @echo "  just ci-fmt          - Check code formatting"
    @echo "  just ci-clippy       - Run clippy lints (workspace)"
    @echo "  just ci-test         - Run test suite with caps"
    @echo "  just ci-doc          - Check documentation builds"
    @echo "  just ci-perf         - Run performance benchmarks"
    @echo ""
    @echo "Development:"
    @echo "  just fmt             - Format code (check mode)"
    @echo "  just fix             - Fix clippy warnings"
    @echo "  just clippy          - Run clippy (core pkgs)"
    @echo "  just test            - Run tests (core pkgs)"
    @echo "  just build           - Build workspace"
    @echo ""
    @echo "Testing:"
    @echo "  just test-safe       - Run tests (single thread)"
    @echo "  just snap            - Review insta snapshots"
    @echo "  just matrix          - Run test matrix"
    @echo ""
    @echo "For CI usage:"
    @echo "  nix develop . --command just ci-all"

# ============================================================================
# CI Integration Commands (used by GitHub Actions and local Nix shell)
# ============================================================================

# Run complete CI suite (formatting, linting, tests, docs)
ci-all: ci-fmt ci-clippy ci-test ci-doc
    @echo "✅ All CI checks passed"

# Check code formatting without making changes
ci-fmt:
    @echo "🔍 Checking code formatting..."
    cargo fmt --all -- --check

# Run clippy on all workspace members (strict mode)
ci-clippy:
    @echo "🔍 Running clippy lints (workspace)..."
    cargo clippy --workspace --all-targets -- -D warnings

# Run test suite with capped concurrency (RUST_TEST_THREADS=2)
ci-test:
    @echo "🧪 Running test suite..."
    cargo test --workspace -- --test-threads=2

# Run tests with ultra-safe concurrency (single thread)
ci-test-safe:
    @echo "🧪 Running test suite (safe mode)..."
    RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 cargo test --workspace -- --test-threads=1

# Check documentation builds without errors
ci-doc:
    @echo "📚 Checking documentation..."
    cargo doc --no-deps --workspace

# Run performance benchmarks and save baselines
ci-perf:
    @echo "⚡ Running performance benchmarks..."
    cargo bench --workspace

# GLR-specific tests with required features
ci-glr:
    @echo "🌳 Running GLR-specific tests..."
    cargo test -p rust-sitter-glr-core --features test-api
    cargo test -p rust-sitter-runtime2 --features glr-core

# Verify all checks pass before pushing
pre-push: ci-all
    @echo "✅ All CI checks passed - safe to push"

# ============================================================================
# Original Development Commands
# ============================================================================

# Format all code
fmt:
    cargo fmt --all --check

# Run clippy on core workspace members
clippy:
    cargo clippy -p rust-sitter -p rust-sitter-glr-core -p rust-sitter-ir -p rust-sitter-tablegen --lib -- -D warnings

# Run tests on core workspace members
test:
    cargo test -p rust-sitter -p rust-sitter-glr-core -p rust-sitter-ir -p rust-sitter-tablegen --lib

# Run pre-commit checks
pre:
    .githooks/pre-commit

# Run pre-commit with test clippy enabled
pre-tests:
    CLIPPY_TESTS=1 .githooks/pre-commit

# Run pre-commit with strict docs enabled
pre-docs:
    STRICT_DOCS=1 .githooks/pre-commit

# Run pre-commit with warnings as errors
pre-warn:
    RUSTC_WARN_FATAL=1 .githooks/pre-commit

# Build everything
build:
    cargo build --all

# Build with release optimizations
release:
    cargo build --release --all

# Run test matrix script
matrix:
    ./scripts/test-matrix.sh

# Check ts-bridge linking
smoke:
    ./scripts/smoke-link.sh ts-bridge

# Run tests with perf counters enabled
bench-perf:
    cargo test --features perf-counters

# Update insta snapshots
snap:
    cargo insta review

# Clean build artifacts
clean:
    cargo clean

# Fix clippy warnings automatically where possible
fix:
    @echo "🔧 Fixing clippy warnings..."
    cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged