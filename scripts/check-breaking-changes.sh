#!/bin/bash
# Script to detect breaking changes in the public API
# Run this before releases to ensure semver compliance

set -e

echo "🔍 Checking for breaking changes in public API..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BASELINE_DIR=""

cleanup_baseline() {
    if [[ -n "$BASELINE_DIR" && -d "$BASELINE_DIR" ]]; then
        git worktree remove --force "$BASELINE_DIR" >/dev/null 2>&1 || true
    fi
}

prepare_baseline() {
    local baseline_ref=$1

    if [[ -n "$BASELINE_DIR" && -d "$BASELINE_DIR" ]]; then
        return 0
    fi

    BASELINE_DIR="$(mktemp -d)"
    trap cleanup_baseline EXIT

    git worktree add "$BASELINE_DIR" "$baseline_ref" >/dev/null

    sed -i 's/^name = "rust-sitter"$/name = "adze"/' "$BASELINE_DIR/runtime/Cargo.toml"
    sed -i 's/^name = "rust-sitter-macro"$/name = "adze-macro"/' "$BASELINE_DIR/macro/Cargo.toml"
    sed -i 's/^name = "rust-sitter-tool"$/name = "adze-tool"/' "$BASELINE_DIR/tool/Cargo.toml"
    sed -i 's/^name = "rust-sitter-common"$/name = "adze-common"/' "$BASELINE_DIR/common/Cargo.toml"
    sed -i 's/^name = "rust-sitter-ir"$/name = "adze-ir"/' "$BASELINE_DIR/ir/Cargo.toml"
    sed -i 's/^name = "rust-sitter-glr-core"$/name = "adze-glr-core"/' "$BASELINE_DIR/glr-core/Cargo.toml"
    sed -i 's/^name = "rust-sitter-tablegen"$/name = "adze-tablegen"/' "$BASELINE_DIR/tablegen/Cargo.toml"

    perl -0pi -e 's/rust-sitter-macro\/pure-rust/adze-macro\/pure-rust/g; s/rust-sitter-macro = \{/adze-macro = {/g; s/rust-sitter-ir = \{/adze-ir = {/g; s/rust-sitter-glr-core = \{/adze-glr-core = {/g; s/rust-sitter-tablegen = \{/adze-tablegen = {/g' "$BASELINE_DIR/runtime/Cargo.toml"
    perl -0pi -e 's/rust-sitter-common = \{/adze-common = {/g' "$BASELINE_DIR/macro/Cargo.toml"
    perl -0pi -e 's/rust-sitter-ir\/optimize/adze-ir\/optimize/g; s/rust-sitter-common = \{/adze-common = {/g; s/rust-sitter-ir = \{/adze-ir = {/g; s/rust-sitter-glr-core = \{/adze-glr-core = {/g; s/rust-sitter-tablegen = \{/adze-tablegen = {/g; s/name = "rust-sitter-gen"/name = "adze-gen"/g' "$BASELINE_DIR/tool/Cargo.toml"
    perl -0pi -e 's/rust-sitter-glr-core = \{/adze-glr-core = {/g' "$BASELINE_DIR/ir/Cargo.toml"
    perl -0pi -e 's/rust-sitter-ir = \{/adze-ir = {/g' "$BASELINE_DIR/glr-core/Cargo.toml"
    perl -0pi -e 's/rust-sitter-ir = \{/adze-ir = {/g; s/rust-sitter-glr-core = \{/adze-glr-core = {/g' "$BASELINE_DIR/tablegen/Cargo.toml"
}

# Function to check a crate for breaking changes
check_crate() {
    local crate=$1
    local manifest_path=$2
    local baseline_subdir=$3
    local baseline=${4:-HEAD~1}
    
    echo -e "\n📦 Checking $crate against $baseline..."

    prepare_baseline "$baseline"
    
    if cargo semver-checks check-release \
        --manifest-path "$manifest_path" \
        --baseline-root "$BASELINE_DIR/$baseline_subdir" 2>/dev/null; then
        echo -e "${GREEN}✅ No breaking changes detected in $crate${NC}"
        return 0
    else
        echo -e "${RED}❌ Breaking changes detected in $crate${NC}"
        return 1
    fi
}

# Function to check if cargo-semver-checks is installed
check_dependencies() {
    if ! command -v cargo-semver-checks &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-semver-checks not found. Installing...${NC}"
        cargo install cargo-semver-checks --locked
    fi
}

