#!/usr/bin/env bash
set -euo pipefail

echo "=== WASM Build Verification ==="

# Core crates that should compile for wasm32-unknown-unknown
WASM_CRATES=(
    "adze-ir"
    "adze-glr-core"
    "adze-tablegen"
    "adze-common"
)

TARGET="wasm32-unknown-unknown"

# Ensure target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Installing $TARGET target..."
    rustup target add "$TARGET"
fi

FAILED=0
PASSED=0

for crate in "${WASM_CRATES[@]}"; do
    echo -n "Checking $crate... "
    if cargo check --target "$TARGET" -p "$crate" 2>/dev/null; then
        echo "PASS"
        PASSED=$((PASSED + 1))
    else
        echo "FAIL"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "=== Results: $PASSED passed, $FAILED failed ==="

if [ "$FAILED" -gt 0 ]; then
    echo "ERROR: $FAILED crate(s) failed WASM compilation"
    exit 1
fi

echo "All core crates compile for $TARGET"
