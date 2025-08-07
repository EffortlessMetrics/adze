#!/bin/bash
# Script to automatically fix clippy warnings where possible
set -e

echo "=== Fixing Clippy Warnings ==="
echo ""

# First, try auto-fix for simple issues
echo "--- Attempting auto-fix with cargo clippy --fix ---"
cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged 2>/dev/null || true

# Now run clippy again to see what's left
echo ""
echo "--- Remaining warnings after auto-fix ---"
cargo clippy --workspace --all-targets 2>&1 | grep "^warning:" | head -20 || echo "No warnings found!"

echo ""
echo "✅ Clippy auto-fix complete. Manual fixes may still be needed."