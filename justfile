#!/usr/bin/env just --justfile
# Adze development shortcuts

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Format all code
fmt:
    cargo fmt --all --check

# Run clippy on core workspace members
clippy:
    cargo clippy -p adze -p adze-glr-core -p adze-ir -p adze-tablegen --lib -- -D warnings

# Run tests on core workspace members
test:
    cargo test -p adze -p adze-glr-core -p adze-ir -p adze-tablegen --lib

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

supported_crates := "-p adze -p adze-macro -p adze-tool -p adze-common -p adze-ir -p adze-glr-core -p adze-tablegen"

# Required PR gate: this is the single supported CI lane for branch protection
# See docs/status/KNOWN_RED.md; update it whenever ci-supported command targets change.
ci-supported:
    #!/usr/bin/env bash
    set -euo pipefail
    export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
    export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"
    cargo fmt --all -- --check
    cargo clippy {{supported_crates}} --all-targets -- -D warnings
    cargo test {{supported_crates}} --lib --tests --bins -- --test-threads="$RUST_TEST_THREADS"
    cargo test -p adze-glr-core --features serialization --doc -- --test-threads="$RUST_TEST_THREADS"

# Clean build artifacts
clean:
    cargo clean
