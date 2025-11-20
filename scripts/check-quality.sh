#!/usr/bin/env bash
# Quality verification script for rust-sitter
# Part of Policy-as-Code (AC-P4: Quality Verification Scripts)
#
# Usage:
#   ./scripts/check-quality.sh
#
# Performance target: <30 seconds (typical)
#
# Exit codes:
#   0 - All checks passed
#   1 - One or more checks failed

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track failures
FAILED=0

echo -e "${BLUE}🔍 Checking Quality...${NC}"
echo ""

# ============================================================================
# Formatting Check
# ============================================================================
echo -n "✅ Formatting (cargo fmt): "
if cargo fmt --all -- --check >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo -e "${YELLOW}  Run: cargo fmt${NC}"
    FAILED=1
fi

# ============================================================================
# Clippy Check (Zero Warnings)
# ============================================================================
echo -n "✅ Clippy (cargo clippy): "
CLIPPY_OUTPUT=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1)
if echo "$CLIPPY_OUTPUT" | grep -q "0 warnings emitted"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    # Show first 5 warnings
    echo "$CLIPPY_OUTPUT" | grep "warning:" | head -5
    echo -e "${YELLOW}  Run: cargo clippy --fix${NC}"
    FAILED=1
fi

# ============================================================================
# Tests (100% Pass Rate)
# ============================================================================
echo -n "✅ Tests (cargo test): "
if cargo test --workspace -- --test-threads=2 >/tmp/test-quality.log 2>&1; then
    # Extract test count
    TEST_COUNT=$(grep -oP '\d+(?= passed)' /tmp/test-quality.log | head -1 || echo "?")
    echo -e "${GREEN}PASS ($TEST_COUNT tests)${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo -e "${YELLOW}  Last 20 lines of output:${NC}"
    tail -20 /tmp/test-quality.log
    FAILED=1
fi

# ============================================================================
# Documentation (Zero Warnings)
# ============================================================================
echo -n "✅ Documentation (cargo doc): "
if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo -e "${YELLOW}  Fix documentation warnings${NC}"
    FAILED=1
fi

# ============================================================================
# Test Connectivity (No .rs.disabled)
# ============================================================================
echo -n "✅ Test Connectivity: "
if find . -name "*.rs.disabled" | grep -q .; then
    echo -e "${RED}FAIL${NC}"
    echo -e "${YELLOW}  Found .rs.disabled files:${NC}"
    find . -name "*.rs.disabled"
    echo -e "${YELLOW}  Use #[ignore] attribute instead${NC}"
    FAILED=1
else
    echo -e "${GREEN}PASS${NC}"
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All quality checks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Quality checks failed.${NC}"
    echo ""
    echo "Fix the issues above and run again."
    exit 1
fi
