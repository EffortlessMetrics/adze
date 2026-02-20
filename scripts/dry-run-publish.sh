#!/bin/bash
set -e

echo "=== Package validation for rust-sitter workspace ==="
echo ""

# All crates validated via cargo package (topological order)
CRATES=(
    "rust-sitter-common"
    "rust-sitter-ir"
    "rust-sitter-glr-core"
    "rust-sitter-tablegen"
    "rust-sitter-macro"
    "rust-sitter"
    "rust-sitter-tool"
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
