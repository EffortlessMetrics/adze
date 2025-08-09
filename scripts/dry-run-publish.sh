#!/bin/bash
set -e

echo "=== Dry-run publish for rust-sitter v0.6.0 ==="
echo ""

# Publish order based on dependencies
CRATES=(
    "rust-sitter-common"
    "rust-sitter-ir" 
    "rust-sitter-glr-core"
    "rust-sitter-tablegen"
    "rust-sitter"
    "rust-sitter-macro"
    "rust-sitter-tool"
)

for crate in "${CRATES[@]}"; do
    echo ">>> Dry-run publishing $crate..."
    # Find the crate directory
    case $crate in
        "rust-sitter")
            dir="runtime"
            ;;
        "rust-sitter-macro")
            dir="macro"
            ;;
        "rust-sitter-tool")
            dir="tool"
            ;;
        "rust-sitter-common")
            dir="common"
            ;;
        "rust-sitter-ir")
            dir="ir"
            ;;
        "rust-sitter-glr-core")
            dir="glr-core"
            ;;
        "rust-sitter-tablegen")
            dir="tablegen"
            ;;
    esac
    
    (cd "/home/steven/code/rust-sitter/$dir" && cargo publish --dry-run)
    echo "✓ $crate ready for publish"
    echo ""
done

echo "=== All crates ready for v0.6.0 release! ==="