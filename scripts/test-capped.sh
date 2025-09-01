#!/usr/bin/env bash
set -euo pipefail

# Source system caps
source "$(dirname "$0")/preflight.sh"

# Default to running all tests with caps
TEST_TARGET="${1:-}"
EXTRA_ARGS="${2:-}"

echo "Running tests with capped concurrency..."

if [ -n "$TEST_TARGET" ]; then
    # Run specific test target with caps
    cargo test -p "$TEST_TARGET" -- --test-threads="$RUST_TEST_THREADS" $EXTRA_ARGS
else
    # Run all workspace tests with caps
    cargo test -- --test-threads="$RUST_TEST_THREADS" $EXTRA_ARGS
fi