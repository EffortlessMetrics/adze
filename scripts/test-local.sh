#!/usr/bin/env bash
set -euo pipefail
TIMEOUT="${TIMEOUT:-300s}"

if command -v cargo-nextest >/dev/null; then
  cargo nextest run --workspace --failure-output=immediate-final --retries=0 --slow-timeout 120s "$@"
else
  timeout "$TIMEOUT" cargo test --workspace "$@"
fi