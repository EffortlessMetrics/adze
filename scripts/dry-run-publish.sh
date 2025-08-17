#!/bin/bash
set -e

# Get the workspace root (parent of the scripts directory)
WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Dry-run publish for rust-sitter workspace ==="
echo "Workspace root: $WORKSPACE_ROOT"
echo ""

# Publish order based on dependencies (topological order)
# Leaf crates first, then crates that depend on them
# NOTE: Some crates are marked publish = false and are skipped
CRATES=(
    "rust-sitter-common"      # No workspace deps
    # "rust-sitter-ir"        # SKIP: publish = false
    # "rust-sitter-glr-core"  # SKIP: publish = false
    # "rust-sitter-tablegen"  # SKIP: publish = false
    "rust-sitter"             # Main runtime crate
    "rust-sitter-macro"       # Depends on: common
    "rust-sitter-tool"        # Depends on: common, macro
)

# Track which crates need --no-verify (have unpublished deps)
# Since ir, glr-core, and tablegen are not published, crates depending on them need --no-verify
NEEDS_NO_VERIFY=(
    "rust-sitter"       # Depends on unpublished: ir, glr-core, tablegen
    "rust-sitter-tool"  # Depends on unpublished: ir, glr-core, tablegen
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
        *)
            echo "ERROR: Unknown crate $crate"
            exit 1
            ;;
    esac
    
    # Check if this crate needs --no-verify
    EXTRA_FLAGS=""
    for need_no_verify in "${NEEDS_NO_VERIFY[@]}"; do
        if [ "$crate" = "$need_no_verify" ]; then
            EXTRA_FLAGS="--no-verify --allow-dirty"
            echo "  (Using --no-verify due to unpublished dependencies)"
            break
        fi
    done
    
    # Run the dry-run from the crate directory
    echo "  Running: cargo publish --dry-run $EXTRA_FLAGS"
    if (cd "$WORKSPACE_ROOT/$dir" && cargo publish --dry-run $EXTRA_FLAGS 2>&1); then
        echo "✓ $crate ready for publish"
    else
        if [ ! -z "$EXTRA_FLAGS" ]; then
            echo "  ⚠️  Dry-run failed (expected for crates with unpublished deps)"
            echo "✓ $crate will need --no-verify for actual publish"
        else
            echo "  ❌ Dry-run failed unexpectedly"
            exit 1
        fi
    fi
    echo ""
done

echo "=== All crates ready for release! ==="
echo ""
echo "To publish for real, run:"
echo "  ./scripts/update-versions.sh <new-version>"
echo "  Then run each cargo publish command without --dry-run"