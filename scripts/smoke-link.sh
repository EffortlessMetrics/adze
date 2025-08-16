#!/usr/bin/env bash
set -euo pipefail

crate="${1:-ts-bridge}"
echo "Building $crate in release mode..."
export CARGO_TARGET_DIR=target
cargo build -p "$crate" --release 2>&1 | tail -5

# Find the library file (convert - to _ for library name)
lib_name=$(echo "$crate" | tr '-' '_')
so=$(find target/release -maxdepth 1 -name "lib${lib_name}.so" -o -name "lib${lib_name}.dylib" 2>/dev/null | head -n1)

if [ -z "$so" ]; then
    echo "ERROR: Could not find library file for $crate"
    exit 1
fi

echo "→ Checking symbols in $so"
nm -g --defined-only "$so" 2>/dev/null | grep -E 'rs_ts_bridge_version|ts_' | head -20 || echo "No matching symbols found"

# Check for undefined symbols
echo ""
echo "→ Checking for undefined symbols..."
nm -u "$so" 2>/dev/null | grep -E '^[[:space:]]+U ts_' | head -10 || echo "No undefined ts_ symbols"

echo ""
echo "→ Library info:"
file "$so"

echo ""
echo "OK: Build completed"