#!/usr/bin/env bash
# Pre-push validation script for rust-sitter
# Part of Policy-as-Code (AC-P4: Quality Verification Scripts)
#
# Usage:
#   ./scripts/pre-push.sh
#
# Or install as git hook:
#   ln -sf ../../scripts/pre-push.sh .git/hooks/pre-push
#
# Performance target: <60 seconds (runs quality + security)
#
# Exit codes:
#   0 - All checks passed
#   1 - One or more checks failed
#   2 - User cancelled

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Pre-Push Validation...${NC}"
echo ""

# ============================================================================
# Check Branch Name (Warn if pushing to main/master)
# ============================================================================
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

if [[ "$BRANCH" == "main" ]] || [[ "$BRANCH" == "master" ]]; then
    echo -e "${YELLOW}⚠️  WARNING: Pushing directly to $BRANCH${NC}"
    echo -e "${YELLOW}Consider using a feature branch instead.${NC}"
    echo ""
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Push cancelled."
        exit 2
    fi
    echo ""
fi

# ============================================================================
# Run Quality Checks
# ============================================================================
echo -e "${BLUE}Step 1/2: Quality Checks${NC}"
echo ""

if ./scripts/check-quality.sh; then
    echo ""
    echo -e "${GREEN}✅ Quality checks passed${NC}"
else
    echo ""
    echo -e "${RED}❌ Quality checks failed. Fix before pushing.${NC}"
    echo ""
    echo "To skip quality checks (not recommended):"
    echo "  git push --no-verify"
    exit 1
fi

# ============================================================================
# Run Security Checks
# ============================================================================
echo ""
echo -e "${BLUE}Step 2/2: Security Checks${NC}"
echo ""

if ./scripts/check-security.sh; then
    echo ""
    echo -e "${GREEN}✅ Security checks passed${NC}"
else
    echo ""
    echo -e "${RED}❌ Security checks failed. Fix before pushing.${NC}"
    echo ""
    echo "To skip security checks (not recommended):"
    echo "  git push --no-verify"
    exit 1
fi

# ============================================================================
# Success
# ============================================================================
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Pre-push validation passed!${NC}"
echo -e "${GREEN}Safe to push to $BRANCH${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""

exit 0