# Function to run API contract tests
run_contract_tests() {
    echo -e "\n🧪 Running API contract tests..."
    
    if cargo test -p adze --test api_contract 2>/dev/null; then
        echo -e "${GREEN}✅ API contract tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ API contract tests failed${NC}"
        return 1
    fi
}

# Function to check documentation coverage
check_doc_coverage() {
    echo -e "\n📚 Checking documentation coverage..."
    
    if cargo doc -p adze --no-deps \
        --features strict_docs 2>&1 | grep -q "warning"; then
        echo -e "${YELLOW}⚠️  Documentation warnings found${NC}"
        return 1
    else
        echo -e "${GREEN}✅ Documentation complete${NC}"
        return 0
    fi
}

# Function to generate API report
generate_api_report() {
    local output_file=${1:-"api-report.md"}
    
    echo -e "\n📊 Generating API report to $output_file..."
    
    cat > "$output_file" << EOF
# API Surface Report

Generated: $(date)

## Public Types

\`\`\`
$(cargo rustdoc -p adze -- -Z unstable-options --output-format json 2>/dev/null | \
    jq -r '.index | to_entries[] | select(.value.visibility == "public") | .value.name' | \
    sort | head -20 || echo "Unable to generate type list")
\`\`\`

## Breaking Change Check Results

EOF
    
    echo "### adze" >> "$output_file"
    if check_crate "adze" "runtime/Cargo.toml" "runtime" HEAD~1 &>/dev/null; then
        echo "✅ No breaking changes" >> "$output_file"
    else
        echo "⚠️  Potential breaking changes detected" >> "$output_file"
    fi
    echo "" >> "$output_file"

    echo "### adze-macro" >> "$output_file"
    if check_crate "adze-macro" "macro/Cargo.toml" "macro" HEAD~1 &>/dev/null; then
        echo "✅ No breaking changes" >> "$output_file"
    else
        echo "⚠️  Potential breaking changes detected" >> "$output_file"
    fi
    echo "" >> "$output_file"

    echo "### adze-tool" >> "$output_file"
    if check_crate "adze-tool" "tool/Cargo.toml" "tool" HEAD~1 &>/dev/null; then
        echo "✅ No breaking changes" >> "$output_file"
    else
        echo "⚠️  Potential breaking changes detected" >> "$output_file"
    fi
    echo "" >> "$output_file"
    
    echo -e "${GREEN}✅ API report generated: $output_file${NC}"
}

# Main execution
main() {
    local baseline=${1:-HEAD~1}
    local failed=0
    
    echo "================================================"
    echo "     Breaking Change Detection Tool"
    echo "================================================"
    
    # Check dependencies
    check_dependencies
    
    # Run checks
    echo -e "\n${YELLOW}Checking against baseline: $baseline${NC}"
    
    # Check each crate
    if ! check_crate "adze" "runtime/Cargo.toml" "runtime" "$baseline"; then
        failed=$((failed + 1))
    fi
    if ! check_crate "adze-macro" "macro/Cargo.toml" "macro" "$baseline"; then
        failed=$((failed + 1))
    fi
    if ! check_crate "adze-tool" "tool/Cargo.toml" "tool" "$baseline"; then
        failed=$((failed + 1))
    fi
    
    # Run contract tests
    if ! run_contract_tests; then
        failed=$((failed + 1))
    fi
    
    # Check documentation
    if ! check_doc_coverage; then
        echo -e "${YELLOW}Note: Documentation issues don't block, but should be fixed${NC}"
    fi
    
    # Generate report
    generate_api_report "api-report-$(date +%Y%m%d).md"
    
    # Summary
    echo -e "\n================================================"
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✅ All checks passed! Safe to release.${NC}"
        echo "No breaking changes detected."
    else
        echo -e "${RED}❌ $failed check(s) failed!${NC}"
        echo ""
        echo "Breaking changes detected. You should either:"
        echo "1. Revert the breaking changes, or"
        echo "2. Bump the major version number"
        echo ""
        echo "Run with a different baseline:"
        echo "  $0 <git-ref>"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [baseline-ref]"
        echo ""
        echo "Check for breaking changes in the public API"
        echo ""
        echo "Arguments:"
        echo "  baseline-ref   Git ref to compare against (default: HEAD~1)"
        echo ""
        echo "Examples:"
        echo "  $0              # Check against previous commit"
        echo "  $0 main         # Check against main branch"
        echo "  $0 v0.1.0       # Check against a tag"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
