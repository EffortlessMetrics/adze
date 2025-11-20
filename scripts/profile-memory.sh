#!/usr/bin/env bash
# DEPRECATED: Use rust-native xtask command instead
# New command: cargo xtask profile memory --language <lang> --fixture <path>
# This shell script will be removed in a future version.
#
# Memory Profiling Script for rust-sitter
# Part of v0.8.0 Performance Optimization (AC-PERF1)
#
# Usage:
#   ./scripts/profile-memory.sh <language> <fixture-path>
#
# Example:
#   ./scripts/profile-memory.sh python benches/fixtures/python/medium/sample.py
#
# Requirements:
#   - heaptrack (install: apt-get install heaptrack on Ubuntu)
#   - OR valgrind with massif tool
#
# Output:
#   - Memory profile data in docs/analysis/memory-{language}-{size}.txt
#   - Peak memory usage summary

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check arguments
if [ $# -ne 2 ]; then
    echo -e "${RED}Usage: $0 <language> <fixture-path>${NC}"
    echo -e "${YELLOW}Example: $0 python benches/fixtures/python/medium/sample.py${NC}"
    exit 1
fi

LANGUAGE=$1
FIXTURE_PATH=$2

# Validate language
case "$LANGUAGE" in
    python|javascript|rust)
        ;;
    *)
        echo -e "${RED}Error: Unsupported language: $LANGUAGE${NC}"
        echo -e "${YELLOW}Supported: python, javascript, rust${NC}"
        exit 1
        ;;
esac

# Validate fixture path
if [ ! -f "$FIXTURE_PATH" ]; then
    echo -e "${RED}Error: Fixture not found: $FIXTURE_PATH${NC}"
    exit 1
fi

# Determine size from path
if [[ "$FIXTURE_PATH" == *"/small/"* ]]; then
    SIZE="small"
elif [[ "$FIXTURE_PATH" == *"/medium/"* ]]; then
    SIZE="medium"
elif [[ "$FIXTURE_PATH" == *"/large/"* ]]; then
    SIZE="large"
else
    SIZE="unknown"
fi

# Create output directory
mkdir -p docs/analysis

# Output file
OUTPUT_FILE="docs/analysis/memory-${LANGUAGE}-${SIZE}.txt"

echo -e "${BLUE}💾 Memory Profiling: ${LANGUAGE} (${SIZE})${NC}"
echo -e "${BLUE}Fixture: ${FIXTURE_PATH}${NC}"
echo ""

# Check for profiling tools
HAS_HEAPTRACK=false
HAS_VALGRIND=false

if command -v heaptrack &> /dev/null; then
    HAS_HEAPTRACK=true
    echo -e "${GREEN}✅ heaptrack found${NC}"
elif command -v valgrind &> /dev/null; then
    HAS_VALGRIND=true
    echo -e "${GREEN}✅ valgrind found${NC}"
else
    echo -e "${YELLOW}⚠️  No memory profiling tool found${NC}"
    echo -e "${YELLOW}Install heaptrack: apt-get install heaptrack${NC}"
    echo -e "${YELLOW}Or install valgrind: apt-get install valgrind${NC}"
fi

echo ""

# TODO: This is a placeholder until we have actual parsing benchmarks
# For now, create a placeholder report
echo -e "${YELLOW}Note: Memory profiling infrastructure setup. Actual profiling will be added.${NC}"
echo ""

# Create placeholder report
echo -e "${BLUE}Creating memory profile report...${NC}"

cat > "$OUTPUT_FILE" <<EOF
Memory Profile: ${LANGUAGE} (${SIZE})
Fixture: ${FIXTURE_PATH}
Date: $(date)

=== Placeholder Report ===

This is a placeholder. Actual memory profiling will be implemented in Week 3 Day 2.

Expected metrics:
- Peak memory usage (MB)
- Total allocations
- Allocation hotspots (top 5)
- Object lifetimes (short-lived vs long-lived)
- Memory usage per input size ratio

Profiling tools available:
- heaptrack: ${HAS_HEAPTRACK}
- valgrind: ${HAS_VALGRIND}

Next steps:
1. Implement parsing benchmark
2. Run memory profiler on benchmark
3. Analyze allocation patterns
4. Document findings

EOF

if [ "$HAS_HEAPTRACK" = true ]; then
    echo "heaptrack command: heaptrack [benchmark-binary] $FIXTURE_PATH" >> "$OUTPUT_FILE"
elif [ "$HAS_VALGRIND" = true ]; then
    echo "valgrind command: valgrind --tool=massif [benchmark-binary] $FIXTURE_PATH" >> "$OUTPUT_FILE"
fi

echo -e "${GREEN}✅ Report created: ${OUTPUT_FILE}${NC}"

echo ""
echo -e "${BLUE}📊 Summary:${NC}"
echo -e "  Language: ${LANGUAGE}"
echo -e "  Size: ${SIZE}"
echo -e "  Fixture: ${FIXTURE_PATH}"
echo -e "  Report: ${OUTPUT_FILE}"
echo ""

# Placeholder metrics (will be replaced with actual profiling data)
echo -e "${BLUE}📈 Placeholder Metrics:${NC}"
echo -e "  Peak Memory: TBD"
echo -e "  Allocations: TBD"
echo -e "  Top Hotspots: TBD"
echo ""

echo -e "${BLUE}📊 Next steps:${NC}"
echo -e "  1. Review report: ${OUTPUT_FILE}"
echo -e "  2. Identify allocation hotspots"
echo -e "  3. Document findings in docs/analysis/PERFORMANCE_ANALYSIS_V0.7.0.md"
echo ""
echo -e "${GREEN}✅ Memory profiling complete${NC}"
