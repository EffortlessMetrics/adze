#!/bin/bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Test Connectivity Check ==="
echo

# Check for disabled test files
echo "Checking for disabled test files..."
DISABLED_FILES=$(find . -name "*.rs.disabled" -type f 2>/dev/null || true)
if [ -n "$DISABLED_FILES" ]; then
    echo -e "${RED}ERROR: Found disabled test files:${NC}"
    echo "$DISABLED_FILES"
    echo -e "${YELLOW}Action: Re-enable by removing .disabled suffix or use #[ignore] attribute${NC}"
    echo
fi

# Count tests per crate and feature set
echo "Test counts per crate:"
echo

for crate in runtime glr-core tablegen tools/ts-bridge ir common macro tool example; do
    if [ -d "$crate" ]; then
        crate_name=$(basename "$crate")
        if [ "$crate" = "tools/ts-bridge" ]; then
            crate_name="ts-bridge"
        else
            crate_name="rust-sitter-$crate_name"
        fi
        
        echo "  $crate_name:"
        
        # Default features
        count=$(cargo test -p "$crate_name" --no-run 2>&1 | grep -E "Running.*test" | wc -l 2>/dev/null || echo "0")
        count=$(echo "$count" | xargs)
        echo "    default features: $count tests"
        
        # With test-helpers if applicable
        if cargo metadata --format-version 1 2>/dev/null | jq -r ".packages[] | select(.name == \"$crate_name\") | .features | keys[]" | grep -q "test-helpers"; then
            count=$(cargo test -p "$crate_name" --features test-helpers --no-run 2>&1 | grep -E "Running.*test" | wc -l 2>/dev/null || echo "0")
            count=$(echo "$count" | xargs)
            echo "    with test-helpers: $count tests"
        fi
        
        # Check for #[ignore] tests
        ignored=$(rg "#\[ignore\]" "$crate" --glob "*.rs" 2>/dev/null | wc -l || echo "0")
        ignored=$(echo "$ignored" | xargs)
        if [ "$ignored" -gt "0" ]; then
            echo -e "    ${YELLOW}ignored tests: $ignored${NC}"
        fi
    fi
done

echo
echo "Checking for potentially orphaned test modules..."
ORPHANS=$(rg "^mod.*test.*;" --glob "*.rs" -n 2>/dev/null | grep -v "^tests/" | grep -v "#\[cfg\(test\)\]" || true)
if [ -n "$ORPHANS" ]; then
    echo -e "${YELLOW}Warning: Potential orphaned test modules:${NC}"
    echo "$ORPHANS"
fi

echo
echo "=== Summary ==="
if [ -n "$DISABLED_FILES" ]; then
    echo -e "${RED}✗ Found disabled test files that need attention${NC}"
    exit 1
else
    echo -e "${GREEN}✓ No disabled test files found${NC}"
fi