#!/usr/bin/env just --justfile
# Rust-sitter development shortcuts

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

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

# Supported CI lane - must always be green
# See docs/status/KNOWN_RED.md for what is excluded and why.
ci-supported:
    cargo fmt --all -- --check
    cargo clippy -p rust-sitter --lib -- -D warnings
    cargo clippy -p rust-sitter-macro -p rust-sitter-tool \
        -p rust-sitter-common -p rust-sitter-ir -p rust-sitter-glr-core \
        -p rust-sitter-tablegen \
        --all-targets -- -D warnings
    cargo test -p rust-sitter --lib
    cargo test -p rust-sitter-macro -p rust-sitter-tool \
        -p rust-sitter-common -p rust-sitter-ir -p rust-sitter-glr-core \
        -p rust-sitter-tablegen \
        --lib --tests --bins
    cargo test -p rust-sitter-glr-core --features serialization --doc

# Clean build artifacts
clean:
    cargo clean