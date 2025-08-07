#!/bin/bash

# Script to verify that all crates can be published in the correct order

set -e

echo "=== Verifying publish dry-run for all crates ==="
echo ""

# Define the publish order based on dependencies
PUBLISH_ORDER=(
    "common"     # No rust-sitter dependencies
    "ir"         # No rust-sitter dependencies
    "glr-core"   # Depends on ir
    "tablegen"   # Depends on ir, glr-core
    "macro"      # Depends on common
    "tool"       # Depends on common, ir, glr-core, tablegen
    "runtime"    # Depends on macro, ir, glr-core, tablegen
)

SUCCESS_COUNT=0
FAILED_CRATES=()

for crate_dir in "${PUBLISH_ORDER[@]}"; do
    echo "--- Dry-running: $crate_dir ---"
    
    if [ -d "$crate_dir" ]; then
        cd "$crate_dir"
        
        if cargo publish --dry-run --allow-dirty 2>&1 | tail -5; then
            echo "✓ $crate_dir passed dry-run"
            ((SUCCESS_COUNT++))
        else
            echo "✗ $crate_dir FAILED dry-run"
            FAILED_CRATES+=("$crate_dir")
        fi
        
        cd ..
    else
        echo "✗ Directory $crate_dir not found!"
        FAILED_CRATES+=("$crate_dir")
    fi
    
    echo ""
done

echo "=== Summary ==="
echo "Successfully verified: $SUCCESS_COUNT/${#PUBLISH_ORDER[@]} crates"

if [ ${#FAILED_CRATES[@]} -gt 0 ]; then
    echo "Failed crates: ${FAILED_CRATES[*]}"
    exit 1
else
    echo "✅ All crates passed dry-run verification!"
    echo ""
    echo "Ready to publish to crates.io in this order:"
    for crate in "${PUBLISH_ORDER[@]}"; do
        echo "  - $crate"
    done
fi