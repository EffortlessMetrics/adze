#!/bin/bash
# Check for bare #[no_mangle] attributes (should be #[unsafe(no_mangle)] in Rust 2024)

set -e

echo "Checking for bare #[no_mangle] attributes..."

# Exclude target directories and look for bare #[no_mangle] that aren't preceded by #[cfg(...)]
# This allows the conditional pattern:
#   #[cfg(adze_unsafe_attrs)]
#   #[unsafe(no_mangle)]
#   #[cfg(not(adze_unsafe_attrs))]
#   #[no_mangle]

# Look for bare #[no_mangle] attributes not in target directories
# We exclude the pattern where it's preceded by #[cfg(not(adze_unsafe_attrs))]
PROBLEMATIC_FILES=$(grep -r -n '^\s*#\s*\[no_mangle\]' --include="*.rs" . --exclude-dir=target 2>/dev/null | while IFS=: read -r file line content; do
    # Check if this line is preceded by the cfg attribute
    if [ "$line" -gt 1 ] 2>/dev/null; then
        prev_line=$((line - 1))
        if ! sed -n "${prev_line}p" "$file" | grep -q '^\s*#\s*\[cfg(not(adze_unsafe_attrs))\]'; then
            echo "$file:$line:$content"
        fi
    else
        echo "$file:$line:$content"
    fi
done)

if [ -n "$PROBLEMATIC_FILES" ]; then
    echo "$PROBLEMATIC_FILES"
    echo "❌ Found bare #[no_mangle] attributes! These must be updated to use conditional compilation:"
    echo "  #[cfg(adze_unsafe_attrs)]"
    echo "  #[unsafe(no_mangle)]"
    echo "  #[cfg(not(adze_unsafe_attrs))]"
    echo "  #[no_mangle]"
    exit 1
else
    echo "✅ No bare #[no_mangle] attributes found."
    exit 0
fi