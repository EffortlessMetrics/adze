#!/usr/bin/env bash
set -euo pipefail

# Colors (fallback to no color if not TTY)
if [ -t 1 ]; then
  RED=$'\e[31m'
  GRN=$'\e[32m'
  YLW=$'\e[33m'
  BLD=$'\e[1m'
  RST=$'\e[0m'
else
  RED=""
  GRN=""
  YLW=""
  BLD=""
  RST=""
fi

ensure_tests() {
  local label="$1"; shift
  local -a args=("$@")
  echo "${YLW}${BLD}→ Ensuring tests exist:${RST} ${label}"
  # Discover tests
  set +e
  local out
  out=$(cargo test "${args[@]}" -- --list 2>&1)
  local rc=$?
  set -e
  if [ $rc -ne 0 ]; then
    echo "${RED}✗ Failed to list tests:${RST} cargo test ${args[*]} -- --list"
    echo "$out"
    exit 1
  fi
  # Count looks like lines with a path-like test name (`foo::bar` or `test foo`)
  local count
  count=$(printf "%s\n" "$out" | grep -E '(^[A-Za-z0-9_].*::|^test\s)' | wc -l | tr -d ' ')
  if [ "$count" -eq 0 ]; then
    echo "$out" | sed -e 's/^/  | /'
    echo "${RED}✗ No tests discovered for:${RST} cargo test ${args[*]}"
    exit 1
  fi
  echo "${GRN}✓ Found ${count} tests${RST} in ${label}"
  # Now actually run them
  cargo test "${args[@]}"
  echo "${GRN}✓ Tests passed:${RST} ${label}"
}

# Helper that tries multiple query styles and passes if any returns ≥1 test
ensure_any_tests() {
  # Args come in tuples: "<label>" <cargo args...> -- "<label>" <cargo args...> ...
  # Passes if at least one set has >=1 test discovered. Runs all non-empty sets.
  local had_any=0
  local ok_sets=()

  while [ "$#" -gt 0 ]; do
    local label="$1"; shift
    # collect args until next "--" or end
    local set_args=()
    while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do
      set_args+=("$1"); shift
    done
    [ "$#" -gt 0 ] && shift  # consume the "--" separator if present

    echo "${YLW}${BLD}→ Checking:${RST} ${label}"
    set +e
    local out
    out=$(cargo test "${set_args[@]}" -- --list 2>&1)
    local rc=$?
    set -e
    if [ $rc -ne 0 ]; then
      echo "${RED}✗ Failed to list tests for:${RST} cargo test ${set_args[*]} -- --list"
      echo "$out"
      exit 1
    fi
    local count
    count=$(printf "%s\n" "$out" | grep -E '(^[A-Za-z0-9_].*::|^test\s)' | wc -l | tr -d ' ')
    if [ "$count" -gt 0 ]; then
      had_any=1
      ok_sets+=("${label}::$(printf "%q " "${set_args[@]}")")
      echo "${GRN}✓ Found ${count} tests${RST} in ${label}"
    else
      echo "${YLW}• No tests discovered in ${label}${RST}"
    fi
  done

  if [ "$had_any" -eq 0 ]; then
    echo "${RED}✗ No tests discovered in any provided set.${RST}"
    exit 1
  fi

  # Run all discovered sets
  for entry in "${ok_sets[@]}"; do
    local label="${entry%%::*}"
    # shellcheck disable=SC2206
    local args=( ${entry#*::} )
    echo "${YLW}${BLD}→ Running:${RST} ${label}"
    cargo test "${args[@]}"
    echo "${GRN}✓ Passed:${RST} ${label}"
  done
}

echo "${BLD}Formatting & linting…${RST}"
cargo fmt --all -- --check
# Only lint core crates with strict warnings - other crates may have more relaxed requirements
cargo clippy -p rust-sitter -p rust-sitter-glr-core -p rust-sitter-ir -p rust-sitter-tablegen --lib -- -D warnings

echo "${BLD}Running test matrix…${RST}"

# Core (dev features)
ensure_tests "glr-core (dev)" -p rust-sitter-glr-core --features test-helpers

# Core (strict invariants) — some manual-table tests should be gated off here
ensure_tests "glr-core (strict)" -p rust-sitter-glr-core --features strict-invariants,test-helpers

# Runtime (lib + integration): pass if either exists, run both if present
ensure_any_tests \
  "runtime (lib)" -p rust-sitter --lib -- \
  "runtime (integration)" -p rust-sitter --tests

# Tablegen tests
ensure_tests "tablegen" -p rust-sitter-tablegen

# IR tests
ensure_tests "ir" -p rust-sitter-ir

# Tool tests
ensure_tests "tool" -p rust-sitter-tool

# Macro tests
ensure_tests "macro" -p rust-sitter-macro

# Example tests
ensure_tests "example" -p example

# Optional: ts-bridge tests when feature is enabled
if [ "${TS_BRIDGE_TESTS:-0}" = "1" ]; then
  ensure_tests "ts-bridge" -p ts-bridge --features ts-ffi-raw
fi

# Optional: runtime E2E pack (opt-in only)
if [ "${RUNTIME_E2E:-0}" = "1" ]; then
  ensure_tests "runtime (e2e)" -p rust-sitter --features runtime-e2e
fi

echo "${GRN}${BLD}All checks passed.${RST}"