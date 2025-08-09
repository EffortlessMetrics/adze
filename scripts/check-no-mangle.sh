#!/bin/bash
# Check for bare #[no_mangle] attributes (should be #[unsafe(no_mangle)] in Rust 2024)

set -e

echo "Checking for bare #[no_mangle] attributes..."

# Search for bare #[no_mangle] in all Rust files
if grep -r -n '^\s*#\s*\[no_mangle\]' --include="*.rs" .; then
    echo "❌ Found bare #[no_mangle] attributes! These must be updated to #[unsafe(no_mangle)] for Rust 2024."
    echo "Please update all occurrences to use the new syntax:"
    echo "  #[unsafe(no_mangle)]"
    exit 1
else
    echo "✅ No bare #[no_mangle] attributes found."
    exit 0
fi