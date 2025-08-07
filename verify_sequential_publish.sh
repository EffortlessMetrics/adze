#!/bin/bash
# Script to verify that all crates can be published in the correct order.
# This simulates the exact sequence of a real publish to catch dependency errors.
set -e

echo "=== Verifying sequential publish dry-run for all crates ==="
echo ""

# Define the publish order based on dependencies
PUBLISH_ORDER=(
    "common"
    "ir"
    "glr-core"
    "tablegen"
    "macro"
    "tool"
    "runtime"
)

# Track success
ALL_PASSED=true

for crate_dir in "${PUBLISH_ORDER[@]}"; do
    echo "--- Dry-running: $crate_dir ---"
    
    if [ -d "$crate_dir" ]; then
        if (cd "$crate_dir" && cargo publish --dry-run --allow-dirty 2>&1 | tee /tmp/publish_$crate_dir.log | grep -q "warning: aborting upload due to dry run"); then
            echo "✓ $crate_dir passed dry-run"
        else
            echo "✗ $crate_dir FAILED dry-run"
            echo "  Check /tmp/publish_$crate_dir.log for details"
            ALL_PASSED=false
            # Don't exit immediately - continue to see all failures
        fi
    else
        echo "✗ Directory $crate_dir not found!"
        ALL_PASSED=false
    fi
    echo ""
done

if [ "$ALL_PASSED" = true ]; then
    echo "✅ All crates passed the sequential dry-run. The workspace is ready to publish."
    exit 0
else
    echo "❌ Some crates failed the dry-run. Fix the issues above before publishing."
    exit 1
fi