#!/usr/bin/env bash
set -euo pipefail

echo "=== WASM Compatibility Check ==="

# Pure data crates that MUST compile for WASM
WASM_CRATES=(
    "adze-linecol-core"
    "adze-stack-pool-core"
    "adze-parsetable-metadata"
    "adze-feature-policy-core"
    "adze-bdd-grid-core"
    "adze-governance-metadata"
    "adze-bdd-grid-contract"
    "adze-bdd-contract"
)

FAILED=0

for crate in "${WASM_CRATES[@]}"; do
    echo -n "Checking $crate... "
    if cargo check --target wasm32-unknown-unknown -p "$crate" 2>/dev/null; then
        echo "OK"
    else
        echo "FAIL"
        FAILED=$((FAILED + 1))
    fi
done

echo "=== Done ==="

if [ "$FAILED" -gt 0 ]; then
    echo "ERROR: $FAILED crate(s) failed WASM compilation"
    exit 1
fi
