#!/usr/bin/env bash
# Security verification script for rust-sitter
# Part of Policy-as-Code (AC-P3: Security Policies, AC-P4: Quality Verification Scripts)
#
# Usage:
#   ./scripts/check-security.sh
#
# Performance target: <10 seconds (typical)
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

echo -e "${BLUE}🔒 Checking Security...${NC}"
echo ""

# ============================================================================
# Cargo Audit (Vulnerability Scanning)
# ============================================================================
echo -n "🔍 Vulnerability Scan (cargo audit): "

# Check if cargo-audit is installed
if ! command -v cargo-audit >/dev/null 2>&1; then
    echo -e "${YELLOW}SKIP (cargo-audit not installed)${NC}"
    echo -e "${YELLOW}  Install: cargo install cargo-audit${NC}"
else
    if cargo audit >/tmp/audit.log 2>&1; then
        echo -e "${GREEN}PASS${NC}"
    else
        echo -e "${RED}FAIL${NC}"
        echo ""
        cat /tmp/audit.log
        echo ""
        echo -e "${YELLOW}  Action: Update vulnerable dependencies${NC}"
        echo -e "${YELLOW}  Example: cargo update -p <crate-name>${NC}"
        FAILED=1
    fi
fi

# ============================================================================
# Cargo Deny (License Compliance & Security)
# ============================================================================
echo -n "📜 License Compliance (cargo deny): "

# Check if cargo-deny is installed
if ! command -v cargo-deny >/dev/null 2>&1; then
    echo -e "${YELLOW}SKIP (cargo-deny not installed)${NC}"
    echo -e "${YELLOW}  Install: cargo install cargo-deny${NC}"
else
    if cargo deny check >/tmp/deny.log 2>&1; then
        echo -e "${GREEN}PASS${NC}"
    else
        echo -e "${RED}FAIL${NC}"
        echo ""
        cat /tmp/deny.log
        echo ""
        echo -e "${YELLOW}  Action: Fix license or dependency issues${NC}"
        FAILED=1
    fi
fi

# ============================================================================
# Secret Detection (Local)
# ============================================================================
echo -n "🔐 Secret Detection (staged changes): "

# Check if there are staged changes
if git diff --cached --quiet; then
    echo -e "${YELLOW}SKIP (no staged changes)${NC}"
else
    # Simple pattern matching for common secrets
    if git diff --cached | grep -qiE "(api[_-]?key|password|secret|token|private[_-]?key|sk_live|pk_live)"; then
        echo -e "${YELLOW}WARNING${NC}"
        echo -e "${YELLOW}  Possible secret detected in staged changes${NC}"
        echo -e "${YELLOW}  Review carefully:${NC}"
        git diff --cached | grep -iE "(api[_-]?key|password|secret|token|private[_-]?key|sk_live|pk_live)" | head -5
        echo -e "${YELLOW}  If false positive: Proceed with caution${NC}"
        echo -e "${YELLOW}  If real secret: Remove and rotate credential${NC}"
        # Warning only, don't fail
    else
        echo -e "${GREEN}PASS${NC}"
    fi
fi

# ============================================================================
# Dependency Count & Size Check
# ============================================================================
echo -n "📦 Dependency Health: "

# Count total dependencies
DEP_COUNT=$(cargo tree --workspace --depth 0 2>/dev/null | wc -l || echo "?")
echo -e "${GREEN}$DEP_COUNT direct dependencies${NC}"

# ============================================================================
# Summary
# ============================================================================
echo ""
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All security checks passed!${NC}"
    echo ""
    echo "Note: Install cargo-audit and cargo-deny for complete scanning:"
    echo "  cargo install cargo-audit cargo-deny"
    exit 0
else
    echo -e "${RED}❌ Security checks failed.${NC}"
    echo ""
    echo "Fix the issues above and run again."
    exit 1
fi
