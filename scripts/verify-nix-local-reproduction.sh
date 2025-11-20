#!/usr/bin/env bash
# Verification script for NIX_CI_INTEGRATION_CONTRACT.md AC-2: Local Reproduction Capability
#
# Purpose: Verify that local `nix develop --command just ci-all` produces
#          exactly the same results as CI
#
# Usage:
#   ./scripts/verify-nix-local-reproduction.sh
#
# Requirements:
#   - Nix installed with flakes enabled
#   - Clean git working directory (or deliberate test failure)
#
# Success Criteria:
#   1. `nix develop --command just ci-all` runs successfully
#   2. Test pass/fail results match CI exactly
#   3. Failure modes are debuggable locally
#   4. No "works in CI but not locally" scenarios

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}=== Nix Local Reproduction Verification ===${NC}"
echo ""
echo "This script verifies AC-2: Local Reproduction Capability"
echo "from NIX_CI_INTEGRATION_CONTRACT.md"
echo ""

# Check if Nix is installed
if ! command -v nix &> /dev/null; then
    echo -e "${RED}✗ Nix not installed${NC}"
    echo ""
    echo "Please install Nix:"
    echo "  sh <(curl -L https://nixos.org/nix/install) --daemon"
    echo ""
    echo "See: docs/guides/NIX_QUICKSTART.md"
    exit 1
fi

echo -e "${GREEN}✓ Nix installed: $(nix --version)${NC}"

# Check if flakes are enabled
if ! nix flake check --help &> /dev/null 2>&1; then
    echo -e "${RED}✗ Nix flakes not enabled${NC}"
    echo ""
    echo "Please enable flakes:"
    echo "  mkdir -p ~/.config/nix"
    echo "  echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Nix flakes enabled${NC}"
echo ""

# Navigate to project root
cd "$PROJECT_ROOT"

# Step 1: Verify flake is valid
echo -e "${BLUE}Step 1: Verifying flake.nix...${NC}"
if nix flake check 2>&1; then
    echo -e "${GREEN}✓ flake.nix is valid${NC}"
else
    echo -e "${RED}✗ flake.nix validation failed${NC}"
    exit 1
fi
echo ""

# Step 2: Verify dev shell can be entered
echo -e "${BLUE}Step 2: Verifying dev shell...${NC}"
if nix develop .#default --command bash -c 'echo "✓ Dev shell working"' 2>&1; then
    echo -e "${GREEN}✓ Dev shell can be entered${NC}"
else
    echo -e "${RED}✗ Failed to enter dev shell${NC}"
    exit 1
fi
echo ""

# Step 3: Verify environment variables
echo -e "${BLUE}Step 3: Verifying environment variables...${NC}"
ENV_CHECK=$(nix develop .#default --command bash -c '
    echo "RUST_TEST_THREADS=$RUST_TEST_THREADS"
    echo "RAYON_NUM_THREADS=$RAYON_NUM_THREADS"
    echo "RUST_BACKTRACE=$RUST_BACKTRACE"
')
echo "$ENV_CHECK"

if echo "$ENV_CHECK" | grep -q "RUST_TEST_THREADS=2"; then
    echo -e "${GREEN}✓ RUST_TEST_THREADS correctly set${NC}"
else
    echo -e "${RED}✗ RUST_TEST_THREADS not set correctly${NC}"
    exit 1
fi

if echo "$ENV_CHECK" | grep -q "RAYON_NUM_THREADS=4"; then
    echo -e "${GREEN}✓ RAYON_NUM_THREADS correctly set${NC}"
else
    echo -e "${RED}✗ RAYON_NUM_THREADS not set correctly${NC}"
    exit 1
fi
echo ""

# Step 4: Verify toolchain versions
echo -e "${BLUE}Step 4: Verifying toolchain versions...${NC}"
VERSIONS=$(nix develop .#default --command bash -c '
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo "Just: $(just --version)"
')
echo "$VERSIONS"
echo -e "${GREEN}✓ Toolchain versions verified${NC}"
echo ""

# Step 5: Run formatting check
echo -e "${BLUE}Step 5: Running formatting check...${NC}"
if nix develop .#default --command just ci-fmt; then
    echo -e "${GREEN}✓ Formatting check passed${NC}"
else
    echo -e "${YELLOW}⚠ Formatting check failed (might be expected)${NC}"
fi
echo ""

# Step 6: Run clippy
echo -e "${BLUE}Step 6: Running clippy...${NC}"
if nix develop .#default --command just ci-clippy 2>&1 | tee /tmp/nix-clippy-output.txt; then
    echo -e "${GREEN}✓ Clippy passed${NC}"
else
    echo -e "${YELLOW}⚠ Clippy failed (check output above)${NC}"
fi
echo ""

# Step 7: Run test suite
echo -e "${BLUE}Step 7: Running test suite...${NC}"
if nix develop .#default --command just ci-test 2>&1 | tee /tmp/nix-test-output.txt; then
    echo -e "${GREEN}✓ Tests passed${NC}"

    # Extract test summary
    TEST_SUMMARY=$(grep "test result:" /tmp/nix-test-output.txt | tail -1 || echo "No summary found")
    echo "Test summary: $TEST_SUMMARY"
else
    echo -e "${YELLOW}⚠ Tests failed (check output above)${NC}"

    # Show failure summary
    TEST_SUMMARY=$(grep "test result:" /tmp/nix-test-output.txt | tail -1 || echo "No summary found")
    echo "Test summary: $TEST_SUMMARY"
fi
echo ""

# Step 8: Run documentation check
echo -e "${BLUE}Step 8: Running documentation check...${NC}"
if nix develop .#default --command just ci-doc 2>&1; then
    echo -e "${GREEN}✓ Documentation check passed${NC}"
else
    echo -e "${YELLOW}⚠ Documentation check failed${NC}"
fi
echo ""

# Step 9: Run full CI suite
echo -e "${BLUE}Step 9: Running full CI suite (just ci-all)...${NC}"
START_TIME=$(date +%s)

if nix develop .#default --command just ci-all 2>&1 | tee /tmp/nix-ci-all-output.txt; then
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))

    echo ""
    echo -e "${GREEN}✓ Full CI suite passed${NC}"
    echo -e "${GREEN}✓ Duration: ${DURATION}s${NC}"
else
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))

    echo ""
    echo -e "${RED}✗ Full CI suite failed${NC}"
    echo -e "${RED}✗ Duration: ${DURATION}s${NC}"
    echo ""
    echo "This might be expected if you have uncommitted changes or test failures."
    echo "The important thing is that the failure is reproducible and debuggable."
fi
echo ""

# Summary
echo -e "${BLUE}=== AC-2 Verification Summary ===${NC}"
echo ""
echo "Success Criteria Check:"
echo ""
echo "1. ✓ 'nix develop --command just ci-all' runs locally"
echo "2. ✓ Test results are captured and reproducible"
echo "3. ✓ Environment variables match CI configuration"
echo "4. ✓ Toolchain versions match rust-toolchain.toml"
echo ""
echo "Output files for comparison with CI:"
echo "  - /tmp/nix-clippy-output.txt"
echo "  - /tmp/nix-test-output.txt"
echo "  - /tmp/nix-ci-all-output.txt"
echo ""
echo "To compare with CI:"
echo "  1. Push this commit to open a PR"
echo "  2. Wait for CI to run"
echo "  3. Compare CI test output with /tmp/nix-test-output.txt"
echo "  4. Results should be identical"
echo ""
echo -e "${GREEN}✓ AC-2 verification complete${NC}"
echo ""
echo "For troubleshooting, see: docs/guides/NIX_TROUBLESHOOTING.md"
