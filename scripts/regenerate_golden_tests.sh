#!/bin/bash
# Script to regenerate golden test outputs for all test grammars
# This is useful when parser output format changes or when adding new tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔄 Regenerating golden test outputs for rust-sitter"
echo "================================================="

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to run tests and capture new outputs
regenerate_grammar() {
    local grammar_name=$1
    local grammar_path=$2
    
    echo -e "\n${YELLOW}Processing $grammar_name grammar...${NC}"
    
    # Build the grammar first
    cd "$grammar_path"
    if cargo build 2>/dev/null; then
        echo -e "${GREEN}✓ Built $grammar_name successfully${NC}"
    else
        echo -e "${RED}✗ Failed to build $grammar_name${NC}"
        return 1
    fi
    
    # Run tests with REGENERATE_GOLDEN environment variable
    # This tells the test framework to write new golden files instead of comparing
    if REGENERATE_GOLDEN=1 cargo test 2>/dev/null; then
        echo -e "${GREEN}✓ Regenerated golden files for $grammar_name${NC}"
    else
        echo -e "${YELLOW}⚠ No golden tests found for $grammar_name${NC}"
    fi
}

# Process example grammars
echo -e "\n${YELLOW}=== Processing example grammars ===${NC}"
for dir in "$ROOT_DIR"/example/*/; do
    if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
        grammar_name=$(basename "$dir")
        regenerate_grammar "$grammar_name" "$dir"
    fi
done

# Process language grammars
echo -e "\n${YELLOW}=== Processing language grammars ===${NC}"
for lang in python javascript go; do
    grammar_path="$ROOT_DIR/crates/language-$lang"
    if [ -d "$grammar_path" ]; then
        regenerate_grammar "$lang" "$grammar_path"
    fi
done

# Run main workspace golden tests
echo -e "\n${YELLOW}=== Processing main workspace golden tests ===${NC}"
cd "$ROOT_DIR"
if REGENERATE_GOLDEN=1 cargo test golden_tests 2>/dev/null; then
    echo -e "${GREEN}✓ Regenerated main workspace golden files${NC}"
else
    echo -e "${YELLOW}⚠ No golden tests found in main workspace${NC}"
fi

# Summary
echo -e "\n${GREEN}=== Regeneration Complete ===${NC}"
echo "Golden test files have been regenerated."
echo "Review the changes with: git diff"
echo "Commit the updated golden files if the changes are expected."

# Check if any golden files were actually modified
if git -C "$ROOT_DIR" diff --quiet; then
    echo -e "\n${YELLOW}Note: No golden files were modified.${NC}"
else
    echo -e "\n${YELLOW}Golden files were modified. Review changes before committing.${NC}"
    echo "Modified files:"
    git -C "$ROOT_DIR" diff --name-only | grep -E "\.(golden|expected|txt)$" || true
fi