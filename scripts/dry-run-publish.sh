#!/bin/bash
set -e

echo "=== Package validation for adze workspace ==="
echo ""

# All crates validated via cargo package (topological order)
CRATES=(
    "adze-common"
    "adze-ir"
    "adze-glr-core"
    "adze-tablegen"
    "adze-macro"
    "adze"
    "adze-tool"
)

for crate in "${CRATES[@]}"; do
    echo ">>> Validating $crate..."
    if cargo package -p "$crate" --no-verify 2>&1; then
        echo "  $crate OK"
    else
        echo "  $crate FAILED"
        exit 1
    fi
    echo ""
done

echo "=== All crate manifests valid ==="
